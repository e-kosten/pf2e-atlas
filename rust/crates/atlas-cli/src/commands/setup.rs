use std::process::ExitCode;

use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use serde_json::json;

use crate::SetupOptions;

pub(crate) fn run_setup(options: SetupOptions) -> Result<ExitCode, String> {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        path_mode: options.path_mode.into(),
        overrides: AtlasPathOverrides {
            source_root: options.source,
            embedding_cache_root: options.embedding_cache_path,
            index_path: options.index,
        },
    })?;

    if options.fetch_source {
        runtime.fetch_source()?;
    }

    let paths = runtime.paths();
    let status = runtime.setup_status();
    let ready = status.ready();

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": if ready { "ready" } else { "not_ready" },
                "path_mode": paths.mode.as_str(),
                "repo_root": paths.repo_root.as_ref().map(|path| path.display().to_string()),
                "source": {
                    "path": paths.source_root.display().to_string(),
                    "exists": status.source_exists,
                },
                "embedding": {
                    "model": status.embedding_model,
                    "cache_root": paths.embedding_cache_root.display().to_string(),
                    "model_path": status.model_cache.model_dir.display().to_string(),
                    "ready": status.model_cache.ready,
                    "missing_files": status.model_cache
                        .missing_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>(),
                },
                "index": {
                    "path": paths.index_path.display().to_string(),
                    "exists": status.index_exists,
                },
                "next": {
                    "build_index": format!(
                        "atlas index build --path-mode {}",
                        paths.mode.suggested_path_mode()
                    ),
                },
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!("mode: {}", paths.mode.label());
        if let Some(repo_root) = &paths.repo_root {
            println!("repo root: {}", repo_root.display());
        }
        println!(
            "source: {} {}",
            status_label(status.source_exists),
            paths.source_root.display()
        );
        println!("embedding model: {}", status.embedding_model);
        println!(
            "embedding cache: {} {}",
            status_label(status.model_cache.ready),
            status.model_cache.model_dir.display()
        );
        for missing_file in &status.model_cache.missing_files {
            println!("missing model file: {}", missing_file.display());
        }
        println!(
            "index: {} {}",
            status_label(status.index_exists),
            paths.index_path.display()
        );
        if ready {
            println!();
            println!("ready:");
            println!(
                "  atlas index build --path-mode {}",
                paths.mode.suggested_path_mode()
            );
        } else {
            println!();
            println!("not ready:");
            if !status.source_exists {
                println!("  run atlas setup --fetch-source");
            }
            if !status.model_cache.ready {
                println!("  prepare the default embedding model cache, then rerun atlas setup");
            }
        }
    }

    Ok(if ready {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn status_label(ok: bool) -> &'static str {
    if ok { "ok" } else { "missing" }
}
