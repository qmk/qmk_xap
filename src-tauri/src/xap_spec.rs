
#[allow(dead_code)]
#[allow(unused_imports)]
pub mod xap {
    use std::sync::Arc;

    use binrw::{BinRead, BinWrite};
    use bitflags::bitflags;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use specta::Type;
    #[cfg(feature = "tauri-codegen")]
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XapClient;
    use crate::xap::FrontendResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XapRequest;
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

    impl XapRequest for XapVersionRequest {
        type Response = XapVersionResponse;

        fn id() -> &'static [u8] {
            &[0x00, 0x00]
        }

        fn xap_version() -> u32 {
            0x00000001
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct XapVersionResponse(pub u32);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn xap_version(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<XapVersionResponse> {
        state
            .lock()
            .query(id, XapVersionRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    ///  capabilities
    ///
    /// XAP subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapCapabilitiesRequest(pub ());

    impl XapRequest for XapCapabilitiesRequest {
        type Response = XapCapabilitiesFlags;

        fn id() -> &'static [u8] {
            &[0x00, 0x01]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct XapCapabilitiesFlags(u32);

    bitflags! {
                    impl XapCapabilitiesFlags: u32 {

    const Version = 1 << 0;
    const Capabilities = 1 << 1;
    const EnabledSubsystemCapabilities = 1 << 2;
    const SecureStatus = 1 << 3;
    const SecureUnlock = 1 << 4;
    const SecureLock = 1 << 5;
    }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn xap_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<XapCapabilitiesFlags> {
        state
            .lock()
            .query(id, XapCapabilitiesRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Enabled subsystem capabilities
    ///
    /// XAP protocol subsystem query. Each bit should be considered as a "usable" subsystem. For example, checking `(value & (1 << XAP_ROUTE_QMK) != 0)` means the QMK subsystem is enabled and available for querying.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapEnabledSubsystemCapabilitiesRequest(pub ());

    impl XapRequest for XapEnabledSubsystemCapabilitiesRequest {
        type Response = XapEnabledSubsystemCapabilitiesFlags;

        fn id() -> &'static [u8] {
            &[0x00, 0x02]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct XapEnabledSubsystemCapabilitiesFlags(u32);

    bitflags! {
                    impl XapEnabledSubsystemCapabilitiesFlags: u32 {

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

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn xap_enabled_subsystem_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<XapEnabledSubsystemCapabilitiesFlags> {
        state
            .lock()
            .query(id, XapEnabledSubsystemCapabilitiesRequest(()))
            .map_err(Into::into)
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

    impl XapRequest for XapSecureStatusRequest {
        type Response = XapSecureStatusResponse;

        fn id() -> &'static [u8] {
            &[0x00, 0x03]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct XapSecureStatusResponse(pub u8);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn xap_secure_status(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<XapSecureStatusResponse> {
        state
            .lock()
            .query(id, XapSecureStatusRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Secure Unlock
    ///
    /// Initiate secure route unlock sequence
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapSecureUnlockRequest(pub ());

    impl XapRequest for XapSecureUnlockRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[0x00, 0x04]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn xap_secure_unlock(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<()> {
        state
            .lock()
            .query(id, XapSecureUnlockRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Secure Lock
    ///
    /// Disable secure routes
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct XapSecureLockRequest(pub ());

    impl XapRequest for XapSecureLockRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[0x00, 0x05]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn xap_secure_lock(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<()> {
        state
            .lock()
            .query(id, XapSecureLockRequest(()))
            .map_err(Into::into)
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
    #[cfg(feature = "tauri-codegen")]
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XapClient;
    use crate::xap::FrontendResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XapRequest;
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

    impl XapRequest for QmkVersionRequest {
        type Response = QmkVersionResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x00]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkVersionResponse(pub u32);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_version(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkVersionResponse> {
        state
            .lock()
            .query(id, QmkVersionRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    ///  capabilities
    ///
    /// QMK subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkCapabilitiesRequest(pub ());

    impl XapRequest for QmkCapabilitiesRequest {
        type Response = QmkCapabilitiesFlags;

        fn id() -> &'static [u8] {
            &[0x01, 0x01]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct QmkCapabilitiesFlags(u32);

    bitflags! {
                    impl QmkCapabilitiesFlags: u32 {

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

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkCapabilitiesFlags> {
        state
            .lock()
            .query(id, QmkCapabilitiesRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Board identifiers
    ///
    /// Retrieves the set of identifying information for the board.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkBoardIdentifiersRequest(pub ());

    impl XapRequest for QmkBoardIdentifiersRequest {
        type Response = QmkBoardIdentifiersResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x02]
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

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_board_identifiers(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkBoardIdentifiersResponse> {
        state
            .lock()
            .query(id, QmkBoardIdentifiersRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Board Manufacturer
    ///
    /// Retrieves the name of the manufacturer
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkBoardManufacturerRequest(pub ());

    impl XapRequest for QmkBoardManufacturerRequest {
        type Response = QmkBoardManufacturerResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x03]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkBoardManufacturerResponse(pub UTF8String);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_board_manufacturer(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkBoardManufacturerResponse> {
        state
            .lock()
            .query(id, QmkBoardManufacturerRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Product Name
    ///
    /// Retrieves the product name
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkProductNameRequest(pub ());

    impl XapRequest for QmkProductNameRequest {
        type Response = QmkProductNameResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x04]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkProductNameResponse(pub UTF8String);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_product_name(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkProductNameResponse> {
        state
            .lock()
            .query(id, QmkProductNameRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Config Blob Length
    ///
    /// Retrieves the length of the configuration data bundled within the firmware
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkConfigBlobLengthRequest(pub ());

    impl XapRequest for QmkConfigBlobLengthRequest {
        type Response = QmkConfigBlobLengthResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x05]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkConfigBlobLengthResponse(pub u16);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_config_blob_length(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkConfigBlobLengthResponse> {
        state
            .lock()
            .query(id, QmkConfigBlobLengthRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Config Blob Chunk
    ///
    /// Retrieves a chunk of the configuration data bundled within the firmware
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkConfigBlobChunkRequest(pub u16);

    impl XapRequest for QmkConfigBlobChunkRequest {
        type Response = QmkConfigBlobChunkResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x06]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkConfigBlobChunkResponse(pub [u8; 32]);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_config_blob_chunk(
        id: Uuid,
        arg: u16,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkConfigBlobChunkResponse> {
        state
            .lock()
            .query(id, QmkConfigBlobChunkRequest(arg))
            .map_err(Into::into)
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

    impl XapRequest for QmkJumpToBootloaderRequest {
        type Response = QmkJumpToBootloaderResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x07]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkJumpToBootloaderResponse(pub u8);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_jump_to_bootloader(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkJumpToBootloaderResponse> {
        state
            .lock()
            .query(id, QmkJumpToBootloaderRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Hardware Identifier
    ///
    /// Retrieves a unique identifier for the board.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkHardwareIdentifierRequest(pub ());

    impl XapRequest for QmkHardwareIdentifierRequest {
        type Response = QmkHardwareIdentifierResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x08]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkHardwareIdentifierResponse(pub [u32; 4]);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_hardware_identifier(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkHardwareIdentifierResponse> {
        state
            .lock()
            .query(id, QmkHardwareIdentifierRequest(()))
            .map_err(Into::into)
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

    impl XapRequest for QmkReinitializeEepromRequest {
        type Response = QmkReinitializeEepromResponse;

        fn id() -> &'static [u8] {
            &[0x01, 0x09]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct QmkReinitializeEepromResponse(pub u8);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn qmk_reinitialize_eeprom(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<QmkReinitializeEepromResponse> {
        state
            .lock()
            .query(id, QmkReinitializeEepromRequest(()))
            .map_err(Into::into)
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
    #[cfg(feature = "tauri-codegen")]
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XapClient;
    use crate::xap::FrontendResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XapRequest;
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
    #[cfg(feature = "tauri-codegen")]
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XapClient;
    use crate::xap::FrontendResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XapRequest;
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
    #[cfg(feature = "tauri-codegen")]
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XapClient;
    use crate::xap::FrontendResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XapRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    ///  capabilities
    ///
    /// Keymap subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapCapabilitiesRequest(pub ());

    impl XapRequest for KeymapCapabilitiesRequest {
        type Response = KeymapCapabilitiesFlags;

        fn id() -> &'static [u8] {
            &[0x04, 0x01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct KeymapCapabilitiesFlags(u32);

    bitflags! {
                    impl KeymapCapabilitiesFlags: u32 {

    const Capabilities = 1 << 1;
    const GetLayerCount = 1 << 2;
    const GetKeycode = 1 << 3;
    const GetEncoderKeycode = 1 << 4;
    }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn keymap_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<KeymapCapabilitiesFlags> {
        state
            .lock()
            .query(id, KeymapCapabilitiesRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Get Layer Count
    ///
    /// Query maximum number of layers that can be addressed within the keymap.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetLayerCountRequest(pub ());

    impl XapRequest for KeymapGetLayerCountRequest {
        type Response = KeymapGetLayerCountResponse;

        fn id() -> &'static [u8] {
            &[0x04, 0x02]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetLayerCountResponse(pub u8);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn keymap_get_layer_count(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<KeymapGetLayerCountResponse> {
        state
            .lock()
            .query(id, KeymapGetLayerCountRequest(()))
            .map_err(Into::into)
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

    impl XapRequest for KeymapGetKeycodeRequest {
        type Response = KeymapGetKeycodeResponse;

        fn id() -> &'static [u8] {
            &[0x04, 0x03]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetKeycodeResponse(pub u16);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn keymap_get_keycode(
        id: Uuid,
        arg: KeymapGetKeycodeArg,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<KeymapGetKeycodeResponse> {
        state
            .lock()
            .query(id, KeymapGetKeycodeRequest(arg))
            .map_err(Into::into)
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

    impl XapRequest for KeymapGetEncoderKeycodeRequest {
        type Response = KeymapGetEncoderKeycodeResponse;

        fn id() -> &'static [u8] {
            &[0x04, 0x04]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct KeymapGetEncoderKeycodeResponse(pub u16);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn keymap_get_encoder_keycode(
        id: Uuid,
        arg: KeymapGetEncoderKeycodeArg,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<KeymapGetEncoderKeycodeResponse> {
        state
            .lock()
            .query(id, KeymapGetEncoderKeycodeRequest(arg))
            .map_err(Into::into)
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
    #[cfg(feature = "tauri-codegen")]
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XapClient;
    use crate::xap::FrontendResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XapRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    ///  capabilities
    ///
    /// Remapping subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingCapabilitiesRequest(pub ());

    impl XapRequest for RemappingCapabilitiesRequest {
        type Response = RemappingCapabilitiesFlags;

        fn id() -> &'static [u8] {
            &[0x05, 0x01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct RemappingCapabilitiesFlags(u32);

    bitflags! {
                    impl RemappingCapabilitiesFlags: u32 {

    const Capabilities = 1 << 1;
    const GetLayerCount = 1 << 2;
    const SetKeycode = 1 << 3;
    const SetEncoderKeycode = 1 << 4;
    }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn remapping_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<RemappingCapabilitiesFlags> {
        state
            .lock()
            .query(id, RemappingCapabilitiesRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Get Layer Count
    ///
    /// Query maximum number of layers that can be addressed within the keymap.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingGetLayerCountRequest(pub ());

    impl XapRequest for RemappingGetLayerCountRequest {
        type Response = RemappingGetLayerCountResponse;

        fn id() -> &'static [u8] {
            &[0x05, 0x02]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
    pub struct RemappingGetLayerCountResponse(pub u8);

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn remapping_get_layer_count(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<RemappingGetLayerCountResponse> {
        state
            .lock()
            .query(id, RemappingGetLayerCountRequest(()))
            .map_err(Into::into)
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

    impl XapRequest for RemappingSetKeycodeRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[0x05, 0x03]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn remapping_set_keycode(
        id: Uuid,
        arg: RemappingSetKeycodeArg,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<()> {
        state
            .lock()
            .query(id, RemappingSetKeycodeRequest(arg))
            .map_err(Into::into)
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

    impl XapRequest for RemappingSetEncoderKeycodeRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[0x05, 0x04]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn remapping_set_encoder_keycode(
        id: Uuid,
        arg: RemappingSetEncoderKeycodeArg,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<()> {
        state
            .lock()
            .query(id, RemappingSetEncoderKeycodeRequest(arg))
            .map_err(Into::into)
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
    #[cfg(feature = "tauri-codegen")]
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XapClient;
    use crate::xap::FrontendResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XapRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    ///  capabilities
    ///
    /// Lighting subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct LightingCapabilitiesRequest(pub ());

    impl XapRequest for LightingCapabilitiesRequest {
        type Response = LightingCapabilitiesFlags;

        fn id() -> &'static [u8] {
            &[0x06, 0x01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct LightingCapabilitiesFlags(u32);

    bitflags! {
                    impl LightingCapabilitiesFlags: u32 {

    const Capabilities = 1 << 1;
    const Backlight = 1 << 2;
    const Rgblight = 1 << 3;
    const Rgbmatrix = 1 << 4;
    }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn lighting_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<LightingCapabilitiesFlags> {
        state
            .lock()
            .query(id, LightingCapabilitiesRequest(()))
            .map_err(Into::into)
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
        #[cfg(feature = "tauri-codegen")]
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XapClient;
        use crate::xap::FrontendResult;
        use crate::xap_spec::types::*;
        use xap_specs::request::XapRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        ///  capabilities
        ///
        /// backlight subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightCapabilitiesRequest(pub ());

        impl XapRequest for BacklightCapabilitiesRequest {
            type Response = BacklightCapabilitiesFlags;

            fn id() -> &'static [u8] {
                &[0x06, 0x02, 0x01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(
            BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
        )]
        pub struct BacklightCapabilitiesFlags(u32);

        bitflags! {
                        impl BacklightCapabilitiesFlags: u32 {

        const Capabilities = 1 << 1;
        const GetEnabledEffects = 1 << 2;
        const GetConfig = 1 << 3;
        const SetConfig = 1 << 4;
        const SaveConfig = 1 << 5;
        }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn backlight_capabilities(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<BacklightCapabilitiesFlags> {
            state
                .lock()
                .query(id, BacklightCapabilitiesRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightGetEnabledEffectsRequest(pub ());

        impl XapRequest for BacklightGetEnabledEffectsRequest {
            type Response = BacklightGetEnabledEffectsResponse;

            fn id() -> &'static [u8] {
                &[0x06, 0x02, 0x02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightGetEnabledEffectsResponse(pub u8);

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn backlight_get_enabled_effects(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<BacklightGetEnabledEffectsResponse> {
            state
                .lock()
                .query(id, BacklightGetEnabledEffectsRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightGetConfigRequest(pub ());

        impl XapRequest for BacklightGetConfigRequest {
            type Response = BacklightConfig;

            fn id() -> &'static [u8] {
                &[0x06, 0x02, 0x03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn backlight_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<BacklightConfig> {
            state
                .lock()
                .query(id, BacklightGetConfigRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightSetConfigRequest(pub BacklightConfig);

        impl XapRequest for BacklightSetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[0x06, 0x02, 0x04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn backlight_set_config(
            id: Uuid,
            arg: BacklightConfig,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<()> {
            state
                .lock()
                .query(id, BacklightSetConfigRequest(arg))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct BacklightSaveConfigRequest(pub ());

        impl XapRequest for BacklightSaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[0x06, 0x02, 0x05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn backlight_save_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<()> {
            state
                .lock()
                .query(id, BacklightSaveConfigRequest(()))
                .map_err(Into::into)
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
        #[cfg(feature = "tauri-codegen")]
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XapClient;
        use crate::xap::FrontendResult;
        use crate::xap_spec::types::*;
        use xap_specs::request::XapRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        ///  capabilities
        ///
        /// rgblight subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightCapabilitiesRequest(pub ());

        impl XapRequest for RgblightCapabilitiesRequest {
            type Response = RgblightCapabilitiesFlags;

            fn id() -> &'static [u8] {
                &[0x06, 0x03, 0x01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(
            BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
        )]
        pub struct RgblightCapabilitiesFlags(u32);

        bitflags! {
                        impl RgblightCapabilitiesFlags: u32 {

        const Capabilities = 1 << 1;
        const GetEnabledEffects = 1 << 2;
        const GetConfig = 1 << 3;
        const SetConfig = 1 << 4;
        const SaveConfig = 1 << 5;
        }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_capabilities(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<RgblightCapabilitiesFlags> {
            state
                .lock()
                .query(id, RgblightCapabilitiesRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightGetEnabledEffectsRequest(pub ());

        impl XapRequest for RgblightGetEnabledEffectsRequest {
            type Response = RgblightGetEnabledEffectsResponse;

            fn id() -> &'static [u8] {
                &[0x06, 0x03, 0x02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightGetEnabledEffectsResponse(pub u64);

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_get_enabled_effects(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<RgblightGetEnabledEffectsResponse> {
            state
                .lock()
                .query(id, RgblightGetEnabledEffectsRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightGetConfigRequest(pub ());

        impl XapRequest for RgblightGetConfigRequest {
            type Response = RgbLightConfig;

            fn id() -> &'static [u8] {
                &[0x06, 0x03, 0x03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<RgbLightConfig> {
            state
                .lock()
                .query(id, RgblightGetConfigRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightSetConfigRequest(pub RgbLightConfig);

        impl XapRequest for RgblightSetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[0x06, 0x03, 0x04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_set_config(
            id: Uuid,
            arg: RgbLightConfig,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<()> {
            state
                .lock()
                .query(id, RgblightSetConfigRequest(arg))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgblightSaveConfigRequest(pub ());

        impl XapRequest for RgblightSaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[0x06, 0x03, 0x05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgblight_save_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<()> {
            state
                .lock()
                .query(id, RgblightSaveConfigRequest(()))
                .map_err(Into::into)
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
        #[cfg(feature = "tauri-codegen")]
        use tauri::State;
        use uuid::Uuid;

        use crate::xap::hid::XapClient;
        use crate::xap::FrontendResult;
        use crate::xap_spec::types::*;
        use xap_specs::request::XapRequest;
        use xap_specs::response::UTF8String;

        /// ======================================================================
        ///  capabilities
        ///
        /// rgb matrix subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixCapabilitiesRequest(pub ());

        impl XapRequest for RgbmatrixCapabilitiesRequest {
            type Response = RgbmatrixCapabilitiesFlags;

            fn id() -> &'static [u8] {
                &[0x06, 0x04, 0x01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(
            BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
        )]
        pub struct RgbmatrixCapabilitiesFlags(u32);

        bitflags! {
                        impl RgbmatrixCapabilitiesFlags: u32 {

        const Capabilities = 1 << 1;
        const GetEnabledEffects = 1 << 2;
        const GetConfig = 1 << 3;
        const SetConfig = 1 << 4;
        const SaveConfig = 1 << 5;
        }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_capabilities(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<RgbmatrixCapabilitiesFlags> {
            state
                .lock()
                .query(id, RgbmatrixCapabilitiesRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixGetEnabledEffectsRequest(pub ());

        impl XapRequest for RgbmatrixGetEnabledEffectsRequest {
            type Response = RgbmatrixGetEnabledEffectsResponse;

            fn id() -> &'static [u8] {
                &[0x06, 0x04, 0x02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixGetEnabledEffectsResponse(pub u64);

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_get_enabled_effects(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<RgbmatrixGetEnabledEffectsResponse> {
            state
                .lock()
                .query(id, RgbmatrixGetEnabledEffectsRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixGetConfigRequest(pub ());

        impl XapRequest for RgbmatrixGetConfigRequest {
            type Response = RgbMatrixConfig;

            fn id() -> &'static [u8] {
                &[0x06, 0x04, 0x03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_get_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<RgbMatrixConfig> {
            state
                .lock()
                .query(id, RgbmatrixGetConfigRequest(()))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixSetConfigRequest(pub RgbMatrixConfig);

        impl XapRequest for RgbmatrixSetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[0x06, 0x04, 0x04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_set_config(
            id: Uuid,
            arg: RgbMatrixConfig,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<()> {
            state
                .lock()
                .query(id, RgbmatrixSetConfigRequest(arg))
                .map_err(Into::into)
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================
        #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
        pub struct RgbmatrixSaveConfigRequest(pub ());

        impl XapRequest for RgbmatrixSaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[0x06, 0x04, 0x05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[cfg(feature = "tauri-codegen")]
        #[tauri::command]
        #[specta::specta]
        pub fn rgbmatrix_save_config(
            id: Uuid,
            state: State<'_, Arc<Mutex<XapClient>>>,
        ) -> FrontendResult<()> {
            state
                .lock()
                .query(id, RgbmatrixSaveConfigRequest(()))
                .map_err(Into::into)
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
    #[cfg(feature = "tauri-codegen")]
    use tauri::State;
    use uuid::Uuid;

    use crate::xap::hid::XapClient;
    use crate::xap::FrontendResult;
    use crate::xap_spec::types::*;
    use xap_specs::request::XapRequest;
    use xap_specs::response::UTF8String;

    /// ======================================================================
    ///  capabilities
    ///
    /// Audio subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioCapabilitiesRequest(pub ());

    impl XapRequest for AudioCapabilitiesRequest {
        type Response = AudioCapabilitiesFlags;

        fn id() -> &'static [u8] {
            &[0x07, 0x01]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[derive(
        BinRead, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Type,
    )]
    pub struct AudioCapabilitiesFlags(u32);

    bitflags! {
                    impl AudioCapabilitiesFlags: u32 {

    const Capabilities = 1 << 1;
    const GetConfig = 1 << 3;
    const SetConfig = 1 << 4;
    const SaveConfig = 1 << 5;
    }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn audio_capabilities(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<AudioCapabilitiesFlags> {
        state
            .lock()
            .query(id, AudioCapabilitiesRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Get Config
    ///
    /// Query the current config.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioGetConfigRequest(pub ());

    impl XapRequest for AudioGetConfigRequest {
        type Response = AudioConfig;

        fn id() -> &'static [u8] {
            &[0x07, 0x03]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn audio_get_config(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<AudioConfig> {
        state
            .lock()
            .query(id, AudioGetConfigRequest(()))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Set Config
    ///
    /// Set the current config.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioSetConfigRequest(pub AudioConfig);

    impl XapRequest for AudioSetConfigRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[0x07, 0x04]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn audio_set_config(
        id: Uuid,
        arg: AudioConfig,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<()> {
        state
            .lock()
            .query(id, AudioSetConfigRequest(arg))
            .map_err(Into::into)
    }

    /// ======================================================================
    /// Save Config
    ///
    /// Save the current config.
    /// ======================================================================
    #[derive(BinWrite, Default, Debug, Clone, Serialize, Type)]
    pub struct AudioSaveConfigRequest(pub ());

    impl XapRequest for AudioSaveConfigRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[0x07, 0x05]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[cfg(feature = "tauri-codegen")]
    #[tauri::command]
    #[specta::specta]
    pub fn audio_save_config(
        id: Uuid,
        state: State<'_, Arc<Mutex<XapClient>>>,
    ) -> FrontendResult<()> {
        state
            .lock()
            .query(id, AudioSaveConfigRequest(()))
            .map_err(Into::into)
    }
}

pub mod types {
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};
    use specta::Type;

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

    /// Packet format for broadcast messages.
    #[derive(BinRead, BinWrite, Default, Debug, Clone, Serialize, Deserialize, Type)]
    pub struct BroadcastHeader {
        pub r#type: u8,
        pub length: u8,
    }
}

#[cfg(feature = "tauri-codegen")]
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
