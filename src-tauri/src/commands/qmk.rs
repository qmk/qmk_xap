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
    let result = state.lock().query(id, QMKJumpToBootloaderQuery {})?;
    result.into()
}

#[tauri::command]
pub(crate) async fn reset_eeprom(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> XAPResult<()> {
    let result = state.lock().query(id, QMKReinitializeEepromQuery {})?;
    result.into()
}
