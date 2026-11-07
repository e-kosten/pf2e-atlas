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

    pub fn source_root(&self) -> &Path {
        &self.paths.source_root
    }

    pub fn embedding_cache_root(&self) -> &Path {
        &self.paths.embedding_cache_root
    }

    pub fn index_path(&self) -> &Path {
        &self.paths.index_path
    }

    /// Apply executable-owned schema and projection upgrades before serving
    /// requests. An absent workspace remains a setup concern.
    pub fn upgrade_workspace_if_present(&self) -> Result<bool, RuntimeError> {
        if !self.index_path().exists() {
            return Ok(false);
        }
        let workspace = crate::initialize_workspace(self.index_path())?;
        Ok(atlas_index::ensure_reference_projections(
            &workspace,
            crate::WORKSPACE_SCHEMA_VERSION,
        )?)
    }

    pub fn open_index(&self) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexError> {
        crate::workspace::open_workspace_index(&self.paths.index_path)
    }

    pub fn ensure_setup(&self, options: RuntimeSetupOptions) -> RuntimeSetupReport {
        setup::ensure_setup(&self.paths, options)
    }

    /// Resolve and persist preparation intent before a manual build. Repeated
    /// model selections add to the configured set; omission never removes one.
    pub fn configure_reference_build(
        &self,
        options: &RuntimeSetupOptions,
        manifest: Option<&Path>,
    ) -> Result<atlas_index::RequestedReferenceConfig, String> {
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

    /// Rebuild generated reference data in an existing configured workspace.
    pub fn rebuild_reference(&self) -> Result<atlas_ingest::BuildArtifactReport, String> {
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
        let config = crate::workspace::read_rebuild_configuration(&owner)?;
        if config.source_root == paths.managed_source_root {
            crate::source_acquisition::validate(&config.source_root)?;
        }
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
            &owner,
            crate::WORKSPACE_SCHEMA_VERSION,
            atlas_ingest::BuildArtifactOptions {
                source_root: config.source_root,
                output_path: paths.index_path.clone(),
                manifest_path: config.manifest_path,
                locale: config.indexing_locale,
                search_profiles,
                reuse_embeddings: false,
                reuse_passage_layouts: false,
                embedding_batch_size: 32,
            },
        )
        .map_err(|e| e.to_string())
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
        atlas_index::validate_reference(index.workspace_session().connection())
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
