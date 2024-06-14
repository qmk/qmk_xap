use serde::Serialize;
// 
// use tauri_specta::Event;
use uuid::Uuid;
use xap_specs::XapSecureStatus;

#[derive(Clone, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum XapEvent {
    LogReceived {
        id: Uuid,
        log: String,
    },
    SecureStatusChanged {
        id: Uuid,
        secure_status: XapSecureStatus,
    },
    NewDevice {
        id: Uuid,
    },
    RemovedDevice {
        id: Uuid,
    },
}
