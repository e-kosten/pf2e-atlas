//! Build orchestration: source admission, preparation, inference, atomic publication.
use crate::{
    BuildArtifactOptions, BuildArtifactReport, IngestError, SourceEnrichmentContext,
    compute_source_fingerprint, enrich_loaded_source, load_foundry_documents,
    source_git_commit_if_clean,
};
use crate::{index_build_input::index_build_input, source::localization::LocalizationCatalog};
use atlas_embedding::{TextEmbedder, TextEmbeddingTokenizer, generate_prepared_embeddings};
use atlas_index::{
    ARTIFACT_FORMAT_VERSION, ASSET_POLICY_VERSION, IndexArtifactWriter, LEXICAL_SELECTION_VERSION,
    LOCALIZATION_POLICY_VERSION, RELATIONSHIP_POLICY_VERSION, SOURCE_LOOKUP_VERSION,
    SOURCE_PROJECTION_VERSION, SourceArtifactBuildContext, SourceSemanticModelIdentity,
    SqliteIndexWriter,
};
use atlas_record::{
    source_content::{CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentVisibilityRule},
    source_record::SOURCE_CONTENT_SELECTION_VERSION,
};
use std::{collections::BTreeMap, time::Instant};

pub(crate) fn default_audience() -> ContentAudience {
    ContentAudience {
        include_gm: true,
        include_owner: true,
        implicit_check_dc: ContentVisibilityRule::Gm,
    }
}
fn progress(phase: &'static str, message: &str) {
    tracing::info!(target:"atlas_progress",phase,"{message}");
}
pub(crate) struct PreparedSourceBuild {
    pub(crate) prepared: crate::index_build_input::PreparedBuildInput,
    pub(crate) source_report: crate::SourceEnrichmentReport,
    pub(crate) fingerprint: crate::SourceFingerprint,
    pub(crate) catalog: LocalizationCatalog,
}
pub(crate) fn prepare_build(
    options: &BuildArtifactOptions,
) -> Result<PreparedSourceBuild, IngestError> {
    progress("load_source", "Loading typed Foundry source");
    let fingerprint =
        compute_source_fingerprint(&options.source_root, options.manifest_path.as_deref())?;
    let catalog = LocalizationCatalog::load(&options.source_root, &options.locale)?;
    let loaded = load_foundry_documents(&options.source_root, options.manifest_path.as_deref())?;
    let source = enrich_loaded_source(
        loaded,
        SourceEnrichmentContext {
            audience: default_audience(),
            localization: Some(&catalog),
        },
    );
    let source_report = source.report();
    if source_report.addressed_documents == 0 {
        return Err(IngestError::NoRecordsLoaded);
    }
    let semantic_model = options
        .embedding
        .as_ref()
        .map(|_| SourceSemanticModelIdentity::current());
    let context = SourceArtifactBuildContext {
        used_trait_labels: crate::source::localization::used_trait_labels(&source, &catalog)?,
        format_version: ARTIFACT_FORMAT_VERSION,
        snapshot_version: atlas_foundry_model::SNAPSHOT_VERSION,
        source_contract: atlas_foundry_model::SOURCE_CONTRACT_ID.into(),
        source_revision: source_git_commit_if_clean(&options.source_root).ok(),
        source_fingerprint: fingerprint.value.clone(),
        source_file_count: fingerprint.file_count,
        indexing_locale: catalog.locale.clone(),
        locale_catalog_sha256: catalog.locale_sha256.clone(),
        english_catalog_sha256: catalog.english_sha256.clone(),
        localization_policy: LOCALIZATION_POLICY_VERSION.into(),
        audience: default_audience(),
        content_interpretation_version: CONTENT_INTERPRETATION_VERSION.into(),
        content_selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
        asset_policy: ASSET_POLICY_VERSION.into(),
        relationship_policy: RELATIONSHIP_POLICY_VERSION.into(),
        query_projection_version: SOURCE_PROJECTION_VERSION.into(),
        query_catalog_version: atlas_index::query::QUERY_CAPABILITY_VERSION,
        lookup_version: SOURCE_LOOKUP_VERSION.into(),
        lexical_selection_version: LEXICAL_SELECTION_VERSION.into(),
        fts_tokenizer: "unicode61 remove_diacritics 2".into(),
        semantic_model,
    };
    progress("prepare_search", "Preparing attributable search passages");
    let tokenizer = options
        .embedding
        .as_ref()
        .map(TextEmbeddingTokenizer::load)
        .transpose()
        .map_err(embedding_error)?;
    let prepared = index_build_input(source, context, tokenizer.as_ref())?;
    Ok(PreparedSourceBuild {
        prepared,
        source_report,
        fingerprint,
        catalog,
    })
}
pub(crate) fn build_artifact(
    options: BuildArtifactOptions,
) -> Result<BuildArtifactReport, IngestError> {
    let started = Instant::now();
    let PreparedSourceBuild {
        mut prepared,
        source_report,
        fingerprint,
        catalog,
    } = prepare_build(&options)?;
    let mut inferred_inputs = 0;
    let mut reused_inputs = 0;
    if let Some(config) = &options.embedding {
        progress("load_embeddings", "Loading verified local embedding model");
        let mut embedder = TextEmbedder::load(config).map_err(embedding_error)?;
        let inputs = prepared
            .pending
            .iter()
            .map(|pending| pending.input.clone())
            .collect::<Vec<_>>();
        let mut reusable = BTreeMap::new();
        if options.reuse_embeddings && options.output_path.is_file() {
            match atlas_index::SqliteIndexReader::open_read_only(&options.output_path) {
                Ok(reader) => {
                    let model =
                        prepared
                            .index
                            .context
                            .semantic_model
                            .as_ref()
                            .ok_or_else(|| {
                                IngestError::DocumentEmbeddingFailed(
                                    "semantic model identity is unavailable".into(),
                                )
                            })?;
                    let tokenizer =
                        TextEmbeddingTokenizer::load(config).map_err(embedding_error)?;
                    reusable = crate::source_embedding_reuse::verified_reusable_vectors(
                        &reader, &inputs, &tokenizer, model,
                    )?;
                }
                Err(error) => {
                    tracing::warn!(reason=%error,"existing artifact is unavailable for reuse; all inputs require inference")
                }
            }
        }
        progress("embed_source", "Embedding complete source passages");
        let generated = generate_prepared_embeddings(
            &mut embedder,
            &inputs,
            &reusable,
            options.embedding_batch_size,
        )
        .map_err(embedding_error)?;
        inferred_inputs = generated.inferred_inputs;
        reused_inputs = generated.reused_inputs;
        for (pending, vector) in prepared.pending.into_iter().zip(generated.vectors) {
            let mut unit = pending.unit;
            unit.vector = vector;
            prepared.index.semantic_units.push(unit);
        }
    }
    // Fail before touching the output if source changes while preparation runs.
    let final_fingerprint =
        compute_source_fingerprint(&options.source_root, options.manifest_path.as_deref())?;
    let final_catalog = LocalizationCatalog::load(&options.source_root, &options.locale)?;
    if fingerprint != final_fingerprint
        || catalog.locale_sha256 != final_catalog.locale_sha256
        || catalog.english_sha256 != final_catalog.english_sha256
    {
        return Err(IngestError::SourceUnavailable(
            "source changed during artifact preparation; retry build".into(),
        ));
    }
    progress(
        "write_artifact",
        "Validating and publishing the source-backed artifact",
    );
    SqliteIndexWriter::new(options.output_path.clone())
        .write(&prepared.index)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let report = BuildArtifactReport {
        output_path: options.output_path,
        pack_count: prepared.index.packs.len(),
        record_count: prepared.index.records.len(),
        product_record_count: prepared
            .index
            .records
            .iter()
            .filter(|record| {
                !matches!(
                    record.record.source(),
                    atlas_foundry_model::FoundryDocumentSource::Macro(_)
                )
            })
            .count(),
        semantic_unit_count: prepared.index.semantic_units.len(),
        inferred_inputs,
        reused_inputs,
        context_shortened_sections: prepared.context_shortened,
        source_fingerprint: fingerprint.value,
        build_duration_ms: started.elapsed().as_millis(),
        source_report,
    };
    tracing::info!(target:"atlas_progress",complete=true,"artifact build complete");
    Ok(report)
}
fn embedding_error(error: atlas_embedding::EmbeddingError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(error.to_string())
}
