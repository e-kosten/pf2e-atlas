//! Versioned subprocess messages and exact record links shared by Atlas and its desktop host.
#![deny(unsafe_code)]

pub use atlas_domain::SearchProfileId;
use atlas_domain::{DefinitionKey, RecordKey, TraitKey};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const VERSION: u32 = 1;
pub const SETUP_EVENT_VERSION: u32 = 1;
pub const REFERENCE_REFRESH_VERSION: u32 = 1;
pub const BUNDLE_ID: &str = "com.ekosten.pf2e-atlas";
const KEY_ENCODING: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'%')
    .add(b':')
    .add(b'/')
    .add(b'?')
    .add(b'#')
    .add(b'"')
    .add(b'\'')
    .add(b'<')
    .add(b'>')
    .add(b'\\');

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStatus {
    NeedsSetup,
    Browsable,
    Ready,
    UpgradeRequired,
    Incompatible,
    Busy,
    RecoveryRequired,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceProbe {
    pub version: u32,
    pub workspace: PathBuf,
    pub status: WorkspaceStatus,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<SetupStage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupStage {
    FirstRun,
    Resume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupPublication {
    pub setup_event_version: u32,
    pub workspace: PathBuf,
    pub generation: String,
    pub semantic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSelection {
    pub profiles: Vec<SearchProfileId>,
    pub preferred: Option<SearchProfileId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchModelStatus {
    pub id: SearchProfileId,
    pub label: String,
    pub installed: bool,
    pub usable: bool,
}

/// Private, read-only metadata for the desktop setup drawer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceStatus {
    pub version: u32,
    pub workspace: PathBuf,
    pub generation: String,
    pub revision: Option<String>,
    pub target: String,
    pub target_version: String,
    pub managed: bool,
    pub offer: Option<String>,
    pub model_offer: Option<String>,
    pub can_change_models: bool,
    pub preferred: Option<SearchProfileId>,
    pub models: Vec<SearchModelStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceRefreshResult {
    pub reference_refresh_version: u32,
    #[serde(flatten)]
    pub outcome: ReferenceRefreshOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ReferenceRefreshOutcome {
    Ready {
        generation: String,
        updated: bool,
        model_repaired: bool,
    },
    SeedUnavailable {
        retry_offer: String,
    },
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherReady {
    pub version: u32,
    pub workspace: PathBuf,
    pub endpoint: String,
    pub instance: String,
    pub api_contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityLink {
    Record(RecordKey),
    List(String),
    Encounter(String),
    Trait(TraitKey),
    Definition(DefinitionKey),
    CreatureFamily(String),
    EquipmentFamily(String),
}

impl EntityLink {
    pub fn parse(link: &str) -> Result<Self, &'static str> {
        let (kind, key) = link
            .strip_prefix("pf2e-atlas://v1/")
            .and_then(|target| target.split_once('/'))
            .ok_or("Expected an Atlas v1 link.")?;
        if kind == "record" {
            return parse_encoded_key(key).map(Self::Record);
        }
        Self::from_key(kind, &decode_segment(key)?)
    }

    pub fn from_key(kind: &str, key: &str) -> Result<Self, &'static str> {
        let invalid = "The Atlas link has an invalid destination.";
        match kind {
            "record" => RecordKey::parse(key).map(Self::Record).map_err(|_| invalid),
            "list" if stable_key(key, "list_") => Ok(Self::List(key.into())),
            "encounter" if stable_key(key, "encounter_") => Ok(Self::Encounter(key.into())),
            "trait" => key.parse().map(Self::Trait).map_err(|_| invalid),
            "definition" => {
                let definition: DefinitionKey = key.parse().map_err(|_| invalid)?;
                Ok(match definition.trait_key() {
                    Some(trait_key) => Self::Trait(trait_key),
                    None => Self::Definition(definition),
                })
            }
            "creature-family" if family_id(key) => Ok(Self::CreatureFamily(key.into())),
            "equipment-family" if family_id(key) => Ok(Self::EquipmentFamily(key.into())),
            _ => Err(invalid),
        }
    }

    pub fn uri(&self) -> String {
        let (kind, _, key) = self.parts();
        format!(
            "pf2e-atlas://v1/{kind}/{}",
            utf8_percent_encode(&key, KEY_ENCODING)
        )
    }

    pub fn path(&self) -> String {
        let (_, route, key) = self.parts();
        format!("{route}/{}", utf8_percent_encode(&key, KEY_ENCODING))
    }

    pub fn from_path(path: &str) -> Option<Self> {
        let (route, key) = path.trim_end_matches('/').rsplit_once('/')?;
        let kind = match route {
            "/records" | "/search/records" => "record",
            "/lists" => "list",
            "/encounters" => "encounter",
            "/traits" => "trait",
            "/definitions" => "definition",
            "/families/creatures" => "creature-family",
            "/families/equipment" => "equipment-family",
            _ => return None,
        };
        Self::from_key(kind, &decode_segment(key).ok()?).ok()
    }

    fn parts(&self) -> (&'static str, &'static str, String) {
        match self {
            Self::Record(key) => ("record", "/search/records", key.to_string()),
            Self::List(key) => ("list", "/lists", key.clone()),
            Self::Encounter(key) => ("encounter", "/encounters", key.clone()),
            Self::Trait(key) => ("trait", "/traits", key.to_string()),
            Self::Definition(key) => ("definition", "/definitions", key.to_string()),
            Self::CreatureFamily(key) => ("creature-family", "/families/creatures", key.clone()),
            Self::EquipmentFamily(key) => ("equipment-family", "/families/equipment", key.clone()),
        }
    }
}

fn stable_key(key: &str, prefix: &str) -> bool {
    key.strip_prefix(prefix).is_some_and(|id| {
        id.len() == 16
            && id
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn family_id(id: &str) -> bool {
    id.len() <= 128
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn decode_segment(key: &str) -> Result<std::borrow::Cow<'_, str>, &'static str> {
    if key.contains(['/', '?', '#', '\\'])
        || key.chars().any(char::is_control)
        || key.as_bytes().iter().enumerate().any(|(index, byte)| {
            *byte == b'%'
                && !key
                    .as_bytes()
                    .get(index + 1..index + 3)
                    .is_some_and(|escape| escape.iter().all(u8::is_ascii_hexdigit))
        })
    {
        return Err("The Atlas record link has an invalid destination.");
    }
    let decoded = percent_decode_str(key)
        .decode_utf8()
        .map_err(|_| "The Atlas record key is invalid.")?;
    if decoded.chars().any(char::is_control) {
        return Err("The Atlas record key is invalid.");
    }
    Ok(decoded)
}

fn parse_encoded_key(key: &str) -> Result<RecordKey, &'static str> {
    RecordKey::parse(&decode_segment(key)?).map_err(|_| "The Atlas record key is invalid.")
}

pub fn parse_record_path(path: &str) -> Option<RecordKey> {
    let segment = path
        .strip_prefix("/search/records/")
        .or_else(|| path.strip_prefix("/records/"))
        .or_else(|| {
            let (slug, tail) = path.strip_prefix("/lists/")?.split_once('/')?;
            (!slug.is_empty()).then_some(tail)?.strip_prefix("records/")
        })?;
    parse_encoded_key(segment).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_key_round_trip() {
        let target = EntityLink::from_key("record", "spells:100%2Fé/?#\\").unwrap();
        assert_eq!(EntityLink::parse(&target.uri()).unwrap(), target);
    }

    #[test]
    fn reader_routes_match_the_exact_key() {
        let key = RecordKey::parse("spells:one").unwrap();
        assert_eq!(
            parse_record_path(&EntityLink::Record(key.clone()).path()),
            Some(key.clone())
        );
        assert_eq!(parse_record_path("/records/spells:one"), Some(key.clone()));
        assert_eq!(
            parse_record_path("/lists/prepared/records/spells%3Aone"),
            Some(key)
        );
        assert_eq!(parse_record_path("/search/records/spells:one/extra"), None);
    }

    #[test]
    fn entity_links_round_trip_to_their_detail_routes() {
        for (kind, key, path) in [
            (
                "list",
                "list_1234567890abcdef",
                "/lists/list_1234567890abcdef",
            ),
            (
                "encounter",
                "encounter_1234567890abcdef",
                "/encounters/encounter_1234567890abcdef",
            ),
            ("trait", "trait:agile", "/traits/trait%3Aagile"),
            (
                "definition",
                "rule:pf2e:foo",
                "/definitions/rule%3Apf2e%3Afoo",
            ),
            ("creature-family", "goblin", "/families/creatures/goblin"),
            (
                "equipment-family",
                "jolt-coil",
                "/families/equipment/jolt-coil",
            ),
        ] {
            let target = EntityLink::from_key(kind, key).unwrap();
            assert_eq!(EntityLink::parse(&target.uri()).unwrap(), target);
            assert_eq!(target.path(), path);
            assert_eq!(EntityLink::from_path(path), Some(target.clone()));
            assert_eq!(
                EntityLink::from_path(&format!("{}/", path.replace("%3A", ":"))),
                Some(target)
            );
        }
    }

    #[test]
    fn refuses_other_destinations_and_invalid_encoding() {
        for link in [
            "pf2e-atlas://v1/list/prepared",
            "pf2e-atlas://v1/creature-family/%2e%2e",
            "pf2e-atlas://v1/equipment-family/a%2fb",
            "pf2e-atlas://v1/other/key",
            "pf2e-atlas://v2/record/spells%3Aone",
            "pf2e-atlas://v1/record/spells%3Aone?x",
            "pf2e-atlas://v1/record/spells%3Aone/extra",
            "pf2e-atlas://v1/record/spells%3A%zz",
            "pf2e-atlas://v1/record/spells%3A%00",
        ] {
            assert!(EntityLink::parse(link).is_err(), "{link}");
        }
    }
}
