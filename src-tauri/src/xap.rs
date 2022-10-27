use std::fmt::Debug;
use std::fmt::Display;
use std::io::Cursor;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use anyhow::{anyhow, bail};
use binrw::BinWriterExt;
use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use log::error;

use log::trace;

use crate::protocol::RGBLightConfig;
use crate::protocol::RGBLightConfigCommand;
use crate::protocol::RequestRaw;
use crate::protocol::ResponseRaw;
use crate::protocol::Token;
use crate::protocol::XAPError;
use crate::protocol::XAPRequest;
use crate::protocol::XAPResult;
use crate::protocol::XAPVersion;
use crate::protocol::XAPVersionQuery;

const XAP_USAGE_PAGE: u16 = 0xFF51;
const XAP_USAGE: u16 = 0x0058;
const XAP_REPORT_SIZE: usize = 64;

pub(crate) struct XAPClient {
    hid: HidApi,
}

pub(crate) struct XAPDevice {
    info: DeviceInfo,
    tx: HidDevice,
    _rx_thread: JoinHandle<()>,
    rx_channel: Receiver<ResponseRaw>,
    broadcast_channel: Receiver<ResponseRaw>,
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
    pub fn query_xap_version(&self) -> XAPResult<XAPVersion> {
        self.do_query(XAPVersionQuery {})
    }

    pub fn set_rgblight_config(&self) -> XAPResult<()> {
        let request = RGBLightConfigCommand {
            config: RGBLightConfig {
                enable: 1,
                mode: 1,
                hue: rand::random(),
                sat: 255,
                val: 255,
                speed: 50,
            },
        };
        self.do_query(request)
    }

    pub fn do_query<T: XAPRequest>(&self, request: T) -> XAPResult<T::Response> {
        let request = RequestRaw::new(request);
        let mut report = [0; XAP_REPORT_SIZE];

        let mut writer = Cursor::new(&mut report[1..]);
        writer.write_le(&request)?;

        trace!("send XAP report with token {:?}", request.token());
        self.tx.write(&report)?;

        let start = Instant::now();

        let response = loop {
            let response = self
                .rx_channel
                .recv_timeout(Duration::from_millis(500))
                .map_err(|err| anyhow!("failed to reveice response {}", err))?;

            if response.token() == request.token() {
                break response;
            }
            if start.elapsed() > Duration::from_secs(5) {
                return Err(XAPError::Protocol(format!(
                    "failed to receive XAP response for request {:?} in 5 seconds",
                    request.token()
                )));
            }
        };

        response.into_xap_response::<T>()
    }

    pub fn new(info: DeviceInfo, rx: HidDevice, tx: HidDevice) -> Self {
        let (tx_channel, rx_channel) = unbounded();
        let (broadcast_tx_channel, broadcast_rx_channel) = unbounded();

        Self {
            info,
            tx,
            _rx_thread: Self::start_rx_thread(rx, broadcast_tx_channel, tx_channel),
            rx_channel,
            broadcast_channel: broadcast_rx_channel,
        }
    }

    fn start_rx_thread(
        rx: HidDevice,
        broadcast_tx_channel: Sender<ResponseRaw>,
        tx_channel: Sender<ResponseRaw>,
    ) -> JoinHandle<()> {
        // TODO: not happy with the heavy nesting, this should be cleaned-up.
        // Also nobody consumes the broadcast messages ATM.
        thread::spawn(move || loop {
            let result: anyhow::Result<()> = (|| {
                let mut report = [0_u8; XAP_REPORT_SIZE];
                loop {
                    rx.read(&mut report)?;

                    match ResponseRaw::from_raw_report(&report) {
                        Ok(response) => {
                            if *response.token() == Token::Broadcast {
                                trace!(
                                    "received XAP broadcast package with payload {:#?}",
                                    response.payload()
                                );
                                broadcast_tx_channel.send(response)?;
                            } else {
                                trace!(
                                    "
                                received XAP package with token {:?} and payload {:#?}",
                                    response.token(),
                                    response.payload()
                                );
                                tx_channel.send(response)?;
                            }
                        }
                        Err(err) => error!("received malformed XAP HID report {}", err),
                    }
                }
            })();

            if let Err(err) = result {
                error!("error in XAP receive thread {}", err);
            }
        })
    }
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
