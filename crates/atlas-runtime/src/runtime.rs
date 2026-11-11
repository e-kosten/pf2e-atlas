use std::path::Path;

use crate::error::RuntimeError;
use crate::paths::{AtlasPathMode, AtlasPathOverrides, ResolvedAtlasPaths, resolve_atlas_paths};
use crate::setup;
use crate::setup_clean;
use crate::setup_model::{
    RuntimeSetupCleanOptions, RuntimeSetupCleanReport, RuntimeSetupOptions, RuntimeSetupReport,
};

#[derive(Debug, Clone)]
pub struct AtlasRuntimeOptions {
    /// None selects the sole stored profile, or BGE Base when both are stored.
    pub default_search_profile: Option<atlas_domain::SearchProfileId>,
    pub path_mode: AtlasPathMode,
    pub overrides: AtlasPathOverrides,
}

fn rebuild_source_error(
    error: impl std::fmt::Display,
    source: &Path,
    manifest: Option<&Path>,
) -> String {
    let manifest = manifest
        .map(|path| format!(" and manifest {}", path.display()))
        .unwrap_or_default();
    format!(
        "reference rebuild using source {}{manifest} failed: {error}. To change the repair inputs, use --source or --manifest",
        source.display()
    )
}

#[cfg(feature = "seed-tools")]
fn completed_seed_build(report: RuntimeSetupReport) -> Result<crate::SetupBuildReport, String> {
    use crate::SetupActionKind::{BuildIndex, ValidateIndex};
    use crate::SetupActionStatus::{Blocked, Failed};

    if !report.ready {
        return Err(report
            .actions
            .into_iter()
            .chain(report.checks)
            .filter(|action| matches!(action.status, Failed | Blocked))
            .filter_map(|action| {
                let generic_block =
                    action.status == Blocked && matches!(action.kind, BuildIndex | ValidateIndex);
                action.reason.map(|reason| (generic_block, reason))
            })
            .min_by_key(|(generic_block, _)| *generic_block)
            .map(|(_, reason)| reason)
            .unwrap_or_else(|| "embedding setup did not become ready".into()));
    }
    report
        .build
        .ok_or_else(|| "embedding setup did not run a source build".into())
}

impl Default for AtlasRuntimeOptions {
    fn default() -> Self {
        Self {
            default_search_profile: None,
            path_mode: AtlasPathMode::Global,
            overrides: AtlasPathOverrides::default(),
        }
    }
}

pub struct AtlasRuntime {
    paths: ResolvedAtlasPaths,
    pub(crate) default_profile: Option<atlas_domain::SearchProfileId>,
    pub(crate) inference: atlas_embedding::SharedQueryInference,
    pub(crate) profile_assets: crate::profile_assets::ProfileAssets,
    pub(crate) native_neighbor_reuse: std::sync::Arc<atlas_search::NativeNeighborReuse>,
}

impl AtlasRuntime {
    pub fn resolve(options: AtlasRuntimeOptions) -> Result<Self, RuntimeError> {
        if let Some(id) = &options.default_search_profile {
            atlas_embedding::search_profile(id).map_err(|error| {
                RuntimeError::unsupported_search_profile(format!(
                    "unsupported default search profile `{}`: {error}",
                    id
                ))
            })?;
        }
        let paths = resolve_atlas_paths(options.path_mode, options.overrides)?;
        Ok(Self {
            inference: atlas_embedding::SharedQueryInference::new(&paths.embedding_cache_root),
            profile_assets: Default::default(),
            native_neighbor_reuse: Default::default(),
            default_profile: options.default_search_profile,
            paths,
        })
    }

