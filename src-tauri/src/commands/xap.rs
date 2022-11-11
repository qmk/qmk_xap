use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::xap::{
    protocol::{XAPResult, XAPSecureStatus, XAPSecureStatusLock, XAPSecureStatusUnlock},
    XAPClient,
};

#[tauri::command]
pub(crate) async fn secure_status_get(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<XAPSecureStatus> {
    state.lock().get_device(&id)?.query_secure_status()
}

#[tauri::command]
pub(crate) async fn secure_lock(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, XAPSecureStatusLock {})
}

#[tauri::command]
pub(crate) async fn secure_unlock(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, XAPSecureStatusUnlock {})
}
