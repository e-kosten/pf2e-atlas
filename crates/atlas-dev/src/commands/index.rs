use atlas_cli_support::write_json_data;
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use std::process::ExitCode;
pub(crate) mod args;
use args::IndexPathOptions;
pub(crate) fn run_index_inspect(options: IndexPathOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: None,
            embedding_cache_root: None,
            index_path: options.index,
        },
    })
    .map_err(|e| e.to_string())?;
    let reader = runtime.open_index().map_err(|e| e.to_string())?;
    let stats = reader.statistics().map_err(|e| e.to_string())?;
    if options.json {
        write_json_data(serde_json::json!({"context":reader.context(),"statistics":stats}))?;
    } else {
        println!(
            "{} roots, {} product roots, {} prepared fields, {} lexical units, {} semantic units",
            stats.records,
            stats.product_records,
            stats.prepared_fields,
            stats.lexical_units,
            stats.semantic_units
        );
        println!(
            "snapshot bytes={} prepared HTML bytes={} artifact bytes={}",
            stats.snapshot_bytes, stats.prepared_html_bytes, stats.artifact_bytes
        );
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn run_index_record(o: args::RecordInspectOptions) -> Result<ExitCode, String> {
    use sha2::{Digest, Sha256};
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: o.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: o.source,
            embedding_cache_root: None,
            index_path: o.index,
        },
    })
    .map_err(|e| e.to_string())?;
    let key = atlas_domain::RecordKey::parse(&o.key).map_err(|e| e.to_string())?;
    let reader = runtime.open_index().map_err(|e| e.to_string())?;
    let source = reader
        .read_source_record_for_inspection(&key)
        .map_err(|e| e.to_string())?
        .ok_or("checked source record not found")?;
    let summary = reader
        .read_summary_for_inspection(&key)
        .map_err(|e| e.to_string())?
        .ok_or("source provenance not found")?;
    let snapshot: serde_json::Value = serde_json::from_slice(
        &atlas_foundry_model::encode_snapshot(source.source()).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let original = if o.original {
        let relative = std::path::Path::new(&summary.source_path);
        if relative
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err("stored source path is not a checked relative path".into());
        }
        let root = runtime
            .source_root()
            .canonicalize()
            .map_err(|e| format!("configured source clone unavailable: {e}"))?;
        let path = root
            .join(relative)
            .canonicalize()
            .map_err(|e| format!("original source file unavailable: {e}"))?;
        if !path.starts_with(&root) {
            return Err("original source path escapes configured clone".into());
        }
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let hash = format!("{:x}", Sha256::digest(&bytes));
        if hash != summary.content_hash {
            return Err(format!(
                "original source hash mismatch: expected {}, found {hash}; artifact and clone differ",
                summary.content_hash
            ));
        }
        Some(serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| e.to_string())?)
    } else {
        None
    };
    let data = serde_json::json!({"key":key,"provenance":{"path":summary.source_path,"sha256":summary.content_hash},"checked_snapshot":snapshot,"original_source":original});
    if o.json {
        write_json_data(data)?;
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?
        );
    }
    Ok(ExitCode::SUCCESS)
}
