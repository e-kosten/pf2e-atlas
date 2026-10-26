use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AddSavedListItemRequest, AdvanceEncounterTurnRequest,
    AppError, AppErrorCode, AppReadinessStatus, AppReadinessView, ApplyEncounterHpActionRequest,
    CreateEncounterRequest, CreateSavedListRequest, DeleteEncounterView, DeleteSavedListView,
    DiscoverFilterEditorRequest, DiscoverFilterValuesRequest, EncounterConditionApplicabilityView,
    EncounterConditionAutomationLevelView, EncounterConditionCatalogView,
    EncounterConditionCategoryView, EncounterConditionDefinitionView, EncounterCreateView,
    EncounterDetailView, EncounterIndexView, EncounterMutationView,
    EncounterParticipantConditionView, EncounterParticipantKindView, EncounterParticipantSideView,
    EncounterParticipantStatusView, EncounterParticipantVariantView, EncounterParticipantView,
    EncounterStatusView, EncounterSummaryView, EncounterTurnReviewView, FilterEditorView,
    FilterSavedListRequest, FilterValueListView, RecordDetailView, RecordSummaryView,
    RemoveSavedListItemRequest, ReorderEncounterParticipantRequest, SavedListCreateView,
    SavedListDetailView, SavedListIndexView, SavedListItemMutationView, SavedListItemNoteView,
    SavedListItemSnapshotView, SavedListItemStatusView, SavedListItemView, SavedListSummaryView,
    SavedListUpdateView, SearchModeSummary, SearchPageView, SearchResultsPage,
    SearchResultsRequest, SetEncounterTurnRequest, UpdateEncounterParticipantConditionRequest,
    UpdateEncounterParticipantRequest, UpdateEncounterRequest, UpdateSavedListItemNoteRequest,
    UpdateSavedListRequest,
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

#[tokio::test]
async fn compatibility_handshake_is_available_without_service_capacity() {
    let app = router_with_state(AtlasWebState::from_service_with_blocking_capacity(
        MockService::default(),
        0,
    ));
    let response = app
        .oneshot(
            Request::get("/api/compatibility")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[super::compatibility::HEADER],
        super::compatibility::CONTRACT
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&body).unwrap()["api_contract"],
        super::compatibility::CONTRACT
    );
}

#[tokio::test]
async fn incompatible_requests_are_rejected_before_reads_or_mutations() {
    for (method, path) in [
        (Method::GET, "/api/readiness"),
        (Method::POST, "/api/lists"),
    ] {
        let app = router_with_state(AtlasWebState::from_service_with_blocking_capacity(
            MockService::default(),
            0,
        ));
        let response = app
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(super::compatibility::HEADER, "old-frontend")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(
            response.headers()[super::compatibility::HEADER],
            super::compatibility::CONTRACT
        );
    }
}

