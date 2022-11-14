use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;
use xap_specs::protocol::lighting::{
    RGBLightConfig, RGBLightConfigGet, RGBLightConfigSave, RGBLightConfigSet,
};

use crate::xap::hid::XAPClient;
use crate::xap::ClientResult;

#[tauri::command]
pub(crate) async fn rgblight_config_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<RGBLightConfig> {
    state.lock().query(id, RGBLightConfigGet {})
}

#[tauri::command]
pub(crate) async fn rgblight_config_set(
    id: Uuid,
    arg: RGBLightConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, RGBLightConfigSet { config: arg })
}

#[tauri::command]
pub(crate) async fn rgblight_config_save(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, RGBLightConfigSave {})
}
