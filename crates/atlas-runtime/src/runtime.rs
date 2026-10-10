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
    pub path_mode: AtlasPathMode,
    pub overrides: AtlasPathOverrides,
}

impl Default for AtlasRuntimeOptions {
    fn default() -> Self {
        Self {
            path_mode: AtlasPathMode::Global,
            overrides: AtlasPathOverrides::default(),
        }
    }
}

pub struct AtlasRuntime {
    paths: ResolvedAtlasPaths,
}

impl AtlasRuntime {
    pub fn resolve(options: AtlasRuntimeOptions) -> Result<Self, RuntimeError> {
        Ok(Self {
            paths: resolve_atlas_paths(options.path_mode, options.overrides)?,
        })
    }

    pub fn paths(&self) -> &ResolvedAtlasPaths {
        &self.paths
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

    pub fn local_state_path(&self) -> &Path {
        &self.paths.local_state_path
    }

    pub fn open_index(&self) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexError> {
        atlas_index::SqliteIndexReader::open_read_only(&self.paths.index_path)
    }

    pub fn ensure_setup(&self, options: RuntimeSetupOptions) -> RuntimeSetupReport {
        setup::ensure_setup(&self.paths, options)
    }

    pub fn clean_setup(&self, options: RuntimeSetupCleanOptions) -> RuntimeSetupCleanReport {
        setup_clean::clean_setup(&self.paths, options)
    }

    pub fn validate_index_report(
        &self,
        target: crate::SetupTarget,
    ) -> Result<atlas_index::ArtifactValidationReport, atlas_index::IndexError> {
        if target.requires_embeddings() {
            self.open_search_index()?;
        }
        atlas_index::validate_artifact(&self.paths.index_path)
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
        atlas_index::SqliteIndexReader::open_read_only_with_vectors(&self.paths.index_path)
    }
}
