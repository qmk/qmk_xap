use parking_lot::Mutex;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::xap::protocol::{
    RGBLightConfig, RGBLightConfigGet, RGBLightConfigSave, RGBLightConfigSet, RGBLightEffectsQuery,
    XAPResult, XAPSecureStatus, XAPSecureStatusQuery, XAPVersion, XAPVersionQuery,
};
use crate::AppState;

#[tauri::command]
pub(crate) fn get_xap_device(
    id: Uuid,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> XAPResult<String> {
    state
        .lock()
        .do_action(id, |device| Ok(format!("{}", device)))
}

#[tauri::command]
pub(crate) async fn get_secure_status(
    id: Uuid,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> XAPResult<XAPSecureStatus> {
    state.lock().do_query(id, XAPSecureStatusQuery {})
}

#[tauri::command]
pub(crate) async fn get_xap_version(
    id: Uuid,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> XAPResult<XAPVersion> {
    state.lock().do_query(id, XAPVersionQuery {})
}

#[tauri::command]
pub(crate) async fn get_rgblight_config(
    id: Uuid,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> XAPResult<RGBLightConfig> {
    state.lock().do_query(id, RGBLightConfigGet {})
}

#[tauri::command]
pub(crate) async fn get_rgblight_effects(
    id: Uuid,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> XAPResult<Vec<u8>> {
    state
        .lock()
        .do_query(id, RGBLightEffectsQuery {})
        .map(|effects| effects.enabled_effect_list())
}

#[tauri::command]
pub(crate) async fn set_rgblight_config(
    id: Uuid,
    arg: RGBLightConfig,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> XAPResult<()> {
    state.lock().do_query(id, RGBLightConfigSet { config: arg })
}

#[tauri::command]
pub(crate) async fn save_rgblight_config(
    id: Uuid,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> XAPResult<()> {
    state.lock().do_query(id, RGBLightConfigSave {})
}
