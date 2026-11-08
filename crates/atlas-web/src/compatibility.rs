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

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompatibilityConflict {
    message: &'static str,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub(crate) struct CompatibilityHandshake {
    api_contract: &'static str,
}

#[utoipa::path(get, path = "/api/compatibility", operation_id = "handshake", responses((status = 200, description = "Successful response", body = CompatibilityHandshake), (status = 409, description = "Application or browser contract conflict", body = crate::error::ConflictResponse), (status = "default", description = "Atlas error", body = atlas_app_model::AppError)))]
pub(crate) async fn handshake() -> impl IntoResponse {
    Json(CompatibilityHandshake {
        api_contract: CONTRACT,
    })
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
        (StatusCode::CONFLICT, Json(CompatibilityConflict {
            message: "Atlas frontend and backend versions differ. Reload the page or restart a matching backend.",
        })).into_response()
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
