use std::sync::Arc;

use parking_lot::Mutex;
use serde::Serialize;
use specta::Type;
use tauri::State;
use uuid::Uuid;
use xap_specs::KeyPositionConfig;

use crate::{
    aggregation::XapConstants,
    xap::{
        client::{XapClient, XapClientError},
        device::Keymap,
    },
};

pub type FrontendResult<T> = Result<T, FrontendError>;

#[derive(Debug, Serialize, Type)]
pub struct FrontendError(pub String);

impl From<XapClientError> for FrontendError {
    fn from(err: XapClientError) -> Self {
        Self(err.to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub fn xap_constants_get(state: State<'_, Arc<Mutex<XapClient>>>) -> XapConstants {
    state.lock().xap_constants().into()
}

#[tauri::command]
#[specta::specta]
pub fn keycode_set(
    id: Uuid,
    arg: KeyPositionConfig,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> FrontendResult<()> {
    state
        .lock()
        .get_device_mut(&id)?
        .set_keycode(arg)
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn keymap_get(id: Uuid, state: State<'_, Arc<Mutex<XapClient>>>) -> FrontendResult<Keymap> {
    Ok(state.lock().get_device(&id)?.keymap())
}
