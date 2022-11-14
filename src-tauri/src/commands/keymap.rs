use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use xap_specs::{
    constants::keycode::XAPKeyCodeConfig,
    protocol::keymap::{
        EncoderPosition, KeyCode, KeyPosition, KeymapEncoderQuery, KeymapKeycodeQuery,
    },
};

use crate::xap::{hid::XAPClient, ClientResult};

#[tauri::command]
pub(crate) async fn keycode_get(
    id: Uuid,
    arg: KeyPosition,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<KeyCode> {
    state.lock().query(id, KeymapKeycodeQuery(arg))
}

#[tauri::command]
pub(crate) async fn encoder_keycode_get(
    id: Uuid,
    arg: EncoderPosition,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<KeyCode> {
    state.lock().query(id, KeymapEncoderQuery(arg))
}

#[tauri::command]
pub(crate) async fn keymap_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<Vec<Vec<Vec<XAPKeyCodeConfig>>>> {
    Ok(state.lock().get_device(&id)?.keymap())
}
