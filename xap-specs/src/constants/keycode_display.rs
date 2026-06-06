use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::Path,
};

use anyhow::Result;
use log::warn;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::keycode::KeyCode;

#[derive(Deserialize, Debug, Clone)]
pub struct KeycodeDisplay {
    #[serde(default)]
    pub version: String,
    pub target_keycode_version: String,
    pub tabs: Vec<TabSpec>,
    #[serde(default)]
    pub keycodes: HashMap<String, KeyOverride>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TabSpec {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub color: Option<String>,
    pub subgroups: Vec<SubgroupSpec>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SubgroupSpec {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub render_mode: Option<String>,
    #[serde(default)]
    pub from_group: Option<String>,
    #[serde(default)]
    pub from_groups: Vec<String>,
    #[serde(default)]
    pub keys: Option<Vec<String>>,
    #[serde(default)]
    pub template: Option<SubgroupTemplate>,
}

/// Parameterised subgroup descriptor. The wire type ships the descriptor
/// un-expanded; the frontend renders one button per layer / mod combination
/// using the connected keyboard's layer count.
#[derive(Deserialize, Serialize, Debug, Clone, Type)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubgroupTemplate {
    Mo,
    Tg,
    To,
    Df,
    Osl,
    Tt,
    Pdf,
    Lt,
    Lm,
    Mt { mods: Vec<String> },
    QkMods { mods: Vec<String> },
}

impl SubgroupSpec {
    fn iter_groups(&self) -> impl Iterator<Item = &str> {
        self.from_group
            .as_deref()
            .into_iter()
            .chain(self.from_groups.iter().map(String::as_str))
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct KeyOverride {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub hidden: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct KeycodeView {
    pub tabs: Vec<KeycodeViewTab>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct KeycodeViewTab {
    pub id: String,
    pub label: String,
    pub is_fallback: bool,
    pub color: Option<String>,
    pub subgroups: Vec<KeycodeViewSubgroup>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct KeycodeViewSubgroup {
    pub id: String,
    pub label: Option<String>,
    pub render_mode: Option<String>,
    pub is_fallback: bool,
    pub codes: Vec<KeyCode>,
    /// Parameterised subgroup. When present, `codes` is empty and the
    /// frontend expands one button per layer / mod combination at render time.
    pub template: Option<SubgroupTemplate>,
}

pub fn read_keycode_display(path: impl AsRef<Path>) -> Result<KeycodeDisplay> {
    let raw = read_to_string(path)?;
    Ok(deser_hjson::from_str(&raw)?)
}

pub fn build_name_to_code(codes: &HashMap<u16, KeyCode>) -> HashMap<String, u16> {
    let mut map = HashMap::with_capacity(codes.len());
    for (code, kc) in codes {
        map.insert(kc.key.clone(), *code);
    }
    map
}

pub fn apply_overrides(
    codes: &mut HashMap<u16, KeyCode>,
    display: &KeycodeDisplay,
    name_to_code: &HashMap<String, u16>,
) -> HashSet<u16> {
    let mut hidden = HashSet::new();
    for (name, ov) in &display.keycodes {
        let Some(code) = name_to_code.get(name).copied() else {
            warn!("keycode_display override targets unknown key '{name}'");
            continue;
        };
        let Some(kc) = codes.get_mut(&code) else {
            continue;
        };
        if let Some(label) = &ov.label {
            kc.label = Some(label.clone());
        }
        if let Some(desc) = &ov.description {
            kc.description = Some(desc.clone());
        }
        if ov.hidden.unwrap_or(false) {
            hidden.insert(code);
        }
    }
    hidden
}

pub fn build_remapped_view(
    display: &KeycodeDisplay,
    codes: &HashMap<u16, KeyCode>,
    name_to_code: &HashMap<String, u16>,
    hidden: &HashSet<u16>,
) -> KeycodeView {
    let mut tabs = Vec::new();

    // Pre-pass: every code mentioned in any explicit `keys: [...]` belongs to
    // its first-listed subgroup, regardless of declaration order. This lets us
    // keep `ansi` (`from_group: "basic"`) at the top of the basic tab while the
    // overflow subgroups still claim their keys.
    let mut explicit_owner: HashMap<u16, (usize, usize)> = HashMap::new();
    for (ti, tab) in display.tabs.iter().enumerate() {
        for (si, sg) in tab.subgroups.iter().enumerate() {
            let Some(keys) = sg.keys.as_ref() else {
                continue;
            };
            for name in keys {
                let Some(&code) = name_to_code.get(name) else {
                    continue;
                };
                if hidden.contains(&code) {
                    continue;
                }
                explicit_owner.entry(code).or_insert((ti, si));
            }
        }
    }

    let mut claimed: HashSet<u16> = hidden.clone();
    claimed.extend(explicit_owner.keys().copied());

    for (ti, tab) in display.tabs.iter().enumerate() {
        let mut view_subgroups = Vec::new();
        for (si, sg) in tab.subgroups.iter().enumerate() {
            // Parameterised subgroups carry a template descriptor and no concrete
            // catalog codes; the frontend expands them at render time.
            if let Some(template) = &sg.template {
                view_subgroups.push(KeycodeViewSubgroup {
                    id: sg.id.clone(),
                    label: sg.label.clone(),
                    render_mode: sg.render_mode.clone(),
                    is_fallback: false,
                    codes: Vec::new(),
                    template: Some(template.clone()),
                });
                continue;
            }

            let mut ordered: Vec<u16> = Vec::new();
            let mut seen_in_sg: HashSet<u16> = HashSet::new();

            if let Some(keys) = &sg.keys {
                for name in keys {
                    let Some(&code) = name_to_code.get(name) else {
                        warn!(
                            "keycode_display: subgroup '{}' references unknown key '{}'",
                            sg.id, name
                        );
                        continue;
                    };
                    // Only emit if this subgroup is the first owner of the code.
                    if explicit_owner.get(&code) == Some(&(ti, si)) && seen_in_sg.insert(code) {
                        ordered.push(code);
                    }
                }
            }

            let group_set: HashSet<&str> = sg.iter_groups().collect();
            if !group_set.is_empty() {
                let mut from_group: Vec<u16> = codes
                    .iter()
                    .filter_map(|(code, kc)| {
                        let in_group = kc
                            .group
                            .as_deref()
                            .is_some_and(|g| group_set.contains(g));
                        if in_group
                            && !hidden.contains(code)
                            && !claimed.contains(code)
                            && !seen_in_sg.contains(code)
                        {
                            Some(*code)
                        } else {
                            None
                        }
                    })
                    .collect();
                from_group.sort();
                for c in from_group {
                    if seen_in_sg.insert(c) {
                        ordered.push(c);
                    }
                }
            }

            for c in &ordered {
                claimed.insert(*c);
            }

            let codes_list: Vec<KeyCode> = ordered
                .iter()
                .filter_map(|c| codes.get(c).cloned())
                .collect();

            if codes_list.is_empty() {
                continue;
            }

            view_subgroups.push(KeycodeViewSubgroup {
                id: sg.id.clone(),
                label: sg.label.clone(),
                render_mode: sg.render_mode.clone(),
                is_fallback: false,
                codes: codes_list,
                template: None,
            });
        }

        if view_subgroups.is_empty() {
            continue;
        }

        tabs.push(KeycodeViewTab {
            id: tab.id.clone(),
            label: tab.label.clone(),
            is_fallback: false,
            color: tab.color.clone(),
            subgroups: view_subgroups,
        });
    }

    let mut fallback_by_group: HashMap<String, Vec<u16>> = HashMap::new();
    for (code, kc) in codes {
        if claimed.contains(code) {
            continue;
        }
        let group = kc.group.clone().unwrap_or_else(|| "other".to_owned());
        fallback_by_group.entry(group).or_default().push(*code);
    }
    let mut fallback_groups: Vec<(String, Vec<u16>)> = fallback_by_group.into_iter().collect();
    fallback_groups.sort_by(|a, b| a.0.cmp(&b.0));
    for (group, mut codes_in_group) in fallback_groups {
        codes_in_group.sort();
        let codes_list: Vec<KeyCode> = codes_in_group
            .iter()
            .filter_map(|c| codes.get(c).cloned())
            .collect();
        tabs.push(KeycodeViewTab {
            id: group.clone(),
            label: group.clone(),
            is_fallback: true,
            color: None,
            subgroups: vec![KeycodeViewSubgroup {
                id: group.clone(),
                label: None,
                render_mode: None,
                is_fallback: true,
                codes: codes_list,
                template: None,
            }],
        });
    }

    if !hidden.is_empty() {
        let mut codes_in_hidden: Vec<u16> = hidden.iter().copied().collect();
        codes_in_hidden.sort();
        let codes_list: Vec<KeyCode> = codes_in_hidden
            .iter()
            .filter_map(|c| codes.get(c).cloned())
            .collect();
        if !codes_list.is_empty() {
            tabs.push(KeycodeViewTab {
                id: "hidden".to_owned(),
                label: "Hidden".to_owned(),
                is_fallback: false,
                color: None,
                subgroups: vec![KeycodeViewSubgroup {
                    id: "hidden".to_owned(),
                    label: None,
                    render_mode: None,
                    is_fallback: false,
                    codes: codes_list,
                    template: None,
                }],
            });
        }
    }

    KeycodeView { tabs }
}

pub fn build_raw_view(codes: &HashMap<u16, KeyCode>) -> KeycodeView {
    let mut by_group: HashMap<String, Vec<u16>> = HashMap::new();
    for (code, kc) in codes {
        let group = kc.group.clone().unwrap_or_else(|| "other".to_owned());
        by_group.entry(group).or_default().push(*code);
    }
    let mut groups: Vec<(String, Vec<u16>)> = by_group.into_iter().collect();
    groups.sort_by(|a, b| a.0.cmp(&b.0));
    let tabs = groups
        .into_iter()
        .map(|(name, mut codes_in_group)| {
            codes_in_group.sort();
            let codes_list: Vec<KeyCode> = codes_in_group
                .iter()
                .filter_map(|c| codes.get(c).cloned())
                .collect();
            KeycodeViewTab {
                id: name.clone(),
                label: name.clone(),
                is_fallback: true,
                color: None,
                subgroups: vec![KeycodeViewSubgroup {
                    id: name.clone(),
                    label: None,
                    render_mode: None,
                    is_fallback: true,
                    codes: codes_list,
                    template: None,
                }],
            }
        })
        .collect();
    KeycodeView { tabs }
}

pub fn build_view_for_version(
    display: Option<&KeycodeDisplay>,
    active_version: &str,
    codes: &HashMap<u16, KeyCode>,
    name_to_code: &HashMap<String, u16>,
    hidden: &HashSet<u16>,
) -> KeycodeView {
    if let Some(display) = display {
        if display.target_keycode_version == active_version {
            return build_remapped_view(display, codes, name_to_code, hidden);
        }
    }
    build_raw_view(codes)
}

#[cfg(test)]
mod test {
    use super::*;
    use similar_asserts::assert_eq;

    fn kc(code: u16, key: &str, group: &str, label: &str) -> KeyCode {
        KeyCode {
            code,
            key: key.to_owned(),
            group: Some(group.to_owned()),
            label: Some(label.to_owned()),
            top: None,
            bottom: None,
            aliases: vec![],
            description: None,
            template: None,
        }
    }

    fn fixture_codes() -> HashMap<u16, KeyCode> {
        let entries = [
            kc(0x0001, "KC_TRANSPARENT", "internal", "Transparent"),
            kc(0x0004, "KC_A", "basic", "A"),
            kc(0x0005, "KC_B", "basic", "B"),
            kc(0x00E1, "KC_LSFT", "modifiers", "L-Shift"),
            kc(0x0078, "BL_ON", "backlight", "BL On"),
            kc(0x0080, "RGB_TOG", "rgb", "RGB Tog"),
            kc(0x0090, "QK_NO_HOME", "media", "Media Home"),
        ];
        entries.into_iter().map(|c| (c.code, c)).collect()
    }

    fn parse_display(raw: &str) -> KeycodeDisplay {
        deser_hjson::from_str(raw).expect("failed to parse display fixture")
    }

    #[test]
    fn applies_label_and_description_overrides() {
        let mut codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [],
                "keycodes": {
                    "KC_A": { "label": "Aa", "description": "Sends a/A" }
                }
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        apply_overrides(&mut codes, &display, &n2c);
        let a = codes.get(&0x0004).unwrap();
        assert_eq!(a.label, Some("Aa".to_owned()));
        assert_eq!(a.description, Some("Sends a/A".to_owned()));
    }

    #[test]
    fn hidden_overrides_collected_but_keycode_unchanged() {
        let mut codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [],
                "keycodes": {
                    "KC_TRANSPARENT": { "hidden": true }
                }
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let hidden = apply_overrides(&mut codes, &display, &n2c);
        assert!(hidden.contains(&0x0001));
        let trans = codes.get(&0x0001).unwrap();
        assert_eq!(trans.label, Some("Transparent".to_owned()));
    }

    #[test]
    fn unknown_override_name_is_skipped() {
        let mut codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [],
                "keycodes": {
                    "KC_DOES_NOT_EXIST": { "label": "X" }
                }
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let hidden = apply_overrides(&mut codes, &display, &n2c);
        assert!(hidden.is_empty());
    }

    #[test]
    fn remapped_view_respects_declared_order_and_subgroups() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "basic", "label": "Basic",
                        "subgroups": [
                            { "id": "ansi", "label": "Standard", "render_mode": "ansi", "from_group": "basic" },
                            { "id": "modifiers", "label": "Modifiers", "from_group": "modifiers" }
                        ]
                    },
                    {
                        "id": "lighting", "label": "Lighting",
                        "subgroups": [
                            { "id": "backlight", "label": "Backlight", "from_group": "backlight" },
                            { "id": "rgb", "label": "RGB", "from_group": "rgb" }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_remapped_view(&display, &codes, &n2c, &HashSet::new());
        let ids: Vec<&str> = view.tabs.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(&ids[..2], &["basic", "lighting"]);

        let basic = &view.tabs[0];
        assert!(!basic.is_fallback);
        let sg_ids: Vec<&str> = basic.subgroups.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(sg_ids, vec!["ansi", "modifiers"]);
        assert_eq!(basic.subgroups[0].render_mode.as_deref(), Some("ansi"));
        let ansi_codes: Vec<u16> = basic.subgroups[0].codes.iter().map(|c| c.code).collect();
        assert_eq!(ansi_codes, vec![0x0004, 0x0005]);
        let mods_codes: Vec<u16> = basic.subgroups[1].codes.iter().map(|c| c.code).collect();
        assert_eq!(mods_codes, vec![0x00E1]);
    }

    #[test]
    fn explicit_keys_preserve_order_and_dedupe() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "letters", "label": "Letters",
                        "subgroups": [
                            { "id": "pick", "keys": ["KC_B", "KC_A", "KC_A"] }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_remapped_view(&display, &codes, &n2c, &HashSet::new());
        let pick_codes: Vec<u16> = view.tabs[0].subgroups[0].codes.iter().map(|c| c.code).collect();
        assert_eq!(pick_codes, vec![0x0005, 0x0004]);
    }

    #[test]
    fn keys_claim_wins_over_later_from_group() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "basic", "label": "Basic",
                        "subgroups": [
                            { "id": "letters_first", "keys": ["KC_A"] },
                            { "id": "rest", "from_group": "basic" }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_remapped_view(&display, &codes, &n2c, &HashSet::new());
        let first_codes: Vec<u16> = view.tabs[0].subgroups[0].codes.iter().map(|c| c.code).collect();
        let rest_codes: Vec<u16> = view.tabs[0].subgroups[1].codes.iter().map(|c| c.code).collect();
        assert_eq!(first_codes, vec![0x0004]);
        assert_eq!(rest_codes, vec![0x0005]);
    }

    #[test]
    fn from_groups_unions_multiple_groups() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "basic", "label": "Basic",
                        "subgroups": [
                            { "id": "ansi", "from_groups": ["basic", "modifiers"] }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_remapped_view(&display, &codes, &n2c, &HashSet::new());
        let ansi_codes: Vec<u16> = view.tabs[0].subgroups[0].codes.iter().map(|c| c.code).collect();
        assert_eq!(ansi_codes, vec![0x0004, 0x0005, 0x00E1]);
        // No standalone modifiers fallback tab should appear since the modifier got claimed.
        assert!(view.tabs.iter().all(|t| t.id != "modifiers"));
    }

    #[test]
    fn keys_claim_wins_over_earlier_from_group() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "basic", "label": "Basic",
                        "subgroups": [
                            { "id": "ansi", "from_group": "basic" },
                            { "id": "picked_letter", "keys": ["KC_A"] }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_remapped_view(&display, &codes, &n2c, &HashSet::new());
        let ansi_codes: Vec<u16> = view.tabs[0].subgroups[0].codes.iter().map(|c| c.code).collect();
        let picked_codes: Vec<u16> = view.tabs[0].subgroups[1].codes.iter().map(|c| c.code).collect();
        assert_eq!(ansi_codes, vec![0x0005], "explicit subgroup must steal KC_A from earlier from_group");
        assert_eq!(picked_codes, vec![0x0004]);
    }

    #[test]
    fn hidden_keys_skipped_in_explicit_subgroups_and_emitted_in_hidden_tab() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "basic", "label": "Basic",
                        "subgroups": [
                            { "id": "letters", "keys": ["KC_TRANSPARENT", "KC_A"] }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let mut hidden = HashSet::new();
        hidden.insert(0x0001);
        let view = build_remapped_view(&display, &codes, &n2c, &hidden);
        let basic_codes: Vec<u16> = view.tabs[0].subgroups[0].codes.iter().map(|c| c.code).collect();
        assert_eq!(basic_codes, vec![0x0004]);
        let hidden_tab = view.tabs.iter().find(|t| t.id == "hidden").expect("hidden tab");
        let hidden_codes: Vec<u16> = hidden_tab.subgroups[0].codes.iter().map(|c| c.code).collect();
        assert_eq!(hidden_codes, vec![0x0001]);
    }

    #[test]
    fn fallback_tabs_for_unmapped_groups() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "basic", "label": "Basic",
                        "subgroups": [
                            { "id": "letters", "from_group": "basic" }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_remapped_view(&display, &codes, &n2c, &HashSet::new());
        let fallback_ids: Vec<&str> = view
            .tabs
            .iter()
            .filter(|t| t.is_fallback)
            .map(|t| t.id.as_str())
            .collect();
        assert_eq!(
            fallback_ids,
            vec!["backlight", "internal", "media", "modifiers", "rgb"]
        );
    }

    #[test]
    fn unknown_subgroup_keys_silently_skipped() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "basic", "label": "Basic",
                        "subgroups": [
                            { "id": "picky", "keys": ["KC_DOESNT_EXIST", "KC_A"] }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_remapped_view(&display, &codes, &n2c, &HashSet::new());
        let picky_codes: Vec<u16> = view.tabs[0].subgroups[0].codes.iter().map(|c| c.code).collect();
        assert_eq!(picky_codes, vec![0x0004]);
    }

    #[test]
    fn template_subgroups_pass_through_without_codes() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "layer", "label": "Layer", "color": "blue",
                        "subgroups": [
                            { "id": "mo", "label": "Momentary (MO)", "template": { "kind": "MO" } },
                            { "id": "mt", "label": "Mod-Tap", "template": { "kind": "MT", "mods": ["LSFT", "MEH"] } }
                        ]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_remapped_view(&display, &codes, &n2c, &HashSet::new());
        let tab = view.tabs.iter().find(|t| t.id == "layer").expect("layer tab");
        assert_eq!(tab.color.as_deref(), Some("blue"));
        assert_eq!(tab.subgroups.len(), 2);
        let mo = &tab.subgroups[0];
        assert!(mo.codes.is_empty());
        assert!(matches!(mo.template, Some(SubgroupTemplate::Mo)));
        let mt = &tab.subgroups[1];
        match &mt.template {
            Some(SubgroupTemplate::Mt { mods }) => {
                assert_eq!(mods, &vec!["LSFT".to_owned(), "MEH".to_owned()]);
            }
            other => panic!("expected MT template, got {other:?}"),
        }
    }

    #[test]
    fn raw_view_path_when_version_mismatches() {
        let codes = fixture_codes();
        let display = parse_display(
            r#"{
                "target_keycode_version": "0.0.8",
                "tabs": [
                    {
                        "id": "basic", "label": "Basic",
                        "subgroups": [{ "id": "letters", "from_group": "basic" }]
                    }
                ]
            }"#,
        );
        let n2c = build_name_to_code(&codes);
        let view = build_view_for_version(Some(&display), "0.0.5", &codes, &n2c, &HashSet::new());
        assert!(view.tabs.iter().all(|t| t.is_fallback));
        assert!(view.tabs.iter().all(|t| t.subgroups.iter().all(|s| s.is_fallback)));
        assert!(!view.tabs.iter().any(|t| t.id == "hidden"));
    }
}
