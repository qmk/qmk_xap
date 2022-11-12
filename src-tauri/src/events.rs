use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::aggregation::XAPDevice as XAPDeviceDTO;
use crate::xap::XAPSecureStatus;

#[derive(Clone, Serialize, TS)]
#[serde(tag = "kind", content = "data")]
#[ts(export)]
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
