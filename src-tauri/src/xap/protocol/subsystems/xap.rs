use binrw::*;
use bitflags::bitflags;
use serde::Serialize;

use crate::xap::XAPRequest;

// ==============================
// 0x0 0x0
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

// ==============================
// 0x0 0x1
bitflags! {
    #[binread]
    pub struct XAPCapabilities: u32 {
        const VERSION = 1 << 0x0;
        const CAPABILITIES = 1 << 0x1;
        const SUBSYSTEMS = 1 << 0x2;
        const SECURE_STATUS = 1 << 0x3;
        const SECURE_UNLOCK = 1 << 0x4;
        const SECURE_LOCK = 1 << 0x5;
    }
}

#[derive(BinWrite, Debug)]
pub struct XAPCapabilitiesQuery;

impl XAPRequest for XAPCapabilitiesQuery {
    type Response = XAPCapabilities;

    fn id() -> &'static [u8] {
        &[0x00, 0x01]
    }
}

// ==============================
// 0x0 0x2
bitflags! {
    #[binread]
    pub struct XAPEnabledSubsystems: u32 {
        const QMK = 1 << 0x1;
        const KEYBOARD = 1 << 0x2;
        const USER = 1 << 0x3;
        const KEYMAP = 1 << 0x4;
        const REMAPPING = 1 << 0x5;
        const LIGHTING = 1 << 0x6;
    }
}

#[derive(BinWrite, Debug)]
pub struct XAPEnabledSubsystemsQuery;

impl XAPRequest for XAPEnabledSubsystemsQuery {
    type Response = XAPEnabledSubsystems;

    fn id() -> &'static [u8] {
        &[0x00, 0x02]
    }
}

// ==============================
// 0x0 0x3
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
pub struct XAPSecureStatusQuery;

impl XAPRequest for XAPSecureStatusQuery {
    type Response = XAPSecureStatus;

    fn id() -> &'static [u8] {
        &[0x0, 0x3]
    }
}

// ==============================
// 0x0 0x4
#[derive(BinWrite, Debug)]
pub struct XAPSecureStatusUnlock;

impl XAPRequest for XAPSecureStatusUnlock {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x0, 0x4]
    }
}

// ==============================
// 0x0 0x5
#[derive(BinWrite, Debug)]
pub struct XAPSecureStatusLock;

impl XAPRequest for XAPSecureStatusLock {
    type Response = ();

    fn id() -> &'static [u8] {
        &[0x0, 0x5]
    }
}
