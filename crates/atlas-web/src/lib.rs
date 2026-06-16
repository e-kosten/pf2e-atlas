#![deny(unsafe_code)]

use std::sync::Arc;

use atlas_app_model::{
    AppError, AppErrorCode, AppReadinessView, DiscoverFilterEditorRequest,
    DiscoverFilterValuesRequest, FilterEditorView, FilterValueListView, OpenResultWindowRequest,
    ReadResultWindowPageRequest, RecordDetailView, ResultWindowPage, SavedListDetailView,
    SavedListIndexView,
};
use atlas_app_service::{AppServiceError, AtlasAppService};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rust_embed::Embed;
use tokio::sync::Semaphore;

const MAX_BLOCKING_SERVICE_CALLS: usize = 64;

#[derive(Embed)]
#[folder = "../../web/atlas-ui/dist"]
struct WebAssets;

#[derive(Clone)]
struct AtlasWebState {
    service: Arc<dyn AtlasWebService>,
    blocking_calls: Arc<Semaphore>,
    blocking_call_capacity: usize,
}

impl AtlasWebState {
    fn new(service: AtlasAppService) -> Self {
        Self::from_service(service)
    }

    fn from_service(service: impl AtlasWebService + 'static) -> Self {
        Self::from_service_with_blocking_capacity(service, MAX_BLOCKING_SERVICE_CALLS)
    }

    fn from_service_with_blocking_capacity(
        service: impl AtlasWebService + 'static,
        blocking_call_capacity: usize,
    ) -> Self {
        Self {
            service: Arc::new(service),
            blocking_calls: Arc::new(Semaphore::new(blocking_call_capacity)),
            blocking_call_capacity,
        }
    }
}

pub fn router(service: AtlasAppService) -> Router {
    router_with_state(AtlasWebState::new(service))
}

fn router_with_state(state: AtlasWebState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/api/readiness", get(readiness))
        .route("/api/lists", get(saved_lists))
        .route("/api/lists/{slug}", get(saved_list))
        .route("/api/filters/editor", post(discover_filter_editor))
        .route("/api/filters/values", post(discover_filter_values))
        .route("/api/result-windows", post(open_result_window))
        .route(
            "/api/result-windows/{window_id}/page",
            post(read_result_window_page),
        )
        .route("/api/records/{record_key}", get(record_detail))
        .fallback(get(static_asset))
        .with_state(state)
}

trait AtlasWebService: Send + Sync {
    fn readiness(&self) -> AppReadinessView;

    fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> Result<FilterEditorView, AppServiceError>;

    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
    ) -> Result<FilterValueListView, AppServiceError>;

    fn open_result_window(
        &self,
        request: OpenResultWindowRequest,
    ) -> Result<ResultWindowPage, AppServiceError>;

    fn read_result_window_page(
        &self,
        window_id: u64,
        request: ReadResultWindowPageRequest,
    ) -> Result<ResultWindowPage, AppServiceError>;

    fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError>;

    fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError>;

    fn saved_list(&self, slug: &str) -> Result<SavedListDetailView, AppServiceError>;
}

impl AtlasWebService for AtlasAppService {
    fn readiness(&self) -> AppReadinessView {
        self.readiness()
    }

    fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> Result<FilterEditorView, AppServiceError> {
        self.discover_filter_editor(request)
    }

    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
    ) -> Result<FilterValueListView, AppServiceError> {
        self.discover_filter_values(request)
    }

    fn open_result_window(
        &self,
        request: OpenResultWindowRequest,
    ) -> Result<ResultWindowPage, AppServiceError> {
        self.open_result_window(request)
    }

    fn read_result_window_page(
        &self,
        window_id: u64,
        request: ReadResultWindowPageRequest,
    ) -> Result<ResultWindowPage, AppServiceError> {
        self.read_result_window_page(window_id, request)
    }

    fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError> {
        self.record_detail(record_key)
    }

    fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError> {
        self.saved_lists()
    }

    fn saved_list(&self, slug: &str) -> Result<SavedListDetailView, AppServiceError> {
        self.saved_list(slug)
    }
}

