use atlas_embedding::EmbeddingUnitKind;
use atlas_record::{
    ContentDocument, ContentSectionNode, ContentSectionOrigin, ContentSourceKind,
    ContentVisibility, NormalizedRecord, build_content_section_tree, render_plain_text,
};

#[derive(Debug, Clone)]
pub(crate) struct EvidenceUnit {
    pub(crate) evidence_unit_key: String,
    pub(crate) record_key: String,
    pub(crate) source_content_unit_key: Option<String>,
    pub(crate) source_kind: ContentSourceKind,
    pub(crate) visibility: ContentVisibility,
    pub(crate) unit_kind: String,
    pub(crate) label: Option<String>,
    pub(crate) ordinal: i64,
    pub(crate) search_text: String,
    pub(crate) document: ContentDocument,
}

pub(crate) fn evidence_units(records: &[&NormalizedRecord]) -> Vec<EvidenceUnit> {
    let mut units = Vec::new();
    for record in records {
        let mut evidence_ordinal = 0i64;
        let mut embedding_child_ordinal = 0i64;
        if let Some(document) = &record.description {
            push_document_evidence_units(
                &mut units,
                DocumentEvidenceInput {
                    record,
                    source_content_unit_key: None,
                    source_kind: ContentSourceKind::Description,
                    visibility: ContentVisibility::Public,
                    label: Some("Description".to_string()),
                    document,
                    can_map_embedding: true,
                },
                &mut embedding_child_ordinal,
                &mut evidence_ordinal,
            );
        }
        if let Some(document) = &record.blurb {
            push_document_evidence_units(
                &mut units,
                DocumentEvidenceInput {
                    record,
                    source_content_unit_key: None,
                    source_kind: ContentSourceKind::Blurb,
                    visibility: ContentVisibility::Public,
                    label: Some("Summary".to_string()),
                    document,
                    can_map_embedding: true,
                },
                &mut embedding_child_ordinal,
                &mut evidence_ordinal,
            );
        }
        for (content_ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            let source_content_unit_key = Some(format!("{}#content#{content_ordinal}", record.key));
            let can_map_embedding =
                supplemental.contributes_to_search && !supplemental.source_kind.is_embedded();
            if supplemental.source_kind.is_embedded() {
                evidence_ordinal += 1;
                units.push(EvidenceUnit {
                    evidence_unit_key: format!("{}#evidence#content#{content_ordinal}", record.key),
                    record_key: record.key.to_string(),
                    source_content_unit_key,
                    source_kind: supplemental.source_kind,
                    visibility: supplemental.visibility,
                    unit_kind: "content_unit".to_string(),
                    label: supplemental
                        .label
                        .clone()
                        .or_else(|| Some(supplemental.source_kind.as_str().to_string())),
                    ordinal: evidence_ordinal,
                    search_text: render_plain_text(&supplemental.document),
                    document: supplemental.document.clone(),
                });
            } else {
                push_document_evidence_units(
                    &mut units,
                    DocumentEvidenceInput {
                        record,
                        source_content_unit_key,
                        source_kind: supplemental.source_kind,
                        visibility: supplemental.visibility,
                        label: supplemental
                            .label
                            .clone()
                            .or_else(|| Some(supplemental.source_kind.as_str().to_string())),
                        document: &supplemental.document,
                        can_map_embedding,
                    },
                    &mut embedding_child_ordinal,
                    &mut evidence_ordinal,
                );
            }
        }
    }
    units
}

struct DocumentEvidenceInput<'a> {
    record: &'a NormalizedRecord,
    source_content_unit_key: Option<String>,
    source_kind: ContentSourceKind,
    visibility: ContentVisibility,
    label: Option<String>,
    document: &'a ContentDocument,
    can_map_embedding: bool,
}

fn push_document_evidence_units(
    units: &mut Vec<EvidenceUnit>,
    input: DocumentEvidenceInput<'_>,
    embedding_child_ordinal: &mut i64,
    evidence_ordinal: &mut i64,
) {
    let DocumentEvidenceInput {
        record,
        source_content_unit_key,
        source_kind,
        visibility,
        label,
        document,
        can_map_embedding,
    } = input;
    let sections = explicit_heading_evidence_sections(document);
    if sections.is_empty() {
        *evidence_ordinal += 1;
        units.push(EvidenceUnit {
            evidence_unit_key: format!(
                "{}#evidence#{}#{}",
                record.key,
                source_kind.as_str(),
                *evidence_ordinal
            ),
            record_key: record.key.to_string(),
            source_content_unit_key,
            source_kind,
            visibility,
            unit_kind: "document".to_string(),
            label,
            ordinal: *evidence_ordinal,
            search_text: render_plain_text(document),
            document: document.clone(),
        });
        return;
    }

    for section in sections {
        *evidence_ordinal += 1;
        let evidence_unit_key = if can_map_embedding {
            *embedding_child_ordinal += 1;
            format!(
                "{}#{}:{}",
                record.key,
                EmbeddingUnitKind::HeadingSection.as_str(),
                *embedding_child_ordinal
            )
        } else {
            format!(
                "{}#evidence#{}#{}",
                record.key,
                source_kind.as_str(),
                *evidence_ordinal
            )
        };
        units.push(EvidenceUnit {
            evidence_unit_key,
            record_key: record.key.to_string(),
            source_content_unit_key: source_content_unit_key.clone(),
            source_kind,
            visibility,
            unit_kind: EmbeddingUnitKind::HeadingSection.as_str().to_string(),
            label: Some(section.label),
            ordinal: *evidence_ordinal,
            search_text: section.search_text,
            document: section.document,
        });
    }
}

#[derive(Debug, Clone)]
struct EvidenceSection {
    label: String,
    search_text: String,
    document: ContentDocument,
}

fn explicit_heading_evidence_sections(document: &ContentDocument) -> Vec<EvidenceSection> {
    let tree = build_content_section_tree(document);
    let mut sections = Vec::new();
    collect_explicit_heading_evidence_sections(&tree, &mut sections);
    sections
}

fn collect_explicit_heading_evidence_sections(
    node: &ContentSectionNode,
    sections: &mut Vec<EvidenceSection>,
) {
    let document = ContentDocument::new(node.source_blocks.clone());
    let search_text = render_plain_text(&document);
    if !search_text.trim().is_empty()
        && node.origin == ContentSectionOrigin::ExplicitHeading
        && let Some(label) = node.title.as_deref()
    {
        sections.push(EvidenceSection {
            label: label.to_string(),
            search_text,
            document,
        });
    }
    for child in &node.children {
        collect_explicit_heading_evidence_sections(child, sections);
    }
}
