//! Explicit setup is the sole runtime path allowed to inspect or acquire the managed source snapshot.
use crate::setup_freshness::{
    capture_build_inputs, has_current_reference_preparation, source_is_fresh,
};
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
        embedding_coverage_gap: None,
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
    options: RuntimeSetupOptions,
) -> RuntimeSetupReport {
    ensure_setup_with_seed_policy(
        paths,
        options,
        &mut crate::embedding_seeds::open_published,
        atlas_ingest::SeedBuildPolicy::Strict,
    )
}

pub(crate) fn eligible_desktop_stage(
    paths: &ResolvedAtlasPaths,
) -> Result<DesktopSetupStage, String> {
    desktop_stage(paths).map(|(stage, _)| stage)
}

fn desktop_stage(
    paths: &ResolvedAtlasPaths,
) -> Result<
    (
        DesktopSetupStage,
        Option<atlas_index::RequestedReferenceConfig>,
    ),
    String,
> {
    if paths.mode != crate::ResolvedPathMode::Global || paths.source_root_explicit {
        return Err("staged setup requires the managed Atlas workspace".into());
    }
    let workspace = match atlas_storage::Workspace::existing(&paths.index_path) {
        Ok(workspace) => workspace,
        Err(atlas_storage::StorageError::Missing(_)) => {
            return Ok((DesktopSetupStage::FirstRun, None));
        }
        Err(error) => return Err(error.to_string()),
    };
    crate::workspace::register_extensions().map_err(|error| error.to_string())?;
    let read = workspace
        .read(crate::WORKSPACE_SCHEMA_VERSION)
        .map_err(|error| error.to_string())?;
    let config =
        atlas_index::read_requested_reference_config(&read).map_err(|error| error.to_string())?;
    if config.as_ref().is_some_and(|config| {
        config.source_root != paths.managed_source_root
            || config.manifest_path.is_some()
            || config.indexing_locale != "en"
    }) {
        return Err("reference configuration changed; update reference data explicitly".into());
    }
    let revision = read
        .reference_revision()
        .map_err(|error| error.to_string())?;
    let context = atlas_index::read_stored_reference_context(read.connection())
        .map_err(|error| error.to_string())?;
    match context {
        None if revision == 0 => Ok((DesktopSetupStage::FirstRun, None)),
        Some(context)
            if context.search_profiles.is_empty()
                && context.indexing_locale == "en"
                && has_current_reference_preparation(&context)
                && context.source_revision.as_deref()
                    == Some(atlas_foundry_model::generated::REFERENCE_SOURCE_COMMIT)
                && config
                    .as_ref()
                    .is_some_and(|config| !config.search_profiles.is_empty()) =>
        {
            Ok((DesktopSetupStage::Resume, config))
        }
        _ => Err("the workspace does not need staged semantic setup".into()),
    }
}

fn report_publication(
    paths: &ResolvedAtlasPaths,
    semantic: bool,
    expected_revision: u64,
    on_publish: &mut dyn FnMut(SetupPublication),
) -> Result<u64, String> {
    let index = open_workspace_index(&paths.index_path).map_err(|error| error.to_string())?;
    let instance = index.artifact_instance();
    let revision = instance.reference_revision();
    if revision != expected_revision || index.context().search_profiles.is_empty() == semantic {
        return Err("reference publication changed during setup; reopen Atlas".into());
    }
    let generation = instance.generation();
    let workspace = paths
        .index_path
        .canonicalize()
        .map_err(|error| error.to_string())?;
    drop(index);
    on_publish(SetupPublication {
        workspace,
        generation,
        semantic,
    });
    Ok(revision)
}

pub(crate) fn staged_desktop_setup(
    paths: &ResolvedAtlasPaths,
    options: RuntimeSetupOptions,
    stage: DesktopSetupStage,
    mut on_publish: impl FnMut(SetupPublication),
) -> RuntimeSetupReport {
    run_setup(
        paths,
        options,
        &mut crate::embedding_seeds::open_published,
        atlas_ingest::SeedBuildPolicy::Strict,
        Some((stage, &mut on_publish)),
    )
}

pub(crate) fn ensure_setup_with_seed_policy(
    paths: &ResolvedAtlasPaths,
    options: RuntimeSetupOptions,
    seed_source: &mut crate::embedding_seeds::SeedSource<'_>,
    seed_policy: atlas_ingest::SeedBuildPolicy,
) -> RuntimeSetupReport {
    run_setup(paths, options, seed_source, seed_policy, None)
}