async fn root() -> Response {
    asset_response("index.html")
}

async fn static_asset(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    if path.is_empty() {
        return asset_response("index.html");
    }
    if WebAssets::get(path).is_some() {
        return asset_response(path);
    }
    if !path.contains('.') {
        return asset_response("index.html");
    }
    StatusCode::NOT_FOUND.into_response()
}

async fn readiness(State(state): State<AtlasWebState>) -> Result<impl IntoResponse, WebError> {
    Ok(Json(state.service.readiness()))
}

async fn saved_lists(State(state): State<AtlasWebState>) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.saved_lists()).await?,
    ))
}

async fn saved_list(
    State(state): State<AtlasWebState>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.saved_list(&slug)).await?,
    ))
}

async fn discover_filter_editor(
    State(state): State<AtlasWebState>,
    payload: Result<Json<DiscoverFilterEditorRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.discover_filter_editor(request)).await?,
    ))
}

async fn discover_filter_values(
    State(state): State<AtlasWebState>,
    payload: Result<Json<DiscoverFilterValuesRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.discover_filter_values(request)).await?,
    ))
}

async fn open_result_window(
    State(state): State<AtlasWebState>,
    payload: Result<Json<OpenResultWindowRequest>, JsonRejection>,
) -> Result<impl IntoResponse, WebError> {
    let Json(request) = payload.map_err(WebError::invalid_request)?;
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.open_result_window(request)).await?,
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
        call_service(state, move || {
            service.read_result_window_page(window_id, request)
        })
        .await?,
    ))
}

