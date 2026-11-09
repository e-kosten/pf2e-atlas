//! Explicit setup is the sole runtime path allowed to inspect or acquire the managed source snapshot.
use crate::setup_freshness::{capture_build_inputs, source_is_fresh};
use crate::workspace::open_workspace_index;
use crate::{ResolvedAtlasPaths, setup_model::*};
use atlas_embedding::{
    EmbeddingRuntimeConfig, prepare_embedding_model_cache, required_embedding_model_cache_files,
};
use atlas_ingest::{BuildArtifactOptions, BuildArtifactReport};
fn progress(phase: &str, message: &str) {
    tracing::info!(target:"atlas_progress",phase,"{message}");
}
fn configuration_failure(
    paths: &ResolvedAtlasPaths,
    options: &RuntimeSetupOptions,
    error: String,
) -> RuntimeSetupReport {
    RuntimeSetupReport {
        target: options.target,
        ready: false,
        path_mode: paths.mode.as_str(),
        repo_root: paths.repo_root.as_ref().map(|p| p.display().to_string()),
        offline: options.offline,
        check: options.check,
        force_rebuild: options.force_rebuild,
        checks: vec![SetupAction::with_reason(
            SetupActionKind::ValidateIndex,
            SetupActionStatus::Failed,
            &error,
        )],
        actions: vec![],
        readiness: SetupReadiness {
            source: SetupReadinessItem::not_ready(false, &error),
            embedding_model: SetupReadinessItem::not_ready(
                options.target.requires_embeddings(),
                &error,
            ),
            records: SetupReadinessItem::not_ready(true, &error),
            semantic_search: SetupReadinessItem::not_ready(
                options.target.requires_embeddings(),
                &error,
            ),
        },
        paths: SetupPathsReport {
            source: paths.source_root.display().to_string(),
            embedding_cache: paths.embedding_cache_root.display().to_string(),
            index: paths.index_path.display().to_string(),
        },
        search_profiles: vec![],
        build: None,
    }
}
fn stored_reference(
    path: &std::path::Path,
) -> Result<
    (
        atlas_index::SourceArtifactBuildContext,
        atlas_index::ReferenceUpgrade,
    ),
    atlas_index::IndexError,
> {
    let workspace = atlas_storage::Workspace::at(path)?;
    let read = workspace.read(crate::WORKSPACE_SCHEMA_VERSION)?;
    let context =
        atlas_index::read_stored_reference_context(read.connection())?.ok_or_else(|| {
            atlas_index::IndexError::Unavailable("reference context is absent".into())
        })?;
    let upgrade = atlas_index::classify_reference_upgrade(read.connection())?;
    Ok((context, upgrade))
}

