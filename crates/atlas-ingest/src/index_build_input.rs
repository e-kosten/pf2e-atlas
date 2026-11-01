//! Record-sized passage preparation shared by the bounded product pipeline.
use crate::IngestError;
use atlas_embedding::{
    IDENTITY_TOKEN_BUDGET, PassageSection, PreparedEmbeddingInput, TextEmbeddingTokenizer,
    prepare_embedding_section, reuse_embedding_section,
};
use atlas_record::source_record::{SOURCE_CONTENT_SELECTION_VERSION, SourceSearchSelection};
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DocumentAttribution {
    pub record: atlas_domain::RecordKey,
    pub owners: Vec<atlas_domain::OwnedContentLocator>,
    pub field: Option<String>,
    pub address: atlas_domain::SourcePassageAddress,
    pub chunk_ordinal: usize,
}
pub(crate) fn semantic_units(
    selection: &SourceSearchSelection,
    tokenizer: &TextEmbeddingTokenizer,
    output: &mut dyn FnMut(DocumentAttribution, PreparedEmbeddingInput) -> Result<(), IngestError>,
    shortened: &mut usize,
    layouts: &[atlas_index::SourcePassageLayout],
) -> Result<usize, IngestError> {
    if selection.identities.is_empty() {
        return Ok(0);
    }
    let mut identity = selection.identity_required.clone();
    if tokenizer
        .count_tokens(&identity, false)
        .map_err(embedding_error)?
        > IDENTITY_TOKEN_BUDGET
    {
        return Err(IngestError::DocumentEmbeddingFailed(format!(
            "required root identity exceeds budget: {}",
            selection.root
        )));
    }
    for optional in &selection.identity_optional {
        let candidate = format!("{identity}\n{optional}");
        if tokenizer
            .count_tokens(&candidate, false)
            .map_err(embedding_error)?
            <= IDENTITY_TOKEN_BUDGET
        {
            identity = candidate;
        } else {
            *shortened += 1;
        }
    }
    for input in prepare_embedding_section(tokenizer, PassageSection::Identity { text: &identity })
        .map_err(embedding_error)?
    {
        push_pending(output, selection, vec![], None, 0, input)?;
    }
    let mut reused = 0;
    for field in &selection.fields {
        for section in &field.sections {
            let owner_name = selection
                .identities
                .iter()
                .find(|identity| identity.owners == field.locator.owners)
                .map(|identity| identity.name.as_str())
                .unwrap_or("");
            let context = if field.locator.owners.is_empty() {
                format!(
                    "{}\n{}\n{}\n{}",
                    selection.identity_required,
                    selection.pack_label,
                    owner_name,
                    section.heading_chain.join(" / ")
                )
            } else {
                format!(
                    "{}\n{}\n{}",
                    selection.identity_required,
                    owner_name,
                    section.heading_chain.join(" / ")
                )
            };
            let passage = if let Some(html_hash) = &field.prepared_html_sha256 {
                PassageSection::Html {
                    text: &section.text,
                    context: &context,
                    prepared_html_sha256: html_hash,
                    selection_version: SOURCE_CONTENT_SELECTION_VERSION,
                    section_ordinal: section.section_ordinal,
                    label: section.label.as_deref(),
                }
            } else {
                PassageSection::Plain {
                    text: &section.text,
                    context: &context,
                    selection_version: SOURCE_CONTENT_SELECTION_VERSION,
                    section_ordinal: section.section_ordinal,
                    label: section.label.as_deref(),
                }
            };
            let mut candidates = layouts
                .iter()
                .filter(|layout| {
                    layout.owners == field.locator.owners
                        && layout.field == field.locator.field
                        && match &layout.passage.address {
                            atlas_domain::SourcePassageAddress::HtmlSection {
                                section_ordinal,
                                ..
                            }
                            | atlas_domain::SourcePassageAddress::PlainSection {
                                section_ordinal,
                                ..
                            } => *section_ordinal == section.section_ordinal,
                            atlas_domain::SourcePassageAddress::Identity {} => false,
                        }
                })
                .collect::<Vec<_>>();
            candidates.sort_by_key(|layout| layout.chunk_ordinal);
            let cached = if candidates
                .iter()
                .enumerate()
                .all(|(ordinal, layout)| ordinal == layout.chunk_ordinal)
            {
                reuse_embedding_section(
                    tokenizer,
                    passage,
                    &candidates
                        .iter()
                        .map(|layout| layout.passage.clone())
                        .collect::<Vec<_>>(),
                )
                .map_err(embedding_error)?
            } else {
                None
            };
            let inputs = match cached {
                Some(inputs) => {
                    reused += 1;
                    inputs
                }
                None => prepare_embedding_section(tokenizer, passage).map_err(embedding_error)?,
            };
            for (ordinal, input) in inputs.into_iter().enumerate() {
                *shortened += usize::from(input.context_shortened);
                push_pending(
                    output,
                    selection,
                    field.locator.owners.clone(),
                    Some(field.locator.field.clone()),
                    ordinal,
                    input,
                )?;
            }
        }
    }
    Ok(reused)
}
fn push_pending(
    output: &mut dyn FnMut(DocumentAttribution, PreparedEmbeddingInput) -> Result<(), IngestError>,
    selection: &SourceSearchSelection,
    owners: Vec<atlas_domain::OwnedContentLocator>,
    field: Option<String>,
    chunk_ordinal: usize,
    input: PreparedEmbeddingInput,
) -> Result<(), IngestError> {
    let unit = DocumentAttribution {
        record: selection.root.clone(),
        owners,
        field,
        address: input.address.clone(),
        chunk_ordinal,
    };
    output(unit, input)
}
fn embedding_error(error: atlas_embedding::EmbeddingError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(error.to_string())
}
