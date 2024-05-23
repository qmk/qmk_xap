use serde::Serialize;
use specta::Type;
use tauri_specta::Event;
use uuid::Uuid;
use xap_specs::XapSecureStatus;

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
    NewDevice {
        id: Uuid,
    },
    RemovedDevice {
        id: Uuid,
    },
}

impl XapEvent {
    pub fn frontend_id(&self) -> &'static str {
        match self {
            XapEvent::LogReceived { .. } => "log",
            XapEvent::SecureStatusChanged { .. } => "secure-status-changed",
            XapEvent::NewDevice { .. } => "new-device",
            XapEvent::RemovedDevice { .. } => "removed-device",
        }
    }
}
