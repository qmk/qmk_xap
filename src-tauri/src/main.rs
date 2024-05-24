#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod aggregation;
mod rpc;
mod xap;

use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::Result;
use env_logger::Env;
use log::{error, info};
use tauri::path::BaseDirectory;
use tauri::{
    plugin::{Builder, TauriPlugin},
    RunEvent, Runtime,
};
use tauri::{AppHandle, Manager};

use rpc::commands::{device_get, keymap_get, remap_key, xap_constants_get};
use rpc::events::XapEvent;
use xap::client::XapClient;

use xap_specs::constants::XapConstants;

enum InternalEvent {
    Shutdown,
    FrontendNotify,
}

fn shutdown_event_loop<R: Runtime>(tx: Sender<InternalEvent>) -> TauriPlugin<R> {
    Builder::new("event loop shutdown")
        .on_event(move |_, event| {
            if let RunEvent::ExitRequested { .. } = event {
                tx.send(InternalEvent::Shutdown).unwrap();
            }
        })
        .build()
}

struct App {
    handle: AppHandle,
    state: Arc<Mutex<XapClient>>,
    event_channel: Receiver<InternalEvent>,
}

impl App {
    fn new(
        handle: AppHandle,
        state: Arc<Mutex<XapClient>>,
        event_channel: Receiver<InternalEvent>,
    ) -> Self {
        Self {
            handle,
            state,
            event_channel,
        }
    }

    fn start_event_loop(&mut self) {
        info!("started event loop");

        let mut last_enumeration = Instant::now();

        loop {
            match self.event_channel.try_recv() {
                Ok(InternalEvent::Shutdown) => {
                    info!("shutting down event loop");
                    return;
                }
                Ok(InternalEvent::FrontendNotify) => {
                    for device in self.state.lock().unwrap().get_devices() {
                        let event = XapEvent::NewDevice { id: device.id() };
                        self.handle.emit(event.frontend_id(), event).unwrap();
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    error!("event loop channel disconnected");
                    return;
                }
                _ => {}
            }

            if last_enumeration.elapsed() > Duration::from_secs(1) {
                last_enumeration = Instant::now();

                match self.state.lock().unwrap().enumerate_xap_devices() {
                    Ok(events) => {
                        for event in events {
                            self.handle_event(event);
                        }
                    }
                    Err(err) => {
                        error!("failed to enumerate XAP devices: {err}");
                    }
                }
            }

            match self.state.lock().unwrap().poll_devices() {
                Ok(events) => {
                    for event in events {
                        self.handle_event(event);
                    }
                }
                Err(err) => {
                    error!("failed to poll XAP devices: {err}");
                }
            }
            sleep(std::time::Duration::from_millis(100));
        }
    }

    fn handle_event(&self, event: XapEvent) {
        self.handle.emit(event.frontend_id(), event).unwrap();
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let specta_config = specta::ts::ExportConfig::default()
        .bigint(specta::ts::BigIntExportBehavior::BigInt)
        .formatter(specta::ts::formatter::prettier);

    let mut specta_builder =
        generate_specta_builder!(commands: [xap_constants_get, remap_key, keymap_get, device_get], events: [XapEvent])
            .config(specta_config);

    if cfg!(debug_assertions) {
        specta_builder = specta_builder.path("../src/generated/xap.ts");
    }

    let (xap_handler, xap_events) = specta_builder.build()?;
    let (tx, rx) = channel();

    tauri::Builder::default()
        .invoke_handler(xap_handler)
        .plugin(shutdown_event_loop(tx.clone()))
        .setup(move |app| {
            xap_events(app);

            let xap_specs = app
                .path()
                .resolve("../xap-specs/assets", BaseDirectory::Resource)?;

            let state = Arc::new(Mutex::new(XapClient::new(XapConstants::new(xap_specs)?)?));

            app.manage(Arc::clone(&state));

            let handle = app.handle().clone();
            std::thread::spawn(|| App::new(handle, state, rx).start_event_loop());

            app.listen("frontend-loaded", move |_| {
                tx.clone()
                    .send(InternalEvent::FrontendNotify)
                    .expect("failed to notify frontend");
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running QMK XAP client");

    Ok(())
}
