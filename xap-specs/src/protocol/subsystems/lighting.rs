// Aggregator of the different lighting modes controlled by XAP, each one on its own file + Capabilities query

use binrw::*;
use bitflags::bitflags;

use crate::request::XAPRequest;

mod backlight;
mod rgblight;
mod rgbmatrix;

pub use backlight::*;
pub use rgblight::*;
pub use rgbmatrix::*;

// ==============================
// 0x6 0x1
bitflags! {
    #[binread]
    pub struct LightingCapabilities: u32 {
        const BACKLIGHT = 1 << 0x2;
        const RGBLIGHT = 1 << 0x3;
        const RGBMATRIX = 1 << 0x4;
    }
}

#[derive(BinWrite, Debug)]
pub struct LightingCapabilitiesQuery;

impl XAPRequest for LightingCapabilitiesQuery {
    type Response = LightingCapabilities;

    fn id() -> &'static [u8] {
        &[0x6, 0x1]
    }
}
