use serde::{Deserialize, Serialize};
use specta::Type;

/// Bit-pattern constants mirroring `qmk_firmware_ref/quantum/keycodes.h` and
/// `qmk_firmware_ref/quantum/modifiers.h`. The `qmk_header_constants_match_local`
/// test in this module reads those headers at test time and fails loudly if any
/// of these values drift from upstream QMK.
pub mod consts {
    pub const QK_MODS: u16 = 0x0100;
    pub const QK_MOD_TAP: u16 = 0x2000;
    pub const QK_LAYER_TAP: u16 = 0x4000;
    pub const QK_LAYER_MOD: u16 = 0x5000;
    pub const QK_TO: u16 = 0x5200;
    pub const QK_MOMENTARY: u16 = 0x5220;
    pub const QK_DEF_LAYER: u16 = 0x5240;
    pub const QK_TOGGLE_LAYER: u16 = 0x5260;
    pub const QK_ONE_SHOT_LAYER: u16 = 0x5280;
    pub const QK_ONE_SHOT_MOD: u16 = 0x52A0;
    pub const QK_LAYER_TAP_TOGGLE: u16 = 0x52C0;
    pub const QK_PERSISTENT_DEF_LAYER: u16 = 0x52E0;

    pub const MOD_LCTL: u8 = 0x01;
    pub const MOD_LSFT: u8 = 0x02;
    pub const MOD_LALT: u8 = 0x04;
    pub const MOD_LGUI: u8 = 0x08;
    pub const MOD_RCTL: u8 = 0x11;
    pub const MOD_RSFT: u8 = 0x12;
    pub const MOD_RALT: u8 = 0x14;
    pub const MOD_RGUI: u8 = 0x18;
    pub const MOD_HYPR: u8 = MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI;
    pub const MOD_MEH: u8 = MOD_LCTL | MOD_LSFT | MOD_LALT;
}

use consts::*;

#[derive(Type, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerOp {
    MO,
    TG,
    TO,
    DF,
    OSL,
    TT,
    PDF,
}

impl LayerOp {
    pub fn base(self) -> u16 {
        match self {
            LayerOp::TO => QK_TO,
            LayerOp::MO => QK_MOMENTARY,
            LayerOp::DF => QK_DEF_LAYER,
            LayerOp::TG => QK_TOGGLE_LAYER,
            LayerOp::OSL => QK_ONE_SHOT_LAYER,
            LayerOp::TT => QK_LAYER_TAP_TOGGLE,
            LayerOp::PDF => QK_PERSISTENT_DEF_LAYER,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            LayerOp::MO => "MO",
            LayerOp::TG => "TG",
            LayerOp::TO => "TO",
            LayerOp::DF => "DF",
            LayerOp::OSL => "OSL",
            LayerOp::TT => "TT",
            LayerOp::PDF => "PDF",
        }
    }
}

/// A parameterized keycode the GUI either decoded from firmware or is in the
/// middle of assembling via the picker. Variants with `Option` slots are the
/// two-step pickers: the slot is `None` while the picker is collecting the
/// remaining input.
#[derive(Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum KeycodeTemplate {
    LayerOp { op: LayerOp, layer: u8 },
    OneShotMod { mod_mask: u8 },
    LayerTap { layer: u8, tap_kc: Option<u8> },
    ModTap { mod_mask: u8, tap_kc: Option<u8> },
    LayerMod { layer: u8, mod_mask: Option<u8> },
    Modified { mod_mask: u8, base_kc: Option<u8> },
}

impl KeycodeTemplate {
    pub fn is_complete(&self) -> bool {
        use KeycodeTemplate::*;
        match self {
            LayerOp { .. } | OneShotMod { .. } => true,
            LayerTap { tap_kc, .. } => tap_kc.is_some(),
            ModTap { tap_kc, .. } => tap_kc.is_some(),
            LayerMod { mod_mask, .. } => mod_mask.is_some(),
            Modified { base_kc, .. } => base_kc.is_some(),
        }
    }

