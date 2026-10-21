//! Consuming projection from admitted records into explicit artifact inputs.
use crate::{EnrichedDocumentOutcome, EnrichedFoundrySource, IngestError};
use atlas_embedding::{
    IDENTITY_TOKEN_BUDGET, PassageSection, PreparedEmbeddingInput, TextEmbeddingTokenizer,
    prepare_embedding_section,
};
use atlas_index::{
    IndexBuildInput, IndexBuildPack, SourceArtifactBuildContext, SourceArtifactDiagnostic,
    SourceArtifactRecordInput, SourceSemanticUnitInput,
};
use atlas_record::source_record::{
    SOURCE_CONTENT_SELECTION_VERSION, SourceSearchSelection, select_record_search,
};

#[derive(Clone)]
pub(crate) struct PendingSemanticUnit {
    pub(crate) input: PreparedEmbeddingInput,
    pub(crate) unit: SourceSemanticUnitInput,
}
pub(crate) struct PreparedBuildInput {
    pub(crate) index: IndexBuildInput,
    pub(crate) pending: Vec<PendingSemanticUnit>,
    pub(crate) trait_pending: Vec<crate::traits::PendingTraitInput>,
    pub(crate) context_shortened: usize,
}
pub(crate) fn index_build_input(
    source: EnrichedFoundrySource,
    context: SourceArtifactBuildContext,
    tokenizer: Option<&TextEmbeddingTokenizer>,
    capture: &crate::source_fingerprint::SourceCapture,
) -> Result<PreparedBuildInput, IngestError> {
    tracing::info!("Selecting search content");
    let source_root = source.metadata.source_root.clone();
    let mut selections = Vec::new();
    let context_hash = context
        .preparation_sha256()
        .map_err(|e| IngestError::ArtifactWriteFailed(e.to_string()))?;
    let mut output = PreparedBuildInput {
        index: IndexBuildInput {
            traits: vec![],
            trait_vectors: vec![],
            records: vec![],
            packs: vec![],
            context,
            aliases: vec![],
            remaster_pairs: vec![],
            lexical_units: vec![],
            semantic_units: vec![],
        },
        pending: vec![],
        trait_pending: vec![],
        context_shortened: 0,
    };
    for pack in source.packs {
        output.index.packs.push(IndexBuildPack {
            pack_id: pack.metadata.name.clone(),
            label: pack.metadata.label.clone(),
            document_kind: pack.metadata.document_type,
            declared_path: pack.metadata.declared_path,
        });
        for document in pack.documents {
            let EnrichedDocumentOutcome::Addressed {
                record,
                content,
                relationships,
            } = document.outcome
            else {
                continue;
            };
            let selection = select_record_search(
                &record,
                &content,
                output.index.context.audience,
                &pack.metadata.label,
                &output.index.context.used_trait_labels,
            )
            .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?;
            selections.push(selection);
            let diagnostics = document
                .admission_diagnostics
                .into_iter()
                .map(|diagnostic| SourceArtifactDiagnostic {
                    owners: None,
                    field: Some(diagnostic.json_path.clone()),
                    stage: "admission".into(),
                    code: "unparseable_source_field".into(),
                    message: diagnostic.to_string(),
                    source_diagnostic: Some(diagnostic),
                })
                .collect();
            output.index.records.push(SourceArtifactRecordInput {
                record: *record,
                source_path: document.provenance.source_path,
                content_hash: document.content_hash,
                preparation_context_hash: context_hash.clone(),
                content,
                relationships,
                diagnostics,
            });
        }
    }
    tracing::info!("Reading alternate record names");
    crate::source_aliases::extract_source_aliases(&mut output.index, &source_root, capture)?;
    for selection in &selections {
        let aliases = output
            .index
            .aliases
            .iter()
            .filter(|alias| alias.record == selection.root)
            .map(|alias| alias.alias.clone())
            .collect::<Vec<_>>();
        output
            .index
            .lexical_units
            .extend(atlas_index::project_lexical_units(selection, &aliases));
    }
    if let Some(tokenizer) = tokenizer {
        let total = selections.len();
        for (ordinal, mut selection) in selections.into_iter().enumerate() {
            selection.identity_optional.extend(
                output
                    .index
                    .aliases
                    .iter()
                    .filter(|alias| alias.record == selection.root)
                    .map(|alias| alias.alias.clone()),
            );
            semantic_units(
                &selection,
                tokenizer,
                &mut output.pending,
                &mut output.context_shortened,
            )?;
            if (ordinal + 1) % 1000 == 0 || ordinal + 1 == total {
                tracing::info!(target:"atlas_progress",phase="prepare_search",current=(ordinal+1) as u64,total=total as u64,unit="records",semantic_units=output.pending.len(),record=%selection.root,"Preparing search content");
            }
        }
    }
    Ok(output)
}
pub(crate) fn semantic_units(
    selection: &SourceSearchSelection,
    tokenizer: &TextEmbeddingTokenizer,
    output: &mut Vec<PendingSemanticUnit>,
    shortened: &mut usize,
) -> Result<(), IngestError> {
    if selection.identities.is_empty() {
        return Ok(());
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
        push_pending(output, selection, vec![], None, 0, input);
    }
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
            for (ordinal, input) in prepare_embedding_section(tokenizer, passage)
                .map_err(embedding_error)?
                .into_iter()
                .enumerate()
            {
                *shortened += usize::from(input.context_shortened);
                push_pending(
                    output,
                    selection,
                    field.locator.owners.clone(),
                    Some(field.locator.field.clone()),
                    ordinal,
                    input,
                );
            }
        }
    }
    Ok(())
}
fn push_pending(
    output: &mut Vec<PendingSemanticUnit>,
    selection: &SourceSearchSelection,
    owners: Vec<atlas_record::source_content::OwnedContentLocator>,
    field: Option<String>,
    chunk_ordinal: usize,
    input: PreparedEmbeddingInput,
) {
    let unit = SourceSemanticUnitInput {
        profile_id: atlas_domain::SearchProfileId::new("bge-base"),
        record: selection.root.clone(),
        owners,
        field,
        address: input.address.clone(),
        chunk_ordinal,
        input_token_count: input.token_count,
        input_hash: input.input_sha256.clone(),
        vector: vec![],
    };
    output.push(PendingSemanticUnit { input, unit });
}
fn embedding_error(error: atlas_embedding::EmbeddingError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(error.to_string())
}
