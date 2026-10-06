use std::process::ExitCode;

use atlas_cli_support::write_json_data;
use atlas_ingest::{
    SourcePathAuditOptions, SourcePathAuditReport, SourcePathCoverageStatus,
    analyze_foundry_source, audit_source_paths,
};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};

pub(crate) mod args;
use args::{AnalyzeOptions, AuditPathsOptions};

pub(crate) fn run_source_analyze(options: AnalyzeOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: None,
            index_path: None,
        },
    })
    .map_err(|error| error.to_string())?;
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

pub(crate) fn run_source_audit_paths(options: AuditPathsOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: None,
            index_path: None,
        },
    })
    .map_err(|error| error.to_string())?;
    let paths = runtime.paths();
    let report = audit_source_paths(SourcePathAuditOptions {
        source_root: paths.source_root.clone(),
        manifest_path: options.manifest,
        pack_name: options.pack_name,
        document_type: options.document_type,
        record_type: options.record_type,
        min_records: options.min_records,
        limit: Some(options.limit),
    })
    .map_err(|error| error.to_string())?;

    if options.json {
        write_json_data(&report)?;
    } else {
        print_source_path_audit(&report);
    }

    Ok(ExitCode::SUCCESS)
}

fn print_source_path_audit(report: &SourcePathAuditReport) {
    println!(
        "ok: audited {} records from {} packs in {}",
        report.record_count, report.pack_count, report.source_root
    );
    println!(
        "paths: showing {} paths with min_records={}",
        report.paths.len(),
        report.filters.min_records
    );
    for path in &report.paths {
        let consumers = if path.known_consumers.is_empty() {
            "none".to_string()
        } else {
            path.known_consumers.join(",")
        };
        println!(
            "{} records={} occurrences={} status={} consumers={}",
            path.path,
            path.record_count,
            path.occurrence_count,
            coverage_status_label(path.coverage_status),
            consumers
        );
        for example in &path.examples {
            println!(
                "  e.g. {} {} = {}",
                example.record_key, example.source_path, example.value
            );
        }
    }
}

fn coverage_status_label(status: SourcePathCoverageStatus) -> &'static str {
    match status {
        SourcePathCoverageStatus::Consumed => "consumed",
        SourcePathCoverageStatus::Partial => "partial",
        SourcePathCoverageStatus::Uncovered => "uncovered",
    }
}
