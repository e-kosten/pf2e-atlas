use atlas_domain::PublicationFamily;
use serde_json::Value;

use crate::{LoadedRecord, ReferenceEdge};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AfflictionFamily {
    Curse,
    Disease,
    Poison,
}

#[derive(Debug, Clone)]
pub(crate) struct AfflictionOccurrence {
    pub(crate) host_record: LoadedRecord,
    pub(crate) source_record: Option<LoadedRecord>,
    pub(crate) source_raw: Option<Value>,
    pub(crate) child_raw: Value,
    pub(crate) family: AfflictionFamily,
    pub(crate) name: String,
    pub(crate) slug: Option<String>,
    pub(crate) traits: Vec<String>,
    pub(crate) linked_names: Vec<String>,
    pub(crate) source_path: String,
    pub(crate) occurrence_ref: String,
    pub(crate) candidate_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratedAfflictionBuild {
    pub(crate) records: Vec<LoadedRecord>,
    pub(crate) references: Vec<ReferenceEdge>,
}

pub(crate) struct DerivedAfflictionRecordInput {
    pub(crate) key: atlas_domain::RecordKey,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) record_type: &'static str,
    pub(crate) family: AfflictionFamily,
    pub(crate) traits: Vec<String>,
    pub(crate) description_text: Option<String>,
    pub(crate) blurb_text: Option<String>,
    pub(crate) level: Option<i64>,
    pub(crate) rarity: Option<String>,
    pub(crate) publication_title: Option<String>,
    pub(crate) publication_remaster: bool,
    pub(crate) publication_family: PublicationFamily,
    pub(crate) source_path: String,
    pub(crate) is_default_visible: bool,
    pub(crate) search_text_projection: String,
    pub(crate) raw: Value,
}
