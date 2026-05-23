use std::collections::BTreeMap;

use atlas_embedding::{
    DocumentEmbeddingContentSource, DocumentEmbeddingSource, EmbeddingUnitKind,
    PendingDocumentEmbedding, build_document_embedding_units,
};
use atlas_record::{ContentSourceKind, NormalizedRecord, build_record_presentation_document};

use crate::records::visibility::RetrievalVisibility;
use crate::records::{LoadedSourceRecord, RecordAlias, RemasterLink};

pub(crate) mod generation;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DocumentEmbeddingUnitSummary {
    pub total_units: usize,
    pub parent_units: usize,
    pub child_units: usize,
    pub child_candidate_units: usize,
    pub records_with_child_units: usize,
    pub records_with_child_candidates: usize,
    pub records_over_20_child_units: usize,
    pub records_over_50_child_units: usize,
    pub records_over_100_child_units: usize,
    pub max_child_units_per_record: usize,
    pub max_child_candidates_per_record: usize,
}

pub(crate) fn build_pending_document_embeddings(
    records: &[LoadedSourceRecord],
    aliases: &[RecordAlias],
    remaster_links: &[RemasterLink],
) -> Vec<PendingDocumentEmbedding> {
    let aliases_by_key = aliases_by_record_key(aliases);
    let retrieval_visibility = RetrievalVisibility::from_remaster_links(remaster_links);

    let sources = records
        .iter()
        .filter_map(|loaded| {
            let record = &loaded.record;
            if !retrieval_visibility.is_default_visible(record) {
                return None;
            }
            let record_key = record.key.to_string();
            Some(DocumentEmbeddingSource {
                record_key,
                record_name: record.name.clone(),
                document: build_record_presentation_document(&embedding_parent_record(record)),
                aliases: aliases_by_key
                    .get(&record.key.to_string())
                    .cloned()
                    .unwrap_or_default(),
                content_documents: embedding_content_documents(record),
            })
        })
        .collect::<Vec<_>>();

    build_document_embedding_units(&sources)
}

pub(crate) fn summarize_pending_document_embeddings(
    pending: &[PendingDocumentEmbedding],
) -> DocumentEmbeddingUnitSummary {
    let mut child_units_by_record = BTreeMap::<&str, usize>::new();
    let mut child_candidates_by_record = BTreeMap::<&str, usize>::new();
    let mut summary = DocumentEmbeddingUnitSummary {
        total_units: pending.len(),
        ..Default::default()
    };
    for unit in pending {
        if !unit.child_candidates.is_empty() {
            summary.child_candidate_units += unit.child_candidates.len();
            *child_candidates_by_record
                .entry(unit.record_key.as_str())
                .or_default() += unit.child_candidates.len();
        }
        if unit.unit_kind == EmbeddingUnitKind::Parent {
            summary.parent_units += 1;
        } else {
            summary.child_units += 1;
            *child_units_by_record
                .entry(unit.record_key.as_str())
                .or_default() += 1;
        }
    }
    summary.records_with_child_units = child_units_by_record.len();
    summary.records_with_child_candidates = child_candidates_by_record.len();
    for child_units in child_units_by_record.values().copied() {
        summary.max_child_units_per_record = summary.max_child_units_per_record.max(child_units);
        summary.records_over_20_child_units += usize::from(child_units > 20);
        summary.records_over_50_child_units += usize::from(child_units > 50);
        summary.records_over_100_child_units += usize::from(child_units > 100);
    }
    for child_candidates in child_candidates_by_record.values().copied() {
        summary.max_child_candidates_per_record = summary
            .max_child_candidates_per_record
            .max(child_candidates);
    }
    summary
}

fn embedding_parent_record(record: &NormalizedRecord) -> NormalizedRecord {
    let mut record = record.clone();
    record
        .supplemental_content
        .retain(|content| !content.source_kind.is_embedded());
    record
}

fn embedding_content_documents(record: &NormalizedRecord) -> Vec<DocumentEmbeddingContentSource> {
    let mut documents = Vec::new();
    if let Some(document) = &record.description {
        documents.push(DocumentEmbeddingContentSource {
            source_kind: ContentSourceKind::Description,
            label: Some("Description".to_string()),
            document: document.clone(),
        });
    }
    if let Some(document) = &record.blurb {
        documents.push(DocumentEmbeddingContentSource {
            source_kind: ContentSourceKind::Blurb,
            label: Some("Summary".to_string()),
            document: document.clone(),
        });
    }
    documents.extend(
        record
            .supplemental_content
            .iter()
            .filter(|content| content.contributes_to_search && !content.source_kind.is_embedded())
            .map(|content| DocumentEmbeddingContentSource {
                source_kind: content.source_kind,
                label: content
                    .label
                    .clone()
                    .or_else(|| Some(content.source_kind.as_str().to_string())),
                document: content.document.clone(),
            }),
    );
    documents
}

