use std::fmt::Display;

use anyhow::anyhow;
use binrw::{BinRead, BinReaderExt, BinResult, Endian};
use serde::Serialize;
use specta::Type;

pub mod broadcast;
pub mod constants;
pub mod request;
pub mod response;
pub mod token;

#[derive(Debug, Serialize, Clone, Copy, Type)]
pub enum XapSecureStatus {
    Locked,
    Unlocking,
    Unlocked,
}

impl From<u8> for XapSecureStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Unlocking,
            2 => Self::Unlocked,
            _ => Self::Locked,
        }
    }
}

impl Default for XapSecureStatus {
    fn default() -> Self {
        Self::Locked
    }
}

impl BinRead for XapSecureStatus {
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

impl Display for XapSecureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XapSecureStatus::Locked => write!(f, "Locked"),
            XapSecureStatus::Unlocking => write!(f, "Unlocking"),
            XapSecureStatus::Unlocked => write!(f, "Unlocked"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct XapVersion(u32);

impl TryFrom<u32> for XapVersion {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x01 | 0x0100 | 0x0200 | 0x0300 => Ok(Self(value)),
            _ => Err(anyhow!(
                "{value:06X} is not a valid BCD encoded Xap version"
            )),
        }
    }
}

impl Display for XapVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for digit in self.0.to_be_bytes() {
            write!(f, "{digit:02X}")?;
        }
        Ok(())
    }
}
