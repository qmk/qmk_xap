use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;
use xap_specs::{constants::keycode::XapKeyCodeConfig, KeyPositionConfig};

use crate::{
    aggregation::XapConstants,
    xap::{hid::XapClient, FrontendResult},
};

#[tauri::command]
#[specta::specta]
pub(crate) fn xap_constants_get(state: State<'_, Arc<Mutex<XapClient>>>) -> XapConstants {
    state.lock().xap_constants().into()
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn keycode_set(
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
pub(crate) async fn keymap_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> FrontendResult<Vec<Vec<Vec<XapKeyCodeConfig>>>> {
    Ok(state.lock().get_device(&id)?.keymap())
}
