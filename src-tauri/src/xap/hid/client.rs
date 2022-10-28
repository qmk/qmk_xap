use anyhow::anyhow;
use hidapi::HidApi;
use log::error;

use crate::xap::{XAPDevice, XAPError, XAPResult};

const XAP_USAGE_PAGE: u16 = 0xFF51;
const XAP_USAGE: u16 = 0x0058;

pub struct XAPClient {
    hid: HidApi,
}

impl XAPClient {
    pub fn new() -> XAPResult<Self> {
        Ok(XAPClient {
            hid: HidApi::new()?,
        })
    }

    pub fn get_first_xap_device(&mut self) -> XAPResult<XAPDevice> {
        self.hid.refresh_devices()?;

        match self
            .hid
            .device_list()
            .find(|info| info.usage_page() == XAP_USAGE_PAGE && info.usage() == XAP_USAGE)
        {
            Some(info) => Ok(XAPDevice::new(
                info.clone(),
                info.open_device(&self.hid)?,
                info.open_device(&self.hid)?,
            )),
            None => return Err(XAPError::Other(anyhow!("no XAP compatible device found!"))),
        }
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
