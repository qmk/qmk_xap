use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;
use xap_specs::protocol::qmk::{QMKJumpToBootloaderQuery, QMKReinitializeEepromQuery};

use crate::xap::{hid::XAPClient, ClientResult};

#[tauri::command]
pub(crate) async fn jump_to_bootloader(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, QMKJumpToBootloaderQuery {})?;
    Ok(())
}

#[tauri::command]
pub(crate) async fn reset_eeprom(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, QMKReinitializeEepromQuery {})?;
    Ok(())
}
