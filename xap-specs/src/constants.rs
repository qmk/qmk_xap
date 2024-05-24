pub mod keycode;
pub mod lighting;

use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;
use specta::Type;

use self::keycode::{read_xap_keycodes, KeyCode, XapKeyCodeCategory};
use self::lighting::{read_xap_lighting_effects, LightingEffects};

#[derive(Debug, Clone, Serialize, Type)]
pub struct XapConstants {
    pub keycodes: Vec<XapKeyCodeCategory>,
    pub rgblight_modes: LightingEffects,
    pub rgb_matrix_modes: LightingEffects,
    pub led_matrix_modes: LightingEffects,
}

impl XapConstants {
    pub fn new(specs_path: PathBuf) -> Result<Self> {
        Ok(Self {
            keycodes: read_xap_keycodes(&specs_path)?,
            rgblight_modes: read_xap_lighting_effects(&specs_path, "rgblight")?,
            rgb_matrix_modes: read_xap_lighting_effects(&specs_path, "rgb_matrix")?,
            led_matrix_modes: read_xap_lighting_effects(&specs_path, "led_matrix")?,
        })
    }

    pub fn get_keycode(&self, code: u16) -> KeyCode {
        for category in &self.keycodes {
            if let Some(code) = category.codes.iter().find(|keycode| keycode.code == code) {
                return code.clone();
            }
        }
        KeyCode::new_custom(code)
    }
}
