use atlas_domain::{PublicationCategory, Rarity};
use atlas_record::{AtlasRecord, ReferenceEdge, RichDocument};
use serde_json::Value;

use crate::records::LoadedSourceRecord;

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
    pub(crate) description: Option<RichDocument>,
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
    pub(crate) relationships: Vec<GeneratedAfflictionRelationship>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GeneratedAfflictionRole {
    Canonical,
    SourceInstance,
}

impl GeneratedAfflictionRole {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::SourceInstance => "source_instance",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GeneratedAfflictionRelationshipKind {
    HostHasSourceInstance,
    SourceInstanceOfCanonical,
    CanonicalDerivedFromHostOccurrence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedAfflictionRelationship {
    pub(crate) kind: GeneratedAfflictionRelationshipKind,
    pub(crate) from: atlas_domain::RecordKey,
    pub(crate) to: atlas_domain::RecordKey,
}

pub(crate) struct DerivedAfflictionRecordInput {
    pub(crate) key: atlas_domain::RecordKey,
    pub(crate) name: String,
    pub(crate) record_type: &'static str,
    pub(crate) family: AfflictionFamily,
    pub(crate) traits: Vec<String>,
    pub(crate) description: Option<RichDocument>,
    pub(crate) blurb: Option<RichDocument>,
    pub(crate) level: Option<i64>,
    pub(crate) rarity: Option<Rarity>,
    pub(crate) publication_title: Option<String>,
    pub(crate) publication_remaster: bool,
    pub(crate) category: PublicationCategory,
    pub(crate) source_path: String,
    pub(crate) role: GeneratedAfflictionRole,
    pub(crate) raw: Value,
}
