use binrw::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::request::XAPRequest;

// ==============================
// 0x4 0x1
bitflags! {
    #[binread]
    pub struct KeymapCapabilities: u32 {
        const CAPABILITIES = 1 << 0x1;
        const LAYER_COUNT = 1 << 0x2;
        const GET_KEYCODE = 1 << 0x3;
        const GET_ENCODER_KEYCODE = 1 << 0x4;
    }
}

#[derive(BinWrite, Debug)]
pub struct KeymapCapabilitiesQuery;

impl XAPRequest for KeymapCapabilitiesQuery {
    type Response = KeymapCapabilities;

    fn id() -> &'static [u8] {
        &[0x4, 0x1]
    }
}

// ==============================
// 0x4 0x2
#[derive(BinRead, Debug)]
pub struct KeymapLayerCount(pub u8);

#[derive(BinWrite, Debug)]
pub struct KeymapLayerCountQuery;

impl XAPRequest for KeymapLayerCountQuery {
    type Response = KeymapLayerCount;

    fn id() -> &'static [u8] {
        &[0x4, 0x2]
    }
}

// ==============================
// 0x4 0x3
#[derive(BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "../bindings/")]
pub struct KeyCode(pub u16);

#[derive(BinWrite, Debug, Default, Clone, TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "../bindings/")]
pub struct KeyPosition {
    pub layer: u8,
    pub row: u8,
    pub col: u8,
}

#[derive(BinWrite, Debug)]
pub struct KeymapKeycodeQuery(pub KeyPosition);

impl XAPRequest for KeymapKeycodeQuery {
    type Response = KeyCode;

    fn id() -> &'static [u8] {
        &[0x4, 0x3]
    }
}

// ==============================
// 0x4 0x4
#[derive(BinWrite, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "../bindings/")]
pub struct EncoderPosition {
    layer: u8,
    encoder: u8,
    clockwise: u8,
}

#[derive(BinWrite, Debug)]
pub struct KeymapEncoderQuery(pub EncoderPosition);

impl XAPRequest for KeymapEncoderQuery {
    type Response = KeyCode;

    fn id() -> &'static [u8] {
        &[0x4, 0x4]
    }
}
