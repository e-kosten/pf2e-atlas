use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AddSavedListItemRequest, AppError, AppErrorCode,
    AppReadinessStatus, AppReadinessView, CreateEncounterRequest, CreateSavedListRequest,
    DeleteEncounterView, DeleteSavedListView, DiscoverFilterEditorRequest,
    DiscoverFilterValuesRequest, EncounterConditionApplicabilityView,
    EncounterConditionAutomationLevelView, EncounterConditionCatalogView,
    EncounterConditionCategoryView, EncounterConditionDefinitionView, EncounterCreateView,
    EncounterDetailView, EncounterIndexView, EncounterMutationView,
    EncounterParticipantConditionView, EncounterParticipantKindView, EncounterParticipantSideView,
    EncounterParticipantStatusView, EncounterParticipantVariantView, EncounterParticipantView,
    EncounterStatusView, EncounterSummaryView, FilterEditorView, FilterSavedListRequest,
    FilterValueListView, RecordDetailView, RecordSummaryView, RemoveSavedListItemRequest,
    ReorderEncounterParticipantRequest, SavedListCreateView, SavedListDetailView,
    SavedListIndexView, SavedListItemMutationView, SavedListItemSnapshotView,
    SavedListItemStatusView, SavedListItemView, SavedListSummaryView, SavedListUpdateView,
    SearchModeSummary, SearchPageView, SearchResultsPage, SearchResultsRequest,
    SetEncounterTurnRequest, UpdateEncounterParticipantConditionRequest,
    UpdateEncounterParticipantRequest, UpdateEncounterRequest, UpdateSavedListRequest,
};
use atlas_app_model::{
    DiscoverFilterCountsRequest, FilterCountsView, FilterValidationResult, RecordDetailRequest,
};
use atlas_app_service::{AppServiceError, ReadExecutionContext};
use atlas_domain::QueryPredicate;
use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::assets::WebAssets;
use super::error::status_for_error;
use super::router::router_with_state;
use super::service::{AtlasWebService, AtlasWebState, call_service};

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
        status_for_error(AppErrorCode::ArtifactGenerationChanged),
        StatusCode::CONFLICT
    );
    assert_eq!(
        status_for_error(AppErrorCode::SavedListNotFound),
        StatusCode::NOT_FOUND
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

#[tokio::test]
async fn call_service_preserves_successful_result() {
    let value = call_service(AtlasWebState::from_service(MockService::default()), || {
        Ok::<_, AppServiceError>("ready")
    })
    .await
    .expect("successful app-service call should pass through");

    assert_eq!(value, "ready");
}

#[tokio::test]
async fn call_service_maps_service_error_to_http_envelope() {
    let error = call_service(AtlasWebState::from_service(MockService::default()), || {
        Err::<(), _>(AppServiceError::new(
            AppErrorCode::ArtifactGenerationChanged,
            "changed",
        ))
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

    assert_eq!(parts.status, StatusCode::CONFLICT);
    assert_eq!(app_error.code, AppErrorCode::ArtifactGenerationChanged);
    assert_eq!(app_error.message, "changed");
}

#[tokio::test]
async fn call_service_maps_panic_to_internal_error() {
    let error = call_service(
        AtlasWebState::from_service(MockService::default()),
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
    for path in ["/api/records/actions:testAction1", "/api/readiness"] {
        let app = router_with_state(AtlasWebState::from_service_with_blocking_capacity(
            MockService::default(),
            0,
        ));
        let response = app
            .oneshot(
                Request::get(path)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");
        let (status, body) = response_json(response).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{path}");
        assert_eq!(body["code"], "service_busy", "{path}");
        assert_eq!(body["retryable"], true, "{path}");
        assert!(
            body["message"]
                .as_str()
                .expect("message should be string")
                .contains("blocking service call limit is full"),
            "{path}",
        );
    }
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
async fn stateless_search_route_carries_criteria_and_generation_and_rejects_stale_reads() {
    let first_body = json!({
        "criteria": {
            "kind": "list_records",
            "filter": null
        },
        "page": { "number": 1, "size": 25 }
    });
    let (status, body) = route_json(Method::POST, "/api/search/page", Some(first_body)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["artifact_generation"], "current-generation");
    assert_eq!(body["mode"]["kind"], "list_records");

    let mut page_body = json!({
        "criteria": {"kind":"text_search","query":"Record","filter":null,"mode":"hybrid"},
        "page": {"number":2,"size":25},
        "expected_artifact_generation":"current-generation"
    });
    let (status, body) =
        route_json(Method::POST, "/api/search/page", Some(page_body.clone())).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["page"]["number"], 2);
    assert_eq!(
        body["mode"],
        json!({"kind":"text_search","query":"Record","mode":"hybrid"})
    );

    page_body["expected_artifact_generation"] = json!("previous-generation");
    let (status, body) = route_json(Method::POST, "/api/search/page", Some(page_body)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "artifact_generation_changed");
    assert_eq!(
        body["details"],
        json!({"expected_artifact_generation":"previous-generation","actual_artifact_generation":"current-generation"})
    );
    assert_eq!(body["recoverable_action"], "refresh_results");
}

#[tokio::test]
async fn search_page_rejects_malformed_criteria_and_retired_window_routes() {
    let (status, body) = route_json(
        Method::POST,
        "/api/search/page",
        Some(json!({
            "criteria":{"kind":"text_search","query":"Record","mode":"unknown"},
            "page":{"number":1,"size":25}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");

    for path in ["/api/result-windows", "/api/result-windows/1/page"] {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}

#[tokio::test]
async fn record_and_filter_routes_use_real_router_wiring() {
    let (status, body) = route_json(Method::GET, "/api/records/actions:testAction1", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["record"]["record_key"], "actions:testAction1");
    assert_eq!(body["presentation"]["identity"]["title"], "Test Action 1");

    let editor_request = json!({
        "selected_field_ids": []
    });
    let (status, body) =
        route_json(Method::POST, "/api/filters/editor", Some(editor_request)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["groups"][0]["id"], "standard");
    assert_eq!(
        body["groups"][0]["fields"][0]["definition"]["id"],
        "record.kind"
    );

    let values_request = json!({
        "kind":"contextual",
        "context": { "kind": "filtered", "filter": null,"text":null,"mode":"lexical" },
        "field_id":"source.pack","clause_id":null,"text":null,"offset":0,"limit":100
    });
    let (status, body) =
        route_json(Method::POST, "/api/filters/values", Some(values_request)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["outcome"]["value"]["field"], "source.pack");
    assert_eq!(body["outcome"]["value"]["options"][0]["value"], "Actions");
    let (status, body) = route_json(
        Method::POST,
        "/api/filters/counts",
        Some(json!({
            "context":{"kind":"filtered","filter":null,"text":null,"mode":"lexical"},
            "field_id":"source.pack","clause_id":null
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["counts"]["value"]["counting_scope"], "roots");
    let (status, body) = route_json(
        Method::POST,
        "/api/filters/validate",
        Some(json!({
            "kind":"compare","field":"record.kind","op":"eq","value":"rule"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["predicate"]["field"], "record.kind");
    let (status, body) = route_json(Method::POST, "/api/records/detail", Some(json!({
        "record_key":"actions:testAction1","owners":[],"fields":[],"passage":null,"source_fingerprint":null
    }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["record"]["record_key"], "actions:testAction1");
}

#[tokio::test]
async fn saved_list_routes_use_real_router_wiring() {
    let (status, body) = route_json(Method::GET, "/api/lists", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["lists"][0]["list_key"], "list_research");
    assert_eq!(body["lists"][0]["slug"], "research");

    let (status, body) = route_json(
        Method::POST,
        "/api/lists",
        Some(json!({
            "slug": "encounters",
            "name": "Encounters",
            "description": "Session prep"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list"]["list_key"], "list_created");
    assert_eq!(body["list"]["slug"], "encounters");

    let (status, body) = route_json(Method::GET, "/api/lists/list_research", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list"]["slug"], "research");
    assert_eq!(body["items"][0]["record_key"], "actions:testAction1");
    assert_eq!(body["items"][0]["status"], "active");

    let (status, body) = route_json(
        Method::POST,
        "/api/lists/list_research/filter",
        Some(json!({
            "list_ref": "ignored",
            "filter":{"kind":"in","clause_id":"kind","field":"record.kind","values":["rule"]}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list"]["list_key"], "list_research");
    assert_eq!(body["items"][0]["record_key"], "actions:testAction1");

    let (status, body) = route_json(
        Method::PATCH,
        "/api/lists/list_research",
        Some(json!({
            "list_key": "ignored",
            "slug": "renamed-research",
            "name": "Renamed Research",
            "description": "Updated prep"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list"]["list_key"], "list_research");
    assert_eq!(body["list"]["slug"], "renamed-research");
    assert_eq!(body["list"]["name"], "Renamed Research");
    assert_eq!(body["list"]["description"], "Updated prep");

    let (status, body) = route_json(
        Method::POST,
        "/api/lists/list_research/items",
        Some(json!({
            "list_ref": "ignored",
            "record_ref": "actions:testAction2",
            "note": null
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list_key"], "list_research");
    assert_eq!(body["slug"], "research");
    assert_eq!(body["record_key"], "actions:testAction2");
    assert_eq!(body["outcome"], "added");

    let (status, body) = route_json(
        Method::DELETE,
        "/api/lists/list_research/items",
        Some(json!({
            "list_ref": "ignored",
            "record_ref": "actions:testAction1"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list_key"], "list_research");
    assert_eq!(body["slug"], "research");
    assert_eq!(body["record_key"], "actions:testAction1");
    assert_eq!(body["outcome"], "removed");

    let (status, body) = route_json(Method::DELETE, "/api/lists/list_research", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list_key"], "list_research");
    assert_eq!(body["slug"], "research");
    assert_eq!(body["deleted"], true);

    let (status, body) = route_json(Method::GET, "/api/lists/missing", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "saved_list_not_found");
}

#[tokio::test]
async fn encounter_routes_use_real_router_wiring() {
    let (status, body) = route_json(Method::GET, "/api/encounters", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounters"][0]["slug"], "ambush");

    let (status, body) =
        route_json(Method::GET, "/api/encounters/condition-definitions", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["conditions"][0]["condition_ref"],
        "conditionitems:TBSHQspnbcqxsmjL"
    );

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters",
        Some(json!({
            "name": "Boss Fight",
            "description": "Climax",
            "note": "Bring maps"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter"]["slug"], "boss-fight");

    let (status, body) = route_json(Method::GET, "/api/encounters/ambush", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter"]["encounter_key"], "ambush");
    assert_eq!(body["participants"][0]["participant_key"], "participant_a");

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush",
        Some(json!({
            "encounter_key": "ignored",
            "slug": "renamed-ambush",
            "name": "Renamed Ambush",
            "description": "Updated",
            "note": "Updated note",
            "status": "complete"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter_key"], "ambush");
    assert_eq!(body, json!({"encounter_key": "ambush"}));

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/record",
        Some(json!({
            "encounter_ref": "ignored",
            "record_ref": "actors:testCreature",
            "quantity": 2,
            "initiative": 18
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter_key"], "ambush");

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/manual",
        Some(json!({
            "encounter_ref": "ignored",
            "display_name": "Kyra",
            "max_hp": 24,
            "current_hp": 20,
            "initiative": 15
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter_key"], "ambush");

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/participant_a",
        Some(json!({
            "participant_key": "ignored",
            "expected_revision": 7,
            "display_name": "Renamed Goblin",
            "side": "enemy",
            "participant_variant": "normal",
            "initiative": 19,
            "max_hp": 12,
            "current_hp": 6,
            "temporary_hp": 1,
            "defeated": false,
            "hidden": false,
            "note": "wounded"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key": "ambush"}));

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/reorder",
        Some(json!({
            "participant_key": "participant_b",
            "target_participant_key": "participant_a",
            "placement": "before"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter_key"], "ambush");

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/turn",
        Some(json!({
            "encounter_ref": "ignored",
            "participant_key": "participant_a"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key": "ambush"}));

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/conditions",
        Some(json!({
            "participant_key": "ignored",
            "name": "Frightened",
            "value": 1,
            "duration_rounds": 2
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key": "ambush"}));

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/participant_a/conditions/7",
        Some(json!({
            "condition_id": 999,
            "name": "Frightened",
            "value": 2,
            "duration_rounds": 1
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key": "ambush"}));

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/wrong_participant/conditions/7",
        Some(json!({
            "condition_id": 999,
            "name": "Frightened",
            "value": 3
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_participant_not_found");

    let (status, body) = route_json(
        Method::DELETE,
        "/api/encounters/ambush/participants/wrong_participant/conditions/7",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_participant_not_found");

    let (status, body) = route_json(
        Method::DELETE,
        "/api/encounters/ambush/participants/participant_a/conditions/7",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key": "ambush"}));

    let (status, body) = route_json(
        Method::DELETE,
        "/api/encounters/ambush/participants/participant_a",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter_key"], "ambush");

    let (status, body) = route_json(Method::DELETE, "/api/encounters/ambush", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter_key"], "ambush");
    assert_eq!(body["deleted"], true);

    let (status, body) = route_json(Method::GET, "/api/encounters/missing", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_not_found");
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

#[tokio::test]
async fn participant_revision_is_required_and_stale_edits_return_409() {
    let (status, detail) = route_json(Method::GET, "/api/encounters/ambush", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["participants"][0]["revision"], json!(7));
    let mut request = json!({
        "participant_key":"participant_a", "display_name":"Edited", "side":"enemy",
        "participant_variant":"normal", "temporary_hp":0, "defeated":false, "hidden":false,
    });
    let path = "/api/encounters/ambush/participants/participant_a";
    let (status, body) = route_json(Method::PATCH, path, Some(request.clone())).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");
    request["expected_revision"] = json!(6);
    let (status, body) = route_json(Method::PATCH, path, Some(request.clone())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_participant_conflict");
    request["expected_revision"] = json!(7);
    let (status, body) = route_json(Method::PATCH, path, Some(request)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key":"ambush"}));
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
    router_with_state(AtlasWebState::from_service(MockService::default()))
}

#[derive(Default)]
struct MockService {
    read_probe: Option<std::sync::Arc<read_abandonment_tests::ReadProbe>>,
}

mod read_abandonment_tests;

impl AtlasWebService for MockService {
    fn readiness(&self) -> AppReadinessView {
        AppReadinessView {
            default_search_profile: atlas_domain::SearchProfileId::new("bge-base"),
            artifact_generation: None,
            search_profiles: vec![],
            status: AppReadinessStatus::Ready,
            message: "fixture ready".to_string(),
        }
    }

    fn discover_filter_editor(
        &self,
        _request: DiscoverFilterEditorRequest,
    ) -> Result<FilterEditorView, AppServiceError> {
        Ok(serde_json::from_value(json!({"catalog_version":2,"common_field_ids":["record.kind"],"limits":{"source_bytes":16384,"nodes":256,"depth":32,"literal_list":128},"groups":[{"id":"standard","label":"Standard","fields":[{"definition":{"id":"record.kind","path":"record.kind","scope":null,"field_type":"string","label":"Kind","family_types":[],"units":null,"basis":"authored","choices":["rule"],"operators":["eq","in","state"],"examples":[],"value_discovery":"closed_choices"},"control":"text","placement":"initially_visible","label":"Kind","help":null,"choices":[{"value":"rule","label":"Rule"}],"operators":[{"operator":"eq","label":"is","operand":"scalar"},{"operator":"in","label":"is any of","operand":"list"},{"operator":"state","label":"availability is","operand":"state"}]}]}]})).expect("typed catalog fixture"))
    }
    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
        _execution: &ReadExecutionContext,
    ) -> Result<FilterValueListView, AppServiceError> {
        if let DiscoverFilterValuesRequest::Vocabulary {
            scope,
            field_id,
            selected_values,
            ..
        } = request
        {
            return Ok(FilterValueListView::Vocabulary {
                scope,
                values: atlas_domain::QueryVocabulary {
                    field: field_id,
                    options: vec![],
                    selected_values,
                    total_values: 0,
                    basis:
                        atlas_domain::QueryVocabularyBasis::ProductKeyScopeBeforeEditionPreference,
                },
            });
        }
        Ok(serde_json::from_value(json!({"kind":"contextual","search_profile":"bge-base","artifact_generation":"fixture-generation","outcome":{"status":"available","value":{"field":"source.pack","options":[{"value":"Actions","distinct_roots":3,"selected":false}],"total_values":1,"exhaustive":true,"count_basis":"distinct roots"}}})).expect("typed values fixture"))
    }
    fn discover_filter_counts(
        &self,
        request: DiscoverFilterCountsRequest,
        _execution: &ReadExecutionContext,
    ) -> Result<FilterCountsView, AppServiceError> {
        if request.clause_id.is_some() {
            return Ok(FilterCountsView {
                search_profile: atlas_domain::SearchProfileId::new("bge-base"),
                artifact_generation: "fixture-generation".into(),
                counts: atlas_domain::QueryFacetOutcome::Unavailable {
                    reason: atlas_domain::FacetUnavailableReason::NonConjunctiveContext,
                },
            });
        }
        Ok(serde_json::from_value(json!({"search_profile":"bge-base","artifact_generation":"fixture-generation","counts":{"status":"available","value":{"field":"source.pack","counting_scope":"roots","states":[],"exhaustive":true,"count_basis":"distinct roots","minimum":null,"maximum":null}}})).expect("typed counts fixture"))
    }
    fn validate_filter(
        &self,
        predicate: QueryPredicate,
    ) -> Result<FilterValidationResult, AppServiceError> {
        Ok(FilterValidationResult {
            predicate: Some(predicate),
            errors: vec![],
        })
    }
    fn record_detail_at(
        &self,
        request: RecordDetailRequest,
    ) -> Result<RecordDetailView, AppServiceError> {
        self.record_detail(&request.record_key)
    }
    fn encounter_reference(
        &self,
        _encounter_ref: &str,
        _participant_key: &str,
        request: atlas_app_model::EncounterReferenceRequest,
    ) -> Result<RecordDetailView, AppServiceError> {
        self.record_detail_at(request.selection)
    }
    fn owned_reading(
        &self,
        request: atlas_app_model::OwnedReadingRequest,
    ) -> Result<atlas_app_model::OwnedReadingBatchView, AppServiceError> {
        Ok(atlas_app_model::OwnedReadingBatchView {
            artifact_generation: request.artifact_generation,
            parent: request.parent,
            availability: atlas_domain::QueryFieldState::Value,
            total: 0,
            start: 0,
            end: 0,
            previous: None,
            next: None,
            entries: vec![],
        })
    }

    fn search_page(
        &self,
        request: SearchResultsRequest,
        execution: &ReadExecutionContext,
    ) -> Result<SearchResultsPage, AppServiceError> {
        if let Some(probe) = &self.read_probe {
            probe.observe(execution)?;
        }
        if let Some(expected) = request.expected_artifact_generation
            && expected != "current-generation"
        {
            let mut error = AppError::new(AppErrorCode::ArtifactGenerationChanged, "reference data changed")
                .with_details(json!({"expected_artifact_generation":expected,"actual_artifact_generation":"current-generation"}));
            error.recoverable_action = Some(atlas_app_model::AppRecoverableAction::RefreshResults);
            return Err(error.into());
        }
        let mut page = search_results_page(request.page.number);
        page.mode = match request.criteria {
            atlas_app_model::SearchCriteria::SimilarRecords {
                seed_record_key, ..
            } => {
                let mut seed = record_summary();
                seed.record_key = seed_record_key;
                SearchModeSummary::SimilarRecords {
                    seed: Box::new(seed),
                }
            }
            atlas_app_model::SearchCriteria::ListRecords { .. } => SearchModeSummary::ListRecords,
            atlas_app_model::SearchCriteria::TextSearch { query, mode, .. } => {
                SearchModeSummary::TextSearch { query, mode }
            }
        };
        Ok(page)
    }

    fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError> {
        let mut record = record_summary();
        record.record_key = record_key.to_string();
        Ok(serde_json::from_value(json!({"artifact_generation":"test-generation","record":record.clone(),"presentation":{"classification":record.kind_label,"identity":record,"content":[],"owned":[],"body":{"kind":"content"}},"selected":{"record_key":record_key,"owners":[],"field":null,"passage":null,"source_fingerprint":null},"relationships":[],"relationships_truncated":false})).expect("typed record fixture"))
    }

    fn related_records(
        &self,
        request: atlas_app_model::RelatedRecordsRequest,
    ) -> Result<atlas_app_model::RelatedRecordsView, AppServiceError> {
        Ok(atlas_app_model::RelatedRecordsView {
            selection: request.selection,
            artifact_generation: request.artifact_generation,
            seed: record_summary(),
            references: atlas_app_model::RelatedCategoryView::Empty,
            counterparts: atlas_app_model::RelatedCategoryView::Empty,
            variants: atlas_app_model::RelatedCategoryView::Empty,
        })
    }
    fn encounters(&self) -> Result<EncounterIndexView, AppServiceError> {
        Ok(EncounterIndexView {
            encounters: vec![encounter_summary("ambush", "Ambush")],
        })
    }

    fn encounter_condition_definitions(
        &self,
    ) -> Result<EncounterConditionCatalogView, AppServiceError> {
        Ok(EncounterConditionCatalogView {
            conditions: vec![EncounterConditionDefinitionView {
                condition_ref: "conditionitems:TBSHQspnbcqxsmjL".to_string(),
                name: "Frightened".to_string(),
                automation_level: EncounterConditionAutomationLevelView::Automated,
                applies_to: vec![EncounterConditionApplicabilityView::Creature],
                categories: vec![EncounterConditionCategoryView::StatModifier],
                has_value: true,
                default_value: Some(1),
            }],
        })
    }

    fn encounter(&self, encounter_ref: &str) -> Result<EncounterDetailView, AppServiceError> {
        if encounter_ref == "missing" {
            return Err(encounter_not_found(encounter_ref));
        }
        Ok(encounter_detail(encounter_ref, None, false))
    }

    fn create_encounter(
        &self,
        request: CreateEncounterRequest,
    ) -> Result<EncounterCreateView, AppServiceError> {
        Ok(EncounterCreateView {
            encounter: encounter_summary("boss-fight", &request.name),
        })
    }

    fn delete_encounter(
        &self,
        encounter_ref: &str,
    ) -> Result<DeleteEncounterView, AppServiceError> {
        Ok(DeleteEncounterView {
            encounter_key: encounter_ref.to_string(),
            slug: encounter_ref.to_string(),
            deleted: true,
        })
    }

    fn update_encounter(
        &self,
        request: UpdateEncounterRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: request.encounter_key,
        })
    }

    fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: request.encounter_ref,
        })
    }

    fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: request.encounter_ref,
        })
    }

    fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        if encounter_ref == "missing" {
            return Err(encounter_not_found(encounter_ref));
        }
        if request.participant_key != "participant_a" {
            return Err(AppServiceError::invalid_request(
                "participant path override missing",
            ));
        }
        if request.expected_revision != 7 {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterParticipantConflict,
                "participant changed",
            ));
        }
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.to_string(),
        })
    }

    fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        _request: ReorderEncounterParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.to_string(),
        })
    }

    fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        _participant_key: &str,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.to_string(),
        })
    }

    fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: request.encounter_ref,
        })
    }

    fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        _request: AddEncounterParticipantConditionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.to_string(),
        })
    }

    fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        _request: UpdateEncounterParticipantConditionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        if participant_key != "participant_a" {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                "condition not found for participant",
            ));
        }
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.to_string(),
        })
    }

    fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        _condition_id: i64,
    ) -> Result<EncounterMutationView, AppServiceError> {
        if participant_key != "participant_a" {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                "condition not found for participant",
            ));
        }
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.to_string(),
        })
    }

    fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError> {
        Ok(SavedListIndexView {
            lists: vec![saved_list_summary()],
        })
    }

    fn saved_list(&self, list_ref: &str) -> Result<SavedListDetailView, AppServiceError> {
        if list_ref == "missing" {
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

    fn filter_saved_list(
        &self,
        request: FilterSavedListRequest,
    ) -> Result<SavedListDetailView, AppServiceError> {
        self.saved_list(&request.list_ref)
    }

    fn create_saved_list(
        &self,
        request: CreateSavedListRequest,
    ) -> Result<SavedListCreateView, AppServiceError> {
        Ok(SavedListCreateView {
            list: SavedListSummaryView {
                list_key: "list_created".to_string(),
                slug: request.slug,
                name: request.name,
                description: request.description,
                tags: request.tags,
                item_count: 0,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            },
        })
    }

    fn update_saved_list(
        &self,
        request: UpdateSavedListRequest,
    ) -> Result<SavedListUpdateView, AppServiceError> {
        Ok(SavedListUpdateView {
            list: SavedListSummaryView {
                list_key: request.list_key,
                slug: request.slug,
                name: request.name,
                description: request.description,
                tags: request.tags,
                item_count: 1,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-02T00:00:00Z".to_string(),
            },
        })
    }

    fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError> {
        Ok(SavedListItemMutationView {
            list_key: request.list_ref,
            slug: "research".to_string(),
            record_key: request.record_ref,
            record_name: Some("Test Action 1".to_string()),
            outcome: atlas_app_model::SavedListItemMutationOutcomeView::Added,
        })
    }

    fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError> {
        Ok(SavedListItemMutationView {
            list_key: request.list_ref,
            slug: "research".to_string(),
            record_key: request.record_ref,
            record_name: None,
            outcome: atlas_app_model::SavedListItemMutationOutcomeView::Removed,
        })
    }

    fn delete_saved_list(&self, list_ref: &str) -> Result<DeleteSavedListView, AppServiceError> {
        Ok(DeleteSavedListView {
            list_key: list_ref.to_string(),
            slug: "research".to_string(),
            deleted: true,
        })
    }
}

fn encounter_not_found(encounter_ref: &str) -> AppServiceError {
    AppServiceError::new(
        AppErrorCode::EncounterNotFound,
        format!("encounter `{encounter_ref}` not found"),
    )
}

fn encounter_summary(encounter_key: &str, name: &str) -> EncounterSummaryView {
    EncounterSummaryView {
        encounter_key: encounter_key.to_string(),
        slug: encounter_key.to_string(),
        name: name.to_string(),
        description: Some("Fixture encounter".to_string()),
        status: EncounterStatusView::Draft,
        round_number: 1,
        participant_count: 2,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

fn encounter_detail(
    encounter_ref: &str,
    current_turn_participant_key: Option<String>,
    include_condition: bool,
) -> EncounterDetailView {
    EncounterDetailView {
        artifact_generation: Some("test-generation".into()),
        encounter: encounter_summary(encounter_ref, "Ambush"),
        note: Some("Fixture notes".to_string()),
        current_turn_participant_key,
        participants: vec![
            encounter_participant("participant_a", "Goblin", Some(18), include_condition),
            encounter_participant("participant_b", "Kyra", Some(15), false),
        ],
    }
}

fn encounter_participant(
    participant_key: &str,
    display_name: &str,
    initiative: Option<i64>,
    include_condition: bool,
) -> EncounterParticipantView {
    EncounterParticipantView {
        revision: 7,
        hp_origin: "explicit".into(),
        variant_origin: "explicit".into(),
        participant_key: participant_key.to_string(),
        record_key: Some("actors:testCreature".to_string()),
        participant_kind: EncounterParticipantKindView::Creature,
        participant_variant: EncounterParticipantVariantView::Normal,
        status: EncounterParticipantStatusView::Active,
        position: 1,
        display_name: display_name.to_string(),
        side: EncounterParticipantSideView::Enemy,
        initiative,
        initiative_order: 1,
        max_hp: Some(12),
        current_hp: Some(6),
        temporary_hp: 0,
        defeated: false,
        hidden: false,
        note: Some("wounded".to_string()),
        note_hint: Some("wounded".to_string()),
        conditions: if include_condition {
            vec![EncounterParticipantConditionView {
                condition_id: 7,
                condition_key: None,
                name: "Frightened".to_string(),
                value: Some(1),
                source_participant_key: None,
                duration_rounds: Some(2),
                note: None,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            }]
        } else {
            vec![]
        },
        record: Some(record_summary()),
        presentation: None,
    }
}

fn saved_list_summary() -> SavedListSummaryView {
    SavedListSummaryView {
        list_key: "list_research".to_string(),
        slug: "research".to_string(),
        name: "Research".to_string(),
        description: Some("Campaign prep".to_string()),
        tags: vec!["story-beat".to_string()],
        item_count: 1,
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
        source_type: Some("action".into()),
        level_basis: None,
        level_label: None,
        rarity: None,
        traits: vec![],
        publication: None,
        pack: Some("Actions".to_string()),
    }
}

fn search_results_page(page_number: u32) -> SearchResultsPage {
    SearchResultsPage {
        search_profile: atlas_domain::SearchProfileId::new("bge-base"),
        artifact_generation: "current-generation".into(),
        coverage: None,
        mode: SearchModeSummary::ListRecords,
        page: SearchPageView {
            number: page_number,
            size: 25,
            count: 1,
            total: 3,
            has_more: page_number < 3,
            next_page: Some(page_number + 1),
        },
        rows: vec![atlas_app_model::SearchResultRow {
            record: record_summary(),
            matches: vec![],
        }],
    }
}

#[tokio::test]
async fn filter_discovery_transport_keeps_explicit_vocabulary_and_optional_unavailability() {
    let (status,body)=route_json(Method::POST,"/api/filters/values",Some(json!({"kind":"vocabulary","scope":{"kind":"saved_list","list_ref":"my-list"},"field_id":"traits","text":"fire","limit":20,"selected_values":["retained"]}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["kind"], "vocabulary");
    assert_eq!(body["scope"]["list_ref"], "my-list");
    assert_eq!(body["values"]["selected_values"][0], "retained");
    assert!(body["values"].get("count_basis").is_none());
    let (status,body)=route_json(Method::POST,"/api/filters/counts",Some(json!({"context":{"kind":"filtered","filter":null,"text":null,"mode":"hybrid"},"field_id":"traits","clause_id":"negative-condition"}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["counts"]["status"], "unavailable");
    let (status,_)=route_json(Method::POST,"/api/filters/values",Some(json!({"kind":"vocabulary","scope":{"kind":"artifact"},"field_id":"traits","limit":20,"context":{"kind":"filtered","filter":null,"text":"dragon","mode":"hybrid"}}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn related_request_transports_complete_selection_and_generation() {
    let selection = json!({"record_key":"actors:test000000000000","owners":[{"collection":"/items","identity":{"SnapshotLocal":{"index":0}}}],"field":"/system/description/value","passage":null,"source_fingerprint":"source-fingerprint"});
    let (status, body) = route_json(
        Method::POST,
        "/api/related",
        Some(json!({"selection":selection,"artifact_generation":"full-generation"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["selection"], selection);
    assert_eq!(body["artifact_generation"], "full-generation");
    assert_eq!(body["references"]["kind"], "empty");
    let (status, _) = route_json(
        Method::POST,
        "/api/related",
        Some(json!({"selection":selection})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn similar_page_transport_carries_exact_seed_and_body_free_summary() {
    let (status, body) = route_json(Method::POST, "/api/search/page", Some(json!({"criteria":{"kind":"similar_records","seed_record_key":"items:seed000000000000","filter":null},"page":{"number":2,"size":5},"expected_artifact_generation":"current-generation"}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mode"]["kind"], "similar_records");
    assert_eq!(body["mode"]["seed"]["record_key"], "items:seed000000000000");
    assert!(body["mode"]["seed"].get("presentation").is_none());
    assert_eq!(body["page"]["number"], 2);
}
