use atlas_record::{
    ContentDocument, ContentSectionNode, build_content_section_tree, render_plain_text,
};

use crate::document_input::hash_document_embedding_input;
use crate::document_renderer::{
    EmbeddingInputChunk, EmbeddingInputSection, render_embedding_chunks_for_embedding,
    render_presentation_document_embedding_chunks,
};
use crate::unit_kind::EmbeddingUnitKind;

use super::model::{DocumentEmbeddingSource, PendingDocumentEmbedding};

pub fn build_document_embedding_units(
    sources: &[DocumentEmbeddingSource],
) -> Vec<PendingDocumentEmbedding> {
    sources
        .iter()
        .flat_map(build_record_embedding_units)
        .collect()
}

fn build_record_embedding_units(source: &DocumentEmbeddingSource) -> Vec<PendingDocumentEmbedding> {
    let parent_chunks = document_embedding_input_chunks(&source.document, &source.aliases);
    let mut units = vec![pending_embedding_unit(
        format!("{}#parent", source.record_key),
        source.record_key.clone(),
        EmbeddingUnitKind::Parent,
        None,
        0,
        parent_chunks,
    )];

    let context = child_embedding_context_chunks(&source.document);
    let mut ordinal = 0;
    for content_source in &source.content_documents {
        let tree = build_content_section_tree(&content_source.document);
        for section in content_tree_embedding_sections(&tree) {
            ordinal += 1;
            let mut chunks = context.clone();
            chunks.push(EmbeddingInputChunk::line(
                EmbeddingInputSection::Description,
                format!(
                    "Section: {} ({})",
                    section.label,
                    content_source.source_kind.as_str()
                ),
            ));
            chunks.push(EmbeddingInputChunk::truncatable_line(
                EmbeddingInputSection::Description,
                format!("Description: {}", section.body),
            ));
            units.push(pending_embedding_unit(
                format!(
                    "{}#{}:{}",
                    source.record_key,
                    section.kind.as_str(),
                    ordinal
                ),
                source.record_key.clone(),
                section.kind,
                Some(section.label),
                ordinal,
                chunks,
            ));
        }
    }

    units
}

pub(super) fn pending_embedding_unit(
    embedding_unit_key: String,
    record_key: String,
    unit_kind: EmbeddingUnitKind,
    label: Option<String>,
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
        ordinal,
        input_chunks,
        input_text,
        input_hash,
    }
}

fn document_embedding_input_chunks(
    document: &atlas_record::RecordPresentationDocument,
    aliases: &[String],
) -> Vec<EmbeddingInputChunk> {
    let mut chunks = render_presentation_document_embedding_chunks(document);
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
struct ContentTreeEmbeddingSection {
    kind: EmbeddingUnitKind,
    label: String,
    body: String,
}

fn content_tree_embedding_sections(root: &ContentSectionNode) -> Vec<ContentTreeEmbeddingSection> {
    let mut sections = Vec::new();
    collect_content_tree_embedding_sections(root, &mut sections);
    sections
}

fn collect_content_tree_embedding_sections(
    node: &ContentSectionNode,
    sections: &mut Vec<ContentTreeEmbeddingSection>,
) {
    let body = render_plain_text(&ContentDocument::new(node.blocks.clone()));
    if !body.trim().is_empty()
        && let Some(label) = node.title.as_deref()
    {
        sections.push(ContentTreeEmbeddingSection {
            kind: EmbeddingUnitKind::HeadingSection,
            label: label.to_string(),
            body,
        });
    }
    for child in &node.children {
        collect_content_tree_embedding_sections(child, sections);
    }
}
