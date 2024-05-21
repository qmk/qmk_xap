use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crossbeam_channel::Sender;
use hidapi::{DeviceInfo, HidApi};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;
use xap_specs::{constants::XapConstants, error::XapError, request::XapRequest};

use crate::XapEvent;

use super::device::XapDevice;

const XAP_USAGE_PAGE: u16 = 0xFF51;
const XAP_USAGE: u16 = 0x0058;

pub(crate) struct XapClient {
    hid: HidApi,
    devices: HashMap<Uuid, XapDevice>,
    event_channel: Sender<XapEvent>,
    constants: Arc<XapConstants>,
}

impl Debug for XapClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("device", &self.devices)
            .finish()
    }
}

impl XapClient {
    pub fn new(event_channel: Sender<XapEvent>, xap_constants: XapConstants) -> XapClientResult<Self> {
        Ok(Self {
            devices: HashMap::new(),
            hid: HidApi::new_without_enumerate()?,
            event_channel,
            constants: Arc::new(xap_constants),
        })
    }

    pub fn query<T>(&self, id: Uuid, request: T) -> XapClientResult<T::Response>
    where
        T: XapRequest,
    {
        match self.devices.get(&id) {
            Some(device) => device.query(request),
            None => Err(XapClientError::UnknownDevice(id)),
        }
    }

    pub fn xap_constants(&self) -> XapConstants {
        self.constants.as_ref().clone()
    }

    pub fn enumerate_xap_devices(&mut self) -> XapClientResult<()> {
        // 1. Device already enumerated - don't start new capturing thread (announce nothing)
        // 2. Device already enumerated but error occured - remove old device and restart device (announce removal + announce new device)
        // 3. Device not enumerated - add device and start capturing (announce new device)
        self.hid.refresh_devices()?;

        let xap_devices: Vec<DeviceInfo> = self
            .hid
            .device_list()
            .filter(|info| info.usage_page() == XAP_USAGE_PAGE && info.usage() == XAP_USAGE)
            .cloned()
            .collect();

        self.devices.retain(|id, known_device| {
            if known_device.is_running()
                || xap_devices
                    .iter()
                    .any(|candidate| known_device.is_hid_device(candidate))
            {
                true
            } else {
                self.event_channel
                    .send(XapEvent::RemovedDevice(*id))
                    .expect("failed to announce removal of xap device");
                false
            }
        });

        for device in xap_devices {
            if self
                .devices
                .iter()
                .any(|(_, known_device)| known_device.is_hid_device(&device))
            {
                continue;
            }

            let new_device = XapDevice::new(
                device.clone(),
                Arc::clone(&self.constants),
                self.event_channel.clone(),
                device.open_device(&self.hid)?,
                device.open_device(&self.hid)?,
            )?;
            let id = new_device.id();
            self.devices.insert(id, new_device);
            self.event_channel
                .send(XapEvent::NewDevice(id))
                .expect("failed to announce new xap device");
        }

        Ok(())
    }

    pub fn get_device(&self, id: &Uuid) -> XapClientResult<&XapDevice> {
        self.devices.get(id).ok_or(XapClientError::UnknownDevice(*id))
    }

    pub fn get_device_mut(&mut self, id: &Uuid) -> XapClientResult<&mut XapDevice> {
        self.devices
            .get_mut(id)
            .ok_or(XapClientError::UnknownDevice(*id))
    }

    pub fn get_devices(&self) -> Vec<&XapDevice> {
        self.devices.values().collect()
    }
}

pub type XapClientResult<T> = Result<T, XapClientError>;

#[derive(Error, Debug)]
pub enum XapClientError {
    #[error("HID communication failed {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("unkown device {0}")]
    UnknownDevice(Uuid),
    #[error("JSON (de)serialization error {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("HJSON (de)serialization error {0}")]
    HJSONError(#[from] deser_hjson::Error),
    #[error("unknown error {0}")]
    Other(#[from] anyhow::Error),
    #[error("XAP protocol error {0}")]
    ProtocolError(#[from] XapError),
}

impl Serialize for XapClientError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
