use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use ts_rs::TS;
use uuid::Uuid;

use crate::xap::{keycode::XAPKeyCode, KeyPositionConfig, QMKBoardIdentifiers, XAPSecureStatus};

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct XAPDevice {
    pub id: Uuid,
    pub info: XAPDeviceInfo,
    pub keymap: Vec<Vec<Vec<KeyPositionConfig>>>,
    pub secure_status: XAPSecureStatus,
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
    pub version: String,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct QMKInfo {
    pub version: String,
    pub board_ids: QMKBoardIdentifiers,
    pub manufacturer: String,
    pub product_name: String,
    pub config: String,
    pub hardware_id: String,
    pub jump_to_bootloader_enabled: bool,
    pub eeprom_reset_enabled: bool,
}

#[derive(Deserialize, Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct Matrix {
    pub cols: u8,
    pub rows: u8,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct KeymapInfo {
    pub matrix: Matrix,
    pub layer_count: Option<u8>,
    pub get_keycode_enabled: bool,
    pub get_encoder_keycode_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RemapInfo {
    pub layer_count: Option<u8>,
    pub set_keycode_enabled: bool,
    pub set_encoder_keycode_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct LightingInfo {
    pub backlight: Option<BacklightInfo>,
    pub rgblight: Option<RGBLightInfo>,
    pub rgbmatrix: Option<RGBMatrixInfo>,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct BacklightInfo {
    pub effects: Option<Vec<u8>>,
    pub get_config_enabled: bool,
    pub set_config_enabled: bool,
    pub save_config_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RGBLightInfo {
    pub effects: Option<Vec<u8>>,
    pub get_config_enabled: bool,
    pub set_config_enabled: bool,
    pub save_config_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct RGBMatrixInfo {
    pub effects: Option<Vec<u8>>,
    pub get_config_enabled: bool,
    pub set_config_enabled: bool,
    pub save_config_enabled: bool,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct XAPKeyCodeCategory {
    name: String,
    codes: Vec<XAPKeyCode>,
}

#[derive(Debug, Serialize, TS, Clone)]
#[ts(export)]
pub struct XAPConstants {
    keycodes: Vec<XAPKeyCodeCategory>,
}

impl From<crate::xap::XAPConstants> for XAPConstants {
    fn from(constants: crate::xap::XAPConstants) -> Self {
        let keycodes =
            constants
                .keycodes
                .into_iter()
                .fold(HashMap::new(), |mut category, (_, keycode)| {
                    category
                        .entry(keycode.group.clone().unwrap_or("Other".to_owned()))
                        .or_insert(Vec::new())
                        .push(keycode);

                    category
                });

        let keycodes = keycodes
            .into_iter()
            .map(|(name, codes)| XAPKeyCodeCategory { name, codes })
            .collect();

        Self { keycodes }
    }
}
