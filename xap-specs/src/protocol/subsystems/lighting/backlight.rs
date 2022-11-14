use binrw::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::request::XAPRequest;

// ==============================
// 0x6 0x2 0x1
bitflags! {
    #[binread]
    pub struct BacklightCapabilities: u32 {
        const CAPABILITIES = 1 << 0x1;
        const ENABLED_EFFECTS = 1 << 0x2;
        const GET_CONFIG = 1 << 0x3;
        const SET_CONFIG = 1 << 0x4;
        const SAVE_CONFIG = 1 << 0x5;
    }
}

#[derive(BinWrite, Debug)]
pub struct BacklightCapabilitiesQuery;

impl XAPRequest for BacklightCapabilitiesQuery {
    type Response = BacklightCapabilities;

    fn id() -> &'static [u8] {
        &[0x6, 0x2, 0x1]
    }
}

// ==============================
// 0x6 0x2 0x2
#[derive(BinRead, Debug)]
pub struct BacklightEffects(u8);

impl BacklightEffects {
    pub fn enabled_effect_list(&self) -> Vec<u8> {
        let mut effects = Vec::with_capacity(64);

        let bits = self.0;

        for i in 0..8 {
            if ((bits >> i) & 1) == 1 {
                effects.push(i)
            }
        }

        effects
    }
}

#[derive(BinWrite, Debug)]
pub struct BacklightEffectsQuery;

impl XAPRequest for BacklightEffectsQuery {
    type Response = BacklightEffects;

    fn id() -> &'static [u8] {
        &[0x6, 0x2, 0x2]
    }
}

// ==============================
// 0x6 0x2 0x3
#[derive(BinWrite, BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
#[ts(export_to = "../bindings/")]
pub struct BacklightConfig {
    pub enable: u8,
    pub mode: u8,
    pub val: u8,
}

#[derive(BinWrite, Debug)]
pub struct BacklightConfigGet;

impl XAPRequest for BacklightConfigGet {
    type Response = BacklightConfig;

    fn id() -> &'static [u8] {
        &[0x6, 0x2, 0x3]
    }
}

// ==============================
// 0x6 0x2 0x4
#[derive(BinWrite, Debug)]
pub struct BacklightConfigSet {
    pub config: BacklightConfig,
}

impl XAPRequest for BacklightConfigSet {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x2, 0x4]
    }
}

// ==============================
// 0x6 0x2 0x5
#[derive(BinWrite, Debug)]
pub struct BacklightConfigSave;

impl XAPRequest for BacklightConfigSave {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x2, 0x5]
    }
}
