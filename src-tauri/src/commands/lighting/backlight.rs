use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::xap::{
    protocol::{
        BacklightConfig, BacklightConfigGet, BacklightConfigSave, BacklightConfigSet, XAPResult,
    },
    XAPClient,
};

#[tauri::command]
pub(crate) async fn backlight_config_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<BacklightConfig> {
    state.lock().query(id, BacklightConfigGet {})
}

#[tauri::command]
pub(crate) async fn backlight_config_set(
    id: Uuid,
    arg: BacklightConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, BacklightConfigSet { config: arg })
}

#[tauri::command]
pub(crate) async fn backlight_config_save(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, BacklightConfigSave {})
}
