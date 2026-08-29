use std::collections::BTreeMap;

use atlas_embedding::{
    DocumentEmbeddingContentSource, DocumentEmbeddingSource, EmbeddingUnitKind,
    PendingDocumentEmbedding, build_document_embedding_units,
};
use atlas_record::{
    AtlasRecord, ContentSourceKind, DuplicateContentStatus, OwnedRichContent, RecordAlias,
    RecordBody, RemasterLink, build_record_presentation_document_with_content_filter,
};

use crate::records::LoadedSourceRecord;
use crate::records::visibility::ProductRetrievalPolicy;

pub(crate) mod generation;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DocumentEmbeddingUnitSummary {
    pub total_units: usize,
    pub parent_units: usize,
    pub child_units: usize,
    pub records_with_child_units: usize,
    pub records_over_20_child_units: usize,
    pub records_over_50_child_units: usize,
    pub records_over_100_child_units: usize,
    pub max_child_units_per_record: usize,
}

pub(crate) fn build_pending_document_embeddings(
    records: &[LoadedSourceRecord],
    aliases: &[RecordAlias],
    remaster_links: &[RemasterLink],
) -> Vec<PendingDocumentEmbedding> {
    let aliases_by_key = aliases_by_record_key(aliases);
    let retrieval_policy = ProductRetrievalPolicy::from_remaster_links(remaster_links);

    let sources = records
        .iter()
        .filter_map(|loaded| {
            let record = &loaded.record;
            if !retrieval_policy.is_ordinary(loaded) {
                return None;
            }
            let canonical_content = canonical_embedding_content_documents(loaded);
            let record_key = record.identity.key.to_string();
            Some(DocumentEmbeddingSource {
                record_key,
                record_name: record.identity.name.clone(),
                document: build_record_presentation_document_with_content_filter(
                    record,
                    |content| canonical_content.is_none() && content.contributes_to_search(),
                ),
                aliases: aliases_by_key
                    .get(&record.identity.key.to_string())
                    .cloned()
                    .unwrap_or_default(),
                content_documents: canonical_content
                    .unwrap_or_else(|| embedding_content_documents(record)),
            })
        })
        .collect::<Vec<_>>();

    build_document_embedding_units(&sources)
}

fn canonical_embedding_content_documents(
    loaded: &LoadedSourceRecord,
) -> Option<Vec<DocumentEmbeddingContentSource>> {
    let RecordBody::Creature(creature) = loaded.facts.canonical_body.as_ref()?;
    Some(embedding_content_documents_from_owned(&creature.content))
}

fn embedding_content_documents_from_owned(
    content: &OwnedRichContent,
) -> Vec<DocumentEmbeddingContentSource> {
    content
        .documents
        .iter()
        .filter(|document| matches!(document.duplicate_status, DuplicateContentStatus::Unique))
        .map(|document| DocumentEmbeddingContentSource {
            source_kind: document.source_kind,
            label: document.label.clone().or_else(|| {
                Some(match document.source_kind {
                    ContentSourceKind::Description => "Description".to_string(),
                    ContentSourceKind::Blurb => "Summary".to_string(),
                    other => other.as_str().to_string(),
                })
            }),
            document: document.document.clone(),
        })
        .collect()
}

