use atlas_domain::PublicationFamily;
use atlas_record::ContentDocument;
use serde_json::Value;

use crate::records::{LoadedSourceRecord, NormalizedRecord, ReferenceEdge};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AfflictionFamily {
    Curse,
    Disease,
    Poison,
}

#[derive(Debug, Clone)]
pub(crate) struct AfflictionOccurrence {
    pub(crate) host_record: NormalizedRecord,
    pub(crate) source_record: Option<NormalizedRecord>,
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
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) record_type: &'static str,
    pub(crate) family: AfflictionFamily,
    pub(crate) traits: Vec<String>,
    pub(crate) description: Option<ContentDocument>,
    pub(crate) blurb: Option<ContentDocument>,
    pub(crate) level: Option<i64>,
    pub(crate) rarity: Option<String>,
    pub(crate) publication_title: Option<String>,
    pub(crate) publication_remaster: bool,
    pub(crate) publication_family: PublicationFamily,
    pub(crate) source_path: String,
    pub(crate) is_default_visible: bool,
    pub(crate) raw: Value,
}
