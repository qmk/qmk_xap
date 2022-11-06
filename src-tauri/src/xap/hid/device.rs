use std::{
    fmt::{Debug, Display},
    io::{Cursor, Read},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use binrw::{BinRead, BinWriterExt};
use crossbeam_channel::{unbounded, Receiver, Sender};
use flate2::read::GzDecoder;
use hidapi::{DeviceInfo, HidDevice};
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use ts_rs::TS;
use uuid::Uuid;

use crate::{xap::*, XAPEvent};

const XAP_REPORT_SIZE: usize = 64;

pub struct XAPDevice {
    id: Uuid,
    hid_info: DeviceInfo,
    xap_info: Option<XAPDeviceInfo>,
    keymap: Vec<Vec<Vec<KeyPositionConfig>>>,
    tx_device: HidDevice,
    rx_thread: JoinHandle<()>,
    rx_channel: Receiver<ResponseRaw>,
}

impl Debug for XAPDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for XAPDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VID: {:04x}, PID: {:04x}, Serial: {}, Product name: {}, Manufacturer: {}",
            self.hid_info.vendor_id(),
            self.hid_info.product_id(),
            match self.hid_info.serial_number() {
                Some(s) => s,
                _ => "<COULD NOT FETCH>",
            },
            match self.hid_info.product_string() {
                Some(s) => s,
                _ => "<COULD NOT FETCH>",
            },
            match self.hid_info.manufacturer_string() {
                Some(s) => s,
                _ => "<COULD_NOT_FETCH>",
            }
        )
    }
}

