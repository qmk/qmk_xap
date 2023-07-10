
#[allow(dead_code)]
#[allow(unused_imports)]
pub mod xap {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    /// Version
    ///
    /// XAP protocol version query.
    ///
    /// * Returns the BCD-encoded version in the format of XX.YY.ZZZZ => `0xXXYYZZZZ`
    ///     * e.g. 3.2.115 will match `0x03020115`, or bytes {0x15,0x01,0x02,0x03}.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapVersionRequest(pub ());

    impl XAPRequest for XapVersionRequest {
        type Response = XapVersionResponse;

        fn id() -> &'static [u8] {
            &[00, 00]
        }

        fn xap_version() -> u32 {
            0x00000001
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct XapVersionResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn xap_version(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<XapVersionResponse> {
        state.lock().query(id, XapVersionRequest(()))
    }

    /// ======================================================================
    ///  capabilities
    ///
    /// XAP subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapCapabilitiesRequest(pub ());

    impl XAPRequest for XapCapabilitiesRequest {
        type Response = XapCapabilities;

        fn id() -> &'static [u8] {
            &[00, 01]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct XapCapabilities(u32);

    bitflags! {
                    impl XapCapabilities: u32 {

    const Version = 1 << 0;
    const Capabilities = 1 << 1;
    const EnabledSubsystemCapabilities = 1 << 2;
    const SecureStatus = 1 << 3;
    const SecureUnlock = 1 << 4;
    const SecureLock = 1 << 5;
    }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn xap_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<XapCapabilities> {
        state.lock().query(id, XapCapabilitiesRequest(()))
    }

    /// ======================================================================
    /// Enabled subsystem capabilities
    ///
    /// XAP protocol subsystem query. Each bit should be considered as a "usable" subsystem. For example, checking `(value & (1 << XAP_ROUTE_QMK) != 0)` means the QMK subsystem is enabled and available for querying.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapEnabledSubsystemCapabilitiesRequest(pub ());

    impl XAPRequest for XapEnabledSubsystemCapabilitiesRequest {
        type Response = XapEnabledSubsystemCapabilities;

        fn id() -> &'static [u8] {
            &[00, 02]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct XapEnabledSubsystemCapabilities(u32);

    bitflags! {
                    impl XapEnabledSubsystemCapabilities: u32 {

    const Xap = 1 << 0;
    const Qmk = 1 << 1;
    const Keyboard = 1 << 2;
    const User = 1 << 3;
    const Keymap = 1 << 4;
    const Remapping = 1 << 5;
    const Lighting = 1 << 6;
    const Audio = 1 << 7;
    }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn xap_enabled_subsystem_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<XapEnabledSubsystemCapabilities> {
        state
            .lock()
            .query(id, XapEnabledSubsystemCapabilitiesRequest(()))
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
pub mod qmk {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    /// Version
    ///
    /// QMK protocol version query.
    ///
    /// * Returns the BCD-encoded version in the format of XX.YY.ZZZZ => `0xXXYYZZZZ`
    ///     * e.g. 3.2.115 will match `0x03020115`, or bytes {0x15,0x01,0x02,0x03}.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkVersionRequest(pub ());

    impl XAPRequest for QmkVersionRequest {
        type Response = QmkVersionResponse;

        fn id() -> &'static [u8] {
            &[01, 00]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkVersionResponse(pub u32);

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_version(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkVersionResponse> {
        state.lock().query(id, QmkVersionRequest(()))
    }

    /// ======================================================================
    ///  capabilities
    ///
    /// QMK subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkCapabilitiesRequest(pub ());

    impl XAPRequest for QmkCapabilitiesRequest {
        type Response = QmkCapabilities;

        fn id() -> &'static [u8] {
            &[01, 01]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct QmkCapabilities(u32);

    bitflags! {
                    impl QmkCapabilities: u32 {

    const Version = 1 << 0;
    const Capabilities = 1 << 1;
    const BoardIdentifiers = 1 << 2;
    const BoardManufacturer = 1 << 3;
    const ProductName = 1 << 4;
    const ConfigBlobLength = 1 << 5;
    const ConfigBlobChunk = 1 << 6;
    const JumpToBootloader = 1 << 7;
    const HardwareIdentifier = 1 << 8;
    const ReinitializeEeprom = 1 << 9;
    }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn qmk_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<QmkCapabilities> {
        state.lock().query(id, QmkCapabilitiesRequest(()))
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
pub mod keyboard {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod user {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod keymap {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    ///  capabilities
    ///
    /// Keymap subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapCapabilitiesRequest(pub ());

    impl XAPRequest for KeymapCapabilitiesRequest {
        type Response = KeymapCapabilities;

        fn id() -> &'static [u8] {
            &[04, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct KeymapCapabilities(u32);

    bitflags! {
                    impl KeymapCapabilities: u32 {

    const Capabilities = 1 << 1;
    const GetLayerCount = 1 << 2;
    const GetKeycode = 1 << 3;
    const GetEncoderKeycode = 1 << 4;
    }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn keymap_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<KeymapCapabilities> {
        state.lock().query(id, KeymapCapabilitiesRequest(()))
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
    pub struct KeymapGetKeycodeRequest(pub KeymapGetKeycodeArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct KeymapGetKeycodeArg {
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
        arg: KeymapGetKeycodeArg,
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
    pub struct KeymapGetEncoderKeycodeRequest(pub KeymapGetEncoderKeycodeArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct KeymapGetEncoderKeycodeArg {
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
        arg: KeymapGetEncoderKeycodeArg,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<KeymapGetEncoderKeycodeResponse> {
        state.lock().query(id, KeymapGetEncoderKeycodeRequest(arg))
    }
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod remapping {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    ///  capabilities
    ///
    /// Remapping subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingCapabilitiesRequest(pub ());

    impl XAPRequest for RemappingCapabilitiesRequest {
        type Response = RemappingCapabilities;

        fn id() -> &'static [u8] {
            &[05, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct RemappingCapabilities(u32);

    bitflags! {
                    impl RemappingCapabilities: u32 {

    const Capabilities = 1 << 1;
    const GetLayerCount = 1 << 2;
    const SetKeycode = 1 << 3;
    const SetEncoderKeycode = 1 << 4;
    }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn remapping_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<RemappingCapabilities> {
        state.lock().query(id, RemappingCapabilitiesRequest(()))
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
    pub struct RemappingSetKeycodeRequest(pub RemappingSetKeycodeArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct RemappingSetKeycodeArg {
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
        arg: RemappingSetKeycodeArg,
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
    pub struct RemappingSetEncoderKeycodeRequest(pub RemappingSetEncoderKeycodeArg);

    #[derive(BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct RemappingSetEncoderKeycodeArg {
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
        arg: RemappingSetEncoderKeycodeArg,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<()> {
        state
            .lock()
            .query(id, RemappingSetEncoderKeycodeRequest(arg))
    }
}

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod lighting {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    ///  capabilities
    ///
    /// Lighting subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct LightingCapabilitiesRequest(pub ());

    impl XAPRequest for LightingCapabilitiesRequest {
        type Response = LightingCapabilities;

        fn id() -> &'static [u8] {
            &[06, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct LightingCapabilities(u32);

    bitflags! {
                    impl LightingCapabilities: u32 {

    const Capabilities = 1 << 1;
    const Backlight = 1 << 2;
    const Rgblight = 1 << 3;
    const Rgbmatrix = 1 << 4;
    }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn lighting_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<LightingCapabilities> {
        state.lock().query(id, LightingCapabilitiesRequest(()))
    }

    #[allow(dead_code)]
    #[allow(unused_imports)]
    pub mod backlight {
        use std::sync::Arc;

        use binrw::{BinRead, BinWrite};
        use bitflags::bitflags;
        use parking_lot::Mutex;
        use serde::{Deserialize, Serialize};
        use specta::Type;
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XAPClient;
        use crate::xap::ClientResult;
        use crate::xap_spec::types::*;
        use xap_specs::request::XAPRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        ///  capabilities
        ///
        /// backlight subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightCapabilitiesRequest(pub ());

        impl XAPRequest for BacklightCapabilitiesRequest {
            type Response = BacklightCapabilities;

            fn id() -> &'static [u8] {
                &[06, 02, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(
            BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
        )]
        pub struct BacklightCapabilities(u32);

        bitflags! {
                        impl BacklightCapabilities: u32 {

        const Capabilities = 1 << 1;
        const GetEnabledEffects = 1 << 2;
        const GetConfig = 1 << 3;
        const SetConfig = 1 << 4;
        const SaveConfig = 1 << 5;
        }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn backlight_capabilities(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<BacklightCapabilities> {
            state.lock().query(id, BacklightCapabilitiesRequest(()))
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
            type Response = BacklightConfig;

            fn id() -> &'static [u8] {
                &[06, 02, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn backlight_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<BacklightConfig> {
            state.lock().query(id, BacklightGetConfigRequest(()))
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightSetConfigRequest(pub BacklightConfig);

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
            arg: BacklightConfig,
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
    pub mod rgblight {
        use std::sync::Arc;

        use binrw::{BinRead, BinWrite};
        use bitflags::bitflags;
        use parking_lot::Mutex;
        use serde::{Deserialize, Serialize};
        use specta::Type;
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XAPClient;
        use crate::xap::ClientResult;
        use crate::xap_spec::types::*;
        use xap_specs::request::XAPRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        ///  capabilities
        ///
        /// rgblight subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightCapabilitiesRequest(pub ());

        impl XAPRequest for RgblightCapabilitiesRequest {
            type Response = RgblightCapabilities;

            fn id() -> &'static [u8] {
                &[06, 03, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(
            BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
        )]
        pub struct RgblightCapabilities(u32);

        bitflags! {
                        impl RgblightCapabilities: u32 {

        const Capabilities = 1 << 1;
        const GetEnabledEffects = 1 << 2;
        const GetConfig = 1 << 3;
        const SetConfig = 1 << 4;
        const SaveConfig = 1 << 5;
        }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_capabilities(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgblightCapabilities> {
            state.lock().query(id, RgblightCapabilitiesRequest(()))
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
            type Response = RgbLightConfig;

            fn id() -> &'static [u8] {
                &[06, 03, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgbLightConfig> {
            state.lock().query(id, RgblightGetConfigRequest(()))
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightSetConfigRequest(pub RgbLightConfig);

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
            arg: RgbLightConfig,
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
    pub mod rgbmatrix {
        use std::sync::Arc;

        use binrw::{BinRead, BinWrite};
        use bitflags::bitflags;
        use parking_lot::Mutex;
        use serde::{Deserialize, Serialize};
        use specta::Type;
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XAPClient;
        use crate::xap::ClientResult;
        use crate::xap_spec::types::*;
        use xap_specs::request::XAPRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        ///  capabilities
        ///
        /// rgb matrix subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixCapabilitiesRequest(pub ());

        impl XAPRequest for RgbmatrixCapabilitiesRequest {
            type Response = RgbmatrixCapabilities;

            fn id() -> &'static [u8] {
                &[06, 04, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(
            BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
        )]
        pub struct RgbmatrixCapabilities(u32);

        bitflags! {
                        impl RgbmatrixCapabilities: u32 {

        const Capabilities = 1 << 1;
        const GetEnabledEffects = 1 << 2;
        const GetConfig = 1 << 3;
        const SetConfig = 1 << 4;
        const SaveConfig = 1 << 5;
        }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_capabilities(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgbmatrixCapabilities> {
            state.lock().query(id, RgbmatrixCapabilitiesRequest(()))
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
            type Response = RgbMatrixConfig;

            fn id() -> &'static [u8] {
                &[06, 04, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XAPClient>>>,
        ) -> ClientResult<RgbMatrixConfig> {
            state.lock().query(id, RgbmatrixGetConfigRequest(()))
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixSetConfigRequest(pub RgbMatrixConfig);

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
            arg: RgbMatrixConfig,
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
pub mod audio {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XAPClient;
    use crate::xap::ClientResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XAPRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    ///  capabilities
    ///
    /// Audio subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioCapabilitiesRequest(pub ());

    impl XAPRequest for AudioCapabilitiesRequest {
        type Response = AudioCapabilities;

        fn id() -> &'static [u8] {
            &[07, 01]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct AudioCapabilities(u32);

    bitflags! {
                    impl AudioCapabilities: u32 {

    const Capabilities = 1 << 1;
    const GetConfig = 1 << 3;
    const SetConfig = 1 << 4;
    const SaveConfig = 1 << 5;
    }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn audio_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<AudioCapabilities> {
        state.lock().query(id, AudioCapabilitiesRequest(()))
    }

    /// ======================================================================
    /// Get Config
    ///
    /// Query the current config.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioGetConfigRequest(pub ());

    impl XAPRequest for AudioGetConfigRequest {
        type Response = AudioConfig;

        fn id() -> &'static [u8] {
            &[07, 03]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[tauri::command]
    #[specta::specta]
    pub fn audio_get_config(
        id: Uuid,
        state: State<'_, Arc<Mutex<XAPClient>>>,
    ) -> ClientResult<AudioConfig> {
        state.lock().query(id, AudioGetConfigRequest(()))
    }

    /// ======================================================================
    /// Set Config
    ///
    /// Set the current config.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioSetConfigRequest(pub AudioConfig);

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
        arg: AudioConfig,
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

pub mod types {
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};
    use specta::Type;

    /// Packet format for broadcast messages.
    #[derive(BinRead, BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct BroadcastHeader {
        pub r#type: u8,
        pub length: u8,
    }

    /// Packet format for inbound data.
    #[derive(BinRead, BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct RequestHeader {
        pub length: u8,
    }

    /// Config for lighting subsystem
    #[derive(BinRead, BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct BacklightConfig {
        pub enable: u8,
        pub mode: u8,
        pub val: u8,
    }

    /// RGB config for RGB matrix subsystem
    #[derive(BinRead, BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct RgbMatrixConfig {
        pub enable: u8,
        pub mode: u8,
        pub hue: u8,
        pub sat: u8,
        pub val: u8,
        pub speed: u8,
        pub flags: u8,
    }

    /// RGB config for RGB lighting subsystem
    #[derive(BinRead, BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct RgbLightConfig {
        pub enable: u8,
        pub mode: u8,
        pub hue: u8,
        pub sat: u8,
        pub val: u8,
        pub speed: u8,
    }

    /// Config for audio subsystem
    #[derive(BinRead, BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct AudioConfig {
        pub enable: u8,
        pub clicky_enable: u8,
    }

    /// Packet format for outbound data.
    #[derive(BinRead, BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct ResponseHeader {
        pub length: u8,
    }
}

#[macro_export]
macro_rules! generate_specta_builder {
                (commands: [$($command:ident),*], events: [$($event:ident),*]) => {{
                    let specta_builder = tauri_specta::ts::builder()
                        .commands(tauri_specta::collect_commands![
                            crate::xap_spec::xap::xap_version,
                            crate::xap_spec::xap::xap_capabilities,
                            crate::xap_spec::xap::xap_enabled_subsystem_capabilities,
                            crate::xap_spec::xap::xap_secure_status,
                            crate::xap_spec::xap::xap_secure_unlock,
                            crate::xap_spec::xap::xap_secure_lock,
                            crate::xap_spec::qmk::qmk_version,
                            crate::xap_spec::qmk::qmk_capabilities,
                            crate::xap_spec::qmk::qmk_board_identifiers,
                            crate::xap_spec::qmk::qmk_board_manufacturer,
                            crate::xap_spec::qmk::qmk_product_name,
                            crate::xap_spec::qmk::qmk_config_blob_length,
                            crate::xap_spec::qmk::qmk_config_blob_chunk,
                            crate::xap_spec::qmk::qmk_jump_to_bootloader,
                            crate::xap_spec::qmk::qmk_hardware_identifier,
                            crate::xap_spec::qmk::qmk_reinitialize_eeprom,
                            crate::xap_spec::keymap::keymap_capabilities,
                            crate::xap_spec::keymap::keymap_get_layer_count,
                            crate::xap_spec::keymap::keymap_get_keycode,
                            crate::xap_spec::keymap::keymap_get_encoder_keycode,
                            crate::xap_spec::remapping::remapping_capabilities,
                            crate::xap_spec::remapping::remapping_get_layer_count,
                            crate::xap_spec::remapping::remapping_set_keycode,
                            crate::xap_spec::remapping::remapping_set_encoder_keycode,
                            crate::xap_spec::lighting::lighting_capabilities,
                            crate::xap_spec::lighting::backlight::backlight_capabilities,
                            crate::xap_spec::lighting::backlight::backlight_get_enabled_effects,
                            crate::xap_spec::lighting::backlight::backlight_get_config,
                            crate::xap_spec::lighting::backlight::backlight_set_config,
                            crate::xap_spec::lighting::backlight::backlight_save_config,
                            crate::xap_spec::lighting::rgblight::rgblight_capabilities,
                            crate::xap_spec::lighting::rgblight::rgblight_get_enabled_effects,
                            crate::xap_spec::lighting::rgblight::rgblight_get_config,
                            crate::xap_spec::lighting::rgblight::rgblight_set_config,
                            crate::xap_spec::lighting::rgblight::rgblight_save_config,
                            crate::xap_spec::lighting::rgbmatrix::rgbmatrix_capabilities,
                            crate::xap_spec::lighting::rgbmatrix::rgbmatrix_get_enabled_effects,
                            crate::xap_spec::lighting::rgbmatrix::rgbmatrix_get_config,
                            crate::xap_spec::lighting::rgbmatrix::rgbmatrix_set_config,
                            crate::xap_spec::lighting::rgbmatrix::rgbmatrix_save_config,
                            crate::xap_spec::audio::audio_capabilities,
                            crate::xap_spec::audio::audio_get_config,
                            crate::xap_spec::audio::audio_set_config,
                            crate::xap_spec::audio::audio_save_config,
                            $($command),*
                        ]).events(tauri_specta::collect_events![$($event),*]);

                    specta_builder
                }};
            }
