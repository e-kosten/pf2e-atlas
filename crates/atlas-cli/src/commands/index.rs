use atlas_cli_support::write_json_data;
use atlas_embedding::EmbeddingRuntimeConfig;
use atlas_ingest::{BuildArtifactOptions, build_artifact_with_revision};
use atlas_runtime::{
    AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, SetupTarget,
};
use std::path::PathBuf;
use std::process::ExitCode;
pub(crate) mod args;
use args::{BuildIndexOptions, CheckIndexOptions, RebuildIndexOptions, ValidateIndexOptions};

pub(crate) fn run_index_build(options: BuildIndexOptions) -> Result<ExitCode, String> {
    let json = options.json;
    match build_index(options) {
        Ok(report) => write_build_report(report, json),
        Err(atlas_runtime::ReferenceBuildError::InvalidInput(message)) => {
            super::product::command_error(json, "invalid_input", message, 2)
        }
        Err(atlas_runtime::ReferenceBuildError::Environment(message)) => {
            super::product::command_error(json, "index_build_failed", message, 3)
        }
    }
}

fn build_index(
    options: BuildIndexOptions,
) -> Result<atlas_ingest::BuildArtifactReport, atlas_runtime::ReferenceBuildError> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
        path_mode: crate::cli::runtime_path_mode(options.path_mode),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.output,
        },
    })
    .map_err(|e| e.to_string())?;
    let workspace = runtime
        .workspace_location()
        .map_err(|error| error.to_string())?;
    let _setup = workspace.setup_guard().map_err(|error| error.to_string())?;
    let configuration = runtime.configure_reference_build(
        &atlas_runtime::RuntimeSetupOptions {
            target: if options.no_embeddings {
                SetupTarget::Records
            } else {
                SetupTarget::Full
            },
            locale: options.locale,
            search_profiles: options.search_profiles,
            ..Default::default()
        },
        options.manifest.as_deref(),
    )?;
    if options.embedding_batch_size == 0 && !configuration.search_profiles.is_empty() {
        return Err(atlas_runtime::ReferenceBuildError::InvalidInput(
            "Embedding batch size must be greater than zero.".into(),
        ));
    }
    let paths = runtime.paths();
    let workspace =
        atlas_runtime::initialize_workspace(&paths.index_path).map_err(|e| e.to_string())?;
    let revision = runtime.verified_source_revision(&configuration);
    let report = build_artifact_with_revision(
        &workspace,
        atlas_runtime::WORKSPACE_SCHEMA_VERSION,
        BuildArtifactOptions {
            source_root: configuration.source_root,
            output_path: paths.index_path.clone(),
            manifest_path: configuration.manifest_path,
            locale: configuration.indexing_locale,
            search_profiles: configuration
                .search_profiles
                .into_iter()
                .map(|id| {
                    atlas_embedding::search_profile(&id)
                        .map(|profile| {
                            EmbeddingRuntimeConfig::new(profile.model, &paths.embedding_cache_root)
                        })
                        .map_err(|e| e.to_string())
                })
                .collect::<Result<Vec<_>, _>>()?,
            reuse_passage_layouts: true,
            embedding_batch_size: options.embedding_batch_size,
        },
        revision,
    )
    .map_err(|e| e.to_string())?;
    tracing::info!(target:"atlas_progress",complete=true,"Index build finished");
    Ok(report)
}

pub(crate) fn run_index_rebuild(options: RebuildIndexOptions) -> Result<ExitCode, String> {
    let result = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: None,
        path_mode: crate::cli::runtime_path_mode(options.path_mode),
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    })
    .map_err(|e| e.to_string())
    .and_then(|runtime| {
        runtime.rebuild_reference(options.source.as_deref(), options.manifest.as_deref())
    });
    match result {
        Ok(report) => {
            tracing::info!(target:"atlas_progress",complete=true,"Reference rebuild finished");
            write_build_report(report, options.json)
        }
        Err(message) => {
            super::product::command_error(options.json, "index_rebuild_failed", message, 3)
        }
    }
}

