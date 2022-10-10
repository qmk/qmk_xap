#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod types;
mod xap_client;

use anyhow::{bail, Result};
use once_cell::sync::OnceCell;
use tauri::async_runtime::Mutex;

use xap_client::XAPClient;

static XAP_CLIENT: OnceCell<Mutex<XAPClient>> = OnceCell::new();

#[tauri::command]
async fn get_xap_devices() -> Vec<String> {
    let devices: Vec<String> = XAP_CLIENT.get().unwrap().lock().await.get_xap_devices();
    dbg!(devices)
}

fn main() -> Result<()> {
    match XAPClient::new() {
        Ok(api) => XAP_CLIENT
            .set(Mutex::new(api))
            .map_err(|_| anyhow::anyhow!("failed to instanciate mutex")),
        Err(err) => bail!("failed to create a USB HID instance with {err}"),
    }?;

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_xap_devices])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
