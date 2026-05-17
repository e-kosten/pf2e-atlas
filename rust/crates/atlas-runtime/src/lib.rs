#![deny(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use atlas_embedding::DEFAULT_EMBEDDING_MODEL;
use atlas_search::{AtlasRetrievalService, SearchEmbeddingConfig, SearchError};

mod setup;
mod setup_model;

pub use setup_model::{
    RuntimeSetupOptions, RuntimeSetupReport, SetupAction, SetupActionKind, SetupActionStatus,
    SetupBuildReport, SetupEmbeddingReport, SetupExitClass, SetupPathsReport, SetupReadiness,
    SetupReadinessItem, SetupReadinessStatus, SetupTarget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasPathMode {
    Auto,
    Repo,
    User,
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
            path_mode: AtlasPathMode::Auto,
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

    pub fn open_index(&self) -> Result<atlas_index::AtlasIndex, atlas_index::IndexValidationError> {
        atlas_index::AtlasIndex::open_read_only(&self.paths.index_path)
    }

    pub fn ensure_setup(&self, options: RuntimeSetupOptions) -> RuntimeSetupReport {
        setup::ensure_setup(&self.paths, options)
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
    ) -> Result<atlas_index::AtlasIndex, atlas_index::IndexValidationError> {
        atlas_index::AtlasIndex::open_read_only_with_vectors(&self.paths.index_path)
    }

    pub fn open_retrieval_service(&self) -> Result<AtlasRetrievalService, SearchError> {
        self.open_retrieval_service_with_model(DEFAULT_EMBEDDING_MODEL.to_string())
    }

    pub fn open_record_retrieval_service(&self) -> Result<AtlasRetrievalService, SearchError> {
        Ok(AtlasRetrievalService::without_embeddings(
            self.open_index()?,
        ))
    }

    pub fn open_retrieval_service_with_model(
        &self,
        model_id: impl Into<String>,
    ) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_search_index()?;
        let config = SearchEmbeddingConfig {
            model_id: model_id.into(),
            cache_root: self.paths.embedding_cache_root.clone(),
        };
        AtlasRetrievalService::new(index, &config)
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
    User,
}

impl ResolvedPathMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::User => "user",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Repo => "repo checkout",
            Self::User => "user install",
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
    let current_dir = std::env::current_dir().map_err(|error| error.to_string())?;
    let repo_root = find_git_repo_root(&current_dir);
    let resolved_mode = match path_mode {
        AtlasPathMode::Auto => {
            if repo_root.is_some() {
                ResolvedPathMode::Repo
            } else {
                ResolvedPathMode::User
            }
        }
        AtlasPathMode::Repo => {
            if repo_root.is_none() {
                return Err(
                    "--path-mode repo requires running inside a git checkout with rust/Cargo.toml"
                        .to_string(),
                );
            }
            ResolvedPathMode::Repo
        }
        AtlasPathMode::User => ResolvedPathMode::User,
    };

    let defaults = match resolved_mode {
        ResolvedPathMode::Repo => {
            let repo_root = repo_root
                .clone()
                .expect("repo path mode only selected when repo root exists");
            AtlasPathOverrides {
                source_root: Some(repo_root.join("vendor").join("pf2e")),
                embedding_cache_root: Some(repo_root.join(".cache").join("hf-models")),
                index_path: Some(repo_root.join(".cache").join("pf2e-rust-index.sqlite")),
            }
        }
        ResolvedPathMode::User => {
            let cache_root = platform_cache_root()?.join("pf2e-atlas");
            AtlasPathOverrides {
                source_root: Some(cache_root.join("vendor").join("pf2e")),
                embedding_cache_root: Some(cache_root.join("hf-models")),
                index_path: Some(cache_root.join("pf2e-rust-index.sqlite")),
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
            .expect("source default is always resolved"),
        embedding_cache_root: overrides
            .embedding_cache_root
            .or(defaults.embedding_cache_root)
            .expect("embedding cache default is always resolved"),
        index_path: overrides
            .index_path
            .or(defaults.index_path)
            .expect("index default is always resolved"),
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
    if root.join("rust").join("Cargo.toml").is_file()
        && root
            .join("rust")
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
