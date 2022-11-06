use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::xap::{
    protocol::XAPResult, EncoderPositionConfig, KeyPositionConfig, RemapEncoderQuery, XAPClient,
    XAPError,
};

#[tauri::command]
pub(crate) async fn keycode_set(
    id: Uuid,
    arg: KeyPositionConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    if let Some(device) = state.lock().get_device_mut(&id) {
        device.set_keycode(arg)
    } else {
        Err(XAPError::UnknownDevice(id))
    }
}

#[tauri::command]
pub(crate) async fn encoder_keycode_set(
    id: Uuid,
    arg: EncoderPositionConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    // TODO handle as regular keymap
    state.lock().query(id, RemapEncoderQuery(arg))
}
