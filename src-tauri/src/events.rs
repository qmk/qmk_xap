use serde::Serialize;
use specta::Type;
use tauri_specta::Event;
use uuid::Uuid;
use xap_specs::XAPSecureStatus;

use crate::aggregation::XAPDevice as XAPDeviceDTO;

#[derive(Clone, Serialize, Type, Event)]
#[serde(tag = "kind", content = "data")]
pub(crate) enum FrontendEvent {
    NewDevice {
        device: XAPDeviceDTO,
    },
    RemovedDevice {
        id: Uuid,
    },
    SecureStatusChanged {
        id: Uuid,
        secure_status: XAPSecureStatus,
    },
    LogReceived {
        id: Uuid,
        log: String,
    },
}

pub(crate) enum XAPEvent {
    LogReceived {
        id: Uuid,
        log: String,
    },
    SecureStatusChanged {
        id: Uuid,
        secure_status: XAPSecureStatus,
    },
    NewDevice(Uuid),
    RemovedDevice(Uuid),
    AnnounceAllDevices,
    RxError,
    Exit,
}
