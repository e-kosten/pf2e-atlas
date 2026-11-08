use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub retryable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum AppErrorCode {
    InvalidRequest,
    InvalidRecordKey,
    RecordNotFound,
    RecordResolutionMiss,
    RecordResolutionAmbiguous,
    SavedListNotFound,
    SavedListAlreadyExists,
    PartyNotFound,
    PartyConflict,
    EncounterNotFound,
    EncounterAlreadyExists,
    EncounterParticipantNotFound,
    EncounterParticipantConflict,
    EncounterTurnConflict,
    ArtifactGenerationChanged,
    FilterInvalid,
    FilterFieldInvalid,
    FilterOptionInvalid,
    FilterFieldNotApplicable,
    FilterEditorConflict,
    QueryFailed,
    IndexUnavailable,
    ArtifactNotReady,
    ArtifactIncompatible,
    VectorReadinessRequired,
    EmbeddingModelUnavailable,
    WorkspaceAccessFailed,
    WorkspaceMaintenanceRequired,
    SetupRequired,
    SetupInProgress,
    ServiceBusy,
    OperationCancelled,
    OperationTimeout,
    InternalError,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum AppRecoverableAction {
    RunSetupCheck,
    RunSetup,
    RefreshResults,
    RefreshReadiness,
    ReloadRecord,
    RebuildFilterEditor,
    Retry,
    None,
}
