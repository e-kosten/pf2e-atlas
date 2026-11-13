//! Versioned subprocess messages and exact record links shared by Atlas and its desktop host.
#![deny(unsafe_code)]

use atlas_domain::RecordKey;
pub use atlas_domain::SearchProfileId;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const VERSION: u32 = 1;
pub const SETUP_EVENT_VERSION: u32 = 1;
pub const REFERENCE_REFRESH_VERSION: u32 = 1;
pub const BUNDLE_ID: &str = "com.ekosten.pf2e-atlas";
const LINK_PREFIX: &str = "pf2e-atlas://v1/record/";
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

pub fn parse_record_link(link: &str) -> Result<RecordKey, &'static str> {
    let key = link
        .strip_prefix(LINK_PREFIX)
        .ok_or("Expected an Atlas v1 record link.")?;
    parse_encoded_key(key)
}

fn parse_encoded_key(key: &str) -> Result<RecordKey, &'static str> {
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
    RecordKey::parse(&decoded).map_err(|_| "The Atlas record key is invalid.")
}

pub fn record_link(key: &RecordKey) -> String {
    format!("{LINK_PREFIX}{}", encoded_key(key))
}

pub fn record_path(key: &RecordKey) -> String {
    format!("/search/records/{}", encoded_key(key))
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

pub fn api_record_path(key: &RecordKey) -> String {
    format!("/api/records/{}", encoded_key(key))
}

fn encoded_key(key: &RecordKey) -> String {
    utf8_percent_encode(&key.to_string(), KEY_ENCODING).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_key_round_trip() {
        let key = RecordKey::parse("spells:100%2Fé").unwrap();
        assert_eq!(parse_record_link(&record_link(&key)).unwrap(), key);
    }

    #[test]
    fn reader_routes_match_the_exact_key() {
        let key = RecordKey::parse("spells:one").unwrap();
        assert_eq!(parse_record_path(&record_path(&key)), Some(key.clone()));
        assert_eq!(parse_record_path("/records/spells:one"), Some(key.clone()));
        assert_eq!(
            parse_record_path("/lists/prepared/records/spells%3Aone"),
            Some(key)
        );
        assert_eq!(parse_record_path("/search/records/spells:one/extra"), None);
    }

    #[test]
    fn refuses_other_destinations_and_invalid_encoding() {
        for link in [
            "pf2e-atlas://v2/record/spells%3Aone",
            "pf2e-atlas://v1/record/spells%3Aone?x",
            "pf2e-atlas://v1/record/spells%3Aone/extra",
            "pf2e-atlas://v1/record/spells%3A%zz",
            "pf2e-atlas://v1/record/spells%3A%00",
        ] {
            assert!(parse_record_link(link).is_err(), "{link}");
        }
    }
}
