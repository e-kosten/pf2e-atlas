use std::process::ExitCode;

use atlas_ingest::{SourcePathAuditOptions, SourcePathAuditReport, audit_source_paths};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};

use super::args::SourceSchemaOptions;
use crate::output::write_json_data;

pub(crate) fn run_source_schema(options: SourceSchemaOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.selection.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.selection.source,
            embedding_cache_root: None,
            index_path: None,
        },
    })
    .map_err(|error| error.to_string())?;
    let paths = runtime.paths();
    let report = audit_source_paths(SourcePathAuditOptions {
        source_root: paths.source_root.clone(),
        manifest_path: options.selection.manifest,
        pack_name: options.selection.pack_name,
        document_type: options.selection.document_type,
        record_type: options.selection.record_type,
        min_records: options.min_records,
        limit: (options.limit > 0).then_some(options.limit),
        strict: options.strict,
        baseline_report: options.baseline,
    })
    .map_err(|error| error.to_string())?;

    if options.json {
        write_json_data(&report)?;
    } else {
        print_source_path_audit(&report);
    }

    if !options.strict
        || report
            .source_diff
            .as_ref()
            .is_none_or(|diff| diff.change_count() == 0)
    {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(3))
    }
}

fn print_source_path_audit(report: &SourcePathAuditReport) {
    println!(
        "ok: discovered {} paths in {} records across {} packs ({})",
        report.path_count, report.record_count, report.pack_count, report.schema_version
    );
    println!(
        "source_signature={} complete={}",
        report.source_signature, report.complete
    );
    if let Some(diff) = &report.source_diff {
        println!(
            "schema diff: added={} removed={} changed_types={} changed_duplicate_members={}",
            diff.added_paths.len(),
            diff.removed_paths.len(),
            diff.changed_types.len(),
            diff.changed_duplicate_members.len()
        );
        for key in &diff.added_paths {
            println!("+ {}|{} {}", key.document_type, key.record_type, key.path);
        }
        for key in &diff.removed_paths {
            println!("- {}|{} {}", key.document_type, key.record_type, key.path);
        }
        for change in &diff.changed_types {
            println!(
                "~ {}|{} {} {:?} -> {:?}",
                change.key.document_type,
                change.key.record_type,
                change.key.path,
                change.before,
                change.after
            );
        }
        for key in &diff.changed_duplicate_members {
            println!(
                "~ {}|{} {} duplicate members changed",
                key.document_type, key.record_type, key.path
            );
        }
    }
    for path in &report.paths {
        println!(
            "{}|{} {} records={} occurrences={} types={:?}",
            path.key.document_type,
            path.key.record_type,
            path.key.path,
            path.record_count,
            path.occurrence_count,
            path.value_types
                .iter()
                .map(|value| value.kind.as_str())
                .collect::<Vec<_>>()
        );
        for example in &path.examples {
            println!(
                "  e.g. {} {} = {}",
                example.record_key, example.source_path, example.value
            );
        }
    }
}
