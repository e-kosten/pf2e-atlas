use std::process::ExitCode;

use atlas_cli_support::write_json_data;
use atlas_ingest::{
    SourcePathAuditOptions, SourcePathAuditReport, analyze_foundry_source, audit_source_paths,
    load_foundry_documents,
};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};

pub(crate) mod args;
use args::{AnalyzeOptions, AuditPathsOptions, LoadOptions};

pub(crate) fn run_source_load(options: LoadOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: None,
            index_path: None,
        },
    })
    .map_err(|error| error.to_string())?;
    let loaded = load_foundry_documents(&runtime.paths().source_root, options.manifest.as_deref())
        .map_err(|error| error.to_string())?;
    let report = loaded.report();
    if options.json {
        write_json_data(&report)?;
    } else {
        let counts = &report.counts;
        println!(
            "loaded {} documents from {} manifest packs in {}",
            counts.retained_documents,
            report.pack_count,
            report.metadata.source_root.display()
        );
        println!(
            "documents: modeled={} diagnostic_free={} partial={} raw_only={} quarantined={}",
            counts.modeled_documents,
            counts.diagnostic_free_documents,
            counts.partial_documents,
            counts.raw_only_documents,
            counts.quarantined_files
        );
        println!(
            "diagnostics={} unavailable_packs={}",
            counts.diagnostics, counts.unavailable_packs
        );
        for diagnostic in &report.diagnostics {
            println!("diagnostic: {diagnostic}");
        }
        for failure in &report.failures {
            println!(
                "failure: {} {} ({:?}): {}",
                failure.provenance.pack_name,
                failure.provenance.source_path,
                failure.stage,
                failure.message
            );
        }
    }
    // Invalid fields are retained outcomes. Missing packs, quarantine and raw-only
    // roots signal incomplete loading to scripts, while still emitting the report.
    Ok(
        if report.counts.quarantined_files > 0
            || report.counts.unavailable_packs > 0
            || report.counts.raw_only_documents > 0
            || report.counts.retained_documents == 0
        {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        },
    )
}

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
            "ok: analyzed {} source records in {}",
            report.source.retained_documents,
            paths.source_root.display()
        );
        println!(
            "products={} selected fields={} sections={} prose bytes={} named definitions={} locale={}",
            report.product_records,
            report.selected_fields,
            report.selected_sections,
            report.selected_body_bytes,
            report.lexical_definitions,
            report.indexing_locale
        );
        println!(
            "source diagnostics={} partial documents={} quarantined={}",
            report.source.admission_diagnostics,
            report.source.partial_documents,
            report.source.quarantined_files
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
        println!(
            "{} records={} occurrences={}",
            path.path, path.record_count, path.occurrence_count
        );
        for example in &path.examples {
            println!(
                "  e.g. {} {} = {}",
                example.record_key, example.source_path, example.value
            );
        }
    }
}
