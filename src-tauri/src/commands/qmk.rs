use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::xap::{hid::XAPClient, ClientResult};
use xap_specs::xap_generated::qmk_routes::{JumpToBootloaderRequest, ReinitializeEepromRequest};

#[tauri::command]
pub(crate) async fn jump_to_bootloader(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, JumpToBootloaderRequest(()))
}

#[tauri::command]
pub(crate) async fn reset_eeprom(
    id: Uuid,
    state: State<'_, Arc<Mutex<XAPClient>>>,
) -> ClientResult<()> {
    state.lock().query(id, ReinitializeEepromRequest(()))
}
