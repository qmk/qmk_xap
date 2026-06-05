use serde::Serialize;
use specta::Type;
use tauri_specta::Event;
use uuid::Uuid;
use xap_specs::XapSecureStatus;

#[derive(Clone, Copy, Serialize, Type)]
pub enum RawBroadcastType {
    Keyboard,
    User,
}

#[derive(Clone, Serialize, Type, Event)]
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
    RawBroadcastReceived {
        id: Uuid,
        broadcast_type: RawBroadcastType,
        payload: Vec<u8>,
    },
    NewDevice {
        id: Uuid,
    },
    RemovedDevice {
        id: Uuid,
    },
}
