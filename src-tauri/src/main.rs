#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod aggregation;
mod rpc;
mod xap;

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::tick;
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use env_logger::Env;
use log::{error, info};
use parking_lot::Mutex;
use tauri::path::BaseDirectory;
use tauri::{
    plugin::{Builder, TauriPlugin},
    RunEvent, Runtime,
};
use tauri::{AppHandle, Manager};

use rpc::events::{FrontendEvent, XapEvent};
use rpc::commands::{keycode_set, keymap_get, xap_constants_get};
use xap_specs::constants::XapConstants;
use xap::client::XapClient;
use xap::client::XapClientResult;

fn shutdown_event_loop<R: Runtime>(sender: Sender<XapEvent>) -> TauriPlugin<R> {
    Builder::new("event loop shutdown")
        .on_event(move |_, event| {
            if let RunEvent::ExitRequested { .. } = event {
                sender.send(XapEvent::Exit).unwrap();
            }
        })
        .build()
}

fn start_event_loop(
    app: AppHandle,
    state: Arc<Mutex<XapClient>>,
    event_channel: Receiver<XapEvent>,
) {
    _ = std::thread::spawn(move || {
        let ticker = tick(Duration::from_millis(500));
        let state = state;
        info!("started event loop");
        'event_loop: loop {
            select! {
                recv(event_channel) -> msg => {
                    match msg {
                        Ok(XapEvent::Exit) => {
                            info!("received shutdown signal, exiting!");
                            break 'event_loop;
                        },
                        Ok(XapEvent::LogReceived{id, log}) => {
                            info!("LOG: {id} {log}");
                                app.emit("log", FrontendEvent::LogReceived{ id, log }).unwrap();
                        },
                        Ok(XapEvent::SecureStatusChanged{id, secure_status}) => {
                            info!("Secure status changed: {id} - {secure_status}");
                            app.emit("secure-status-changed", FrontendEvent::SecureStatusChanged{ id, secure_status }).unwrap();
                        },
                        Ok(XapEvent::NewDevice(id)) => {
                            if let Ok(device) = state.lock().get_device(&id){
                                info!("detected new device - notifying frontend!");
                                app.emit("new-device", FrontendEvent::NewDevice{ device: device.as_dto() }).unwrap();
                            }
                        },
                        Ok(XapEvent::RemovedDevice(id)) => {
                            info!("removed device - notifying frontend!");
                            app.emit("removed-device", FrontendEvent::RemovedDevice{ id }).unwrap();
                        },
                        Ok(XapEvent::AnnounceAllDevices) => {
                            let mut state = state.lock();
                            info!("announcing all xap devices to the frontend");
                            if let Ok(()) = state.enumerate_xap_devices() {
                                for device in state.get_devices() {
                                    app.emit("new-device", FrontendEvent::NewDevice{ device: device.as_dto() }).unwrap();
                                }
                            }
                        },
                        Ok(XapEvent::RxError) => {
                            if let Err(err) = state.lock().enumerate_xap_devices() {
                                error!("failed to enumerate Xap devices: {err}");
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
                            if let Err(err) = state.lock().enumerate_xap_devices() {
                                error!("failed to enumerate Xap devices: {err}");
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

fn main() -> XapClientResult<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let specta_config = specta::ts::ExportConfig::default()
        .bigint(specta::ts::BigIntExportBehavior::BigInt)
        .formatter(specta::ts::formatter::prettier);

    let mut specta_builder =
        generate_specta_builder!(commands: [xap_constants_get, keycode_set, keymap_get], events: [FrontendEvent])
            .config(specta_config);

    if cfg!(debug_assertions) {
        specta_builder = specta_builder.path("../src/generated/xap.ts");
    }

    let (xap_handler, xap_events) = specta_builder.build().expect("failed to build specta");

    let (event_channel_tx, event_channel_rx): (Sender<XapEvent>, Receiver<XapEvent>) = unbounded();

    tauri::Builder::default()
        .invoke_handler(xap_handler)
        .plugin(shutdown_event_loop(Sender::clone(&event_channel_tx)))
        .setup(move |app| {
            xap_events(app);

            let xap_specs = app
                .path()
                .resolve("../xap-specs/specs", BaseDirectory::Resource)?;

            let state = Arc::new(Mutex::new(XapClient::new(
                Sender::clone(&event_channel_tx),
                XapConstants::new(xap_specs)?,
            )?));

            app.manage(Arc::clone(&state));

            start_event_loop(app.handle().clone(), state, event_channel_rx);

            app.listen("frontend-loaded", move |_| {
                info!("frontend loaded - announcing all devices");
                let event_tx = event_channel_tx.clone();
                event_tx.send(XapEvent::AnnounceAllDevices).unwrap();
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running QMK XAP client");

    Ok(())
}
