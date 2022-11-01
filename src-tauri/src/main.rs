#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
mod commands;
mod xap;

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::tick;
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use log::{error, info, LevelFilter};
use parking_lot::Mutex;
use tauri::{
    plugin::{Builder, TauriPlugin},
    RunEvent, Runtime,
};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use commands::*;
use xap::{ResponseRaw, XAPClient, XAPError, XAPResult};

fn shutdown_event_loop<R: Runtime>(sender: Sender<XAPEvent>) -> TauriPlugin<R> {
    Builder::new("event loop shutdown")
        .on_event(move |_, event| match event {
            RunEvent::ExitRequested { .. } => {
                sender.send(XAPEvent::Exit).unwrap();
            }
            _ => {}
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

#[derive(Clone, serde::Serialize)]
pub(crate) enum FrontendEvent {
    NewDevice(Uuid),
    RemovedDevice(Uuid),
}

fn start_event_loop(
    app: AppHandle,
    state: Arc<Mutex<XAPClient>>,
    event_channel: Receiver<XAPEvent>,
) {
    let _ = std::thread::spawn(move || {
        let new_device_ticker = tick(Duration::from_millis(500));
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
                            info!("detected new device - notifying frontend!");
                            app.emit_all("new-device", FrontendEvent::NewDevice(id));
                        },
                        Ok(XAPEvent::RemovedDevice(id)) => {
                            info!("removed device - notifying frontend!");
                            app.emit_all("removed-device", FrontendEvent::RemovedDevice(id));
                        }
                        Ok(XAPEvent::RxError{id, error}) => {
                            info!("error for device {id} in receive thread : {error}");
                            state.lock().enumerate_xap_devices();
                        },
                        Err(err) => {
                            error!("error receiving event {err}");
                        },
                    }

                },
                recv(new_device_ticker) -> msg => {
                    match msg {
                        Ok(_) => {
                            state.lock().enumerate_xap_devices();
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
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();

    let (event_channel_tx, event_channel_rx): (Sender<XAPEvent>, Receiver<XAPEvent>) = unbounded();
    let state = Arc::new(Mutex::new(XAPClient::new(event_channel_tx.clone())));

    tauri::Builder::default()
        .plugin(shutdown_event_loop(event_channel_tx.clone()))
        .invoke_handler(tauri::generate_handler![
            get_xap_device,
            get_secure_status,
            get_xap_version,
            get_rgblight_config,
            set_rgblight_config,
            save_rgblight_config,
            get_rgblight_effects
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
