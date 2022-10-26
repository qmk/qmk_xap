use std::{
    fmt::Debug,
    io::{Cursor, Seek, Write},
};

use anyhow::{anyhow, bail, Result};
use binrw::{prelude::*, ReadOptions};
use bitflags::bitflags;
use log::debug;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone)]
#[binwrite]
#[repr(C, u16)]
pub enum Token {
    WithResponse {
        token: u16,
    },
    #[br(magic = 0xFFFE)]
    WithoutResponse,
    #[br(magic = 0xFFFF)]
    Broadcast,
}

fn random_xap_token_value() -> u16 {
    let mut token;
    loop {
        token = rand::random();
        match token {
            0x0100..=0xFFFD => break,
            _ => continue,
        }
    }
    token
}

impl Token {
    pub(crate) fn regular_token() -> Token {
        Self::WithResponse {
            token: random_xap_token_value(),
        }
    }

    pub(crate) fn broadcast_token() -> Token {
        Self::Broadcast
    }

    pub(crate) fn without_response_token() -> Token {
        Self::WithoutResponse
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
    payload_len: u8,
    payload: T,
}

impl<T> RequestRaw<T>
where
    T: XAPRequest,
{
    pub fn to_response(&self, report: &[u8]) -> Result<T::Response> {
        let mut reader = Cursor::new(report);
        let raw_response = ResponseRaw::read_le(&mut reader)?;

        debug!("received raw XAP response: {:#?}", raw_response);

        // TODO add flag handling here
        if !raw_response.flags.contains(ResponseFlags::SUCCESS) {
            bail!("XAP responded with a failed transaction!");
        }

        let mut reader = Cursor::new(raw_response.payload);
        T::Response::read_le(&mut reader)
            .map_err(|err| anyhow!("failed to deserialize XAP response with {}", err))
    }

    pub fn new(payload: T) -> Self {
        Self {
            token: Token::regular_token(),
            payload_len: (T::id().len() + std::mem::size_of::<T>()) as u8,
            payload,
        }
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
        writer.write_le(&self.payload_len)?;
        writer.write_le(&T::id())?;
        writer.write_le(&self.payload)
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
        let mut status = [0_u8];

        reader.read_exact(&mut status)?;

        Ok(match status[0] {
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

#[binrw]
#[derive(Debug, TS, Serialize)]
#[repr(C, packed)]
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
