use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::xap::{
    protocol::{QMKJumpToBootloaderQuery, QMKReinitializeEepromQuery, XAPResult},
    XAPClient,
};

#[tauri::command]
pub(crate) async fn jump_to_bootloader(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state.lock().query(id, QMKJumpToBootloaderQuery {})?.into()
}

#[tauri::command]
pub(crate) async fn reset_eeprom(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    state
        .lock()
        .query(id, QMKReinitializeEepromQuery {})?
        .into()
}
