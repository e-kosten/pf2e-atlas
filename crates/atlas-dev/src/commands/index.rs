use std::path::PathBuf;
use std::process::ExitCode;

use atlas_cli_support::write_json_data;
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};

pub(crate) mod args;
use args::IndexPathOptions;

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

fn index_runtime(path_mode: AtlasPathMode, index: Option<PathBuf>) -> Result<AtlasRuntime, String> {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: index,
        },
    })
    .map_err(|error| error.to_string())
}
