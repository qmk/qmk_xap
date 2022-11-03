use core::fmt::Debug;
use std::io::Cursor;

use binrw::{binread, BinRead, BinReaderExt};
use log::trace;

use super::{XAPResult, XAPSecureStatus};
use crate::xap::protocol::Token;

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
    #[br(count = payload_len)]
    payload: Vec<u8>,
}

impl BroadcastRaw {
    pub fn broadcast_type(&self) -> &BroadcastType {
        &self.broadcast_type
    }

    pub fn from_raw_report(report: &[u8]) -> XAPResult<Self> {
        let mut reader = Cursor::new(report);
        let broadcast = Self::read_le(&mut reader)?;
        trace!("received raw XAP broadcast: {:#?}", broadcast);
        Ok(broadcast)
    }

    pub fn into_xap_broadcast<T>(self) -> XAPResult<T>
    where
        T: XAPBroadcast,
    {
        let mut reader = Cursor::new(&self.payload);
        Ok(T::read_le(&mut reader)?)
    }
}

pub trait XAPBroadcast: Sized + Debug + BinRead<Args = ()> {}

#[derive(Debug)]
pub struct LogBroadcast(pub String);

impl BinRead for LogBroadcast {
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _options: &binrw::ReadOptions,
        _args: Self::Args,
    ) -> binrw::BinResult<Self> {
        let len: u8 = reader.read_le()?;
        let mut bytes = Vec::with_capacity(len as usize);
        reader.read_exact(&mut bytes[..len as usize])?;
        let mut cursor = Cursor::new(&bytes);
        Ok(Self(std::io::read_to_string(&mut cursor)?))
    }
}

impl XAPBroadcast for LogBroadcast {}

#[derive(BinRead, Debug)]
pub struct SecureStatusBroadcast(pub XAPSecureStatus);

impl XAPBroadcast for SecureStatusBroadcast {}
