use std::fmt::Display;

use binrw::{BinRead, BinReaderExt, BinResult, BinWrite, Endian};
use error::XAPError;
use serde::{Deserialize, Serialize};
use specta::Type;

pub mod constants;
pub mod error;
pub mod protocol;
pub mod broadcast;
pub mod request;
pub mod response;
pub mod token;

#[derive(BinWrite, Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct KeyPosition {
    pub layer: u8,
    pub row: u8,
    pub column: u8,
}

// ==============================
// 0x4 0x4
#[derive(BinWrite, Debug, Serialize, Deserialize, Type)]
pub struct EncoderPosition {
    pub layer: u8,
    pub encoder: u8,
    pub clockwise: u8,
}

#[derive(BinRead, Debug, Serialize, Deserialize, Type)]
pub struct KeyCode(pub u16);

// ==============================
// 0x5 0x3
#[derive(BinWrite, BinRead, Clone, Debug, Default, Serialize, Deserialize, Type)]
pub struct KeyPositionConfig {
    pub layer: u8,
    pub row: u8,
    pub col: u8,
    pub keycode: u16,
}

// ==============================
// 0x5 0x4
#[derive(BinWrite, BinRead, Debug, Serialize, Deserialize, Type)]
pub struct EncoderPositionConfig {
    pub layer: u8,
    pub encoder: u8,
    pub clockwise: u8,
    pub keycode: u16,
}

#[derive(Debug, Serialize, Clone, Copy, Type)]
pub enum XAPSecureStatus {
    Locked,
    Unlocking,
    Unlocked,
}

impl From<u8> for XAPSecureStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Unlocking,
            2 => Self::Unlocked,
            _ => Self::Locked,
        }
    }
}

impl Default for XAPSecureStatus {
    fn default() -> Self {
        Self::Locked
    }
}

impl BinRead for XAPSecureStatus {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let raw_status: u8 = reader.read_le()?;
        Ok(match raw_status {
            1 => Self::Unlocking,
            2 => Self::Unlocked,
            _ => Self::Locked,
        })
    }
}

impl Display for XAPSecureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XAPSecureStatus::Locked => write!(f, "Locked"),
            XAPSecureStatus::Unlocking => write!(f, "Unlocking"),
            XAPSecureStatus::Unlocked => write!(f, "Unlocked"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct XAPVersion(u32);

impl TryFrom<u32> for XAPVersion {
    type Error = XAPError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x01 | 0x0100 | 0x0200 | 0x0300 => Ok(Self(value)),
            _ => Err(XAPError::Protocol(format!(
                "{value:06X} is not a valid BCD encoded XAP version"
            ))),
        }
    }
}

impl Display for XAPVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for digit in self.0.to_be_bytes() {
            write!(f, "{digit:02X}")?;
        }
        Ok(())
    }
}
