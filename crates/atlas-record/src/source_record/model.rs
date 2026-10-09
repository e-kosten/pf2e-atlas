use super::nodes::source_nodes;
use atlas_domain::RecordKey;
use atlas_foundry_model::FoundryDocumentSource;

/// The source body uses atlas-foundry-model's checked snapshot codec. Deliberately
/// no envelope Serde: it must not bypass that codec's identity and stack guards.
#[derive(Debug, PartialEq, Eq)]
pub struct SourceBackedRecord {
    pub key: RecordKey,
    pub source: FoundryDocumentSource,
}

/// Developer reporting uses the same traversal as content and reference indexing.
/// No owned-document inventory is retained on the authoritative record.
pub fn source_owned_document_count(record: &SourceBackedRecord) -> usize {
    source_nodes(&record.source).len().saturating_sub(1)
}
