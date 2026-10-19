use atlas_cli_support::write_json_data;
use atlas_embedding::EmbeddingRuntimeConfig;
use atlas_ingest::{BuildArtifactOptions, build_artifact};
use atlas_runtime::{
    AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, SetupTarget,
};
use std::path::PathBuf;
use std::process::ExitCode;
pub(crate) mod args;
use args::{BuildIndexOptions, CheckIndexOptions, ValidateIndexOptions};

pub(crate) fn run_index_build(options: BuildIndexOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.output,
        },
    })
    .map_err(|e| e.to_string())?;
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
    let paths = runtime.paths();
    let workspace =
        atlas_runtime::initialize_workspace(&paths.index_path).map_err(|e| e.to_string())?;
    let report = build_artifact(
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
            reuse_embeddings: !options.no_reuse_embeddings,
            embedding_batch_size: options.embedding_batch_size,
        },
    )
    .map_err(|e| e.to_string())?;
    if options.json {
        write_json_data(
            serde_json::json!({"output_path":report.output_path,"record_count":report.record_count,"product_record_count":report.product_record_count,"pack_count":report.pack_count,"semantic_unit_count":report.semantic_unit_count,"search_profiles":report.profiles,"inferred_inputs":report.inferred_inputs,"reused_inputs":report.reused_inputs,"context_shortened_sections":report.context_shortened_sections,"source_fingerprint":report.source_fingerprint,"build_duration_ms":report.build_duration_ms}),
        )?;
    } else {
        println!(
            "ok: wrote {} records ({} products) from {} packs to {}",
            report.record_count,
            report.product_record_count,
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
        println!("source fingerprint: {}", report.source_fingerprint);
        println!(
            "duration: {}",
            atlas_cli_support::format_duration_ms(report.build_duration_ms)
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_index_check(options: CheckIndexOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime
        .check_index_report(if options.no_embeddings {
            SetupTarget::Records
        } else {
            SetupTarget::Full
        })
        .map_err(|e| e.to_string())?;
    if options.json {
        write_json_data(&report)?;
    } else {
        println!(
            "ok: {} records, {} products, {} lexical units, {} semantic units",
            report.records, report.product_records, report.lexical_units, report.semantic_units
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_index_validate(options: ValidateIndexOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime
        .validate_index_report(if options.no_embeddings {
            SetupTarget::Records
        } else {
            SetupTarget::Full
        })
        .map_err(|e| e.to_string())?;
    crate::output::write_validation_report(report, options.json)
}
fn index_runtime(path_mode: AtlasPathMode, index: Option<PathBuf>) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index,
        },
    })
    .map_err(|e| e.to_string())
}
