use binrw::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::xap::XAPRequest;

// ==============================
// 0x6 0x2 0x1
#[derive(BinRead, Debug)]
pub struct BacklightCapabilities(u32);

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

        for i in 0..64 {
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
