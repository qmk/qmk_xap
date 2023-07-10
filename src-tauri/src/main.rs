#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
mod commands;
mod aggregation;
mod events;
mod xap;
mod xap_spec;

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::tick;
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use env_logger::Env;
use log::{error, info};
use parking_lot::Mutex;
use tauri::{
    plugin::{Builder, TauriPlugin},
    RunEvent, Runtime,
};
use tauri::{AppHandle, Manager};

use commands::{keycode_set, keymap_get, xap_constants_get};
use events::{FrontendEvent, XAPEvent};
use xap::hid::XAPClient;
use xap::ClientResult;
use xap_specs::constants::XAPConstants;

fn shutdown_event_loop<R: Runtime>(sender: Sender<XAPEvent>) -> TauriPlugin<R> {
    Builder::new("event loop shutdown")
        .on_event(move |_, event| {
            if let RunEvent::ExitRequested { .. } = event {
                sender.send(XAPEvent::Exit).unwrap();
            }
        })
        .build()
}

fn start_event_loop(
    app: AppHandle,
    state: Arc<Mutex<XAPClient>>,
    event_channel: Receiver<XAPEvent>,
) {
    _ = std::thread::spawn(move || {
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
                        Ok(XAPEvent::LogReceived{id, log}) => {
                            info!("LOG: {id} {log}");
                                app.emit_all("log", FrontendEvent::LogReceived{ id, log }).unwrap();
                        },
                        Ok(XAPEvent::SecureStatusChanged{id, secure_status}) => {
                            info!("Secure status changed: {id} - {secure_status}");
                            app.emit_all("secure-status-changed", FrontendEvent::SecureStatusChanged{ id, secure_status }).unwrap();
                        },
                        Ok(XAPEvent::NewDevice(id)) => {
                            if let Ok(device) = state.lock().get_device(&id){
                                info!("detected new device - notifying frontend!");
                                app.emit_all("new-device", FrontendEvent::NewDevice{ device: device.as_dto() }).unwrap();
                            }
                        },
                        Ok(XAPEvent::RemovedDevice(id)) => {
                            info!("removed device - notifying frontend!");
                            app.emit_all("removed-device", FrontendEvent::RemovedDevice{ id }).unwrap();
                        },
                        Ok(XAPEvent::AnnounceAllDevices) => {
                            let mut state = state.lock();
                            info!("announcing all xap devices to the frontend");
                            if let Ok(()) = state.enumerate_xap_devices() {
                                for device in state.get_devices() {
                                    app.emit_all("new-device", FrontendEvent::NewDevice{ device: device.as_dto() }).unwrap();
                                }
                            }
                        },
                        Ok(XAPEvent::RxError) => {
                            if let Err(err) = state.lock().enumerate_xap_devices() {
                                error!("failed to enumerate XAP devices: {err}");
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
                                error!("failed to enumerate XAP devices: {err}");
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

fn main() -> ClientResult<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

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

    let (event_channel_tx, event_channel_rx): (Sender<XAPEvent>, Receiver<XAPEvent>) = unbounded();

    tauri::Builder::default()
        .invoke_handler(xap_handler)
        .plugin(shutdown_event_loop(Sender::clone(&event_channel_tx)))
        .setup(move |app| {
            xap_events(app);

            let xap_specs = app
                .path_resolver()
                .resolve_resource("../xap-specs/specs/constants/keycodes")
                .expect("unable to find XAP specifications");

            let state = Arc::new(Mutex::new(
                XAPClient::new(
                    Sender::clone(&event_channel_tx),
                    XAPConstants::new(xap_specs)?,
                )
                .expect("failed to initialize XAP state"),
            ));
            app.manage(Arc::clone(&state));

            start_event_loop(app.handle(), state, event_channel_rx);

            app.listen_global("frontend-loaded", move |_| {
                let event_tx = event_channel_tx.clone();
                event_tx.send(XAPEvent::AnnounceAllDevices).unwrap();
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running QMK XAP client");

    Ok(())
}