    pub fn paths(&self) -> &ResolvedAtlasPaths {
        &self.paths
    }
    pub fn requested_search_profile(&self) -> Option<&atlas_domain::SearchProfileId> {
        self.default_profile.as_ref()
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn native_neighbor_reuse_stats(&self) -> atlas_search::NativeNeighborReuseStats {
        self.native_neighbor_reuse.stats()
    }

    pub fn workspace_location(&self) -> Result<atlas_storage::Workspace, RuntimeError> {
        Ok(atlas_storage::Workspace::at(self.index_path())?)
    }

    pub fn source_root(&self) -> &Path {
        &self.paths.source_root
    }

    pub fn embedding_cache_root(&self) -> &Path {
        &self.paths.embedding_cache_root
    }

    pub fn index_path(&self) -> &Path {
        &self.paths.index_path
    }

    /// Authenticate an existing workspace without changing its format or projections.
    pub fn check_workspace_if_present(&self) -> Result<(), RuntimeError> {
        if let Some(workspace) = self.check_workspace_history_if_present(false)? {
            workspace.read(crate::WORKSPACE_SCHEMA_VERSION)?;
        }
        Ok(())
    }

    fn check_workspace_history_if_present(
        &self,
        allow_empty: bool,
    ) -> Result<Option<atlas_storage::Workspace>, RuntimeError> {
        crate::workspace::register_extensions()?;
        let workspace = match atlas_storage::Workspace::existing(self.index_path()) {
            Ok(workspace) => workspace,
            Err(atlas_storage::StorageError::Missing(_)) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let version = workspace.check_migration_history(crate::workspace::RELEASE_HISTORY)?;
        if version < crate::WORKSPACE_SCHEMA_VERSION && !(version == 0 && allow_empty) {
            return Err(RuntimeError::workspace_upgrade_required(version));
        }
        Ok(Some(workspace))
    }

    #[cfg(test)]
    /// Apply executable-owned schema and projection upgrades before serving
    /// requests. An absent workspace remains a setup concern.
    pub fn upgrade_workspace_if_present(&self) -> Result<bool, RuntimeError> {
        let Some(workspace) = self.admit_workspace_if_present()? else {
            return Ok(false);
        };
        Ok(atlas_index::ensure_reference_projections(
            &workspace,
            crate::WORKSPACE_SCHEMA_VERSION,
        )?)
    }

    #[cfg(test)]
    /// Admit and upgrade saved workspace schema without reference projection work.
    pub fn admit_workspace_if_present(
        &self,
    ) -> Result<Option<atlas_storage::Workspace>, RuntimeError> {
        if self
            .index_path()
            .try_exists()
            .map_err(atlas_storage::StorageError::from)?
        {
            Ok(Some(crate::initialize_workspace(self.index_path())?))
        } else {
            Ok(None)
        }
    }

    pub fn open_index(&self) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexError> {
        crate::workspace::open_workspace_index(&self.paths.index_path)
    }

    #[cfg(feature = "seed-tools")]
    pub fn verify_local_embedding_seeds(
        &self,
        files: &[crate::LocalEmbeddingSeed],
    ) -> Result<(crate::SetupBuildReport, Vec<atlas_domain::SearchProfileId>), String> {
        let mut loaded = Vec::new();
        let report = setup::ensure_setup_with_seed_policy(
            &self.paths,
            RuntimeSetupOptions {
                search_profiles: files.iter().map(|entry| entry.profile.clone()).collect(),
                offline: true,
                ..Default::default()
            },
            &mut |model, scratch, _| {
                let seed = crate::embedding_seeds::open_local(files, model, scratch)?;
                loaded.push(model.profile_id.clone());
                Ok(Some(seed))
            },
            atlas_ingest::SeedBuildPolicy::Strict,
        );
        let build = completed_seed_build(report)?;
        if build.inferred_inputs != 0 || loaded.len() != files.len() {
            return Err(
                "verification did not consume every supplied seed with zero corpus inference"
                    .into(),
            );
        }
        Ok((build, loaded))
    }

    pub fn audit_embedding_seed(
        &self,
        options: RuntimeSetupOptions,
    ) -> Result<RuntimeSetupReport, String> {
        if options.search_profiles.len() != 1
            || self
                .paths
                .index_path
                .try_exists()
                .map_err(|error| error.to_string())?
        {
            return Err(
                "embedding coverage audit requires exactly one profile and a fresh workspace"
                    .into(),
            );
        }
        Ok(setup::ensure_setup_with_seed_policy(
            &self.paths,
            options,
            &mut crate::embedding_seeds::open_published,
            atlas_ingest::SeedBuildPolicy::Audit,
        ))
    }

    #[cfg(feature = "seed-tools")]
    pub fn refresh_embedding_seeds(
        &self,
        options: RuntimeSetupOptions,
        files: &[crate::LocalEmbeddingSeed],
        rebuild: &[atlas_domain::SearchProfileId],
    ) -> Result<crate::SetupBuildReport, String> {
        completed_seed_build(setup::ensure_setup_with_seed_policy(
            &self.paths,
            options,
            &mut |model, scratch, _| {
                if rebuild.contains(&model.profile_id) {
                    return Ok(None);
                }
                crate::embedding_seeds::open_local(files, model, scratch).map(Some)
            },
            atlas_ingest::SeedBuildPolicy::Refresh,
        ))
    }

    pub fn ensure_setup(&self, options: RuntimeSetupOptions) -> RuntimeSetupReport {
        setup::ensure_setup(&self.paths, options)
    }

    pub fn staged_desktop_setup(
        &self,
        options: RuntimeSetupOptions,
        stage: crate::DesktopSetupStage,
        on_publish: impl FnMut(crate::SetupPublication),
    ) -> RuntimeSetupReport {
        setup::staged_desktop_setup(&self.paths, options, stage, on_publish)
    }

    pub fn desktop_setup_stage(&self) -> Option<crate::DesktopSetupStage> {
        setup::eligible_desktop_stage(&self.paths).ok()
    }

    /// Resolve and persist preparation intent before a manual build. Repeated
    /// model selections add to the configured set; omission never removes one.
    pub fn configure_reference_build(
        &self,
        options: &RuntimeSetupOptions,
        manifest: Option<&Path>,
    ) -> Result<atlas_index::RequestedReferenceConfig, String> {
        self.check_workspace_history_if_present(true)
            .map_err(|error| error.to_string())?;
        let configuration =
            crate::source_configuration::resolve_configuration(&self.paths, options, manifest)?;
        if configuration.source_root == self.paths.managed_source_root {
            crate::source_acquisition::validate(&configuration.source_root)?;
        }
        atlas_ingest::validate_source_compatibility(
            &configuration.source_root,
            configuration.manifest_path.as_deref(),
        )
        .map_err(|e| e.to_string())?;
        Ok(configuration)
    }

    /// Attribution for a source previously verified by runtime's managed-source admission.
    pub fn verified_source_revision(
        &self,
        configuration: &atlas_index::RequestedReferenceConfig,
    ) -> atlas_ingest::SourceRevision<'static> {
        crate::source_acquisition::revision(
            configuration.source_root == self.paths.managed_source_root,
            configuration.manifest_path.is_some(),
        )
    }

    /// Rebuild generated reference data in an existing configured workspace.
    pub fn rebuild_reference(
        &self,
        source: Option<&Path>,
        manifest: Option<&Path>,
    ) -> Result<atlas_ingest::BuildArtifactReport, String> {
        let paths = self.paths();
        if !paths.index_path.is_file() {
            return Err(format!(
                "workspace does not exist at {}",
                paths.index_path.display()
            ));
        }
        crate::workspace::register_extensions().map_err(|e| e.to_string())?;
        let workspace =
            atlas_storage::Workspace::at(&paths.index_path).map_err(|e| e.to_string())?;
        let owner = workspace.update_owner().map_err(|e| e.to_string())?;
        let (repair, mut config) = crate::workspace::begin_reference_repair(&owner)?;
        if let Some(source) = source {
            config.source_root = std::path::absolute(source).map_err(|e| e.to_string())?;
        }
        if let Some(manifest) = manifest {
            config.manifest_path = Some(std::path::absolute(manifest).map_err(|e| e.to_string())?);
        }
        let source_root = config.source_root.clone();
        let manifest_path = config.manifest_path.clone();
        if config.source_root == paths.managed_source_root {
            crate::source_acquisition::validate(&config.source_root).map_err(|error| {
                rebuild_source_error(error, &source_root, manifest_path.as_deref())
            })?;
        }
        let revision = self.verified_source_revision(&config);
        let search_profiles = config
            .search_profiles
            .into_iter()
            .map(|id| {
                atlas_embedding::search_profile(&id)
                    .map(|profile| {
                        atlas_embedding::EmbeddingRuntimeConfig::new(
                            profile.model,
                            &paths.embedding_cache_root,
                        )
                    })
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        atlas_ingest::rebuild_artifact(
            repair,
            atlas_ingest::BuildArtifactOptions {
                source_root: config.source_root,
                output_path: paths.index_path.clone(),
                manifest_path: config.manifest_path,
                locale: config.indexing_locale,
                search_profiles,
                reuse_passage_layouts: false,
                embedding_batch_size: 32,
            },
            revision,
        )
        .map_err(|error| {
            use atlas_ingest::IngestError;
            match error {
                IngestError::SourceUnavailable(_)
                | IngestError::SourceIncompatible(_)
                | IngestError::ManifestParseFailed(_)
                | IngestError::RecordParseFailed(_)
                | IngestError::SourceSelectionFailed(_)
                | IngestError::LocalizationFailed(_)
                | IngestError::NoRecordsLoaded => {
                    rebuild_source_error(error, &source_root, manifest_path.as_deref())
                }
                _ => error.to_string(),
            }
        })
    }

    pub fn clean_setup(&self, options: RuntimeSetupCleanOptions) -> RuntimeSetupCleanReport {
        setup_clean::clean_setup(&self.paths, options)
    }

    pub fn validate_index_report(
        &self,
        target: crate::SetupTarget,
    ) -> Result<atlas_index::ArtifactValidationReport, atlas_index::IndexError> {
        let index = if target.requires_embeddings() {
            self.open_search_index()?
        } else {
            self.open_index()?
        };
        atlas_index::validate_reference_with_token_count_digests(
            index.workspace_session().connection(),
        )
    }
    pub fn check_index_report(
        &self,
        target: crate::SetupTarget,
    ) -> Result<atlas_index::SourceArtifactStatistics, atlas_index::IndexError> {
        let index = if target.requires_embeddings() {
            self.open_search_index()?
        } else {
            self.open_index()?
        };
        index.statistics()
    }

    pub(crate) fn open_search_index(
        &self,
    ) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexError> {
        let index = self.open_index()?;
        if let Some(reason) = index.capabilities().vector_unavailable_reason {
            return Err(atlas_index::IndexError::Unavailable(reason));
        }
        Ok(index)
    }
}
