use binrw::*;
use bitflags::bitflags;

use crate::xap::XAPRequest;

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
#[derive(BinRead, Debug)]
pub struct Keycode(u16);

#[derive(BinWrite, Debug)]
pub struct KeymapKeycodeQuery;

impl XAPRequest for KeymapKeycodeQuery {
    type Response = Keycode;

    fn id() -> &'static [u8] {
        &[0x4, 0x3]
    }
}

// ==============================
// 0x4 0x4
#[derive(BinWrite, Debug)]
pub struct KeymapEncoderQuery;

impl XAPRequest for KeymapEncoderQuery {
    type Response = Keycode;

    fn id() -> &'static [u8] {
        &[0x4, 0x4]
    }
}