    /// Encode a complete template into the 16-bit value the firmware expects.
    /// Returns `None` when the template still has a missing slot.
    pub fn encode(&self) -> Option<u16> {
        use KeycodeTemplate::*;
        match *self {
            LayerOp { op, layer } => Some(op.base() | (layer as u16 & 0x1F)),
            OneShotMod { mod_mask } => Some(QK_ONE_SHOT_MOD | (mod_mask as u16 & 0x1F)),
            LayerTap { layer, tap_kc: Some(kc) } => {
                Some(QK_LAYER_TAP | ((layer as u16 & 0x0F) << 8) | kc as u16)
            }
            ModTap { mod_mask, tap_kc: Some(kc) } => {
                Some(QK_MOD_TAP | ((mod_mask as u16 & 0x1F) << 8) | kc as u16)
            }
            LayerMod { layer, mod_mask: Some(mods) } => {
                Some(QK_LAYER_MOD | ((layer as u16 & 0x0F) << 5) | (mods as u16 & 0x1F))
            }
            Modified { mod_mask, base_kc: Some(kc) } => {
                // QMK's QK_MODS keys are encoded as `(mod_mask << 8) | kc` with no
                // additional range offset. The `QK_MODS` constant is just the
                // start of the decoded range (which equals MOD_LCTL << 8).
                Some(((mod_mask as u16 & 0x1F) << 8) | kc as u16)
            }
            _ => None,
        }
    }
}

