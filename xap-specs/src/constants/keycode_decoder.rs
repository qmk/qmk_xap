use std::collections::HashMap;

use super::keycode::KeyCode;
use super::keycode_encoder::{KeycodeTemplate, LayerOp};

// Range bases / ends, mirroring `qmk_firmware_ref/quantum/quantum_keycodes.h`.
const QK_MODS_START: u16 = 0x0100;
const QK_MODS_END_EXCL: u16 = 0x2000;
const QK_MOD_TAP_START: u16 = 0x2000;
const QK_MOD_TAP_END_EXCL: u16 = 0x4000;
const QK_LAYER_TAP_START: u16 = 0x4000;
const QK_LAYER_TAP_END_EXCL: u16 = 0x5000;
const QK_LAYER_MOD_START: u16 = 0x5000;
const QK_LAYER_MOD_END_EXCL: u16 = 0x5200;
const QK_TO_START: u16 = 0x5200;
const QK_MOMENTARY_START: u16 = 0x5220;
const QK_DEF_LAYER_START: u16 = 0x5240;
const QK_TOGGLE_LAYER_START: u16 = 0x5260;
const QK_ONE_SHOT_LAYER_START: u16 = 0x5280;
const QK_ONE_SHOT_MOD_START: u16 = 0x52A0;
const QK_LAYER_TAP_TOGGLE_START: u16 = 0x52C0;
const QK_PERSISTENT_DEF_LAYER_START: u16 = 0x52E0;
const LAYER_OP_SPAN: u16 = 0x20;

pub(crate) fn decode_parameterized(
    code: u16,
    lookup: &HashMap<u16, KeyCode>,
) -> Option<KeyCode> {
    // Narrow 0x52xx sub-ranges first; QK_LAYER_MOD / QK_LAYER_TAP / QK_MOD_TAP /
    // QK_MODS are the wide fallbacks.
    if in_layer_op(code, QK_TO_START) {
        return Some(layer_op(code, "TO", QK_TO_START, LayerOp::TO));
    }
    if in_layer_op(code, QK_MOMENTARY_START) {
        return Some(layer_op(code, "MO", QK_MOMENTARY_START, LayerOp::MO));
    }
    if in_layer_op(code, QK_DEF_LAYER_START) {
        return Some(layer_op(code, "DF", QK_DEF_LAYER_START, LayerOp::DF));
    }
    if in_layer_op(code, QK_TOGGLE_LAYER_START) {
        return Some(layer_op(code, "TG", QK_TOGGLE_LAYER_START, LayerOp::TG));
    }
    if in_layer_op(code, QK_ONE_SHOT_LAYER_START) {
        return Some(layer_op(code, "OSL", QK_ONE_SHOT_LAYER_START, LayerOp::OSL));
    }
    if in_layer_op(code, QK_ONE_SHOT_MOD_START) {
        return Some(one_shot_mod(code));
    }
    if in_layer_op(code, QK_LAYER_TAP_TOGGLE_START) {
        return Some(layer_op(code, "TT", QK_LAYER_TAP_TOGGLE_START, LayerOp::TT));
    }
    if in_layer_op(code, QK_PERSISTENT_DEF_LAYER_START) {
        return Some(layer_op(code, "PDF", QK_PERSISTENT_DEF_LAYER_START, LayerOp::PDF));
    }
    if (QK_LAYER_MOD_START..QK_LAYER_MOD_END_EXCL).contains(&code) {
        return Some(layer_mod(code));
    }
    if (QK_LAYER_TAP_START..QK_LAYER_TAP_END_EXCL).contains(&code) {
        return Some(layer_tap(code, lookup));
    }
    if (QK_MOD_TAP_START..QK_MOD_TAP_END_EXCL).contains(&code) {
        return Some(mod_tap(code, lookup));
    }
    if (QK_MODS_START..QK_MODS_END_EXCL).contains(&code) {
        return Some(qk_mods(code, lookup));
    }
    None
}

fn in_layer_op(code: u16, base: u16) -> bool {
    (base..base + LAYER_OP_SPAN).contains(&code)
}

fn layer_op(code: u16, prefix: &str, base: u16, op: LayerOp) -> KeyCode {
    let layer = (code - base) as u8;
    let mut kc = decoded(code, format!("{prefix}({layer})"), "layer");
    kc.template = Some(KeycodeTemplate::LayerOp { op, layer });
    kc
}

