use super::route_imports::*;

#[derive(BinWrite, BinRead, Debug, TS, Serialize)]
#[ts(export)]
pub struct RGBLightConfig {
    pub enable: u8,
    pub mode: u8,
    pub hue: u8,
    pub sat: u8,
    pub val: u8,
    pub speed: u8,
}

#[derive(BinWrite, Debug)]
pub struct RGBLightConfigQuery;

impl XAPRequest for RGBLightConfigQuery {
    type Response = RGBLightConfig;

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x3]
    }
}

#[derive(BinWrite, Debug)]
pub struct RGBLightConfigCommand {
    pub config: RGBLightConfig,
}

impl XAPRequest for RGBLightConfigCommand {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x4]
    }
}
