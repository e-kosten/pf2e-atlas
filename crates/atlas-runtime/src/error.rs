use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
    message: String,
}

impl RuntimeError {
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
