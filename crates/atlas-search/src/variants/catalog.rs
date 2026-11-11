//! Shared contract for the authored catalog and its bundled payload.

use crate::family_axes::{Axis, Coordinates};
use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) fn valid_family_id(id: &str) -> bool {
    id.len() <= 128
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Catalog {
    pub(super) families: BTreeMap<String, Family>,
    pub(super) exclusions: Vec<Exclusion>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Family {
    pub(super) name: String,
    pub(super) source_note: String,
    pub(super) item_type: String,
    pub(super) category: Option<String>,
    pub(super) group: Option<String>,
    pub(super) members: Vec<Member>,
    #[serde(default)]
    pub(super) axes: Vec<Axis>,
    #[serde(default)]
    pub(super) all_variants_by_default: bool,
    #[serde(default)]
    pub(super) general_records: Vec<GeneralRecord>,
    #[serde(default)]
    pub(super) order_by_level: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Member {
    pub(super) key: RecordKey,
    pub(super) name: String,
    pub(super) category: Option<String>,
    #[serde(default)]
    pub(super) attributes: Vec<String>,
    pub(super) fit: Option<Fit>,
    #[serde(default)]
    pub(super) coordinates: Coordinates,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GeneralRecord {
    pub(super) key: RecordKey,
    pub(super) name: String,
    pub(super) scope: Coordinates,
}

impl GeneralRecord {
    pub(super) fn covers(&self, member: &Member) -> bool {
        self.scope
            .iter()
            .all(|(axis, value)| member.coordinates.get(axis) == Some(value))
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Fit {
    pub(super) ammo_type: String,
    pub(super) weapons: Vec<Weapon>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Weapon {
    pub(super) key: RecordKey,
    pub(super) name: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Exclusion {
    pub(super) source_note: String,
    pub(super) members: Vec<RecordKey>,
    pub(super) others: Vec<RecordKey>,
}
