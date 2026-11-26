use atlas_app_model::{AppError, AppErrorCode};
use atlas_app_service::AppServiceError;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(utoipa::ToSchema)]
#[serde(untagged)]
#[allow(
    dead_code,
    reason = "schema for the two existing sources of an HTTP conflict"
)]
pub(crate) enum ConflictResponse {
    Application(AppError),
    Compatibility(crate::compatibility::CompatibilityConflict),
}

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
        | AppErrorCode::SimilaritySourceLimit
        | AppErrorCode::SimilarityCandidateLimit
        | AppErrorCode::SimilarityNoContent
        | AppErrorCode::SimilarityMethodUnavailable
        | AppErrorCode::FilterInvalid
        | AppErrorCode::FilterFieldInvalid
        | AppErrorCode::FilterOptionInvalid => StatusCode::BAD_REQUEST,
        AppErrorCode::CharacterNotFound
        | AppErrorCode::PartyNotFound
        | AppErrorCode::RecordNotFound
        | AppErrorCode::EquipmentFamilyNotFound
        | AppErrorCode::SavedListItemNotFound
        | AppErrorCode::SavedListNotFound
        | AppErrorCode::EncounterNotFound
        | AppErrorCode::EncounterParticipantNotFound => StatusCode::NOT_FOUND,
        AppErrorCode::CharacterConflict
        | AppErrorCode::PartyConflict
        | AppErrorCode::SavedEditConflict
        | AppErrorCode::SavedListAlreadyExists
        | AppErrorCode::SavedListNameConflict
        | AppErrorCode::EncounterAdditionConflict
        | AppErrorCode::EncounterTurnConflict
        | AppErrorCode::EncounterParticipantConflict
        | AppErrorCode::EncounterAlreadyExists
        | AppErrorCode::FilterEditorConflict
        | AppErrorCode::SetupInProgress
        | AppErrorCode::SimilaritySourceStale
        | AppErrorCode::SimilaritySectionsStale
        | AppErrorCode::ArtifactGenerationChanged => StatusCode::CONFLICT,
        AppErrorCode::ArtifactNotReady
        | AppErrorCode::ArtifactIncompatible
        | AppErrorCode::WorkspaceMaintenanceRequired
        | AppErrorCode::SetupRequired
        | AppErrorCode::FilterFieldNotApplicable => StatusCode::UNPROCESSABLE_ENTITY,
        AppErrorCode::IndexUnavailable
        | AppErrorCode::VectorReadinessRequired
        | AppErrorCode::EmbeddingModelUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        AppErrorCode::ServiceBusy => StatusCode::SERVICE_UNAVAILABLE,
        AppErrorCode::OperationCancelled => StatusCode::REQUEST_TIMEOUT,
        AppErrorCode::OperationTimeout => StatusCode::REQUEST_TIMEOUT,
        AppErrorCode::QueryFailed
        | AppErrorCode::WorkspaceAccessFailed
        | AppErrorCode::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// URL parameter rejections use the same JSON envelope as service errors.
pub(crate) struct Path<T>(pub T);

impl<T, S> axum::extract::FromRequestParts<S> for Path<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, WebError> {
        axum::extract::Path::<T>::from_request_parts(parts, state)
            .await
            .map(|path| Self(path.0))
            .map_err(|error| {
                WebError(AppError::new(
                    AppErrorCode::InvalidRequest,
                    error.body_text(),
                ))
            })
    }
}

impl From<axum::extract::rejection::QueryRejection> for WebError {
    fn from(error: axum::extract::rejection::QueryRejection) -> Self {
        Self(AppError::new(
            AppErrorCode::InvalidRequest,
            error.body_text(),
        ))
    }
}
