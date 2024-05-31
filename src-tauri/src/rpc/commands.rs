use std::result::Result;
use std::sync::{Arc, Mutex};

use tauri::State;
use uuid::Uuid;
use xap_specs::constants::XapConstants;

use crate::aggregation::keymap::MappedKeymap;
use crate::xap::device::XapDeviceState;
use crate::xap::{client::XapClient, spec::remapping::RemappingSetKeycodeArg};

use crate::rpc::spec::error::Error;

#[tauri::command]
#[specta::specta]
pub fn xap_constants_get(state: State<'_, Arc<Mutex<XapClient>>>) -> XapConstants {
    state.lock().unwrap().xap_constants()
}

#[tauri::command]
#[specta::specta]
pub fn remap_key(
    id: Uuid,
    arg: RemappingSetKeycodeArg,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> Result<(), Error> {
    Ok(state.lock().unwrap().get_device_mut(&id)?.remap_key(arg)?)
}

#[tauri::command]
#[specta::specta]
pub fn keymap_get(
    id: Uuid,
    layout: String,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> Result<MappedKeymap, Error> {
    state
        .lock()
        .unwrap()
        .get_device(&id)?
        .keymap_with_layout(layout)
        .map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub fn device_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> Result<XapDeviceState, Error> {
    Ok(state.lock().unwrap().get_device(&id)?.state().clone())
}

#[tauri::command]
#[specta::specta]
pub fn devices_get(state: State<'_, Arc<Mutex<XapClient>>>) -> Vec<XapDeviceState> {
    state
        .lock()
        .unwrap()
        .get_devices()
        .iter()
        .map(|device| device.state())
        .cloned()
        .collect()
}
