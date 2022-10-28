use binrw::*;
use crate::xap::XAPRequest;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ==============================
// 0x5 0x1
#[derive(BinRead, Debug)]
pub struct RemapCapabilities(u32);

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
pub struct RemapLayerCount(u8);

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
#[derive(BinWrite, BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct RemapKeycodeConfig {
    pub layer: u8,
    pub row: u8,
    pub column: u8,
    pub keycode: u16,
}

#[derive(BinWrite, Debug)]
pub struct RemapKeycodeQuery {
    pub config: RemapKeycodeConfig,
}

impl XAPRequest for RemapKeycodeQuery {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x5, 0x3]
    }
}

// ==============================
// 0x5 0x4
#[derive(BinWrite, BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct RemapEncoderConfig {
    pub layer: u8,
    pub encoder: u8,
    pub clockwise: u8,
    pub keycode: u16,
}

#[derive(BinWrite, Debug)]
pub struct RemapEncoderQuery {
    config: RemapEncoderConfig,
}

impl XAPRequest for RemapEncoderQuery {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x5, 0x4]
    }
}