/// Parse a QMK-style modifier name (e.g. `LSFT`, `RCTL`, `MEH`, `HYPR`, `LCAG`)
/// into its 5-bit mod mask. Recognises both the single-side aliases (`MEH`,
/// `HYPR`) and the L/R-prefixed combos emitted by the decoder
/// (`LCS`, `RCAG`, ...). Returns `None` for unknown names.
pub fn parse_mod_name(name: &str) -> Option<u8> {
    Some(match name {
        "LCTL" => MOD_LCTL,
        "LSFT" => MOD_LSFT,
        "LALT" => MOD_LALT,
        "LGUI" => MOD_LGUI,
        "RCTL" => MOD_RCTL,
        "RSFT" => MOD_RSFT,
        "RALT" => MOD_RALT,
        "RGUI" => MOD_RGUI,
        "MEH" => MOD_MEH,
        "HYPR" => MOD_HYPR,
        "LCS" => MOD_LCTL | MOD_LSFT,
        "LCA" => MOD_LCTL | MOD_LALT,
        "LCG" => MOD_LCTL | MOD_LGUI,
        "LSA" => MOD_LSFT | MOD_LALT,
        "LSG" => MOD_LSFT | MOD_LGUI,
        "LAG" => MOD_LALT | MOD_LGUI,
        "LCAG" => MOD_LCTL | MOD_LALT | MOD_LGUI,
        "LCSG" => MOD_LCTL | MOD_LSFT | MOD_LGUI,
        "LSAG" => MOD_LSFT | MOD_LALT | MOD_LGUI,
        "LCSA" => MOD_LCTL | MOD_LSFT | MOD_LALT,
        "LCSAG" => MOD_HYPR,
        "RCS" => 0x10 | MOD_LCTL | MOD_LSFT,
        "RCA" => 0x10 | MOD_LCTL | MOD_LALT,
        "RCG" => 0x10 | MOD_LCTL | MOD_LGUI,
        "RSA" => 0x10 | MOD_LSFT | MOD_LALT,
        "RSG" => 0x10 | MOD_LSFT | MOD_LGUI,
        "RAG" => 0x10 | MOD_LALT | MOD_LGUI,
        "RCAG" => 0x10 | MOD_LCTL | MOD_LALT | MOD_LGUI,
        "RCSG" => 0x10 | MOD_LCTL | MOD_LSFT | MOD_LGUI,
        "RSAG" => 0x10 | MOD_LSFT | MOD_LALT | MOD_LGUI,
        "RCSA" => 0x10 | MOD_LCTL | MOD_LSFT | MOD_LALT,
        "RCSAG" => 0x10 | MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::keycode_decoder::decode_parameterized;
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// Spec-anchored fixed-value table. Each row cites the QMK source line for
    /// human verification. Source files live under `qmk_firmware_ref/`.
    #[test]
    fn fixed_value_table() {
        // qmk_firmware_ref/quantum/quantum_keycodes.h:28  QK_LSFT = 0x0200
        // qmk_firmware_ref/quantum/quantum_keycodes.h:46  LSFT(kc) = QK_LSFT | kc
        // -> LSFT(KC_A) = 0x0200 | 0x04 = 0x0204
        assert_eq!(
            KeycodeTemplate::Modified { mod_mask: MOD_LSFT, base_kc: Some(0x04) }.encode(),
            Some(0x0204)
        );
        // qmk_firmware_ref/quantum/keycodes.h:41  QK_MOD_TAP = 0x2000
        // -> LSFT_T(KC_A) = 0x2000 | (0x02<<8) | 0x04 = 0x2204
        assert_eq!(
            KeycodeTemplate::ModTap { mod_mask: MOD_LSFT, tap_kc: Some(0x04) }.encode(),
            Some(0x2204)
        );
        // qmk_firmware_ref/quantum/keycodes.h:43  QK_LAYER_TAP = 0x4000
        // -> LT(2, KC_A) = 0x4000 | (2<<8) | 0x04 = 0x4204
        assert_eq!(
            KeycodeTemplate::LayerTap { layer: 2, tap_kc: Some(0x04) }.encode(),
            Some(0x4204)
        );
        // qmk_firmware_ref/quantum/keycodes.h:45  QK_LAYER_MOD = 0x5000
        // -> LM(3, LCS) = 0x5000 | (3<<5) | 0x03 = 0x5063
        assert_eq!(
            KeycodeTemplate::LayerMod { layer: 3, mod_mask: Some(0x03) }.encode(),
            Some(0x5063)
        );
        // qmk_firmware_ref/quantum/keycodes.h:47  QK_TO = 0x5200          -> TO(2) = 0x5202
        // qmk_firmware_ref/quantum/keycodes.h:49  QK_MOMENTARY = 0x5220   -> MO(1) = 0x5221
        // qmk_firmware_ref/quantum/keycodes.h:51  QK_DEF_LAYER = 0x5240   -> DF(0) = 0x5240
        // qmk_firmware_ref/quantum/keycodes.h:53  QK_TOGGLE_LAYER = 0x5260 -> TG(3) = 0x5263
        // qmk_firmware_ref/quantum/keycodes.h:55  QK_ONE_SHOT_LAYER = 0x5280 -> OSL(5) = 0x5285
        // qmk_firmware_ref/quantum/keycodes.h:57  QK_ONE_SHOT_MOD = 0x52A0   -> OSM(LSFT) = 0x52A2
        // qmk_firmware_ref/quantum/keycodes.h:59  QK_LAYER_TAP_TOGGLE = 0x52C0 -> TT(2) = 0x52C2
        // qmk_firmware_ref/quantum/keycodes.h:61  QK_PERSISTENT_DEF_LAYER = 0x52E0 -> PDF(0) = 0x52E0
        for (template, expected) in [
            (KeycodeTemplate::LayerOp { op: LayerOp::TO, layer: 2 }, 0x5202),
            (KeycodeTemplate::LayerOp { op: LayerOp::MO, layer: 1 }, 0x5221),
            (KeycodeTemplate::LayerOp { op: LayerOp::DF, layer: 0 }, 0x5240),
            (KeycodeTemplate::LayerOp { op: LayerOp::TG, layer: 3 }, 0x5263),
            (KeycodeTemplate::LayerOp { op: LayerOp::OSL, layer: 5 }, 0x5285),
            (KeycodeTemplate::OneShotMod { mod_mask: MOD_LSFT }, 0x52A2),
            (KeycodeTemplate::LayerOp { op: LayerOp::TT, layer: 2 }, 0x52C2),
            (KeycodeTemplate::LayerOp { op: LayerOp::PDF, layer: 0 }, 0x52E0),
        ] {
            assert_eq!(template.encode(), Some(expected), "{template:?} -> {expected:#06X}");
        }
    }

    #[test]
    fn incomplete_templates_do_not_encode() {
        assert_eq!(KeycodeTemplate::LayerTap { layer: 0, tap_kc: None }.encode(), None);
        assert_eq!(KeycodeTemplate::ModTap { mod_mask: MOD_LSFT, tap_kc: None }.encode(), None);
        assert_eq!(KeycodeTemplate::LayerMod { layer: 0, mod_mask: None }.encode(), None);
        assert_eq!(KeycodeTemplate::Modified { mod_mask: MOD_LSFT, base_kc: None }.encode(), None);
    }

    /// Round-trip: every encodable template must decode back to a KeyCode
    /// carrying the same template.
    #[test]
    fn encode_decode_round_trip() {
        let lookup = HashMap::new(); // labels not relevant here
        let cases = [
            KeycodeTemplate::LayerOp { op: LayerOp::MO, layer: 1 },
            KeycodeTemplate::LayerOp { op: LayerOp::TG, layer: 3 },
            KeycodeTemplate::LayerOp { op: LayerOp::TO, layer: 2 },
            KeycodeTemplate::LayerOp { op: LayerOp::DF, layer: 0 },
            KeycodeTemplate::LayerOp { op: LayerOp::OSL, layer: 5 },
            KeycodeTemplate::LayerOp { op: LayerOp::TT, layer: 2 },
            KeycodeTemplate::LayerOp { op: LayerOp::PDF, layer: 0 },
            KeycodeTemplate::OneShotMod { mod_mask: MOD_LSFT },
            KeycodeTemplate::OneShotMod { mod_mask: MOD_MEH },
            KeycodeTemplate::LayerTap { layer: 0, tap_kc: Some(0x04) },
            KeycodeTemplate::LayerTap { layer: 15, tap_kc: Some(0x2C) },
            KeycodeTemplate::ModTap { mod_mask: MOD_LSFT, tap_kc: Some(0x04) },
            KeycodeTemplate::ModTap { mod_mask: MOD_HYPR, tap_kc: Some(0x09) },
            KeycodeTemplate::ModTap { mod_mask: MOD_RSFT, tap_kc: Some(0x04) },
            KeycodeTemplate::LayerMod { layer: 0, mod_mask: Some(MOD_LSFT) },
            KeycodeTemplate::LayerMod { layer: 3, mod_mask: Some(MOD_LCTL | MOD_LSFT) },
            KeycodeTemplate::Modified { mod_mask: MOD_LSFT, base_kc: Some(0x04) },
            KeycodeTemplate::Modified { mod_mask: MOD_MEH, base_kc: Some(0x04) },
            KeycodeTemplate::Modified { mod_mask: MOD_HYPR, base_kc: Some(0x04) },
        ];
        for t in cases {
            let encoded = t.encode().unwrap_or_else(|| panic!("encode failed for {t:?}"));
            let kc = decode_parameterized(encoded, &lookup)
                .unwrap_or_else(|| panic!("decode failed for {encoded:#06X} ({t:?})"));
            assert_eq!(kc.template.as_ref(), Some(&t), "round-trip mismatch for {t:?} ({encoded:#06X})");
        }
    }

    fn read_qmk_file(rel: &str) -> String {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xap-specs has a parent dir")
            .join("qmk_firmware_ref")
            .join(rel);
        std::fs::read_to_string(&p)
            .unwrap_or_else(|e| panic!("can't read {}: {e}", p.display()))
    }

    /// Extracts `NAME = 0xXXXX` from a header file, respecting word boundaries.
    fn extract_hex(content: &str, name: &str) -> Option<u32> {
        for line in content.lines() {
            let line = line.trim();
            let Some(rest) = line.strip_prefix(name) else { continue };
            // Word boundary: next char must not be alnum/underscore.
            let next = rest.chars().next();
            if next.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }
            let rest = rest.trim_start();
            let Some(rest) = rest.strip_prefix('=') else { continue };
            let rest = rest.trim_start();
            let Some(hex) = rest.strip_prefix("0x") else { continue };
            let end = hex.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(hex.len());
            return u32::from_str_radix(&hex[..end], 16).ok();
        }
        None
    }

    /// Header-drift detector. If QMK upstream renames or renumbers any of
    /// these constants, this test fails with a message naming the offender.
    #[test]
    fn qmk_header_constants_match_local() {
        let header = read_qmk_file("quantum/keycodes.h");
        let modifiers = read_qmk_file("quantum/modifiers.h");

        let keycode_consts: &[(&str, u32)] = &[
            ("QK_MODS", QK_MODS as u32),
            ("QK_MOD_TAP", QK_MOD_TAP as u32),
            ("QK_LAYER_TAP", QK_LAYER_TAP as u32),
            ("QK_LAYER_MOD", QK_LAYER_MOD as u32),
            ("QK_TO", QK_TO as u32),
            ("QK_MOMENTARY", QK_MOMENTARY as u32),
            ("QK_DEF_LAYER", QK_DEF_LAYER as u32),
            ("QK_TOGGLE_LAYER", QK_TOGGLE_LAYER as u32),
            ("QK_ONE_SHOT_LAYER", QK_ONE_SHOT_LAYER as u32),
            ("QK_ONE_SHOT_MOD", QK_ONE_SHOT_MOD as u32),
            ("QK_LAYER_TAP_TOGGLE", QK_LAYER_TAP_TOGGLE as u32),
            ("QK_PERSISTENT_DEF_LAYER", QK_PERSISTENT_DEF_LAYER as u32),
        ];
        for (name, expected) in keycode_consts {
            let actual = extract_hex(&header, name)
                .unwrap_or_else(|| panic!("QMK keycodes.h missing `{name}`"));
            assert_eq!(
                actual, *expected,
                "{name}: QMK={actual:#06X}, local={expected:#06X}"
            );
        }

        let mod_consts: &[(&str, u8)] = &[
            ("MOD_LCTL", MOD_LCTL),
            ("MOD_LSFT", MOD_LSFT),
            ("MOD_LALT", MOD_LALT),
            ("MOD_LGUI", MOD_LGUI),
            ("MOD_RCTL", MOD_RCTL),
            ("MOD_RSFT", MOD_RSFT),
            ("MOD_RALT", MOD_RALT),
            ("MOD_RGUI", MOD_RGUI),
        ];
        for (name, expected) in mod_consts {
            let actual = extract_hex(&modifiers, name)
                .unwrap_or_else(|| panic!("QMK modifiers.h missing `{name}`"));
            assert_eq!(
                actual as u8, *expected,
                "{name}: QMK={actual:#04X}, local={expected:#04X}"
            );
        }
    }

    #[test]
    fn parse_mod_name_round_trip_against_decoder() {
        // Every label the decoder might emit must be parseable back to the same mask.
        // The decoder's mods_name in keycode_decoder.rs is the source of these labels.
        for mods in 0u8..=0x1F {
            // Compose the same label the decoder would produce.
            let bits = mods & 0x0F;
            let is_right = mods & 0x10 != 0;
            // The decoder only ever sees mods with at least one mod-bit set
            // (the right-side flag alone is invalid input). Skip those here so
            // we don't trip the decoder's own `unreachable!` on `bits == 0`.
            if bits == 0 {
                continue;
            }
            let label = if mods == 0 {
                "0".to_owned()
            } else if !is_right && bits == 0x07 {
                "MEH".to_owned()
            } else if !is_right && bits == 0x0F {
                "HYPR".to_owned()
            } else {
                let abbrev = match bits {
                    0x01 => "CTL",
                    0x02 => "SFT",
                    0x03 => "CS",
                    0x04 => "ALT",
                    0x05 => "CA",
                    0x06 => "SA",
                    0x07 => "CSA",
                    0x08 => "GUI",
                    0x09 => "CG",
                    0x0A => "SG",
                    0x0B => "CSG",
                    0x0C => "AG",
                    0x0D => "CAG",
                    0x0E => "SAG",
                    0x0F => "CSAG",
                    _ => unreachable!(),
                };
                let side = if is_right { 'R' } else { 'L' };
                format!("{side}{abbrev}")
            };
            if label == "0" {
                continue; // no parse rule for the empty modifier
            }
            assert_eq!(
                parse_mod_name(&label),
                Some(mods),
                "label `{label}` should parse back to mod mask {mods:#04X}"
            );
        }
    }
}
