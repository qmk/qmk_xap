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
use log::{debug, error, info, trace};
use parking_lot::RwLock;
use serde_json::{Map, Value};
use uuid::Uuid;

use xap_specs::{
    constants::{
        keycode::{KeyCoords, XAPKeyCodeConfig, XAPKeyInfo},
        XAPConstants,
    },
    error::{XAPError, XAPResult},
    protocol::{
        keymap::{
            KeyCode, KeyPosition, KeymapCapabilities, KeymapCapabilitiesQuery, KeymapKeycodeQuery,
            KeymapLayerCountQuery,
        },
        lighting::{
            BacklightCapabilities, BacklightCapabilitiesQuery, BacklightEffectsQuery,
            LightingCapabilities, LightingCapabilitiesQuery, RGBLightCapabilities,
            RGBLightCapabilitiesQuery, RGBLightEffectsQuery, RGBMatrixCapabilities,
            RGBMatrixCapabilitiesQuery, RGBMatrixEffectsQuery,
        },
        qmk::{
            ConfigBlobChunkQuery, QMKBoardIdentifiersQuery, QMKBoardManufacturerQuery,
            QMKCapabilities, QMKCapabilitiesQuery, QMKConfigBlobLengthQuery,
            QMKHardwareIdentifierQuery, QMKProductNameQuery, QMKVersionQuery,
        },
        remap::{
            KeyPositionConfig, RemapCapabilities, RemapCapabilitiesQuery, RemapKeycodeQuery,
            RemapLayerCountQuery,
        },
        xap::{
            XAPEnabledSubsystems, XAPEnabledSubsystemsQuery, XAPSecureStatus, XAPSecureStatusQuery,
            XAPVersionQuery,
        },
        *,
    },
    request::{RawRequest, XAPRequest},
    response::RawResponse,
    token::Token,
};

use crate::{
    aggregation::{
        BacklightInfo, KeymapInfo, LightingInfo, QMKInfo, RGBLightInfo, RGBMatrixInfo, RemapInfo,
        XAPDevice as XAPDeviceDto, XAPDeviceInfo, XAPInfo,
    },
    xap::{ClientError, ClientResult},
    XAPEvent,
};

const XAP_REPORT_SIZE: usize = 64;

#[derive(Debug, Default)]
struct XAPDeviceState {
    xap_info: Option<XAPDeviceInfo>,
    keymap: Vec<Vec<Vec<XAPKeyCodeConfig>>>,
    key_info: Vec<Vec<Vec<Option<XAPKeyInfo>>>>,
    coords_from_rowcol: Vec<Vec<Option<KeyCoords>>>,
    secure_status: XAPSecureStatus,
}

#[derive(Debug)]
pub struct XAPDevice {
    id: Uuid,
    info: DeviceInfo,
    rx_thread: JoinHandle<()>,
    tx_device: HidDevice,
    rx_channel: Receiver<RawResponse>,
    constants: Arc<XAPConstants>,
    state: Arc<RwLock<XAPDeviceState>>,
}

