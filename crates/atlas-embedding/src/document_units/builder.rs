use atlas_record::{
    ContentSectionNode, ContentSectionOrigin, ContentSourceKind, RichDocument, RichNode,
    build_content_section_tree, render_plain_text,
};

use crate::document_input::hash_document_embedding_input;
use crate::document_renderer::{
    EmbeddingInputChunk, EmbeddingInputSection, render_embedding_chunks_for_embedding,
    render_presentation_document_embedding_chunks,
};
use crate::unit_kind::EmbeddingUnitKind;

use super::model::{
    DocumentEmbeddingContentSource, DocumentEmbeddingSource, PendingDocumentEmbedding,
    PendingDocumentEmbeddingCandidate,
};

pub fn build_document_embedding_units(
    sources: &[DocumentEmbeddingSource],
) -> Vec<PendingDocumentEmbedding> {
    sources
        .iter()
        .flat_map(build_record_embedding_units)
        .collect()
}

fn build_record_embedding_units(source: &DocumentEmbeddingSource) -> Vec<PendingDocumentEmbedding> {
    let content_groups = embedding_content_groups(&source.content_documents);
    let parent_chunks =
        document_embedding_input_chunks(&source.document, &source.aliases, &content_groups);
    let child_candidates = child_embedding_candidates(source, &content_groups);
    let mut parent = pending_embedding_unit(
        format!("{}#parent", source.record_key),
        source.record_key.clone(),
        EmbeddingUnitKind::Parent,
        None,
        None,
        0,
        parent_chunks,
    );
    parent.child_candidates = child_candidates;
    vec![parent]
}

fn child_embedding_candidates(
    source: &DocumentEmbeddingSource,
    content_groups: &[EmbeddingContentGroup],
) -> Vec<PendingDocumentEmbeddingCandidate> {
    let context = child_embedding_context_chunks(&source.document);
    let mut ordinal = 0;
    content_groups
        .iter()
        .map(|group| {
            ordinal += 1;
            let mut chunks = context.clone();
            chunks.extend(group.chunks.clone());
            PendingDocumentEmbeddingCandidate {
                embedding_unit_key: format!(
                    "{}#{}:{}",
                    source.record_key,
                    group.unit_kind.as_str(),
                    ordinal
                ),
                record_key: source.record_key.clone(),
                unit_kind: group.unit_kind,
                label: Some(group.label.clone()),
                source_kind: group.source_kind,
                group_key: group.group_key.clone(),
                ordinal,
                input_chunks: chunks,
            }
        })
        .collect()
}

pub(super) fn pending_embedding_unit(
    embedding_unit_key: String,
    record_key: String,
    unit_kind: EmbeddingUnitKind,
    label: Option<String>,
    source_kind: Option<ContentSourceKind>,
    ordinal: usize,
    input_chunks: Vec<EmbeddingInputChunk>,
) -> PendingDocumentEmbedding {
    let input_text = render_embedding_chunks_for_embedding(&input_chunks);
    let input_hash = hash_document_embedding_input(&input_text);
    PendingDocumentEmbedding {
        embedding_unit_key,
        record_key,
        unit_kind,
        label,
        source_kind,
        ordinal,
        input_chunks,
        input_text,
        input_hash,
        child_candidates: Vec::new(),
    }
}

fn document_embedding_input_chunks(
    document: &atlas_record::RecordPresentationDocument,
    aliases: &[String],
    content_groups: &[EmbeddingContentGroup],
) -> Vec<EmbeddingInputChunk> {
    let mut chunks = render_presentation_document_embedding_chunks(document)
        .into_iter()
        .filter(|chunk| {
            content_groups.is_empty() || chunk.section != EmbeddingInputSection::Description
        })
        .collect::<Vec<_>>();
    chunks.extend(
        content_groups
            .iter()
            .flat_map(|group| group.chunks.iter().cloned()),
    );
    if !aliases.is_empty() {
        chunks.push(EmbeddingInputChunk::truncatable_line(
            EmbeddingInputSection::Aliases,
            format!("Aliases: {}", aliases.join(", ")),
        ));
    }
    chunks
}

