// RGBLIGHT SUBSYSTEM - INCOMPLETE!

use super::route_imports::*;

#[derive(BinWrite, BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct RGBConfig {
    pub enable: u8,
    pub mode: u8,
    pub hue: u8,
    pub sat: u8,
    pub val: u8,
    pub speed: u8,
}

#[derive(BinRead, Debug)]
pub struct RGBLightCapabilities(u32);

#[derive(BinWrite, Debug)]
pub struct RGBLightCapabilitiesQuery;

impl XAPRequest for RGBLightCapabilitiesQuery {
    type Response = RGBConfig;

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x1]
    }
}

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

#[derive(BinWrite, Debug)]
pub struct RGBLightConfigGet;

impl XAPRequest for RGBLightConfigGet {
    type Response = RGBConfig;

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x3]
    }
}

#[derive(BinWrite, Debug)]
pub struct RGBLightConfigSet {
    pub config: RGBConfig,
}

impl XAPRequest for RGBLightConfigSet {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x4]
    }
}

#[derive(BinWrite, Debug)]
pub struct RGBLightConfigSave;

impl XAPRequest for RGBLightConfigSave {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x5]
    }
}