fn aliases_by_record_key(aliases: &[RecordAlias]) -> BTreeMap<String, Vec<String>> {
    let mut by_key = BTreeMap::<String, Vec<String>>::new();
    for alias in aliases {
        by_key
            .entry(alias.canonical_record_key.to_string())
            .or_default()
            .push(alias.alias_text.clone());
    }
    for aliases in by_key.values_mut() {
        aliases.sort();
        aliases.dedup();
    }
    by_key
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, PublicationFamily, RecordFamily, RecordId, RecordKey};
    use atlas_record::{
        ContentBlock, ContentDocument, ContentInline, ContentSourceKind, ContentVisibility,
        NormalizedRecord, SupplementalContentDocument,
    };

    use super::{build_pending_document_embeddings, summarize_pending_document_embeddings};

    #[test]
    fn embedded_content_is_excluded_from_embedding_inputs_until_promoted() {
        let mut record = base_record();
        record.description = Some(text_document("Primary description"));
        record
            .supplemental_content
            .push(SupplementalContentDocument {
                source_kind: ContentSourceKind::EmbeddedItemDescription,
                visibility: ContentVisibility::Public,
                contributes_to_search: true,
                contributes_to_references: true,
                label: Some("Embedded Strike".to_string()),
                document: ContentDocument::new(vec![
                    ContentBlock::Heading {
                        level: 2,
                        content: vec![ContentInline::Text {
                            text: "Embedded Strike".to_string(),
                        }],
                    },
                    ContentBlock::Paragraph {
                        content: vec![ContentInline::Text {
                            text: "Embedded capability text".to_string(),
                        }],
                    },
                ]),
            });

        let pending = build_pending_document_embeddings(
            &[crate::records::LoadedSourceRecord::new(
                record,
                crate::records::SourceConstructionFacts::empty(),
            )],
            &[],
            &[],
        );

        let parent = pending
            .iter()
            .find(|unit| unit.embedding_unit_key == "test-pack:TestRecord#parent")
            .expect("parent unit exists");
        assert!(!parent.input_text.contains("Embedded capability text"));
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn summarizes_embedding_unit_fanout() {
        let mut record = base_record();
        record.description = Some(ContentDocument::new(vec![
            ContentBlock::Heading {
                level: 2,
                content: vec![ContentInline::Text {
                    text: "First".to_string(),
                }],
            },
            ContentBlock::Paragraph {
                content: vec![ContentInline::Text {
                    text: "First section text".to_string(),
                }],
            },
            ContentBlock::Heading {
                level: 2,
                content: vec![ContentInline::Text {
                    text: "Second".to_string(),
                }],
            },
            ContentBlock::Paragraph {
                content: vec![ContentInline::Text {
                    text: "Second section text".to_string(),
                }],
            },
        ]));
        let pending = build_pending_document_embeddings(
            &[crate::records::LoadedSourceRecord::new(
                record,
                crate::records::SourceConstructionFacts::empty(),
            )],
            &[],
            &[],
        );

        let summary = summarize_pending_document_embeddings(&pending);

        assert_eq!(summary.total_units, 1);
        assert_eq!(summary.parent_units, 1);
        assert_eq!(summary.child_units, 0);
        assert_eq!(summary.child_candidate_units, 2);
        assert_eq!(summary.records_with_child_units, 0);
        assert_eq!(summary.records_with_child_candidates, 1);
        assert_eq!(summary.max_child_units_per_record, 0);
        assert_eq!(summary.max_child_candidates_per_record, 2);
        assert_eq!(pending[0].child_candidates.len(), 2);
    }

    fn text_document(text: &str) -> ContentDocument {
        ContentDocument::new(vec![ContentBlock::Paragraph {
            content: vec![ContentInline::Text {
                text: text.to_string(),
            }],
        }])
    }

    fn base_record() -> NormalizedRecord {
        let pack_name = PackName::new("test-pack").expect("pack parses");
        let id = RecordId::new("TestRecord").expect("id parses");
        NormalizedRecord {
            key: RecordKey::new(pack_name.clone(), id.clone()),
            id,
            name: "Test Record".to_string(),
            normalized_name: "test record".to_string(),
            record_family: RecordFamily::Rule,
            pack_name,
            pack_label: "Test Pack".to_string(),
            foundry_document_type: "Item".to_string(),
            foundry_record_type: "action".to_string(),
            level: None,
            rarity: None,
            traits: Vec::new(),
            prerequisites: Vec::new(),
            system_category: None,
            system_group: None,
            system_base_item: None,
            system_usage: None,
            system_price_json: None,
            system_actions_value: None,
            system_time_value: None,
            system_duration_value: None,
            price_cp: None,
            activation_time: None,
            duration: None,
            metrics: Vec::new(),
            actor_data: None,
            item_data: None,
            spell_data: None,
            publication_title: None,
            publication_remaster: false,
            description: None,
            blurb: None,
            supplemental_content: Vec::new(),
            publication_family: PublicationFamily::Unknown,
            folder_id: None,
            taxonomy_families: Vec::new(),
            variant_group_key: None,
            variant_base_name: None,
            variant_label: None,
            variant_axes: Vec::new(),
            variant_confidence: None,
            variant_source: "none".to_string(),
            source_path: "test.json".to_string(),
            is_default_visible: true,
            raw_json: "{}".to_string(),
        }
    }
}
