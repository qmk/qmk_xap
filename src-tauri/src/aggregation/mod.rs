pub mod config;

use serde::Serialize;
use specta::Type;
use xap_specs::{constants::keycode::XapKeyCode, constants::lighting::LightingEffect};

use crate::xap::spec::qmk::QmkBoardIdentifiersResponse;

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
    pub hardware_id: String,
    pub jump_to_bootloader_enabled: bool,
    pub eeprom_reset_enabled: bool,
}

#[derive(Debug, Serialize, Clone, Type)]
pub struct KeymapInfo {
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
