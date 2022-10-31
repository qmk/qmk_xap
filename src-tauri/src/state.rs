use std::{collections::HashMap, fmt::Debug};

use anyhow::anyhow;
use crossbeam_channel::Sender;
use log::info;
use uuid::Uuid;

use crate::{
    xap::{XAPClient, XAPDevice, XAPError, XAPRequest, XAPResult},
    XAPEvent,
};

pub(crate) struct AppState {
    pub(crate) devices: HashMap<Uuid, XAPDevice>,
    pub(crate) client: XAPClient,
    pub(crate) event_channel: Sender<XAPEvent>,
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("device", &self.devices)
            .finish()
    }
}
impl AppState {
    pub fn do_action<T, F>(&self, id: Uuid, action: F) -> XAPResult<T>
    where
        F: FnOnce(&XAPDevice) -> XAPResult<T>,
    {
        // TODO actually implement multi-device handling
        match self.devices.iter().next() {
            Some(device) => action(device.1),
            None => Err(XAPError::Other(anyhow!("device not available"))),
        }
    }

    pub fn do_query<T>(&self, id: Uuid, request: T) -> XAPResult<T::Response>
    where
        T: XAPRequest,
    {
        // TODO actually implement multi-device handling
        match self.devices.iter().next() {
            Some(device) => device.1.do_query(request),
            None => Err(XAPError::Other(anyhow!("device not available"))),
        }
    }

    pub fn new(client: XAPClient, event_channel: Sender<XAPEvent>) -> Self {
        AppState {
            client,
            devices: HashMap::new(),
            event_channel,
        }
    }

    pub fn query_all_devices(&mut self) -> XAPResult<()> {
        if self.devices.is_empty() {
            info!("querying for compatible XAP devices");
            if let Ok(devices) = self.client.xap_devices() {
                for device in devices {
                    let device = XAPDevice::new(
                        device.clone(),
                        self.event_channel.clone(),
                        device.open_device(&self.client.hid)?,
                        device.open_device(&self.client.hid)?,
                    );
                    let id = device.id();
                    self.devices.insert(id, device);
                    self.event_channel
                        .send(XAPEvent::NewDevice(id))
                        .expect("failed to announce new device");
                }
            }
        }
        Ok(())
    }

    pub fn remove_disconnected_devices(&mut self) {
        self.devices
            .retain(|_, device| self.client.is_device_connected(&device));
    }
}
