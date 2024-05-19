// Module which contains everything needed to communicate to a XAP-enabled device

use serde::Serialize;
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

use xap_specs::error::XapError;

// pub mod constant;
pub mod hid;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HID communication failed {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("unkown device {0}")]
    UnknownDevice(Uuid),
    #[error("JSON (de)serialization error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("HJSON (de)serialization error {0}")]
    HJSONError(#[from] deser_hjson::Error),
    #[error("unknown error {0}")]
    Other(#[from] anyhow::Error),
    #[error("XAP protocol error {0}")]
    ProtocolError(#[from] XapError),
}

impl Serialize for ClientError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type FrontendResult<T> = Result<T, FrontendError>;

#[derive(Debug, Serialize, Type)]
pub struct FrontendError(pub String);

impl From<ClientError> for FrontendError {
    fn from(err: ClientError) -> Self {
        Self(err.to_string())
    }
}
