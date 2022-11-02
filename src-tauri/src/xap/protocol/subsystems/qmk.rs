use std::ffi::CStr;

use binrw::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
bitflags! {
    #[binread]
    pub struct QMKCapabilities: u32 {
        const VERSION = 1 << 0x0;
        const CAPABILITIES = 1 << 0x1;
        const BOARD_ID = 1 << 0x2;
        const BOARD_MANUFACTURER = 1 << 0x3;
        const PRODUCT_NAME = 1 << 0x4;
        const CONFIG_BLOB_LENGTH = 1 << 0x5;
        const CONFIG_BLOB_CHUNK = 1 << 0x6;
        const JUMP_TO_BOOTLOADER = 1 << 0x7;
        const HARDWARE_ID = 1 << 0x8;
        const EEPROM_RESET = 1 << 0x9;
    }
}

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
#[derive(BinRead, Debug, Serialize, TS)]
#[ts(export)]
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
#[derive(Debug)]
pub struct CStringResponse(String);

impl BinRead for CStringResponse {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        use binrw::Error::Custom;
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;

        // TODO this isn't pretty, maybe there is a less verbose and elegant
        // solution?
        let string = CStr::from_bytes_with_nul(&bytes)
            .map_err(|err| Custom {
                err: Box::new(err),
                pos: 0,
            })
            .and_then(|cstr| {
                cstr.to_str().map_err(|err| Custom {
                    err: Box::new(err),
                    pos: 0,
                })
            })
            .map(|str| str.to_owned())?;

        Ok(Self(string))
    }
}

#[derive(BinWrite, Debug)]
pub struct QMKBoardManufacturerQuery;

impl XAPRequest for QMKBoardManufacturerQuery {
    type Response = CStringResponse;

    fn id() -> &'static [u8] {
        &[0x1, 0x3]
    }
}

// ==============================
// 0x1 0x4

#[derive(BinWrite, Debug)]
pub struct QMKProductNameQuery;

impl XAPRequest for QMKProductNameQuery {
    type Response = CStringResponse;

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

#[derive(BinWrite, BinRead, Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ConfigBlobOffset(u16);

#[derive(BinWrite, Debug)]
pub struct ConfigBlobChunkQuery(u16);

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

    fn is_secure() -> bool {
        true
    }
}

// ==============================
// 0x1 0x8
#[derive(BinRead, Debug)]
pub struct QMKHardwareIdentifier([u32; 4]);

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

    fn is_secure() -> bool {
        true
    }
}
