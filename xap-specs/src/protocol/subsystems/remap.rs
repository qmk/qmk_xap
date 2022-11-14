use binrw::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::request::XAPRequest;

// ==============================
// 0x5 0x1
bitflags! {
    #[binread]
    pub struct RemapCapabilities: u32 {
        const CAPABILITIES = 1 << 0x1;
        const LAYER_COUNT = 1 << 0x2;
        const SET_KEYCODE = 1 << 0x3;
        const SET_ENCODER_KEYCODE = 1 << 0x4;
    }
}

#[derive(BinWrite, Debug)]
pub struct RemapCapabilitiesQuery;

impl XAPRequest for RemapCapabilitiesQuery {
    type Response = RemapCapabilities;

    fn id() -> &'static [u8] {
        &[0x5, 0x1]
    }
}

// ==============================
// 0x5 0x2
#[derive(BinRead, Debug)]
pub struct RemapLayerCount(pub u8);

#[derive(BinWrite, Debug)]
pub struct RemapLayerCountQuery;

impl XAPRequest for RemapLayerCountQuery {
    type Response = RemapLayerCount;

    fn id() -> &'static [u8] {
        &[0x5, 0x2]
    }
}

// ==============================
// 0x5 0x3
#[derive(BinWrite, BinRead, Clone, Debug, Default, TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "../bindings/")]
pub struct KeyPositionConfig {
    pub layer: u8,
    pub row: u8,
    pub col: u8,
    pub keycode: u16,
}

#[derive(BinWrite, Debug)]
pub struct RemapKeycodeQuery(pub KeyPositionConfig);

impl XAPRequest for RemapKeycodeQuery {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x5, 0x3]
    }

    fn is_secure() -> bool {
        true
    }
}

// ==============================
// 0x5 0x4
#[derive(BinWrite, BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "../bindings/")]
pub struct EncoderPositionConfig {
    pub layer: u8,
    pub encoder: u8,
    pub clockwise: u8,
    pub keycode: u16,
}

#[derive(BinWrite, Debug)]
pub struct RemapEncoderQuery(pub EncoderPositionConfig);

impl XAPRequest for RemapEncoderQuery {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x5, 0x4]
    }

    fn is_secure() -> bool {
        true
    }
}