fn write_build_report(
    report: atlas_ingest::BuildArtifactReport,
    json: bool,
) -> Result<ExitCode, String> {
    if json {
        write_json_data(
            serde_json::json!({"admission":report.admission,"output_path":report.output_path,"record_count":report.record_count,"product_record_count":report.product_record_count,"pack_count":report.pack_count,"semantic_unit_count":report.semantic_unit_count,"search_profiles":report.profiles,"inferred_inputs":report.inferred_inputs,"reused_inputs":report.reused_inputs,"context_shortened_sections":report.context_shortened_sections,"source_fingerprint":report.source_fingerprint,"build_duration_ms":report.build_duration_ms}),
        )?;
    } else {
        println!(
            "ok: wrote {} records ({} available in Atlas, {} developer-only macros) from {} packs to {}",
            report.record_count,
            report.product_record_count,
            report
                .record_count
                .saturating_sub(report.product_record_count),
            report.pack_count,
            report.output_path.display()
        );
        println!(
            "semantic units={} inferred inputs={} reused inputs={} shortened contexts={}",
            report.semantic_unit_count,
            report.inferred_inputs,
            report.reused_inputs,
            report.context_shortened_sections
        );
        for profile in &report.profiles {
            println!(
                "profile {}: units={} inferred={} reused={}",
                profile.profile_id,
                profile.semantic_units,
                profile.inferred_inputs,
                profile.reused_inputs
            );
        }
        println!(
            "source files: {} discovered, {} admitted, {} partial, {} quarantined, {} excluded",
            report.admission.discovered_files,
            report.admission.admitted_records,
            report.admission.partial_records,
            report.admission.quarantined_files,
            report.admission.excluded_documents
        );
        println!("source fingerprint: {}", report.source_fingerprint);
        println!(
            "duration: {}",
            atlas_cli_support::format_duration_ms(report.build_duration_ms)
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_index_check(options: CheckIndexOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(
        crate::cli::runtime_path_mode(options.path_mode),
        options.index,
    );
    let target = diagnostic_target(options.no_embeddings);
    if options.json {
        let report = runtime.and_then(|runtime| runtime.check_index_report(target));
        crate::output::index_diagnostic(report, true, |_| {})
    } else {
        let counts = runtime.and_then(|runtime| runtime.check_index_counts(target));
        crate::output::index_diagnostic(counts, false, |report| {
            println!(
                "ok: {} records ({} available in Atlas, {} developer-only macros), {} lexical units, {} semantic units",
                report.records,
                report.product_records,
                report.records.saturating_sub(report.product_records),
                report.lexical_units,
                report.semantic_units,
            )
        })
    }
}

pub(crate) fn run_index_validate(options: ValidateIndexOptions) -> Result<ExitCode, String> {
    let report = index_runtime(
        crate::cli::runtime_path_mode(options.path_mode),
        options.index,
    )
    .and_then(|runtime| runtime.validate_index_report(diagnostic_target(options.no_embeddings)));
    crate::output::index_diagnostic(report, options.json, |report| {
        println!(
            "ok: checked {} source snapshots, {} prepared fields, {} projections, {} lexical units, {} semantic units",
            report.checked_snapshots,
            report.prepared_fields,
            report.projection_rows,
            report.lexical_units,
            report.semantic_units,
        )
    })
}

fn diagnostic_target(no_embeddings: bool) -> SetupTarget {
    if no_embeddings {
        SetupTarget::Records
    } else {
        SetupTarget::Full
    }
}

fn index_runtime(
    path_mode: AtlasPathMode,
    index: Option<PathBuf>,
) -> Result<AtlasRuntime, atlas_runtime::IndexDiagnosticError> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index,
        },
    })
    .map_err(|error| {
        atlas_runtime::IndexDiagnosticError::CouldNotRun(Box::new(
            atlas_index::IndexError::Unavailable(error.to_string()),
        ))
    })
}
