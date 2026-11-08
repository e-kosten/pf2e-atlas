//! Selected-locale trait vocabulary.
use crate::{IngestError, source::localization::LocalizationCatalog};
use atlas_domain::TraitKey;
use atlas_index::{ReferenceSourceLocation, TraitDefinition};
use atlas_record::source_content::LocalizationResolver;
pub(crate) fn prepare_traits_stream(
    catalog: &LocalizationCatalog,
    context: &atlas_index::SourceArtifactBuildContext,
    reference_index: &atlas_record::source_record::SourceReferenceIndex,
    emit: &mut dyn FnMut(TraitDefinition) -> Result<(), IngestError>,
) -> Result<(), IngestError> {
    let resolve = |kind: &atlas_record::source_content::ContentReferenceKind, target: &str| {
        reference_index.resolve_global_content_reference(kind, target)
    };
    for authored in atlas_foundry_model::generated::trait_definitions() {
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
                atlas_record::source_content::prepare_reference_definition(
                    key.clone().into(),
                    markup,
                    context.audience,
                    Some(catalog),
                    Some(&resolve),
                )
            })
            .transpose()
            .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?;
        emit(TraitDefinition {
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
                .map(|s| ReferenceSourceLocation {
                    file: s.file.into(),
                    line: s.line,
                    column: s.column,
                })
                .collect(),
            source_revision: context.source_revision.clone(),
            indexing_locale: context.indexing_locale.clone(),
            content_locale: authored
                .description_key
                .or(authored.label_key)
                .and_then(|key| catalog.content_locale(key))
                .map(str::to_owned),
        })?;
    }
    Ok(())
}
