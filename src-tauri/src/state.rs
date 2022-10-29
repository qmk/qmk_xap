use std::sync::Arc;

use anyhow::anyhow;
use tokio::sync::Mutex;

use crate::xap::{XAPClient, XAPDevice, XAPError, XAPRequest, XAPResult};

pub(crate) struct AppState {
    pub(crate) device: Arc<Mutex<Option<XAPDevice>>>,
    pub(crate) client: Arc<Mutex<XAPClient>>,
}

impl AppState {
    pub async fn do_query<T>(&self, request: T) -> XAPResult<T::Response>
    where
        T: XAPRequest,
    {
        let result = match &*self.device.lock().await {
            Some(device) => device.do_query(request),
            None => Err(XAPError::Other(anyhow!("Device not available"))),
        };

        if result.is_err() {
            self.process_error().await;
        }

        result
    }

    pub async fn do_action<T, F>(&self, action: F) -> XAPResult<T>
    where
        F: FnOnce(&XAPDevice) -> XAPResult<T>,
    {
        let result = match &*self.device.lock().await {
            Some(device) => action(device),
            None => Err(XAPError::Other(anyhow!("Device not available"))),
        };

        if result.is_err() {
            self.process_error().await;
        }

        result
    }

    // TODO MOVE THIS INTO SINGLETON STATE HANDLING INSTANCE
    async fn process_error(&self) {
        let mut client = self.client.lock().await;
        let mut device = self.device.lock().await;

        if let Some(inner_device) = &(*device) {
            if !client.is_device_connected(inner_device) {
                *device = None;
            }
        }
    }
}
