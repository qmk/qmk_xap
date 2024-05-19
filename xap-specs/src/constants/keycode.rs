use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::Path,
};

use convert_case::{Case, Casing};
use log::error;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, skip_serializing_none, NoneAsEmptyString};
use specta::Type;

use crate::{error::XapResult, KeyPosition};

#[serde_as]
#[skip_serializing_none]
#[derive(Deserialize, Clone, Serialize, Default, Debug, PartialEq, Eq, Type)]
pub struct XapKeyCode {
    #[serde(default)]
    pub code: u16,
    pub key: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub label: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

impl XapKeyCode {
    pub fn new_custom(code: u16) -> Self {
        Self {
            code,
            key: format!("USER-CUSTOM-{code}"),
            group: Some("USER-CUSTOM".to_owned()),
            label: Some(format!("{code}")),
            aliases: vec![],
        }
    }
}

#[derive(Deserialize, Debug)]
struct KeyCodes {
    #[serde(deserialize_with = "xap_keycode_from_hex_map")]
    keycodes: HashMap<u16, XapKeyCode>,
}

pub(crate) fn read_xap_keycodes(path: impl AsRef<Path>) -> XapResult<HashMap<u16, XapKeyCode>> {
    let mut all = HashMap::new();

    for entry in fs::read_dir(path)?.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir()
            || path
                .file_name()
                .is_some_and(|filename| !filename.to_string_lossy().starts_with("keycodes"))
        {
            continue;
        }

        let raw_hjson = read_to_string(&path)?;

        match deser_hjson::from_str::<KeyCodes>(&raw_hjson) {
            Ok(codes) => {
                all.extend(codes.keycodes);
            }
            Err(err) => {
                error!("failed to deserialize keycodes from file {path:?} with error: {err}",);
            }
        }
    }

    Ok(all)
}

fn xap_keycode_from_hex_map<'de, D>(deserializer: D) -> Result<HashMap<u16, XapKeyCode>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, XapKeyCode> = Deserialize::deserialize(deserializer)?;

    map.into_iter()
        .try_fold(HashMap::new(), |mut result, (raw_code, mut keycode)| {
            let code = u16::from_str_radix(raw_code.trim_start_matches("0x"), 16).ok()?;
            keycode.code = code;
            result.insert(code, keycode);
            Some(result)
        })
        .ok_or(D::Error::custom("failed to parse keycode table"))
}

#[derive(Debug, Default, Clone, Serialize, Type)]
pub struct XapKeyCodeConfig {
    pub code: XapKeyCode,
    pub position: KeyPosition,
}

#[derive(Clone, Deserialize, Serialize, Debug, Type)]
pub struct LightingEffects {
    pub groups: Option<HashMap<String, LightingGroup>>,
    #[serde(deserialize_with = "effect_from_hex_map")]
    pub effects: HashMap<u16, LightingEffect>,
}

impl LightingEffects {
    pub fn get_effect_map(&self, effects: u64) -> Vec<LightingEffect> {
        self.effects
            .iter()
            .filter(|(code, _)| ((effects >> **code) & 1 == 1))
            .map(|(_, effect)| effect)
            .cloned()
            .collect()
    }
}

fn effect_from_hex_map<'de, D>(deserializer: D) -> Result<HashMap<u16, LightingEffect>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, LightingEffect> = Deserialize::deserialize(deserializer)?;

    map.into_iter()
        .try_fold(HashMap::new(), |mut result, (raw_code, mut effect)| {
            let code = u16::from_str_radix(raw_code.trim_start_matches("0x"), 16).ok()?;
            effect.code = code;
            // TODO: add label property to effect
            effect.label = effect.key.to_case(Case::Title);
            result.insert(code, effect);
            Some(result)
        })
        .ok_or(D::Error::custom("failed to parse lighting effect table"))
}

#[derive(Clone, Deserialize, Serialize, Type, PartialEq, Debug)]
pub struct LightingEffect {
    #[serde(default)]
    pub code: u16,
    pub key: String,
    pub group: Option<String>,
    #[serde(default)]
    pub label: String,
}

#[derive(Clone, Deserialize, Serialize, Type, Debug)]
pub struct LightingGroup {
    pub define: String,
}