impl XAPDevice {
    pub(crate) fn new(
        info: DeviceInfo,
        constants: Arc<XAPConstants>,
        event_channel: Sender<XAPEvent>,
        rx_device: HidDevice,
        tx_device: HidDevice,
    ) -> ClientResult<Self> {
        let (tx_channel, rx_channel) = unbounded();
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(XAPDeviceState::default()));
        let device = Self {
            info,
            id,
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
        device.generate_coords_from_rowcol()?;
        device.generate_key_info();
        device.query_secure_status()?;
        Ok(device)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn is_running(&self) -> bool {
        !self.rx_thread.is_finished()
    }

    pub fn xap_info(&self) -> XAPDeviceInfo {
        self.state
            .read()
            .xap_info
            .clone()
            .expect("XAP device wasn't properly initialized")
    }

    pub fn keymap(&self) -> Vec<Vec<Vec<XAPKeyCodeConfig>>> {
        self.state.read().keymap.clone()
    }

    pub fn key_info(&self) -> Vec<Vec<Vec<Option<XAPKeyInfo>>>> {
        self.state.read().key_info.clone()
    }

    fn coords_from_rowcol(&self) -> Vec<Vec<Option<KeyCoords>>> {
        self.state.read().coords_from_rowcol.clone()
    }

    pub fn as_dto(&self) -> XAPDeviceDto {
        let state = self.state.read();
        XAPDeviceDto {
            id: self.id,
            info: state
                .xap_info
                .as_ref()
                .expect("XAP device wasn't properly initialized")
                .clone(),
            key_info: state.key_info.clone(),
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

    pub fn set_keycode(&self, config: KeyPositionConfig) -> ClientResult<()> {
        self.query(RemapKeycodeQuery(config.clone()))?;
        let (layer, row, col) = (config.layer, config.row, config.col);

        self.state.write().keymap[layer as usize][row as usize][col as usize] = XAPKeyCodeConfig {
            code: self.constants.get_keycode(config.keycode),
            position: KeyPosition {
                layer: config.layer,
                row: config.row,
                col: config.col,
            },
        };

        Ok(())
    }

    pub fn query_keycode(&self, position: KeyPosition) -> ClientResult<KeyCode> {
        self.query(KeymapKeycodeQuery(position))
    }

    pub fn query<T: XAPRequest>(&self, request: T) -> ClientResult<T::Response> {
        let request = RawRequest::new(request);
        let mut report = [0; XAP_REPORT_SIZE + 1];

        // Add trailing zero byte for the report Id to HID report
        trace!("send XAP report with payload {:?}", &report[1..]);
        let mut writer = Cursor::new(&mut report[1..]);
        writer
            .write_le(&request)
            .map_err(|err| ClientError::from(XAPError::BitHandling(err)))?;

        self.tx_device.write(&report)?;

        let start = Instant::now();

        let response = loop {
            let response = self
                .rx_channel
                .recv_timeout(Duration::from_millis(500))
                .map_err(|err| XAPError::Protocol(format!("failed to reveice response {}", err)))?;

            if response.token() == request.token() {
                break response;
            }
            if start.elapsed() > Duration::from_secs(5) {
                return Err(XAPError::Protocol(format!(
                    "failed to receive XAP response for request {:?} in 5 seconds",
                    request.token()
                ))
                .into());
            }
        };

        response
            .into_xap_response::<T>()
            .map_err(|err| ClientError::from(err))
    }

    pub fn query_secure_status(&self) -> ClientResult<XAPSecureStatus> {
        let status = self.query(XAPSecureStatusQuery {})?;
        self.state.write().secure_status = status;
        Ok(status)
    }

    fn query_device_info(&self) -> ClientResult<()> {
        let subsystems = self.query(XAPEnabledSubsystemsQuery)?;

        let xap_info = XAPInfo {
            version: self.query(XAPVersionQuery)?.0.to_string(),
        };

        let qmk_caps = self.query(QMKCapabilitiesQuery)?;
        let board_ids = self.query(QMKBoardIdentifiersQuery)?;
        // TODO: why do these strings have leading and trailing " characters -
        // should be removed in QMK
        let manufacturer = self
            .query(QMKBoardManufacturerQuery)?
            .0
            .trim_matches('"')
            .to_owned();
        let product_name = self
            .query(QMKProductNameQuery)?
            .0
            .trim_matches('"')
            .to_owned();
        let config = self.query_config_blob()?;
        let hardware_id = self.query(QMKHardwareIdentifierQuery)?.to_string();

        let qmk_info = QMKInfo {
            version: self.query(QMKVersionQuery)?.0.to_string(),
            board_ids,
            manufacturer,
            product_name,
            config: serde_json::to_string_pretty(&config).unwrap(),
            hardware_id,
            jump_to_bootloader_enabled: qmk_caps.contains(QMKCapabilities::JUMP_TO_BOOTLOADER),
            eeprom_reset_enabled: qmk_caps.contains(QMKCapabilities::EEPROM_RESET),
        };

        let keymap_info = if subsystems.contains(XAPEnabledSubsystems::KEYMAP) {
            let keymap_caps = self.query(KeymapCapabilitiesQuery)?;

            let layer_count = if keymap_caps.contains(KeymapCapabilities::LAYER_COUNT) {
                Some(self.query(KeymapLayerCountQuery)?.0)
            } else {
                None
            };

            // TODO ugly bodge
            let matrix = if let Some(value) = config.get("matrix_size") {
                serde_json::from_value(value.clone())
                    .map_err(|err| ClientError::Other(anyhow!("malformed matrix_size entry {err}")))
            } else {
                return Err(ClientError::Other(anyhow!(
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
            let keymap_caps = self.query(RemapCapabilitiesQuery)?;

            let layer_count = if keymap_caps.contains(RemapCapabilities::LAYER_COUNT) {
                Some(self.query(RemapLayerCountQuery)?.0)
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
            let lighting_caps = self.query(LightingCapabilitiesQuery)?;

            let backlight_info = if lighting_caps.contains(LightingCapabilities::BACKLIGHT) {
                let backlight_caps = self.query(BacklightCapabilitiesQuery)?;

                let effects = if backlight_caps.contains(BacklightCapabilities::ENABLED_EFFECTS) {
                    Some(self.query(BacklightEffectsQuery)?.enabled_effect_list())
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
                let rgblight_caps = self.query(RGBLightCapabilitiesQuery)?;

                let effects = if rgblight_caps.contains(RGBLightCapabilities::ENABLED_EFFECTS) {
                    Some(self.query(RGBLightEffectsQuery)?.enabled_effect_list())
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
                let rgbmatrix_caps = self.query(RGBMatrixCapabilitiesQuery)?;

                let effects = if rgbmatrix_caps.contains(RGBMatrixCapabilities::ENABLED_EFFECTS) {
                    Some(self.query(RGBMatrixEffectsQuery)?.enabled_effect_list())
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

        self.state.write().xap_info = Some(XAPDeviceInfo {
            xap: xap_info,
            qmk: qmk_info,
            keymap: keymap_info,
            remap: remap_info,
            lighting: lighting_info,
        });

        Ok(())
    }

    fn query_config_blob(&self) -> ClientResult<Map<String, Value>> {
        // Query data size
        let size = self.query(QMKConfigBlobLengthQuery {})?.0;

        // Query all chunks and merge them in a Vec
        let mut data: Vec<u8> = Vec::with_capacity(size as usize);
        let mut offset: u16 = 0;
        while offset < size {
            let chunk = self.query(ConfigBlobChunkQuery(offset))?;
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

    fn query_keymap(&self) -> ClientResult<()> {
        // Reset keymap
        self.state.write().keymap = Default::default();

        if let Some(keymap) = &self.xap_info().keymap {
            let layers = keymap.layer_count.unwrap_or_default();
            let cols = keymap.matrix.cols;
            let rows = keymap.matrix.rows;

            let keymap: Result<Vec<Vec<Vec<XAPKeyCodeConfig>>>, ClientError> = (0..layers)
                .map(|layer| {
                    (0..rows)
                        .map(|row| {
                            (0..cols)
                                .map(|col| {
                                    let keycode =
                                        self.query_keycode(KeyPosition { layer, row, col })?;

                                    let xap = XAPKeyCodeConfig {
                                        code: self.constants.get_keycode(keycode.0),
                                        position: KeyPosition { layer, row, col },
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

    fn get_coords_from_rowcol(&self, row: u8, col: u8) -> Option<KeyCoords> {
        let json: Map<String, Value> =
            serde_json::from_str(self.xap_info().qmk.config.as_str()).ok()?;

        // TODO: Dynamic layout name
        let layout_info = json
            .get("layouts")?
            .get("LAYOUT")?
            .get("layout")?
            .as_array()?;

        // TODO: Handle JSONs that dont contain matrix info (?)
        let Some(key) = layout_info.iter().find(|&key| {
                key.get("matrix")
                    .unwrap()
                    .as_array()
                    .unwrap() == &[row, col]
            }) else {
                debug!("There's no key at matrix ({row}, {col})");
                return None;
            };

        let x = key.get("x")?.as_u64()? as u8;
        let y = key.get("y")?.as_u64()? as u8;
        let w = if let Some(w) = key.get("w") {
            w.as_u64()? as u8
        } else {
            1
        };
        let h = if let Some(h) = key.get("h") {
            h.as_u64()? as u8
        } else {
            1
        };

        debug!("Matrix ({row}, {col}) -> Position ({x}, {y})");

        Some(KeyCoords { x, y, w, h })
    }

    fn get_rowcol_from_coords(&self, x: u8, y: u8) -> Option<KeyPosition> {
        for (row, values) in self.coords_from_rowcol().iter().enumerate() {
            for (col, value) in values.iter().enumerate() {
                if let Some(key) = value {
                    if key.x == x && key.y == y {
                        let row = row as u8;
                        let col = col as u8;
                        return Some(KeyPosition { layer: 0, row, col });
                    }
                }
            }
        }

        None
    }

    fn generate_coords_from_rowcol(&self) -> ClientResult<()> {
        self.state.write().coords_from_rowcol = Default::default();

        if let Some(keymap) = &self.xap_info().keymap {
            let cols = keymap.matrix.cols;
            let rows = keymap.matrix.rows;

            let coords_from_rowcol: Vec<Vec<Option<KeyCoords>>> = (0..rows)
                .map(|row| {
                    (0..cols)
                        .map(|col| self.get_coords_from_rowcol(row, col))
                        .collect()
                })
                .collect();

            self.state.write().coords_from_rowcol = coords_from_rowcol;
        }

        Ok(())
    }

    fn generate_key_info(&self) {
        let coords = self.coords_from_rowcol();

        info!("{}||{}", coords.len(), coords[0].len());

        let flat_coords: Vec<_> = coords.iter().flatten().flatten().collect();

        let max_x = flat_coords.iter().max_by_key(|coord| coord.x).unwrap().x;
        let max_y = flat_coords.iter().max_by_key(|coord| coord.y).unwrap().y;

        let keymap = self.keymap();
        let layers = keymap.len();

        let key_info: Vec<Vec<Vec<Option<XAPKeyInfo>>>> = (0..layers)
            .map(|layer| {
                (0..=max_y)
                    .map(|y| {
                        (0..=max_x)
                            .map(|x| {
                                let mut position = self.get_rowcol_from_coords(x, y)?;
                                position.layer = layer as u8;

                                let KeyPosition { layer: _, row, col } = position;

                                let keycode =
                                    keymap[layer][row as usize][col as usize].code.clone();

                                Some(XAPKeyInfo {
                                    coords: coords[row as usize][col as usize].clone()?,
                                    keycode,
                                    position,
                                })
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect();

        self.state.write().key_info = key_info;
    }
}

fn start_rx_thread(
    id: Uuid,
    state: Arc<RwLock<XAPDeviceState>>,
    rx: HidDevice,
    event_channel: Sender<XAPEvent>,
    tx_channel: Sender<RawResponse>,
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
            if let Err(err) = handle_report(id, &state, report, &tx_channel, &event_channel) {
                error!("handling response failed: {err}")
            }
        }
        info!("terminating capture thread for {id}");
    })
}

fn handle_report(
    id: Uuid,
    state: &Arc<RwLock<XAPDeviceState>>,
    report: [u8; XAP_REPORT_SIZE],
    tx_channel: &Sender<RawResponse>,
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
                    .send(XAPEvent::LogReceived { id, log: log.0 })
                    .expect("failed to send broadcast event!");
            }
            BroadcastType::SecureStatus => {
                let secure_status: SecureStatusBroadcast = broadcast.into_xap_broadcast()?;
                state.write().secure_status = secure_status.0;
                event_channel
                    .send(XAPEvent::SecureStatusChanged {
                        id,
                        secure_status: secure_status.0,
                    })
                    .expect("failed to send broadcast event!");
            }
            BroadcastType::Keyboard => info!("keyboard broadcasts are not implemented!"),
            BroadcastType::User => info!("keyboard broadcasts are not implemented!"),
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
