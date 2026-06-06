#![deny(unsafe_code)]

use std::sync::Arc;

use atlas_app_model::{
    AppError, AppErrorCode, OpenResultWindowRequest, ReadResultWindowPageRequest,
};
use atlas_app_service::{AppServiceError, AtlasAppService};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

#[derive(Clone)]
struct AtlasWebState {
    service: Arc<AtlasAppService>,
}

impl AtlasWebState {
    fn new(service: AtlasAppService) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

pub fn router(service: AtlasAppService) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/api/readiness", get(readiness))
        .route("/api/result-windows", post(open_result_window))
        .route(
            "/api/result-windows/{window_id}/page",
            post(read_result_window_page),
        )
        .route("/api/records/{record_key}", get(record_detail))
        .with_state(AtlasWebState::new(service))
}

async fn root() -> &'static str {
    "Atlas web API is running."
}

async fn readiness(State(state): State<AtlasWebState>) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(call_service(move || Ok(service.readiness())).await?))
}

async fn open_result_window(
    State(state): State<AtlasWebState>,
    payload: Result<Json<OpenResultWindowRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(move || service.open_result_window(request)).await?,
    ))
}

async fn read_result_window_page(
    State(state): State<AtlasWebState>,
    Path(window_id): Path<String>,
    payload: Result<Json<ReadResultWindowPageRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let window_id = parse_window_id(&window_id)?;
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(move || service.read_result_window_page(window_id, request)).await?,
    ))
}

async fn record_detail(
    State(state): State<AtlasWebState>,
    Path(record_key): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(move || service.record_detail(&record_key)).await?,
    ))
}

#[derive(Debug)]
struct WebError(AppError);

impl From<AppServiceError> for WebError {
    fn from(error: AppServiceError) -> Self {
        Self(error.into_app_error())
    }
}

impl WebError {
    fn invalid_request(error: JsonRejection) -> Self {
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

fn status_for_error(code: AppErrorCode) -> StatusCode {
    match code {
        AppErrorCode::InvalidRequest
        | AppErrorCode::InvalidRecordKey
        | AppErrorCode::FilterInvalid
        | AppErrorCode::FilterFieldInvalid
        | AppErrorCode::FilterOptionInvalid => StatusCode::BAD_REQUEST,
        AppErrorCode::RecordNotFound | AppErrorCode::WindowNotFound => StatusCode::NOT_FOUND,
        AppErrorCode::WindowExpired => StatusCode::GONE,
        AppErrorCode::FilterEditorConflict | AppErrorCode::SetupInProgress => StatusCode::CONFLICT,
        AppErrorCode::ArtifactNotReady
        | AppErrorCode::ArtifactIncompatible
        | AppErrorCode::SetupRequired
        | AppErrorCode::FilterFieldNotApplicable
        | AppErrorCode::FilterMetricAmbiguous => StatusCode::UNPROCESSABLE_ENTITY,
        AppErrorCode::IndexUnavailable
        | AppErrorCode::VectorReadinessRequired
        | AppErrorCode::EmbeddingModelUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        AppErrorCode::OperationCancelled => StatusCode::REQUEST_TIMEOUT,
        AppErrorCode::OperationTimeout => StatusCode::REQUEST_TIMEOUT,
        AppErrorCode::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn parse_window_id(value: &str) -> Result<u64, WebError> {
    value
        .parse()
        .map_err(|error| WebError::invalid_path(format!("invalid result window id: {error}")))
}

async fn call_service<T: Send + 'static>(
    call: impl FnOnce() -> Result<T, AppServiceError> + Send + 'static,
) -> Result<T, WebError> {
    tokio::task::spawn_blocking(call)
        .await
        .map_err(|error| {
            WebError(AppError::new(
                AppErrorCode::InternalError,
                format!("app-service task failed: {error}"),
            ))
        })?
        .map_err(WebError::from)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::body::to_bytes;
    use axum::extract::Path;
    use axum::http::Request;
    use axum::response::IntoResponse;
    use axum::routing::post;
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn app_error_codes_map_to_expected_http_statuses() {
        assert_eq!(
            status_for_error(AppErrorCode::InvalidRequest),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_for_error(AppErrorCode::WindowNotFound),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_for_error(AppErrorCode::WindowExpired),
            StatusCode::GONE
        );
        assert_eq!(
            status_for_error(AppErrorCode::ArtifactNotReady),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            status_for_error(AppErrorCode::VectorReadinessRequired),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            status_for_error(AppErrorCode::InternalError),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn invalid_window_id_maps_to_app_error() {
        let error = parse_window_id("not-a-number")
            .expect_err("invalid path segment should map into app error")
            .0;

        assert_eq!(error.code, AppErrorCode::InvalidRequest);
        assert!(error.message.contains("invalid result window id"));
    }

    #[tokio::test]
    async fn invalid_window_id_route_returns_app_error_envelope() {
        async fn invalid_id_handler(Path(window_id): Path<String>) -> Result<Response, WebError> {
            let _ = parse_window_id(&window_id)?;
            Ok(StatusCode::NO_CONTENT.into_response())
        }

        let app = Router::new().route(
            "/api/result-windows/{window_id}/page",
            post(invalid_id_handler),
        );
        let response = app
            .oneshot(
                Request::post("/api/result-windows/not-a-number/page")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("error body should be readable");
        let app_error: AppError =
            serde_json::from_slice(&body).expect("error body should be AppError JSON");

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(app_error.code, AppErrorCode::InvalidRequest);
        assert!(app_error.message.contains("invalid result window id"));
    }

    #[tokio::test]
    async fn call_service_preserves_successful_result() {
        let value = call_service(|| Ok::<_, AppServiceError>("ready"))
            .await
            .expect("successful app-service call should pass through");

        assert_eq!(value, "ready");
    }

    #[tokio::test]
    async fn call_service_maps_service_error_to_http_envelope() {
        let error = call_service(|| {
            Err::<(), _>(AppServiceError::new(AppErrorCode::WindowExpired, "expired"))
        })
        .await
        .expect_err("service errors should map to web errors");

        let response = error.into_response();
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("error body should be readable");
        let app_error: AppError =
            serde_json::from_slice(&body).expect("error body should be AppError JSON");

        assert_eq!(parts.status, StatusCode::GONE);
        assert_eq!(app_error.code, AppErrorCode::WindowExpired);
        assert_eq!(app_error.message, "expired");
    }

    #[tokio::test]
    async fn call_service_maps_panic_to_internal_error() {
        let error = call_service(|| -> Result<(), AppServiceError> {
            panic!("simulated app-service panic");
        })
        .await
        .expect_err("join errors should map to web errors");

        let response = error.into_response();
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("error body should be readable");
        let app_error: AppError =
            serde_json::from_slice(&body).expect("error body should be AppError JSON");

        assert_eq!(parts.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(app_error.code, AppErrorCode::InternalError);
    }
}
