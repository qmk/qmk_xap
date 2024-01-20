use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use xap_specs::KeyCode;
use xap_specs::{
    constants::keycode::XAPKeyCodeConfig,
    EncoderPosition,
    KeyPosition,
};

use xap_specs::xap_generated::keymap_routes::{
    GetEncoderKeycodeRequest, GetEncoderKeycodeRequestArg, GetKeycodeRequest, GetKeycodeRequestArg,
};

use crate::xap::{hid::XAPClient, ClientResult};

#[tauri::command]
pub(crate) async fn keycode_get(
    id: Uuid,
    arg: KeyPosition,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<KeyCode> {
    // TODO
    let arg = GetKeycodeRequestArg {
        layer: arg.layer,
        row: arg.row,
        column: arg.col,
    };

    state
        .lock()
        .query(id, GetKeycodeRequest(arg))
        .map(|res| KeyCode(res.0))
}

#[tauri::command]
pub(crate) async fn encoder_keycode_get(
    id: Uuid,
    arg: EncoderPosition,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<KeyCode> {
    // TODO
    let arg = GetEncoderKeycodeRequestArg {
        layer: arg.layer,
        encoder: arg.encoder,
        clockwise: arg.clockwise,
    };

    state
        .lock()
        .query(id, GetEncoderKeycodeRequest(arg))
        .map(|res| KeyCode(res.0))
}

#[tauri::command]
pub(crate) async fn keymap_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<Vec<Vec<Vec<XAPKeyCodeConfig>>>> {
    Ok(state.lock().get_device(&id)?.keymap())
}
