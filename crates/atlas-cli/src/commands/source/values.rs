use std::process::ExitCode;

use atlas_ingest::{
    IngestError, SourcePathAuditFilters, SourceValueDiscoveryOptions, SourceValueDiscoveryReport,
    discover_source_values,
};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};

use super::args::SourceValuesOptions;
use crate::output::{write_json_data, write_json_error};

pub(crate) fn run_source_values(options: SourceValuesOptions) -> Result<ExitCode, String> {
    let runtime = match AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.selection.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.selection.source,
            embedding_cache_root: None,
            index_path: None,
        },
    }) {
        Ok(runtime) => runtime,
        Err(error) => return discovery_failure(options.json, false, error.to_string()),
    };
    let report = match discover_source_values(SourceValueDiscoveryOptions {
        source_root: runtime.paths().source_root.clone(),
        manifest_path: options.selection.manifest,
        filters: SourcePathAuditFilters {
            pack_name: options.selection.pack_name,
            document_type: options.selection.document_type,
            record_type: options.selection.record_type,
        },
        path: options.path,
        sample_limit: options.sample_limit,
        limit: (options.limit > 0).then_some(options.limit),
    }) {
        Ok(report) => report,
        Err(error) => {
            return discovery_failure(
                options.json,
                matches!(error, IngestError::InvalidSourceDiscoveryPath(_)),
                error.to_string(),
            );
        }
    };
    if options.json {
        write_json_data(&report)?;
    } else {
        print_source_values(&report);
    }
    Ok(ExitCode::SUCCESS)
}

fn discovery_failure(json: bool, invalid_input: bool, message: String) -> Result<ExitCode, String> {
    if json {
        write_json_error(
            if invalid_input {
                "invalid_input"
            } else {
                "source_discovery_failed"
            },
            message,
        )?;
    } else {
        eprintln!("{message}");
    }
    Ok(ExitCode::from(if invalid_input { 2 } else { 3 }))
}

fn print_source_values(report: &SourceValueDiscoveryReport) {
    println!(
        "{}: {} source documents across {} packs",
        report.path, report.record_count, report.pack_count
    );
    for field in &report.fields {
        println!(
            "\n{} / {}: {} present, {} missing, {} occurrences, {} duplicate members, {} distinct values",
            field.key.document_type,
            field.key.record_type,
            field.record_count,
            field.missing_record_count,
            field.occurrence_count,
            field.duplicate_member_count,
            field.distinct_value_count,
        );
        for value in &field.values {
            let preview = value.value_json.chars().take(160).collect::<String>();
            let suffix = if preview.len() < value.value_json.len() {
                "…"
            } else {
                ""
            };
            println!(
                "  {} occurrences / {} documents: {preview}{suffix}",
                value.occurrence_count, value.record_count
            );
            for sample in &value.examples {
                println!(
                    "    {} @ {}#{}",
                    sample.record_key, sample.source_path, sample.source_pointer
                );
            }
        }
        if !field.complete {
            println!(
                "  Showing {} of {} values; counts cover the complete scan.",
                field.values.len(),
                field.distinct_value_count
            );
        }
    }
}
