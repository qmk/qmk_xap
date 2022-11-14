use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;
use xap_specs::protocol::lighting::{
    BacklightConfig, BacklightConfigGet, BacklightConfigSave, BacklightConfigSet,
};

use crate::xap::hid::XAPClient;
use crate::xap::ClientResult;

#[tauri::command]
pub(crate) async fn backlight_config_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<BacklightConfig> {
    state.lock().query(id, BacklightConfigGet {})
}

#[tauri::command]
pub(crate) async fn backlight_config_set(
    id: Uuid,
    arg: BacklightConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, BacklightConfigSet { config: arg })
}

#[tauri::command]
pub(crate) async fn backlight_config_save(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, BacklightConfigSave {})
}
