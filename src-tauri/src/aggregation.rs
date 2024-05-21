use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;
use xap_specs::{
    constants::keycode::{XapKeyCode, XapKeyCodeConfig},
    constants::lighting::{LightingEffect, LightingEffects},
    XapSecureStatus,
};

use crate::xap_spec::qmk::QmkBoardIdentifiersResponse;

#[derive(Clone, Serialize, Type)]
pub struct XapDevice {
    pub id: Uuid,
    pub info: XapDeviceInfo,
    pub keymap: Vec<Vec<Vec<XapKeyCodeConfig>>>,
    pub secure_status: XapSecureStatus,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct XapDeviceInfo {
    pub xap: XapInfo,
    pub qmk: QmkInfo,
    pub keymap: Option<KeymapInfo>,
    pub remap: Option<RemapInfo>,
    pub lighting: Option<LightingInfo>,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct XapInfo {
    pub version: u32,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct QmkInfo {
    pub version: String,
    pub board_ids: QmkBoardIdentifiersResponse,
    pub manufacturer: String,
    pub product_name: String,
    pub config: String,
    pub hardware_id: String,
    pub jump_to_bootloader_enabled: bool,
    pub eeprom_reset_enabled: bool,
}

#[derive(Deserialize, Debug, Serialize, Clone, Type)]
pub struct Matrix {
    pub cols: u8,
    pub rows: u8,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct KeymapInfo {
    pub matrix: Matrix,
    pub layer_count: Option<u8>,
    pub get_keycode_enabled: bool,
    pub get_encoder_keycode_enabled: bool,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct RemapInfo {
    pub layer_count: Option<u8>,
    pub set_keycode_enabled: bool,
    pub set_encoder_keycode_enabled: bool,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct LightingInfo {
    pub backlight: Option<LightingCapabilities>,
    pub rgblight: Option<LightingCapabilities>,
    pub rgbmatrix: Option<LightingCapabilities>,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct LightingCapabilities {
    pub effects: Vec<LightingEffect>,
    pub get_config_enabled: bool,
    pub set_config_enabled: bool,
    pub save_config_enabled: bool,
}

impl LightingCapabilities {
    pub fn new(
        mut effects: Vec<LightingEffect>,
        get_config_enabled: bool,
        set_config_enabled: bool,
        save_config_enabled: bool,
    ) -> Self {
        effects.sort_by(|lhs, rhs| lhs.label.cmp(&rhs.label));

        Self {
            effects,
            get_config_enabled,
            set_config_enabled,
            save_config_enabled,
        }
    }
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct XapKeyCodeCategory {
    name: String,
    codes: Vec<XapKeyCode>,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct XapConstants {
    keycodes: Vec<XapKeyCodeCategory>,
    rgblight_modes: LightingEffects,
    rgb_matrix_modes: LightingEffects,
    led_matrix_modes: LightingEffects,
}

impl From<xap_specs::constants::XapConstants> for XapConstants {
    fn from(constants: xap_specs::constants::XapConstants) -> Self {
        let keycodes =
            constants
                .keycodes
                .into_iter()
                .fold(HashMap::new(), |mut category, (_, keycode)| {
                    category
                        .entry(keycode.group.clone().unwrap_or("other".to_owned()))
                        .or_insert(Vec::new())
                        .push(keycode);

                    category
                });

        let keycodes = keycodes
            .into_iter()
            .map(|(name, mut codes)| {
                codes.sort_by_key(|code| code.code);
                XapKeyCodeCategory { name, codes }
            })
            .collect();

        Self {
            keycodes,
            rgblight_modes: constants.rgblight_modes,
            rgb_matrix_modes: constants.rgb_matrix_modes,
            led_matrix_modes: constants.led_matrix_modes,
        }
    }
}
