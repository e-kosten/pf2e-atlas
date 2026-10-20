use atlas_app_model::{AppError, AppErrorCode};
use atlas_app_service::AppServiceError;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub(crate) struct WebError(pub(crate) AppError);

impl From<AppServiceError> for WebError {
    fn from(error: AppServiceError) -> Self {
        Self(error.into_app_error())
    }
}

impl WebError {
    pub(crate) fn invalid_request(error: JsonRejection) -> Self {
        Self(AppError::new(
            AppErrorCode::InvalidRequest,
            format!("invalid JSON request body: {error}"),
        ))
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let status = status_for_error(self.0.code);
        (status, Json(self.0)).into_response()
    }
}

pub(crate) fn status_for_error(code: AppErrorCode) -> StatusCode {
    match code {
        AppErrorCode::InvalidRequest
        | AppErrorCode::InvalidRecordKey
        | AppErrorCode::RecordResolutionMiss
        | AppErrorCode::RecordResolutionAmbiguous
        | AppErrorCode::FilterInvalid
        | AppErrorCode::FilterFieldInvalid
        | AppErrorCode::FilterOptionInvalid => StatusCode::BAD_REQUEST,
        AppErrorCode::PartyNotFound
        | AppErrorCode::RecordNotFound
        | AppErrorCode::SavedListNotFound
        | AppErrorCode::EncounterNotFound
        | AppErrorCode::EncounterParticipantNotFound => StatusCode::NOT_FOUND,
        AppErrorCode::PartyConflict
        | AppErrorCode::SavedListAlreadyExists
        | AppErrorCode::EncounterTurnConflict
        | AppErrorCode::EncounterParticipantConflict
        | AppErrorCode::EncounterAlreadyExists
        | AppErrorCode::FilterEditorConflict
        | AppErrorCode::SetupInProgress
        | AppErrorCode::ArtifactGenerationChanged => StatusCode::CONFLICT,
        AppErrorCode::ArtifactNotReady
        | AppErrorCode::ArtifactIncompatible
        | AppErrorCode::SetupRequired
        | AppErrorCode::FilterFieldNotApplicable => StatusCode::UNPROCESSABLE_ENTITY,
        AppErrorCode::IndexUnavailable
        | AppErrorCode::VectorReadinessRequired
        | AppErrorCode::EmbeddingModelUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        AppErrorCode::ServiceBusy => StatusCode::SERVICE_UNAVAILABLE,
        AppErrorCode::OperationCancelled => StatusCode::REQUEST_TIMEOUT,
        AppErrorCode::OperationTimeout => StatusCode::REQUEST_TIMEOUT,
        AppErrorCode::QueryFailed | AppErrorCode::InternalError => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
