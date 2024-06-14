use std::ops::Deref;
use std::result::Result;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use uuid::Uuid;
use wasm_bindgen::prelude::wasm_bindgen;
use xap_specs::constants::XapConstants;

use crate::aggregation::keymap::MappedKeymap;
use crate::xap::device::XapDeviceState;
use crate::xap::{client::XapClient, spec::remapping::RemappingSetKeycodeArg};

use crate::rpc::spec::error::Error;

use crate::XAP_STATE;

#[wasm_bindgen]
pub fn xap_constants_get() -> XapConstants {
    XAP_STATE.get().unwrap().lock().unwrap().xap_constants()
}

//
// #[tauri::command]
// #[specta::specta]
// pub fn remap_key(
//     id: Uuid,
//     arg: RemappingSetKeycodeArg,
//     state: State<'_, Arc<Mutex<XapClient>>>,
// ) -> Result<(), Error> {
//     Ok(state.lock().unwrap().get_device_mut(&id)?.remap_key(arg)?)
// }
//
// #[tauri::command]
// #[specta::specta]
// pub fn keymap_get(
//     id: Uuid,
//     layout: String,
//     state: State<'_, Arc<Mutex<XapClient>>>,
// ) -> Result<MappedKeymap, Error> {
//     state
//         .lock()
//         .unwrap()
//         .get_device(&id)?
//         .keymap_with_layout(layout)
//         .map_err(Into::into)
// }
//
//

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Id(Uuid);

impl Deref for Id {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[wasm_bindgen]
pub fn device_get(id: Id) -> Result<XapDeviceState, Error> {
    Ok(XAP_STATE
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .get_device(&id)?
        .state()
        .clone())
}

// #[wasm_bindgen]
// pub fn devices_get() -> Vec<XapDeviceState> {
//     let result: Vec<XapDeviceState> = XAP_STATE
//         .get()
//         .unwrap()
//         .lock()
//         .unwrap()
//         .get_devices()
//         .iter()
//         .map(|device| device.state())
//         .cloned()
//         .collect();
//
//     serde_wasm_bindgen::to_value(&result).unwrap()
// }
//
