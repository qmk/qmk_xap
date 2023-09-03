#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
mod commands;
mod aggregation;
mod events;
mod user;
mod xap;

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

use commands::*;
use events::{FrontendEvent, XAPEvent};
use user::UserData;
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
    user_data: Arc<Mutex<UserData>>,
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
                            let client = state.lock();
                            let mut user_data = user_data.lock();
                            user::on_close(&client, &mut user_data);
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

                                let mut user_data = user_data.lock();
                                user::new_device(device, &mut user_data);
                            }
                        },
                        Ok(XAPEvent::RemovedDevice(id)) => {
                            info!("removed device - notifying frontend!");
                            app.emit_all("removed-device", FrontendEvent::RemovedDevice{ id }).unwrap();

                            let mut user_data = user_data.lock();
                            user::removed_device(&id, &mut user_data);
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
                        Ok(XAPEvent::ReceivedUserBroadcast{id, broadcast}) => {
                            let state = state.lock();
                            let device = state.get_device(&id).unwrap();
                            let mut user_data = user_data.lock();
                            user::broadcast_callback(broadcast, device, &mut user_data);
                        },
                        Err(err) => {
                            error!("error receiving event {err}");
                        },
                    }

                },
                recv(ticker) -> msg => {
                    match msg {
                        Ok(_) => {
                            let mut state = state.lock();

                            if let Err(err) = state.enumerate_xap_devices() {
                                error!("failed to enumerate XAP devices: {err}");
                                return;
                            }

                            let mut user_data = user_data.lock();
                            user::housekeeping(&state, &mut user_data);
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

    let user_data = Arc::new(Mutex::new(UserData::default()));

    user::pre_init();

    let (event_channel_tx, event_channel_rx): (Sender<XAPEvent>, Receiver<XAPEvent>) = unbounded();

    tauri::Builder::default()
        .plugin(shutdown_event_loop(Sender::clone(&event_channel_tx)))
        .invoke_handler(tauri::generate_handler![
            xap_constants_get,
            secure_lock,
            secure_unlock,
            secure_status_get,
            jump_to_bootloader,
            reset_eeprom,
            keycode_get,
            keycode_set,
            keymap_get,
            encoder_keycode_get,
            encoder_keycode_set,
            backlight_config_get,
            backlight_config_set,
            backlight_config_save,
            rgblight_config_get,
            rgblight_config_set,
            rgblight_config_save,
            rgbmatrix_config_get,
            rgbmatrix_config_set,
            rgbmatrix_config_save,
        ])
        .setup(move |app| {
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

            start_event_loop(app.handle(), state, event_channel_rx, user_data);

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
