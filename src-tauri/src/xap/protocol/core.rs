// This file defines the base structs and implements the needed traits for them

use anyhow::{bail, Result};
use binrw::prelude::*;
use hidapi::HidError;
use log::debug;
use std::{
    fmt::Debug,
    io::{Cursor, Seek, Write}
};
use serde::Serialize;
use super::token::*;

pub type XAPResult<T> = core::result::Result<T, XAPError>;

#[derive(thiserror::Error, Debug)]
pub enum XAPError {
    // TODO find better names and description
    #[error(transparent)]
    BitHandling(#[from] binrw::Error),
    #[error("XAP communication failed")]
    Protocol(String),
    #[error(transparent)]
    HID(#[from] HidError),
    #[error("something happened")]
    Other(#[from] anyhow::Error),
}

// TODO structured JSON error?
impl Serialize for XAPError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub struct RequestRaw<T: XAPRequest> {
    token: Token,
    payload: T,
}

impl<T> RequestRaw<T>
where
    T: XAPRequest,
{
    pub fn new(payload: T) -> Self {
        Self {
            token: Token::regular_token(),
            payload,
        }
    }

    pub fn token(&self) -> &Token {
        &self.token
    }
}

impl<T> BinWrite for RequestRaw<T>
where
    T: XAPRequest,
{
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _options: &binrw::WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        writer.write_le(&self.token)?;
        // Dummy write of the payload length, which is not known at this point.
        writer.write_le(&0_u8)?;
        writer.write_le(&T::id())?;
        writer.write_le(&self.payload)?;

        // Calculate payload size from current position in the writer stream,
        // which points at the end of payload and contains the Token and payload
        // lenght field itself. These have to be substracted to get the total
        // size of the payload.
        let payload_length = writer.stream_position()?
            - std::mem::size_of::<u16>() as u64 // Token
            - std::mem::size_of::<u8>() as u64; // payload length field

        // Position our writer on the payload_length field again and write the correct value.
        writer.seek(std::io::SeekFrom::Start(2))?;
        writer.write_le(&(payload_length as u8))
    }
}

#[binread]
#[derive(Debug)]
pub struct ResponseRaw {
    token: Token,
    flags: ResponseFlags,
    #[br(temp)]
    payload_len: u8,
    #[br(count = payload_len)]
    payload: Vec<u8>,
}

impl ResponseRaw {
    pub fn from_raw_report(report: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(report);
        let raw_response = ResponseRaw::read_le(&mut reader)?;

        debug!("received raw XAP response: {:#?}", raw_response);

        // TODO add flag handling here
        if !raw_response.flags.contains(ResponseFlags::SUCCESS) {
            bail!("XAP responded with a failed transaction!");
        }

        Ok(raw_response)
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn into_xap_response<T>(self) -> XAPResult<T::Response>
    where
        T: XAPRequest,
    {
        let mut reader = Cursor::new(self.payload);

        Ok(T::Response::read_le(&mut reader)?)
    }
}

pub trait XAPRequest: Sized + Debug + BinWrite<Args = ()> {
    type Response: BinRead<Args = ()>;

    fn id() -> &'static [u8];

    fn is_secure() -> bool {
        false
    }
}