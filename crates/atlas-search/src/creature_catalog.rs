//! Shared contract for the reviewed creature declarations and bundled catalog.

use crate::family_axes::{Axis, Coordinates};
use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) type CreatureCatalog = BTreeMap<String, CreatureFamily>;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreatureFamily {
    pub name: String,
    pub source_note: String,
    pub members: Vec<Member>,
    #[serde(default)]
    pub axes: Vec<Axis>,
    #[serde(default)]
    pub all_variants_by_default: bool,
    #[serde(default)]
    pub omit_axis_qualifiers: bool,
    #[serde(default)]
    pub overrides: Vec<AdjustmentOverride>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Member {
    pub key: RecordKey,
    pub name: String,
    pub label: Option<String>,
    #[serde(default)]
    pub searchable_label: bool,
    pub qualifier: Option<String>,
    #[serde(default)]
    pub coordinates: Coordinates,
    pub named_base: Option<NamedCreatureBase>,
    #[serde(default)]
    pub uses: Vec<ScenarioUse>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NamedCreatureBase {
    pub key: RecordKey,
    pub name: String,
}

impl CreatureFamily {
    pub(super) fn local_named_base<'a>(&self, member: &'a Member) -> Option<&'a RecordKey> {
        let base = member.named_base.as_ref()?;
        self.members
            .iter()
            .any(|candidate| candidate.key == base.key)
            .then_some(&base.key)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScenarioUse {
    pub id: String,
    pub party_levels: Option<[u8; 2]>,
    pub minimum_challenge_points: Option<u16>,
    pub qualifier: Option<String>,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatureAdjustment {
    Weak,
    Elite,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdjustmentOverride {
    pub base: RecordKey,
    pub adjustment: CreatureAdjustment,
    pub scenario_use: String,
    pub target: RecordKey,
    pub source_note: String,
}
