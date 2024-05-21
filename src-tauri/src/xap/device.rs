use std::{
    fmt::Debug,
    io::{Cursor, Read},
    sync::Arc,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use binrw::{BinRead, BinWriterExt};
use crossbeam_channel::{unbounded, Receiver, Sender};
use flate2::read::GzDecoder;
use hidapi::{DeviceInfo, HidDevice};
use log::{error, info, trace};
use parking_lot::RwLock;
use serde_json::{Map, Value};
use uuid::Uuid;

use xap_specs::{
    broadcast::{BroadcastRaw, BroadcastType, LogBroadcast, SecureStatusBroadcast},
    constants::{keycode::XapKeyCodeConfig, XapConstants},
    error::{XapError, XapResult},
    request::{RawRequest, XapRequest},
    response::RawResponse,
    token::Token,
    KeyPosition, KeyPositionConfig, XapSecureStatus,
};

use crate::{
    aggregation::{
        KeymapInfo, LightingCapabilities, LightingInfo, QmkInfo, RemapInfo,
        XapDevice as XapDeviceDto, XapDeviceInfo, XapInfo,
    },
    xap::spec::{
        keymap::{
            KeymapCapabilitiesFlags, KeymapCapabilitiesRequest, KeymapGetKeycodeArg,
            KeymapGetKeycodeRequest, KeymapGetKeycodeResponse, KeymapGetLayerCountRequest,
        },
        lighting::{
            backlight::{
                BacklightCapabilitiesFlags, BacklightCapabilitiesRequest,
                BacklightGetEnabledEffectsRequest,
            },
            rgblight::{
                RgblightCapabilitiesFlags, RgblightCapabilitiesRequest,
                RgblightGetEnabledEffectsRequest,
            },
            rgbmatrix::{
                RgbmatrixCapabilitiesFlags, RgbmatrixCapabilitiesRequest,
                RgbmatrixGetEnabledEffectsRequest,
            },
            LightingCapabilitiesFlags, LightingCapabilitiesRequest,
        },
        qmk::{
            QmkBoardIdentifiersRequest, QmkBoardManufacturerRequest, QmkCapabilitiesFlags,
            QmkCapabilitiesRequest, QmkConfigBlobChunkRequest, QmkConfigBlobLengthRequest,
            QmkHardwareIdentifierRequest, QmkProductNameRequest, QmkVersionRequest,
        },
        remapping::{
            RemappingCapabilitiesFlags, RemappingCapabilitiesRequest,
            RemappingGetLayerCountRequest, RemappingSetKeycodeArg, RemappingSetKeycodeRequest,
        },
        xap::{
            XapEnabledSubsystemCapabilitiesFlags, XapEnabledSubsystemCapabilitiesRequest,
            XapSecureStatusRequest, XapVersionRequest,
        },
    },
    xap::client::{XapClientError, XapClientResult},
    XapEvent,
};

const XAP_REPORT_SIZE: usize = 64;

#[derive(Debug, Default)]
struct XapDeviceState {
    xap_info: Option<XapDeviceInfo>,
    keymap: Vec<Vec<Vec<XapKeyCodeConfig>>>,
    secure_status: XapSecureStatus,
}

#[derive(Debug)]
pub struct XapDevice {
    id: Uuid,
    info: DeviceInfo,
    rx_thread: JoinHandle<()>,
    tx_device: HidDevice,
    rx_channel: Receiver<RawResponse>,
    constants: Arc<XapConstants>,
    state: Arc<RwLock<XapDeviceState>>,
}

impl XapDevice {
    pub(crate) fn new(
        info: DeviceInfo,
        constants: Arc<XapConstants>,
        event_channel: Sender<XapEvent>,
        rx_device: HidDevice,
        tx_device: HidDevice,
    ) -> XapClientResult<Self> {
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(XapDeviceState::default()));

        let (tx_channel, rx_channel) = unbounded();

        let device = Self {
            id,
            info,
            tx_device,
            rx_channel,
            rx_thread: start_rx_thread(
                id,
                Arc::clone(&state),
                rx_device,
                event_channel,
                tx_channel,
            ),
            state,
            constants,
        };
        device.query_device_info()?;
        device.query_keymap()?;
        device.query_secure_status()?;
        Ok(device)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn is_running(&self) -> bool {
        !self.rx_thread.is_finished()
    }

    pub fn xap_info(&self) -> XapDeviceInfo {
        self.state
            .read()
            .xap_info
            .clone()
            .expect("XAP device wasn't properly initialized")
    }

    pub fn keymap(&self) -> Vec<Vec<Vec<XapKeyCodeConfig>>> {
        self.state.read().keymap.clone()
    }

    pub fn as_dto(&self) -> XapDeviceDto {
        let state = self.state.read();
        XapDeviceDto {
            id: self.id,
            info: state
                .xap_info
                .as_ref()
                .expect("XAP device wasn't properly initialized")
                .clone(),
            keymap: state.keymap.clone(),
            secure_status: state.secure_status,
        }
    }

    pub fn is_hid_device(&self, candidate: &DeviceInfo) -> bool {
        candidate.path() == self.info.path()
            && candidate.product_id() == self.info.product_id()
            && candidate.vendor_id() == self.info.vendor_id()
            && candidate.usage_page() == self.info.usage_page()
            && candidate.usage() == self.info.usage()
    }

    pub fn set_keycode(&self, config: KeyPositionConfig) -> XapClientResult<()> {
        let (layer, row, column, keycode) = (config.layer, config.row, config.col, config.keycode);

        let arg = RemappingSetKeycodeArg {
            layer,
            row,
            column,
            keycode,
        };

        self.query(RemappingSetKeycodeRequest(arg))?;

        self.state.write().keymap[layer as usize][row as usize][column as usize] =
            XapKeyCodeConfig {
                code: self.constants.get_keycode(keycode),
                position: KeyPosition { layer, row, column },
            };

        Ok(())
    }

    pub fn query_keycode(&self, position: KeyPosition) -> XapClientResult<KeymapGetKeycodeResponse> {
        self.query(KeymapGetKeycodeRequest(KeymapGetKeycodeArg {
            layer: position.layer,
            row: position.row,
            column: position.column,
        }))
    }

    pub fn query<T: XapRequest>(&self, request: T) -> XapClientResult<T::Response> {
        if let Some(xap_info) = &self.state.read().xap_info {
            if !T::xap_version() < xap_info.xap.version {
                return Err(XapClientError::ProtocolError(XapError::Protocol(format!(
                    "can't do xap request [{:?}] with client of version {}",
                    T::id(),
                    xap_info.xap.version
                ))));
            }
        }

        let request = RawRequest::new(request);
        let mut report = [0; XAP_REPORT_SIZE + 1];

        // Add trailing zero byte for the report Id to HID report
        let mut writer = Cursor::new(&mut report[1..]);
        writer
            .write_le(&request)
            .map_err(|err| XapClientError::from(XapError::BitHandling(err)))?;

        trace!("send XAP report with payload {:?}", &report[1..]);

        self.tx_device.write(&report)?;

        let start = Instant::now();

        let response = loop {
            let response = self
                .rx_channel
                .recv_timeout(Duration::from_millis(500))
                .map_err(|err| XapError::Protocol(format!("failed to reveice response {}", err)))?;

            if response.token() == request.token() {
                break response;
            }
            if start.elapsed() > Duration::from_secs(5) {
                return Err(XapError::Protocol(format!(
                    "failed to receive XAP response for request {:?} in 5 seconds",
                    request.token()
                ))
                .into());
            }
        };

        response.into_xap_response::<T>().map_err(XapClientError::from)
    }

    pub fn query_secure_status(&self) -> XapClientResult<XapSecureStatus> {
        let status = self.query(XapSecureStatusRequest(()))?.0.into();
        self.state.write().secure_status = status;
        Ok(status)
    }

    fn query_device_info(&self) -> XapClientResult<()> {
        let subsystems = self.query(XapEnabledSubsystemCapabilitiesRequest(()))?;

        let xap_info = XapInfo {
            version: self.query(XapVersionRequest(()))?.0,
        };

        let qmk_caps = self.query(QmkCapabilitiesRequest(()))?;
        let board_ids = self.query(QmkBoardIdentifiersRequest(()))?;
        // TODO: why do these strings have leading and trailing " characters -
        // should be removed in QMK
        let manufacturer = self
            .query(QmkBoardManufacturerRequest(()))?
            .0
             .0
            .trim_matches('"')
            .to_owned();
        let product_name = self
            .query(QmkProductNameRequest(()))?
            .0
             .0
            .trim_matches('"')
            .to_owned();
        let config = self.query_config_blob()?;
        let hardware_id = self.query(QmkHardwareIdentifierRequest(()))?.0;

        let qmk_info = QmkInfo {
            version: self.query(QmkVersionRequest(()))?.0.to_string(),
            board_ids,
            manufacturer,
            product_name,
            config: serde_json::to_string_pretty(&config).unwrap(),
            hardware_id: format!(
                "{}{}{}{}",
                hardware_id[0], hardware_id[1], hardware_id[2], hardware_id[3]
            ),
            jump_to_bootloader_enabled: qmk_caps.contains(QmkCapabilitiesFlags::JumpToBootloader),
            eeprom_reset_enabled: qmk_caps.contains(QmkCapabilitiesFlags::ReinitializeEeprom),
        };

        let keymap_info = if subsystems.contains(XapEnabledSubsystemCapabilitiesFlags::Keymap) {
            let keymap_caps = self.query(KeymapCapabilitiesRequest(()))?;

            let layer_count = if keymap_caps.contains(KeymapCapabilitiesFlags::GetLayerCount) {
                Some(self.query(KeymapGetLayerCountRequest(()))?.0)
            } else {
                None
            };

            // TODO ugly bodge
            let matrix = if let Some(value) = config.get("matrix_size") {
                serde_json::from_value(value.clone())
                    .map_err(|err| XapClientError::Other(anyhow!("malformed matrix_size entry {err}")))
            } else {
                return Err(XapClientError::Other(anyhow!(
                    "matrix size not found in JSON config"
                )));
            }?;

            Some(KeymapInfo {
                matrix,
                layer_count,
                get_keycode_enabled: keymap_caps.contains(KeymapCapabilitiesFlags::GetKeycode),
                get_encoder_keycode_enabled: keymap_caps
                    .contains(KeymapCapabilitiesFlags::GetEncoderKeycode),
            })
        } else {
            info!("keymap subsystem not active!");
            None
        };

        let remap_info = if subsystems.contains(XapEnabledSubsystemCapabilitiesFlags::Remapping) {
            let keymap_caps = self.query(RemappingCapabilitiesRequest(()))?;

            let layer_count = if keymap_caps.contains(RemappingCapabilitiesFlags::GetLayerCount) {
                Some(self.query(RemappingGetLayerCountRequest(()))?.0)
            } else {
                None
            };

            Some(RemapInfo {
                layer_count,
                set_keycode_enabled: keymap_caps.contains(RemappingCapabilitiesFlags::SetKeycode),
                set_encoder_keycode_enabled: keymap_caps
                    .contains(RemappingCapabilitiesFlags::SetEncoderKeycode),
            })
        } else {
            None
        };

        let lighting_info = if subsystems.contains(XapEnabledSubsystemCapabilitiesFlags::Lighting) {
            let lighting_caps = self.query(LightingCapabilitiesRequest(()))?;

            let backlight_info = if lighting_caps.contains(LightingCapabilitiesFlags::Backlight) {
                let backlight_caps = self.query(BacklightCapabilitiesRequest(()))?;

                let effects =
                    if backlight_caps.contains(BacklightCapabilitiesFlags::GetEnabledEffects) {
                        self.query(BacklightGetEnabledEffectsRequest(()))?.0
                    } else {
                        0
                    };

                Some(LightingCapabilities::new(
                    // Todo: implement backlight effects
                    self.constants
                        .led_matrix_modes
                        .get_effect_map(effects as u64),
                    backlight_caps.contains(BacklightCapabilitiesFlags::GetConfig),
                    backlight_caps.contains(BacklightCapabilitiesFlags::SetConfig),
                    backlight_caps.contains(BacklightCapabilitiesFlags::SaveConfig),
                ))
            } else {
                None
            };

            let rgblight_info = if lighting_caps.contains(LightingCapabilitiesFlags::Rgblight) {
                let rgblight_caps = self.query(RgblightCapabilitiesRequest(()))?;

                let effects =
                    if rgblight_caps.contains(RgblightCapabilitiesFlags::GetEnabledEffects) {
                        self.query(RgblightGetEnabledEffectsRequest(()))?.0
                    } else {
                        0
                    };

                Some(LightingCapabilities::new(
                    self.constants.rgblight_modes.get_effect_map(effects),
                    rgblight_caps.contains(RgblightCapabilitiesFlags::GetConfig),
                    rgblight_caps.contains(RgblightCapabilitiesFlags::SetConfig),
                    rgblight_caps.contains(RgblightCapabilitiesFlags::SaveConfig),
                ))
            } else {
                None
            };

            let rgbmatrix_info = if lighting_caps.contains(LightingCapabilitiesFlags::Rgbmatrix) {
                let rgbmatrix_caps = self.query(RgbmatrixCapabilitiesRequest(()))?;

                let effects =
                    if rgbmatrix_caps.contains(RgbmatrixCapabilitiesFlags::GetEnabledEffects) {
                        self.query(RgbmatrixGetEnabledEffectsRequest(()))?.0
                    } else {
                        0
                    };

                Some(LightingCapabilities::new(
                    self.constants.rgb_matrix_modes.get_effect_map(effects),
                    rgbmatrix_caps.contains(RgbmatrixCapabilitiesFlags::GetConfig),
                    rgbmatrix_caps.contains(RgbmatrixCapabilitiesFlags::SetConfig),
                    rgbmatrix_caps.contains(RgbmatrixCapabilitiesFlags::SaveConfig),
                ))
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

        self.state.write().xap_info = Some(XapDeviceInfo {
            xap: xap_info,
            qmk: qmk_info,
            keymap: keymap_info,
            remap: remap_info,
            lighting: lighting_info,
        });

        Ok(())
    }

    fn query_config_blob(&self) -> XapClientResult<Map<String, Value>> {
        //  data size
        let size = self.query(QmkConfigBlobLengthRequest(()))?.0;

        //  all chunks and merge them in a Vec
        let mut data: Vec<u8> = Vec::with_capacity(size as usize);
        let mut offset: u16 = 0;
        while offset < size {
            let chunk = self.query(QmkConfigBlobChunkRequest(offset))?;
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

        Ok(serde_json::from_str(&decompressed)?)
    }

    fn query_keymap(&self) -> XapClientResult<()> {
        // Reset keymap
        self.state.write().keymap = Default::default();

        if let Some(keymap) = &self.xap_info().keymap {
            let layers = keymap.layer_count.unwrap_or_default();
            let cols = keymap.matrix.cols;
            let rows = keymap.matrix.rows;

            let keymap: Result<Vec<Vec<Vec<XapKeyCodeConfig>>>, XapClientError> = (0..layers)
                .map(|layer| {
                    (0..rows)
                        .map(|row| {
                            (0..cols)
                                .map(|col| {
                                    let keycode = self.query_keycode(KeyPosition {
                                        layer,
                                        row,
                                        column: col,
                                    })?;

                                    let xap = XapKeyCodeConfig {
                                        code: self.constants.get_keycode(keycode.0),
                                        position: KeyPosition {
                                            layer,
                                            row,
                                            column: col,
                                        },
                                    };

                                    Ok(xap)
                                })
                                .collect()
                        })
                        .collect()
                })
                .collect();

            self.state.write().keymap = keymap?;
        }

        Ok(())
    }
}

fn start_rx_thread(
    id: Uuid,
    state: Arc<RwLock<XapDeviceState>>,
    rx: HidDevice,
    event_channel: Sender<XapEvent>,
    tx_channel: Sender<RawResponse>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut report = [0_u8; XAP_REPORT_SIZE];
        loop {
            if let Err(err) = rx.read(&mut report) {
                error!("failed to receive HID report: {err}");
                event_channel
                    .send(XapEvent::RxError)
                    .expect("failed to send error event");
                break;
            }
            if let Err(err) = handle_report(id, &state, report, &tx_channel, &event_channel) {
                error!("handling response failed: {err}")
            }
        }
        info!("terminating capture thread for {id}");
    })
}

fn handle_report(
    id: Uuid,
    state: &Arc<RwLock<XapDeviceState>>,
    report: [u8; XAP_REPORT_SIZE],
    tx_channel: &Sender<RawResponse>,
    event_channel: &Sender<XapEvent>,
) -> XapResult<()> {
    let mut reader = Cursor::new(&report);
    let token = Token::read_le(&mut reader)?;

    if let Token::Broadcast = token {
        let broadcast = BroadcastRaw::from_raw_report(&report)?;

        match broadcast.broadcast_type() {
            BroadcastType::Log => {
                let log: LogBroadcast = broadcast.into_xap_broadcast()?;
                event_channel
                    .send(XapEvent::LogReceived { id, log: log.0 })
                    .expect("failed to send broadcast event!");
            }
            BroadcastType::SecureStatus => {
                let secure_status: SecureStatusBroadcast = broadcast.into_xap_broadcast()?;
                state.write().secure_status = secure_status.0;
                event_channel
                    .send(XapEvent::SecureStatusChanged {
                        id,
                        secure_status: secure_status.0,
                    })
                    .expect("failed to send broadcast event!");
            }
            BroadcastType::Keyboard => info!("keyboard broadcasts are not implemented!"),
            BroadcastType::User => info!("user broadcasts are not implemented!"),
        }
    } else {
        let response = RawResponse::from_raw_report(&report)?;
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
