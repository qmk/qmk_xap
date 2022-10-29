#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
mod commands;
mod state;
mod xap;

use std::{sync::Arc, time::Duration};

use log::{info, LevelFilter};
use tokio::sync::Mutex;
use tokio::task;

use commands::*;
use state::AppState;
use xap::{XAPClient, XAPResult};

#[tokio::main]
async fn main() -> XAPResult<()> {
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

    let handle = task::spawn(async move {
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

    handle.await.unwrap();

    Ok(())
}
