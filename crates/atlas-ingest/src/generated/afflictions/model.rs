use atlas_domain::{PublicationCategory, Rarity};
use atlas_record::ContentDocument;
use serde_json::Value;

use crate::records::{AtlasRecord, LoadedSourceRecord, ReferenceEdge};

#[derive(Debug, thiserror::Error)]
pub(crate) enum GeneratedAfflictionError {
    #[error("invalid generated affliction pack name `{value}`: {message}")]
    InvalidPackName {
        value: &'static str,
        message: String,
    },
    #[error("invalid generated affliction record id `{value}`: {message}")]
    InvalidRecordId { value: String, message: String },
    #[error("generated affliction clustering produced an empty occurrence cluster")]
    EmptyCluster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AfflictionFamily {
    Curse,
    Disease,
    Poison,
}

#[derive(Debug, Clone)]
pub(crate) struct AfflictionOccurrence {
    pub(crate) host_record: AtlasRecord,
    pub(crate) source_record: Option<AtlasRecord>,
    pub(crate) description: Option<ContentDocument>,
    pub(crate) raw_provenance: Option<Value>,
    pub(crate) family: AfflictionFamily,
    pub(crate) name: String,
    pub(crate) traits: Vec<String>,
    pub(crate) linked_names: Vec<String>,
    pub(crate) source_path: String,
    pub(crate) occurrence_ref: String,
    pub(crate) candidate_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratedAfflictionBuild {
    pub(crate) records: Vec<LoadedSourceRecord>,
    pub(crate) references: Vec<ReferenceEdge>,
}

pub(crate) struct DerivedAfflictionRecordInput {
    pub(crate) key: atlas_domain::RecordKey,
    pub(crate) name: String,
    pub(crate) record_type: &'static str,
    pub(crate) family: AfflictionFamily,
    pub(crate) traits: Vec<String>,
    pub(crate) description: Option<ContentDocument>,
    pub(crate) blurb: Option<ContentDocument>,
    pub(crate) level: Option<i64>,
    pub(crate) rarity: Option<Rarity>,
    pub(crate) publication_title: Option<String>,
    pub(crate) publication_remaster: bool,
    pub(crate) category: PublicationCategory,
    pub(crate) source_path: String,
    pub(crate) is_default_visible: bool,
    pub(crate) raw: Value,
}
