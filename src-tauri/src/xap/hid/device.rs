use std::{
    fmt::{Debug, Display},
    io::Cursor,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use binrw::BinWriterExt;
use crossbeam_channel::{unbounded, Receiver, Sender};
use hidapi::{DeviceInfo, HidDevice};
use log::{error, trace};
use uuid::Uuid;

use crate::{xap::*, XAPEvent};

const XAP_REPORT_SIZE: usize = 64;

pub struct XAPDevice {
    info: DeviceInfo,
    id: Uuid,
    tx_device: HidDevice,
    _rx_thread: JoinHandle<()>,
    rx_channel: Receiver<ResponseRaw>,
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
    pub fn do_query<T: XAPRequest>(&self, request: T) -> XAPResult<T::Response> {
        let request = RequestRaw::new(request);
        let mut report = [0; XAP_REPORT_SIZE];

        let mut writer = Cursor::new(&mut report[1..]);
        writer.write_le(&request)?;

        trace!("send XAP report with token {:?}", request.token());
        self.tx_device.write(&report)?;

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

    pub(crate) fn new(
        info: DeviceInfo,
        event_channel: Sender<XAPEvent>,
        rx: HidDevice,
        tx: HidDevice,
    ) -> Self {
        let (tx_channel, rx_channel) = unbounded();
        let id = Uuid::new_v4();
        Self {
            info,
            id: id,
            tx_device: tx,
            _rx_thread: Self::start_rx_thread(id, rx, event_channel, tx_channel),
            rx_channel,
        }
    }

    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    fn start_rx_thread(
        device_id: Uuid,
        rx: HidDevice,
        event_channel: Sender<XAPEvent>,
        tx_channel: Sender<ResponseRaw>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let result: XAPResult<()> = (|| {
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
                                event_channel
                                    .send(XAPEvent::Broadcast {
                                        id: device_id,
                                        response: response,
                                    })
                                    .expect("failed to send broadcast event!");
                            } else {
                                trace!(
                                    "
                                received XAP package with token {:?} and payload {:#?}",
                                    response.token(),
                                    response.payload()
                                );
                                tx_channel
                                    .send(response)
                                    .expect("failed to forward received XAP report");
                            }
                        }
                        Err(err) => error!("received malformed XAP HID report {err}"),
                    }
                }
            })();

            if let Err(err) = result {
                // Terminate thread and notify state
                event_channel
                    .send(XAPEvent::RxError {
                        id: device_id,
                        error: err,
                    })
                    .expect("failed to send error event!");
            }
        })
    }
}
