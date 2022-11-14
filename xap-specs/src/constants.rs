pub mod keycode;

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;

use crate::constants::keycode::{read_xap_keycodes, XAPKeyCode};
use crate::error::XAPResult;

#[derive(Debug, Clone, Serialize)]
pub struct XAPConstants {
    pub keycodes: HashMap<u16, XAPKeyCode>,
}

impl XAPConstants {
    pub fn new(specs_path: PathBuf) -> XAPResult<Self> {
        Ok(Self {
            keycodes: read_xap_keycodes(specs_path)?,
        })
    }

    pub fn get_keycode(&self, code: u16) -> XAPKeyCode {
        self.keycodes
            .get(&code)
            .cloned()
            .unwrap_or_else(|| XAPKeyCode::new_custom(code))
    }
}
