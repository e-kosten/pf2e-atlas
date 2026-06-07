use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

use crate::TagId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignmentFile {
    pub records: Vec<RecordTagAssignments>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordTagAssignments {
    pub record_key: RecordKey,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<TagAssignment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagAssignment {
    pub tag_id: TagId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<AssignmentEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum AssignmentEvidence {
    ContentExcerpt {
        path: String,
        quote: String,
    },
    PresentationSection {
        section: String,
        summary: String,
    },
    NormalizedFact {
        field: TagFactField,
        value: String,
    },
    TagGuidanceMatch {
        signal: String,
        explanation: String,
    },
    SourceReference {
        record_key: RecordKey,
        relationship: String,
        summary: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagFactField {
    RecordKind,
    FoundryRecordType,
    Trait,
    Publication,
    Metric,
    Metadata,
}
