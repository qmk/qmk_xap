use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::Path,
};

use log::error;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, skip_serializing_none, NoneAsEmptyString};
use specta::Type;

use crate::error::XapResult;

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

#[cfg(test)]
mod test {
    use similar_asserts::assert_eq;

    use super::*;

    #[test]
    pub fn deserialize() {
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
}
