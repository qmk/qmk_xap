use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    fmt::Debug,
    io::{Cursor, Read},
    sync::Arc,
    time::{Duration, Instant},
    vec,
};

use anyhow::{anyhow, Result};
use binrw::{BinRead, BinWriterExt};
use flate2::read::GzDecoder;
// use hidapi::{DeviceInfo, HidDevice};
use log::{info, trace};
use serde::Serialize;

use tsify::Tsify;
use uuid::Uuid;

use xap_specs::{
    broadcast::{BroadcastRaw, BroadcastType, SecureStatusBroadcast},
    constants::{keycode::KeyCode, XapConstants},
    request::{RawRequest, XapRequest},
    response::RawResponse,
    token::Token,
    XapSecureStatus,
};

use crate::{
    aggregation::{
        config::Config, keymap::MappedKeymap, KeymapInfo, LightingCapabilities, LightingInfo,
        Point2D, Point3D, QmkInfo, RemapInfo, XapDeviceInfo, XapInfo,
    },
    xap::spec::{
        keymap::{
            KeymapCapabilitiesFlags, KeymapCapabilitiesRequest, KeymapGetKeycodeRequest,
            KeymapGetLayerCountRequest,
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
};

#[derive(Clone, Debug, Serialize)]
pub struct Keymap {
    keys: Vec<Vec<Vec<KeymapKey>>>,
    dimensions: Point3D,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct KeymapKey {
    pub code: KeyCode,
    pub position: Point3D,
}

impl Keymap {
    pub fn new(layers: u64, rows: u64, columns: u64) -> Self {
        Self {
            keys: vec![
                vec![vec![KeymapKey::default(); columns as usize]; rows as usize];
                layers as usize
            ],
            dimensions: Point3D {
                z: layers,
                y: rows,
                x: columns,
            },
        }
    }

    pub fn remap_key(&mut self, key: &KeymapKey) -> Result<()> {
        if key.position.z >= self.dimensions.z
            || key.position.y >= self.dimensions.y
            || key.position.x >= self.dimensions.x
        {
            anyhow::bail!(
                "key position {:?} out of bounds for keymap with dimensions {:?}",
                key.position,
                self.dimensions
            )
        }

        self.keys[key.position.z as usize][key.position.y as usize][key.position.x as usize] =
            key.clone();

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct XapDeviceState {
    pub id: Uuid,
    pub info: Option<XapDeviceInfo>,
    #[serde(skip)]
    pub keymap: Keymap,
    pub config: Config,
    pub secure_status: XapSecureStatus,
}

const XAP_REPORT_SIZE: usize = 64;

#[derive(Debug)]
pub struct XapDevice {
    id: Uuid,
    // info: DeviceInfo,
    // hid_device: HidDevice,
    constants: Arc<XapConstants>,
    state: XapDeviceState,
    pub broadcast_queue: VecDeque<BroadcastRaw>,
    responses: HashMap<Token, Option<RawResponse>>,
}

impl XapDevice {
    pub(crate) fn new(
        // info: DeviceInfo,
        constants: Arc<XapConstants>,
        // hid_device: HidDevice,
    ) -> Result<Self> {
        // We are polling for reports, so we need to set the device to non-blocking mode otherwise
        // we will block forever in case that there is no report to read
        // hid_device.set_blocking_mode(false)?;

        let id = Uuid::new_v4();
        let state = XapDeviceState {
            id,
            info: None,
            keymap: Keymap::new(0, 0, 0),
            config: Config {
                layouts: HashMap::new(),
                matrix_size: Point2D { x: 0, y: 0 },
            },
            secure_status: XapSecureStatus::Locked,
        };

        let mut device = Self {
            id,
            // info,
            // hid_device,
            state,
            constants,
            responses: HashMap::new(),
            broadcast_queue: VecDeque::new(),
        };
        device.query_device_info()?;
        device.query_keymap()?;
        device.query_secure_status()?;
        Ok(device)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn state(&self) -> &XapDeviceState {
        &self.state
    }

    pub fn xap_info(&self) -> XapDeviceInfo {
        self.state
            .info
            .clone()
            .expect("XAP device wasn't properly initialized")
    }

    pub fn keymap(&self) -> &Keymap {
        &self.state.keymap
    }

    pub fn keymap_with_layout(&self, layout: String) -> Result<MappedKeymap> {
        let layout = self
            .state
            .config
            .layouts
            .get(&layout)
            .ok_or_else(|| anyhow!("layout {layout} not found in device {}", self.id))?;

        let mut keymap = MappedKeymap::new(
            self.state.keymap.dimensions.z,
            self.state.keymap.dimensions.y,
            self.state.keymap.dimensions.x,
        );

        for (_layer, keys) in self.keymap().keys.iter().enumerate() {
            for (row, keys) in keys.iter().enumerate() {
                for (column, key) in keys.iter().enumerate() {
                    if let Some(entry) = layout.find(Point2D { x: column as u64, y: row as u64}) {
                        keymap.insert(key.clone(), entry.clone());
                    }
                }
            }
        }

        Ok(keymap)
    }

    pub fn is_hid_device(&self/* , candidate: &DeviceInfo*/) -> bool {
        true
        // candidate.path() == self.info.path()
        //     && candidate.product_id() == self.info.product_id()
        //     && candidate.vendor_id() == self.info.vendor_id()
        //     && candidate.usage_page() == self.info.usage_page()
        //     && candidate.usage() == self.info.usage()
    }

    pub fn remap_key(&mut self, key: RemappingSetKeycodeArg) -> Result<()> {
        self.query(RemappingSetKeycodeRequest(key.clone()))?;

        let keycode = self.query_key(Point3D {
            z: key.layer as u64,
            y: key.row as u64,
            x: key.column as u64,
        })?;

        self.state.keymap.remap_key(&keycode)?;

        Ok(())
    }

    pub fn query_key(&mut self, position: Point3D) -> Result<KeymapKey> {
        let code_raw = self.query(KeymapGetKeycodeRequest(position.into()))?;

        let key = KeymapKey {
            code: self.constants.get_keycode(code_raw.0),
            position,
        };

        self.state.keymap.remap_key(&key)?;

        Ok(key)
    }

    pub fn query<T: XapRequest>(&mut self, request: T) -> Result<T::Response> {
        if let Some(xap_info) = &self.state.info {
            if !T::xap_version() < xap_info.xap.version {
                return Err(anyhow!(
                    "can't do xap request [{:?}] with client of version {}",
                    T::id(),
                    xap_info.xap.version
                ));
            }
        }

        let request = RawRequest::new(request);
        let mut report = [0; XAP_REPORT_SIZE + 1];

        // Add trailing zero byte for the report Id to HID report
        let mut writer = Cursor::new(&mut report[1..]);
        writer.write_le(&request)?;

        trace!("send XAP report with payload {:?}", &report[1..]);

        self.responses.insert(request.token().clone(), None);
        // self.hid_device.write(&report)?;

        let start = Instant::now();

        loop {
            let length = self.poll()?;

            if length == 0 {
                if start.elapsed() > Duration::from_secs(5) {
                    return Err(anyhow!("timeout waiting for response to request"));
                }
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }

            if let Entry::Occupied(response) = self.responses.entry(request.token().clone()) {
                if response.get().is_none() {
                    continue;
                }

                let (_, response) = response.remove_entry();

                return response
                    .expect("response was just checked for None")
                    .into_xap_response::<T>();
            }
        }
    }

    pub fn query_secure_status(&mut self) -> Result<XapSecureStatus> {
        let status = self.query(XapSecureStatusRequest(()))?.0.into();
        self.state.secure_status = status;
        Ok(status)
    }

    fn query_device_info(&mut self) -> Result<()> {
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

        self.query_config()?;

        let hardware_id = self.query(QmkHardwareIdentifierRequest(()))?.0;

        let qmk_info = QmkInfo {
            version: self.query(QmkVersionRequest(()))?.0.to_string(),
            board_ids,
            manufacturer,
            product_name,
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

            Some(KeymapInfo {
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

        self.state.info = Some(XapDeviceInfo {
            xap: xap_info,
            qmk: qmk_info,
            keymap: keymap_info,
            remap: remap_info,
            lighting: lighting_info,
        });

        Ok(())
    }

    fn query_config(&mut self) -> Result<()> {
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

        decoder.read_to_string(&mut decompressed)?;

        self.state.config = serde_json::from_str(&decompressed)?;

        Ok(())
    }

    fn query_keymap(&mut self) -> Result<()> {
        let layers: u64 = if let Some(keymap) = &self.xap_info().keymap {
            keymap.layer_count.unwrap_or_default() as u64
        } else {
            0
        };

        let Point2D {
            x: columns,
            y: rows,
        } = self.state.config.matrix_size;

        self.state.keymap = Keymap::new(layers, rows, columns);

        for layer in 0..layers {
            for row in 0..rows {
                for column in 0..columns {
                    _ = self.query_key(Point3D {
                        z: layer,
                        y: row,
                        x: column,
                    })?;
                }
            }
        }

        Ok(())
    }

    pub fn poll(&mut self) -> Result<usize> {
        let mut report = [0_u8; XAP_REPORT_SIZE];

        // let length = self.hid_device.read(&mut report)?;
        let length = 0;

        if length == 0 {
            return Ok(0);
        }

        let mut reader = Cursor::new(&report);
        let token = Token::read_le(&mut reader)?;

        if let Token::Broadcast = token {
            let broadcast = BroadcastRaw::from_raw_report(&report)?;
            trace!("received XAP broadcast {:?}", broadcast);

            // TODO nicer way to handle this without clone?
            if matches!(broadcast.broadcast_type(), BroadcastType::SecureStatus) {
                broadcast
                    .clone()
                    .into_xap_broadcast::<SecureStatusBroadcast>()
                    .map(|broadcast| {
                        self.state.secure_status = broadcast.0;
                    })?;
            }

            self.broadcast_queue.push_back(broadcast);
        } else {
            let response = RawResponse::from_raw_report(&report)?;
            trace!(
                "received XAP package with token {:?} and payload {:#?}",
                response.token(),
                response.payload()
            );

            match self.responses.entry(token) {
                Entry::Occupied(mut request) => {
                    if request.get().is_some() {
                        trace!(
                            "received duplicate response with token {:?}, discarding",
                            response.token()
                        );
                        return Ok(0);
                    }
                    request.insert(Some(response));
                }
                Entry::Vacant(_) => {
                    trace!(
                        "received unsolicited response with token {:?}, discarding",
                        response.token()
                    );
                    return Ok(0);
                }
            }
        }

        Ok(length)
    }

    pub fn secure_status(&self) -> &XapSecureStatus {
        &self.state.secure_status
    }
}
