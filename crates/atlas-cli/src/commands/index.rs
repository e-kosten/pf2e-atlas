use std::path::PathBuf;
use std::process::ExitCode;

use atlas_index::ValidationTarget;
use atlas_ingest::{
    BuildArtifactOptions, BuildArtifactReport, DocumentEmbeddingTokenizationReport,
    DocumentEmbeddingTruncationExampleReport, IngestDiagnostics, SkippedRecord,
    analyze_foundry_source, build_artifact,
};
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use serde_json::{Value, json};

use crate::output::{format_duration_ms, write_json_data, write_validation_report};

pub(crate) mod args;

use args::{
    AnalyzeIndexOptions, BuildIndexOptions, CheckIndexOptions, IndexPathOptions,
    ValidateIndexOptions,
};

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
        write_json_data(&report)?;
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
        write_json_data(build_artifact_json(&report))?;
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
            "embeddings: pending_document={} document={} reused={} generated={} truncated={} max_tokens={} max_observed_tokens={}",
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
        );
        eprintln!(
            "timing: build={} embedding_tokenization={} embedding_model_load={} embedding_generation={}",
            format_duration_ms(report.build_duration_ms),
            format_duration_ms(report.embedding_timing.tokenization_duration_ms),
            format_duration_ms(report.embedding_timing.model_load_duration_ms),
            format_duration_ms(report.embedding_timing.generation_duration_ms),
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

fn build_artifact_json(report: &BuildArtifactReport) -> Value {
    json!({
        "output": report.output_path.display().to_string(),
        "pack_count": report.pack_count,
        "record_count": report.record_count,
        "source_record_count": report.source_record_count,
        "artifact_record_count": report.artifact_record_count,
        "generated_record_count": report.generated_record_count,
        "pending_document_embedding_count": report.pending_document_embedding_count,
        "document_embedding_count": report.document_embedding_count,
        "reused_document_embedding_count": report.reused_document_embedding_count,
        "generated_document_embedding_count": report.generated_document_embedding_count,
        "document_embedding_tokenization": document_embedding_tokenization_json(
            &report.document_embedding_tokenization,
        ),
        "embedding_timing": {
            "tokenization_duration_ms": report.embedding_timing.tokenization_duration_ms,
            "model_load_duration_ms": report.embedding_timing.model_load_duration_ms,
            "generation_duration_ms": report.embedding_timing.generation_duration_ms,
            "batch_count": report.embedding_timing.batch_count,
            "batch_duration_min_ms": report.embedding_timing.batch_duration_min_ms,
            "batch_duration_p50_ms": report.embedding_timing.batch_duration_p50_ms,
            "batch_duration_p95_ms": report.embedding_timing.batch_duration_p95_ms,
            "batch_duration_max_ms": report.embedding_timing.batch_duration_max_ms,
        },
        "build_duration_ms": report.build_duration_ms,
        "source_signature": report.source_signature,
        "diagnostics": diagnostics_json(&report.diagnostics),
        "skipped_record_count": report.skipped_records.len(),
        "skipped_records": skipped_records_json(&report.skipped_records),
        "warnings": report.warnings,
    })
}

fn diagnostics_json(diagnostics: &IngestDiagnostics) -> Value {
    json!({
        "taxonomy": {
            "folder_records": diagnostics.taxonomy_folder_records,
            "glossary_records": diagnostics.taxonomy_glossary_records,
        },
        "variants": {
            "parenthetical_records": diagnostics.variant_parenthetical_records,
            "suffix_records": diagnostics.variant_suffix_records,
            "creature_blurb_records": diagnostics.variant_creature_blurb_records,
            "creature_suffix_records": diagnostics.variant_creature_suffix_records,
            "exact_base_records": diagnostics.variant_exact_base_records,
        },
        "generated_afflictions": {
            "canonical_records": diagnostics.generated_affliction_canonical_records,
            "instance_records": diagnostics.generated_affliction_instance_records,
            "reference_edges": diagnostics.generated_affliction_reference_edges,
        },
        "dropped_inline_macros": diagnostics.dropped_inline_macros.iter().map(|(name, diagnostic)| {
            json!({
                "name": name,
                "count": diagnostic.count,
                "examples": diagnostic.examples,
            })
        }).collect::<Vec<_>>(),
    })
}

fn skipped_records_json(skipped_records: &[SkippedRecord]) -> Vec<Value> {
    skipped_records
        .iter()
        .map(|record| {
            json!({
                "path": record.path.display().to_string(),
                "reason": record.reason,
            })
        })
        .collect()
}

fn document_embedding_tokenization_json(telemetry: &DocumentEmbeddingTokenizationReport) -> Value {
    json!({
        "document_count": telemetry.document_count,
        "truncated_document_count": telemetry.truncated_document_count,
        "max_token_count": telemetry.max_token_count,
        "max_observed_token_count": telemetry.max_observed_token_count,
        "total_observed_token_count": telemetry.total_observed_token_count,
        "total_tokens_over_limit": telemetry.total_tokens_over_limit,
        "unit_kind_truncations": telemetry.unit_kind_truncations
            .iter()
            .map(|truncation| {
                json!({
                    "unit_kind": truncation.unit_kind,
                    "unit_count": truncation.unit_count,
                    "record_count": truncation.record_count,
                    "total_tokens_over_limit": truncation.total_tokens_over_limit,
                    "max_observed_token_count": truncation.max_observed_token_count,
                    "examples": truncation.examples
                        .iter()
                        .map(truncation_example_json)
                        .collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
        "record_truncation_coverage": {
            "record_count": telemetry.record_truncation_coverage.record_count,
            "records_with_child_units": telemetry.record_truncation_coverage.records_with_child_units,
            "records_with_any_truncated_unit": telemetry.record_truncation_coverage.records_with_any_truncated_unit,
            "records_with_truncated_parent_unit": telemetry.record_truncation_coverage.records_with_truncated_parent_unit,
            "records_with_truncated_child_unit": telemetry.record_truncation_coverage.records_with_truncated_child_unit,
            "records_with_truncated_parent_and_child_units": telemetry.record_truncation_coverage.records_with_truncated_parent_and_child_units,
            "records_with_truncated_parent_and_all_child_units_fit": telemetry.record_truncation_coverage.records_with_truncated_parent_and_all_child_units_fit,
            "records_with_truncated_parent_without_child_units": telemetry.record_truncation_coverage.records_with_truncated_parent_without_child_units,
        },
        "section_truncations": telemetry.section_truncations
            .iter()
            .map(|section| {
                json!({
                    "section": section.section,
                    "document_count": section.document_count,
                    "dropped_chunk_count": section.dropped_chunk_count,
                })
            })
            .collect::<Vec<_>>(),
        "truncated_examples": telemetry.truncated_examples
            .iter()
            .map(truncation_example_json)
            .collect::<Vec<_>>(),
    })
}

fn truncation_example_json(example: &DocumentEmbeddingTruncationExampleReport) -> Value {
    json!({
        "embedding_unit_key": example.embedding_unit_key,
        "record_key": example.record_key,
        "unit_kind": example.unit_kind,
        "label": example.label,
        "token_count": example.token_count,
        "max_token_count": example.max_token_count,
        "truncated_sections": example.truncated_sections,
    })
}

pub(crate) fn run_index_inspect(options: IndexPathOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let report = runtime
        .open_index()
        .map_err(|error| error.to_string())?
        .inspect()
        .map_err(|error| error.to_string())?;

    if options.json {
        write_json_data(&report)?;
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

pub(crate) fn run_index_check(options: CheckIndexOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let target = if options.no_embeddings {
        ValidationTarget::BaseOnly
    } else {
        ValidationTarget::Full
    };
    let report = runtime.check_index_report(target);
    write_validation_report(report, options.json)
}

pub(crate) fn run_index_validate(options: ValidateIndexOptions) -> Result<ExitCode, String> {
    let runtime = index_runtime(options.path_mode.into(), options.index)?;
    let target = if options.no_embeddings {
        ValidationTarget::BaseOnly
    } else if options.embeddings_only {
        ValidationTarget::EmbeddingsOnly
    } else {
        ValidationTarget::Full
    };
    let report = runtime.validate_index_report(target);
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
