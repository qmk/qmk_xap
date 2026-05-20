pub mod keycode;
pub mod lighting;

use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;
use specta::Type;

use self::keycode::{read_xap_keycode_catalog, KeyCode, XapKeyCodeCatalog, XapKeyCodeCategory};
use self::lighting::{read_xap_lighting_effects, LightingEffects};

#[derive(Debug, Clone, Serialize, Type)]
pub struct XapConstants {
    pub keycode_version: String,
    pub keycode_versions: Vec<String>,
    pub keycodes: Vec<XapKeyCodeCategory>,
    pub rgblight_modes: LightingEffects,
    pub rgb_matrix_modes: LightingEffects,
    pub led_matrix_modes: LightingEffects,
    #[serde(skip)]
    #[specta(skip)]
    keycode_catalog: XapKeyCodeCatalog,
}

impl XapConstants {
    pub fn new(specs_path: PathBuf) -> Result<Self> {
        let keycode_catalog = read_xap_keycode_catalog(&specs_path)?;

        Ok(Self {
            keycode_version: keycode_catalog.latest.clone(),
            keycode_versions: keycode_catalog.versions.clone(),
            keycodes: keycode_catalog.latest_keycodes(),
            rgblight_modes: read_xap_lighting_effects(&specs_path, "rgblight")?,
            rgb_matrix_modes: read_xap_lighting_effects(&specs_path, "rgb_matrix")?,
            led_matrix_modes: read_xap_lighting_effects(&specs_path, "led_matrix")?,
            keycode_catalog,
        })
    }

    pub fn get_keycode(&self, code: u16) -> KeyCode {
        self.get_keycode_for_version(None, code)
    }

    pub fn get_keycode_for_version(&self, version: Option<&str>, code: u16) -> KeyCode {
        self.keycode_catalog.get_keycode(version, code)
    }
}
