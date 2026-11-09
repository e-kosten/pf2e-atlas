//! Shared contract for the authored catalog and its bundled payload.

use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub(super) order_by_level: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Member {
    pub(super) key: RecordKey,
    pub(super) name: String,
    #[serde(default)]
    pub(super) attributes: Vec<String>,
    pub(super) fit: Option<Fit>,
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
    pub(super) members: Vec<RecordKey>,
    pub(super) others: Vec<RecordKey>,
}
