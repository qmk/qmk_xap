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
#[derive(BinRead, Debug)]
pub struct RemapKeycode(u16);

#[derive(BinWrite, Debug)]
pub struct RemapKeycodeQuery;

impl XAPRequest for RemapKeycodeQuery {
    type Response = RemapKeycode;

    fn id() -> &'static [u8] {
        &[0x5, 0x3]
    }
}

// ==============================
// 0x5 0x4
#[derive(BinRead, Debug)]
pub struct RemapEncoder(u16);

#[derive(BinWrite, Debug)]
pub struct RemapEncoderQuery;

impl XAPRequest for RemapEncoderQuery {
    type Response = RemapEncoder;

    fn id() -> &'static [u8] {
        &[0x5, 0x4]
    }
}