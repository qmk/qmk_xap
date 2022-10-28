use binrw::*;
use crate::xap::XAPRequest;

// ==============================
// 0x1 0x0
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

// ==============================
// 0x1 0x1
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

// ==============================
// 0x1 0x2
#[derive(BinRead, Debug)]
pub struct QMKBoardIdentifiers {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_version: u16,
    pub qmk_identifier: u32,
}

#[derive(BinWrite, Debug)]
pub struct QMKBoardIdentifiersQuery;

impl XAPRequest for QMKBoardIdentifiersQuery {
    type Response = QMKBoardIdentifiers;

    fn id() -> &'static [u8] {
        &[0x1, 0x2]
    }
}

// ==============================
// 0x1 0x3
// TODO: Implement BinRead for String
#[derive(BinRead, Debug)]
pub struct QMKBoardManufacturer(String);

#[derive(BinWrite, Debug)]
pub struct QMKBoardManufacturerQuery;

impl XAPRequest for QMKBoardManufacturerQuery {
    type Response = QMKBoardManufacturer;

    fn id() -> &'static [u8] {
        &[0x1, 0x3]
    }
}

// ==============================
// 0x1 0x4
// TODO: Implement BinRead for String
#[derive(BinRead, Debug)]
pub struct QMKProductName(String);

#[derive(BinWrite, Debug)]
pub struct QMKProductNameQuery;

impl XAPRequest for QMKProductNameQuery {
    type Response = QMKProductName;

    fn id() -> &'static [u8] {
        &[0x1, 0x4]
    }
}

// ==============================
// 0x1 0x5
#[derive(BinRead, Debug)]
pub struct QMKConfigBlobLength(u16);

#[derive(BinWrite, Debug)]
pub struct QMKConfigBlobLengthQuery;

impl XAPRequest for QMKConfigBlobLengthQuery {
    type Response = QMKConfigBlobLength;

    fn id() -> &'static [u8] {
        &[0x1, 0x5]
    }
}

// ==============================
// 0x1 0x6
#[derive(BinRead, Debug)]
pub struct ConfigBlobChunk([u8; 32]);

#[derive(BinWrite, Debug)]
pub struct ConfigBlobChunkQuery;

impl XAPRequest for ConfigBlobChunkQuery {
    type Response = ConfigBlobChunk;

    fn id() -> &'static [u8] {
        &[0x1, 0x6]
    }
}

// ==============================
// 0x1 0x7
#[derive(BinRead, Debug)]
pub struct QMKJumpToBootloader(u8);

#[derive(BinWrite, Debug)]
pub struct QMKJumpToBootloaderQuery;

impl XAPRequest for QMKJumpToBootloaderQuery {
    type Response = QMKJumpToBootloader;

    fn id() -> &'static [u8] {
        &[0x1, 0x7]
    }
}

// ==============================
// 0x1 0x8
#[derive(BinRead, Debug)]
pub struct QMKHardwareIdentifier([u32;4]);

#[derive(BinWrite, Debug)]
pub struct QMKHardwareIdentifierQuery;

impl XAPRequest for QMKHardwareIdentifierQuery {
    type Response = QMKHardwareIdentifier;

    fn id() -> &'static [u8] {
        &[0x1, 0x8]
    }
}

// ==============================
// 0x1 0x9
#[derive(BinRead, Debug)]
pub struct QMKReinitializeEeprom(u8);

#[derive(BinWrite, Debug)]
pub struct QMKReinitializeEepromQuery;

impl XAPRequest for QMKReinitializeEepromQuery {
    type Response = QMKReinitializeEeprom;

    fn id() -> &'static [u8] {
        &[0x1, 0x9]
    }
}