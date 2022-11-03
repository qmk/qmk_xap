use std::{
    fmt::{Debug, Display},
    io::{Cursor, Read},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use binrw::BinWriterExt;
use crossbeam_channel::{unbounded, Receiver, Sender};
use flate2::read::GzDecoder;
use hidapi::{DeviceInfo, HidDevice};
use log::{error, trace};
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::{xap::*, XAPEvent};

const XAP_REPORT_SIZE: usize = 64;

pub struct XAPDevice {
    id: Uuid,
    info: DeviceInfo,
    xap_info: Option<XAPDeviceInfo>,
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
            self.info.vendor_id(),
            self.info.product_id(),
            match self.info.serial_number() {
                Some(s) => s,
                _ => "<COULD NOT FETCH>",
            },
            match self.info.product_string() {
                Some(s) => s,
                _ => "<COULD NOT FETCH>",
            },
            match self.info.manufacturer_string() {
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
            info,
            xap_info: None,
            id,
            tx_device: tx,
            rx_thread: Self::start_rx_thread(id, rx, event_channel, tx_channel),
            rx_channel,
        };
        device.query_device_info()?;
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

    pub fn is_hid_device(&self, candidate: &DeviceInfo) -> bool {
        candidate.path() == self.info.path()
            && candidate.product_id() == self.info.product_id()
            && candidate.vendor_id() == self.info.vendor_id()
            && candidate.usage_page() == self.info.usage_page()
            && candidate.usage() == self.info.usage()
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

        let xap_caps = self.do_query(XAPCapabilitiesQuery)?;

        let xap_info = XAPInfo {
            version: self.do_query(XAPVersionQuery)?.0.to_string(),
            secure_status: xap_caps.contains(XAPCapabilities::SECURE_STATUS),
            secure_unlock: xap_caps.contains(XAPCapabilities::SECURE_UNLOCK),
            secure_lock: xap_caps.contains(XAPCapabilities::SECURE_LOCK),
        };

        let qmk_caps = self.do_query(QMKCapabilitiesQuery)?;

        let board_ids = if qmk_caps.contains(QMKCapabilities::BOARD_IDS) {
            Some(self.do_query(QMKBoardIdentifiersQuery)?)
        } else {
            None
        };

        let manufacturer = if qmk_caps.contains(QMKCapabilities::BOARD_MANUFACTURER) {
            Some(self.do_query(QMKBoardManufacturerQuery)?.0)
        } else {
            None
        };

        let product_name = if qmk_caps.contains(QMKCapabilities::PRODUCT_NAME) {
            Some(self.do_query(QMKProductNameQuery)?.0)
        } else {
            None
        };

        let config = if qmk_caps
            .contains(QMKCapabilities::CONFIG_BLOB_CHUNK & QMKCapabilities::CONFIG_BLOB_LENGTH)
        {
            Some(self.query_config_blob()?)
        } else {
            None
        };

        let hardware_id = if qmk_caps.contains(QMKCapabilities::HARDWARE_ID) {
            Some(self.do_query(QMKHardwareIdentifierQuery)?.to_string())
        } else {
            None
        };

        let qmk_info = QMKInfo {
            version: self.do_query(QMKVersionQuery)?.0.to_string(),
            board_ids,
            manufacturer,
            product_name,
            config,
            hardware_id,
            jump_to_bootloader: qmk_caps.contains(QMKCapabilities::JUMP_TO_BOOTLOADER),
            eeprom_reset: qmk_caps.contains(QMKCapabilities::EEPROM_RESET),
        };

        let keymap_info = if subsystems.contains(XAPEnabledSubsystems::KEYMAP) {
            let keymap_caps = self.do_query(KeymapCapabilitiesQuery)?;

            let layer_count = if keymap_caps.contains(KeymapCapabilities::LAYER_COUNT) {
                Some(self.do_query(KeymapLayerCountQuery)?.0)
            } else {
                None
            };

            Some(KeymapInfo {
                layer_count,
                get_keycode: keymap_caps.contains(KeymapCapabilities::GET_KEYCODE),
                get_encoder_keycode: keymap_caps.contains(KeymapCapabilities::GET_ENCODER_KEYCODE),
            })
        } else {
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
                set_keycode: keymap_caps.contains(RemapCapabilities::SET_KEYCODE),
                set_encoder_keycode: keymap_caps.contains(RemapCapabilities::SET_ENCODER_KEYCODE),
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
                    get_config: backlight_caps.contains(BacklightCapabilities::GET_CONFIG),
                    set_config: backlight_caps.contains(BacklightCapabilities::SET_CONFIG),
                    save_config: backlight_caps.contains(BacklightCapabilities::SAVE_CONFIG),
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
                    get_config: rgblight_caps.contains(RGBLightCapabilities::GET_CONFIG),
                    set_config: rgblight_caps.contains(RGBLightCapabilities::SET_CONFIG),
                    save_config: rgblight_caps.contains(RGBLightCapabilities::SAVE_CONFIG),
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
                    get_config: rgbmatrix_caps.contains(RGBMatrixCapabilities::GET_CONFIG),
                    set_config: rgbmatrix_caps.contains(RGBMatrixCapabilities::SET_CONFIG),
                    save_config: rgbmatrix_caps.contains(RGBMatrixCapabilities::SAVE_CONFIG),
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

    fn query_config_blob(&self) -> XAPResult<String> {
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
        let mut string = String::new();
        decoder
            .read_to_string(&mut string)
            .map_err(|err| anyhow!("failed to decompress config json blob: {}", err))?;

        Ok(string)
    }

    fn start_rx_thread(
        device_id: Uuid,
        rx: HidDevice,
        event_channel: Sender<XAPEvent>,
        tx_channel: Sender<ResponseRaw>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let result: XAPResult<()> = (|| {
                let mut report = [0_u8; XAP_REPORT_SIZE];
                loop {
                    rx.read(&mut report)?;

                    match ResponseRaw::from_raw_report(&report) {
                        Ok(response) => {
                            if *response.token() == Token::Broadcast {
                                trace!(
                                    "received XAP broadcast package with payload {:#?}",
                                    response.payload()
                                );
                                event_channel
                                    .send(XAPEvent::Broadcast {
                                        id: device_id,
                                        response,
                                    })
                                    .expect("failed to send broadcast event!");
                            } else {
                                trace!(
                                    "
                                received XAP package with token {:?} and payload {:#?}",
                                    response.token(),
                                    response.payload()
                                );
                                tx_channel
                                    .send(response)
                                    .expect("failed to forward received XAP report");
                            }
                        }
                        Err(err) => error!("received malformed XAP HID report {err}"),
                    }
                }
            })();

            if let Err(err) = result {
                // Terminate thread and notify state
                event_channel
                    .send(XAPEvent::RxError {
                        id: device_id,
                        error: err,
                    })
                    .expect("failed to send error event!");
            }
        })
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
    secure_status: bool,
    secure_unlock: bool,
    secure_lock: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct QMKInfo {
    version: String,
    board_ids: Option<QMKBoardIdentifiers>,
    manufacturer: Option<String>,
    product_name: Option<String>,
    config: Option<String>,
    hardware_id: Option<String>,
    jump_to_bootloader: bool,
    eeprom_reset: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct KeymapInfo {
    layer_count: Option<u8>,
    get_keycode: bool,
    get_encoder_keycode: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RemapInfo {
    layer_count: Option<u8>,
    set_keycode: bool,
    set_encoder_keycode: bool,
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
    get_config: bool,
    set_config: bool,
    save_config: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RGBLightInfo {
    effects: Option<Vec<u8>>,
    get_config: bool,
    set_config: bool,
    save_config: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RGBMatrixInfo {
    effects: Option<Vec<u8>>,
    get_config: bool,
    set_config: bool,
    save_config: bool,
}