fn one_shot_mod(code: u16) -> KeyCode {
    let mods = (code & 0x1F) as u8;
    let mut kc = decoded(code, format!("OSM({})", mods_name(mods)), "one_shot_mod");
    kc.template = Some(KeycodeTemplate::OneShotMod { mod_mask: mods });
    kc
}

fn layer_mod(code: u16) -> KeyCode {
    let layer = ((code >> 5) & 0x0F) as u8;
    let mods = (code & 0x1F) as u8;
    let mods_label = mods_name(mods);
    let mut kc = decoded_split(
        code,
        format!("LM({layer}, {mods_label})"),
        format!("LM({layer})"),
        mods_label,
        "layer_mod",
    );
    kc.template = Some(KeycodeTemplate::LayerMod { layer, mod_mask: Some(mods) });
    kc
}

fn layer_tap(code: u16, lookup: &HashMap<u16, KeyCode>) -> KeyCode {
    let layer = ((code >> 8) & 0x0F) as u8;
    let basic = (code & 0xFF) as u8;
    let kc_label = basic_kc_label(basic, lookup);
    let mut kc = decoded_split(
        code,
        format!("LT({layer}, {kc_label})"),
        format!("LT({layer})"),
        kc_label,
        "layer_tap",
    );
    kc.template = Some(KeycodeTemplate::LayerTap { layer, tap_kc: Some(basic) });
    kc
}

fn mod_tap(code: u16, lookup: &HashMap<u16, KeyCode>) -> KeyCode {
    let mods = ((code >> 8) & 0x1F) as u8;
    let basic = (code & 0xFF) as u8;
    let kc_label = basic_kc_label(basic, lookup);
    let mut kc = if mods == 0 {
        // Degenerate MT(0, kc) - no meaningful hold action, render as single label.
        decoded(code, format!("MT(0, {kc_label})"), "mod_tap")
    } else {
        let top = format!("{}_T", mods_name(mods));
        decoded_split(code, format!("{top}({kc_label})"), top, kc_label, "mod_tap")
    };
    kc.template = Some(KeycodeTemplate::ModTap { mod_mask: mods, tap_kc: Some(basic) });
    kc
}

fn qk_mods(code: u16, lookup: &HashMap<u16, KeyCode>) -> KeyCode {
    let mods = ((code >> 8) & 0x1F) as u8;
    let basic = (code & 0xFF) as u8;
    let kc_label = basic_kc_label(basic, lookup);
    let label = if mods == 0 {
        kc_label
    } else if lookup
        .get(&(basic as u16))
        .is_some_and(|kc| kc.label.as_deref() == Some(""))
    {
        // QMK aliases KC_HYPR/KC_MEH to HYPR(KC_NO)/MEH(KC_NO).
        // An intentionally blank base label denotes a standalone modifier.
        mods_name(mods)
    } else {
        format!("{}({kc_label})", mods_name(mods))
    };
    let mut kc = decoded(code, label, "mods");
    kc.template = Some(KeycodeTemplate::Modified { mod_mask: mods, base_kc: Some(basic) });
    kc
}

fn decoded(code: u16, label: String, group: &str) -> KeyCode {
    KeyCode {
        code,
        key: label.clone(),
        group: Some(group.to_owned()),
        label: Some(label),
        top: None,
        bottom: None,
        aliases: Vec::new(),
        description: None,
        template: None,
    }
}

/// Decoded keycode whose visual representation splits into an upper "hold"
/// label and a lower "tap" / "secondary" label. `label` keeps the combined
/// macro form (e.g. `LSFT_T(A)`) for use in tooltips and the picker palette.
fn decoded_split(
    code: u16,
    label: String,
    top: String,
    bottom: String,
    group: &str,
) -> KeyCode {
    KeyCode {
        code,
        key: label.clone(),
        group: Some(group.to_owned()),
        label: Some(label),
        top: Some(top),
        bottom: Some(bottom),
        aliases: Vec::new(),
        description: None,
        template: None,
    }
}

/// Resolve the embedded basic keycode to a friendly label.
///
/// Prefers the catalog's `label`, falls back to `key` with the `KC_` prefix
/// stripped, and finally to `0xNN` if the keycode is unknown.
fn basic_kc_label(kc: u8, lookup: &HashMap<u16, KeyCode>) -> String {
    if let Some(found) = lookup.get(&(kc as u16)) {
        if let Some(label) = found.label.as_deref() {
            if !label.is_empty() {
                return label.to_owned();
            }
        }
        return found
            .key
            .strip_prefix("KC_")
            .unwrap_or(&found.key)
            .to_owned();
    }
    format!("0x{kc:02X}")
}