fn child_embedding_context_chunks(
    document: &atlas_record::RecordPresentationDocument,
) -> Vec<EmbeddingInputChunk> {
    render_presentation_document_embedding_chunks(document)
        .into_iter()
        .filter(|chunk| {
            matches!(
                chunk.section,
                EmbeddingInputSection::Identity
                    | EmbeddingInputSection::Traits
                    | EmbeddingInputSection::Classification
                    | EmbeddingInputSection::Summary
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EmbeddingContentGroup {
    group_key: String,
    source_kind: ContentSourceKind,
    unit_kind: EmbeddingUnitKind,
    label: String,
    chunks: Vec<EmbeddingInputChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContentTreeEmbeddingSection {
    kind: EmbeddingUnitKind,
    label: String,
    nodes: Vec<RichNode>,
}

fn embedding_content_groups(
    content_sources: &[DocumentEmbeddingContentSource],
) -> Vec<EmbeddingContentGroup> {
    content_sources
        .iter()
        .flat_map(embedding_content_groups_for_source)
        .collect()
}

fn embedding_content_groups_for_source(
    content_source: &DocumentEmbeddingContentSource,
) -> Vec<EmbeddingContentGroup> {
    let tree = build_content_section_tree(&content_source.document);
    let mut sections = content_tree_embedding_sections(content_source, &tree);
    if sections.is_empty() {
        let label = content_source
            .label
            .clone()
            .unwrap_or_else(|| content_source.source_kind.as_str().to_string());
        sections.push(ContentTreeEmbeddingSection {
            kind: EmbeddingUnitKind::HeadingSection,
            label,
            nodes: content_source.document.nodes.clone(),
        });
    }

    sections
        .into_iter()
        .enumerate()
        .filter_map(|(index, section)| {
            let group_key = format!(
                "{}:{}:{}",
                content_source.source_kind.as_str(),
                index,
                stable_group_label(&section.label, index)
            );
            let chunks = content_group_chunks(content_source.source_kind, &group_key, &section);
            (!chunks.is_empty()).then_some(EmbeddingContentGroup {
                group_key: group_key.clone(),
                source_kind: content_source.source_kind,
                unit_kind: section.kind,
                label: section.label,
                chunks,
            })
        })
        .collect()
}

fn content_tree_embedding_sections(
    content_source: &DocumentEmbeddingContentSource,
    root: &ContentSectionNode,
) -> Vec<ContentTreeEmbeddingSection> {
    let mut sections = Vec::new();
    let leading_body = render_plain_text(&RichDocument::new(root.source_nodes.clone()));
    if !leading_body.trim().is_empty() {
        sections.push(ContentTreeEmbeddingSection {
            kind: EmbeddingUnitKind::HeadingSection,
            label: content_source
                .label
                .clone()
                .unwrap_or_else(|| content_source.source_kind.as_str().to_string()),
            nodes: root.source_nodes.clone(),
        });
    }
    collect_content_tree_embedding_sections(root, &mut sections);
    sections
}

fn collect_content_tree_embedding_sections(
    node: &ContentSectionNode,
    sections: &mut Vec<ContentTreeEmbeddingSection>,
) {
    let body = render_content_section_embedding_body(node);
    if !body.trim().is_empty()
        && node.origin == ContentSectionOrigin::ExplicitHeading
        && let Some(label) = node.title.as_deref()
    {
        sections.push(ContentTreeEmbeddingSection {
            kind: EmbeddingUnitKind::HeadingSection,
            label: label.to_string(),
            nodes: node.source_nodes.clone(),
        });
    }
    for child in &node.children {
        collect_content_tree_embedding_sections(child, sections);
    }
}

fn render_content_section_embedding_body(node: &ContentSectionNode) -> String {
    render_plain_text(&RichDocument::new(node.source_nodes.clone()))
}

fn content_group_chunks(
    source_kind: ContentSourceKind,
    group_key: &str,
    section: &ContentTreeEmbeddingSection,
) -> Vec<EmbeddingInputChunk> {
    let mut chunks = Vec::new();
    chunks.push(
        EmbeddingInputChunk::line(
            EmbeddingInputSection::Description,
            format!("Section: {} ({})", section.label, source_kind.as_str()),
        )
        .with_source_kind(source_kind)
        .with_group_key(group_key),
    );
    for node in &section.nodes {
        let text = render_plain_text(&RichDocument::new(vec![node.clone()]));
        if text.trim().is_empty() {
            continue;
        }
        chunks.push(
            EmbeddingInputChunk::truncatable_line(
                EmbeddingInputSection::Description,
                format!("Description: {text}"),
            )
            .with_source_kind(source_kind)
            .with_group_key(group_key),
        );
    }
    chunks
}

fn stable_group_label(label: &str, index: usize) -> String {
    let stable = label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if stable.is_empty() {
        index.to_string()
    } else {
        stable
    }
}
