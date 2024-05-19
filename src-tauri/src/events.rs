use serde::Serialize;
use specta::Type;
use tauri_specta::Event;
use uuid::Uuid;
use xap_specs::XapSecureStatus;

use crate::aggregation::XapDevice as XapDeviceDTO;

#[derive(Clone, Serialize, Type, Event)]
#[serde(tag = "kind", content = "data")]
pub(crate) enum FrontendEvent {
    NewDevice {
        device: XapDeviceDTO,
    },
    RemovedDevice {
        id: Uuid,
    },
    SecureStatusChanged {
        id: Uuid,
        secure_status: XapSecureStatus,
    },
    LogReceived {
        id: Uuid,
        log: String,
    },
}

pub(crate) enum XapEvent {
    LogReceived {
        id: Uuid,
        log: String,
    },
    SecureStatusChanged {
        id: Uuid,
        secure_status: XapSecureStatus,
    },
    NewDevice(Uuid),
    RemovedDevice(Uuid),
    AnnounceAllDevices,
    RxError,
    Exit,
}
