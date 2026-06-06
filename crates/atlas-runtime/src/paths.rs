use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::error::{RuntimeError, RuntimePathTarget, RuntimePlatform};

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

pub(crate) fn resolve_atlas_paths(
    path_mode: AtlasPathMode,
    overrides: AtlasPathOverrides,
) -> Result<ResolvedAtlasPaths, RuntimeError> {
    let (resolved_mode, repo_root) = match path_mode {
        AtlasPathMode::Repo => {
            let current_dir =
                std::env::current_dir().map_err(RuntimeError::current_directory_unavailable)?;
            let repo_root = find_git_repo_root(&current_dir);
            if repo_root.is_none() {
                return Err(RuntimeError::repo_mode_outside_checkout());
            }
            (ResolvedPathMode::Repo, repo_root)
        }
        AtlasPathMode::Global => (ResolvedPathMode::Global, None),
    };

    let defaults = match resolved_mode {
        ResolvedPathMode::Repo => {
            let Some(repo_root) = repo_root.clone() else {
                return Err(RuntimeError::repo_mode_outside_checkout());
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
            .ok_or_else(|| RuntimeError::path_default_unavailable(RuntimePathTarget::SourceRoot))?,
        embedding_cache_root: overrides
            .embedding_cache_root
            .or(defaults.embedding_cache_root)
            .ok_or_else(|| {
                RuntimeError::path_default_unavailable(RuntimePathTarget::EmbeddingCacheRoot)
            })?,
        index_path: overrides
            .index_path
            .or(defaults.index_path)
            .ok_or_else(|| RuntimeError::path_default_unavailable(RuntimePathTarget::IndexPath))?,
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

fn platform_cache_root() -> Result<PathBuf, RuntimeError> {
    if cfg!(target_os = "macos") {
        return home_dir()
            .map(|home| home.join("Library").join("Caches"))
            .ok_or_else(|| RuntimeError::cache_root_unavailable(RuntimePlatform::Macos));
    }
    if cfg!(target_os = "windows") {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data));
        }
        return home_dir()
            .map(|home| home.join("AppData").join("Local"))
            .ok_or_else(|| RuntimeError::cache_root_unavailable(RuntimePlatform::Windows));
    }
    if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(cache_home));
    }
    home_dir()
        .map(|home| home.join(".cache"))
        .ok_or_else(|| RuntimeError::cache_root_unavailable(RuntimePlatform::Unix))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
