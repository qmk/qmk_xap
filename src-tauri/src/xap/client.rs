use std::{collections::HashMap, fmt::Debug, sync::Arc};

use anyhow::{anyhow, Result};
use hidapi::{DeviceInfo, HidApi};
use log::error;
use uuid::Uuid;

use xap_specs::{
    broadcast::{BroadcastType, LogBroadcast},
    constants::XapConstants,
    request::XapRequest,
};

use crate::XapEvent;

use super::device::XapDevice;

const XAP_USAGE_PAGE: u16 = 0xFF51;
const XAP_USAGE: u16 = 0x0058;

pub(crate) struct XapClient {
    hid: HidApi,
    devices: HashMap<Uuid, XapDevice>,
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
    pub fn new(xap_constants: XapConstants) -> Result<Self> {
        Ok(Self {
            devices: HashMap::new(),
            hid: HidApi::new_without_enumerate()?,
            constants: Arc::new(xap_constants),
        })
    }

    pub fn poll_devices(&mut self) -> Result<Vec<XapEvent>> {
        // TODO: implement as callback functions?
        let mut events = Vec::new();
        for device in self.devices.values_mut() {
            device.poll()?;

            while let Some(broadcast) = device.broadcast_queue.pop_front() {
                match broadcast.broadcast_type() {
                    BroadcastType::Log => {
                        let log: LogBroadcast = broadcast.into_xap_broadcast()?;
                        events.push(XapEvent::LogReceived {
                            id: device.id(),
                            log: log.0,
                        });
                    }
                    BroadcastType::SecureStatus => {
                        events.push(XapEvent::SecureStatusChanged {
                            id: device.id(),
                            secure_status: *device.secure_status(),
                        });
                    }
                    BroadcastType::Keyboard => error!("keyboard broadcasts are not implemented!"),
                    BroadcastType::User => error!("user broadcasts are not implemented!"),
                }
            }
        }

        Ok(events)
    }

    pub fn query<T>(&mut self, id: Uuid, request: T) -> Result<T::Response>
    where
        T: XapRequest,
    {
        match self.devices.get_mut(&id) {
            Some(device) => device.query(request),
            None => Err(anyhow!("unknown device id: {id}")),
        }
    }

    pub fn xap_constants(&self) -> XapConstants {
        self.constants.as_ref().clone()
    }

    pub fn enumerate_xap_devices(&mut self) -> Result<Vec<XapEvent>> {
        // TODO: implement as callback functions?
        let mut events = Vec::new();
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
            if xap_devices
                .iter()
                .any(|candidate| known_device.is_hid_device(candidate))
            {
                true
            } else {
                events.push(XapEvent::RemovedDevice { id: *id });
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
                device.open_device(&self.hid)?,
            )?;
            let id = new_device.id();
            self.devices.insert(id, new_device);
            events.push(XapEvent::NewDevice { id });
        }

        Ok(events)
    }

    pub fn get_device(&self, id: &Uuid) -> Result<&XapDevice> {
        self.devices
            .get(id)
            .ok_or(anyhow!("unknown device id: {id}"))
    }

    pub fn get_device_mut(&mut self, id: &Uuid) -> Result<&mut XapDevice> {
        self.devices
            .get_mut(id)
            .ok_or(anyhow!("unknown device id: {id}"))
    }

    pub fn get_devices(&self) -> Vec<&XapDevice> {
        self.devices.values().collect()
    }
}
