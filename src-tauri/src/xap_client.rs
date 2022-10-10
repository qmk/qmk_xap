use anyhow::Result;
use hidapi::HidApi;

pub(crate) struct XAPClient {
    hid: HidApi,
}

const XAP_USAGE_PAGE: u16 = 0xFF51;
const XAP_USAGE: u16 = 0xFF51;

impl XAPClient {
    pub fn new() -> Result<Self> {
        Ok(XAPClient {
            hid: HidApi::new()?,
        })
    }

    pub fn get_xap_devices(&self) -> Vec<String> {
        self.hid
            .device_list()
            .filter(|device| dbg!(device.usage_page()) == XAP_USAGE_PAGE && dbg!(device.usage()) == XAP_USAGE)
            .map(|device| {
                format!(
                    "VID: {:04x}, PID: {:04x}, Serial: {}, Product name: {}, Manufacturer: {}",
                    device.vendor_id(),
                    device.product_id(),
                    match device.serial_number() {
                        Some(s) => s,
                        _ => "<COULD NOT FETCH>",
                    },
                    match device.product_string() {
                        Some(s) => s,
                        _ => "<COULD NOT FETCH>",
                    },
                    match device.manufacturer_string() {
                        Some(s) => s,
                        _ => "<COULD_NOT_FETCH>",
                    }
                )
            })
            .collect()
    }
}
