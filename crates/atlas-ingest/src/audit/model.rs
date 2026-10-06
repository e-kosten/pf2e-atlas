use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct SourcePathAuditOptions {
    pub source_root: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub pack_name: Option<String>,
    pub document_type: Option<String>,
    pub record_type: Option<String>,
    pub min_records: usize,
    pub limit: Option<usize>,
    pub strict: bool,
    pub baseline_report: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct SourceValueDiscoveryOptions {
    pub source_root: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub filters: SourcePathAuditFilters,
    /// Exact normalized path copied from the schema inventory.
    pub path: String,
    /// Source references retained per distinct value; zero disables samples.
    pub sample_limit: usize,
    /// Display limit per document family, applied after complete counting.
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceValueDiscoveryReport {
    pub schema_version: String,
    pub source_signature: String,
    pub filters: SourcePathAuditFilters,
    pub path: String,
    pub pack_count: usize,
    pub record_count: usize,
    pub sample_limit: usize,
    pub complete: bool,
    pub fields: Vec<SourceFieldValueReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFieldValueReport {
    #[serde(flatten)]
    pub key: SourceSchemaKey,
    pub record_count: usize,
    pub missing_record_count: usize,
    pub occurrence_count: usize,
    pub duplicate_member_count: usize,
    pub value_types: Vec<SourcePathAuditValueType>,
    pub distinct_value_count: usize,
    pub complete: bool,
    pub values: Vec<SourceFieldValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFieldValue {
    /// Complete compact JSON; preserves duplicate members and authored ordering.
    pub value_json: String,
    pub value_type: String,
    pub record_count: usize,
    pub occurrence_count: usize,
    pub examples: Vec<SourceValueReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceValueReference {
    pub source_path: String,
    pub record_key: String,
    /// Concrete RFC 6901 pointer; duplicate keys share a pointer.
    pub source_pointer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePathAuditReport {
    pub schema_version: String,
    /// Manifest and selected source bytes/paths, independent of checkout location.
    pub source_signature: String,
    pub filters: SourcePathAuditFilters,
    pub pack_count: usize,
    pub record_count: usize,
    pub path_count: usize,
    /// False when display filters omit observed paths; then unsuitable as a baseline.
    pub complete: bool,
    pub paths: Vec<SourcePathAuditPathReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_diff: Option<SourcePathAuditDiff>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePathAuditFilters {
    pub pack_name: Option<String>,
    pub document_type: Option<String>,
    pub record_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceSchemaKey {
    pub document_type: String,
    pub record_type: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePathAuditPathReport {
    #[serde(flatten)]
    pub key: SourceSchemaKey,
    /// Counts source documents, even when IDs repeat or are absent.
    pub record_count: usize,
    /// Includes each array/map member and duplicate object member.
    pub occurrence_count: usize,
    pub duplicate_member_count: usize,
    pub value_types: Vec<SourcePathAuditValueType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<SourcePathAuditSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePathAuditValueType {
    pub kind: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePathAuditSample {
    pub record_key: String,
    pub source_path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePathAuditDiff {
    pub added_paths: Vec<SourceSchemaKey>,
    pub removed_paths: Vec<SourceSchemaKey>,
    pub changed_types: Vec<SourceSchemaTypeChange>,
    pub changed_duplicate_members: Vec<SourceSchemaKey>,
}

impl SourcePathAuditDiff {
    pub fn change_count(&self) -> usize {
        self.added_paths.len()
            + self.removed_paths.len()
            + self.changed_types.len()
            + self.changed_duplicate_members.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSchemaTypeChange {
    pub key: SourceSchemaKey,
    pub before: Vec<String>,
    pub after: Vec<String>,
}
