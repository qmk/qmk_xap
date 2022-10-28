#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod types;
mod xap;

use anyhow::Result;
use log::{info, LevelFilter};
use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};
use tauri::State;
use xap::*;

pub(crate) struct AppState {
    device: Arc<Mutex<XAPDevice>>,
}

impl AppState {
    pub(crate) fn device(&self) -> MutexGuard<XAPDevice> {
        self.device
            .lock()
            .expect("couldn't acquire lock to XAP device")
    }
}

#[tauri::command]
fn get_xap_device(state: State<AppState>) -> String {
    format!("{}", state.device())
}

#[tauri::command]
fn get_secure_status(state: State<AppState>) -> XAPResult<XAPSecureStatus> {
    state.device().do_query(XAPSecureStatusQuery {})
}

#[tauri::command]
fn get_xap_version(state: State<AppState>) -> XAPResult<XAPVersion> {
    state.device().do_query(XAPVersionQuery {})
}

#[tauri::command]
fn get_rgblight_config(state: State<AppState>) -> XAPResult<RGBConfig> {
    state.device().do_query(RGBLightConfigGet {})
}

#[tauri::command]
fn get_rgblight_effects(state: State<AppState>) -> XAPResult<Vec<u8>> {
    state
        .device()
        .do_query(RGBLightEffectsQuery {})
        .map(|effects| effects.enabled_effect_list())
}

#[tauri::command]
fn set_rgblight_config(arg: RGBConfig, state: State<AppState>) -> XAPResult<()> {
    state.device().do_query(RGBLightConfigSet { config: arg })
}

#[tauri::command]
fn save_rgblight_config(state: State<AppState>) -> XAPResult<()> {
    state.device().do_query(RGBLightConfigSave {})
}

fn main() -> Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();

    let mut xap_client = XAPClient::new()?;

    info!("querying for compatible XAP devices");
    let device = loop {
        if let Ok(device) = xap_client.get_first_xap_device() {
            break device;
        } else {
            info!(".");
            std::thread::sleep(Duration::from_secs(1));
        }
    };

    tauri::Builder::default()
        .manage(AppState {
            device: Arc::new(Mutex::new(device)),
        })
        .invoke_handler(tauri::generate_handler![
            get_xap_device,
            get_secure_status,
            get_xap_version,
            get_rgblight_config,
            set_rgblight_config,
            save_rgblight_config,
            get_rgblight_effects
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
