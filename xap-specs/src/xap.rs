
pub mod xap_routes {
    use crate::{request::XAPRequest, response::UTF8String};
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};

    /// ======================================================================
    /// Version Query
    ///
    /// XAP protocol version query.
    ///
    /// * Returns the BCD-encoded version in the format of XX.YY.ZZZZ => `0xXXYYZZZZ`
    ///     * e.g. 3.2.115 will match `0x03020115`, or bytes {0x15,0x01,0x02,0x03}.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct VersionQueryRequest(pub ());

    impl XAPRequest for VersionQueryRequest {
        type Response = VersionQueryResponseArg;

        fn id() -> &'static [u8] {
            &[00, 00]
        }

        fn xap_version() -> u32 {
            0x00000001
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct VersionQueryResponseArg(pub u32);

    /// ======================================================================
    /// Capabilities Query
    ///
    /// XAP subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryRequest(pub ());

    impl XAPRequest for CapabilitiesQueryRequest {
        type Response = CapabilitiesQueryResponseArg;

        fn id() -> &'static [u8] {
            &[00, 01]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryResponseArg(pub u32);

    /// ======================================================================
    /// Enabled subsystem query
    ///
    /// XAP protocol subsystem query. Each bit should be considered as a "usable" subsystem. For example, checking `(value & (1 << XAP_ROUTE_QMK) != 0)` means the QMK subsystem is enabled and available for querying.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct EnabledSubsystemQueryRequest(pub ());

    impl XAPRequest for EnabledSubsystemQueryRequest {
        type Response = EnabledSubsystemQueryResponseArg;

        fn id() -> &'static [u8] {
            &[00, 02]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct EnabledSubsystemQueryResponseArg(pub u32);

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

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SecureStatusRequest(pub ());

    impl XAPRequest for SecureStatusRequest {
        type Response = SecureStatusResponseArg;

        fn id() -> &'static [u8] {
            &[00, 03]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct SecureStatusResponseArg(pub u8);

    /// ======================================================================
    /// Secure Unlock
    ///
    /// Initiate secure route unlock sequence
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SecureUnlockRequest(pub ());

    impl XAPRequest for SecureUnlockRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[00, 04]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    /// ======================================================================
    /// Secure Lock
    ///
    /// Disable secure routes
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SecureLockRequest(pub ());

    impl XAPRequest for SecureLockRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[00, 05]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }
}

pub mod qmk_routes {
    use crate::{request::XAPRequest, response::UTF8String};
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};

    /// ======================================================================
    /// Version Query
    ///
    /// QMK protocol version query.
    ///
    /// * Returns the BCD-encoded version in the format of XX.YY.ZZZZ => `0xXXYYZZZZ`
    ///     * e.g. 3.2.115 will match `0x03020115`, or bytes {0x15,0x01,0x02,0x03}.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct VersionQueryRequest(pub ());

    impl XAPRequest for VersionQueryRequest {
        type Response = VersionQueryResponseArg;

        fn id() -> &'static [u8] {
            &[01, 00]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct VersionQueryResponseArg(pub u32);

    /// ======================================================================
    /// Capabilities Query
    ///
    /// QMK subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryRequest(pub ());

    impl XAPRequest for CapabilitiesQueryRequest {
        type Response = CapabilitiesQueryResponseArg;

        fn id() -> &'static [u8] {
            &[01, 01]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryResponseArg(pub u32);

    /// ======================================================================
    /// Board identifiers
    ///
    /// Retrieves the set of identifying information for the board.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct BoardIdentifiersRequest(pub ());

    impl XAPRequest for BoardIdentifiersRequest {
        type Response = BoardIdentifiersResponseArg;

        fn id() -> &'static [u8] {
            &[01, 02]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct BoardIdentifiersResponseArg {
        pub vendor_id: u16,
        pub product_id: u16,
        pub product_version: u16,
        pub qmk_unique_identifier: u32,
    }

    /// ======================================================================
    /// Board Manufacturer
    ///
    /// Retrieves the name of the manufacturer
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct BoardManufacturerRequest(pub ());

    impl XAPRequest for BoardManufacturerRequest {
        type Response = BoardManufacturerResponseArg;

        fn id() -> &'static [u8] {
            &[01, 03]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct BoardManufacturerResponseArg(pub UTF8String);

    /// ======================================================================
    /// Product Name
    ///
    /// Retrieves the product name
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct ProductNameRequest(pub ());

    impl XAPRequest for ProductNameRequest {
        type Response = ProductNameResponseArg;

        fn id() -> &'static [u8] {
            &[01, 04]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct ProductNameResponseArg(pub UTF8String);

    /// ======================================================================
    /// Config Blob Length
    ///
    /// Retrieves the length of the configuration data bundled within the firmware
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct ConfigBlobLengthRequest(pub ());

    impl XAPRequest for ConfigBlobLengthRequest {
        type Response = ConfigBlobLengthResponseArg;

        fn id() -> &'static [u8] {
            &[01, 05]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct ConfigBlobLengthResponseArg(pub u16);

    /// ======================================================================
    /// Config Blob Chunk
    ///
    /// Retrieves a chunk of the configuration data bundled within the firmware
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct ConfigBlobChunkRequest(pub u16);

    impl XAPRequest for ConfigBlobChunkRequest {
        type Response = ConfigBlobChunkResponseArg;

        fn id() -> &'static [u8] {
            &[01, 06]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct ConfigBlobChunkResponseArg(pub [u8; 32]);

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

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct JumpToBootloaderRequest(pub ());

    impl XAPRequest for JumpToBootloaderRequest {
        type Response = JumpToBootloaderResponseArg;

        fn id() -> &'static [u8] {
            &[01, 07]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct JumpToBootloaderResponseArg(pub u8);

    /// ======================================================================
    /// Hardware Identifier
    ///
    /// Retrieves a unique identifier for the board.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct HardwareIdentifierRequest(pub ());

    impl XAPRequest for HardwareIdentifierRequest {
        type Response = HardwareIdentifierResponseArg;

        fn id() -> &'static [u8] {
            &[01, 08]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct HardwareIdentifierResponseArg(pub [u32; 4]);

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

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct ReinitializeEepromRequest(pub ());

    impl XAPRequest for ReinitializeEepromRequest {
        type Response = ReinitializeEepromResponseArg;

        fn id() -> &'static [u8] {
            &[01, 09]
        }

        fn xap_version() -> u32 {
            0x00000100
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct ReinitializeEepromResponseArg(pub u8);
}

pub mod keyboard_routes {
    use crate::{request::XAPRequest, response::UTF8String};
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};
}

pub mod user_routes {
    use crate::{request::XAPRequest, response::UTF8String};
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};
}

pub mod keymap_routes {
    use crate::{request::XAPRequest, response::UTF8String};
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};

    /// ======================================================================
    /// Capabilities Query
    ///
    /// Keymap subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryRequest(pub ());

    impl XAPRequest for CapabilitiesQueryRequest {
        type Response = CapabilitiesQueryResponseArg;

        fn id() -> &'static [u8] {
            &[04, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryResponseArg(pub u32);

    /// ======================================================================
    /// Get Layer Count
    ///
    /// Query maximum number of layers that can be addressed within the keymap.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct GetLayerCountRequest(pub ());

    impl XAPRequest for GetLayerCountRequest {
        type Response = GetLayerCountResponseArg;

        fn id() -> &'static [u8] {
            &[04, 02]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct GetLayerCountResponseArg(pub u8);

    /// ======================================================================
    /// Get Keycode
    ///
    /// Query the Keycode at the requested location.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct GetKeycodeRequestArg {
        pub layer: u8,
        pub row: u8,
        pub column: u8,
    }

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct GetKeycodeRequest(pub GetKeycodeRequestArg);

    impl XAPRequest for GetKeycodeRequest {
        type Response = GetKeycodeResponseArg;

        fn id() -> &'static [u8] {
            &[04, 03]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct GetKeycodeResponseArg(pub u16);

    /// ======================================================================
    /// Get Encoder Keycode
    ///
    /// Query the Keycode at the requested location.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct GetEncoderKeycodeRequestArg {
        pub layer: u8,
        pub encoder: u8,
        pub clockwise: u8,
    }

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct GetEncoderKeycodeRequest(pub GetEncoderKeycodeRequestArg);

    impl XAPRequest for GetEncoderKeycodeRequest {
        type Response = GetEncoderKeycodeResponseArg;

        fn id() -> &'static [u8] {
            &[04, 04]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct GetEncoderKeycodeResponseArg(pub u16);
}

pub mod remapping_routes {
    use crate::{request::XAPRequest, response::UTF8String};
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};

    /// ======================================================================
    /// Capabilities Query
    ///
    /// Remapping subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryRequest(pub ());

    impl XAPRequest for CapabilitiesQueryRequest {
        type Response = CapabilitiesQueryResponseArg;

        fn id() -> &'static [u8] {
            &[05, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryResponseArg(pub u32);

    /// ======================================================================
    /// Get Layer Count
    ///
    /// Query maximum number of layers that can be addressed within the keymap.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct GetLayerCountRequest(pub ());

    impl XAPRequest for GetLayerCountRequest {
        type Response = GetLayerCountResponseArg;

        fn id() -> &'static [u8] {
            &[05, 02]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct GetLayerCountResponseArg(pub u8);

    /// ======================================================================
    /// Set Keycode
    ///
    /// Modify the Keycode at the requested location.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SetKeycodeRequestArg {
        pub layer: u8,
        pub row: u8,
        pub column: u8,
        pub keycode: u16,
    }

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SetKeycodeRequest(pub SetKeycodeRequestArg);

    impl XAPRequest for SetKeycodeRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[05, 03]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    /// ======================================================================
    /// Set Encoder Keycode
    ///
    /// Modify the Keycode at the requested location.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SetEncoderKeycodeRequestArg {
        pub layer: u8,
        pub encoder: u8,
        pub clockwise: u8,
        pub keycode: u16,
    }

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SetEncoderKeycodeRequest(pub SetEncoderKeycodeRequestArg);

    impl XAPRequest for SetEncoderKeycodeRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[05, 04]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }
}

pub mod lighting_routes {
    use crate::{request::XAPRequest, response::UTF8String};
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};

    /// ======================================================================
    /// Capabilities Query
    ///
    /// Lighting subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryRequest(pub ());

    impl XAPRequest for CapabilitiesQueryRequest {
        type Response = CapabilitiesQueryResponseArg;

        fn id() -> &'static [u8] {
            &[06, 01]
        }

        fn xap_version() -> u32 {
            0x00000200
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryResponseArg(pub u32);

    pub mod backlight_routes {
        use crate::{request::XAPRequest, response::UTF8String};
        use binrw::{BinRead, BinWrite};
        use serde::{Deserialize, Serialize};

        /// ======================================================================
        /// Capabilities Query
        ///
        /// backlight subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct CapabilitiesQueryRequest(pub ());

        impl XAPRequest for CapabilitiesQueryRequest {
            type Response = CapabilitiesQueryResponseArg;

            fn id() -> &'static [u8] {
                &[06, 02, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct CapabilitiesQueryResponseArg(pub u32);

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct GetEnabledEffectsRequest(pub ());

        impl XAPRequest for GetEnabledEffectsRequest {
            type Response = GetEnabledEffectsResponseArg;

            fn id() -> &'static [u8] {
                &[06, 02, 02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct GetEnabledEffectsResponseArg(pub u8);

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct GetConfigRequest(pub ());

        impl XAPRequest for GetConfigRequest {
            type Response = GetConfigResponseArg;

            fn id() -> &'static [u8] {
                &[06, 02, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct GetConfigResponseArg {
            pub enable: u8,
            pub mode: u8,
            pub val: u8,
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SetConfigRequestArg {
            pub enable: u8,
            pub mode: u8,
            pub val: u8,
        }

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SetConfigRequest(pub SetConfigRequestArg);

        impl XAPRequest for SetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 02, 04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SaveConfigRequest(pub ());

        impl XAPRequest for SaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 02, 05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }
    }

    pub mod rgblight_routes {
        use crate::{request::XAPRequest, response::UTF8String};
        use binrw::{BinRead, BinWrite};
        use serde::{Deserialize, Serialize};

        /// ======================================================================
        /// Capabilities Query
        ///
        /// rgblight subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct CapabilitiesQueryRequest(pub ());

        impl XAPRequest for CapabilitiesQueryRequest {
            type Response = CapabilitiesQueryResponseArg;

            fn id() -> &'static [u8] {
                &[06, 03, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct CapabilitiesQueryResponseArg(pub u32);

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct GetEnabledEffectsRequest(pub ());

        impl XAPRequest for GetEnabledEffectsRequest {
            type Response = GetEnabledEffectsResponseArg;

            fn id() -> &'static [u8] {
                &[06, 03, 02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct GetEnabledEffectsResponseArg(pub u64);

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct GetConfigRequest(pub ());

        impl XAPRequest for GetConfigRequest {
            type Response = GetConfigResponseArg;

            fn id() -> &'static [u8] {
                &[06, 03, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct GetConfigResponseArg {
            pub enable: u8,
            pub mode: u8,
            pub hue: u8,
            pub sat: u8,
            pub val: u8,
            pub speed: u8,
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SetConfigRequestArg {
            pub enable: u8,
            pub mode: u8,
            pub hue: u8,
            pub sat: u8,
            pub val: u8,
            pub speed: u8,
        }

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SetConfigRequest(pub SetConfigRequestArg);

        impl XAPRequest for SetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 03, 04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SaveConfigRequest(pub ());

        impl XAPRequest for SaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 03, 05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }
    }

    pub mod rgbmatrix_routes {
        use crate::{request::XAPRequest, response::UTF8String};
        use binrw::{BinRead, BinWrite};
        use serde::{Deserialize, Serialize};

        /// ======================================================================
        /// Capabilities Query
        ///
        /// rgb matrix subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct CapabilitiesQueryRequest(pub ());

        impl XAPRequest for CapabilitiesQueryRequest {
            type Response = CapabilitiesQueryResponseArg;

            fn id() -> &'static [u8] {
                &[06, 04, 01]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct CapabilitiesQueryResponseArg(pub u32);

        /// ======================================================================
        /// Get Enabled Effects
        ///
        /// Each bit should be considered as a "usable" effect id
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct GetEnabledEffectsRequest(pub ());

        impl XAPRequest for GetEnabledEffectsRequest {
            type Response = GetEnabledEffectsResponseArg;

            fn id() -> &'static [u8] {
                &[06, 04, 02]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct GetEnabledEffectsResponseArg(pub u64);

        /// ======================================================================
        /// Get Config
        ///
        /// Query the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct GetConfigRequest(pub ());

        impl XAPRequest for GetConfigRequest {
            type Response = GetConfigResponseArg;

            fn id() -> &'static [u8] {
                &[06, 04, 03]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        #[derive(BinRead, Default, Debug, Clone, Serialize)]
        pub struct GetConfigResponseArg {
            pub enable: u8,
            pub mode: u8,
            pub hue: u8,
            pub sat: u8,
            pub val: u8,
            pub speed: u8,
            pub flags: u8,
        }

        /// ======================================================================
        /// Set Config
        ///
        /// Set the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SetConfigRequestArg {
            pub enable: u8,
            pub mode: u8,
            pub hue: u8,
            pub sat: u8,
            pub val: u8,
            pub speed: u8,
            pub flags: u8,
        }

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SetConfigRequest(pub SetConfigRequestArg);

        impl XAPRequest for SetConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 04, 04]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }

        /// ======================================================================
        /// Save Config
        ///
        /// Save the current config.
        /// ======================================================================

        #[derive(BinWrite, Default, Debug, Clone, Serialize)]
        pub struct SaveConfigRequest(pub ());

        impl XAPRequest for SaveConfigRequest {
            type Response = ();

            fn id() -> &'static [u8] {
                &[06, 04, 05]
            }

            fn xap_version() -> u32 {
                0x00000300
            }
        }
    }
}

pub mod audio_routes {
    use crate::{request::XAPRequest, response::UTF8String};
    use binrw::{BinRead, BinWrite};
    use serde::{Deserialize, Serialize};

    /// ======================================================================
    /// Capabilities Query
    ///
    /// Audio subsystem capabilities query. Each bit should be considered as a "usable" route within this subsystem.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryRequest(pub ());

    impl XAPRequest for CapabilitiesQueryRequest {
        type Response = CapabilitiesQueryResponseArg;

        fn id() -> &'static [u8] {
            &[07, 01]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct CapabilitiesQueryResponseArg(pub u32);

    /// ======================================================================
    /// Get Config
    ///
    /// Query the current config.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct GetConfigRequest(pub ());

    impl XAPRequest for GetConfigRequest {
        type Response = GetConfigResponseArg;

        fn id() -> &'static [u8] {
            &[07, 03]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    #[derive(BinRead, Default, Debug, Clone, Serialize)]
    pub struct GetConfigResponseArg {
        pub enable: u8,
        pub clicky_enable: u8,
    }

    /// ======================================================================
    /// Set Config
    ///
    /// Set the current config.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SetConfigRequestArg {
        pub enable: u8,
        pub clicky_enable: u8,
    }

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SetConfigRequest(pub SetConfigRequestArg);

    impl XAPRequest for SetConfigRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[07, 04]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }

    /// ======================================================================
    /// Save Config
    ///
    /// Save the current config.
    /// ======================================================================

    #[derive(BinWrite, Default, Debug, Clone, Serialize)]
    pub struct SaveConfigRequest(pub ());

    impl XAPRequest for SaveConfigRequest {
        type Response = ();

        fn id() -> &'static [u8] {
            &[07, 05]
        }

        fn xap_version() -> u32 {
            0x00000300
        }
    }
}
