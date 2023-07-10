use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;
use xap_specs::{constants::keycode::XAPKeyCodeConfig, KeyPositionConfig};

use crate::{
    aggregation::XAPConstants,
    xap::{hid::XAPClient, ClientResult},
};

#[tauri::command]
#[specta::specta]
pub(crate) fn xap_constants_get(state: State<'_, Arc<Mutex<XAPClient>>>) -> XAPConstants {
    state.lock().xap_constants().into()
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn keycode_set(
    id: Uuid,
    arg: KeyPositionConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().get_device_mut(&id)?.set_keycode(arg)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn keymap_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<Vec<Vec<Vec<XAPKeyCodeConfig>>>> {
    Ok(state.lock().get_device(&id)?.keymap())
}
