use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;
use uuid::Uuid;

use crate::{
    aggregation::{XapConstants, XapDevice},
    xap::{
        client::{XapClient, XapClientError},
        device::Keymap,
        spec::remapping::RemappingSetKeycodeArg,
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
    state.lock().unwrap().xap_constants().into()
}

#[tauri::command]
#[specta::specta]
pub fn keycode_set(
    id: Uuid,
    arg: RemappingSetKeycodeArg,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> FrontendResult<()> {
    state
        .lock()
        .unwrap()
        .get_device_mut(&id)?
        .set_keycode(arg)
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn keymap_get(id: Uuid, state: State<'_, Arc<Mutex<XapClient>>>) -> FrontendResult<Keymap> {
    Ok(state.lock().unwrap().get_device(&id)?.keymap())
}

#[tauri::command]
#[specta::specta]
pub fn device_get(id: Uuid, state: State<'_, Arc<Mutex<XapClient>>>) -> FrontendResult<XapDevice> {
    Ok(state.lock().unwrap().get_device(&id)?.as_dto())
}
