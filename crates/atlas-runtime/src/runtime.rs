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

    pub fn open_index(
        &self,
    ) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexValidationError> {
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
        target: atlas_index::ValidationTarget,
    ) -> atlas_index::ArtifactValidationReport {
        if matches!(
            target,
            atlas_index::ValidationTarget::Full | atlas_index::ValidationTarget::EmbeddingsOnly
        ) {
            return self.validate_vector_target_report(target);
        }
        match self.open_index() {
            Ok(index) => index.validate_target_report(target),
            Err(error) => atlas_index::validation_report_for_error(&self.paths.index_path, error),
        }
    }

    pub fn check_index_report(
        &self,
        target: atlas_index::ValidationTarget,
    ) -> atlas_index::ArtifactValidationReport {
        let base_report = match self.open_index() {
            Ok(index) => index.check_report(),
            Err(error) => {
                return atlas_index::validation_report_for_error(&self.paths.index_path, error);
            }
        };
        if base_report.status != atlas_index::ValidationStatus::Ok
            || matches!(target, atlas_index::ValidationTarget::BaseOnly)
        {
            return base_report;
        }
        match self.open_search_index() {
            Ok(index) => index.check_embedding_readiness_report(),
            Err(error) => match self.open_index() {
                Ok(index) => index.vector_extension_unavailable_report(
                    atlas_index::ValidationTarget::EmbeddingsOnly,
                    error.to_string(),
                ),
                Err(base_error) => {
                    atlas_index::validation_report_for_error(&self.paths.index_path, base_error)
                }
            },
        }
    }

    fn validate_vector_target_report(
        &self,
        target: atlas_index::ValidationTarget,
    ) -> atlas_index::ArtifactValidationReport {
        match self.open_search_index() {
            Ok(index) => index.validate_target_report(target),
            Err(error) => match self.open_index() {
                Ok(index) => index.vector_extension_unavailable_report(target, error.to_string()),
                Err(base_error) => {
                    atlas_index::validation_report_for_error(&self.paths.index_path, base_error)
                }
            },
        }
    }

    pub(crate) fn open_search_index(
        &self,
    ) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexValidationError> {
        atlas_index::SqliteIndexReader::open_read_only_with_vectors(&self.paths.index_path)
    }
}
