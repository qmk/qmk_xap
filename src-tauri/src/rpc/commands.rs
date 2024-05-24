use std::result::Result;
use std::sync::{Arc, Mutex};

use tauri::State;
use uuid::Uuid;
use xap_specs::constants::XapConstants;

use crate::xap::device::XapDeviceState;
use crate::xap::{client::XapClient, device::Keymap, spec::remapping::RemappingSetKeycodeArg};

use crate::rpc::spec::error::Error;

#[tauri::command]
#[specta::specta]
pub fn xap_constants_get(state: State<'_, Arc<Mutex<XapClient>>>) -> XapConstants {
    state.lock().unwrap().xap_constants().into()
}

#[tauri::command]
#[specta::specta]
pub fn keycode_set(
    id: Uuid,
    arg: RemappingSetKeycodeArg,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> Result<(), Error> {
    Ok(state
        .lock()
        .unwrap()
        .get_device_mut(&id)?
        .set_keycode(arg)?)
}

#[tauri::command]
#[specta::specta]
pub fn keymap_get(id: Uuid, state: State<'_, Arc<Mutex<XapClient>>>) -> Result<Keymap, Error> {
    Ok(state.lock().unwrap().get_device(&id)?.keymap())
}

#[tauri::command]
#[specta::specta]
pub fn device_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> Result<XapDeviceState, Error> {
    Ok(state.lock().unwrap().get_device(&id)?.state().clone())
}
