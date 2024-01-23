
#[allow(dead_code)]
#[allow(unused_imports)]
pub mod xap_routes {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    /// Version Query
    ///
    /// XAP protocol version query.
    ///
    /// * Returns the BCD-encoded version in the format of XX.YY.ZZZZ => `0xXXYYZZZZ`
    ///     * e.g. 3.2.115 will match `0x03020115`, or bytes {0x15,0x01,0x02,0x03}.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapVersionQueryRequest(pub ());

    impl XAPRequest for XapVersionQueryRequest {
        type Response = XapVersionQueryResponse;

        fn id() -> &'static [u8] {
            &[00, 00]
        }

        fn xap_version() -> u32 {
            0x00000001
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct XapVersionQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn xap_version_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<XapVersionQueryResponse> {
        state.lock().query(id, XapVersionQueryRequest(()))
    }

    /// ======================================================================
    /// Capabilities Query
    ///
    /// XAP subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapCapabilitiesQueryRequest(pub ());

    impl XAPRequest for XapCapabilitiesQueryRequest {
        type Response = XapCapabilitiesQueryResponse;

        fn id() -> &'static [u8] {
            &[00, 01]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct XapCapabilitiesQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn xap_capabilities_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<XapCapabilitiesQueryResponse> {
        state.lock().query(id, XapCapabilitiesQueryRequest(()))
    }

    /// ======================================================================
    /// Enabled subsystem query
    ///
    /// XAP protocol subsystem query. Each bit should be considered as a "usable" subsystem. For example, checking `(value & (1 << XAP_ROUTE_QMK) != 0)` means the QMK subsystem is enabled and available for querying.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapEnabledSubsystemQueryRequest(pub ());

    impl XAPRequest for XapEnabledSubsystemQueryRequest {
        type Response = XapEnabledSubsystemQueryResponse;

        fn id() -> &'static [u8] {
            &[00, 02]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct XapEnabledSubsystemQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn xap_enabled_subsystem_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<XapEnabledSubsystemQueryResponse> {
        state.lock().query(id, XapEnabledSubsystemQueryRequest(()))
    }

    /// ======================================================================
    /// Secure Status
    ///
    /// Query secure route status
    ///
    /// * 0 means secure routes are disabled
    /// * 1 means unlock sequence initiated but incomplete
    /// * 2 means secure routes are allowed
    /// * any other value should be interpreted as disabled
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapSecureStatusRequest(pub ());

    impl XAPRequest for XapSecureStatusRequest {
        type Response = XapSecureStatusResponse;

        fn id() -> &'static [u8] {
            &[00, 03]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct XapSecureStatusResponse(pub u8);

    #[tauri::command]
    #[specta::specta]
    pub fn xap_secure_status(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<XapSecureStatusResponse> {
        state.lock().query(id, XapSecureStatusRequest(()))
    }

    /// ======================================================================
    /// Secure Unlock
    ///
    /// Initiate secure route unlock sequence
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapSecureUnlockRequest(pub ());

    impl XAPRequest for XapSecureUnlockRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[00, 04]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn xap_secure_unlock(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<()> {
        state.lock().query(id, XapSecureUnlockRequest(()))
    }

    /// ======================================================================
    /// Secure Lock
    ///
    /// Disable secure routes
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapSecureLockRequest(pub ());

    impl XAPRequest for XapSecureLockRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[00, 05]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn xap_secure_lock(id: Uuid, state: State<'_, Arc<Mutex<XAPClient>>>) -> ClientResult<()> {
        state.lock().query(id, XapSecureLockRequest(()))
    }
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod qmk_routes {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    /// Version Query
    ///
    /// QMK protocol version query.
    ///
    /// * Returns the BCD-encoded version in the format of XX.YY.ZZZZ => `0xXXYYZZZZ`
    ///     * e.g. 3.2.115 will match `0x03020115`, or bytes {0x15,0x01,0x02,0x03}.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkVersionQueryRequest(pub ());

    impl XAPRequest for QmkVersionQueryRequest {
        type Response = QmkVersionQueryResponse;

        fn id() -> &'static [u8] {
            &[01, 00]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkVersionQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_version_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkVersionQueryResponse> {
        state.lock().query(id, QmkVersionQueryRequest(()))
    }

    /// ======================================================================
    /// Capabilities Query
    ///
    /// QMK subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkCapabilitiesQueryRequest(pub ());

    impl XAPRequest for QmkCapabilitiesQueryRequest {
        type Response = QmkCapabilitiesQueryResponse;

        fn id() -> &'static [u8] {
            &[01, 01]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkCapabilitiesQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_capabilities_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkCapabilitiesQueryResponse> {
        state.lock().query(id, QmkCapabilitiesQueryRequest(()))
    }

    /// ======================================================================
    /// Board identifiers
    ///
    /// Retrieves the set of identifying information for the board.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkBoardIdentifiersRequest(pub ());

    impl XAPRequest for QmkBoardIdentifiersRequest {
        type Response = QmkBoardIdentifiersResponse;

        fn id() -> &'static [u8] {
            &[01, 02]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkBoardIdentifiersResponse {
        pub vendor_id: u16,
        pub product_id: u16,
        pub product_version: u16,
        pub qmk_unique_identifier: u32,
    }

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_board_identifiers(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkBoardIdentifiersResponse> {
        state.lock().query(id, QmkBoardIdentifiersRequest(()))
    }

    /// ======================================================================
    /// Board Manufacturer
    ///
    /// Retrieves the name of the manufacturer
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkBoardManufacturerRequest(pub ());

    impl XAPRequest for QmkBoardManufacturerRequest {
        type Response = QmkBoardManufacturerResponse;

        fn id() -> &'static [u8] {
            &[01, 03]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkBoardManufacturerResponse(pub UTF8String);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_board_manufacturer(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkBoardManufacturerResponse> {
        state.lock().query(id, QmkBoardManufacturerRequest(()))
    }

    /// ======================================================================
    /// Product Name
    ///
    /// Retrieves the product name
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkProductNameRequest(pub ());

    impl XAPRequest for QmkProductNameRequest {
        type Response = QmkProductNameResponse;

        fn id() -> &'static [u8] {
            &[01, 04]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkProductNameResponse(pub UTF8String);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_product_name(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkProductNameResponse> {
        state.lock().query(id, QmkProductNameRequest(()))
    }

    /// ======================================================================
    /// Config Blob Length
    ///
    /// Retrieves the length of the configuration data bundled within the firmware
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkConfigBlobLengthRequest(pub ());

    impl XAPRequest for QmkConfigBlobLengthRequest {
        type Response = QmkConfigBlobLengthResponse;

        fn id() -> &'static [u8] {
            &[01, 05]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkConfigBlobLengthResponse(pub u16);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_config_blob_length(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkConfigBlobLengthResponse> {
        state.lock().query(id, QmkConfigBlobLengthRequest(()))
    }

    /// ======================================================================
    /// Config Blob Chunk
    ///
    /// Retrieves a chunk of the configuration data bundled within the firmware
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkConfigBlobChunkRequest(pub u16);

    impl XAPRequest for QmkConfigBlobChunkRequest {
        type Response = QmkConfigBlobChunkResponse;

        fn id() -> &'static [u8] {
            &[01, 06]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkConfigBlobChunkResponse(pub [u8; 32]);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_config_blob_chunk(
        id: Uuid,
        arg: u16,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkConfigBlobChunkResponse> {
        state.lock().query(id, QmkConfigBlobChunkRequest(arg))
    }

    /// ======================================================================
    /// Jump to bootloader
    ///
    /// Jump to bootloader
    ///
    /// May not be present - if QMK capabilities query returns “true”, then jump to bootloader is supported
    ///
    /// * 0 means secure routes are disabled, and should be considered as a failure
    /// * 1 means successful, board will jump to bootloader
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkJumpToBootloaderRequest(pub ());

    impl XAPRequest for QmkJumpToBootloaderRequest {
        type Response = QmkJumpToBootloaderResponse;

        fn id() -> &'static [u8] {
            &[01, 07]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkJumpToBootloaderResponse(pub u8);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_jump_to_bootloader(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkJumpToBootloaderResponse> {
        state.lock().query(id, QmkJumpToBootloaderRequest(()))
    }

    /// ======================================================================
    /// Hardware Identifier
    ///
    /// Retrieves a unique identifier for the board.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkHardwareIdentifierRequest(pub ());

    impl XAPRequest for QmkHardwareIdentifierRequest {
        type Response = QmkHardwareIdentifierResponse;

        fn id() -> &'static [u8] {
            &[01, 08]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkHardwareIdentifierResponse(pub [u32; 4]);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_hardware_identifier(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkHardwareIdentifierResponse> {
        state.lock().query(id, QmkHardwareIdentifierRequest(()))
    }

    /// ======================================================================
    /// Reinitialize EEPROM
    ///
    /// Reinitializes the keyboard's EEPROM (persistent memory)
    ///
    /// May not be present - if QMK capabilities query returns “true”, then reinitialize is supported
    ///
    /// * 0 means secure routes are disabled, and should be considered as a failure
    /// * 1 means successful, board will reinitialize and then reboot
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkReinitializeEepromRequest(pub ());

    impl XAPRequest for QmkReinitializeEepromRequest {
        type Response = QmkReinitializeEepromResponse;

        fn id() -> &'static [u8] {
            &[01, 09]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkReinitializeEepromResponse(pub u8);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_reinitialize_eeprom(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkReinitializeEepromResponse> {
        state.lock().query(id, QmkReinitializeEepromRequest(()))
    }
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod keyboard_routes {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod user_routes {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod keymap_routes {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    /// Capabilities Query
    ///
    /// Keymap subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapCapabilitiesQueryRequest(pub ());

    impl XAPRequest for KeymapCapabilitiesQueryRequest {
        type Response = KeymapCapabilitiesQueryResponse;

        fn id() -> &'static [u8] {
            &[04, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapCapabilitiesQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn keymap_capabilities_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<KeymapCapabilitiesQueryResponse> {
        state.lock().query(id, KeymapCapabilitiesQueryRequest(()))
    }

    /// ======================================================================
    /// Get Layer Count
    ///
    /// Query maximum number of layers that can be addressed within the keymap.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetLayerCountRequest(pub ());

    impl XAPRequest for KeymapGetLayerCountRequest {
        type Response = KeymapGetLayerCountResponse;

        fn id() -> &'static [u8] {
            &[04, 02]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetLayerCountResponse(pub u8);

    #[tauri::command]
    #[specta::specta]
    pub fn keymap_get_layer_count(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<KeymapGetLayerCountResponse> {
        state.lock().query(id, KeymapGetLayerCountRequest(()))
    }

    /// ======================================================================
    /// Get Keycode
    ///
    /// Query the Keycode at the requested location.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetKeycodeRequest(pub KeymapGetKeycodeRequestArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetKeycodeRequestArg {
        pub layer: u8,
        pub row: u8,
        pub column: u8,
    }

    impl XAPRequest for KeymapGetKeycodeRequest {
        type Response = KeymapGetKeycodeResponse;

        fn id() -> &'static [u8] {
            &[04, 03]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetKeycodeResponse(pub u16);

    #[tauri::command]
    #[specta::specta]
    pub fn keymap_get_keycode(
        id: Uuid,
        arg: KeymapGetKeycodeRequestArg,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<KeymapGetKeycodeResponse> {
        state.lock().query(id, KeymapGetKeycodeRequest(arg))
    }

    /// ======================================================================
    /// Get Encoder Keycode
    ///
    /// Query the Keycode at the requested location.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetEncoderKeycodeRequest(pub KeymapGetEncoderKeycodeRequestArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetEncoderKeycodeRequestArg {
        pub layer: u8,
        pub encoder: u8,
        pub clockwise: u8,
    }

    impl XAPRequest for KeymapGetEncoderKeycodeRequest {
        type Response = KeymapGetEncoderKeycodeResponse;

        fn id() -> &'static [u8] {
            &[04, 04]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetEncoderKeycodeResponse(pub u16);

    #[tauri::command]
    #[specta::specta]
    pub fn keymap_get_encoder_keycode(
        id: Uuid,
        arg: KeymapGetEncoderKeycodeRequestArg,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<KeymapGetEncoderKeycodeResponse> {
        state.lock().query(id, KeymapGetEncoderKeycodeRequest(arg))
    }
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod remapping_routes {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    /// Capabilities Query
    ///
    /// Remapping subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingCapabilitiesQueryRequest(pub ());

    impl XAPRequest for RemappingCapabilitiesQueryRequest {
        type Response = RemappingCapabilitiesQueryResponse;

        fn id() -> &'static [u8] {
            &[05, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingCapabilitiesQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn remapping_capabilities_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<RemappingCapabilitiesQueryResponse> {
        state
            .lock()
            .query(id, RemappingCapabilitiesQueryRequest(()))
    }

    /// ======================================================================
    /// Get Layer Count
    ///
    /// Query maximum number of layers that can be addressed within the keymap.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingGetLayerCountRequest(pub ());

    impl XAPRequest for RemappingGetLayerCountRequest {
        type Response = RemappingGetLayerCountResponse;

        fn id() -> &'static [u8] {
            &[05, 02]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingGetLayerCountResponse(pub u8);

    #[tauri::command]
    #[specta::specta]
    pub fn remapping_get_layer_count(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<RemappingGetLayerCountResponse> {
        state.lock().query(id, RemappingGetLayerCountRequest(()))
    }

    /// ======================================================================
    /// Set Keycode
    ///
    /// Modify the Keycode at the requested location.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingSetKeycodeRequest(pub RemappingSetKeycodeRequestArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingSetKeycodeRequestArg {
        pub layer: u8,
        pub row: u8,
        pub column: u8,
        pub keycode: u16,
    }

    impl XAPRequest for RemappingSetKeycodeRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[05, 03]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn remapping_set_keycode(
        id: Uuid,
        arg: RemappingSetKeycodeRequestArg,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<()> {
        state.lock().query(id, RemappingSetKeycodeRequest(arg))
    }

    /// ======================================================================
    /// Set Encoder Keycode
    ///
    /// Modify the Keycode at the requested location.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingSetEncoderKeycodeRequest(pub RemappingSetEncoderKeycodeRequestArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingSetEncoderKeycodeRequestArg {
        pub layer: u8,
        pub encoder: u8,
        pub clockwise: u8,
        pub keycode: u16,
    }

    impl XAPRequest for RemappingSetEncoderKeycodeRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[05, 04]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn remapping_set_encoder_keycode(
        id: Uuid,
        arg: RemappingSetEncoderKeycodeRequestArg,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<()> {
        state
            .lock()
            .query(id, RemappingSetEncoderKeycodeRequest(arg))
    }
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod lighting_routes {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    /// Capabilities Query
    ///
    /// Lighting subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct LightingCapabilitiesQueryRequest(pub ());

    impl XAPRequest for LightingCapabilitiesQueryRequest {
        type Response = LightingCapabilitiesQueryResponse;

        fn id() -> &'static [u8] {
            &[06, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct LightingCapabilitiesQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn lighting_capabilities_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<LightingCapabilitiesQueryResponse> {
        state.lock().query(id, LightingCapabilitiesQueryRequest(()))
    }

    #[allow(dead_code)]
    #[allow(unused_imports)]
    pub mod backlight_routes {
        use std::sync::Arc;

        use binrw::{BinRead, BinWrite};
        use parking_lot::Mutex;
        use serde::{Deserialize, Serialize};
        use specta::Type;
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XAPClient;
        use crate::xap::ClientResult;
        use xap_specs::request::XAPRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        /// Capabilities Query
        ///
        /// backlight subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightCapabilitiesQueryRequest(pub ());

        impl XAPRequest for BacklightCapabilitiesQueryRequest {
            type Response = BacklightCapabilitiesQueryResponse;

            fn id() -> &'static [u8] {
                &[06, 02, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightCapabilitiesQueryResponse(pub u32);

        #[tauri::command]
        #[specta::specta]
        pub fn backlight_capabilities_query(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<BacklightCapabilitiesQueryResponse> {
            state
                .lock()
                .query(id, BacklightCapabilitiesQueryRequest(()))
        }

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightGetEnabledEffectsRequest(pub ());

        impl XAPRequest for BacklightGetEnabledEffectsRequest {
            type Response = BacklightGetEnabledEffectsResponse;

            fn id() -> &'static [u8] {
                &[06, 02, 02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightGetEnabledEffectsResponse(pub u8);

        #[tauri::command]
        #[specta::specta]
        pub fn backlight_get_enabled_effects(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<BacklightGetEnabledEffectsResponse> {
            state
                .lock()
                .query(id, BacklightGetEnabledEffectsRequest(()))
        }

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightGetConfigRequest(pub ());

        impl XAPRequest for BacklightGetConfigRequest {
            type Response = BacklightGetConfigResponse;

            fn id() -> &'static [u8] {
                &[06, 02, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightGetConfigResponse {
            pub enable: u8,
            pub mode: u8,
            pub val: u8,
        }

        #[tauri::command]
        #[specta::specta]
        pub fn backlight_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<BacklightGetConfigResponse> {
            state.lock().query(id, BacklightGetConfigRequest(()))
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightSetConfigRequest(pub BacklightSetConfigRequestArg);

        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightSetConfigRequestArg {
            pub enable: u8,
            pub mode: u8,
            pub val: u8,
        }

        impl XAPRequest for BacklightSetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 02, 04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn backlight_set_config(
            id: Uuid,
            arg: BacklightSetConfigRequestArg,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<()> {
            state.lock().query(id, BacklightSetConfigRequest(arg))
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightSaveConfigRequest(pub ());

        impl XAPRequest for BacklightSaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 02, 05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn backlight_save_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<()> {
            state.lock().query(id, BacklightSaveConfigRequest(()))
        }
    }

    #[allow(dead_code)]
    #[allow(unused_imports)]
    pub mod rgblight_routes {
        use std::sync::Arc;

        use binrw::{BinRead, BinWrite};
        use parking_lot::Mutex;
        use serde::{Deserialize, Serialize};
        use specta::Type;
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XAPClient;
        use crate::xap::ClientResult;
        use xap_specs::request::XAPRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        /// Capabilities Query
        ///
        /// rgblight subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightCapabilitiesQueryRequest(pub ());

        impl XAPRequest for RgblightCapabilitiesQueryRequest {
            type Response = RgblightCapabilitiesQueryResponse;

            fn id() -> &'static [u8] {
                &[06, 03, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightCapabilitiesQueryResponse(pub u32);

        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_capabilities_query(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgblightCapabilitiesQueryResponse> {
            state.lock().query(id, RgblightCapabilitiesQueryRequest(()))
        }

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightGetEnabledEffectsRequest(pub ());

        impl XAPRequest for RgblightGetEnabledEffectsRequest {
            type Response = RgblightGetEnabledEffectsResponse;

            fn id() -> &'static [u8] {
                &[06, 03, 02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightGetEnabledEffectsResponse(pub u64);

        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_get_enabled_effects(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgblightGetEnabledEffectsResponse> {
            state.lock().query(id, RgblightGetEnabledEffectsRequest(()))
        }

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightGetConfigRequest(pub ());

        impl XAPRequest for RgblightGetConfigRequest {
            type Response = RgblightGetConfigResponse;

            fn id() -> &'static [u8] {
                &[06, 03, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightGetConfigResponse {
            pub enable: u8,
            pub mode: u8,
            pub hue: u8,
            pub sat: u8,
            pub val: u8,
            pub speed: u8,
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgblightGetConfigResponse> {
            state.lock().query(id, RgblightGetConfigRequest(()))
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightSetConfigRequest(pub RgblightSetConfigRequestArg);

        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightSetConfigRequestArg {
            pub enable: u8,
            pub mode: u8,
            pub hue: u8,
            pub sat: u8,
            pub val: u8,
            pub speed: u8,
        }

        impl XAPRequest for RgblightSetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 03, 04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_set_config(
            id: Uuid,
            arg: RgblightSetConfigRequestArg,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<()> {
            state.lock().query(id, RgblightSetConfigRequest(arg))
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightSaveConfigRequest(pub ());

        impl XAPRequest for RgblightSaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 03, 05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_save_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<()> {
            state.lock().query(id, RgblightSaveConfigRequest(()))
        }
    }

    #[allow(dead_code)]
    #[allow(unused_imports)]
    pub mod rgbmatrix_routes {
        use std::sync::Arc;

        use binrw::{BinRead, BinWrite};
        use parking_lot::Mutex;
        use serde::{Deserialize, Serialize};
        use specta::Type;
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XAPClient;
        use crate::xap::ClientResult;
        use xap_specs::request::XAPRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        /// Capabilities Query
        ///
        /// rgb matrix subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixCapabilitiesQueryRequest(pub ());

        impl XAPRequest for RgbmatrixCapabilitiesQueryRequest {
            type Response = RgbmatrixCapabilitiesQueryResponse;

            fn id() -> &'static [u8] {
                &[06, 04, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixCapabilitiesQueryResponse(pub u32);

        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_capabilities_query(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgbmatrixCapabilitiesQueryResponse> {
            state
                .lock()
                .query(id, RgbmatrixCapabilitiesQueryRequest(()))
        }

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixGetEnabledEffectsRequest(pub ());

        impl XAPRequest for RgbmatrixGetEnabledEffectsRequest {
            type Response = RgbmatrixGetEnabledEffectsResponse;

            fn id() -> &'static [u8] {
                &[06, 04, 02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixGetEnabledEffectsResponse(pub u64);

        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_get_enabled_effects(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgbmatrixGetEnabledEffectsResponse> {
            state
                .lock()
                .query(id, RgbmatrixGetEnabledEffectsRequest(()))
        }

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixGetConfigRequest(pub ());

        impl XAPRequest for RgbmatrixGetConfigRequest {
            type Response = RgbmatrixGetConfigResponse;

            fn id() -> &'static [u8] {
                &[06, 04, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixGetConfigResponse {
            pub enable: u8,
            pub mode: u8,
            pub hue: u8,
            pub sat: u8,
            pub val: u8,
            pub speed: u8,
            pub flags: u8,
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgbmatrixGetConfigResponse> {
            state.lock().query(id, RgbmatrixGetConfigRequest(()))
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixSetConfigRequest(pub RgbmatrixSetConfigRequestArg);

        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixSetConfigRequestArg {
            pub enable: u8,
            pub mode: u8,
            pub hue: u8,
            pub sat: u8,
            pub val: u8,
            pub speed: u8,
            pub flags: u8,
        }

        impl XAPRequest for RgbmatrixSetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 04, 04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_set_config(
            id: Uuid,
            arg: RgbmatrixSetConfigRequestArg,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<()> {
            state.lock().query(id, RgbmatrixSetConfigRequest(arg))
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixSaveConfigRequest(pub ());

        impl XAPRequest for RgbmatrixSaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 04, 05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_save_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<()> {
            state.lock().query(id, RgbmatrixSaveConfigRequest(()))
        }
    }
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod audio_routes {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    /// Capabilities Query
    ///
    /// Audio subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioCapabilitiesQueryRequest(pub ());

    impl XAPRequest for AudioCapabilitiesQueryRequest {
        type Response = AudioCapabilitiesQueryResponse;

        fn id() -> &'static [u8] {
            &[07, 01]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioCapabilitiesQueryResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn audio_capabilities_query(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<AudioCapabilitiesQueryResponse> {
        state.lock().query(id, AudioCapabilitiesQueryRequest(()))
    }

    /// ======================================================================
    /// Get Config
    ///
    /// Query the current config.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioGetConfigRequest(pub ());

    impl XAPRequest for AudioGetConfigRequest {
        type Response = AudioGetConfigResponse;

        fn id() -> &'static [u8] {
            &[07, 03]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioGetConfigResponse {
        pub enable: u8,
        pub clicky_enable: u8,
    }

    #[tauri::command]
    #[specta::specta]
    pub fn audio_get_config(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<AudioGetConfigResponse> {
        state.lock().query(id, AudioGetConfigRequest(()))
    }

    /// ======================================================================
    /// Set Config
    ///
    /// Set the current config.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioSetConfigRequest(pub AudioSetConfigRequestArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioSetConfigRequestArg {
        pub enable: u8,
        pub clicky_enable: u8,
    }

    impl XAPRequest for AudioSetConfigRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[07, 04]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn audio_set_config(
        id: Uuid,
        arg: AudioSetConfigRequestArg,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<()> {
        state.lock().query(id, AudioSetConfigRequest(arg))
    }

    /// ======================================================================
    /// Save Config
    ///
    /// Save the current config.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioSaveConfigRequest(pub ());

    impl XAPRequest for AudioSaveConfigRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[07, 05]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn audio_save_config(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<()> {
        state.lock().query(id, AudioSaveConfigRequest(()))
    }
}