pub(crate) fn summarize_pending_document_embeddings(
    pending: &[PendingDocumentEmbedding],
) -> DocumentEmbeddingUnitSummary {
    let mut child_units_by_record = BTreeMap::<&str, usize>::new();
    let mut summary = DocumentEmbeddingUnitSummary {
        total_units: pending.len(),
        ..Default::default()
    };
    for unit in pending {
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
    for child_units in child_units_by_record.values().copied() {
        summary.max_child_units_per_record = summary.max_child_units_per_record.max(child_units);
        summary.records_over_20_child_units += usize::from(child_units > 20);
        summary.records_over_50_child_units += usize::from(child_units > 50);
        summary.records_over_100_child_units += usize::from(child_units > 100);
    }
    summary
}

fn embedding_content_documents(record: &AtlasRecord) -> Vec<DocumentEmbeddingContentSource> {
    record
        .content
        .searchable_documents()
        .filter(|content| !content.source_kind.is_embedded())
        .map(|content| DocumentEmbeddingContentSource {
            source_kind: content.source_kind,
            label: content.label.clone().or_else(|| {
                Some(match content.source_kind {
                    ContentSourceKind::Description => "Description".to_string(),
                    ContentSourceKind::Blurb => "Summary".to_string(),
                    other => other.as_str().to_string(),
                })
            }),
            document: content.document.clone(),
        })
        .collect()
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
    use std::collections::BTreeMap;

    use atlas_domain::{PackName, RecordId, RecordKey, RecordKind};
    use atlas_record::{
        AtlasRecord, ContentId, ContentIdentityStability, ContentKey, ContentOrigin, ContentOwner,
        ContentProvenance, ContentRole, ContentSourceKind, DuplicateContentStatus,
        FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, OwnedRichContent,
        OwnedRichContentDocument, RecordClassification, RecordContentDocument, RecordIdentity,
        RecordProvenance, RichDocument, RichNode,
    };

    use super::{
        build_pending_document_embeddings, embedding_content_documents_from_owned,
        summarize_pending_document_embeddings,
    };

    #[test]
    fn embedded_content_is_excluded_from_embedding_inputs_until_promoted() {
        let mut record = base_record();
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Description,
            label: None,
            document: text_document("Primary description"),
        });
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::EmbeddedItemDescription,
            label: Some("Embedded Strike".to_string()),
            document: RichDocument::new(vec![
                html_element("h2", vec![text_node("Embedded Strike")]),
                html_element("p", vec![text_node("Embedded capability text")]),
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
    fn canonical_embedding_content_keeps_unique_authored_prose_and_drops_copied_prose() {
        let record_key = RecordKey::parse("test-pack:TestRecord").expect("record key parses");
        let content = OwnedRichContent {
            documents: vec![
                owned_content_document(
                    &record_key,
                    "gm-notes",
                    ContentSourceKind::GmNotes,
                    "Unique GM-authored context",
                    DuplicateContentStatus::Unique,
                ),
                owned_content_document(
                    &record_key,
                    "unique-embedded",
                    ContentSourceKind::EmbeddedItemDescription,
                    "Unique embedded capability",
                    DuplicateContentStatus::Unique,
                ),
                owned_content_document(
                    &record_key,
                    "copied-embedded",
                    ContentSourceKind::EmbeddedSpellDescription,
                    "Copied canonical spell prose",
                    DuplicateContentStatus::CopiedFromCanonicalTarget {
                        target_record_key: RecordKey::parse("spells:CanonicalSpell")
                            .expect("target key parses"),
                    },
                ),
            ],
            exclusions: Vec::new(),
        };

        let documents = embedding_content_documents_from_owned(&content);
        let text = documents
            .iter()
            .map(|document| atlas_record::render_plain_text(&document.document))
            .collect::<Vec<_>>();

        assert_eq!(documents.len(), 2);
        assert!(
            text.iter()
                .any(|value| value == "Unique GM-authored context")
        );
        assert!(
            text.iter()
                .any(|value| value == "Unique embedded capability")
        );
        assert!(
            !text
                .iter()
                .any(|value| value == "Copied canonical spell prose")
        );
    }

    #[test]
    fn summarizes_embedding_unit_fanout() {
        let mut record = base_record();
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Description,
            label: None,
            document: RichDocument::new(vec![
                html_element("h2", vec![text_node("First")]),
                html_element("p", vec![text_node("First section text")]),
                html_element("h2", vec![text_node("Second")]),
                html_element("p", vec![text_node("Second section text")]),
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

        let summary = summarize_pending_document_embeddings(&pending);

        assert_eq!(summary.total_units, 1);
        assert_eq!(summary.parent_units, 1);
        assert_eq!(summary.child_units, 0);
        assert_eq!(summary.records_with_child_units, 0);
        assert_eq!(summary.max_child_units_per_record, 0);
    }

    fn text_document(text: &str) -> RichDocument {
        RichDocument::new(vec![html_element("p", vec![text_node(text)])])
    }

    fn owned_content_document(
        record_key: &RecordKey,
        content_key: &str,
        source_kind: ContentSourceKind,
        text: &str,
        duplicate_status: DuplicateContentStatus,
    ) -> OwnedRichContentDocument {
        OwnedRichContentDocument::new(
            ContentId::new(
                record_key.clone(),
                ContentKey::new(content_key).expect("content key parses"),
            ),
            ContentIdentityStability::StableSourceIdentity,
            ContentOwner::Record(record_key.clone()),
            ContentRole::SupplementalRules,
            ContentOrigin::RecordField {
                source_kind,
                relative_source_path: format!("$.{content_key}"),
            },
            source_kind.default_visibility(),
            ContentProvenance {
                source_record_key: record_key.clone(),
                relative_source_path: "packs/test.json".to_string(),
                field_or_pointer_family: format!("$.{content_key}"),
                nested_source_id: None,
                authored_ordinal_or_range: None,
                authored_label: None,
            },
            source_kind,
            0,
            None,
            text_document(text),
            duplicate_status,
            Vec::new(),
        )
    }

    fn html_element(tag: &str, children: Vec<RichNode>) -> RichNode {
        RichNode::HtmlElement {
            tag: tag.to_string(),
            attributes: BTreeMap::new(),
            children,
        }
    }

    fn text_node(text: &str) -> RichNode {
        RichNode::Text {
            text: text.to_string(),
        }
    }

    fn base_record() -> AtlasRecord {
        let pack_name = PackName::new("test-pack").expect("pack parses");
        let id = RecordId::new("TestRecord").expect("id parses");
        AtlasRecord::new(
            RecordIdentity::new(RecordKey::new(pack_name, id), "Test Record"),
            RecordClassification::new(RecordKind::Rule),
            FoundryRecordInfo::new(
                "Test Pack",
                FoundryDocumentType::Item,
                FoundryRecordType::Action,
            ),
            RecordProvenance::new("test.json").with_raw_json("{}"),
        )
    }
}
