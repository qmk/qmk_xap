use binrw::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::request::XAPRequest;

// ==============================
// 0x6 0x4 0x1
bitflags! {
    #[binread]
    pub struct RGBMatrixCapabilities: u32 {
        const CAPABILITIES = 1 << 0x1;
        const ENABLED_EFFECTS = 1 << 0x2;
        const GET_CONFIG = 1 << 0x3;
        const SET_CONFIG = 1 << 0x4;
        const SAVE_CONFIG = 1 << 0x5;
    }
}

#[derive(BinWrite, Debug)]
pub struct RGBMatrixCapabilitiesQuery;

impl XAPRequest for RGBMatrixCapabilitiesQuery {
    type Response = RGBMatrixCapabilities;

    fn id() -> &'static [u8] {
        &[0x6, 0x4, 0x1]
    }
}

// ==============================
// 0x6 0x4 0x2
#[derive(BinRead, Debug)]
pub struct RGBMatrixEffects(u64);

impl RGBMatrixEffects {
    pub fn enabled_effect_list(&self) -> Vec<u8> {
        let mut effects = Vec::with_capacity(64);

        let bits = self.0;

        for i in 0..64 {
            if ((bits >> i) & 1) == 1 {
                effects.push(i)
            }
        }

        effects
    }
}

#[derive(BinWrite, Debug)]
pub struct RGBMatrixEffectsQuery;

impl XAPRequest for RGBMatrixEffectsQuery {
    type Response = RGBMatrixEffects;

    fn id() -> &'static [u8] {
        &[0x6, 0x4, 0x2]
    }
}

// ==============================
// 0x6 0x4 0x3
#[derive(BinWrite, BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "../bindings/")]
pub struct RGBMatrixConfig {
    pub enable: u8,
    pub mode: u8,
    pub hue: u8,
    pub sat: u8,
    pub val: u8,
    pub speed: u8,
    pub flags: u8,
}

#[derive(BinWrite, Debug)]
pub struct RGBMatrixConfigGet;

impl XAPRequest for RGBMatrixConfigGet {
    type Response = RGBMatrixConfig;

    fn id() -> &'static [u8] {
        &[0x6, 0x4, 0x3]
    }
}

// ==============================
// 0x6 0x4 0x4
#[derive(BinWrite, Debug)]
pub struct RGBMatrixConfigSet {
    pub config: RGBMatrixConfig,
}

impl XAPRequest for RGBMatrixConfigSet {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x4, 0x4]
    }
}

// ==============================
// 0x6 0x4 0x5
#[derive(BinWrite, Debug)]
pub struct RGBMatrixConfigSave;

impl XAPRequest for RGBMatrixConfigSave {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x4, 0x5]
    }
}
