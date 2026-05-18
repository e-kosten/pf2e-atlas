use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use atlas_embedding::{
    EmbeddingRuntimeConfig, prepare_embedding_model_cache, required_embedding_model_cache_files,
};
use atlas_index::{ArtifactValidationReport, ValidationStatus, ValidationTarget};
use atlas_ingest::{
    BuildArtifactOptions, BuildArtifactReport, analyze_foundry_source, build_artifact,
};
use tracing::info;

use crate::ResolvedAtlasPaths;
use crate::setup_model::{
    RuntimeSetupOptions, RuntimeSetupReport, SetupAction, SetupActionKind, SetupActionStatus,
    SetupBuildReport, SetupEmbeddingReport, SetupPathsReport, SetupReadiness, SetupReadinessItem,
    SetupTarget,
};

#[derive(Debug, Clone)]
struct EmbeddingModelCacheStatus {
    model_dir: PathBuf,
    ready: bool,
    missing_files: Vec<PathBuf>,
}

pub(crate) fn ensure_setup(
    paths: &ResolvedAtlasPaths,
    options: RuntimeSetupOptions,
) -> RuntimeSetupReport {
    setup_progress(
        "setup",
        format!(
            "Using {} paths; index {}",
            paths.mode.label(),
            paths.index_path.display()
        ),
    );
    let mut actions = Vec::new();
    let source_exists = paths.source_root.is_dir();
    let mut source_ready = source_exists;
    if source_exists
        && !options.check
        && !options.offline
        && paths.source_root.join(".git").exists()
    {
        setup_progress(
            "fetch_source",
            format!("Updating PF2E source at {}", paths.source_root.display()),
        );
        match fetch_pf2e_source(&paths.source_root) {
            Ok(()) => {
                actions.push(SetupAction::new(
                    SetupActionKind::FetchSource,
                    SetupActionStatus::Done,
                ));
            }
            Err(error) => {
                source_ready = false;
                actions.push(SetupAction::with_reason(
                    SetupActionKind::FetchSource,
                    SetupActionStatus::Failed,
                    error,
                ));
            }
        }
    } else if source_exists {
        actions.push(SetupAction::new(
            SetupActionKind::FetchSource,
            SetupActionStatus::Skipped,
        ));
    } else if options.check {
        let status = if options.offline {
            SetupActionStatus::Blocked
        } else {
            SetupActionStatus::Planned
        };
        actions.push(SetupAction::with_reason(
            SetupActionKind::FetchSource,
            status,
            if options.offline {
                "offline"
            } else {
                "source checkout is missing"
            },
        ));
    } else if options.offline {
        actions.push(SetupAction::with_reason(
            SetupActionKind::FetchSource,
            SetupActionStatus::Blocked,
            "offline",
        ));
    } else {
        setup_progress(
            "fetch_source",
            format!("Cloning PF2E source into {}", paths.source_root.display()),
        );
        match fetch_pf2e_source(&paths.source_root) {
            Ok(()) => {
                source_ready = true;
                actions.push(SetupAction::new(
                    SetupActionKind::FetchSource,
                    SetupActionStatus::Done,
                ));
            }
            Err(error) => {
                source_ready = false;
                actions.push(SetupAction::with_reason(
                    SetupActionKind::FetchSource,
                    SetupActionStatus::Failed,
                    error,
                ));
            }
        }
    }

    let embedding_config =
        EmbeddingRuntimeConfig::new(options.embedding_model_id, &paths.embedding_cache_root);
    let mut model_cache = embedding_model_cache_status(&embedding_config);
    let embedding_required = options.target.requires_embeddings();
    if !embedding_required || model_cache.ready {
        actions.push(SetupAction::new(
            SetupActionKind::PrepareEmbeddingModel,
            SetupActionStatus::Skipped,
        ));
    } else if options.check {
        actions.push(SetupAction::with_reason(
            SetupActionKind::PrepareEmbeddingModel,
            if options.offline {
                SetupActionStatus::Blocked
            } else {
                SetupActionStatus::Planned
            },
            if options.offline {
                "offline"
            } else {
                "embedding model cache is missing required files"
            },
        ));
    } else if options.offline {
        actions.push(SetupAction::with_reason(
            SetupActionKind::PrepareEmbeddingModel,
            SetupActionStatus::Blocked,
            "offline",
        ));
    } else {
        setup_progress(
            "prepare_embedding_model",
            format!(
                "Preparing embedding model cache for {}",
                options.embedding_model_id
            ),
        );
        match prepare_embedding_model_cache(&embedding_config) {
            Ok(_) => {
                model_cache = embedding_model_cache_status(&embedding_config);
                if model_cache.ready {
                    actions.push(SetupAction::new(
                        SetupActionKind::PrepareEmbeddingModel,
                        SetupActionStatus::Done,
                    ));
                } else {
                    actions.push(SetupAction::with_reason(
                        SetupActionKind::PrepareEmbeddingModel,
                        SetupActionStatus::Failed,
                        "embedding model cache is missing required files after preparation",
                    ));
                }
            }
            Err(error) => {
                actions.push(SetupAction::with_reason(
                    SetupActionKind::PrepareEmbeddingModel,
                    SetupActionStatus::Failed,
                    error.to_string(),
                ));
            }
        }
    }

    setup_progress(
        "validate_index",
        format!(
            "Validating existing artifact at {}",
            paths.index_path.display()
        ),
    );
    let validation = validate_for_target(paths, options.target.validation_target());
    let validation = selected_model_validation(validation, options.target, &embedding_config);
    let needs_source_signature =
        source_ready && !options.force_rebuild && validation.status == ValidationStatus::Ok;
    let source_signature = if needs_source_signature {
        setup_progress(
            "analyze_source",
            format!("Analyzing PF2E source at {}", paths.source_root.display()),
        );
        match analyze_foundry_source(&paths.source_root, None) {
            Ok(report) => {
                actions.push(SetupAction::new(
                    SetupActionKind::AnalyzeSource,
                    SetupActionStatus::Done,
                ));
                Some(report.source.source_signature)
            }
            Err(error) => {
                actions.push(SetupAction::with_reason(
                    SetupActionKind::AnalyzeSource,
                    SetupActionStatus::Failed,
                    error.to_string(),
                ));
                None
            }
        }
    } else if source_ready {
        actions.push(SetupAction::with_reason(
            SetupActionKind::AnalyzeSource,
            SetupActionStatus::Skipped,
            if options.force_rebuild {
                "force rebuild requested"
            } else {
                "current artifact already requires rebuild"
            },
        ));
        None
    } else {
        actions.push(SetupAction::with_reason(
            SetupActionKind::AnalyzeSource,
            SetupActionStatus::Blocked,
            "source checkout is not ready",
        ));
        None
    };

    let rebuild_needed = rebuild_needed(
        &validation,
        source_signature.as_deref(),
        options.force_rebuild,
    );
    let build = if rebuild_needed {
        let blocked_reason = build_blocked_reason(&options, source_ready, &model_cache);
        if let Some(reason) = blocked_reason {
            actions.push(SetupAction::with_reason(
                SetupActionKind::BuildIndex,
                if check_can_plan_blocked_build(&options, reason) {
                    SetupActionStatus::Planned
                } else {
                    SetupActionStatus::Blocked
                },
                reason,
            ));
            None
        } else if options.check {
            actions.push(SetupAction::with_reason(
                SetupActionKind::BuildIndex,
                SetupActionStatus::Planned,
                build_reason(
                    &validation,
                    source_signature.as_deref(),
                    options.force_rebuild,
                ),
            ));
            None
        } else {
            setup_progress(
                "build_index",
                format!("Building SQLite artifact at {}", paths.index_path.display()),
            );
            match build_artifact(BuildArtifactOptions {
                source_root: paths.source_root.clone(),
                output_path: paths.index_path.clone(),
                manifest_path: None,
                embedding_model_id: options.embedding_model_id.to_string(),
                embedding_cache_root: if embedding_required {
                    Some(paths.embedding_cache_root.clone())
                } else {
                    None
                },
                reuse_embeddings: true,
                embedding_batch_size: options.embedding_batch_size,
            }) {
                Ok(report) => {
                    actions.push(SetupAction::new(
                        SetupActionKind::BuildIndex,
                        SetupActionStatus::Done,
                    ));
                    Some(SetupBuildReport::from(report))
                }
                Err(error) => {
                    actions.push(SetupAction::with_reason(
                        SetupActionKind::BuildIndex,
                        SetupActionStatus::Failed,
                        error.to_string(),
                    ));
                    None
                }
            }
        }
    } else {
        actions.push(SetupAction::new(
            SetupActionKind::BuildIndex,
            SetupActionStatus::Skipped,
        ));
        None
    };

    let final_validation = if build.is_some() {
        setup_progress(
            "validate_index",
            format!(
                "Validating final artifact at {}",
                paths.index_path.display()
            ),
        );
        selected_model_validation(
            validate_for_target(paths, options.target.validation_target()),
            options.target,
            &embedding_config,
        )
    } else {
        validation
    };
    actions.push(SetupAction::new(
        SetupActionKind::ValidateIndex,
        if final_validation.status == ValidationStatus::Ok {
            SetupActionStatus::Done
        } else {
            SetupActionStatus::Blocked
        },
    ));
    let record_validation = if embedding_required {
        setup_progress("validate_index", "Validating base record readiness");
        validate_for_target(paths, ValidationTarget::BaseOnly)
    } else {
        final_validation.clone()
    };

    let ready = source_ready
        && (!embedding_required || model_cache.ready)
        && final_validation.status == ValidationStatus::Ok
        && !actions.iter().any(|action| {
            matches!(
                action.status,
                SetupActionStatus::Planned | SetupActionStatus::Blocked | SetupActionStatus::Failed
            )
        });
    let report = RuntimeSetupReport {
        target: options.target,
        ready,
        path_mode: paths.mode.as_str(),
        repo_root: paths
            .repo_root
            .as_ref()
            .map(|path| path.display().to_string()),
        offline: options.offline,
        check: options.check,
        force_rebuild: options.force_rebuild,
        actions,
        readiness: SetupReadiness {
            source: if source_ready {
                SetupReadinessItem::ready(true)
            } else {
                SetupReadinessItem::not_ready(true, "source checkout is not ready")
            },
            embedding_model: if embedding_required {
                if model_cache.ready {
                    SetupReadinessItem::ready(true)
                } else {
                    SetupReadinessItem::not_ready(
                        true,
                        "embedding model cache is missing required files",
                    )
                }
            } else {
                SetupReadinessItem::skipped("not required for records target")
            },
            records: if record_validation.status == ValidationStatus::Ok {
                SetupReadinessItem::ready(true)
            } else {
                SetupReadinessItem::not_ready(true, record_validation.message.clone())
            },
            semantic_search: if embedding_required {
                if final_validation.status == ValidationStatus::Ok {
                    SetupReadinessItem::ready(true)
                } else {
                    SetupReadinessItem::not_ready(true, final_validation.message.clone())
                }
            } else {
                SetupReadinessItem::skipped("not required for records target")
            },
        },
        paths: SetupPathsReport {
            source: paths.source_root.display().to_string(),
            embedding_cache: paths.embedding_cache_root.display().to_string(),
            index: paths.index_path.display().to_string(),
        },
        embedding: SetupEmbeddingReport {
            model: options.embedding_model_id.to_string(),
            model_path: model_cache.model_dir.display().to_string(),
            cache_root: paths.embedding_cache_root.display().to_string(),
            ready: model_cache.ready,
            missing_files: model_cache
                .missing_files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
        },
        build,
    };
    setup_progress_complete();
    report
}

