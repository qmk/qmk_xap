// This file defines the different kind of tokens

use anyhow::anyhow;
use binrw::{prelude::*, ReadOptions};
use bitflags::bitflags;
use rand::distributions::{Distribution, Uniform};

#[derive(Debug, Clone, PartialEq, Eq)]
#[binwrite]
#[br(repr = u16)]
pub enum Token {
    WithResponse(u16),
    #[br(magic = 0xFFFE)]
    WithoutResponse,
    #[br(magic = 0xFFFF)]
    Broadcast,
}

impl Token {
    pub(crate) fn regular_token() -> Token {
        Self::WithResponse(Uniform::from(0x0100..=0xFFFD).sample(&mut rand::thread_rng()))
    }
}

impl BinRead for Token {
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        let raw: u16 = reader.read_le()?;

        match raw {
            0x0100..=0xFFFD => Ok(Token::WithResponse(raw)),
            0xFFFE => Ok(Token::WithoutResponse),
            0xFFFF => Ok(Token::Broadcast),
            _ => Err(binrw::Error::Custom {
                pos: 0,
                err: Box::new(anyhow!("XAP token has invalid value of {}", raw)),
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
