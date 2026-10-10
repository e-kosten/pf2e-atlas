//! Explicit setup is the sole runtime path allowed to inspect/fetch the source clone.
use crate::setup_freshness::source_is_fresh;
use crate::{ResolvedAtlasPaths, setup_model::*};
use atlas_embedding::{
    EmbeddingRuntimeConfig, prepare_embedding_model_cache, required_embedding_model_cache_files,
};
use atlas_index::SqliteIndexReader;
use atlas_ingest::{BuildArtifactOptions, BuildArtifactReport, build_artifact};
use std::{path::Path, process::Command};
fn progress(phase: &str, message: &str) {
    tracing::info!(target:"atlas_progress",phase,"{message}");
}
pub(crate) fn ensure_setup(
    paths: &ResolvedAtlasPaths,
    options: RuntimeSetupOptions,
) -> RuntimeSetupReport {
    let mut checks = Vec::new();
    let mut actions = Vec::new();
    let mut source_ready = paths.source_root.is_dir();
    let config =
        EmbeddingRuntimeConfig::new(options.embedding_model_id, &paths.embedding_cache_root);
    let mut model_ready = atlas_embedding::validate_embedding_model_cache(&config).is_ok();
    if !source_ready
        || (!options.check && !options.offline && paths.source_root.join(".git").exists())
    {
        if options.check || options.offline {
            actions.push(SetupAction::with_reason(
                SetupActionKind::FetchSource,
                if options.offline {
                    SetupActionStatus::Blocked
                } else {
                    SetupActionStatus::Planned
                },
                "source checkout is missing or needs updating",
            ));
        } else {
            progress("fetch_source", "Preparing Foundry source");
            match fetch_source(&paths.source_root) {
                Ok(()) => {
                    source_ready = true;
                    actions.push(SetupAction::new(
                        SetupActionKind::FetchSource,
                        SetupActionStatus::Done,
                    ));
                }
                Err(e) => {
                    source_ready = false;
                    actions.push(SetupAction::with_reason(
                        SetupActionKind::FetchSource,
                        SetupActionStatus::Failed,
                        e,
                    ));
                }
            }
        }
    }
    if options.target.requires_embeddings() && !model_ready {
        if options.check || options.offline {
            actions.push(SetupAction::with_reason(
                SetupActionKind::PrepareEmbeddingModel,
                if options.offline {
                    SetupActionStatus::Blocked
                } else {
                    SetupActionStatus::Planned
                },
                "required pinned embedding assets are missing or invalid",
            ));
        } else {
            progress(
                "prepare_embedding_model",
                "Preparing pinned embedding assets",
            );
            match prepare_embedding_model_cache(&config) {
                Ok(_) => {
                    model_ready = true;
                    actions.push(SetupAction::new(
                        SetupActionKind::PrepareEmbeddingModel,
                        SetupActionStatus::Done,
                    ));
                }
                Err(e) => actions.push(SetupAction::with_reason(
                    SetupActionKind::PrepareEmbeddingModel,
                    SetupActionStatus::Failed,
                    e.to_string(),
                )),
            }
        }
    }
    let old = SqliteIndexReader::open_read_only(&paths.index_path);
    let locale = options
        .locale
        .clone()
        .or_else(|| {
            old.as_ref()
                .ok()
                .map(|i| i.context().indexing_locale.clone())
        })
        .unwrap_or_else(|| "en".into());
    let freshness = if source_ready {
        match old.as_ref() {
            Ok(i) => source_is_fresh(paths, i.context()),
            Err(_) => Ok(false),
        }
    } else {
        Err("source checkout is unavailable for explicit freshness checking".into())
    };
    match &freshness {
        Ok(true) => checks.push(SetupAction::new(
            SetupActionKind::AnalyzeSource,
            SetupActionStatus::Done,
        )),
        Ok(false) => checks.push(SetupAction::with_reason(
            SetupActionKind::AnalyzeSource,
            SetupActionStatus::Done,
            "source content differs from the indexed fingerprint",
        )),
        Err(e) => checks.push(SetupAction::with_reason(
            SetupActionKind::AnalyzeSource,
            SetupActionStatus::Blocked,
            e,
        )),
    }
    let old_semantic = if options.target.requires_embeddings() {
        SqliteIndexReader::open_read_only_with_vectors(&paths.index_path).is_ok()
    } else {
        true
    };
    let needs_build = options.force_rebuild
        || old.is_err()
        || !old_semantic
        || freshness.as_ref().is_ok_and(|f| !*f)
        || (source_ready && freshness.is_err())
        || old
            .as_ref()
            .is_ok_and(|i| i.context().indexing_locale != locale);
    let mut build = None;
    let mut build_failed = false;
    if needs_build {
        if options.check {
            actions.push(SetupAction::with_reason(
                SetupActionKind::BuildIndex,
                SetupActionStatus::Planned,
                "artifact requires rebuild",
            ));
        } else if !source_ready || (options.target.requires_embeddings() && !model_ready) {
            actions.push(SetupAction::with_reason(
                SetupActionKind::BuildIndex,
                SetupActionStatus::Blocked,
                "source or required embedding assets are unavailable",
            ));
        } else {
            progress("build_index", "Building source-backed artifact");
            let result = build_artifact(BuildArtifactOptions {
                source_root: paths.source_root.clone(),
                output_path: paths.index_path.clone(),
                manifest_path: None,
                locale,
                embedding: options
                    .target
                    .requires_embeddings()
                    .then_some(config.clone()),
                reuse_embeddings: true,
                embedding_batch_size: options.embedding_batch_size,
            });
            match result {
                Ok(r) => {
                    build = Some(r.into());
                    actions.push(SetupAction::new(
                        SetupActionKind::BuildIndex,
                        SetupActionStatus::Done,
                    ));
                }
                Err(e) => {
                    build_failed = true;
                    actions.push(SetupAction::with_reason(
                        SetupActionKind::BuildIndex,
                        SetupActionStatus::Failed,
                        e.to_string(),
                    ));
                }
            }
        }
    } else {
        actions.push(SetupAction::new(
            SetupActionKind::BuildIndex,
            SetupActionStatus::Skipped,
        ));
    }
    // Publication owns atomic replacement; setup never deletes the prior artifact.
    let current = SqliteIndexReader::open_read_only(&paths.index_path);
    let records_ready = current.is_ok() && !build_failed && (!needs_build || build.is_some());
    let semantic_ready = records_ready
        && model_ready
        && SqliteIndexReader::open_read_only_with_vectors(&paths.index_path).is_ok();
    checks.push(match &current{Ok(_)=>SetupAction::with_reason(SetupActionKind::ValidateIndex,SetupActionStatus::Done,"cheap executable schema/catalog check; full snapshot validation is a separate operation"),Err(e)=>SetupAction::with_reason(SetupActionKind::ValidateIndex,SetupActionStatus::Blocked,e.to_string())});
    let readiness = |ready: bool, required: bool, reason: &str| {
        if ready {
            SetupReadinessItem::ready(required)
        } else {
            SetupReadinessItem::not_ready(required, reason)
        }
    };
    RuntimeSetupReport {
        target: options.target,
        ready: records_ready && (!options.target.requires_embeddings() || semantic_ready),
        path_mode: paths.mode.as_str(),
        repo_root: paths.repo_root.as_ref().map(|p| p.display().to_string()),
        offline: options.offline,
        check: options.check,
        force_rebuild: options.force_rebuild,
        checks,
        actions,
        readiness: SetupReadiness {
            source: readiness(source_ready, false, "source checkout is unavailable"),
            embedding_model: if options.target.requires_embeddings() {
                readiness(model_ready, true, "required pinned assets unavailable")
            } else {
                SetupReadinessItem::skipped("lexical setup does not require embedding assets")
            },
            records: readiness(
                records_ready,
                true,
                "artifact is absent, stale, incompatible, or rebuild failed",
            ),
            semantic_search: if options.target.requires_embeddings() {
                readiness(
                    semantic_ready,
                    true,
                    "semantic vectors or query model cache unavailable",
                )
            } else {
                SetupReadinessItem::skipped("lexical setup requested")
            },
        },
        paths: SetupPathsReport {
            source: paths.source_root.display().to_string(),
            embedding_cache: paths.embedding_cache_root.display().to_string(),
            index: paths.index_path.display().to_string(),
        },
        embedding: SetupEmbeddingReport {
            model: options.embedding_model_id.to_string(),
            model_path: config.model_dir().display().to_string(),
            cache_root: paths.embedding_cache_root.display().to_string(),
            ready: model_ready,
            missing_files: required_embedding_model_cache_files(&config)
                .iter()
                .filter(|f| !f.local_path.is_file())
                .map(|f| f.local_path.display().to_string())
                .collect(),
        },
        build,
    }
}
impl From<BuildArtifactReport> for SetupBuildReport {
    fn from(r: BuildArtifactReport) -> Self {
        Self {
            pack_count: r.pack_count,
            record_count: r.record_count,
            product_record_count: r.product_record_count,
            semantic_unit_count: r.semantic_unit_count,
            inferred_inputs: r.inferred_inputs,
            reused_inputs: r.reused_inputs,
            context_shortened_sections: r.context_shortened_sections,
            source_fingerprint: r.source_fingerprint,
            build_duration_ms: r.build_duration_ms,
        }
    }
}
fn fetch_source(path: &Path) -> Result<(), String> {
    let mut cmd = Command::new("git");
    if path.exists() {
        if !path.join(".git").exists() {
            return Err("existing source directory is not a Git checkout".into());
        }
        cmd.arg("-C")
            .arg(path)
            .args(["pull", "--ff-only", "--quiet"]);
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        cmd.args([
            "clone",
            "--depth",
            "1",
            "--quiet",
            "https://github.com/foundryvtt/pf2e.git",
        ])
        .arg(path);
    }
    if cmd.status().map_err(|e| e.to_string())?.success() {
        Ok(())
    } else {
        Err("Foundry source fetch failed".into())
    }
}
