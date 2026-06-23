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

    fn invalid_path(message: impl Into<String>) -> Self {
        Self(AppError::new(AppErrorCode::InvalidRequest, message))
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
        AppErrorCode::RecordNotFound
        | AppErrorCode::SavedListNotFound
        | AppErrorCode::EncounterNotFound
        | AppErrorCode::EncounterParticipantNotFound
        | AppErrorCode::WindowNotFound => StatusCode::NOT_FOUND,
        AppErrorCode::WindowExpired => StatusCode::GONE,
        AppErrorCode::SavedListAlreadyExists
        | AppErrorCode::EncounterAlreadyExists
        | AppErrorCode::FilterEditorConflict
        | AppErrorCode::SetupInProgress => StatusCode::CONFLICT,
        AppErrorCode::ArtifactNotReady
        | AppErrorCode::ArtifactIncompatible
        | AppErrorCode::SetupRequired
        | AppErrorCode::FilterFieldNotApplicable
        | AppErrorCode::FilterMetricAmbiguous => StatusCode::UNPROCESSABLE_ENTITY,
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

pub(crate) fn parse_window_id(value: &str) -> Result<u64, WebError> {
    value
        .parse()
        .map_err(|error| WebError::invalid_path(format!("invalid result window id: {error}")))
}