#[tokio::test]
async fn matching_contract_reaches_the_service() {
    let app = router_with_state(AtlasWebState::from_service_with_blocking_capacity(
        MockService::default(),
        0,
    ));
    let response = app
        .oneshot(
            Request::get("/api/readiness")
                .header(super::compatibility::HEADER, super::compatibility::CONTRACT)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        response.headers()[super::compatibility::HEADER],
        super::compatibility::CONTRACT
    );
}

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
    assert_eq!(body["groups"][0]["fields"][0]["definition"]["id"], "kind");

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
            "field_id":"source.pack","clause_id":null,"values":["Actions"]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["counts"]["value"]["options"][0]["distinct_roots"], 1);
    let (status, body) = route_json(
        Method::POST,
        "/api/filters/validate",
        Some(json!({
            "kind":"compare","field":"kind","op":"eq","value":"rule"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["predicate"]["field"], "kind");
    let (status, body) = route_json(Method::POST, "/api/records/detail", Some(json!({
        "record_key":"actions:testAction1","embedded_path":[],"fields":[],"passage":null,"source_fingerprint":null
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
            "filter":{"kind":"in","clause_id":"kind","field":"kind","values":["rule"]}
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
            "defeated": false, "hazard_disabled": false,
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
            "request_id": "apply-frightened",
            "expected_encounter_revision": 7,
            "name": "Frightened",
            "value": 1,
            "duration": {"amount":2,"unit":"rounds","timing":{"participant_key":"pc_1","event":"end"}}
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
            "request_id": "edit-frightened",
            "expected_encounter_revision":7,
            "name": "Frightened",
            "value": 2,
            "duration": {"amount":1,"unit":"rounds","timing":{"participant_key":"pc_1","event":"end"}}
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
            "request_id": "edit-frightened",
            "expected_encounter_revision":7,
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
        Some(json!({"request_id":"remove-frightened","expected_encounter_revision":7,"intent":"remove_effect"})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_participant_not_found");

    let (status, body) = route_json(
        Method::DELETE,
        "/api/encounters/ambush/participants/participant_a/conditions/7",
        Some(json!({"request_id":"remove-frightened","expected_encounter_revision":7,"intent":"remove_effect"})),
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
async fn saved_list_reorder_route_uses_path_identity_and_strict_placement() {
    let request = json!({
        "list_ref": "ignored-body-identity", "record_ref": "actions:testAction1",
        "target_record_ref": "actions:testAction2", "placement": "after"
    });
    let (status, body) = route_json(
        Method::POST,
        "/api/lists/list_research/items/reorder",
        Some(request.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list_key"], "list_research");
    assert_eq!(body["record_key"], "actions:testAction1");
    assert_eq!(body["outcome"], "reordered");
    for invalid in [
        {
            let mut value = request.clone();
            value["placement"] = json!("above");
            value
        },
        {
            let mut value = request.clone();
            value.as_object_mut().unwrap().remove("target_record_ref");
            value
        },
        {
            let mut value = request;
            value["unexpected"] = json!(true);
            value
        },
    ] {
        let (status, body) = route_json(
            Method::POST,
            "/api/lists/list_research/items/reorder",
            Some(invalid),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "invalid_request");
    }
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
        "participant_variant":"normal", "temporary_hp":0, "defeated":false, "hazard_disabled":false, "hidden":false,
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

#[tokio::test]
async fn turn_review_and_advance_use_explicit_revision_path_and_typed_failures() {
    let (status, body) = route_json(Method::GET, "/api/encounters/ambush/turn-review", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["expected_revision"], 7);
    assert_eq!(body["next_participant_key"], "pc_1");
    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/advance",
        Some(json!({"request_id":"next","expected_revision":7})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key":"ambush"}));
    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/advance",
        Some(json!({"request_id":"next-stale","expected_revision":6})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_turn_conflict");
    for body in [
        json!({}),
        json!({"expected_revision":null}),
        json!({"expected_revision":1.5}),
        json!({"expected_revision":7,"encounter_ref":"spoof"}),
    ] {
        let (status, body) =
            route_json(Method::POST, "/api/encounters/ambush/advance", Some(body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "invalid_request");
    }
    let (status, _) = route_json(
        Method::POST,
        "/api/encounters/ambush/turn",
        Some(json!({"encounter_ref":"ignored"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn turn_resource_route_preserves_intent_and_uses_participant_path_identity() {
    let path = "/api/encounters/ambush/participants/participant_a/turn-resources";
    let mut request = json!({"request_id":"manual-reaction","participant_key":"ignored","turn_key":"turn:1","expected_encounter_revision":7,"action":{"kind":"spend","resource":"reaction"}});
    let (status, body) = route_json(Method::POST, path, Some(request.clone())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key":"ambush"}));
    request["expected_encounter_revision"] = json!(6);
    let (status, body) = route_json(Method::POST, path, Some(request.clone())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_turn_conflict");
    request["action"] = json!({"kind":"spend","resource":"anything"});
    let (status, _) = route_json(Method::POST, path, Some(request)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn hp_actions_require_explicit_contract_and_use_path_identity_and_receipt() {
    let path = "/api/encounters/ambush/participants/participant_a/hp";
    let mut request = json!({"request_id":"hp-action","participant_key":"ignored_body_identity","action":{"kind":"damage","amount":"10+2"}});
    let (status, body) = route_json(Method::POST, path, Some(request.clone())).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");
    request["expected_revision"] = json!(6);
    let (status, body) = route_json(Method::POST, path, Some(request.clone())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_participant_conflict");
    request["expected_revision"] = json!(7);
    let (status, body) = route_json(Method::POST, path, Some(request.clone())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"encounter_key":"ambush"}));
    request["action"] = json!({"kind":"damage","amount":12});
    let (status, body) = route_json(Method::POST, path, Some(request)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");
}

#[tokio::test]
async fn dice_routes_forward_typed_requests_and_conflicts() {
    let (status, body) = route_json(
        Method::POST,
        "/api/rolls/flat-check",
        Some(json!({"dc":11})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["dc"], 11);
    assert_eq!(body["outcome"]["formula"], "1d20");
    assert_eq!(
        body["success"].as_bool().unwrap(),
        body["outcome"]["total"].as_i64().unwrap() >= 11
    );
    let (status, _) = route_json(
        Method::POST,
        "/api/rolls/flat-check",
        Some(json!({"dc":"bad"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let selection = json!({"record_key":"tables:example","embedded_path":[],"fields":[],"passage":null,"source_fingerprint":null});
    let (status, body) = route_json(Method::GET, "/api/encounters/ambush/initiative", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter_key"], "ambush");
    let (status,body)=route_json(Method::POST,"/api/encounters/ambush/initiative",Some(json!({"request_id":"initiative-id","artifact_generation":null,"expected_revision":7,"participants":[]}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["request_id"], "initiative-id");
    let (status, body) = route_json(
        Method::POST,
        "/api/rolls/table",
        Some(json!({"selection":selection,"artifact_generation":"current"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["outcome"]["expression"]["dice"][0]["value"], 4);
    let (status, body) = route_json(
        Method::POST,
        "/api/rolls/table",
        Some(json!({"selection":selection,"artifact_generation":"old"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "artifact_generation_changed");
    let reference = json!({"selection":selection,"artifact_generation":"current"});
    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/strike-options",
        Some(json!({"participant_key":"goblin","reference":reference})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], "ambush:goblin");
    let (status,body)=route_json(Method::POST,"/api/encounters/ambush/strike-rolls",Some(json!({"participant_key":"goblin","reference":reference,"request_id":"strike-id","expected_revision":7,"expected_participant_revision":3,"attack_number":2,"mode":"fortune","modifier_override":null,"roll_kind":"attack"}))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_participant_conflict");
    assert_eq!(body["message"], "strike-id");
    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/strike-rolls",
        Some(json!({"request_id":"incomplete"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");
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
    fn relationship_occurrences(
        &self,
        _request: atlas_app_model::RelationshipOccurrencesRequest,
        _execution: &ReadExecutionContext,
    ) -> Result<atlas_app_model::RelationshipOccurrencesPage, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "No relationship evidence in this fixture",
        ))
    }

    fn trait_definition(
        &self,
        _: atlas_app_model::TraitDetailRequest,
        _: &ReadExecutionContext,
    ) -> Result<atlas_app_model::TraitDetailView, AppServiceError> {
        unreachable!("trait lookup not used in this fixture")
    }
    fn trait_records(
        &self,
        _: atlas_app_model::TraitRecordsRequest,
        _: &ReadExecutionContext,
    ) -> Result<atlas_app_model::TraitRecordsPage, AppServiceError> {
        unreachable!("trait records not used in this fixture")
    }

    fn parties(&self) -> Result<atlas_app_model::PartyIndexView, AppServiceError> {
        Ok(atlas_app_model::PartyIndexView { parties: vec![] })
    }
    fn create_party(
        &self,
        _request: atlas_app_model::CreatePartyRequest,
    ) -> Result<atlas_app_model::PartyView, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "unused mock party creation",
        ))
    }
    fn update_party(
        &self,
        _key: &str,
        _request: atlas_app_model::UpdatePartyRequest,
    ) -> Result<atlas_app_model::PartyView, AppServiceError> {
        Err(AppServiceError::invalid_request("unused mock party update"))
    }
    fn delete_party(
        &self,
        key: &str,
        _request: atlas_app_model::DeletePartyRequest,
    ) -> Result<atlas_app_model::PartyMutationView, AppServiceError> {
        Ok(atlas_app_model::PartyMutationView {
            party_key: key.into(),
        })
    }
    fn add_encounter_party(
        &self,
        encounter_ref: &str,
        _request: atlas_app_model::AddEncounterPartyRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.into(),
        })
    }

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
        Ok(serde_json::from_value(json!({"artifact_generation":"current-generation","relationship_clauses":[],"catalog_version":2,"common_field_ids":["kind"],"limits":{"source_bytes":16384,"nodes":256,"depth":32,"literal_list":128,"relationship_leaves":16},"groups":[{"id":"standard","label":"Standard","fields":[{"definition":{"id":"kind","path":"kind","scope":null,"field_type":"string","label":"Kind","family_types":[],"units":null,"basis":"authored","choices":["rule"],"operators":["eq","in","state"],"examples":[],"value_discovery":"closed_choices"},"control":"text","short_choices":true,"applicable":true,"unit_fields":[],"collection_control":null,"placement":"initially_visible","label":"Kind","help":null,"choices":[{"value":"rule","label":"Rule"}],"operators":[{"operator":"eq","label":"is","operand":"scalar"},{"operator":"in","label":"is any of","operand":"list"},{"operator":"state","label":"availability is","operand":"state"}]}]}]})).expect("typed catalog fixture"))
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
        Ok(serde_json::from_value(json!({"search_profile":"bge-base","artifact_generation":"fixture-generation","counts":{"status":"available","value":{"field":"source.pack","options":[{"value":"Actions","distinct_roots":1,"selected":false}],"total_values":1,"exhaustive":true,"count_basis":"distinct roots"}}})).expect("typed counts fixture"))
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
    fn embedded_reading(
        &self,
        request: atlas_app_model::EmbeddedReadingRequest,
    ) -> Result<atlas_app_model::EmbeddedReadingBatchView, AppServiceError> {
        Ok(atlas_app_model::EmbeddedReadingBatchView {
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
        Ok(serde_json::from_value(json!({"artifact_generation":"test-generation","record":record.clone(),"presentation":{"classification":record.kind_label,"identity":record,"content":[],"embedded":[],"body":{"kind":"content"}},"selected":{"record_key":record_key,"embedded_path":[],"field":null,"passage":null,"source_fingerprint":null},"relationships":[],"relationships_truncated":false})).expect("typed record fixture"))
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
    fn preview_encounter_condition_add(
        &self,
        encounter_ref: &str,
        _request: atlas_app_model::AddEncounterParticipantConditionRequest,
    ) -> Result<atlas_app_model::ConditionEventPreviewView, AppServiceError> {
        Ok(atlas_app_model::ConditionEventPreviewView {
            persistent_precedence: None,
            hp_change: None,
            encounter_key: encounter_ref.into(),
            encounter_revision: 0,
            consequences: vec![],
        })
    }
    fn preview_encounter_condition_update(
        &self,
        encounter_ref: &str,
        _participant_key: &str,
        _request: atlas_app_model::UpdateEncounterParticipantConditionRequest,
    ) -> Result<atlas_app_model::ConditionEventPreviewView, AppServiceError> {
        Ok(atlas_app_model::ConditionEventPreviewView {
            persistent_precedence: None,
            hp_change: None,
            encounter_key: encounter_ref.into(),
            encounter_revision: 0,
            consequences: vec![],
        })
    }
    fn pending_encounter_condition_consequences(
        &self,
        _encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingConditionConsequenceView>, AppServiceError> {
        Ok(vec![])
    }
    fn preview_encounter_drained_consequence(
        &self,
        _encounter_ref: &str,
        _request: atlas_app_model::ResolveDrainedConsequenceRequest,
    ) -> Result<atlas_app_model::ConditionHpChangeView, AppServiceError> {
        Ok(atlas_app_model::ConditionHpChangeView {
            current_before: Some(20),
            current_after: Some(15),
            maximum_before: Some(20),
            maximum_after: Some(15),
        })
    }
    fn resolve_encounter_drained_consequence(
        &self,
        encounter_ref: &str,
        _request: atlas_app_model::ResolveDrainedConsequenceRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.into(),
        })
    }
    fn set_encounter_life_settings(
        &self,
        encounter_ref: &str,
        _participant_key: &str,
        _request: atlas_app_model::SetEncounterLifeSettingsRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.into(),
        })
    }
    fn recover_persistent_damage(
        &self,
        _encounter_ref: &str,
        _request: atlas_app_model::RecoverPersistentDamageRequest,
    ) -> Result<atlas_app_model::PersistentRecoveryResultView, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "recovery fixture is not configured",
        ))
    }
    fn pending_encounter_hp_lifecycle(
        &self,
        _encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingHpLifecycleView>, AppServiceError> {
        Ok(vec![])
    }
    fn resolve_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
        _request: atlas_app_model::ResolveEncounterHpLifecycleRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.into(),
        })
    }
    fn prepare_persistent_damage(
        &self,
        _encounter_ref: &str,
        _request: atlas_app_model::PreparePersistentDamageRequest,
    ) -> Result<atlas_app_model::PreparedPersistentDamageView, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "persistent preparation fixture is not configured",
        ))
    }
    fn apply_persistent_damage(
        &self,
        _encounter_ref: &str,
        _request: atlas_app_model::ApplyPersistentDamageRequest,
    ) -> Result<atlas_app_model::PersistentDamageResultView, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "persistent damage fixture is not configured",
        ))
    }
    fn apply_condition_lifecycle(
        &self,
        _encounter_ref: &str,
        _request: atlas_app_model::ApplyConditionLifecycleRequest,
    ) -> Result<atlas_app_model::ConditionLifecycleResultView, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "lifecycle fixture is not configured",
        ))
    }
    fn condition_lifecycle_actions(
        &self,
        _encounter_ref: &str,
        _participant_key: &str,
        _boundary_key: Option<&str>,
    ) -> Result<atlas_app_model::ConditionLifecycleActionsView, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "lifecycle fixture is not configured",
        ))
    }
    fn pending_condition_recoveries(
        &self,
        _encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingConditionRecoveryView>, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "lifecycle fixture is not configured",
        ))
    }
    fn persistent_damage_review(
        &self,
        _encounter_ref: &str,
        _participant_key: &str,
        _boundary_key: Option<&str>,
    ) -> Result<atlas_app_model::PersistentReviewView, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "persistent review fixture is not configured",
        ))
    }
    fn preview_encounter_hp_lifecycle(
        &self,
        _encounter_ref: &str,
        _request: atlas_app_model::ResolveEncounterHpLifecycleRequest,
    ) -> Result<atlas_app_model::PendingHpLifecycleView, AppServiceError> {
        Err(AppServiceError::invalid_request(
            "HP lifecycle preview fixture is not configured",
        ))
    }
    fn encounters(&self) -> Result<EncounterIndexView, AppServiceError> {
        Ok(EncounterIndexView {
            encounters: vec![encounter_summary("ambush", "Ambush")],
        })
    }

    fn participant_encounter_condition_definitions(
        &self,
        _encounter_ref: &str,
        _participant_key: &str,
    ) -> Result<atlas_app_model::EncounterConditionCatalogView, AppServiceError> {
        self.encounter_condition_definitions()
    }
    fn encounter_condition_definitions(
        &self,
    ) -> Result<EncounterConditionCatalogView, AppServiceError> {
        Ok(EncounterConditionCatalogView {
            standard_persistent_recovery_dc: 15,
            damage_types: vec![],
            conditions: vec![EncounterConditionDefinitionView {
                editor_kind: atlas_app_model::ConditionEditorKindView::Ordinary,
                eligibility: atlas_app_model::EncounterConditionEligibilityView::Available,
                scope_label: None,
                supports_extra_action: false,
                supports_duration_instead_of_value: false,
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

    fn roll_flat_check(
        &self,
        request: atlas_app_model::RollFlatCheckRequest,
    ) -> Result<atlas_app_model::FlatCheckRollView, AppServiceError> {
        let outcome = serde_json::from_value(
            json!({"formula":"1d20","total":12,"expression":{"kind":"number","value":12}}),
        )
        .unwrap();
        Ok(atlas_app_model::FlatCheckRollView {
            dc: request.dc,
            success: 12 >= request.dc,
            outcome,
        })
    }
    fn roll_table(
        &self,
        request: atlas_app_model::RollTableRequest,
    ) -> Result<atlas_app_model::TableRollView, AppServiceError> {
        if request.artifact_generation != "current" {
            return Err(AppServiceError::new(
                AppErrorCode::ArtifactGenerationChanged,
                "reference changed",
            ));
        }
        assert_eq!(request.selection.record_key.to_string(), "tables:example");
        Ok(serde_json::from_value(json!({"outcome":{"formula":"1d6","total":4,"expression":{"kind":"dice","dice":[{"sides":6,"value":4,"kept":true}],"total":4}},"matches":[],"invalid_ranges":[]})).unwrap())
    }
    fn encounter_strike_options(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::EncounterStrikeOptionsRequest,
    ) -> Result<atlas_app_model::EncounterStrikeOptionsView, AppServiceError> {
        Ok(atlas_app_model::EncounterStrikeOptionsView {
            applied: vec![],
            conditional: vec![],
            expected_revision: 7,
            expected_participant_revision: 3,
            title: format!("{encounter_ref}:{}", request.participant_key),
            attacks: vec![],
            unavailable: None,
            damage_available: false,
            damage_unavailable: None,
        })
    }
    fn roll_encounter_strike(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::RollEncounterStrikeRequest,
    ) -> Result<atlas_app_model::EncounterStrikeRollView, AppServiceError> {
        assert_eq!(encounter_ref, "ambush");
        assert_eq!(request.attack_number, 2);
        assert_eq!(request.reference.artifact_generation, "current");
        Err(AppServiceError::new(
            AppErrorCode::EncounterParticipantConflict,
            request.request_id,
        ))
    }
    fn encounter_initiative_options(
        &self,
        encounter_ref: &str,
    ) -> Result<atlas_app_model::InitiativeOptionsView, AppServiceError> {
        Ok(atlas_app_model::InitiativeOptionsView {
            artifact_generation: None,
            encounter_key: encounter_ref.into(),
            expected_revision: 0,
            participants: vec![],
        })
    }
    fn roll_encounter_initiative(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::RollInitiativeRequest,
    ) -> Result<atlas_app_model::InitiativeRollView, AppServiceError> {
        Ok(atlas_app_model::InitiativeRollView {
            encounter_key: encounter_ref.into(),
            request_id: request.request_id,
            rolls: vec![],
        })
    }

    fn preview_encounter_strike_context(
        &self,
        _encounter_ref: &str,
        _request: atlas_app_model::EncounterStrikeContextRequest,
    ) -> Result<atlas_app_model::EncounterStrikeContextView, AppServiceError> {
        Err(AppServiceError::new(
            AppErrorCode::InvalidRequest,
            "casting mock not configured",
        ))
    }
    fn encounter_casting_reminders(
        &self,
        _encounter_ref: &str,
        _request: atlas_app_model::EncounterCastingRemindersRequest,
    ) -> Result<atlas_app_model::EncounterCastingRemindersView, AppServiceError> {
        Err(AppServiceError::new(
            AppErrorCode::InvalidRequest,
            "casting mock not configured",
        ))
    }
    fn apply_encounter_spell_action(
        &self,
        encounter_ref: &str,
        _request: atlas_app_model::ApplyEncounterSpellActionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.into(),
        })
    }
    fn apply_encounter_hp_action(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterHpActionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
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
        assert_eq!(
            request.action,
            atlas_app_model::EncounterHpAction::Damage {
                amount: "10+2".into()
            }
        );
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.into(),
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

    fn encounter_turn_review(
        &self,
        encounter_ref: &str,
    ) -> Result<EncounterTurnReviewView, AppServiceError> {
        Ok(EncounterTurnReviewView {
            next_turn_unavailable_reason: None,
            outgoing: None,
            encounter_key: encounter_ref.into(),
            expected_revision: 7,
            current_participant_key: None,
            next_participant_key: Some("pc_1".into()),
            reminders: vec![],
        })
    }
    fn update_turn_resources(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::UpdateTurnResourcesRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        assert_eq!(request.participant_key, "participant_a");
        assert_eq!(request.turn_key.as_deref(), Some("turn:1"));
        assert_eq!(request.request_id, "manual-reaction");
        assert_eq!(
            request.action,
            atlas_app_model::TurnResourceAction::Spend {
                resource: atlas_app_model::TurnResourceKind::Reaction
            }
        );
        if request.expected_encounter_revision != 7 {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterTurnConflict,
                "encounter changed",
            ));
        }
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.into(),
        })
    }
    fn advance_encounter_turn(
        &self,
        encounter_ref: &str,
        request: AdvanceEncounterTurnRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        if request.expected_revision != 7 {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterTurnConflict,
                "encounter changed",
            ));
        }
        Ok(EncounterMutationView {
            encounter_key: encounter_ref.into(),
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
        _request: atlas_app_model::RemoveEncounterParticipantConditionRequest,
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

    fn saved_list_item_note(
        &self,
        list_ref: &str,
        record_key: &str,
    ) -> Result<SavedListItemNoteView, AppServiceError> {
        Ok(SavedListItemNoteView {
            list_key: list_ref.into(),
            slug: "research".into(),
            record_key: record_key.into(),
            title: "Test Action 1".into(),
            note: Some("fixture note".into()),
        })
    }
    fn update_saved_list_item_note(
        &self,
        request: UpdateSavedListItemNoteRequest,
    ) -> Result<SavedListItemNoteView, AppServiceError> {
        Ok(SavedListItemNoteView {
            list_key: request.list_ref,
            slug: "research".into(),
            record_key: request.record_key,
            title: "Test Action 1".into(),
            note: request.note,
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
            sort: atlas_domain::ResultSort::SavedOrder {},
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

    fn reorder_saved_list_item(
        &self,
        request: atlas_app_model::ReorderSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError> {
        assert_eq!(request.target_record_ref, "actions:testAction2");
        assert_eq!(
            request.placement,
            atlas_app_model::ReorderSavedListItemPlacementView::After
        );
        Ok(SavedListItemMutationView {
            list_key: request.list_ref,
            slug: "research".into(),
            record_key: request.record_ref,
            record_name: None,
            outcome: atlas_app_model::SavedListItemMutationOutcomeView::Reordered,
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
        revision: 7,
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
        turn_resources: None,
        knockout_options: vec![],
        knockout_unavailable_reason: None,
        effective_max_hp: atlas_app_model::NumberFactView {
            state: atlas_domain::QueryFieldState::Value,
            value: Some(20.into()),
            adjustment: None,
        },
        level_override: None,
        base_death_threshold: 4,
        persistent_damage_groups: vec![],
        track_dying: false,
        mortality: atlas_app_model::MortalityStateView::Alive,
        cast_unavailable_reason: None,
        spellcasting: None,
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
        hazard_disabled: false,
        hidden: false,
        note: Some("wounded".to_string()),
        note_hint: Some("wounded".to_string()),
        conditions: if include_condition {
            vec![EncounterParticipantConditionView {
                scope: atlas_app_model::ConditionScopeView::Everyone,
                extra_action: atlas_app_model::ConditionExtraActionView::Unspecified,
                immunity_override: false,
                parameters: Default::default(),
                origin: None,
                condition_id: 7,
                condition_key: None,
                name: "Frightened".to_string(),
                value: Some(1),
                source_participant_key: None,
                duration: Some(atlas_app_model::ConditionDurationView {
                    amount: 2,
                    unit: atlas_app_model::ConditionDurationUnit::Rounds,
                    timing: None,
                }),
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
        unit_price_label: None,
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
        sort: atlas_domain::ResultSort::attribute(
            atlas_domain::SortField::Name,
            atlas_domain::SortDirection::Asc,
        ),
        relationship_clauses: vec![],
        ranking: Default::default(),
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
            relationship_evidence: vec![],
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
    let (status,body)=route_json(Method::POST,"/api/filters/counts",Some(json!({"context":{"kind":"filtered","filter":null,"text":null,"mode":"hybrid"},"field_id":"traits","clause_id":"negative-condition","values":["fire"]}))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["counts"]["status"], "unavailable");
    let (status,_)=route_json(Method::POST,"/api/filters/values",Some(json!({"kind":"vocabulary","scope":{"kind":"artifact"},"field_id":"traits","limit":20,"context":{"kind":"filtered","filter":null,"text":"dragon","mode":"hybrid"}}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn related_request_transports_complete_selection_and_generation() {
    let selection = json!({"record_key":"actors:test000000000000","embedded_path":[{"collection":"/items","identity":{"SnapshotLocal":{"index":0}}}],"field":"/system/description/value","passage":null,"source_fingerprint":"source-fingerprint"});
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

#[tokio::test]
async fn list_note_routes_take_identity_from_path_and_preserve_explicit_clear() {
    let (status, body) = route_json(
        Method::GET,
        "/api/lists/list_research/items/actions:testAction1/note",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["note"], "fixture note");
    let (status, body) = route_json(
        Method::PATCH,
        "/api/lists/list_research/items/actions:testAction1/note",
        Some(json!({"list_ref":"ignored", "record_key":"ignored", "note":"Purpose"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["list_key"], "list_research");
    assert_eq!(body["record_key"], "actions:testAction1");
    assert_eq!(body["note"], "Purpose");
    let (status, body) = route_json(
        Method::PATCH,
        "/api/lists/list_research/items/actions:testAction1/note",
        Some(json!({"list_ref":"ignored", "record_key":"ignored", "note":null})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["note"].is_null());
}
