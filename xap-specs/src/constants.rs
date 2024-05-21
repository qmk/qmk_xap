pub mod keycode;
pub mod lighting;

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;

use crate::error::XapResult;

use self::keycode::{read_xap_keycodes, XapKeyCode};
use self::lighting::{read_xap_lighting_effects, LightingEffects};

#[derive(Debug, Clone, Serialize)]
pub struct XapConstants {
    pub keycodes: HashMap<u16, XapKeyCode>,
    pub rgblight_modes: LightingEffects,
    pub rgb_matrix_modes: LightingEffects,
    pub led_matrix_modes: LightingEffects,
}

impl XapConstants {
    pub fn new(specs_path: PathBuf) -> XapResult<Self> {
        Ok(Self {
            keycodes: read_xap_keycodes(&specs_path)?,
            rgblight_modes: read_xap_lighting_effects(&specs_path, "rgblight")?,
            rgb_matrix_modes: read_xap_lighting_effects(&specs_path, "rgb_matrix")?,
            led_matrix_modes: read_xap_lighting_effects(&specs_path, "led_matrix")?,
        })
    }

    pub fn get_keycode(&self, code: u16) -> XapKeyCode {
        self.keycodes
            .get(&code)
            .cloned()
            .unwrap_or_else(|| XapKeyCode::new_custom(code))
    }
}
