use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::xap::{
    protocol::{
        RGBLightConfig, RGBLightConfigGet, RGBLightConfigSave, RGBLightConfigSet, XAPResult,
    },
    XAPClient,
};

#[tauri::command]
pub(crate) async fn rgblight_config_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<RGBLightConfig> {
    state.lock().query(id, RGBLightConfigGet {})
}

#[tauri::command]
pub(crate) async fn rgblight_config_set(
    id: Uuid,
    arg: RGBLightConfig,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, RGBLightConfigSet { config: dbg!(arg) })
}

#[tauri::command]
pub(crate) async fn rgblight_config_save(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, dbg!(RGBLightConfigSave {}))
}
