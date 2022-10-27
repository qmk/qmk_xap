use std::{
    fmt::Debug,
    io::{Cursor, Seek, Write},
};

use anyhow::{anyhow, bail};
use binrw::{prelude::*, ReadOptions};
use bitflags::bitflags;
use hidapi::HidError;
use log::debug;
use serde::Serialize;
use ts_rs::TS;

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

#[derive(Debug, Clone, PartialEq, Eq)]
#[binwrite]
#[br(repr = u16)]
pub enum Token {
    WithResponse {
        token: u16,
    },
    #[br(magic = 0xFFFE)]
    WithoutResponse,
    #[br(magic = 0xFFFF)]
    Broadcast,
}

impl Token {
    pub(crate) fn regular_token() -> Token {
        Self::WithResponse {
            token: Self::random_xap_token_value(),
        }
    }

    pub(crate) fn broadcast_token() -> Token {
        Self::Broadcast
    }

    pub(crate) fn without_response_token() -> Token {
        Self::WithoutResponse
    }

    fn random_xap_token_value() -> u16 {
        loop {
            match rand::random() {
                token @ 0x0100..=0xFFFD => break token,
                _ => continue,
            }
        }
    }
}

impl BinRead for Token {
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        let raw_token: u16 = reader.read_le()?;

        match raw_token {
            0x0100..=0xFFFD => Ok(Token::WithResponse { token: raw_token }),
            0xFFFE => Ok(Token::WithoutResponse),
            0xFFFF => Ok(Token::Broadcast),
            _ => Err(binrw::Error::Custom {
                pos: 0,
                err: Box::new(anyhow!("XAP token has invalid value of {}", raw_token)),
            }),
        }
    }
}

bitflags! {
    #[binread]
    pub struct ResponseFlags: u8 {
        const SUCCESS = 0b1;
        const SECURE_FAILURE = 0b10;
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
    pub fn from_raw_report(report: &[u8]) -> XAPResult<Self> {
        let mut reader = Cursor::new(report);
        let raw_response = ResponseRaw::read_le(&mut reader)?;

        debug!("received raw XAP response: {:#?}", raw_response);

        // TODO add flag handling here
        if !raw_response.flags.contains(ResponseFlags::SUCCESS) {
            return Err(XAPError::Protocol(
                "XAP responded with a failed transaction!".to_owned(),
            ));
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

//
// XAP SUBSYSTEM
//

#[binread]
#[derive(Debug, Serialize)]
pub struct XAPVersion(u32);

#[derive(Debug, BinWrite)]
pub struct XAPVersionQuery;

impl XAPRequest for XAPVersionQuery {
    type Response = XAPVersion;

    fn id() -> &'static [u8] {
        &[0x00, 0x00]
    }
}

#[derive(BinRead, Debug)]
pub struct XAPCapabilities(u32);

#[derive(BinWrite, Debug)]
pub struct XAPCapabilitiesQuery;

impl XAPRequest for XAPCapabilitiesQuery {
    type Response = XAPCapabilities;

    fn id() -> &'static [u8] {
        &[0x00, 0x01]
    }
}

#[derive(BinRead, Debug)]
pub struct XAPEnabledSubsystems(u32);

#[derive(BinWrite, Debug)]
pub struct XAPEnabledSubsystemsQuery;

impl XAPRequest for XAPEnabledSubsystemsQuery {
    type Response = XAPEnabledSubsystems;

    fn id() -> &'static [u8] {
        &[0x00, 0x02]
    }
}

#[derive(BinWrite, Debug)]
pub struct XAPSecureStatusQuery;

impl XAPRequest for XAPSecureStatusQuery {
    type Response = XAPSecureStatus;

    fn id() -> &'static [u8] {
        &[0x0, 0x3]
    }
}

#[derive(Debug, Serialize)]
pub enum XAPSecureStatus {
    Disabled,
    UnlockInitiated,
    Unlocked,
}

impl BinRead for XAPSecureStatus {
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        let raw_status: u8 = reader.read_le()?;
        Ok(match raw_status {
            1 => Self::UnlockInitiated,
            2 => Self::Unlocked,
            _ => Self::Disabled,
        })
    }
}

#[derive(BinWrite, Debug)]
pub struct XAPSecureStatusUnlock;

impl XAPRequest for XAPSecureStatusUnlock {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x0, 0x4]
    }
}

#[derive(BinWrite, Debug)]
pub struct XAPSecureStatusLock;

impl XAPRequest for XAPSecureStatusLock {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x0, 0x5]
    }
}

//
// QMK SUBSYSTEM - INCOMPLETE!
//

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

//
// RGB LIGHT SUBSYSTEM - INCOMPLETE!
//

#[derive(BinWrite, BinRead, Debug, TS, Serialize)]
#[ts(export)]
pub struct RGBLightConfig {
    pub enable: u8,
    pub mode: u8,
    pub hue: u8,
    pub sat: u8,
    pub val: u8,
    pub speed: u8,
}

#[derive(BinWrite, Debug)]
pub struct RGBLightConfigQuery;

impl XAPRequest for RGBLightConfigQuery {
    type Response = RGBLightConfig;

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x3]
    }
}

#[derive(BinWrite, Debug)]
pub struct RGBLightConfigCommand {
    pub config: RGBLightConfig,
}

impl XAPRequest for RGBLightConfigCommand {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x6, 0x3, 0x4]
    }
}
