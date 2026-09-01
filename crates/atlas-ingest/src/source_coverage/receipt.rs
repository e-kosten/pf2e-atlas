use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::SourcePresence;

use super::{FinalOwnerStage, SourceLeafIdentity, SourcePin};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafReceipt {
    pub identity: SourceLeafIdentity,
    pub fixture: FixtureReference,
    pub source_pin: SourcePin,
    pub inventory_observed: bool,
    pub source: SourcePresence<SourceLeafValue>,
    #[serde(default)]
    pub reader_id: Option<String>,
    #[serde(default)]
    pub observations: Vec<StageObservation>,
    #[serde(default)]
    pub semantic_output_observed: bool,
    #[serde(default)]
    pub comparator_assertion_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureReference {
    pub case_id: String,
    pub record_key: String,
    pub source_path: String,
    pub source_commit: String,
    pub excerpt_digest: String,
    pub source_grounded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageObservation {
    pub stage: FinalOwnerStage,
    pub destination: String,
    pub value: SourcePresence<SourceLeafValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafValue {
    pub json_type: SourceJsonType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_kind: Option<SourceMemberKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinal: Option<usize>,
    pub multiplicity: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported: Option<TypedUnsupportedValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceJsonType {
    Boolean,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMemberKind {
    Array,
    Map,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedUnsupportedValue {
    pub value: Value,
    pub reason: String,
}
