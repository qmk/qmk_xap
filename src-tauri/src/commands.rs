use parking_lot::Mutex;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::xap::{
    protocol::{
        RGBLightConfig, RGBLightConfigGet, RGBLightConfigSave, RGBLightConfigSet, XAPResult,
        XAPSecureStatus, XAPSecureStatusQuery,
    },
    XAPClient,
};

#[tauri::command]
pub(crate) async fn get_secure_status(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<XAPSecureStatus> {
    state.lock().query(id, XAPSecureStatusQuery {})
}

#[tauri::command]
pub(crate) async fn get_rgblight_config(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<RGBLightConfig> {
    state.lock().query(id, RGBLightConfigGet {})
}

#[tauri::command]
pub(crate) async fn set_rgblight_config(
    id: Uuid,
    arg: RGBLightConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, RGBLightConfigSet { config: arg })
}

#[tauri::command]
pub(crate) async fn save_rgblight_config(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, RGBLightConfigSave {})
}
