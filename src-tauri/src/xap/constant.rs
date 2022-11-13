use std::collections::HashMap;

use serde::Serialize;

use self::keycode::{read_xap_keycodes, XAPKeyCode};

use super::XAPResult;

pub mod keycode;

#[derive(Debug, Clone, Serialize)]
pub struct XAPConstants {
    pub keycodes: HashMap<u16, XAPKeyCode>,
}

impl XAPConstants {
    pub fn new() -> XAPResult<Self> {
        Ok(Self {
            keycodes: read_xap_keycodes()?,
        })
    }
}
