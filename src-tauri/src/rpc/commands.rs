use std::result::Result;
use std::sync::{Arc, Mutex};

use tauri::State;
use uuid::Uuid;
use xap_specs::constants::keycode::KeyCode;
use xap_specs::constants::keycode_encoder::KeycodeTemplate;
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

#[tauri::command]
#[specta::specta]
pub fn keycode_template_encode(template: KeycodeTemplate) -> Result<u16, Error> {
    template
        .encode()
        .ok_or_else(|| Error("keycode template is incomplete".to_owned()))
}

/// Decode a wire keycode (u16) into a fully resolved `KeyCode` entry -- same
/// catalog lookup the keymap fetch uses, exposed so per-key reads (e.g. the
/// encoder map view) don't need to re-implement the lookup in TypeScript.
#[tauri::command]
#[specta::specta]
pub fn decode_keycode(code: u16, state: State<'_, Arc<Mutex<XapClient>>>) -> KeyCode {
    state.lock().unwrap().xap_constants().get_keycode(code)
}

/// Bulk-fetch the full encoder keymap in one Tauri round-trip. Sweeps every
/// (layer, encoder, clockwise) slot via `KeymapGetEncoderKeycode`, decodes
/// each result against the keycode catalog, and returns the
/// `[layer][encoder][clockwise]` tensor. The Rust side logs total wallclock
/// + per-call average so encoder fetches show up alongside the keymap fetch
/// in startup-timing profiles.
///
/// Layer count is taken from `KeymapInfo.layer_count` (or `RemapInfo.layer_count`
/// as a fallback); encoder count comes from the QMK config blob's
/// `encoder.rotary` array. Returns an empty `Vec` when either is zero.
#[tauri::command]
#[specta::specta]
pub fn encoder_keymap_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XapClient>>>,
) -> Result<Vec<Vec<Vec<KeyCode>>>, Error> {
    let mut client = state.lock().unwrap();
    let device = client.get_device_mut(&id)?;
    let layer_count = device
        .state()
        .info
        .as_ref()
        .and_then(|i| i.keymap.as_ref().and_then(|k| k.layer_count))
        .or_else(|| {
            device
                .state()
                .info
                .as_ref()
                .and_then(|i| i.remap.as_ref().and_then(|r| r.layer_count))
        })
        .unwrap_or(0);
    let encoder_count = u8::try_from(device.state().config.encoder.rotary.len()).unwrap_or(u8::MAX);
    if layer_count == 0 || encoder_count == 0 {
        return Ok(Vec::new());
    }
    device
        .query_encoder_keymap(layer_count, encoder_count)
        .map_err(Into::into)
}