async fn record_detail(
    State(state): State<AtlasWebState>,
    Path(record_key): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    let service = state.service.clone();
    Ok(Json(
        call_service(state, move || service.record_detail(&record_key)).await?,
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
        | AppErrorCode::RecordResolutionMiss
        | AppErrorCode::RecordResolutionAmbiguous
        | AppErrorCode::FilterInvalid
        | AppErrorCode::FilterFieldInvalid
        | AppErrorCode::FilterOptionInvalid => StatusCode::BAD_REQUEST,
        AppErrorCode::RecordNotFound
        | AppErrorCode::SavedListNotFound
        | AppErrorCode::WindowNotFound => StatusCode::NOT_FOUND,
        AppErrorCode::WindowExpired => StatusCode::GONE,
        AppErrorCode::SavedListAlreadyExists
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

fn parse_window_id(value: &str) -> Result<u64, WebError> {
    value
        .parse()
        .map_err(|error| WebError::invalid_path(format!("invalid result window id: {error}")))
}

fn asset_response(path: &str) -> Response {
    let Some(asset) = WebAssets::get(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type(path))
        .header(header::CACHE_CONTROL, cache_control(path))
        .body(axum::body::Body::from(asset.data.into_owned()))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("webmanifest") => "application/manifest+json; charset=utf-8",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn cache_control(path: &str) -> &'static str {
    if path == "index.html" {
        "no-cache"
    } else if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=300"
    }
}

async fn call_service<T: Send + 'static>(
    state: AtlasWebState,
    call: impl FnOnce() -> Result<T, AppServiceError> + Send + 'static,
) -> Result<T, WebError> {
    let blocking_call_capacity = state.blocking_call_capacity;
    let permit = state.blocking_calls.try_acquire_owned().map_err(|_| {
        WebError::from(AppServiceError::service_busy(format!(
            "atlas-web blocking service call limit is full; capacity is {blocking_call_capacity}",
        )))
    })?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        call()
    })
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
    use atlas_app_model::{
        AppReadinessStatus, FilterControlView, FilterEditorFieldView, FilterEditorGroupView,
        FilterFieldPlacement, FilterValueOption, RecordSummaryView, ResultWindowModeSummary,
        SavedListDetailView, SavedListIndexView, SavedListItemSnapshotView,
        SavedListItemStatusView, SavedListItemView, SavedListSummaryView, SearchPageView,
    };
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::RecordPresentationDocument;
    use axum::body::Body;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use axum::response::IntoResponse;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn app_error_codes_map_to_expected_http_statuses() {
        assert_eq!(
            status_for_error(AppErrorCode::InvalidRequest),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_for_error(AppErrorCode::RecordResolutionMiss),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_for_error(AppErrorCode::RecordResolutionAmbiguous),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_for_error(AppErrorCode::WindowNotFound),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_for_error(AppErrorCode::SavedListNotFound),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_for_error(AppErrorCode::WindowExpired),
            StatusCode::GONE
        );
        assert_eq!(
            status_for_error(AppErrorCode::SavedListAlreadyExists),
            StatusCode::CONFLICT
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
            status_for_error(AppErrorCode::ServiceBusy),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            status_for_error(AppErrorCode::InternalError),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            status_for_error(AppErrorCode::QueryFailed),
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
    async fn call_service_preserves_successful_result() {
        let value = call_service(AtlasWebState::from_service(MockService), || {
            Ok::<_, AppServiceError>("ready")
        })
        .await
        .expect("successful app-service call should pass through");

        assert_eq!(value, "ready");
    }

    #[tokio::test]
    async fn call_service_maps_service_error_to_http_envelope() {
        let error = call_service(AtlasWebState::from_service(MockService), || {
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
        let error = call_service(
            AtlasWebState::from_service(MockService),
            || -> Result<(), AppServiceError> {
                panic!("simulated app-service panic");
            },
        )
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

    #[tokio::test]
    async fn routes_return_service_busy_when_web_backpressure_is_full() {
        let app = router_with_state(AtlasWebState::from_service_with_blocking_capacity(
            MockService,
            0,
        ));
        let response = app
            .oneshot(
                Request::get("/api/records/actions:testAction1")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (status, body) = response_json(response).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["code"], "service_busy");
        assert_eq!(body["retryable"], true);
        assert!(
            body["message"]
                .as_str()
                .expect("message should be string")
                .contains("blocking service call limit is full")
        );
    }

    #[tokio::test]
    async fn readiness_route_returns_service_readiness() {
        let (status, body) = route_json(Method::GET, "/api/readiness", None).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ready");
        assert_eq!(body["message"], "fixture ready");
    }

    #[tokio::test]
    async fn root_serves_embedded_frontend_index() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::get("/")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("body should be readable");
        let body = String::from_utf8(body.to_vec()).expect("body should be UTF-8");

        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(
            parts.headers.get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            parts.headers.get(header::CACHE_CONTROL).unwrap(),
            "no-cache"
        );
        assert!(body.contains("<div id=\"root\"></div>"));
    }

    #[tokio::test]
    async fn frontend_routes_fall_back_to_index_but_api_routes_do_not() {
        let app = test_router();
        let response = app
            .clone()
            .oneshot(
                Request::get("/search")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("body should be readable");
        let body = String::from_utf8(body.to_vec()).expect("body should be UTF-8");

        assert_eq!(parts.status, StatusCode::OK);
        assert!(body.contains("<div id=\"root\"></div>"));

        let response = app
            .oneshot(
                Request::get("/api/no-such-route")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn embedded_frontend_assets_get_long_cache_headers() {
        let asset_path = WebAssets::iter()
            .find(|path| path.starts_with("assets/") && path.ends_with(".js"))
            .expect("built frontend should include a JavaScript asset");
        let app = test_router();
        let response = app
            .oneshot(
                Request::get(format!("/{asset_path}"))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/javascript; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn result_window_routes_return_success_and_service_errors() {
        let open_body = json!({
            "mode": {
                "kind": "list_records",
                "filter": { "clauses": [] },
                "sort": { "kind": "record_key" }
            },
            "page": { "number": 1, "size": 25 },
            "include_diagnostics": false
        });
        let (status, body) = route_json(Method::POST, "/api/result-windows", Some(open_body)).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["window_id"], 42);
        assert_eq!(body["mode"]["kind"], "list_records");

        let page_body = json!({ "page": { "number": 2, "size": 25 } });
        let (status, body) = route_json(
            Method::POST,
            "/api/result-windows/42/page",
            Some(page_body.clone()),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["page"]["number"], 2);

        let (status, body) = route_json(
            Method::POST,
            "/api/result-windows/not-a-number/page",
            Some(page_body.clone()),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "invalid_request");
        assert!(
            body["message"]
                .as_str()
                .expect("message should be string")
                .contains("invalid result window id")
        );

        let (status, body) = route_json(
            Method::POST,
            "/api/result-windows/999/page",
            Some(page_body),
        )
        .await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "window_not_found");
    }

    #[tokio::test]
    async fn record_and_filter_routes_use_real_router_wiring() {
        let (status, body) =
            route_json(Method::GET, "/api/records/actions:testAction1", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["record_key"], "actions:testAction1");
        assert_eq!(body["presentation"]["title"], "Test Action 1");

        let editor_request = json!({
            "context": { "kind": "filtered", "filter": { "clauses": [] } }
        });
        let (status, body) =
            route_json(Method::POST, "/api/filters/editor", Some(editor_request)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["groups"][0]["id"], "standard");
        assert_eq!(body["groups"][0]["fields"][0]["id"], "kind");

        let values_request = json!({
            "context": { "kind": "filtered", "filter": { "clauses": [] } },
            "field_id": "pack"
        });
        let (status, body) =
            route_json(Method::POST, "/api/filters/values", Some(values_request)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["field_id"], "pack");
        assert_eq!(body["options"][0]["label"], "Actions");
    }

    #[tokio::test]
    async fn saved_list_routes_use_real_router_wiring() {
        let (status, body) = route_json(Method::GET, "/api/lists", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["lists"][0]["slug"], "research");

        let (status, body) = route_json(Method::GET, "/api/lists/research", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["list"]["slug"], "research");
        assert_eq!(body["items"][0]["record_key"], "actions:testAction1");
        assert_eq!(body["items"][0]["status"], "active");

        let (status, body) = route_json(Method::GET, "/api/lists/missing", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "saved_list_not_found");
    }

    #[tokio::test]
    async fn malformed_json_route_body_returns_app_error_envelope() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::post("/api/filters/editor")
                    .header("content-type", "application/json")
                    .body(Body::from("{"))
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (status, body) = response_json(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "invalid_request");
        assert!(
            body["message"]
                .as_str()
                .expect("message should be string")
                .contains("invalid JSON request body")
        );
    }

    async fn route_json(method: Method, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let app = test_router();
        let mut builder = Request::builder().method(method).uri(path);
        let body = if let Some(body) = body {
            builder = builder.header("content-type", "application/json");
            Body::from(body.to_string())
        } else {
            Body::empty()
        };
        let response = app
            .oneshot(builder.body(body).expect("request should build"))
            .await
            .expect("route should respond");
        response_json(response).await
    }

    async fn response_json(response: Response) -> (StatusCode, Value) {
        let (parts, body) = response.into_parts();
        let body = to_bytes(body, usize::MAX)
            .await
            .expect("body should be readable");
        let json = serde_json::from_slice(&body).expect("body should be JSON");
        (parts.status, json)
    }

    fn test_router() -> Router {
        router_with_state(AtlasWebState::from_service(MockService))
    }

    struct MockService;

    impl AtlasWebService for MockService {
        fn readiness(&self) -> AppReadinessView {
            AppReadinessView {
                status: AppReadinessStatus::Ready,
                message: "fixture ready".to_string(),
            }
        }

        fn discover_filter_editor(
            &self,
            _request: DiscoverFilterEditorRequest,
        ) -> Result<FilterEditorView, AppServiceError> {
            Ok(FilterEditorView {
                matching_record_count: 3,
                groups: vec![FilterEditorGroupView {
                    id: "standard".to_string(),
                    label: "Standard".to_string(),
                    fields: vec![FilterEditorFieldView {
                        id: "kind".to_string(),
                        label: "Kinds".to_string(),
                        control: FilterControlView::MultiSelect,
                        placement: FilterFieldPlacement::AlwaysVisible,
                        applicability: atlas_app_model::FilterFieldApplicability::Applicable,
                        allowed_operators: vec![],
                        default_operator: atlas_app_model::FilterClauseOperator::IncludeAny,
                        supports_counts: true,
                    }],
                }],
            })
        }

        fn discover_filter_values(
            &self,
            _request: DiscoverFilterValuesRequest,
        ) -> Result<FilterValueListView, AppServiceError> {
            Ok(FilterValueListView {
                field_id: "pack".to_string(),
                matching_record_count: 3,
                options: vec![FilterValueOption {
                    value: "Actions".to_string(),
                    label: "Actions".to_string(),
                    count: Some(3),
                    selected: false,
                    disabled: false,
                    status: "available".to_string(),
                }],
            })
        }

        fn open_result_window(
            &self,
            _request: OpenResultWindowRequest,
        ) -> Result<ResultWindowPage, AppServiceError> {
            Ok(result_window_page(42, 1))
        }

        fn read_result_window_page(
            &self,
            window_id: u64,
            request: ReadResultWindowPageRequest,
        ) -> Result<ResultWindowPage, AppServiceError> {
            if window_id == 999 {
                return Err(AppServiceError::new(
                    AppErrorCode::WindowNotFound,
                    "window missing",
                ));
            }
            Ok(result_window_page(window_id, request.page.number))
        }

        fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError> {
            Ok(RecordDetailView {
                record_key: record_key.to_string(),
                title: "Test Action 1".to_string(),
                kind: "rule".to_string(),
                presentation: RecordPresentationDocument {
                    record_key: RecordKey::parse(record_key).expect("fixture key should parse"),
                    kind: RecordKind::Rule,
                    title: "Test Action 1".to_string(),
                    identity: vec![],
                    badges: vec![],
                    sections: vec![],
                },
            })
        }

        fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError> {
            Ok(SavedListIndexView {
                lists: vec![saved_list_summary()],
            })
        }

        fn saved_list(&self, slug: &str) -> Result<SavedListDetailView, AppServiceError> {
            if slug == "missing" {
                return Err(AppServiceError::new(
                    AppErrorCode::SavedListNotFound,
                    "saved list missing",
                ));
            }
            Ok(SavedListDetailView {
                list: saved_list_summary(),
                items: vec![SavedListItemView {
                    record_key: "actions:testAction1".to_string(),
                    position: 1,
                    note: Some("fixture note".to_string()),
                    status: SavedListItemStatusView::Active,
                    snapshot: SavedListItemSnapshotView {
                        title: "Test Action 1".to_string(),
                        kind: Some("rule".to_string()),
                    },
                    record: Some(record_summary()),
                }],
            })
        }
    }

    fn saved_list_summary() -> SavedListSummaryView {
        SavedListSummaryView {
            slug: "research".to_string(),
            name: "Research".to_string(),
            description: Some("Campaign prep".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn record_summary() -> RecordSummaryView {
        RecordSummaryView {
            record_key: "actions:testAction1".to_string(),
            title: "Test Action 1".to_string(),
            kind: "rule".to_string(),
            kind_label: "Rule".to_string(),
            level_label: None,
            rarity: None,
            traits: vec![],
            taxonomy: vec![],
            publication: None,
            pack: Some("Actions".to_string()),
            preview: None,
        }
    }

    fn result_window_page(window_id: u64, page_number: u32) -> ResultWindowPage {
        ResultWindowPage {
            window_id,
            mode: ResultWindowModeSummary::ListRecords,
            page: SearchPageView {
                number: page_number,
                size: 25,
                count: 1,
                total: 3,
                has_more: page_number < 3,
                next_page: Some(page_number + 1),
            },
            rows: vec![atlas_app_model::ResultWindowRow {
                record: record_summary(),
                match_summary: None,
            }],
        }
    }
}
