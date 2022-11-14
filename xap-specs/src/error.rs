use serde::Serialize;

pub type XAPResult<T> = core::result::Result<T, XAPError>;

#[derive(thiserror::Error, Debug)]
pub enum XAPError {
    #[error("bit marshalling failed {0}")]
    BitHandling(#[from] binrw::Error),
    #[error("XAP communication failed {0}")]
    Protocol(String),
    #[error("device is locked")]
    SecureLocked,
    #[error("request failed")]
    RequestFailed,
    #[error("io error {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON (de)serialization error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("HJSON (de)serialization error {0}")]
    HJSONError(#[from] deser_hjson::Error),
}

impl Serialize for XAPError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
