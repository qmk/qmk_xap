#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod types;

use anyhow::{bail, Result};
use hidapi::HidApi;
use once_cell::sync::OnceCell;
use tauri::async_runtime::Mutex;

use types::Devices;

static HID_API: OnceCell<Mutex<HidApi>> = OnceCell::new();

#[tauri::command]
async fn get_hid_device() -> Vec<String> {
    let devices: Vec<String> = HID_API
        .get()
        .unwrap()
        .lock()
        .await
        .device_list()
        .map(|device| {
            format!(
                "VID: {:04x}, PID: {:04x}, Serial: {}, Product name: {}, Manufacturer: {}",
                device.vendor_id(),
                device.product_id(),
                match device.serial_number() {
                    Some(s) => s,
                    _ => "<COULD NOT FETCH>",
                },
                match device.product_string() {
                    Some(s) => s,
                    _ => "<COULD NOT FETCH>",
                },
                match device.manufacturer_string() {
                    Some(s) => s,
                    _ => "<COULD_NOT_FETCH>",
                }
            )
        })
        .collect();

    dbg!(devices)
    // Ok(Devices { list: devices })
}

fn main() -> Result<()> {
    match HidApi::new() {
        Ok(api) => HID_API
            .set(Mutex::new(api))
            .map_err(|_| anyhow::anyhow!("failed to instanciate mutex")),
        Err(err) => bail!("failed to create a USB HID instance with {err}"),
    }?;

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_hid_device])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
