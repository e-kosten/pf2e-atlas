//! Consuming projection from admitted records into explicit artifact inputs.
use crate::{EnrichedDocumentOutcome, EnrichedFoundrySource, IngestError};
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use atlas_embedding::{
    IDENTITY_TOKEN_BUDGET, PassageSection, PreparedEmbeddingInput, TextEmbeddingTokenizer,
    hash_document_embedding_input, prepare_embedding_section,
};
use atlas_index::{
    IndexBuildInput, IndexBuildPack, SourceArtifactBuildContext, SourceArtifactDiagnostic,
    SourceArtifactRecordInput, SourceLexicalUnitInput, SourceLexicalUnitKind,
    SourceSemanticUnitInput,
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
    pub(crate) context_shortened: usize,
}
pub(crate) fn index_build_input(
    source: EnrichedFoundrySource,
    context: SourceArtifactBuildContext,
    tokenizer: Option<&TextEmbeddingTokenizer>,
) -> Result<PreparedBuildInput, IngestError> {
    let source_root = source.metadata.source_root.clone();
    let mut selections = Vec::new();
    let context_hash = context
        .preparation_sha256()
        .map_err(|e| IngestError::ArtifactWriteFailed(e.to_string()))?;
    let mut output = PreparedBuildInput {
        index: IndexBuildInput {
            records: vec![],
            packs: vec![],
            context,
            aliases: vec![],
            remaster_pairs: vec![],
            lexical_units: vec![],
            semantic_units: vec![],
        },
        pending: vec![],
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
            lexical_units(&selection, &mut output.index.lexical_units);
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
    crate::source_aliases::extract_source_aliases(&mut output.index, &source_root)?;
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
                tracing::info!(target:"atlas_progress",phase="prepare_search",current=(ordinal+1) as u64,total=total as u64,semantic_units=output.pending.len(),record=%selection.root,"Preparing attributable search passages: {}/{} records, {} semantic units",ordinal+1,total,output.pending.len());
            }
        }
    }
    Ok(output)
}
fn lexical_units(selection: &SourceSearchSelection, output: &mut Vec<SourceLexicalUnitInput>) {
    if selection.identities.is_empty() {
        return;
    }
    let root = selection
        .identities
        .iter()
        .find(|identity| identity.owners.is_empty());
    if let Some(root) = root {
        output.push(SourceLexicalUnitInput {
            record: selection.root.clone(),
            owners: vec![],
            field: None,
            address: None,
            kind: SourceLexicalUnitKind::RootName,
            identity_terms: root.name.clone(),
            alias_terms: String::new(),
            structured_terms: root.vocabulary.join(" "),
            definition_terms: String::new(),
        });
    }
    for identity in selection
        .identities
        .iter()
        .filter(|identity| !identity.owners.is_empty())
    {
        output.push(SourceLexicalUnitInput {
            record: selection.root.clone(),
            owners: identity.owners.clone(),
            field: None,
            address: None,
            kind: SourceLexicalUnitKind::OwnedName,
            identity_terms: root.map(|root| root.name.clone()).unwrap_or_default(),
            alias_terms: String::new(),
            structured_terms: identity.vocabulary.join(" "),
            definition_terms: identity.name.clone(),
        });
    }
    for field in &selection.fields {
        for section in field
            .sections
            .iter()
            .filter(|section| section.is_lexical_definition())
        {
            let kind = match section.kind {
                atlas_record::source_record::SourceSelectedSectionKind::Heading => {
                    SourceLexicalUnitKind::Heading
                }
                atlas_record::source_record::SourceSelectedSectionKind::DefinitionLabel => {
                    SourceLexicalUnitKind::DefinitionLabel
                }
                atlas_record::source_record::SourceSelectedSectionKind::Unlabeled => continue,
            };
            let address = field.prepared_html_sha256.as_ref().map(|html_hash| {
                SourcePassageAddress::HtmlSection {
                    prepared_html_sha256: html_hash.clone(),
                    canonical_text_sha256: hash_document_embedding_input(&section.text),
                    selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                    section_ordinal: section.section_ordinal,
                    label: section.label.clone(),
                    chunk_bytes: SourceByteRange {
                        start: 0,
                        end: section.text.len(),
                    },
                }
            });
            output.push(SourceLexicalUnitInput {
                record: selection.root.clone(),
                owners: field.locator.owners.clone(),
                field: Some(field.locator.field.clone()),
                address,
                kind,
                identity_terms: root.map(|root| root.name.clone()).unwrap_or_default(),
                alias_terms: String::new(),
                structured_terms: root
                    .map(|root| root.vocabulary.join(" "))
                    .unwrap_or_default(),
                definition_terms: section.label.clone().unwrap_or_default(),
            });
        }
    }
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
/// Shared segmentation has finished; apply only the selected native final guard/identity.
pub(crate) fn profile_pending(
    mut pending: PendingSemanticUnit,
    tokenizer: &TextEmbeddingTokenizer,
) -> Result<PendingSemanticUnit, IngestError> {
    let model = atlas_embedding::embedding_model_for_model_id(tokenizer.spec().model_id)
        .ok_or_else(|| IngestError::DocumentEmbeddingFailed("unsupported profile model".into()))?;
    pending.input.token_count = tokenizer
        .validate_document_input(&pending.input.input)
        .map_err(embedding_error)?;
    pending.input.reuse_key = tokenizer.spec().document_reuse_key(&pending.input.input);
    pending.unit.profile_id = atlas_embedding::profile_for_model(model).profile_id();
    pending.unit.input_token_count = pending.input.token_count;
    Ok(pending)
}
fn embedding_error(error: atlas_embedding::EmbeddingError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(error.to_string())
}
