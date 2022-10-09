use serde::Serialize;
use ts_rs::TS;

#[derive(TS, Serialize)]
#[ts(export)]
pub(crate) struct Devices {
    pub(crate) list: Vec<String>,
}
