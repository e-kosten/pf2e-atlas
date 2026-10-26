//! Transport compatibility is independent of reference/workspace readiness.
use axum::{
    Json,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub(crate) const HEADER: &str = "x-atlas-api-contract";
pub(crate) const CONTRACT: &str = env!("ATLAS_API_CONTRACT");

pub(crate) async fn handshake() -> impl IntoResponse {
    Json(serde_json::json!({ "api_contract": CONTRACT }))
}

pub(crate) async fn check(request: Request, next: Next) -> Response {
    if !request.uri().path().starts_with("/api/") {
        return next.run(request).await;
    }
    let mismatch = request
        .headers()
        .get(HEADER)
        .is_some_and(|expected| expected.as_bytes() != CONTRACT.as_bytes());
    let mut response = if mismatch {
        (StatusCode::CONFLICT, Json(serde_json::json!({
            "message": "Atlas frontend and backend versions differ. Reload the page or restart a matching backend."
        }))).into_response()
    } else {
        next.run(request).await
    };
    response
        .headers_mut()
        .insert(HEADER, HeaderValue::from_static(CONTRACT));
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
