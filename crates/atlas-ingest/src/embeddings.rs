use std::collections::BTreeMap;

use atlas_embedding::{
    DocumentEmbeddingContentSource, DocumentEmbeddingSource, EmbeddingUnitKind,
    PendingDocumentEmbedding, build_document_embedding_units,
};
use atlas_record::{
    AtlasRecord, ContentSourceKind, RecordAlias, RemasterLink,
    build_record_presentation_document_with_content_filter,
};

use crate::records::LoadedSourceRecord;
use crate::records::visibility::RetrievalVisibility;

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
    let retrieval_visibility = RetrievalVisibility::from_remaster_links(remaster_links);

    let sources = records
        .iter()
        .filter_map(|loaded| {
            let record = &loaded.record;
            if !retrieval_visibility.is_default_visible(record) {
                return None;
            }
            let record_key = record.identity.key.to_string();
            Some(DocumentEmbeddingSource {
                record_key,
                record_name: record.identity.name.clone(),
                document: build_record_presentation_document_with_content_filter(
                    record,
                    |content| !content.source_kind.is_embedded(),
                ),
                aliases: aliases_by_key
                    .get(&record.identity.key.to_string())
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
        AtlasRecord, ContentSourceKind, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        RecordClassification, RecordContentDocument, RecordIdentity, RecordProvenance,
        RichDocument, RichNode,
    };

    use super::{build_pending_document_embeddings, summarize_pending_document_embeddings};

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