impl XAPDevice {
    pub(crate) fn new(
        info: DeviceInfo,
        event_channel: Sender<XAPEvent>,
        rx: HidDevice,
        tx: HidDevice,
    ) -> XAPResult<Self> {
        let (tx_channel, rx_channel) = unbounded();
        let id = Uuid::new_v4();
        let mut device = Self {
            hid_info: info,
            xap_info: None,
            keymap: Default::default(),
            id,
            tx_device: tx,
            rx_thread: start_rx_thread(id, rx, event_channel, tx_channel),
            rx_channel,
        };
        device.query_device_info()?;
        device.query_keymap()?;
        Ok(device)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn is_running(&self) -> bool {
        !self.rx_thread.is_finished()
    }

    pub fn xap_info(&self) -> &XAPDeviceInfo {
        self.xap_info
            .as_ref()
            .expect("XAP device wasn't properly initialized")
    }

    pub fn keymap(&self) -> &Vec<Vec<Vec<KeyPositionConfig>>> {
        &self.keymap
    }

    pub fn is_hid_device(&self, candidate: &DeviceInfo) -> bool {
        candidate.path() == self.hid_info.path()
            && candidate.product_id() == self.hid_info.product_id()
            && candidate.vendor_id() == self.hid_info.vendor_id()
            && candidate.usage_page() == self.hid_info.usage_page()
            && candidate.usage() == self.hid_info.usage()
    }

    pub fn query_keycode(&self, position: KeyPosition) -> XAPResult<KeyCode> {
        self.do_query(KeymapKeycodeQuery(position))
    }

    pub fn do_query<T: XAPRequest>(&self, request: T) -> XAPResult<T::Response> {
        let request = RequestRaw::new(request);
        let mut report = [0; XAP_REPORT_SIZE];

        let mut writer = Cursor::new(&mut report[1..]);
        writer.write_le(&request)?;

        trace!("send XAP report with payload {:?}", &report[1..]);
        self.tx_device.write(&report)?;

        let start = Instant::now();

        let response = loop {
            let response = self
                .rx_channel
                .recv_timeout(Duration::from_millis(500))
                .map_err(|err| anyhow!("failed to reveice response {}", err))?;

            if response.token() == request.token() {
                break response;
            }
            if start.elapsed() > Duration::from_secs(5) {
                return Err(XAPError::Protocol(format!(
                    "failed to receive XAP response for request {:?} in 5 seconds",
                    request.token()
                )));
            }
        };

        response.into_xap_response::<T>()
    }

    fn query_device_info(&mut self) -> XAPResult<()> {
        let subsystems = self.do_query(XAPEnabledSubsystemsQuery)?;

        let xap_info = XAPInfo {
            version: self.do_query(XAPVersionQuery)?.0.to_string(),
        };

        let qmk_caps = self.do_query(QMKCapabilitiesQuery)?;
        let board_ids = self.do_query(QMKBoardIdentifiersQuery)?;
        let manufacturer = self.do_query(QMKBoardManufacturerQuery)?.0;
        let product_name = self.do_query(QMKProductNameQuery)?.0;
        let config = self.query_config_blob()?;
        let hardware_id = self.do_query(QMKHardwareIdentifierQuery)?.to_string();

        let qmk_info = QMKInfo {
            version: self.do_query(QMKVersionQuery)?.0.to_string(),
            board_ids,
            manufacturer,
            product_name,
            config: serde_json::to_string_pretty(&config).unwrap(),
            hardware_id,
            jump_to_bootloader_enabled: qmk_caps.contains(QMKCapabilities::JUMP_TO_BOOTLOADER),
            eeprom_reset_enabled: qmk_caps.contains(QMKCapabilities::EEPROM_RESET),
        };

        let keymap_info = if subsystems.contains(XAPEnabledSubsystems::KEYMAP) {
            let keymap_caps = self.do_query(KeymapCapabilitiesQuery)?;

            let layer_count = if keymap_caps.contains(KeymapCapabilities::LAYER_COUNT) {
                Some(self.do_query(KeymapLayerCountQuery)?.0)
            } else {
                None
            };

            // TODO ugly bodge
            let matrix = if let Some(value) = config.get("matrix_size") {
                serde_json::from_value(value.clone())
                    .map_err(|err| XAPError::Other(anyhow!("malformed matrix_size entry {err}")))
            } else {
                return Err(XAPError::Other(anyhow!(
                    "matrix size not found in JSON config"
                )));
            }?;

            Some(KeymapInfo {
                matrix,
                layer_count,
                get_keycode_enabled: keymap_caps.contains(KeymapCapabilities::GET_KEYCODE),
                get_encoder_keycode_enabled: keymap_caps
                    .contains(KeymapCapabilities::GET_ENCODER_KEYCODE),
            })
        } else {
            info!("keymap subsystem not active!");
            None
        };

        let remap_info = if subsystems.contains(XAPEnabledSubsystems::REMAPPING) {
            let keymap_caps = self.do_query(RemapCapabilitiesQuery)?;

            let layer_count = if keymap_caps.contains(RemapCapabilities::LAYER_COUNT) {
                Some(self.do_query(RemapLayerCountQuery)?.0)
            } else {
                None
            };

            Some(RemapInfo {
                layer_count,
                set_keycode_enabled: keymap_caps.contains(RemapCapabilities::SET_KEYCODE),
                set_encoder_keycode_enabled: keymap_caps
                    .contains(RemapCapabilities::SET_ENCODER_KEYCODE),
            })
        } else {
            None
        };

        let lighting_info = if subsystems.contains(XAPEnabledSubsystems::LIGHTING) {
            let lighting_caps = self.do_query(LightingCapabilitiesQuery)?;

            let backlight_info = if lighting_caps.contains(LightingCapabilities::BACKLIGHT) {
                let backlight_caps = self.do_query(BacklightCapabilitiesQuery)?;

                let effects = if backlight_caps.contains(BacklightCapabilities::ENABLED_EFFECTS) {
                    Some(self.do_query(BacklightEffectsQuery)?.enabled_effect_list())
                } else {
                    None
                };

                Some(BacklightInfo {
                    effects,
                    get_config_enabled: backlight_caps.contains(BacklightCapabilities::GET_CONFIG),
                    set_config_enabled: backlight_caps.contains(BacklightCapabilities::SET_CONFIG),
                    save_config_enabled: backlight_caps
                        .contains(BacklightCapabilities::SAVE_CONFIG),
                })
            } else {
                None
            };

            let rgblight_info = if lighting_caps.contains(LightingCapabilities::RGBLIGHT) {
                let rgblight_caps = self.do_query(RGBLightCapabilitiesQuery)?;

                let effects = if rgblight_caps.contains(RGBLightCapabilities::ENABLED_EFFECTS) {
                    Some(self.do_query(RGBLightEffectsQuery)?.enabled_effect_list())
                } else {
                    None
                };

                Some(RGBLightInfo {
                    effects,
                    get_config_enabled: rgblight_caps.contains(RGBLightCapabilities::GET_CONFIG),
                    set_config_enabled: rgblight_caps.contains(RGBLightCapabilities::SET_CONFIG),
                    save_config_enabled: rgblight_caps.contains(RGBLightCapabilities::SAVE_CONFIG),
                })
            } else {
                None
            };

            let rgbmatrix_info = if lighting_caps.contains(LightingCapabilities::RGBMATRIX) {
                let rgbmatrix_caps = self.do_query(RGBMatrixCapabilitiesQuery)?;

                let effects = if rgbmatrix_caps.contains(RGBMatrixCapabilities::ENABLED_EFFECTS) {
                    Some(self.do_query(RGBMatrixEffectsQuery)?.enabled_effect_list())
                } else {
                    None
                };

                Some(RGBMatrixInfo {
                    effects,
                    get_config_enabled: rgbmatrix_caps.contains(RGBMatrixCapabilities::GET_CONFIG),
                    set_config_enabled: rgbmatrix_caps.contains(RGBMatrixCapabilities::SET_CONFIG),
                    save_config_enabled: rgbmatrix_caps
                        .contains(RGBMatrixCapabilities::SAVE_CONFIG),
                })
            } else {
                None
            };

            Some(LightingInfo {
                backlight: backlight_info,
                rgblight: rgblight_info,
                rgbmatrix: rgbmatrix_info,
            })
        } else {
            None
        };

        self.xap_info = Some(XAPDeviceInfo {
            xap: xap_info,
            qmk: qmk_info,
            keymap: keymap_info,
            remap: remap_info,
            lighting: lighting_info,
        });

        Ok(())
    }

    fn query_config_blob(&self) -> XAPResult<Map<String, Value>> {
        // Query data size
        let size = self.do_query(QMKConfigBlobLengthQuery {})?.0;

        // Query all chunks and merge them in a Vec
        let mut data: Vec<u8> = Vec::with_capacity(size as usize);
        let mut offset: u16 = 0;
        while offset < size {
            let chunk = self.do_query(ConfigBlobChunkQuery(offset))?;
            data.extend(chunk.0.into_iter());
            offset += chunk.0.len() as u16;
        }

        // Trim trailing zeroes and convert Vec into array
        let data = &data[..(size as usize)];

        // Decompress data
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = String::new();
        decoder
            .read_to_string(&mut decompressed)
            .map_err(|err| anyhow!("failed to decompress config json blob: {}", err))?;

        Ok(serde_json::from_str(&decompressed)
            .map_err(|err| anyhow!("config json is not valid json {err}"))?)
    }

    fn query_keymap(&mut self) -> XAPResult<()> {
        // Reset keymap
        self.keymap = Default::default();

        if let Some(keymap) = &self.xap_info().keymap {
            let layers = keymap.layer_count.unwrap_or_default();
            let cols = keymap.matrix.cols;
            let rows = keymap.matrix.rows;

            let keymap: Result<Vec<Vec<Vec<KeyPositionConfig>>>, XAPError> = (0..layers)
                .map(|layer| {
                    (0..rows)
                        .map(|row| {
                            (0..cols)
                                .map(|col| {
                                    let keycode = self.query_keycode(KeyPosition {
                                        layer: layer,
                                        row: row,
                                        col: col,
                                    })?;

                                    Ok(KeyPositionConfig {
                                        layer: layer,
                                        row: row,
                                        column: col,
                                        keycode: keycode.0,
                                    })
                                })
                                .collect()
                        })
                        .collect()
                })
                .collect();

            self.keymap = keymap?;
        }

        Ok(())
    }
}

fn start_rx_thread(
    device_id: Uuid,
    rx: HidDevice,
    event_channel: Sender<XAPEvent>,
    tx_channel: Sender<ResponseRaw>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut report = [0_u8; XAP_REPORT_SIZE];
        loop {
            if let Err(err) = rx.read(&mut report) {
                error!("failed to receive HID report: {err}");
                event_channel
                    .send(XAPEvent::RxError)
                    .expect("failed to send error event");
                break;
            }
            if let Err(err) = handle_report(device_id, report, &tx_channel, &event_channel) {
                error!("handling response failed: {err}")
            }
        }
        info!("terminating capture thread for {device_id}");
    })
}

