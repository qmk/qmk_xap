use tauri::State;

use crate::xap::protocol::{
    RGBLightConfig, RGBLightConfigGet, RGBLightConfigSave, RGBLightConfigSet, RGBLightEffectsQuery,
    XAPResult, XAPSecureStatus, XAPSecureStatusQuery, XAPVersion, XAPVersionQuery,
};
use crate::AppState;

#[tauri::command]
pub(crate) async fn get_xap_device(state: State<'_, AppState>) -> XAPResult<String> {
    state.do_action(|device| Ok(format!("{}", device))).await
}

#[tauri::command]
pub(crate) async fn get_secure_status(state: State<'_, AppState>) -> XAPResult<XAPSecureStatus> {
    state.do_query(XAPSecureStatusQuery {}).await
}

#[tauri::command]
pub(crate) async fn get_xap_version(state: State<'_, AppState>) -> XAPResult<XAPVersion> {
    state.do_query(XAPVersionQuery {}).await
}

#[tauri::command]
pub(crate) async fn get_rgblight_config(state: State<'_, AppState>) -> XAPResult<RGBLightConfig> {
    state.do_query(RGBLightConfigGet {}).await
}

#[tauri::command]
pub(crate) async fn get_rgblight_effects(state: State<'_, AppState>) -> XAPResult<Vec<u8>> {
    state
        .do_query(RGBLightEffectsQuery {})
        .await
        .map(|effects| effects.enabled_effect_list())
}

#[tauri::command]
pub(crate) async fn set_rgblight_config(
    arg: RGBLightConfig,
    state: State<'_, AppState>,
) -> XAPResult<()> {
    state.do_query(RGBLightConfigSet { config: arg }).await
}

#[tauri::command]
pub(crate) async fn save_rgblight_config(state: State<'_, AppState>) -> XAPResult<()> {
    state.do_query(RGBLightConfigSave {}).await
}
