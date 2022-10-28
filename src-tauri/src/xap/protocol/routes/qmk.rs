// QMK SUBSYSTEM - INCOMPLETE!

use super::route_imports::*;

#[derive(BinRead, Debug)]
pub struct QMKVersion(u32);

#[derive(BinWrite, Debug)]
pub struct QMKVersionQuery;

impl XAPRequest for QMKVersionQuery {
    type Response = QMKVersion;

    fn id() -> &'static [u8] {
        &[0x1, 0x0]
    }
}

#[derive(BinRead, Debug)]
pub struct QMKCapabilities(u32);

#[derive(BinWrite, Debug)]
pub struct QMKCapabilitiesQuery;

impl XAPRequest for QMKCapabilitiesQuery {
    type Response = QMKCapabilities;

    fn id() -> &'static [u8] {
        &[0x1, 0x1]
    }
}