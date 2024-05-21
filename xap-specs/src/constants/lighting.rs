use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::Path,
};

use convert_case::{Case, Casing};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use specta::Type;

use crate::error::XapResult;

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
    fn deserialize() {
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
                label: "Solid Color".to_owned()
            }
        );
        assert_eq!(
            effects.effects[&0x1e],
            LightingEffect {
                code: 0x1e,
                key: "TYPING_HEATMAP".to_owned(),
                group: Some("framebuffer".to_owned()),
                label: "Typing Heatmap".to_owned()
            }
        );
        assert_eq!(
            effects.effects[&0x20],
            LightingEffect {
                code: 0x20,
                key: "SOLID_REACTIVE_SIMPLE".to_owned(),
                group: Some("reactive".to_owned()),
                label: "Solid Reactive Simple".to_owned()
            }
        );
    }
}
