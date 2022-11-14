use binrw::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::request::XAPRequest;

// ==============================
// 0x6 0x3 0x1
bitflags! {
    #[binread]
    pub struct RGBLightCapabilities: u32 {
        const CAPABILITIES = 1 << 0x1;
        const ENABLED_EFFECTS = 1 << 0x2;
        const GET_CONFIG = 1 << 0x3;
        const SET_CONFIG = 1 << 0x4;
        const SAVE_CONFIG = 1 << 0x5;
    }
}

#[derive(BinWrite, Debug)]
pub struct RGBLightCapabilitiesQuery;

impl XAPRequest for RGBLightCapabilitiesQuery {
    type Response = RGBLightCapabilities;

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x1]
    }
}

// ==============================
// 0x6 0x3 0x2
#[derive(BinRead, Debug)]
pub struct RGBLightEffects(u64);

impl RGBLightEffects {
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
pub struct RGBLightEffectsQuery;

impl XAPRequest for RGBLightEffectsQuery {
    type Response = RGBLightEffects;

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x2]
    }
}

// ==============================
// 0x6 0x3 0x3
#[derive(BinWrite, BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "../bindings/")]
pub struct RGBLightConfig {
    pub enable: u8,
    pub mode: u8,
    pub hue: u8,
    pub sat: u8,
    pub val: u8,
    pub speed: u8,
}

#[derive(BinWrite, Debug)]
pub struct RGBLightConfigGet;

impl XAPRequest for RGBLightConfigGet {
    type Response = RGBLightConfig;

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x3]
    }
}

// ==============================
// 0x6 0x3 0x4
#[derive(BinWrite, Debug)]
pub struct RGBLightConfigSet {
    pub config: RGBLightConfig,
}

impl XAPRequest for RGBLightConfigSet {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x4]
    }
}

// ==============================
// 0x6 0x3 0x5
#[derive(BinWrite, Debug)]
pub struct RGBLightConfigSave;

impl XAPRequest for RGBLightConfigSave {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x5]
    }
}
