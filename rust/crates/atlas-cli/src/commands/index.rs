use std::path::PathBuf;
use std::process::ExitCode;

use atlas_ingest::{
    BuildArtifactOptions, analyze_foundry_source, build_artifact, build_artifact_json,
};
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};

use crate::output::write_validation_report;
use crate::{AnalyzeIndexOptions, BuildIndexOptions, IndexPathOptions};

pub(crate) fn run_index_analyze(options: AnalyzeIndexOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: None,
            index_path: None,
        },
    })?;
    let paths = runtime.paths();
    let report = analyze_foundry_source(&paths.source_root, options.manifest.as_deref())
        .map_err(|error| error.to_string())?;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "ok: analyzed {} records from {} packs in {}",
            report.record_count, report.pack_count, report.source.root
        );
        println!("source signature: {}", report.source.source_signature);
        println!(
            "records: source={} generated={} default_visible={} hidden={}",
            report.loaded_source_record_count,
            report.generated_record_count,
            report.default_visible_record_count,
            report.hidden_record_count
        );
        println!(
            "relationships: references={} aliases={} remaster_links={}",
            report.relationships.reference_edges,
            report.relationships.record_aliases,
            report.relationships.remaster_links
        );
        println!(
            "dropped inline macros: {}",
            report
                .diagnostics
                .get("dropped_inline_macros")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len)
        );
    }

    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_index_build(options: BuildIndexOptions) -> Result<ExitCode, String> {
    let no_embeddings = options.no_embeddings;
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.output,
        },
    })?;
    let paths = runtime.paths();
    let report = build_artifact(BuildArtifactOptions {
        source_root: paths.source_root.clone(),
        output_path: paths.index_path.clone(),
        manifest_path: options.manifest,
        embedding_model_id: options.embedding_model.to_string(),
        embedding_cache_root: if no_embeddings {
            None
        } else {
            Some(paths.embedding_cache_root.clone())
        },
        reuse_embeddings: !options.no_reuse_embeddings,
        embedding_batch_size: options.embedding_batch_size,
    })
    .map_err(|error| error.to_string())?;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&build_artifact_json(&report))
                .map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "ok: wrote {} records from {} packs to {}",
            report.artifact_record_count,
            report.pack_count,
            report.output_path.display()
        );
        eprintln!(
            "records: source={} generated={} artifact={}",
            report.source_record_count, report.generated_record_count, report.artifact_record_count
        );
        eprintln!(
            "embeddings: pending_document={} document={} reused={} generated={} truncated={} max_tokens={} max_observed_tokens={} build_duration_ms={}",
            report.pending_document_embedding_count,
            report.document_embedding_count,
            report.reused_document_embedding_count,
            report.generated_document_embedding_count,
            report
                .document_embedding_tokenization
                .truncated_document_count,
            report
                .document_embedding_tokenization
                .max_token_count
                .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
            report
                .document_embedding_tokenization
                .max_observed_token_count,
            report.build_duration_ms
        );
        eprintln!("source signature: {}", report.source_signature);
        eprintln!(
            "diagnostics: taxonomy folder={} glossary={} variants parenthetical={} suffix={} creature_blurb={} creature_suffix={} exact_base={}",
            report.diagnostics.taxonomy_folder_records,
            report.diagnostics.taxonomy_glossary_records,
            report.diagnostics.variant_parenthetical_records,
            report.diagnostics.variant_suffix_records,
            report.diagnostics.variant_creature_blurb_records,
            report.diagnostics.variant_creature_suffix_records,
            report.diagnostics.variant_exact_base_records
        );
        eprintln!(
            "generated afflictions: canonical={} instances={} edges={}",
            report.diagnostics.generated_affliction_canonical_records,
            report.diagnostics.generated_affliction_instance_records,
            report.diagnostics.generated_affliction_reference_edges
        );
        if !report.diagnostics.dropped_inline_macros.is_empty() {
            let dropped_inline_macros = report
                .diagnostics
                .dropped_inline_macros
                .iter()
                .map(|(name, diagnostic)| format!("{name}={}", diagnostic.count))
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!("dropped inline macros: {dropped_inline_macros}");
        }
        for skipped_record in &report.skipped_records {
            eprintln!(
                "skipped record: {}: {}",
                skipped_record.path.display(),
                skipped_record.reason
            );
        }
    }

    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_index_inspect(options: IndexPathOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime
        .open_index()
        .map_err(|error| error.to_string())?
        .inspect()
        .map_err(|error| error.to_string())?;

    if options.json {
        let body = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
        println!("{body}");
    } else {
        println!(
            "ok: inspected {} records in {}",
            report.records.total_records, report.index
        );
        println!(
            "tables: records={} packs={} references={} aliases={} remaster_links={}",
            report.records_table_count(),
            report.packs_table_count(),
            report.reference_edges_table_count(),
            report.record_aliases_table_count(),
            report.remaster_links_table_count()
        );
        println!(
            "coverage: taxonomy_records={} variant_records={} descriptions={} blurbs={}",
            report.taxonomy.records_with_taxonomy_families,
            report.variants.grouped_records,
            report.text.records_with_description,
            report.text.records_with_blurb
        );
    }

    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_index_validate(options: IndexPathOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime.validate_index_report();
    write_validation_report(report, options.json)
}

pub(crate) fn run_index_validate_vectors(options: IndexPathOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime.validate_vector_index_report();
    write_validation_report(report, options.json)
}

fn index_runtime(path_mode: AtlasPathMode, index: Option<PathBuf>) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index,
        },
    })
}
