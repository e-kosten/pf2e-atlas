use atlas_embedding::{DEFAULT_EMBEDDING_MODEL, EmbeddingModelId};
use atlas_index::ValidationTarget;

#[derive(Debug, Clone)]
pub struct RuntimeSetupOptions {
    pub target: SetupTarget,
    pub check: bool,
    pub offline: bool,
    pub force_rebuild: bool,
    pub embedding_model_id: EmbeddingModelId,
    pub embedding_batch_size: usize,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeSetupCleanOptions {
    pub source: bool,
    pub embedding_cache: bool,
    pub artifact: bool,
    pub check: bool,
}

impl Default for RuntimeSetupOptions {
    fn default() -> Self {
        Self {
            target: SetupTarget::Full,
            check: false,
            offline: false,
            force_rebuild: false,
            embedding_model_id: DEFAULT_EMBEDDING_MODEL,
            embedding_batch_size: 32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupTarget {
    Full,
    Records,
}

impl SetupTarget {
    pub const fn requires_embeddings(self) -> bool {
        matches!(self, Self::Full)
    }

    pub const fn validation_target(self) -> ValidationTarget {
        match self {
            Self::Full => ValidationTarget::Full,
            Self::Records => ValidationTarget::BaseOnly,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Records => "records",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupActionKind {
    FetchSource,
    PrepareEmbeddingModel,
    AnalyzeSource,
    BuildIndex,
    ValidateIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupActionStatus {
    Planned,
    Done,
    Skipped,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupCleanTargetKind {
    Source,
    EmbeddingCache,
    Artifact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupCleanTargetStatus {
    Planned,
    Removed,
    Skipped,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SetupCleanTarget {
    pub kind: SetupCleanTargetKind,
    pub status: SetupCleanTargetStatus,
    pub path: String,
    pub reason: Option<String>,
}

impl SetupCleanTarget {
    pub(crate) fn new(
        kind: SetupCleanTargetKind,
        status: SetupCleanTargetStatus,
        path: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            status,
            path: path.into(),
            reason: None,
        }
    }

    pub(crate) fn with_reason(
        kind: SetupCleanTargetKind,
        status: SetupCleanTargetStatus,
        path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            status,
            path: path.into(),
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeSetupCleanReport {
    pub path_mode: &'static str,
    pub repo_root: Option<String>,
    pub check: bool,
    pub targets: Vec<SetupCleanTarget>,
}

impl RuntimeSetupCleanReport {
    pub fn exit_code_class(&self) -> SetupExitClass {
        if self
            .targets
            .iter()
            .any(|target| target.status == SetupCleanTargetStatus::Failed)
        {
            SetupExitClass::RuntimeFailure
        } else {
            SetupExitClass::Success
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupAction {
    pub kind: SetupActionKind,
    pub status: SetupActionStatus,
    pub reason: Option<String>,
}

impl SetupAction {
    pub(crate) fn new(kind: SetupActionKind, status: SetupActionStatus) -> Self {
        Self {
            kind,
            status,
            reason: None,
        }
    }

    pub(crate) fn with_reason(
        kind: SetupActionKind,
        status: SetupActionStatus,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            status,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupReadinessStatus {
    Ready,
    NotReady,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupReadinessItem {
    pub status: SetupReadinessStatus,
    pub required: bool,
    pub reason: Option<String>,
}

impl SetupReadinessItem {
    pub(crate) fn ready(required: bool) -> Self {
        Self {
            status: SetupReadinessStatus::Ready,
            required,
            reason: None,
        }
    }

    pub(crate) fn not_ready(required: bool, reason: impl Into<String>) -> Self {
        Self {
            status: SetupReadinessStatus::NotReady,
            required,
            reason: Some(reason.into()),
        }
    }

    pub(crate) fn skipped(reason: impl Into<String>) -> Self {
        Self {
            status: SetupReadinessStatus::Skipped,
            required: false,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetupReadiness {
    pub source: SetupReadinessItem,
    pub embedding_model: SetupReadinessItem,
    pub records: SetupReadinessItem,
    pub semantic_search: SetupReadinessItem,
}

#[derive(Debug, Clone)]
pub struct RuntimeSetupReport {
    pub target: SetupTarget,
    pub ready: bool,
    pub path_mode: &'static str,
    pub repo_root: Option<String>,
    pub offline: bool,
    pub check: bool,
    pub force_rebuild: bool,
    pub actions: Vec<SetupAction>,
    pub readiness: SetupReadiness,
    pub paths: SetupPathsReport,
    pub embedding: SetupEmbeddingReport,
    pub build: Option<SetupBuildReport>,
}

impl RuntimeSetupReport {
    pub fn exit_code_class(&self) -> SetupExitClass {
        if self
            .actions
            .iter()
            .any(|action| action.status == SetupActionStatus::Failed)
        {
            return SetupExitClass::RuntimeFailure;
        }
        if self.ready {
            return SetupExitClass::Success;
        }
        SetupExitClass::NotReady
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupExitClass {
    Success,
    NotReady,
    RuntimeFailure,
}

#[derive(Debug, Clone)]
pub struct SetupPathsReport {
    pub source: String,
    pub embedding_cache: String,
    pub index: String,
}

#[derive(Debug, Clone)]
pub struct SetupEmbeddingReport {
    pub model: String,
    pub model_path: String,
    pub cache_root: String,
    pub ready: bool,
    pub missing_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SetupBuildReport {
    pub source_signature: String,
    pub source_record_count: usize,
    pub artifact_record_count: usize,
    pub generated_record_count: usize,
    pub document_embedding_count: usize,
}
