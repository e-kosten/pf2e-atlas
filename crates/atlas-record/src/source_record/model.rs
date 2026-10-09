use super::traversal::visit_source_nodes;
use super::{SourceIdentityError, source_record_key};
use atlas_domain::RecordKey;
use atlas_foundry_model::FoundryDocumentSource;

/// The source body uses atlas-foundry-model's checked snapshot codec. Deliberately
/// no envelope Serde: it must not bypass that codec's identity and stack guards.
/// Identity and the authored body are read-only after checked construction.
///
/// ```compile_fail
/// use atlas_record::source_record::SourceBackedRecord;
/// use atlas_domain::RecordKey;
/// fn replace_key(record: &mut SourceBackedRecord, key: RecordKey) {
///     record.key = key;
/// }
/// ```
///
/// ```compile_fail
/// use atlas_record::source_record::SourceBackedRecord;
/// use atlas_foundry_model::FoundryDocumentSource;
/// fn replace_body(record: &mut SourceBackedRecord, source: FoundryDocumentSource) {
///     record.source = source;
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct SourceBackedRecord {
    key: RecordKey,
    source: FoundryDocumentSource,
}

/// Failed construction returns the unchanged body for retention by its owner.
#[derive(Debug, PartialEq, Eq)]
pub struct SourceRecordConstructionError {
    pub reason: SourceIdentityError,
    pub source: FoundryDocumentSource,
}

impl SourceBackedRecord {
    /// Derive identity from the admitted body; corpus-wide collision exclusion
    /// remains the caller's responsibility before building the reference index.
    pub fn new(
        pack: &str,
        source: FoundryDocumentSource,
    ) -> Result<Self, SourceRecordConstructionError> {
        let key = match source_record_key(pack, &source) {
            Ok(key) => key,
            Err(reason) => return Err(SourceRecordConstructionError { reason, source }),
        };
        Ok(Self { key, source })
    }

    pub fn key(&self) -> &RecordKey {
        &self.key
    }

    pub fn source(&self) -> &FoundryDocumentSource {
        &self.source
    }
}

/// Developer reporting uses the same traversal as content and reference indexing.
/// No owned-document inventory is retained on the authoritative record.
pub fn source_owned_document_count(record: &SourceBackedRecord) -> usize {
    let mut count = 0usize;
    visit_source_nodes(record.source(), |owners, _| {
        count += usize::from(!owners.is_empty());
    });
    count
}
