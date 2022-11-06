use std::sync::Arc;

use anyhow::anyhow;
use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::xap::{
    protocol::XAPResult, EncoderPosition, KeyCode, KeyPosition, KeyPositionConfig,
    KeymapEncoderQuery, KeymapKeycodeQuery, XAPClient, XAPError,
};

#[tauri::command]
pub(crate) async fn keycode_get(
    id: Uuid,
    arg: KeyPosition,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<KeyCode> {
    state.lock().query(id, KeymapKeycodeQuery(arg))
}

#[tauri::command]
pub(crate) async fn encoder_keycode_get(
    id: Uuid,
    arg: EncoderPosition,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<KeyCode> {
    state.lock().query(id, KeymapEncoderQuery(arg))
}

#[tauri::command]
pub(crate) async fn keymap_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<Vec<Vec<Vec<KeyPositionConfig>>>> {
    if let Some(device) = state.lock().get_device(&id) {
        Ok(device.keymap().clone())
    } else {
        Err(XAPError::Other(anyhow!("Device not found")))
    }
}