fn setup_progress(phase: &'static str, message: impl AsRef<str>) {
    let message = message.as_ref();
    info!(target: "atlas_progress", phase, "{message}");
}

fn setup_progress_complete() {
    info!(target: "atlas_progress", complete = true, "setup complete");
}

fn selected_model_validation(
    validation: ArtifactValidationReport,
    target: SetupTarget,
    embedding_config: &EmbeddingRuntimeConfig,
) -> ArtifactValidationReport {
    if target != SetupTarget::Full || validation.status != ValidationStatus::Ok {
        return validation;
    }
    let spec = embedding_config.model_spec();
    if validation.embedding_model_id.as_deref() == Some(spec.model_id)
        && validation.embedding_model_revision.as_deref() == Some(spec.model_revision)
        && validation.embedding_tokenizer_id.as_deref() == Some(spec.tokenizer_id)
    {
        return validation;
    }
    let mut validation = validation;
    validation.status = ValidationStatus::Error;
    validation.code = atlas_index::ValidationCode::EmbeddingMismatch;
    validation.message = format!(
        "artifact embedding model does not match selected setup model `{}`",
        embedding_config.model
    );
    validation
        .diagnostics
        .push(atlas_index::ArtifactValidationDiagnostic {
            code: atlas_index::ValidationCode::EmbeddingMismatch,
            family: atlas_index::ArtifactContractFamily::Embedding,
            message: "artifact embedding model does not match selected setup model".to_string(),
            key: Some("embedding_model_id".to_string()),
            expected: Some(spec.model_id.to_string()),
            actual: validation.embedding_model_id.clone(),
        });
    validation
}

