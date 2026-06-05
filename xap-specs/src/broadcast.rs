use core::fmt::Debug;
use std::io::Cursor;

use anyhow::Result;
use binrw::{binread, BinRead, Endian};
use log::trace;

use crate::token::Token;
use crate::XapSecureStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
#[binread]
#[br(repr = u8)]
pub enum BroadcastType {
    Log = 0,
    SecureStatus = 1,
    Keyboard = 2,
    User = 3,
}

#[binread]
#[derive(Debug, Clone)]
pub struct BroadcastRaw {
    _token: Token,
    broadcast_type: BroadcastType,
    #[br(temp)]
    payload_len: u8,
    #[br(count = payload_len as usize)]
    payload: Vec<u8>,
}

impl BroadcastRaw {
    pub fn broadcast_type(&self) -> &BroadcastType {
        &self.broadcast_type
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn from_raw_report(report: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(report);
        let broadcast = Self::read_le(&mut reader)?;
        trace!("received raw XAP broadcast: {:#?}", broadcast);
        Ok(broadcast)
    }

    pub fn into_xap_broadcast<T>(self) -> Result<T>
    where
        T: XapBroadcast,
    {
        let mut reader = Cursor::new(&self.payload);
        Ok(T::read_le(&mut reader)?)
    }
}

pub trait XapBroadcast: Sized + Debug + for<'a> BinRead<Args<'a> = ()> {}

#[derive(Debug)]
pub struct LogBroadcast(pub String);

impl BinRead for LogBroadcast {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        Ok(Self(std::io::read_to_string(reader)?))
    }
}

impl XapBroadcast for LogBroadcast {}

#[derive(BinRead, Debug)]
pub struct SecureStatusBroadcast(pub XapSecureStatus);

impl XapBroadcast for SecureStatusBroadcast {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_non_empty_log_broadcast_payload() {
        let report = [
            0xFF,
            0xFF,
            BroadcastType::Log as u8,
            0x05,
            b'h',
            b'e',
            b'l',
            b'l',
            b'o',
        ];

        let broadcast = BroadcastRaw::from_raw_report(&report).expect("failed to read broadcast");
        let log: LogBroadcast = broadcast
            .into_xap_broadcast()
            .expect("failed to decode log payload");

        assert_eq!(log.0, "hello");
    }

    #[test]
    fn exposes_raw_payload_for_unparsed_broadcasts() {
        let report = [
            0xFF,
            0xFF,
            BroadcastType::User as u8,
            0x04,
            0x01,
            0x2A,
            0xFF,
            0x00,
        ];

        let broadcast = BroadcastRaw::from_raw_report(&report).expect("failed to read broadcast");

        assert_eq!(broadcast.broadcast_type(), &BroadcastType::User);
        assert_eq!(broadcast.payload(), &[0x01, 0x2A, 0xFF, 0x00]);
    }
}
