use std::io::Write;

use anyhow::{bail, Result};
use bitflags::bitflags;

#[derive(Debug, Clone)]
pub enum Token {
    WithResponse { token: u16 },
    WithoutResponse,
    Broadcast,
}

fn random_xap_token_value() -> u16 {
    let mut token = 0_u16;
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
    pub(crate) fn raw_token(&self) -> u16 {
        self.clone().into()
    }

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

impl TryFrom<u16> for Token {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0100..=0xFFFD => Ok(Token::WithResponse { token: value }),
            0xFFFE => Ok(Token::WithoutResponse),
            0xFFFF => Ok(Token::Broadcast),
            _ => bail!("invalid token value {}", value),
        }
    }
}

impl Into<u16> for Token {
    fn into(self) -> u16 {
        match self {
            Token::WithResponse { token } => token,
            Token::WithoutResponse => 0xFFFE,
            Token::Broadcast => 0xFFFF,
        }
    }
}

pub trait XAPRequest: Sized {
    type Response: TryFrom<ResponseRaw, Error = anyhow::Error>;

    fn id() -> &'static [u8];

    fn is_secure() -> bool {
        false
    }

    fn write_raw_report(&self, mut report: &mut [u8]) -> anyhow::Result<()> {
        report.write(&self.token().raw_token().to_le_bytes())?;

        let id = Self::id();
        report.write(id)?;
        report.write(&[id.len() as u8])?;

        Ok(())
    }

    fn token(&self) -> &Token;

    fn has_response(&self) -> bool {
        matches!(self.token(), Token::WithResponse { .. })
    }

    fn to_response(&self, report: &[u8]) -> Result<Self::Response> {
        let raw_response = ResponseRaw::try_from(report)?;
        raw_response.try_into()
    }
}

pub struct XAPVersionQuery {
    token: Token,
}

impl XAPRequest for XAPVersionQuery {
    type Response = XAPVersion;

    fn id() -> &'static [u8] {
        &[0x00, 0x00]
    }

    fn token(&self) -> &Token {
        &self.token
    }
}

pub struct XAPVersion(u32);

impl TryFrom<ResponseRaw> for XAPVersion {
    type Error = anyhow::Error;

    fn try_from(_value: ResponseRaw) -> Result<Self, Self::Error> {
        Ok(XAPVersion(0))
    }
}

pub struct XAPCapabilitiesQuery {
    token: Token,
}

impl XAPRequest for XAPCapabilitiesQuery {
    type Response = XAPCapabilities;

    fn id() -> &'static [u8] {
        &[0x00, 0x01]
    }
    fn token(&self) -> &Token {
        &self.token
    }
}

pub struct XAPCapabilities(u32);

impl TryFrom<ResponseRaw> for XAPCapabilities {
    type Error = anyhow::Error;

    fn try_from(value: ResponseRaw) -> Result<Self, Self::Error> {
        Ok(XAPCapabilities(0))
    }
}

pub struct XAPEnabledSubsystemsQuery {
    token: Token,
}

impl XAPRequest for XAPEnabledSubsystemsQuery {
    type Response = XAPEnabledSubsystems;

    fn id() -> &'static [u8] {
        &[0x00, 0x02]
    }
    fn token(&self) -> &Token {
        &self.token
    }
}

pub struct XAPEnabledSubsystems(u32);

impl TryFrom<ResponseRaw> for XAPEnabledSubsystems {
    type Error = anyhow::Error;

    fn try_from(value: ResponseRaw) -> Result<Self, Self::Error> {
        Ok(XAPEnabledSubsystems(0))
    }
}

pub struct XAPSecureStatusQuery {
    token: Token,
}

impl XAPSecureStatusQuery {
    pub fn new() -> Self {
        Self {
            token: Token::regular_token(),
        }
    }
}

impl XAPRequest for XAPSecureStatusQuery {
    type Response = XAPSecureStatus;

    fn id() -> &'static [u8] {
        &[0x0, 0x3]
    }

    fn token(&self) -> &Token {
        &self.token
    }
}

#[repr(u8)]
pub enum XAPSecureStatus {
    Disabled,
    UnlockInitiated,
    Unlocked,
}

impl From<u8> for XAPSecureStatus {
    fn from(status: u8) -> Self {
        match status {
            1 => Self::UnlockInitiated,
            2 => Self::Unlocked,
            _ => Self::Disabled,
        }
    }
}

impl TryFrom<ResponseRaw> for XAPSecureStatus {
    type Error = anyhow::Error;

    fn try_from(value: ResponseRaw) -> Result<Self, Self::Error> {
        Ok(value.payload[0].into())
    }
}

pub struct EmptyResponse;

impl TryFrom<ResponseRaw> for EmptyResponse {
    type Error = anyhow::Error;

    fn try_from(value: ResponseRaw) -> Result<Self, Self::Error> {
        Ok(EmptyResponse)
    }
}

pub struct XAPSecureStatusUnlock {
    token: Token,
}

impl XAPRequest for XAPSecureStatusUnlock {
    type Response = EmptyResponse;

    fn id() -> &'static [u8] {
        &[0x0, 0x4]
    }
    fn token(&self) -> &Token {
        &self.token
    }
}

pub struct XAPSecureStatusLock {
    token: Token,
}

impl XAPRequest for XAPSecureStatusLock {
    type Response = EmptyResponse;

    fn id() -> &'static [u8] {
        &[0x0, 0x5]
    }

    fn token(&self) -> &Token {
        &self.token
    }
}

pub struct QMKVersionQuery {
    token: Token,
}
pub struct QMKVersionQueryResponse {
    version: u32,
}

pub struct QMKCapabilitiesQuery {
    token: Token,
}

pub struct QMKCapabilitiesResponse {
    capabilities: u32,
}

bitflags! {
    pub struct ResponseFlags: u8 {
        const SUCCESS = 0b1;
        const SECURE_FAILURE = 0b10;
    }
}

pub struct ResponseRaw {
    token: Token,
    flags: ResponseFlags,
    payload: Vec<u8>,
}

impl TryFrom<&[u8]> for ResponseRaw {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}