fn validate_for_target(
    paths: &ResolvedAtlasPaths,
    target: ValidationTarget,
) -> ArtifactValidationReport {
    if matches!(target, ValidationTarget::BaseOnly) {
        return match atlas_index::AtlasIndex::open_read_only(&paths.index_path) {
            Ok(index) => index.validate_target_report(target),
            Err(error) => atlas_index::validation_report_for_error(&paths.index_path, error),
        };
    }

    match atlas_index::AtlasIndex::open_read_only_with_vectors(&paths.index_path) {
        Ok(index) => index.validate_target_report(target),
        Err(error) => match atlas_index::AtlasIndex::open_read_only(&paths.index_path) {
            Ok(index) => index.vector_extension_unavailable_report(target, error.to_string()),
            Err(base_error) => {
                atlas_index::validation_report_for_error(&paths.index_path, base_error)
            }
        },
    }
}

fn rebuild_needed(
    validation: &ArtifactValidationReport,
    source_signature: Option<&str>,
    force_rebuild: bool,
) -> bool {
    if force_rebuild {
        return true;
    }
    if validation.status != ValidationStatus::Ok {
        return true;
    }
    source_signature.is_some_and(|current_source_signature| {
        validation.source_signature.as_deref() != Some(current_source_signature)
    })
}

