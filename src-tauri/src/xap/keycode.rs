use std::collections::HashMap;

use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};

#[serde_as]
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
struct KeyCode {
    #[serde(default)]
    code: u16,
    key: String,
    group: String,
    #[serde_as(as = "NoneAsEmptyString")]
    label: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct KeyCodes {
    #[serde(deserialize_with = "from_hex_keycode")]
    keycodes: HashMap<u16, KeyCode>,
}

fn from_hex_keycode<'de, D>(deserializer: D) -> Result<HashMap<u16, KeyCode>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<&str, KeyCode> = Deserialize::deserialize(deserializer)?;
    let mut result: HashMap<u16, KeyCode> = HashMap::with_capacity(map.len());

    for code_keycode in map.into_iter().map(|(raw_code, mut keycode)| {
        let code =
            u16::from_str_radix(raw_code.trim_start_matches("0x"), 16).map_err(D::Error::custom)?;
        keycode.code = code;
        Ok((code, keycode))
    }) {
        let (code, keycode) = code_keycode?;
        result.insert(code, keycode);
    }

    Ok(result)
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

        let codes: KeyCodes = serde_json::from_str(input).expect("deserialization failed");

        assert_eq!(codes.keycodes.len(), 4);

        assert_eq!(
            codes.keycodes.get(&0),
            Some(&KeyCode {
                code: 0,
                group: "internal".to_owned(),
                key: "KC_NO".to_owned(),
                label: None,
                aliases: vec!["XXXXXXX".to_owned()]
            })
        );

        assert_eq!(
            codes.keycodes.get(&1),
            Some(&KeyCode {
                code: 1,
                group: "internal".to_owned(),
                key: "KC_TRANSPARENT".to_owned(),
                label: None,
                aliases: vec!["_______".to_owned(), "KC_TRNS".to_owned()]
            })
        );

        assert_eq!(
            codes.keycodes.get(&4),
            Some(&KeyCode {
                code: 4,
                group: "basic".to_owned(),
                key: "KC_A".to_owned(),
                label: Some("A".to_owned()),
                aliases: vec![]
            })
        );

        assert_eq!(
            codes.keycodes.get(&5),
            Some(&KeyCode {
                code: 5,
                group: "basic".to_owned(),
                key: "KC_B".to_owned(),
                label: Some("B".to_owned()),
                aliases: vec![]
            })
        );
    }
}
