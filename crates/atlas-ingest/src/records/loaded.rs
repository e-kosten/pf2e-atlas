use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::{AtlasRecord, ContentDocument, ContentSourceKind, RecordContentDocument};
use serde_json::Value;

use crate::source::normalize::ContentParseDiagnostics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceCandidate {
    pub(crate) raw_target: String,
    pub(crate) display_text: Option<String>,
    pub(crate) reference_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LoadedSourceRecord {
    pub(crate) record: AtlasRecord,
    pub(crate) facts: SourceConstructionFacts,
}

impl LoadedSourceRecord {
    pub(crate) const fn new(record: AtlasRecord, facts: SourceConstructionFacts) -> Self {
        Self { record, facts }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceConstructionFacts {
    pub(crate) content_parse_diagnostics: Vec<ContentParseDiagnostics>,
    pub(crate) source_facts: SourceRecordFacts,
}

impl SourceConstructionFacts {
    pub(crate) fn empty() -> Self {
        Self {
            content_parse_diagnostics: Vec::new(),
            source_facts: SourceRecordFacts::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct SourceRecordFacts {
    pub(crate) slug: Option<String>,
    pub(crate) compendium_source: Option<String>,
    pub(crate) source_content: BTreeMap<String, RecordContentDocument>,
    pub(crate) embedded_items: Vec<EmbeddedItemFact>,
    pub(crate) journal_pages: Vec<JournalPageFact>,
    pub(crate) skipped_journal_pages: Vec<SkippedJournalPageFact>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EmbeddedItemFact {
    pub(crate) host_record_key: RecordKey,
    pub(crate) item_id: String,
    pub(crate) name: String,
    pub(crate) normalized_name: String,
    pub(crate) foundry_item_type: String,
    pub(crate) traits: Vec<String>,
    pub(crate) system_category: Option<String>,
    pub(crate) slug: Option<String>,
    pub(crate) compendium_source: Option<String>,
    pub(crate) publication_remaster: bool,
    pub(crate) content_refs: Vec<EmbeddedItemContentRef>,
    /// Provenance-only payload retained for generated instance raw serialization.
    /// Post-normalization consumers must use typed fields instead of parsing it.
    pub(crate) raw_provenance: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmbeddedItemContentRef {
    pub(crate) source_kind: ContentSourceKind,
    pub(crate) local_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JournalPageFact {
    pub(crate) host_record_key: RecordKey,
    pub(crate) page_id: Option<String>,
    pub(crate) name: String,
    pub(crate) normalized_name: String,
    pub(crate) ordinal: i64,
    pub(crate) source_ref: String,
    pub(crate) source_markup: String,
    pub(crate) document: ContentDocument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SkippedJournalPageFact {
    pub(crate) host_record_key: RecordKey,
    pub(crate) page_id: Option<String>,
    pub(crate) name: String,
    pub(crate) normalized_name: String,
    pub(crate) ordinal: i64,
    pub(crate) reason: JournalPageSkipReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JournalPageSkipReason {
    MissingTextContent,
    EmptyTextContent,
    EmptyParsedDocument,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RecordReferenceIndex {
    pub(crate) by_key: BTreeMap<String, AtlasRecord>,
    pub(crate) by_pack_id: BTreeMap<(String, String), RecordKey>,
    pub(crate) by_pack_name: BTreeMap<(String, String), Vec<RecordKey>>,
    pub(crate) by_name: BTreeMap<String, Vec<RecordKey>>,
}
