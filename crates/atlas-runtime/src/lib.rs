#![deny(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use atlas_embedding::{EmbeddingModelId, embedding_model_for_model_id};
use atlas_search::{AtlasRetrievalService, SearchEmbeddingConfig, SearchError};

mod setup;
mod setup_clean;
mod setup_freshness;
mod setup_model;

pub use setup_model::{
    RuntimeSetupCleanOptions, RuntimeSetupCleanReport, RuntimeSetupOptions, RuntimeSetupReport,
    SetupAction, SetupActionKind, SetupActionStatus, SetupBuildReport, SetupCleanTarget,
    SetupCleanTargetKind, SetupCleanTargetStatus, SetupEmbeddingReport, SetupExitClass,
    SetupPathsReport, SetupReadiness, SetupReadinessItem, SetupReadinessStatus, SetupTarget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasPathMode {
    Repo,
    Global,
}

#[derive(Debug, Clone, Default)]
pub struct AtlasPathOverrides {
    pub source_root: Option<PathBuf>,
    pub embedding_cache_root: Option<PathBuf>,
    pub index_path: Option<PathBuf>,
}

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
    pub fn resolve(options: AtlasRuntimeOptions) -> Result<Self, String> {
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

    pub fn open_search_index(
        &self,
    ) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexValidationError> {
        atlas_index::SqliteIndexReader::open_read_only_with_vectors(&self.paths.index_path)
    }

    pub fn open_retrieval_service(&self) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_search_index().map_err(search_error_from_index)?;
        let report = index
            .validate_embedding_readiness()
            .map_err(search_error_from_index)?;
        if report.status != atlas_index::ValidationStatus::Ok {
            return Err(SearchError::vector_readiness_required(report.message));
        }
        let config = SearchEmbeddingConfig {
            model: embedding_model_from_artifact_report(&report)?,
            cache_root: self.paths.embedding_cache_root.clone(),
        };
        AtlasRetrievalService::new(index, &config)
    }

    pub fn open_retrieval_service_no_embeddings(
        &self,
    ) -> Result<AtlasRetrievalService, SearchError> {
        Ok(AtlasRetrievalService::without_embeddings(
            self.open_index().map_err(search_error_from_index)?,
        ))
    }

    pub fn open_vector_record_retrieval_service(
        &self,
    ) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_search_index().map_err(search_error_from_index)?;
        let report = index
            .validate_embedding_readiness()
            .map_err(search_error_from_index)?;
        if report.status != atlas_index::ValidationStatus::Ok {
            return Err(SearchError::artifact_contract_violation(
                atlas_index::IndexValidationError::InvalidArtifact(report.message).to_string(),
            ));
        }
        Ok(AtlasRetrievalService::without_embeddings(index))
    }
}

fn embedding_model_from_artifact_report(
    report: &atlas_index::ArtifactValidationReport,
) -> Result<EmbeddingModelId, SearchError> {
    let model_id = report.embedding_model_id.as_deref().ok_or_else(|| {
        SearchError::artifact_contract_violation(
            "artifact embedding metadata is missing `embedding_model_id`",
        )
    })?;
    embedding_model_for_model_id(model_id).ok_or_else(|| {
        SearchError::artifact_contract_violation(format!(
            "artifact embedding model `{model_id}` is not supported by this runtime"
        ))
    })
}

fn search_error_from_index(error: atlas_index::IndexValidationError) -> SearchError {
    match error {
        atlas_index::IndexValidationError::Unavailable(_) => {
            SearchError::index_unavailable(error.to_string())
        }
        atlas_index::IndexValidationError::InvalidArtifact(_) => {
            SearchError::artifact_contract_violation(error.to_string())
        }
        atlas_index::IndexValidationError::QueryFailed(_) => {
            SearchError::query_failed(error.to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedAtlasPaths {
    pub mode: ResolvedPathMode,
    pub repo_root: Option<PathBuf>,
    pub source_root: PathBuf,
    pub embedding_cache_root: PathBuf,
    pub index_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedPathMode {
    Repo,
    Global,
}

impl ResolvedPathMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::Global => "global",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Repo => "repo checkout",
            Self::Global => "global install",
        }
    }

    pub const fn suggested_path_mode(self) -> &'static str {
        self.as_str()
    }
}

fn resolve_atlas_paths(
    path_mode: AtlasPathMode,
    overrides: AtlasPathOverrides,
) -> Result<ResolvedAtlasPaths, String> {
    let (resolved_mode, repo_root) = match path_mode {
        AtlasPathMode::Repo => {
            let current_dir = std::env::current_dir().map_err(|error| error.to_string())?;
            let repo_root = find_git_repo_root(&current_dir);
            if repo_root.is_none() {
                return Err(
                    "--path-mode repo requires running inside a git checkout with Cargo.toml"
                        .to_string(),
                );
            }
            (ResolvedPathMode::Repo, repo_root)
        }
        AtlasPathMode::Global => (ResolvedPathMode::Global, None),
    };

    let defaults = match resolved_mode {
        ResolvedPathMode::Repo => {
            let Some(repo_root) = repo_root.clone() else {
                return Err(
                    "--path-mode repo requires running inside a git checkout with Cargo.toml"
                        .to_string(),
                );
            };
            AtlasPathOverrides {
                source_root: Some(repo_root.join("vendor").join("pf2e")),
                embedding_cache_root: Some(repo_root.join(".cache").join("hf-models")),
                index_path: Some(repo_root.join(".cache").join("pf2e-index.sqlite")),
            }
        }
        ResolvedPathMode::Global => {
            let cache_root = platform_cache_root()?.join("pf2e-atlas");
            AtlasPathOverrides {
                source_root: Some(cache_root.join("vendor").join("pf2e")),
                embedding_cache_root: Some(cache_root.join("hf-models")),
                index_path: Some(cache_root.join("pf2e-index.sqlite")),
            }
        }
    };

    Ok(ResolvedAtlasPaths {
        mode: resolved_mode,
        repo_root: if resolved_mode == ResolvedPathMode::Repo {
            repo_root
        } else {
            None
        },
        source_root: overrides
            .source_root
            .or(defaults.source_root)
            .ok_or_else(|| "source root default could not be resolved".to_string())?,
        embedding_cache_root: overrides
            .embedding_cache_root
            .or(defaults.embedding_cache_root)
            .ok_or_else(|| "embedding cache default could not be resolved".to_string())?,
        index_path: overrides
            .index_path
            .or(defaults.index_path)
            .ok_or_else(|| "index default could not be resolved".to_string())?,
    })
}

fn find_git_repo_root(current_dir: &Path) -> Option<PathBuf> {
    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(current_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let root = PathBuf::from(stdout.trim());
    if root.join("Cargo.toml").is_file()
        && root
            .join("crates")
            .join("atlas-cli")
            .join("Cargo.toml")
            .is_file()
    {
        Some(root)
    } else {
        None
    }
}

fn platform_cache_root() -> Result<PathBuf, String> {
    if cfg!(target_os = "macos") {
        return home_dir()
            .map(|home| home.join("Library").join("Caches"))
            .ok_or_else(|| "could not resolve HOME for user cache path".to_string());
    }
    if cfg!(target_os = "windows") {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data));
        }
        return home_dir()
            .map(|home| home.join("AppData").join("Local"))
            .ok_or_else(|| "could not resolve LOCALAPPDATA or USERPROFILE".to_string());
    }
    if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(cache_home));
    }
    home_dir()
        .map(|home| home.join(".cache"))
        .ok_or_else(|| "could not resolve HOME for user cache path".to_string())
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
