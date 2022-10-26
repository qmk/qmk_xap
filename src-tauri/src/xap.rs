use std::fmt::Debug;
use std::fmt::Display;
use std::io::Cursor;

use anyhow::{bail, Result};
use binrw::BinWriterExt;
use hidapi::{DeviceInfo, HidApi, HidDevice};

use crate::protocol::RGBLightConfig;
use crate::protocol::RGBLightConfigCommand;
use crate::protocol::RequestRaw;
use crate::protocol::XAPRequest;
use crate::protocol::XAPSecureStatus;
use crate::protocol::XAPSecureStatusQuery;
use crate::protocol::XAPVersion;
use crate::protocol::XAPVersionQuery;

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

impl XAPDevice {
    pub fn query_secure_status(&self) -> Result<XAPSecureStatus> {
        self.do_query(RequestRaw::new(XAPSecureStatusQuery {}))
    }

    pub fn query_xap_version(&self) -> Result<XAPVersion> {
        self.do_query(RequestRaw::new(XAPVersionQuery {}))
    }

    pub fn set_rgblight_config(&self) -> Result<()> {
        let request = RequestRaw::new(RGBLightConfigCommand {
            config: RGBLightConfig {
                enable: 1,
                mode: 1,
                hue: rand::random(),
                sat: 255,
                val: 255,
                speed: 50,
            },
        });
        self.do_query(request)
    }

    fn do_query<T: XAPRequest>(&self, request: RequestRaw<T>) -> Result<T::Response> {
        let mut report: [u8; 64] = [0; 64];

        let mut writer = Cursor::new(&mut report[1..]);
        writer.write_le(&request)?;

        self.device.write(&report)?;

        // TODO handle multi packet and host responses aka. do Token matching
        // and packet re-assembly
        self.device.read_timeout(&mut report, 500)?;
        request.to_response(&report)
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