/// Return the QMK-style name for a 5-bit modifier field.
///
/// Bit layout (see `qmk_firmware_ref/quantum/modifiers.h`):
///   bit 4 = right-hand flag (0=left, 1=right)
///   bits 0-3 = CTRL/SHIFT/ALT/GUI (LSB first)
///
/// Examples: `0x02` -> `LSFT`, `0x12` -> `RSFT`, `0x07` -> `MEH`,
/// `0x0F` -> `HYPR`, `0x05` -> `LCA`, `0x15` -> `RCA`.
fn mods_name(mods: u8) -> String {
    let mods = mods & 0x1F;
    if mods == 0 {
        return "0".to_string();
    }

    // Left-only QMK aliases that don't use the L/R-prefix scheme.
    let bits = mods & 0x0F;
    let is_right = mods & 0x10 != 0;
    if !is_right {
        if bits == 0x07 {
            return "MEH".to_string();
        }
        if bits == 0x0F {
            return "HYPR".to_string();
        }
    }

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
        _ => unreachable!("bits is masked to 0x0F"),
    };
    let side = if is_right { 'R' } else { 'L' };
    format!("{side}{abbrev}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup() -> HashMap<u16, KeyCode> {
        let mut m = HashMap::new();
        m.insert(
            0x0000,
            KeyCode {
                code: 0x0000,
                key: "KC_NO".into(),
                group: Some("internal".into()),
                label: Some("".into()),
                top: None,
                bottom: None,
                aliases: vec![],
                description: None,
                template: None,
            },
        );
        m.insert(
            0x0001,
            KeyCode {
                code: 0x0001,
                key: "KC_TRANSPARENT".into(),
                group: Some("internal".into()),
                label: Some("Transparent".into()),
                top: None,
                bottom: None,
                aliases: vec![],
                description: None,
                template: None,
            },
        );
        m.insert(
            0x0004,
            KeyCode {
                code: 0x0004,
                key: "KC_A".into(),
                group: Some("basic".into()),
                label: Some("A".into()),
                top: None,
                bottom: None,
                aliases: vec![],
                description: None,
                template: None,
            },
        );
        m.insert(
            0x0009,
            KeyCode {
                code: 0x0009,
                key: "KC_F".into(),
                group: Some("basic".into()),
                label: Some("F".into()),
                top: None,
                bottom: None,
                aliases: vec![],
                description: None,
                template: None,
            },
        );
        m.insert(
            0x002C,
            KeyCode {
                code: 0x002C,
                key: "KC_SPACE".into(),
                group: Some("basic".into()),
                label: Some("Space".into()),
                top: None,
                bottom: None,
                aliases: vec![],
                description: None,
                template: None,
            },
        );
        // Entry without a label, to exercise the KC_ stripping fallback.
        m.insert(
            0x0080,
            KeyCode {
                code: 0x0080,
                key: "KC_QUIRKY".into(),
                group: Some("basic".into()),
                label: None,
                top: None,
                bottom: None,
                aliases: vec![],
                description: None,
                template: None,
            },
        );
        m
    }

    fn decode(code: u16) -> KeyCode {
        decode_parameterized(code, &lookup()).expect("expected a decoded keycode")
    }

    fn label(code: u16) -> String {
        decode(code).label.expect("decoded keycode has a label")
    }

    #[test]
    fn layer_single_arg_ops() {
        assert_eq!(label(0x5220), "MO(0)");
        assert_eq!(label(0x5221), "MO(1)");
        assert_eq!(label(0x523F), "MO(31)");
        assert_eq!(label(0x5200), "TO(0)");
        assert_eq!(label(0x5202), "TO(2)");
        assert_eq!(label(0x5240), "DF(0)");
        assert_eq!(label(0x5260), "TG(0)");
        assert_eq!(label(0x5263), "TG(3)");
        assert_eq!(label(0x5280), "OSL(0)");
        assert_eq!(label(0x5285), "OSL(5)");
        assert_eq!(label(0x52C0), "TT(0)");
        assert_eq!(label(0x52C2), "TT(2)");
        assert_eq!(label(0x52E0), "PDF(0)");
    }

    #[test]
    fn layer_tap_uses_recursive_label() {
        // LT(0, KC_A) = 0x4000 | (0<<8) | 0x04 = 0x4004
        assert_eq!(label(0x4004), "LT(0, A)");
        // LT(15, KC_SPACE) = 0x4000 | (15<<8) | 0x2C = 0x4F2C
        assert_eq!(label(0x4F2C), "LT(15, Space)");
        // LT(1, KC_TRANSPARENT) -> uses non-empty label "Transparent"
        assert_eq!(label(0x4101), "LT(1, Transparent)");
        // LT(2, KC_QUIRKY) -> KC_ prefix stripped from key.
        assert_eq!(label(0x4280), "LT(2, QUIRKY)");
        // LT with unknown basic keycode -> hex fallback.
        assert_eq!(label(0x41FE), "LT(1, 0xFE)");
    }

    #[test]
    fn layer_mod_decodes_layer_and_mods() {
        // LM(0, LSFT) = 0x5000 | (0<<5) | 0x02 = 0x5002
        assert_eq!(label(0x5002), "LM(0, LSFT)");
        // LM(3, LCTL|LSFT) = 0x5000 | (3<<5) | 0x03 = 0x5063
        assert_eq!(label(0x5063), "LM(3, LCS)");
    }

    #[test]
    fn mod_tap_single_mod_aliases() {
        // LCTL_T(KC_A) = 0x2000 | (0x01<<8) | 0x04 = 0x2104
        assert_eq!(label(0x2104), "LCTL_T(A)");
        assert_eq!(label(0x2204), "LSFT_T(A)"); // MOD_LSFT << 8 | KC_A
        assert_eq!(label(0x2404), "LALT_T(A)");
        assert_eq!(label(0x2804), "LGUI_T(A)");
        assert_eq!(label(0x3104), "RCTL_T(A)");
        assert_eq!(label(0x3204), "RSFT_T(A)");
        assert_eq!(label(0x3404), "RALT_T(A)");
        assert_eq!(label(0x3804), "RGUI_T(A)");
    }

    #[test]
    fn mod_tap_named_multi_mod_combos() {
        // MEH_T(KC_F) = MT(0x07, KC_F) = 0x2000 | (0x07<<8) | 0x09 = 0x2709
        assert_eq!(label(0x2709), "MEH_T(F)");
        // HYPR_T(KC_F) = MT(0x0F, KC_F) = 0x2F09
        assert_eq!(label(0x2F09), "HYPR_T(F)");
        // LCAG_T(KC_F) = MT(0x0D, KC_F) = 0x2D09
        assert_eq!(label(0x2D09), "LCAG_T(F)");
        // LCS_T(KC_A) = MT(0x03, KC_A) = 0x2304
        assert_eq!(label(0x2304), "LCS_T(A)");
    }

    #[test]
    fn mod_tap_right_side_combos_use_r_prefix() {
        // No firmware MEH/HYPR macro on the right side - we still produce a
        // readable name with an R-prefix.
        // MT(0x17, KC_A) -> "RCSA_T(A)" (bits=0x07, is_right=true)
        assert_eq!(label(0x2704 | 0x1000), "RCSA_T(A)");
        // MT(0x1F, KC_A) -> "RCSAG_T(A)"
        assert_eq!(label(0x2F04 | 0x1000), "RCSAG_T(A)");
        // MT(0x15, KC_A) (RCTL|RALT, no QK_MODS firmware macro) -> "RCA_T(A)"
        assert_eq!(label(0x2504 | 0x1000), "RCA_T(A)");
    }

    #[test]
    fn one_shot_mod() {
        // OSM(MOD_LSFT) = 0x52A0 | 0x02 = 0x52A2
        assert_eq!(label(0x52A2), "OSM(LSFT)");
        // OSM(MOD_MEH) = 0x52A0 | 0x07 = 0x52A7
        assert_eq!(label(0x52A7), "OSM(MEH)");
    }

    #[test]
    fn qk_mods_single_and_named() {
        // LSFT(KC_A) = QK_LSFT | KC_A = 0x0204
        assert_eq!(label(0x0204), "LSFT(A)");
        // HYPR(KC_A) = (QK_LCTL|QK_LSFT|QK_LALT|QK_LGUI) | KC_A = 0x0F04
        assert_eq!(label(0x0F04), "HYPR(A)");
        // MEH(KC_A) = 0x0704
        assert_eq!(label(0x0704), "MEH(A)");
        // KC_HYPR = HYPR(KC_NO); the blank operand produces the standalone alias.
        assert_eq!(label(0x0F00), "HYPR");
        // KC_MEH = MEH(KC_NO); same standalone-alias behavior.
        assert_eq!(label(0x0700), "MEH");
        // LCS(KC_A) = 0x0304
        assert_eq!(label(0x0304), "LCS(A)");
        // RCAG(KC_A) = 0x1D04
        assert_eq!(label(0x1D04), "RCAG(A)");
    }

    #[test]
    fn returns_none_outside_known_ranges() {
        let m = lookup();
        // 0x0080 is in QK_BASIC, not in any parameterised range.
        assert!(decode_parameterized(0x0080, &m).is_none());
        // 0x6500 is in the gap between QK_PERSISTENT_DEF_LAYER and the swap-hands
        // range; we don't decode it.
        assert!(decode_parameterized(0x6500, &m).is_none());
        // 0x5300 is between QK_PERSISTENT_DEF_LAYER (ends 0x52FF) and
        // QK_SWAP_HANDS (0x5600) - not decoded.
        assert!(decode_parameterized(0x5300, &m).is_none());
    }

    #[test]
    fn decoded_keycode_metadata() {
        let kc = decode(0x5221);
        assert_eq!(kc.code, 0x5221);
        assert_eq!(kc.key, "MO(1)");
        assert_eq!(kc.label.as_deref(), Some("MO(1)"));
        assert_eq!(kc.group.as_deref(), Some("layer"));
        assert!(kc.aliases.is_empty());

        let kc = decode(0x2204);
        assert_eq!(kc.group.as_deref(), Some("mod_tap"));
        let kc = decode(0x52A2);
        assert_eq!(kc.group.as_deref(), Some("one_shot_mod"));
        let kc = decode(0x0204);
        assert_eq!(kc.group.as_deref(), Some("mods"));
        let kc = decode(0x4004);
        assert_eq!(kc.group.as_deref(), Some("layer_tap"));
        let kc = decode(0x5002);
        assert_eq!(kc.group.as_deref(), Some("layer_mod"));
    }

    #[test]
    fn mod_tap_emits_split_fields() {
        // LSFT_T(KC_A) -> top "LSFT_T", bottom "A"
        let kc = decode(0x2204);
        assert_eq!(kc.top.as_deref(), Some("LSFT_T"));
        assert_eq!(kc.bottom.as_deref(), Some("A"));
        // HYPR_T(KC_F) -> top "HYPR_T", bottom "F"
        let kc = decode(0x2F09);
        assert_eq!(kc.top.as_deref(), Some("HYPR_T"));
        assert_eq!(kc.bottom.as_deref(), Some("F"));
        // RCAG_T(KC_A) -> top "RCAG_T", bottom "A"
        let kc = decode(0x2D04 | 0x1000);
        assert_eq!(kc.top.as_deref(), Some("RCAG_T"));
        assert_eq!(kc.bottom.as_deref(), Some("A"));
        // Degenerate MT(0, kc) keeps single-label form: no split.
        let kc = decode(0x2004);
        assert_eq!(kc.label.as_deref(), Some("MT(0, A)"));
        assert!(kc.top.is_none());
        assert!(kc.bottom.is_none());
    }

    #[test]
    fn layer_tap_emits_split_fields() {
        // LT(0, KC_A) -> top "LT(0)", bottom "A"
        let kc = decode(0x4004);
        assert_eq!(kc.top.as_deref(), Some("LT(0)"));
        assert_eq!(kc.bottom.as_deref(), Some("A"));
        // LT(15, KC_SPACE) -> top "LT(15)", bottom "Space"
        let kc = decode(0x4F2C);
        assert_eq!(kc.top.as_deref(), Some("LT(15)"));
        assert_eq!(kc.bottom.as_deref(), Some("Space"));
    }

    #[test]
    fn layer_mod_emits_split_fields() {
        // LM(0, LSFT) -> top "LM(0)", bottom "LSFT"
        let kc = decode(0x5002);
        assert_eq!(kc.top.as_deref(), Some("LM(0)"));
        assert_eq!(kc.bottom.as_deref(), Some("LSFT"));
        // LM(3, LCS) -> top "LM(3)", bottom "LCS"
        let kc = decode(0x5063);
        assert_eq!(kc.top.as_deref(), Some("LM(3)"));
        assert_eq!(kc.bottom.as_deref(), Some("LCS"));
    }

    #[test]
    fn single_label_decodings_have_no_split() {
        // Single-arg layer ops, OSM, QK_MODS - all unsplit.
        for code in [0x5221_u16, 0x5202, 0x5240, 0x5260, 0x5283, 0x52A2, 0x52C2, 0x52E0, 0x0204, 0x0F04] {
            let kc = decode(code);
            assert!(kc.top.is_none(), "{code:#06X} unexpectedly has top: {:?}", kc.top);
            assert!(kc.bottom.is_none(), "{code:#06X} unexpectedly has bottom: {:?}", kc.bottom);
        }
    }
}