pub(crate) fn ensure_setup(
    paths: &ResolvedAtlasPaths,
    mut options: RuntimeSetupOptions,
) -> RuntimeSetupReport {
    // Emit progress before configuration and workspace checks, which can take
    // noticeable time on a fresh install or a large existing workspace.
    progress("check_setup", "Checking setup");
    let _setup = if options.check {
        None
    } else {
        match atlas_storage::Workspace::at(&paths.index_path)
            .and_then(|workspace| workspace.setup_guard())
        {
            Ok(guard) => Some(guard),
            Err(atlas_storage::StorageError::Busy) => {
                return configuration_failure(paths, &options, "setup already running".into());
            }
            Err(error) => return configuration_failure(paths, &options, error.to_string()),
        }
    };
    let _upgrade = if !options.check && paths.index_path.is_file() {
        let workspace = match atlas_storage::Workspace::existing(&paths.index_path) {
            Ok(workspace) => workspace,
            Err(error) => return configuration_failure(paths, &options, error.to_string()),
        };
        match workspace.check_migration_history(crate::workspace::RELEASE_HISTORY) {
            Ok(version) if version < crate::WORKSPACE_SCHEMA_VERSION => {
                match workspace.service_guard() {
                    Ok(guard) => Some(guard),
                    Err(atlas_storage::StorageError::Busy) => {
                        return configuration_failure(
                            paths, &options,
                            "close Atlas desktop and atlas web sessions for this workspace before upgrading".into(),
                        );
                    }
                    Err(error) => return configuration_failure(paths, &options, error.to_string()),
                }
            }
            Ok(_) => None,
            Err(error) => return configuration_failure(paths, &options, error.to_string()),
        }
    } else {
        None
    };
    let configuration =
        match crate::source_configuration::resolve_setup_configuration(paths, &options) {
            Ok(configuration) => configuration,
            Err(error) => return configuration_failure(paths, &options, error),
        };
    let managed_source = configuration.source_root == paths.managed_source_root;
    let mut effective_paths = paths.clone();
    effective_paths.source_root = configuration.source_root.clone();
    let paths = &effective_paths;
    options.search_profiles = configuration.search_profiles.clone();
    let prepare_embeddings = !options.search_profiles.is_empty();
    let mut checks = Vec::new();
    let mut actions = Vec::new();
    let mut source_ready = paths.source_root.is_dir();
    let mut configurations = Vec::new();
    let mut profile_error = None;
    let mut seen = std::collections::BTreeSet::new();
    for id in &options.search_profiles {
        match atlas_embedding::search_profile(id) {
            Ok(profile) if seen.insert(id.clone()) => configurations.push(
                EmbeddingRuntimeConfig::new(profile.model, &paths.embedding_cache_root),
            ),
            _ => profile_error = Some(format!("unsupported or duplicate search profile: {id}")),
        }
    }
    configurations.sort_by_key(|c| atlas_embedding::profile_for_model(c.model).profile_id());
    if options.target.requires_embeddings() && configurations.is_empty() {
        profile_error = Some("full setup requires at least one search profile".into());
    }
    if let Some(reason) = &profile_error {
        actions.push(SetupAction::with_reason(
            SetupActionKind::PrepareEmbeddingModel,
            SetupActionStatus::Failed,
            reason,
        ));
    }
    let profile_assets = crate::profile_assets::ProfileAssets::default();
    let mut model_ready = prepare_embeddings
        && profile_error.is_none()
        && configurations.iter().all(|c| {
            profile_assets.ready(&atlas_embedding::profile_for_model(c.model).profile_id(), c)
        })
        && atlas_embedding::TextEmbeddingTokenizer::load_baseline(&paths.embedding_cache_root)
            .is_ok();
    let mut source_issue = None;
    let mut captured = None;
    if !source_ready && managed_source && profile_error.is_none() {
        if options.check || options.offline {
            actions.push(SetupAction::with_reason(
                SetupActionKind::FetchSource,
                if options.offline {
                    SetupActionStatus::Blocked
                } else {
                    SetupActionStatus::Planned
                },
                format!(
                    "pinned PF2e {} reference {} is missing",
                    atlas_foundry_model::generated::REFERENCE_SOURCE_VERSION,
                    atlas_foundry_model::generated::REFERENCE_SOURCE_COMMIT
                ),
            ));
        } else {
            progress("fetch_source", "Preparing Foundry source");
            match crate::source_acquisition::acquire(&paths.source_root) {
                Ok(()) => {
                    source_ready = true;
                    actions.push(SetupAction::new(
                        SetupActionKind::FetchSource,
                        SetupActionStatus::Done,
                    ));
                }
                Err(error) => {
                    source_issue = Some(error.clone());
                    actions.push(SetupAction::with_reason(
                        SetupActionKind::FetchSource,
                        SetupActionStatus::Failed,
                        error,
                    ));
                }
            }
        }
    }
    if source_ready {
        let validation = if managed_source {
            crate::source_acquisition::validate(&paths.source_root).map_err(|error| (error, false))
        } else {
            Ok(())
        }
        .and_then(|()| {
            capture_build_inputs(
                BuildArtifactOptions {
                    source_root: paths.source_root.clone(),
                    output_path: paths.index_path.clone(),
                    manifest_path: configuration.manifest_path.clone(),
                    locale: configuration.indexing_locale.clone(),
                    search_profiles: configurations.clone(),
                    reuse_passage_layouts: !options.force_rebuild,
                    embedding_batch_size: options.embedding_batch_size,
                },
                crate::source_acquisition::revision(
                    managed_source,
                    configuration.manifest_path.is_some(),
                ),
            )
            .map_err(|error| {
                let failed = !matches!(error, atlas_ingest::IngestError::SourceIncompatible(_));
                (error.to_string(), failed)
            })
        });
        match validation {
            Ok(inputs) => captured = Some(inputs),
            Err((error, failed)) => {
                source_ready = false;
                source_issue = Some(error.clone());
                checks.push(SetupAction::with_reason(
                    SetupActionKind::AnalyzeSource,
                    if failed {
                        SetupActionStatus::Failed
                    } else {
                        SetupActionStatus::Blocked
                    },
                    error,
                ));
            }
        }
    }
    if source_ready && prepare_embeddings && !model_ready && profile_error.is_none() {
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
            progress("prepare_embedding_model", "Preparing search models");
            match (|| {
                atlas_embedding::prepare_baseline_tokenizer_cache(&paths.embedding_cache_root)?;
                for config in &configurations {
                    prepare_embedding_model_cache(config)?;
                }
                Ok::<_, atlas_embedding::EmbeddingError>(())
            })() {
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
    progress("check_workspace", "Checking workspace index");
    let locale = configuration.indexing_locale.clone();
    let prior = stored_reference(&paths.index_path);
    let freshness: Result<bool, String> = if options.force_rebuild {
        Ok(false)
    } else if let Some(inputs) = &captured {
        Ok(prior
            .as_ref()
            .is_ok_and(|(context, _)| source_is_fresh(inputs, context)))
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
            if options.force_rebuild {
                "source rebuild explicitly requested"
            } else {
                "reference inputs or glossary preparation differ from the indexed publication"
            },
        )),
        Err(e) => checks.push(SetupAction::with_reason(
            SetupActionKind::AnalyzeSource,
            SetupActionStatus::Blocked,
            e,
        )),
    }
    let profiles_match = prior.as_ref().is_ok_and(|(context, _)| {
        context.search_profiles.len() == options.search_profiles.len()
            && options.search_profiles.iter().all(|id| {
                atlas_embedding::search_profile(id).is_ok_and(|spec| {
                    context.search_profile(id).is_ok_and(|stored| {
                        stored == &atlas_index::SourceSearchProfileIdentity::from_spec(spec)
                    })
                })
            })
    });
    let source_build_required = source_issue.is_some()
        || options.force_rebuild
        || profile_error.is_some()
        || prior.is_err()
        || !profiles_match
        || freshness.as_ref().is_ok_and(|fresh| !*fresh)
        || (source_ready && freshness.is_err())
        || prior.as_ref().is_ok_and(|(context, upgrade)| {
            context.indexing_locale != locale
                || matches!(
                    upgrade,
                    atlas_index::ReferenceUpgrade::SourcePreparation
                        | atlas_index::ReferenceUpgrade::SourceReplacement
                )
        });
    if !source_build_required
        && !options.check
        && prior
            .as_ref()
            .is_ok_and(|(_, upgrade)| *upgrade != atlas_index::ReferenceUpgrade::Current)
    {
        let upgraded = (|| -> Result<(), String> {
            let workspace =
                crate::initialize_workspace(&paths.index_path).map_err(|e| e.to_string())?;
            atlas_index::ensure_reference_projections(&workspace, crate::WORKSPACE_SCHEMA_VERSION)
                .map_err(|e| e.to_string())?;
            Ok(())
        })();
        if let Err(error) = upgraded {
            return configuration_failure(paths, &options, error);
        }
    }
    let old = open_workspace_index(&paths.index_path).map(|reader| reader.context().clone());
    let old_semantic = if prepare_embeddings {
        open_workspace_index(&paths.index_path)
            .and_then(|index| {
                index.require_vectors()?;
                Ok(index)
            })
            .is_ok_and(|index| {
                index.context().search_profiles.len() == options.search_profiles.len()
                    && options.search_profiles.iter().all(|id| {
                        atlas_embedding::search_profile(id).is_ok_and(|spec| {
                            index.context().search_profile(id).is_ok_and(|stored| {
                                stored == &atlas_index::SourceSearchProfileIdentity::from_spec(spec)
                            })
                        })
                    })
            })
    } else {
        true
    };
    let needs_build = source_build_required || old.is_err() || !old_semantic;
    let mut build = None;
    let mut build_failed = false;
    if needs_build {
        if options.check {
            actions.push(SetupAction::with_reason(
                SetupActionKind::BuildIndex,
                SetupActionStatus::Planned,
                "artifact requires rebuild",
            ));
        } else if profile_error.is_some() || !source_ready || (prepare_embeddings && !model_ready) {
            actions.push(SetupAction::with_reason(
                SetupActionKind::BuildIndex,
                SetupActionStatus::Blocked,
                "source or required embedding assets are unavailable",
            ));
        } else {
            progress("build_index", "Building search index");
            let result = (|| -> Result<BuildArtifactReport, String> {
                let workspace =
                    crate::initialize_workspace(&paths.index_path).map_err(|e| e.to_string())?;
                captured
                    .take()
                    .ok_or("source inputs were not captured")?
                    .build(&workspace, crate::WORKSPACE_SCHEMA_VERSION)
                    .map_err(|e| e.to_string())
            })();
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
    if !options.check && !build_failed && build.is_none() {
        let reclaimed = (|| -> Result<(), String> {
            let workspace =
                crate::initialize_workspace(&paths.index_path).map_err(|e| e.to_string())?;
            let owner = workspace.update_owner().map_err(|e| e.to_string())?;
            atlas_index::recover_reference_publication(&owner, crate::WORKSPACE_SCHEMA_VERSION)
                .map_err(|e| e.to_string())?;
            owner
                .reclaim_space(crate::WORKSPACE_SCHEMA_VERSION)
                .map_err(|e| {
                    format!("Reference data is retained; database space was not reclaimed: {e}")
                })?;
            Ok(())
        })();
        if let Err(error) = reclaimed {
            return configuration_failure(paths, &options, error);
        }
    }
    // Publication updates reference rows transactionally, preserving saved domains.
    let current = open_workspace_index(&paths.index_path);
    let records_ready = current.is_ok() && !build_failed && (!needs_build || build.is_some());
    let semantic_ready = records_ready
        && model_ready
        && open_workspace_index(&paths.index_path)
            .and_then(|index| {
                index.require_vectors()?;
                Ok(index)
            })
            .is_ok_and(|index| {
                options
                    .search_profiles
                    .iter()
                    .all(|id| index.context().search_profile(id).is_ok())
            });
    checks.push(match &current{Ok(_)=>SetupAction::with_reason(SetupActionKind::ValidateIndex,SetupActionStatus::Done,"cheap executable schema/catalog check; full snapshot validation is a separate operation"),Err(e)=>SetupAction::with_reason(SetupActionKind::ValidateIndex,SetupActionStatus::Blocked,e.to_string())});
    let readiness = |ready: bool, required: bool, reason: &str| {
        if ready {
            SetupReadinessItem::ready(required)
        } else {
            SetupReadinessItem::not_ready(required, reason)
        }
    };
    let report = RuntimeSetupReport {
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
            source: readiness(
                source_ready,
                false,
                source_issue
                    .as_deref()
                    .unwrap_or("source checkout is unavailable"),
            ),
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
        search_profiles: configurations
            .iter()
            .map(|config| {
                let profile = atlas_embedding::profile_for_model(config.model);
                SetupEmbeddingReport {
                    profile_id: profile.profile_id(),
                    stored: current.as_ref().is_ok_and(|index| {
                        index
                            .context()
                            .search_profile(&profile.profile_id())
                            .is_ok()
                    }),
                    model: config.model_spec().model_id.into(),
                    model_path: config.model_dir().display().to_string(),
                    cache_root: paths.embedding_cache_root.display().to_string(),
                    ready: options.target.requires_embeddings()
                        && profile_assets.ready(&profile.profile_id(), config),
                    missing_files: required_embedding_model_cache_files(config)
                        .iter()
                        .filter(|file| !file.local_path.is_file())
                        .map(|file| file.local_path.display().to_string())
                        .collect(),
                }
            })
            .collect(),
        build,
    };
    tracing::info!(target:"atlas_progress", complete=true, "Setup finished");
    report
}
impl From<BuildArtifactReport> for SetupBuildReport {
    fn from(r: BuildArtifactReport) -> Self {
        Self {
            admission: r.admission,
            profiles: r.profiles,
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
