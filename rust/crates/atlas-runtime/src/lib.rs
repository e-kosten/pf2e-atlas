#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use atlas_embedding::{DEFAULT_EMBEDDING_MODEL, EmbeddingRuntimeConfig};
use atlas_search::{AtlasRetrievalService, SearchEmbeddingConfig, SearchError};

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

    pub fn setup_status(&self) -> SetupStatus {
        check_setup_status(&self.paths)
    }

    pub fn fetch_source(&self) -> Result<(), String> {
        fetch_pf2e_source(&self.paths.source_root)
    }

    pub fn open_index(&self) -> Result<atlas_index::AtlasIndex, atlas_index::IndexValidationError> {
        atlas_index::AtlasIndex::open_read_only(&self.paths.index_path)
    }

    pub fn validate_index_report(&self) -> atlas_index::ArtifactValidationReport {
        match self.open_index() {
            Ok(index) => index.validate_report(),
            Err(error) => atlas_index::validation_report_for_error(&self.paths.index_path, error),
        }
    }

    pub fn validate_vector_index_report(&self) -> atlas_index::ArtifactValidationReport {
        match self.open_search_index() {
            Ok(index) => index.vector_validation_report(),
            Err(error) => atlas_index::validation_report_for_error(&self.paths.index_path, error),
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

#[derive(Debug, Clone)]
pub struct SetupStatus {
    pub source_exists: bool,
    pub embedding_model: String,
    pub model_cache: EmbeddingModelCacheStatus,
    pub index_exists: bool,
}

impl SetupStatus {
    pub const fn ready(&self) -> bool {
        self.source_exists && self.model_cache.ready
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddingModelCacheStatus {
    pub model_dir: PathBuf,
    pub ready: bool,
    pub missing_files: Vec<PathBuf>,
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

fn check_setup_status(paths: &ResolvedAtlasPaths) -> SetupStatus {
    SetupStatus {
        source_exists: paths.source_root.is_dir(),
        embedding_model: DEFAULT_EMBEDDING_MODEL.to_string(),
        model_cache: embedding_model_cache_status(&paths.embedding_cache_root),
        index_exists: paths.index_path.is_file(),
    }
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

fn embedding_model_cache_status(cache_root: &Path) -> EmbeddingModelCacheStatus {
    let config = EmbeddingRuntimeConfig::new(DEFAULT_EMBEDDING_MODEL, cache_root);
    let model_dir = config.model_dir();
    let required_files = [
        model_dir.join("tokenizer.json"),
        model_dir.join("onnx").join("model.onnx"),
    ];
    let missing_files = required_files
        .into_iter()
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

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
