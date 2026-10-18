use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
    message: String,
}

impl RuntimeError {
    pub(crate) fn unsupported_search_profile(error: impl fmt::Display) -> Self {
        Self::new(
            RuntimeErrorKind::UnsupportedSearchProfile,
            error.to_string(),
        )
    }
    pub fn kind(&self) -> &RuntimeErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn current_directory_unavailable(error: impl fmt::Display) -> Self {
        Self::new(
            RuntimeErrorKind::CurrentDirectoryUnavailable,
            format!("failed to determine current directory: {error}"),
        )
    }

    pub(crate) fn repo_mode_outside_checkout() -> Self {
        Self::new(
            RuntimeErrorKind::RepoModeOutsideCheckout,
            "--path-mode repo requires running inside a git checkout with Cargo.toml",
        )
    }

    pub(crate) fn cache_root_unavailable(platform: RuntimePlatform) -> Self {
        let message = match platform {
            RuntimePlatform::Macos | RuntimePlatform::Unix => {
                "could not resolve HOME for user cache path"
            }
            RuntimePlatform::Windows => "could not resolve LOCALAPPDATA or USERPROFILE",
        };
        Self::new(RuntimeErrorKind::CacheRootUnavailable { platform }, message)
    }

    pub(crate) fn path_default_unavailable(target: RuntimePathTarget) -> Self {
        let message = match target {
            RuntimePathTarget::SourceRoot => "source root default could not be resolved",
            RuntimePathTarget::EmbeddingCacheRoot => {
                "embedding cache default could not be resolved"
            }
            RuntimePathTarget::IndexPath => "index default could not be resolved",
        };
        Self::new(RuntimeErrorKind::PathDefaultUnavailable { target }, message)
    }

    fn new(kind: RuntimeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    Workspace,
    WorkspaceBusy,
    WorkspaceMaintenance,
    WorkspaceIncompatible,
    ReferenceChanged { expected: u64, actual: u64 },
    Search(atlas_search::SearchErrorKind),
    UnsupportedSearchProfile,
    CurrentDirectoryUnavailable,
    RepoModeOutsideCheckout,
    CacheRootUnavailable { platform: RuntimePlatform },
    PathDefaultUnavailable { target: RuntimePathTarget },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePlatform {
    Macos,
    Windows,
    Unix,
}

impl RuntimePlatform {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePathTarget {
    SourceRoot,
    EmbeddingCacheRoot,
    IndexPath,
}

impl From<atlas_storage::StorageError> for RuntimeError {
    fn from(error: atlas_storage::StorageError) -> Self {
        let kind = match &error {
            atlas_storage::StorageError::Busy => RuntimeErrorKind::WorkspaceBusy,
            atlas_storage::StorageError::Maintenance(_) => RuntimeErrorKind::WorkspaceMaintenance,
            atlas_storage::StorageError::Schema { .. }
            | atlas_storage::StorageError::Unsupported(_) => {
                RuntimeErrorKind::WorkspaceIncompatible
            }
            _ => RuntimeErrorKind::Workspace,
        };
        Self::new(kind, error.to_string())
    }
}
impl From<atlas_search::SearchError> for RuntimeError {
    fn from(error: atlas_search::SearchError) -> Self {
        Self::new(RuntimeErrorKind::Search(error.kind()), error.to_string())
    }
}
impl From<atlas_index::IndexError> for RuntimeError {
    fn from(error: atlas_index::IndexError) -> Self {
        Self::from(atlas_search::SearchError::from(error))
    }
}
impl RuntimeError {
    pub(crate) fn stale_reference(expected: u64, actual: u64) -> Self {
        Self::new(
            RuntimeErrorKind::ReferenceChanged { expected, actual },
            format!("reference data changed ({expected} to {actual}); refresh before continuing"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_error_preserves_kind_and_message() {
        let error = RuntimeError::repo_mode_outside_checkout();

        assert_eq!(error.kind(), &RuntimeErrorKind::RepoModeOutsideCheckout);
        assert_eq!(
            error.message(),
            "--path-mode repo requires running inside a git checkout with Cargo.toml"
        );
        assert_eq!(error.to_string(), error.message());
    }

    #[test]
    fn path_default_errors_identify_target() {
        let error = RuntimeError::path_default_unavailable(RuntimePathTarget::EmbeddingCacheRoot);

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::PathDefaultUnavailable {
                target: RuntimePathTarget::EmbeddingCacheRoot
            }
        );
        assert_eq!(
            error.message(),
            "embedding cache default could not be resolved"
        );
    }
}
