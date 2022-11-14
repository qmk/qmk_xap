use core::fmt::Debug;
use std::io::{Cursor, Read, Seek};

use binrw::{binread, BinRead, BinResult, ReadOptions};
use bitflags::bitflags;
use log::trace;

use crate::{
    error::{XAPError, XAPResult},
    request::XAPRequest,
    token::Token,
};

bitflags! {
    #[binread]
    pub struct ResponseFlags: u8 {
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
    #[br(count = payload_len)]
    payload: Vec<u8>,
}

impl RawResponse {
    pub fn from_raw_report(report: &[u8]) -> XAPResult<Self> {
        let mut reader = Cursor::new(report);
        let response = RawResponse::read_le(&mut reader)?;

        trace!("received raw XAP response: {:#?}", response);

        if !response.flags.contains(ResponseFlags::SUCCESS) {
            return Err(XAPError::RequestFailed);
        } else if response.flags.contains(ResponseFlags::SECURE_FAILURE) {
            return Err(XAPError::SecureLocked);
        }

        Ok(response)
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

#[derive(Debug)]
pub struct UTF8StringResponse(pub String);

impl BinRead for UTF8StringResponse {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Ok(Self(std::io::read_to_string(reader)?))
    }
}

#[derive(BinRead, Debug)]
pub struct SecureActionResponse(u8);

impl Into<XAPResult<()>> for SecureActionResponse {
    fn into(self) -> XAPResult<()> {
        if self.0 == 0 {
            Err(XAPError::SecureLocked)
        } else {
            Ok(())
        }
    }
}
