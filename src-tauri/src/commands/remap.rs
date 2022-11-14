use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;
use xap_specs::protocol::remap::{EncoderPositionConfig, KeyPositionConfig, RemapEncoderQuery};

use crate::xap::{hid::XAPClient, ClientResult};

#[tauri::command]
pub(crate) async fn keycode_set(
    id: Uuid,
    arg: KeyPositionConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().get_device_mut(&id)?.set_keycode(arg)
}

#[tauri::command]
pub(crate) async fn encoder_keycode_set(
    id: Uuid,
    arg: EncoderPositionConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    // TODO handle as regular keymap
    state.lock().query(id, RemapEncoderQuery(arg))
}