fn build_blocked_reason(
    options: &RuntimeSetupOptions,
    source_ready: bool,
    model_cache: &EmbeddingModelCacheStatus,
) -> Option<&'static str> {
    if !source_ready {
        return Some("source checkout is not ready");
    }
    if options.target.requires_embeddings() && !model_cache.ready {
        return Some("embedding model cache is not ready");
    }
    if options.check && options.force_rebuild {
        return Some("force rebuild requested");
    }
    None
}

fn check_can_plan_blocked_build(options: &RuntimeSetupOptions, reason: &str) -> bool {
    options.check
        && (reason == "force rebuild requested"
            || (!options.offline
                && matches!(
                    reason,
                    "source checkout is not ready" | "embedding model cache is not ready"
                )))
}

fn build_reason(
    validation: &ArtifactValidationReport,
    source_signature: Option<&str>,
    force_rebuild: bool,
) -> String {
    if force_rebuild {
        return "force rebuild requested".to_string();
    }
    if validation.status != ValidationStatus::Ok {
        return validation.message.clone();
    }
    if source_signature.is_some() && validation.source_signature.as_deref() != source_signature {
        return "source signature changed since the artifact was built".to_string();
    }
    "artifact repair is required".to_string()
}

impl From<BuildArtifactReport> for SetupBuildReport {
    fn from(report: BuildArtifactReport) -> Self {
        Self {
            source_signature: report.source_signature,
            source_record_count: report.source_record_count,
            artifact_record_count: report.artifact_record_count,
            generated_record_count: report.generated_record_count,
            document_embedding_count: report.document_embedding_count,
        }
    }
}

