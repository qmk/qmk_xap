use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crossbeam_channel::Sender;
use hidapi::{DeviceInfo, HidApi};
use uuid::Uuid;
use xap_specs::{constants::XAPConstants, request::XAPRequest};

use crate::{
    xap::{ClientError, ClientResult},
    XAPEvent,
};

use super::XAPDevice;

const XAP_USAGE_PAGE: u16 = 0xFF51;
const XAP_USAGE: u16 = 0x0058;

pub(crate) struct XAPClient {
    hid: HidApi,
    devices: HashMap<Uuid, XAPDevice>,
    event_channel: Sender<XAPEvent>,
    constants: Arc<XAPConstants>,
}

impl Debug for XAPClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("device", &self.devices)
            .finish()
    }
}

impl XAPClient {
    pub fn new(event_channel: Sender<XAPEvent>, xap_constants: XAPConstants) -> ClientResult<Self> {
        Ok(Self {
            devices: HashMap::new(),
            hid: HidApi::new_without_enumerate()?,
            event_channel,
            constants: Arc::new(xap_constants),
        })
    }

    #[allow(dead_code)]
    pub fn action<T, F>(&self, id: Uuid, action: F) -> ClientResult<T>
    where
        F: FnOnce(&XAPDevice) -> ClientResult<T>,
    {
        match self.devices.get(&id) {
            Some(device) => action(device),
            None => Err(ClientError::UnknownDevice(id)),
        }
    }

    pub fn query<T>(&self, id: Uuid, request: T) -> ClientResult<T::Response>
    where
        T: XAPRequest,
    {
        match self.devices.get(&id) {
            Some(device) => device.query(request),
            None => Err(ClientError::UnknownDevice(id)),
        }
    }

    pub fn xap_constants(&self) -> XAPConstants {
        self.constants.as_ref().clone()
    }

    pub fn enumerate_xap_devices(&mut self) -> ClientResult<()> {
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
                    .send(XAPEvent::RemovedDevice(*id))
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

            let new_device = XAPDevice::new(
                device.clone(),
                Arc::clone(&self.constants),
                self.event_channel.clone(),
                device.open_device(&self.hid)?,
                device.open_device(&self.hid)?,
            )?;
            let id = new_device.id();
            self.devices.insert(id, new_device);
            self.event_channel
                .send(XAPEvent::NewDevice(id))
                .expect("failed to announce new xap device");
        }

        Ok(())
    }

    pub fn get_device(&self, id: &Uuid) -> ClientResult<&XAPDevice> {
        self.devices.get(id).ok_or(ClientError::UnknownDevice(*id))
    }

    pub fn get_device_mut(&mut self, id: &Uuid) -> ClientResult<&mut XAPDevice> {
        self.devices
            .get_mut(id)
            .ok_or(ClientError::UnknownDevice(*id))
    }

    pub fn get_devices(&self) -> Vec<&XAPDevice> {
        self.devices.values().collect()
    }
}
