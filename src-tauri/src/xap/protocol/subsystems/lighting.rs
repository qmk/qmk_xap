// Aggregator of the different lighting modes controlled by XAP, each one on its own file + Capabilities query

use binrw::*;
use crate::xap::XAPRequest;

mod backlight;
mod rgblight;
mod rgbmatrix;

pub use backlight::*;
pub use rgblight::*;
pub use rgbmatrix::*;

#[derive(BinRead, Debug)]
pub struct LightingCapabilities(u32);

#[derive(BinWrite, Debug)]
pub struct LightingCapabilitiesQuery;

impl XAPRequest for LightingCapabilitiesQuery {
    type Response = LightingCapabilities;

    fn id() -> &'static [u8] {
        &[0x6, 0x1]
    }
}