fn run_setup(
    paths: &ResolvedAtlasPaths,
    mut options: RuntimeSetupOptions,
    seed_source: &mut crate::embedding_seeds::SeedSource<'_>,
    seed_policy: atlas_ingest::SeedBuildPolicy,
    mut staged: Option<(DesktopSetupStage, &mut dyn FnMut(SetupPublication))>,
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
    let mut resume_config = None;
    let _pause = if let Some((stage, _)) = &staged {
        if options.check
            || options.force_rebuild
            || !options.target.requires_embeddings()
            || options.locale.is_some()
            || !options.search_profiles.is_empty()
        {
            return configuration_failure(paths, &options, "invalid staged setup options".into());
        }
        if *stage == DesktopSetupStage::Fixture {
            if !options.offline || !paths.source_root_explicit || paths.index_path.exists() {
                return configuration_failure(
                    paths,
                    &options,
                    "staged fixture requires fresh offline setup".into(),
                );
            }
        } else {
            match desktop_stage(paths) {
                Ok((eligible, config)) if eligible == *stage => resume_config = config,
                Ok(_) => {
                    return configuration_failure(
                        paths,
                        &options,
                        "workspace setup stage changed; reopen Atlas".into(),
                    );
                }
                Err(error) => return configuration_failure(paths, &options, error),
            }
        }
        match atlas_storage::Workspace::at(&paths.index_path)
            .and_then(|workspace| workspace.pause_saved_writes())
        {
            Ok(guard) => {
                progress("saved_writes_paused", "Preparing search");
                Some(guard)
            }
            Err(error) => return configuration_failure(paths, &options, error.to_string()),
        }
    } else {
        None
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
    let configuration = match resume_config.map(Ok).unwrap_or_else(|| {
        crate::source_configuration::resolve_setup_configuration(paths, &options)
    }) {
        Ok(configuration) => configuration,
        Err(error) => return configuration_failure(paths, &options, error),
    };
    let managed_source = configuration.source_root == paths.managed_source_root;
    let mut effective_paths = paths.clone();
    effective_paths.source_root = configuration.source_root.clone();
    let paths = &effective_paths;
    options.search_profiles = configuration.search_profiles.clone();
    let use_seeds = managed_source
        && !paths.source_root_explicit
        && configuration.manifest_path.is_none()
        && configuration.indexing_locale == "en"
        && !options.build_embeddings_locally;
    if seed_policy != atlas_ingest::SeedBuildPolicy::Strict && !use_seeds {
        return configuration_failure(
            paths,
            &options,
            "seed tools require the managed English source and pinned models".into(),
        );
    }
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
    let staged_revision = if let Some((stage, on_publish)) = &mut staged {
        let result = (|| -> Result<u64, String> {
            let inputs = captured.as_mut().ok_or_else(|| {
                source_issue
                    .clone()
                    .unwrap_or_else(|| "reference source is unavailable".into())
            })?;
            let workspace = atlas_storage::Workspace::existing(&paths.index_path)
                .map_err(|error| error.to_string())?;
            if matches!(
                stage,
                DesktopSetupStage::FirstRun | DesktopSetupStage::Fixture
            ) {
                progress("build_keyword_index", "Building keyword search index");
                inputs
                    .build_keyword_stage(&workspace, crate::WORKSPACE_SCHEMA_VERSION)
                    .map_err(|error| error.to_string())?;
                report_publication(paths, false, 1, *on_publish)
            } else {
                let index =
                    open_workspace_index(&paths.index_path).map_err(|error| error.to_string())?;
                if !index.context().search_profiles.is_empty()
                    || !source_is_fresh(inputs, index.context())
                {
                    return Err("reference data changed; update reference data explicitly".into());
                }
                Ok(index.artifact_instance().reference_revision())
            }
        })();
        match result {
            Ok(revision) => Some(revision),
            Err(error) => return configuration_failure(paths, &options, error),
        }
    } else {
        None
    };
    let profile_assets = crate::profile_assets::ProfileAssets::default();
    let mut model_ready = prepare_embeddings
        && profile_error.is_none()
        && configurations.iter().all(|c| {
            profile_assets.ready(&atlas_embedding::profile_for_model(c.model).profile_id(), c)
        })
        && atlas_embedding::TextEmbeddingTokenizer::load_baseline(&paths.embedding_cache_root)
            .is_ok();
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
    let mut embedding_coverage_gap = None;
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
            let mut load = |model: &atlas_index::SourceSearchProfileIdentity,
                            scratch: &std::path::Path| {
                seed_source(model, scratch, options.offline)
            };
            let result = (|| -> Result<BuildArtifactReport, atlas_ingest::IngestError> {
                let inputs = captured.take().ok_or_else(|| {
                    atlas_ingest::IngestError::ArtifactWriteFailed(
                        "source inputs were not captured".into(),
                    )
                })?;
                if let Some(revision) = staged_revision {
                    let workspace = atlas_storage::Workspace::existing(&paths.index_path)?;
                    inputs.build_semantic_stage(
                        &workspace,
                        crate::WORKSPACE_SCHEMA_VERSION,
                        revision,
                        use_seeds.then_some(&mut load),
                    )
                } else {
                    let workspace =
                        crate::initialize_workspace(&paths.index_path).map_err(|error| {
                            atlas_ingest::IngestError::ArtifactWriteFailed(error.to_string())
                        })?;
                    inputs.build_with_seed_policy(
                        &workspace,
                        crate::WORKSPACE_SCHEMA_VERSION,
                        use_seeds.then_some(&mut load),
                        seed_policy,
                    )
                }
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
                    let (kind, reason) = match &e {
                        atlas_ingest::IngestError::EmbeddingCoverageGap(gap) => {
                            embedding_coverage_gap = Some((**gap).clone());
                            (SetupActionKind::PrepareEmbeddingSeed, e.to_string())
                        }
                        atlas_ingest::IngestError::EmbeddingSeedUnavailable { .. } => (
                            SetupActionKind::PrepareEmbeddingSeed,
                            format!(
                                "{e}. Retry setup or use --build-embeddings-locally to generate missing embeddings."
                            ),
                        ),
                        _ => (SetupActionKind::BuildIndex, e.to_string()),
                    };
                    actions.push(SetupAction::with_reason(
                        kind,
                        SetupActionStatus::Failed,
                        reason,
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
    if staged.is_none() && !options.check && !build_failed && build.is_none() {
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
    if semantic_ready
        && let (Some(revision), Some((_, on_publish))) = (staged_revision, staged)
        && let Err(error) = report_publication(paths, true, revision + 1, on_publish)
    {
        return configuration_failure(paths, &options, error);
    }
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
        embedding_coverage_gap,
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
