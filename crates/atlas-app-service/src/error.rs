use atlas_app_model::{AppError, AppErrorCode};
use atlas_search::{SearchError, SearchErrorKind};
use thiserror::Error;

pub type AppServiceResult<T> = Result<T, AppServiceError>;

#[derive(Debug, Error)]
#[error("{error:?}")]
pub struct AppServiceError {
    error: AppError,
}

impl AppServiceError {
    pub fn new(code: AppErrorCode, message: impl Into<String>) -> Self {
        Self {
            error: AppError::new(code, message),
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(AppErrorCode::InvalidRequest, message)
    }

    pub fn service_busy(message: impl Into<String>) -> Self {
        let mut error = AppError::new(AppErrorCode::ServiceBusy, message);
        error.retryable = Some(true);
        Self { error }
    }

    pub fn into_app_error(self) -> AppError {
        self.error
    }
}

impl From<AppError> for AppServiceError {
    fn from(error: AppError) -> Self {
        Self { error }
    }
}

impl From<atlas_runtime::RuntimeError> for AppServiceError {
    fn from(error: atlas_runtime::RuntimeError) -> Self {
        Self::new(AppErrorCode::SetupRequired, error.to_string())
    }
}

impl From<SearchError> for AppServiceError {
    fn from(error: SearchError) -> Self {
        let code = match error.kind() {
            SearchErrorKind::IndexUnavailable => AppErrorCode::IndexUnavailable,
            SearchErrorKind::ArtifactContractViolation => AppErrorCode::ArtifactIncompatible,
            SearchErrorKind::InvalidFilter => AppErrorCode::FilterInvalid,
            SearchErrorKind::InvalidOptions => AppErrorCode::InvalidRequest,
            SearchErrorKind::VectorReadinessRequired => AppErrorCode::VectorReadinessRequired,
            SearchErrorKind::EmbeddingUnavailable => AppErrorCode::EmbeddingModelUnavailable,
            SearchErrorKind::QueryFailed => AppErrorCode::InternalError,
        };
        Self::new(code, error.to_string())
    }
}

impl From<atlas_search::FilterDiscoveryError> for AppServiceError {
    fn from(error: atlas_search::FilterDiscoveryError) -> Self {
        let code = match error {
            atlas_search::FilterDiscoveryError::InvalidField(_) => AppErrorCode::FilterFieldInvalid,
            atlas_search::FilterDiscoveryError::InvalidOption(_) => {
                AppErrorCode::FilterOptionInvalid
            }
            atlas_search::FilterDiscoveryError::FieldNotApplicable(_) => {
                AppErrorCode::FilterFieldNotApplicable
            }
            atlas_search::FilterDiscoveryError::AmbiguousMetric(_) => {
                AppErrorCode::FilterMetricAmbiguous
            }
            atlas_search::FilterDiscoveryError::InvalidFilter(_) => AppErrorCode::FilterInvalid,
            atlas_search::FilterDiscoveryError::QueryFailed(_) => AppErrorCode::InternalError,
        };
        Self::new(code, error.to_string())
    }
}
