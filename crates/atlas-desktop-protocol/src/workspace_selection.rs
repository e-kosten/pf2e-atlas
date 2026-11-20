//! Shared managed-workspace selection and pending desktop transition messages.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Original,
    Separate,
}

impl Role {
    pub fn other(self) -> Self {
        match self {
            Self::Original => Self::Separate,
            Self::Separate => Self::Original,
        }
    }

    pub fn as_arg(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Separate => "separate",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Waiting {
    Hold,
    Continue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReturnChoice {
    pub workspace: Role,
    pub updates: Waiting,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Build {
    pub version: String,
    pub tag: String,
    pub commit: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Admission {
    pub role: Role,
    pub workspace: PathBuf,
    pub key: Option<String>,
    pub schema: Option<u32>,
    pub file_id: Option<(u64, u64)>,
}

/// Failure facts from the private workspace admission helper, not display text.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionFailureKind {
    NewerSchema,
    Busy,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransitionFailure {
    pub kind: TransitionFailureKind,
    pub diagnostic: String,
}

impl From<String> for TransitionFailure {
    fn from(diagnostic: String) -> Self {
        Self {
            kind: TransitionFailureKind::Unknown,
            diagnostic,
        }
    }
}

impl From<&str> for TransitionFailure {
    fn from(diagnostic: &str) -> Self {
        diagnostic.to_owned().into()
    }
}

/// One compact line; a successful reservation stays alive until stdin closes.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TransitionResponse {
    Ok { data: Admission },
    Error { error: TransitionFailure },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Transition {
    pub before: Build,
    pub target: Build,
    pub admission: Admission,
    pub returning_to_stable: bool,
    pub initialization_key: Option<String>,
    #[serde(default)]
    pub previous_resumed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub format: u8,
    pub revision: u64,
    pub active: Role,
    pub returning: Option<ReturnChoice>,
    pub pending: Option<Transition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SelectionSnapshot {
    pub config: Config,
    pub workspaces: Vec<ManagedWorkspaceInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManagedWorkspaceInfo {
    pub role: Role,
    pub path: PathBuf,
    pub exists: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: 2,
            revision: 0,
            active: Role::Original,
            returning: None,
            pending: None,
        }
    }
}
