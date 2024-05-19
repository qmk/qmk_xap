pub mod keycode;

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;

use crate::constants::keycode::{read_xap_keycodes, XapKeyCode};
use crate::error::XapResult as XapResult;

#[derive(Debug, Clone, Serialize)]
pub struct XapConstants {
    pub keycodes: HashMap<u16, XapKeyCode>,
}

impl XapConstants {
    pub fn new(specs_path: PathBuf) -> XapResult<Self> {
        Ok(Self {
            keycodes: read_xap_keycodes(specs_path)?,
        })
    }

    pub fn get_keycode(&self, code: u16) -> XapKeyCode {
        self.keycodes
            .get(&code)
            .cloned()
            .unwrap_or_else(|| XapKeyCode::new_custom(code))
    }
}
