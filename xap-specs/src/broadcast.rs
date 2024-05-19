use core::fmt::Debug;
use std::io::Cursor;

use binrw::{binread, BinRead, BinReaderExt, Endian};
use log::trace;

use crate::error::XapResult;
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
#[derive(Debug)]
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

    pub fn from_raw_report(report: &[u8]) -> XapResult<Self> {
        let mut reader = Cursor::new(report);
        let broadcast = Self::read_le(&mut reader)?;
        trace!("received raw XAP broadcast: {:#?}", broadcast);
        Ok(broadcast)
    }

    pub fn into_xap_broadcast<T>(self) -> XapResult<T>
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
        let len: u8 = reader.read_le()?;
        let mut bytes = Vec::with_capacity(len as usize);
        reader.read_exact(&mut bytes[..len as usize])?;
        let mut cursor = Cursor::new(&bytes);
        Ok(Self(std::io::read_to_string(&mut cursor)?))
    }
}

impl XapBroadcast for LogBroadcast {}

#[derive(BinRead, Debug)]
pub struct SecureStatusBroadcast(pub XapSecureStatus);

impl XapBroadcast for SecureStatusBroadcast {}
