use std::fmt::Debug;
use std::fmt::Display;

use anyhow::{bail, Result};
use hidapi::{DeviceInfo, HidApi, HidDevice};

const XAP_USAGE_PAGE: u16 = 0xFF51;
const XAP_USAGE: u16 = 0x0058;

pub(crate) struct XAPClient {
    hid: HidApi,
}

pub(crate) struct XAPDevice {
    info: DeviceInfo,
    device: HidDevice,
}

impl Debug for XAPDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for XAPDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VID: {:04x}, PID: {:04x}, Serial: {}, Product name: {}, Manufacturer: {}",
            self.info.vendor_id(),
            self.info.product_id(),
            match self.info.serial_number() {
                Some(s) => s,
                _ => "<COULD NOT FETCH>",
            },
            match self.info.product_string() {
                Some(s) => s,
                _ => "<COULD NOT FETCH>",
            },
            match self.info.manufacturer_string() {
                Some(s) => s,
                _ => "<COULD_NOT_FETCH>",
            }
        )
    }
}

impl XAPClient {
    pub fn new() -> Result<Self> {
        Ok(XAPClient {
            hid: HidApi::new()?,
        })
    }

    pub fn get_first_xap_device(&mut self) -> Result<XAPDevice> {
        self.hid.refresh_devices()?;

        match self
            .hid
            .device_list()
            .find(|info| info.usage_page() == XAP_USAGE_PAGE && info.usage() == XAP_USAGE)
        {
            Some(info) => Ok(XAPDevice {
                info: info.clone(),
                device: info.open_device(&self.hid)?,
            }),
            None => bail!("no XAP compatible device found!"),
        }
    }
}