fn embedding_model_cache_status(config: &EmbeddingRuntimeConfig) -> EmbeddingModelCacheStatus {
    let model_dir = config.model_dir();
    let missing_files = required_embedding_model_cache_files(config)
        .into_iter()
        .map(|file| file.local_path)
        .filter(|path| !path.is_file())
        .collect::<Vec<_>>();
    EmbeddingModelCacheStatus {
        model_dir,
        ready: missing_files.is_empty(),
        missing_files,
    }
}

fn fetch_pf2e_source(source_root: &Path) -> Result<(), String> {
    if source_root.exists() {
        if source_root.join(".git").exists() {
            let status = ProcessCommand::new("git")
                .args([
                    "-C",
                    &source_root.display().to_string(),
                    "pull",
                    "--ff-only",
                    "--quiet",
                ])
                .status()
                .map_err(|error| format!("failed to run git pull: {error}"))?;
            if status.success() {
                return Ok(());
            }
            return Err(format!(
                "failed to update PF2E source at {}",
                source_root.display()
            ));
        }
        return Err(format!(
            "source path already exists but is not a git checkout: {}",
            source_root.display()
        ));
    }
    if let Some(parent) = source_root.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create source parent directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let status = ProcessCommand::new("git")
        .args([
            "clone",
            "--quiet",
            "https://github.com/foundryvtt/pf2e.git",
            &source_root.display().to_string(),
        ])
        .status()
        .map_err(|error| format!("failed to run git clone: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to clone PF2E source into {}",
            source_root.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use atlas_embedding::EmbeddingModelId;
    use atlas_index::ArtifactMetadataSummary;

    use super::*;

    #[test]
    fn full_setup_rejects_artifact_built_for_another_supported_model() {
        let config = EmbeddingRuntimeConfig::new(EmbeddingModelId::BgeSmallEnV15, "/tmp/cache");
        let report = ArtifactValidationReport::ok(
            "/tmp/index.sqlite".to_string(),
            ArtifactMetadataSummary {
                embedding_model_id: Some("BAAI/bge-base-en-v1.5".to_string()),
                embedding_model_revision: Some("main".to_string()),
                embedding_tokenizer_id: Some("BAAI/bge-base-en-v1.5".to_string()),
                ..Default::default()
            },
        );

        let report = selected_model_validation(report, SetupTarget::Full, &config);

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, atlas_index::ValidationCode::EmbeddingMismatch);
        assert_eq!(
            report.diagnostics[0].key.as_deref(),
            Some("embedding_model_id")
        );
    }

    #[test]
    fn records_setup_accepts_artifact_built_for_another_model() {
        let config = EmbeddingRuntimeConfig::new(EmbeddingModelId::BgeSmallEnV15, "/tmp/cache");
        let report = ArtifactValidationReport::ok(
            "/tmp/index.sqlite".to_string(),
            ArtifactMetadataSummary {
                embedding_model_id: Some("BAAI/bge-base-en-v1.5".to_string()),
                embedding_model_revision: Some("main".to_string()),
                embedding_tokenizer_id: Some("BAAI/bge-base-en-v1.5".to_string()),
                ..Default::default()
            },
        );

        let report = selected_model_validation(report, SetupTarget::Records, &config);

        assert_eq!(report.status, ValidationStatus::Ok);
    }
}
