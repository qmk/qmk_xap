mod keymap;
mod lighting;
mod qmk;
mod remap;
mod xap;

pub(crate) use keymap::*;
pub(crate) use lighting::*;
pub(crate) use qmk::*;
pub(crate) use remap::*;
pub(crate) use xap::*;

use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;

use crate::{
    aggregation::XAPConstants,
    xap::{XAPClient, XAPResult},
};

#[tauri::command]
pub(crate) fn xap_constants_get(
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<XAPConstants> {
    Ok(state.lock().xap_constants().into())
}
