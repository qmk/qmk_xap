use anyhow::anyhow;
use crossbeam_channel::Sender;
use hidapi::{DeviceInfo, HidApi, HidDeviceInfo};
use log::error;

use crate::{
    xap::{XAPDevice, XAPError, XAPResult},
    XAPEvent,
};

const XAP_USAGE_PAGE: u16 = 0xFF51;
const XAP_USAGE: u16 = 0x0058;

pub struct XAPClient {
    pub(crate) hid: HidApi,
}

impl XAPClient {
    pub fn new() -> XAPResult<Self> {
        Ok(XAPClient {
            hid: HidApi::new()?,
        })
    }

    pub(crate) fn xap_devices(&mut self) -> XAPResult<Vec<DeviceInfo>> {
        self.hid.refresh_devices()?;

        Ok(self
            .hid
            .device_list()
            .filter(|info| info.usage_page() == XAP_USAGE_PAGE && info.usage() == XAP_USAGE)
            .cloned()
            .collect())
    }

    pub fn is_device_connected(&mut self, device: &XAPDevice) -> bool {
        if let Err(err) = self.hid.refresh_devices() {
            error!("error refreshing HID devices {}", err);
            return false;
        }

        let device = device.info();

        self.hid.device_list().any(|candidate| {
            candidate.path() == device.path()
                && candidate.product_id() == device.product_id()
                && candidate.vendor_id() == device.vendor_id()
                && candidate.usage_page() == device.usage_page()
                && candidate.usage() == device.usage()
        })
    }
}
