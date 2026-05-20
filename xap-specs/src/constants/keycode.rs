use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, skip_serializing_none, NoneAsEmptyString};
use specta::Type;

const GENERATED_KEYCODES_PREFIX: &str = "keycodes_";
const GENERATED_KEYCODES_SUFFIX: &str = ".generated.hjson";

#[serde_as]
#[skip_serializing_none]
#[derive(Deserialize, Clone, Serialize, Default, Debug, PartialEq, Eq, Type)]
pub struct KeyCode {
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

#[derive(Debug, Serialize, Clone, Type)]
pub struct XapKeyCodeCategory {
    pub name: String,
    pub codes: Vec<KeyCode>,
}

impl KeyCode {
    pub fn new_custom(code: u16) -> Self {
        let keycode = format!("0x{code:04X}");

        Self {
            code,
            key: keycode.clone(),
            group: Some("USER-CUSTOM".to_owned()),
            label: Some(keycode),
            aliases: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct XapKeyCodeCatalog {
    pub latest: String,
    pub versions: Vec<String>,
    versions_by_name: HashMap<String, XapKeyCodeVersion>,
}

impl XapKeyCodeCatalog {
    pub fn latest_keycodes(&self) -> Vec<XapKeyCodeCategory> {
        self.keycodes_for_version(None)
    }

    pub fn keycodes_for_version(&self, version: Option<&str>) -> Vec<XapKeyCodeCategory> {
        self.resolve_version(version)
            .and_then(|version| self.versions_by_name.get(version))
            .or_else(|| self.versions_by_name.get(&self.latest))
            .map(|version| version.categories.clone())
            .unwrap_or_default()
    }

    pub fn get_keycode(&self, version: Option<&str>, code: u16) -> KeyCode {
        self.resolve_version(version)
            .and_then(|version| self.versions_by_name.get(version))
            .or_else(|| self.versions_by_name.get(&self.latest))
            .and_then(|version| version.lookup.get(&code))
            .cloned()
            .unwrap_or_else(|| KeyCode::new_custom(code))
    }

    fn resolve_version(&self, requested: Option<&str>) -> Option<&str> {
        let Some(requested) = requested else {
            return Some(&self.latest);
        };

        if let Some((version, _)) = self.versions_by_name.get_key_value(requested) {
            return Some(version.as_str());
        }

        let requested = KeyCodeVersion::parse(requested)?;

        self.versions
            .iter()
            .rev()
            .find(|version| {
                KeyCodeVersion::parse(version)
                    .map(|version| version <= requested)
                    .unwrap_or(false)
            })
            .map(String::as_str)
    }
}

#[derive(Clone, Debug)]
struct XapKeyCodeVersion {
    categories: Vec<XapKeyCodeCategory>,
    lookup: HashMap<u16, KeyCode>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct KeyCodeVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl KeyCodeVersion {
    fn parse(raw_version: &str) -> Option<Self> {
        let mut digits = raw_version.split('.');

        let version = Self {
            major: digits.next()?.parse().ok()?,
            minor: digits.next()?.parse().ok()?,
            patch: digits.next()?.parse().ok()?,
        };

        digits.next().is_none().then_some(version)
    }
}

#[derive(Deserialize, Debug)]
struct KeyCodes {
    version: String,
    #[serde(deserialize_with = "keycodes_from_hex_map")]
    keycodes: HashMap<u16, KeyCode>,
}

pub(crate) fn read_xap_keycode_catalog(path: impl AsRef<Path>) -> Result<XapKeyCodeCatalog> {
    let mut version_files = read_generated_keycode_files(path)?;

    if version_files.is_empty() {
        bail!("no generated keycode files found");
    }

    version_files.sort_by(|lhs, rhs| compare_keycode_versions(&lhs.0, &rhs.0));

    let mut versions = Vec::with_capacity(version_files.len());
    let mut versions_by_name = HashMap::new();

    for (_, file) in version_files {
        let raw_keycodes = read_to_string(file)?;
        let keycodes: KeyCodes = deser_hjson::from_str(&raw_keycodes)?;
        let version = keycodes.version.clone();

        versions.push(version.clone());
        versions_by_name.insert(version, keycodes.into());
    }

    let latest = versions
        .last()
        .expect("version_files was checked for emptiness")
        .clone();

    Ok(XapKeyCodeCatalog {
        latest,
        versions,
        versions_by_name,
    })
}

fn read_generated_keycode_files(path: impl AsRef<Path>) -> Result<Vec<(String, PathBuf)>> {
    let mut version_files = Vec::new();

    for entry in read_dir(path)? {
        let path = entry?.path();
        let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
            continue;
        };

        let Some(version) = file_name
            .strip_prefix(GENERATED_KEYCODES_PREFIX)
            .and_then(|file_name| file_name.strip_suffix(GENERATED_KEYCODES_SUFFIX))
        else {
            continue;
        };

        if KeyCodeVersion::parse(version).is_none() {
            continue;
        }

        version_files.push((version.to_owned(), path));
    }

    Ok(version_files)
}

fn compare_keycode_versions(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    match (KeyCodeVersion::parse(lhs), KeyCodeVersion::parse(rhs)) {
        (Some(lhs), Some(rhs)) => lhs.cmp(&rhs),
        _ => lhs.cmp(rhs),
    }
}

impl From<KeyCodes> for XapKeyCodeVersion {
    fn from(keycodes: KeyCodes) -> Self {
        let lookup = keycodes.keycodes;
        let mut categories = lookup.iter().fold(
            HashMap::new(),
            |mut category: HashMap<String, Vec<KeyCode>>, (_, keycode)| {
                category
                    .entry(keycode.group.clone().unwrap_or("other".to_owned()))
                    .or_default()
                    .push(keycode.clone());

                category
            },
        );

        let mut categories = categories
            .drain()
            .map(|(name, mut codes)| {
                codes.sort_by_key(|code| code.code);
                XapKeyCodeCategory { name, codes }
            })
            .collect::<Vec<_>>();

        categories.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

        Self { categories, lookup }
    }
}

fn keycodes_from_hex_map<'de, D>(deserializer: D) -> Result<HashMap<u16, KeyCode>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, KeyCode> = Deserialize::deserialize(deserializer)?;

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
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use similar_asserts::assert_eq;

    use super::*;

    #[test]
    pub fn deserialize() {
        let input = r#"{
            "version": "0.0.1",
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
            KeyCode {
                code: 0,
                group: Some("internal".to_owned()),
                key: "KC_NO".to_owned(),
                label: None,
                aliases: vec!["XXXXXXX".to_owned()]
            }
        );

        assert_eq!(
            codes.keycodes[&1],
            KeyCode {
                code: 1,
                group: Some("internal".to_owned()),
                key: "KC_TRANSPARENT".to_owned(),
                label: None,
                aliases: vec!["_______".to_owned(), "KC_TRNS".to_owned()]
            }
        );

        assert_eq!(
            codes.keycodes[&4],
            KeyCode {
                code: 4,
                group: Some("basic".to_owned()),
                key: "KC_A".to_owned(),
                label: Some("A".to_owned()),
                aliases: vec![]
            }
        );

        assert_eq!(
            codes.keycodes[&5],
            KeyCode {
                code: 5,
                group: Some("basic".to_owned()),
                key: "KC_B".to_owned(),
                label: Some("B".to_owned()),
                aliases: vec![]
            }
        );
    }

    #[test]
    pub fn read_generated_keycode_catalog_uses_latest_by_default() {
        let temp_dir = make_temp_dir();

        fs::write(
            temp_dir.join("keycodes_0.0.1.generated.hjson"),
            r#"{
                "version": "0.0.1",
                "ranges": {},
                "keycodes": {
                    "0x0004": {
                        "group": "basic",
                        "key": "KC_A_OLD",
                        "label": "A"
                    }
                }
            }"#,
        )
        .expect("failed to write old generated keycodes fixture");

        fs::write(
            temp_dir.join("keycodes_0.0.2.generated.hjson"),
            r#"{
                "version": "0.0.2",
                "ranges": {},
                "keycodes": {
                    "0x0005": {
                        "group": "basic",
                        "key": "KC_B",
                        "label": "B"
                    },
                    "0x0004": {
                        "group": "basic",
                        "key": "KC_A",
                        "label": "A"
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
                    "0x0010": {
                        "key": "KC_M"
                    }
                }
            }"#,
        )
        .expect("failed to write latest generated keycodes fixture");

        let catalog =
            read_xap_keycode_catalog(&temp_dir).expect("failed to read generated keycodes");
        let categories = catalog.latest_keycodes();

        assert_eq!(catalog.latest, "0.0.2");
        assert_eq!(catalog.versions, vec!["0.0.1", "0.0.2"]);

        assert_eq!(
            categories
                .iter()
                .map(|category| category.name.as_str())
                .collect::<Vec<_>>(),
            vec!["basic", "internal", "other"]
        );

        assert_eq!(
            categories[0]
                .codes
                .iter()
                .map(|keycode| keycode.key.as_str())
                .collect::<Vec<_>>(),
            vec!["KC_A", "KC_B"]
        );

        assert_eq!(categories[1].codes[0].label, None);
        assert_eq!(catalog.get_keycode(Some("0.0.1"), 0x0004).key, "KC_A_OLD");
        assert_eq!(catalog.get_keycode(Some("0.0.2"), 0x0004).key, "KC_A");
        assert_eq!(catalog.get_keycode(Some("0.0.99"), 0x0004).key, "KC_A");

        fs::remove_dir_all(temp_dir).expect("failed to remove temp dir");
    }

    #[test]
    pub fn custom_keycodes_use_hex_display_names() {
        assert_eq!(
            KeyCode::new_custom(0x000A),
            KeyCode {
                code: 0x000A,
                group: Some("USER-CUSTOM".to_owned()),
                key: "0x000A".to_owned(),
                label: Some("0x000A".to_owned()),
                aliases: vec![]
            }
        );
    }

    fn make_temp_dir() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before unix epoch")
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!(
            "qmk-xap-keycodes-test-{}-{unique}",
            std::process::id()
        ));

        fs::create_dir_all(&temp_dir).expect("failed to create temp dir");

        temp_dir
    }
}
