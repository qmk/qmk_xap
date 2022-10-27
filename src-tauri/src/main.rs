#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod protocol;
mod types;
mod xap;

use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use log::{info, LevelFilter};
use tauri::State;

use protocol::{XAPSecureStatus, XAPVersion, XAPVersionQuery, XAPResult};
use xap::{XAPClient, XAPDevice};

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

use crate::protocol::{RequestRaw, XAPSecureStatusQuery};

#[tauri::command]
fn get_xap_device(state: State<AppState>) -> String {
    format!("{}", state.device())
}

#[tauri::command]
fn get_secure_status(state: State<AppState>) -> XAPResult<XAPSecureStatus> {
    let device = state.device.lock().unwrap();
    dbg!(device.do_query(XAPSecureStatusQuery {}))
}

#[tauri::command]
fn get_xap_version(state: State<AppState>) -> XAPResult<XAPVersion> {
    dbg!(state.device().do_query(XAPVersionQuery {}))
}

#[tauri::command]
fn set_rgblight(state: State<AppState>) -> XAPResult<()> {
    dbg!(state.device().set_rgblight_config())
}

fn main() -> XAPResult<()> {
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
            set_rgblight
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
