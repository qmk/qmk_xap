#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod protocol;
mod types;
mod xap;

use std::time::Duration;

use anyhow::Result;
use log::{info, LevelFilter};
use once_cell::sync::OnceCell;
use tauri::async_runtime::Mutex;

use xap::{XAPClient, XAPDevice};

static XAP_DEVICE: OnceCell<Mutex<XAPDevice>> = OnceCell::new();

#[tauri::command]
async fn get_xap_device() -> Option<String> {
    let device = XAP_DEVICE.get().unwrap().lock().await;

    Some(format!("{}", device))
}

#[tauri::command]
async fn get_secure_status() -> Option<()> {
    let device = XAP_DEVICE.get().unwrap().lock().await;
    device.query_secure_status().unwrap();
    Some(())
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

    XAP_DEVICE
        .set(Mutex::new(device))
        .expect("couldn't move XAP device into Mutex");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_xap_device])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
