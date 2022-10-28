#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod types;
mod xap;

use std::{sync::Arc, time::Duration};

use log::{info, LevelFilter};
use protocol::{
    RGBConfig, RGBLightConfigGet, RGBLightConfigSave, RGBLightConfigSet, RGBLightEffectsQuery,
    XAPError, XAPSecureStatusQuery,
};
use protocol::{XAPResult, XAPSecureStatus, XAPVersion, XAPVersionQuery};
use tauri::{Manager, State};
use tokio::sync::{Mutex, MutexGuard};
use xap::{XAPClient, XAPDevice};

pub(crate) struct AppState {
    device: Arc<Mutex<XAPResult<XAPDevice>>>,
}

#[tauri::command]
async fn get_xap_device(state: State<AppState>) -> XAPResult<String> {
    Ok(format!("{}", *(state.device.lock().await)?))
}

#[tauri::command]
async fn get_secure_status(state: State<AppState>) -> XAPResult<XAPSecureStatus> {
    state.device()?.do_query(XAPSecureStatusQuery {})
}

#[tauri::command]
async fn get_xap_version(state: State<AppState>) -> XAPResult<XAPVersion> {
    state.device().do_query(XAPVersionQuery {})
}

#[tauri::command]
async fn get_rgblight_config(state: State<AppState>) -> XAPResult<RGBConfig> {
    state.device().do_query(RGBLightConfigGet {})
}

#[tauri::command]
async fn get_rgblight_effects(state: State<AppState>) -> XAPResult<Vec<u8>> {
    state
        .device()
        .do_query(RGBLightEffectsQuery {})
        .map(|effects| effects.enabled_effect_list())
}

#[tauri::command]
async fn set_rgblight_config(arg: RGBConfig, state: State<AppState>) -> XAPResult<()> {
    state.device().do_query(RGBLightConfigSet { config: arg })
}

#[tauri::command]
async fn save_rgblight_config(state: State<AppState>) -> XAPResult<()> {
    state.device().do_query(RGBLightConfigSave {})
}

fn main() -> XAPResult<()> {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();

    let mut xap_client = XAPClient::new().expect("couldn't create XAP client, aborting!");

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
        .setup(|app| {
            let handle = app.app_handle();
            let splashscreen_window = app.get_window("splashscreen").unwrap();
            let main_window = app.get_window("main").unwrap();
            tauri::async_runtime::spawn(async move {
                handle.manage(AppState {
                    device: Arc::new(Mutex::new(None)),
                });
                splashscreen_window.close().unwrap();
                main_window.show().unwrap();
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
