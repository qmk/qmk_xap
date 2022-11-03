#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
mod commands;
mod xap;

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::tick;
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use env_logger::Env;
use log::{error, info};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{
    plugin::{Builder, TauriPlugin},
    RunEvent, Runtime,
};
use tauri::{AppHandle, Manager};
use ts_rs::TS;
use uuid::Uuid;

use commands::*;
use xap::{ResponseRaw, XAPClient, XAPDeviceInfo, XAPError, XAPResult};

fn shutdown_event_loop<R: Runtime>(sender: Sender<XAPEvent>) -> TauriPlugin<R> {
    Builder::new("event loop shutdown")
        .on_event(move |_, event| {
            if let RunEvent::ExitRequested { .. } = event {
                sender.send(XAPEvent::Exit).unwrap();
            }
        })
        .build()
}

pub(crate) enum XAPEvent {
    Broadcast { id: Uuid, response: ResponseRaw },
    NewDevice(Uuid),
    RemovedDevice(Uuid),
    RxError { id: Uuid, error: XAPError },
    Exit,
}

#[derive(Clone, Serialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub(crate) enum FrontendEvent {
    NewDevice { id: String, device: XAPDeviceInfo },
    RemovedDevice { id: String },
}

fn start_event_loop(
    app: AppHandle,
    state: Arc<Mutex<XAPClient>>,
    event_channel: Receiver<XAPEvent>,
) {
    let _ = std::thread::spawn(move || {
        let ticker = tick(Duration::from_millis(500));
        let state = state;
        info!("started event loop");
        'event_loop: loop {
            select! {
                recv(event_channel) -> msg => {
                    match msg {
                        Ok(XAPEvent::Exit) => {
                            info!("received shutdown signal, exiting!");
                            break 'event_loop;
                        },
                        Ok(XAPEvent::Broadcast{..}) => {
                            info!("received XAP broadcast - forwarding to frontend!");
                        },
                        Ok(XAPEvent::NewDevice(id)) => {
                            if let Some(device) = state.lock().get_device(&id){
                                info!("detected new device - notifying frontend!");
                                let info = device.xap_info();
                                app.emit_all("new-device", FrontendEvent::NewDevice{id: id.to_string(), device: info.clone()}).unwrap();
                            }
                        },
                        Ok(XAPEvent::RemovedDevice(id)) => {
                            info!("removed device - notifying frontend!");
                            app.emit_all("removed-device", FrontendEvent::RemovedDevice{ id: id.to_string() }).unwrap();
                        }
                        Ok(XAPEvent::RxError{id, error}) => {
                            info!("error for device {id} in receive thread : {error}");
                            if let Err(err) = state.lock().enumerate_xap_devices(){
                                error!("failed to enumerate XAP devices: {err}:\n {:#?}", err.source());
                            }
                        },
                        Err(err) => {
                            error!("error receiving event {err}");
                        },
                    }

                },
                recv(ticker) -> msg => {
                    match msg {
                        Ok(_) => {
                            // TODO maybe this can be done in a more resource effective manner...
                            let mut state = state.lock();
                            if let Err(err) = state.enumerate_xap_devices() {
                                error!("failed to enumerate XAP devices: {err}:\n {:#?}", err.source());
                            }
                            for device in state.get_devices() {
                                let info = device.xap_info();
                                app.emit_all("new-device", FrontendEvent::NewDevice{id: device.id().to_string(), device: info.clone()}).unwrap();
                            }
                        },
                        Err(err) => {
                            error!("failed receiving tick {err}");
                        }
                    }
                }
            }
        }
    });
}

fn main() -> XAPResult<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let (event_channel_tx, event_channel_rx): (Sender<XAPEvent>, Receiver<XAPEvent>) = unbounded();
    let state = Arc::new(Mutex::new(XAPClient::new(event_channel_tx.clone())?));

    tauri::Builder::default()
        .plugin(shutdown_event_loop(event_channel_tx))
        .invoke_handler(tauri::generate_handler![
            get_secure_status,
            get_rgblight_config,
            set_rgblight_config,
            save_rgblight_config
        ])
        .setup(move |app| {
            app.manage(state.clone());
            start_event_loop(app.handle(), state, event_channel_rx);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
