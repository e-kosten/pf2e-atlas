//! Selected-locale trait vocabulary and complete, checkpointed definition seeds.
use crate::{IngestError, source::localization::LocalizationCatalog};
use atlas_domain::TraitKey;
use atlas_embedding::{
    PassageSection, PreparedEmbeddingInput, TextEmbeddingTokenizer, prepare_embedding_section,
};
use atlas_index::{TraitDefinition, TraitSourceLocation};
use atlas_record::source_content::LocalizationResolver;
pub(crate) const TRAIT_SEED_RECIPE: &str = "trait-definition-centroid/v1";
pub(crate) fn prepare_traits_stream(
    catalog: &LocalizationCatalog,
    context: &atlas_index::SourceArtifactBuildContext,
    tokenizer: Option<&TextEmbeddingTokenizer>,
    reference_index: &atlas_record::source_record::SourceReferenceIndex,
    emit: &mut dyn FnMut(TraitDefinition, Vec<PreparedEmbeddingInput>) -> Result<(), IngestError>,
) -> Result<(), IngestError> {
    let resolve = |kind: &atlas_record::source_content::ContentReferenceKind, target: &str| {
        reference_index.resolve_global_content_reference(kind, target)
    };
    for authored in atlas_foundry_model::generated::trait_definitions() {
        let mut pending = Vec::new();
        let key = TraitKey::new(authored.identifier)
            .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?;
        let label = authored
            .label_key
            .and_then(|k| catalog.localized_value(k))
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(authored.identifier)
            .to_owned();
        let markup = authored
            .description_key
            .and_then(|k| catalog.localized_value(k))
            .filter(|v| !v.trim().is_empty());
        let prepared = markup
            .map(|markup| {
                atlas_record::source_content::prepare_trait_definition(
                    key.clone(),
                    markup,
                    context.audience,
                    Some(catalog),
                    Some(&resolve),
                )
            })
            .transpose()
            .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?;
        if let Some(tokenizer) = tokenizer {
            // Plain passage selection retains every definition byte under native budgets.
            let text = format!(
                "{label}\n{}",
                prepared.as_ref().map(|p| p.text.as_str()).unwrap_or("")
            );
            let sections = prepare_embedding_section(
                tokenizer,
                PassageSection::Plain {
                    text: &text,
                    context: &label,
                    selection_version: TRAIT_SEED_RECIPE,
                    section_ordinal: 0,
                    label: Some(&label),
                },
            )
            .map_err(|e| IngestError::DocumentEmbeddingFailed(e.to_string()))?;
            for input in sections {
                pending.push(input);
            }
        }
        emit(
            TraitDefinition {
                key,
                label,
                label_key: authored.label_key.map(str::to_owned),
                description_key: authored.description_key.map(str::to_owned),
                description_html: prepared.as_ref().map(|p| p.html.clone()),
                authored_markup: markup.map(str::to_owned),
                prepared,
                catalogs: authored.catalogs.iter().map(|s| (*s).to_owned()).collect(),
                source_locations: authored
                    .source_locations
                    .iter()
                    .map(|s| TraitSourceLocation {
                        file: s.file.into(),
                        line: s.line,
                        column: s.column,
                    })
                    .collect(),
                source_revision: context.source_revision.clone(),
                indexing_locale: context.indexing_locale.clone(),
            },
            pending,
        )?;
    }
    Ok(())
}