pub(crate) fn read_xap_lighting_effects(
    path: impl AsRef<Path>,
    effect_type: &str,
) -> XapResult<LightingEffects> {
    for entry in fs::read_dir(path.as_ref())?.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir()
            || path
                .file_name()
                .is_some_and(|filename| !filename.to_string_lossy().starts_with(effect_type))
        {
            continue;
        }

        let raw_hjson = read_to_string(&path)?;
        return deser_hjson::from_str::<LightingEffects>(&raw_hjson).map_err(Into::into);
    }

    Ok(LightingEffects {
        effects: HashMap::new(),
        groups: None,
    })
}

#[cfg(test)]
mod test {
    use similar_asserts::assert_eq;

    use super::*;

    #[test]
    pub fn deserialize_keycodes() {
        let input = r#"{
            "keycodes": {
                "0x0000": {
                    "group": "internal",
                    "key": "KC_NO",
                    "label": "",
                    "aliases": [
                        "XXXXXXX"
                    ]
                },
                "0x0001": {
                    "group": "internal",
                    "key": "KC_TRANSPARENT",
                    "label": "",
                    "aliases": [
                        "_______",
                        "KC_TRNS"
                    ]
                },
                "0x0004": {
                    "group": "basic",
                    "key": "KC_A",
                    "label": "A"
                },
                "0x0005": {
                    "group": "basic",
                    "key": "KC_B",
                    "label": "B"
                }
            }
        }"#;

        let codes: KeyCodes = deser_hjson::from_str(input).expect("deserialization failed");

        assert_eq!(codes.keycodes.len(), 4);

        assert_eq!(
            codes.keycodes[&0],
            XapKeyCode {
                code: 0,
                group: Some("internal".to_owned()),
                key: "KC_NO".to_owned(),
                label: None,
                aliases: vec!["XXXXXXX".to_owned()]
            }
        );

        assert_eq!(
            codes.keycodes[&1],
            XapKeyCode {
                code: 1,
                group: Some("internal".to_owned()),
                key: "KC_TRANSPARENT".to_owned(),
                label: None,
                aliases: vec!["_______".to_owned(), "KC_TRNS".to_owned()]
            }
        );

        assert_eq!(
            codes.keycodes[&4],
            XapKeyCode {
                code: 4,
                group: Some("basic".to_owned()),
                key: "KC_A".to_owned(),
                label: Some("A".to_owned()),
                aliases: vec![]
            }
        );

        assert_eq!(
            codes.keycodes[&5],
            XapKeyCode {
                code: 5,
                group: Some("basic".to_owned()),
                key: "KC_B".to_owned(),
                label: Some("B".to_owned()),
                aliases: vec![]
            }
        );
    }

    #[test]
    fn deserialize_lighting_effects() {
        // taken from rgb_matrix_0.0.1.json
        let input = r#"
            {
                "groups": {
                    "framebuffer": {
                        "define": "RGB_MATRIX_FRAMEBUFFER_EFFECTS"
                    },
                    "reactive": {
                        "define": "RGB_MATRIX_KEYREACTIVE_ENABLED"
                    }
                },
                "effects": {
                    "0x00": {
                        "key": "SOLID_COLOR"
                    },
                    "0x1E": {
                        "key": "TYPING_HEATMAP",
                        "group": "framebuffer"
                    },
                    "0x20": {
                        "key": "SOLID_REACTIVE_SIMPLE",
                        "group": "reactive"
                    },
                }
            }
        "#;

        let effects: LightingEffects =
            deser_hjson::from_str(input).expect("deserialization failed");

        assert_eq!(effects.effects.len(), 3);
        assert_eq!(
            effects.effects[&0],
            LightingEffect {
                code: 0,
                key: "SOLID_COLOR".to_owned(),
                group: None,
            }
        );
        assert_eq!(
            effects.effects[&0x1e],
            LightingEffect {
                code: 0x1e,
                key: "TYPING_HEATMAP".to_owned(),
                group: Some("framebuffer".to_owned()),
            }
        );
        assert_eq!(
            effects.effects[&0x20],
            LightingEffect {
                code: 0x20,
                key: "SOLID_REACTIVE_SIMPLE".to_owned(),
                group: Some("reactive".to_owned()),
            }
        );
    }
}
