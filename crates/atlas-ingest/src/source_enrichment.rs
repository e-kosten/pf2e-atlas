//! Consuming handoff from typed admission to database-independent records.
use crate::{
    LoadedFoundrySource, QuarantinedSourceFile, SourceFileProvenance, SourceLoadFailure,
    SourceMetadata, SourcePackMetadata,
};
use atlas_foundry_model::{FoundryDocumentSource, SourceDiagnostic, SourceValue};
use atlas_record::{
    source_content::{ContentAudience, LocalizationResolver},
    source_record::{
        SourceBackedRecord, SourceContentOutcome, SourceContentStatus, SourceIdentityError,
        SourceReferenceIndex, SourceRelationshipOccurrence, prepare_record_content,
        resolve_source_relationships, source_owned_document_count, source_record_key,
    },
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

pub struct SourceEnrichmentContext<'a> {
    pub audience: ContentAudience,
    pub localization: Option<&'a dyn LocalizationResolver>,
}

/// Source bytes/hash/provenance and admission diagnostics remain ingest-owned.
/// There is one DTO body; the admission's additional full raw tree is discarded.
#[derive(Debug, PartialEq, Eq)]
pub struct EnrichedFoundryDocument {
    pub provenance: SourceFileProvenance,
    pub content_hash: String,
    pub bytes: Vec<u8>,
    pub admission_diagnostics: Vec<SourceDiagnostic>,
    pub outcome: EnrichedDocumentOutcome,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EnrichedDocumentOutcome {
    Addressed {
        record: Box<SourceBackedRecord>,
        content: Vec<SourceContentOutcome>,
        relationships: Vec<SourceRelationshipOccurrence>,
    },
    /// No invented key. An admitted model is still retained when only identity
    /// is unavailable; unsupported roots retain exact bytes and diagnostics.
    Unavailable {
        reason: SourceIdentityError,
        source: Option<FoundryDocumentSource>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct EnrichedFoundryPack {
    pub metadata: SourcePackMetadata,
    pub documents: Vec<EnrichedFoundryDocument>,
    pub quarantined_files: Vec<QuarantinedSourceFile>,
    pub discovery_failure: Option<SourceLoadFailure>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct EnrichedFoundrySource {
    pub metadata: SourceMetadata,
    pub packs: Vec<EnrichedFoundryPack>,
}

pub fn enrich_loaded_source(
    source: LoadedFoundrySource,
    context: SourceEnrichmentContext<'_>,
) -> EnrichedFoundrySource {
    let mut counts = BTreeMap::new();
    let mut keys = Vec::with_capacity(source.packs.len());
    for pack in &source.packs {
        let mut pack_keys = Vec::with_capacity(pack.documents.len());
        for document in &pack.documents {
            // Identity exclusion only: unsupported/ambiguous roots can collide
            // with admitted roots. Inspect envelope _id candidates before
            // discarding admission.raw; never derive product fields or a typed
            // record from this evidence. Repeated members reserve each candidate
            // once per root, without choosing a winner.
            if let SourceValue::Object(raw) = &document.admission.raw {
                let ids = raw
                    .fields()
                    .iter()
                    .filter_map(|(name, value)| match (name.as_str(), value) {
                        ("_id", SourceValue::String(id)) => Some(id),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                for id in ids {
                    *counts
                        .entry((pack.metadata.name.clone(), id.clone()))
                        .or_insert(0usize) += 1;
                }
            }
            let key = document
                .admission
                .model
                .as_ref()
                .ok_or(SourceIdentityError::UnsupportedRoot)
                .and_then(|model| source_record_key(&pack.metadata.name, model));
            pack_keys.push(key);
        }
        keys.push(pack_keys);
    }
    let mut resolver = SourceReferenceIndex::default();
    for (pack, keys) in source.packs.iter().zip(&mut keys) {
        for (document, key) in pack.documents.iter().zip(keys) {
            if key.as_ref().is_ok_and(|key| {
                counts
                    .get(&(key.pack().as_str().into(), key.id().as_str().into()))
                    .copied()
                    .unwrap_or(0)
                    > 1
            }) {
                *key = Err(SourceIdentityError::DuplicateId);
            }
            if let (Ok(key), Some(model)) = (key, &document.admission.model) {
                resolver.insert_source(key, model);
            } else if let SourceValue::Object(raw) = &document.admission.raw {
                // Unavailable roots cannot supply a destination, but their
                // authored names must not make another root falsely unique.
                // As with _id above, this is collision exclusion only.
                for (name, value) in raw.fields() {
                    if let ("name", SourceValue::String(name)) = (name.as_str(), value) {
                        resolver.exclude_root_name(
                            &pack.metadata.name,
                            &pack.metadata.document_type,
                            name,
                        );
                    }
                }
            }
        }
    }
    let packs = source
        .packs
        .into_iter()
        .zip(keys)
        .map(|(pack, keys)| {
            let documents = pack
                .documents
                .into_iter()
                .zip(keys)
                .map(|(document, key)| {
                    let outcome = match (key, document.admission.model) {
                        (Ok(key), Some(model)) => {
                            match SourceBackedRecord::new(key.pack().as_str(), model) {
                                Ok(record) => {
                                    let content = prepare_record_content(
                                        &record,
                                        context.audience,
                                        context.localization,
                                        Some(&resolver),
                                    );
                                    let relationships =
                                        resolve_source_relationships(&record, Some(&resolver));
                                    EnrichedDocumentOutcome::Addressed {
                                        record: Box::new(record),
                                        content,
                                        relationships,
                                    }
                                }
                                Err(error) => EnrichedDocumentOutcome::Unavailable {
                                    reason: error.reason,
                                    source: Some(error.source),
                                },
                            }
                        }
                        (Err(reason), model) => EnrichedDocumentOutcome::Unavailable {
                            reason,
                            source: model,
                        },
                        (Ok(_), None) => EnrichedDocumentOutcome::Unavailable {
                            reason: SourceIdentityError::UnsupportedRoot,
                            source: None,
                        },
                    };
                    EnrichedFoundryDocument {
                        provenance: document.provenance,
                        content_hash: document.content_hash,
                        bytes: document.bytes,
                        admission_diagnostics: document.admission.diagnostics,
                        outcome,
                    }
                })
                .collect();
            EnrichedFoundryPack {
                metadata: pack.metadata,
                documents,
                quarantined_files: pack.quarantined_files,
                discovery_failure: pack.discovery_failure,
            }
        })
        .collect();
    EnrichedFoundrySource {
        metadata: source.metadata,
        packs,
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SourceEnrichmentReport {
    pub discovered_files: usize,
    pub retained_documents: usize,
    pub addressed_documents: usize,
    pub typed_documents: usize,
    pub partial_documents: usize,
    pub raw_only_documents: usize,
    pub identity_unavailable: BTreeMap<String, usize>,
    pub quarantined_files: usize,
    pub unavailable_packs: usize,
    pub admission_diagnostics: usize,
    pub owned_nodes: usize,
    pub content_outcomes: BTreeMap<String, usize>,
    pub content_diagnostics: BTreeMap<String, usize>,
    pub references: usize,
    pub relationships: usize,
}

impl EnrichedFoundrySource {
    pub fn report(&self) -> SourceEnrichmentReport {
        let mut report = SourceEnrichmentReport::default();
        for pack in &self.packs {
            report.discovered_files += pack.metadata.discovered_file_count;
            report.quarantined_files += pack.quarantined_files.len();
            report.unavailable_packs += usize::from(pack.discovery_failure.is_some());
            for document in &pack.documents {
                report.retained_documents += 1;
                report.admission_diagnostics += document.admission_diagnostics.len();
                let typed = match &document.outcome {
                    EnrichedDocumentOutcome::Addressed {
                        record,
                        content,
                        relationships,
                    } => {
                        report.addressed_documents += 1;
                        report.owned_nodes += source_owned_document_count(record);
                        report.relationships += relationships.len();
                        for content in content {
                            let state = match &content.status {
                                SourceContentStatus::Prepared(prepared) => {
                                    report.references += prepared.references.len();
                                    for diagnostic in &prepared.diagnostics {
                                        *report
                                            .content_diagnostics
                                            .entry(format!("{:?}", diagnostic.code))
                                            .or_default() += 1;
                                    }
                                    "prepared"
                                }
                                SourceContentStatus::FormatUnavailable { .. } => {
                                    "format_unavailable"
                                }
                                SourceContentStatus::UnsupportedFormat { .. } => {
                                    "unsupported_format"
                                }
                                SourceContentStatus::PreparationFailed { .. } => {
                                    "preparation_failed"
                                }
                            };
                            *report.content_outcomes.entry(state.into()).or_default() += 1;
                        }
                        true
                    }
                    EnrichedDocumentOutcome::Unavailable { reason, source } => {
                        *report
                            .identity_unavailable
                            .entry(format!("{reason:?}"))
                            .or_default() += 1;
                        source.is_some()
                    }
                };
                if typed {
                    report.typed_documents += 1;
                    report.partial_documents +=
                        usize::from(!document.admission_diagnostics.is_empty());
                } else {
                    report.raw_only_documents += 1;
                }
            }
        }
        report
    }
}

#[cfg(test)]
mod tests;
