use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;
use xap_specs::protocol::lighting::{
    RGBMatrixConfig, RGBMatrixConfigGet, RGBMatrixConfigSave, RGBMatrixConfigSet,
};

use crate::xap::{hid::XAPClient, ClientResult};

#[tauri::command]
pub(crate) async fn rgbmatrix_config_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<RGBMatrixConfig> {
    state.lock().query(id, RGBMatrixConfigGet {})
}

#[tauri::command]
pub(crate) async fn rgbmatrix_config_set(
    id: Uuid,
    arg: RGBMatrixConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, RGBMatrixConfigSet { config: arg })
}

#[tauri::command]
pub(crate) async fn rgbmatrix_config_save(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, RGBMatrixConfigSave {})
}
