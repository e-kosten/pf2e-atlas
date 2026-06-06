use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub retryable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub recoverable_action: Option<AppRecoverableAction>,
}

impl AppError {
    pub fn new(code: AppErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            retryable: None,
            recoverable_action: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum AppErrorCode {
    InvalidRequest,
    InvalidRecordKey,
    RecordNotFound,
    WindowNotFound,
    WindowExpired,
    FilterInvalid,
    FilterFieldInvalid,
    FilterOptionInvalid,
    FilterFieldNotApplicable,
    FilterMetricAmbiguous,
    FilterEditorConflict,
    IndexUnavailable,
    ArtifactNotReady,
    ArtifactIncompatible,
    VectorReadinessRequired,
    EmbeddingModelUnavailable,
    SetupRequired,
    SetupInProgress,
    OperationCancelled,
    OperationTimeout,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum AppRecoverableAction {
    RunSetupCheck,
    RunSetup,
    ReopenResultWindow,
    RefreshReadiness,
    ReloadRecord,
    RebuildFilterEditor,
    Retry,
    None,
}
