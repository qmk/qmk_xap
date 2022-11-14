use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;
use xap_specs::protocol::xap::{XAPSecureStatus, XAPSecureStatusLock, XAPSecureStatusUnlock};

use crate::xap::{hid::XAPClient, ClientResult};

#[tauri::command]
pub(crate) async fn secure_status_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<XAPSecureStatus> {
    state.lock().get_device(&id)?.query_secure_status()
}

#[tauri::command]
pub(crate) async fn secure_lock(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, XAPSecureStatusLock {})
}

#[tauri::command]
pub(crate) async fn secure_unlock(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, XAPSecureStatusUnlock {})
}
