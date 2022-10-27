use anyhow::anyhow;
use bitflags::bitflags;
use binrw::{prelude::*, ReadOptions};

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
