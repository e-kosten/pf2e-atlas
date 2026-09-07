use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::{
    AtlasRecord, ContentIdentityStability, ContentSourceKind, RecordBody, RecordContentDocument,
    RichDocument,
};
use serde_json::Value;

use crate::generated::afflictions::GeneratedAfflictionRole;
use crate::source::dto::{SpellDocumentSource, VersionedHazardSource, VersionedNpcSource};
use crate::source::normalize::ContentParseDiagnostics;
use crate::source::npc_core::NpcCoreDiagnostic;
use crate::source::npc_entities::{NpcEmbeddedCandidates, NpcEmbeddedDiagnostic};

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
    pub(crate) npc_source: Option<VersionedNpcSource>,
    pub(crate) hazard_source: Option<VersionedHazardSource>,
    pub(crate) spell_source: Option<SpellDocumentSource>,
    pub(crate) canonical_body: Option<RecordBody>,
    pub(crate) canonical_spell_children: Vec<atlas_record::ConsumableSpellChild>,
    pub(crate) npc_core_diagnostics: Vec<NpcCoreDiagnostic>,
    pub(crate) npc_embedded_candidates: Option<NpcEmbeddedCandidates>,
    pub(crate) npc_embedded_diagnostics: Vec<NpcEmbeddedDiagnostic>,
    pub(crate) generated_affliction_role: Option<GeneratedAfflictionRole>,
}

impl SourceConstructionFacts {
    pub(crate) fn empty() -> Self {
        Self {
            content_parse_diagnostics: Vec::new(),
            source_facts: SourceRecordFacts::default(),
            npc_source: None,
            hazard_source: None,
            spell_source: None,
            canonical_body: None,
            canonical_spell_children: Vec::new(),
            npc_core_diagnostics: Vec::new(),
            npc_embedded_candidates: None,
            npc_embedded_diagnostics: Vec::new(),
            generated_affliction_role: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct SourceRecordFacts {
    pub(crate) slug: Option<String>,
    pub(crate) compendium_source: Option<String>,
    pub(crate) source_content: BTreeMap<String, RecordContentDocument>,
    pub(crate) content_sources: Vec<SourceContentFact>,
    pub(crate) embedded_items: Vec<EmbeddedItemFact>,
    pub(crate) journal_pages: Vec<JournalPageFact>,
    pub(crate) skipped_journal_pages: Vec<SkippedJournalPageFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceContentFact {
    pub(crate) content_key: String,
    pub(crate) identity_stability: ContentIdentityStability,
    pub(crate) source_kind: ContentSourceKind,
    pub(crate) relative_source_path: String,
    pub(crate) nested_source_id: Option<String>,
    pub(crate) authored_ordinal_or_range: Option<String>,
    pub(crate) authored_order: u32,
    pub(crate) label: Option<String>,
    pub(crate) document: RichDocument,
    pub(crate) diagnostics: ContentParseDiagnostics,
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
    pub(crate) document: RichDocument,
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
