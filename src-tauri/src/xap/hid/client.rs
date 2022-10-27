use anyhow::{bail, Result};
use hidapi::HidApi;
use crate::xap::XAPDevice;

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
}
