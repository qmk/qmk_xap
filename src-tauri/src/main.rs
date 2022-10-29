#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod types;
mod xap;

use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use log::{info, LevelFilter};
use tauri::State;
use tokio::sync::Mutex;
use xap::protocol::{
    RGBLightConfig, RGBLightConfigGet, RGBLightConfigSave, RGBLightConfigSet, RGBLightEffectsQuery,
    XAPResult, XAPSecureStatus, XAPSecureStatusQuery, XAPVersion, XAPVersionQuery,
};
use xap::{XAPClient, XAPDevice, XAPError, XAPRequest};

pub(crate) struct AppState {
    device: Arc<Mutex<Option<XAPDevice>>>,
    client: Arc<Mutex<XAPClient>>,
}

impl AppState {
    pub async fn do_query<T>(&self, request: T) -> XAPResult<T::Response>
    where
        T: XAPRequest,
    {
        let result = match &*self.device.lock().await {
            Some(device) => device.do_query(request),
            None => Err(XAPError::Other(anyhow!("Device not available"))),
        };

        if result.is_err() {
            self.process_error().await;
        }

        result
    }

    pub async fn do_action<T, F>(&self, action: F) -> XAPResult<T>
    where
        F: FnOnce(&XAPDevice) -> XAPResult<T>,
    {
        let result = match &*self.device.lock().await {
            Some(device) => action(device),
            None => Err(XAPError::Other(anyhow!("Device not available"))),
        };

        if result.is_err() {
            self.process_error().await;
        }

        result
    }

    // TODO MOVE THIS INTO SINGLETON STATE HANDLING INSTANCE
    async fn process_error(&self) {
        let mut client = self.client.lock().await;
        let mut device = self.device.lock().await;

        if let Some(inner_device) = &(*device) {
            if !client.is_device_connected(inner_device) {
                *device = None;
            }
        }
    }
}

#[tauri::command]
async fn get_xap_device(state: State<'_, AppState>) -> XAPResult<String> {
    state.do_action(|device| Ok(format!("{}", device))).await
}

#[tauri::command]
async fn get_secure_status(state: State<'_, AppState>) -> XAPResult<XAPSecureStatus> {
    state.do_query(XAPSecureStatusQuery {}).await
}

#[tauri::command]
async fn get_xap_version(state: State<'_, AppState>) -> XAPResult<XAPVersion> {
    state.do_query(XAPVersionQuery {}).await
}

#[tauri::command]
async fn get_rgblight_config(state: State<'_, AppState>) -> XAPResult<RGBLightConfig> {
    state.do_query(RGBLightConfigGet {}).await
}

#[tauri::command]
async fn get_rgblight_effects(state: State<'_, AppState>) -> XAPResult<Vec<u8>> {
    state
        .do_query(RGBLightEffectsQuery {})
        .await
        .map(|effects| effects.enabled_effect_list())
}

#[tauri::command]
async fn set_rgblight_config(arg: RGBLightConfig, state: State<'_, AppState>) -> XAPResult<()> {
    state.do_query(RGBLightConfigSet { config: arg }).await
}

#[tauri::command]
async fn save_rgblight_config(state: State<'_, AppState>) -> XAPResult<()> {
    state.do_query(RGBLightConfigSave {}).await
}

fn main() -> XAPResult<()> {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();

    let shared_device = Arc::new(Mutex::new(None));
    let client = Arc::new(Mutex::new(
        XAPClient::new().expect("couldn't create XAP client, aborting!"),
    ));

    let state = AppState {
        device: shared_device.clone(),
        client: client.clone(),
    };

    tauri::async_runtime::spawn(async move {
        info!("querying for compatible XAP devices");
        let device = loop {
            if let Ok(device) = client.lock().await.get_first_xap_device() {
                break device;
            } else {
                info!(".");
                std::thread::sleep(Duration::from_secs(1));
            }
        };

        shared_device.lock().await.replace(device);
    });

    tauri::Builder::default()
        .manage(state)
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
