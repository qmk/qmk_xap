use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;
use xap_specs::protocol::{xap::XAPSecureStatus, BroadcastRaw};

use crate::aggregation::XAPDevice as XAPDeviceDTO;

#[derive(Clone, Serialize, TS)]
#[serde(tag = "kind", content = "data")]
#[ts(export)]
#[ts(export_to = "../bindings/")]
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
    ReceivedUserBroadcast {
        id: Uuid,
        broadcast: BroadcastRaw,
    },
    Exit,
}