fn handle_report(
    device_id: Uuid,
    report: [u8; 64],
    tx_channel: &Sender<ResponseRaw>,
    event_channel: &Sender<XAPEvent>,
) -> XAPResult<()> {
    let mut reader = Cursor::new(&report);
    let token = Token::read_le(&mut reader)?;

    if let Token::Broadcast = token {
        let broadcast = BroadcastRaw::from_raw_report(&report)?;

        match broadcast.broadcast_type() {
            BroadcastType::Log => {
                let log: LogBroadcast = broadcast.into_xap_broadcast()?;
                event_channel
                    .send(XAPEvent::LogReceived {
                        id: device_id,
                        log: log.0,
                    })
                    .expect("failed to send broadcast event!");
            }
            BroadcastType::SecureStatus => {
                let secure_status: SecureStatusBroadcast = broadcast.into_xap_broadcast()?;
                event_channel
                    .send(XAPEvent::SecureStatusChanged {
                        id: device_id,
                        secure_status: secure_status.0,
                    })
                    .expect("failed to send broadcast event!");
            }
            BroadcastType::Keyboard => info!("keyboard broadcasts are not implemented!"),
            BroadcastType::User => info!("keyboard broadcasts are not implemented!"),
        }
    } else {
        let response = ResponseRaw::from_raw_report(&report)?;
        trace!(
            "received XAP package with token {:?} and payload {:#?}",
            response.token(),
            response.payload()
        );
        tx_channel
            .send(response)
            .expect("failed to forward received XAP report");
    }

    Ok(())
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct XAPDeviceDTO {
    id: String,
    info: XAPDeviceInfo,
    keymap: Vec<Vec<Vec<KeyPositionConfig>>>,
    secure_status: XAPSecureStatus
}

impl From<&XAPDevice> for XAPDeviceDTO {
    fn from(device: &XAPDevice) -> Self {
        Self {
            id: device.id.to_string(),
            info: device.xap_info().clone(),
            keymap: device.keymap.clone(),
            // TODO
            secure_status: XAPSecureStatus::Disabled
        }
    }
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct XAPDeviceInfo {
    pub xap: XAPInfo,
    pub qmk: QMKInfo,
    pub keymap: Option<KeymapInfo>,
    pub remap: Option<RemapInfo>,
    pub lighting: Option<LightingInfo>,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct XAPInfo {
    version: String,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct QMKInfo {
    version: String,
    board_ids: QMKBoardIdentifiers,
    manufacturer: String,
    product_name: String,
    config: String,
    hardware_id: String,
    jump_to_bootloader_enabled: bool,
    eeprom_reset_enabled: bool,
}

#[derive(Deserialize, Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct Matrix {
    cols: u8,
    rows: u8,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct KeymapInfo {
    matrix: Matrix,
    layer_count: Option<u8>,
    get_keycode_enabled: bool,
    get_encoder_keycode_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RemapInfo {
    layer_count: Option<u8>,
    set_keycode_enabled: bool,
    set_encoder_keycode_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct LightingInfo {
    backlight: Option<BacklightInfo>,
    rgblight: Option<RGBLightInfo>,
    rgbmatrix: Option<RGBMatrixInfo>,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct BacklightInfo {
    effects: Option<Vec<u8>>,
    get_config_enabled: bool,
    set_config_enabled: bool,
    save_config_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RGBLightInfo {
    effects: Option<Vec<u8>>,
    get_config_enabled: bool,
    set_config_enabled: bool,
    save_config_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RGBMatrixInfo {
    effects: Option<Vec<u8>>,
    get_config_enabled: bool,
    set_config_enabled: bool,
    save_config_enabled: bool,
}
