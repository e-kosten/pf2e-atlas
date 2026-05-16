use atlas_record::{
    PresentationBlock, PresentationSection, PresentationSectionKind, PresentationText,
    RecordPresentationDocument,
};

use crate::document_input::hash_document_embedding_input;
use crate::document_renderer::{
    EmbeddingInputChunk, EmbeddingInputSection, render_embedding_chunks_for_embedding,
    render_presentation_document_embedding_chunks,
};
use crate::structured_units::{
    StructuredEmbeddingUnit, extract_structured_embedding_units, strip_markup_for_embedding_units,
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
    let raw_description = source.source_description_markup.as_deref();
    let compact_description = raw_description
        .map(strip_markup_for_embedding_units)
        .filter(|value| !value.trim().is_empty());
    let document = document_with_description_override(&source.document, compact_description);
    let parent_chunks = document_embedding_input_chunks(&document, &source.aliases);
    let mut units = vec![pending_embedding_unit(
        format!("{}#parent", source.record_key),
        source.record_key.clone(),
        EmbeddingUnitKind::Parent,
        None,
        0,
        parent_chunks,
    )];

    if let Some(markup) = raw_description {
        let child_units: Vec<StructuredEmbeddingUnit> =
            extract_structured_embedding_units(&source.record_name, markup);
        let context = child_embedding_context_chunks(&document);
        for child in child_units {
            let mut chunks = context.clone();
            chunks.push(EmbeddingInputChunk::line(
                EmbeddingInputSection::Description,
                format!("Section: {}", child.label),
            ));
            chunks.push(EmbeddingInputChunk::truncatable_line(
                EmbeddingInputSection::Description,
                format!("Description: {}", child.body),
            ));
            units.push(pending_embedding_unit(
                format!(
                    "{}#{}:{}",
                    source.record_key,
                    child.kind.as_str(),
                    child.ordinal
                ),
                source.record_key.clone(),
                child.kind,
                Some(child.label),
                child.ordinal,
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

fn document_with_description_override(
    document: &RecordPresentationDocument,
    description: Option<String>,
) -> RecordPresentationDocument {
    let Some(description) = description else {
        return document.clone();
    };
    let mut document = document.clone();
    document
        .sections
        .retain(|section| section.kind != PresentationSectionKind::Description);
    document.sections.push(PresentationSection::new(
        PresentationSectionKind::Description,
        vec![PresentationBlock::Prose(PresentationText {
            text: description,
        })],
    ));
    document
}
