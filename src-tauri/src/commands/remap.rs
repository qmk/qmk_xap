use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::xap::{
    protocol::XAPResult, EncoderPositionConfig, KeyPositionConfig, RemapEncoderQuery,
    RemapKeycodeQuery, XAPClient,
};

#[tauri::command]
pub(crate) async fn keycode_set(
    id: Uuid,
    arg: KeyPositionConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, RemapKeycodeQuery(arg))
}

#[tauri::command]
pub(crate) async fn encoder_keycode_set(
    id: Uuid,
    arg: EncoderPositionConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, RemapEncoderQuery(arg))
}
