use core::fmt::Debug;
use std::io::{Cursor, Read, Seek};

use binrw::{binread, BinRead, BinResult, Endian};
use bitflags::bitflags;
use log::trace;
use serde::Serialize;
use specta::Type;

use crate::{
    error::{XapError, XapResult},
    request::XapRequest,
    token::Token,
};

#[derive(Serialize, BinRead, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResponseFlags(u8);

bitflags! {
    impl ResponseFlags: u8 {
        const SUCCESS = 0b1;
        const SECURE_FAILURE = 0b10;
    }
}

#[binread]
#[derive(Debug)]
pub struct RawResponse {
    token: Token,
    flags: ResponseFlags,
    #[br(temp)]
    payload_len: u8,
    #[br(count = payload_len as usize)]
    payload: Vec<u8>,
}

impl RawResponse {
    pub fn from_raw_report(report: &[u8]) -> XapResult<Self> {
        let mut reader = Cursor::new(report);
        let response = RawResponse::read_le(&mut reader)?;

        trace!("received raw XAP response: {:#?}", response);

        match response.flags {
            ResponseFlags::SUCCESS => Ok(response),
            ResponseFlags::SECURE_FAILURE => Err(XapError::SecureLocked),
            _ => Err(XapError::Protocol(format!(
                "unknown response flag {:?}",
                response.flags
            ))),
        }
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn into_xap_response<T>(self) -> XapResult<T::Response>
    where
        T: XapRequest,
    {
        let mut reader = Cursor::new(self.payload);

        Ok(T::Response::read_le(&mut reader)?)
    }
}

#[derive(Debug, Default, Clone, Serialize, Type)]
pub struct UTF8String(pub String);

impl BinRead for UTF8String {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(Self(std::io::read_to_string(reader)?))
    }
}

#[derive(BinRead, Debug)]
pub struct SecureActionResponse(u8);

impl From<SecureActionResponse> for XapResult<()> {
    fn from(val: SecureActionResponse) -> Self {
        if val.0 == 0 {
            Err(XapError::SecureLocked)
        } else {
            Ok(())
        }
    }
}
