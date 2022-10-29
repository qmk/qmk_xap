use binrw::*;
use crate::xap::XAPRequest;

// ==============================
// 0x4 0x1
#[derive(BinRead, Debug)]
pub struct KeymapCapabilities(u32);

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
pub struct KeymapLayerCount(u8);

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