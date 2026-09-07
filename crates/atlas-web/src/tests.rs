use std::fs;
use std::path::PathBuf;

use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AddSavedListItemRequest, AppError, AppErrorCode,
    AppReadinessStatus, AppReadinessView, CreateEncounterRequest, CreateSavedListRequest,
    CreatureSurfaceActivityTypeView, CreatureSurfaceActivityView, CreatureSurfaceContentBlockView,
    CreatureSurfaceContentInlineView, CreatureSurfaceContentProvenanceView,
    CreatureSurfaceContentRoleView, CreatureSurfaceContentView, CreatureSurfaceDefensesView,
    CreatureSurfaceDomainUnavailableView, CreatureSurfaceFactOwnerView,
    CreatureSurfaceFactProvenanceView, CreatureSurfaceFrequencyView,
    CreatureSurfaceIntegerPresenceView, CreatureSurfaceOccurrenceIdentityStabilityView,
    CreatureSurfaceOccurrenceProvenanceView, CreatureSurfaceProvenanceView,
    CreatureSurfaceShieldView, CreatureSurfaceSkillSourceEntryView, CreatureSurfaceSourceFieldView,
    CreatureSurfaceSourceLocatorView, CreatureSurfaceSpellView, CreatureSurfaceSpellcastingView,
    CreatureSurfaceUnavailableCauseView, CreatureSurfaceUnavailableDomainsView,
    CreatureSurfaceUnavailableFieldView, CreatureSurfaceUnavailableStateView,
    CreatureSurfaceUnmodeledSkillReasonView, CreatureSurfaceUnmodeledSkillView,
    CreatureSurfaceView, DeleteEncounterView, DeleteSavedListView, DiscoverFilterEditorRequest,
    DiscoverFilterValuesRequest, EncounterConditionApplicabilityView,
    EncounterConditionAutomationLevelView, EncounterConditionCatalogView,
    EncounterConditionCategoryView, EncounterConditionDefinitionView, EncounterCreateView,
    EncounterDetailView, EncounterIndexView, EncounterParticipantKindView,
    EncounterParticipantPreservedDomainView, EncounterParticipantResetAvailabilityView,
    EncounterParticipantResetDomainView, EncounterParticipantResetResultView,
    EncounterParticipantSideView, EncounterParticipantStatusView, EncounterParticipantVariantView,
    EncounterParticipantView, EncounterRuntimeActionBudgetView,
    EncounterRuntimeAutomationLimitationCodeView, EncounterRuntimeAutomationLimitationTargetView,
    EncounterRuntimeAutomationLimitationView, EncounterRuntimeConditionView, EncounterRuntimeView,
    EncounterRuntimeVitalsView, EncounterSpellCastAvailabilityView,
    EncounterSpellCastOperationView, EncounterSpellCastRequest, EncounterSpellCastResultView,
    EncounterSpellCastStateView, EncounterSpellSpendTargetView, EncounterStatusView,
    EncounterSummaryView, EncounterUpdateView, FilterControlView, FilterEditorFieldView,
    FilterEditorGroupView, FilterEditorView, FilterFieldPlacement, FilterSavedListRequest,
    FilterValueListView, FilterValueOption, OpenResultWindowRequest, ReadResultWindowPageRequest,
    RecordDetailRequest, RecordDetailView, RecordSummaryView,
    RecordSurfaceEditionCounterpartRoleView, RecordSurfaceEditionCounterpartView,
    RecordSurfaceEditionStatusView, RecordSurfaceEditionView, RecordSurfaceMetadataView,
    RecordSurfacePresentationView, RecordSurfaceProfileView, RecordSurfaceSourceView,
    RecordSurfaceView, RemoveSavedListItemRequest, ReorderEncounterParticipantPlacementView,
    ReorderEncounterParticipantRequest, ResetEncounterParticipantRequest, ResultWindowModeSummary,
    ResultWindowPage, RuntimeCanonicalTargetView, RuntimeCapabilityView, RuntimeCountSegmentView,
    RuntimeCountView, RuntimeFactProvenanceView, RuntimeFactSourceView, RuntimeNumberView,
    RuntimeRuleView, SavedListCreateView, SavedListDetailView, SavedListIndexView,
    SavedListItemMutationView, SavedListItemSnapshotView, SavedListItemStatusView,
    SavedListItemView, SavedListSummaryView, SavedListUpdateView, SearchPageView,
    SetEncounterTurnRequest, SurfaceUnavailableReasonView, SurfaceUnavailableView,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest, UpdateSavedListRequest,
};
use atlas_app_service::AppServiceError;
use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::assets::WebAssets;
use super::error::{parse_window_id, status_for_error};
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
    let (status, body) = route_json(Method::GET, "/api/records/actions:testAction1", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["surface"]["metadata"]["record_key"],
        "actions:testAction1"
    );
    assert_eq!(body["surface"]["metadata"]["title"], "Test Action 1");
    assert_eq!(body["surface"]["profile"], "record_detail");
    assert_eq!(
        body["surface"]["presentation"]["presentation_type"],
        "unavailable"
    );
    assert!(body["surface"].get("sections").is_none());
    assert!(body["surface"]["metadata"].get("traits").is_none());
    assert_no_empty_containers(&body["surface"]);

    let (status, body) = route_json(Method::GET, "/api/records/hazards:testHazard", None).await;
    assert_eq!(status, StatusCode::OK);
    let surface = &body["surface"];
    assert_eq!(surface["metadata"]["record_key"], "hazards:testHazard");
    assert_eq!(surface["presentation"]["presentation_type"], "hazard");
    let hazard = &surface["presentation"]["body"];
    assert_eq!(hazard["complexity"], "complex");
    assert_eq!(hazard["detection"]["stealth_modifier"], 12);
    assert_eq!(hazard["detection"]["difficulty_class"], 22);
    assert_eq!(hazard["defenses"]["saves"]["fortitude"], 0);
    assert_eq!(
        hazard["activities"]
            .as_array()
            .expect("hazard activities")
            .iter()
            .map(|activity| activity["occurrence_id"].as_str().expect("occurrence id"))
            .collect::<Vec<_>>(),
        vec![
            "occurrence-action",
            "occurrence-strike",
            "occurrence-unsupported"
        ]
    );
    assert!(hazard["activities"][0].get("slug").is_none());
    assert_eq!(hazard["activities"][0]["rules"][0]["slug"], "fixture-aura");
    assert!(
        hazard["unavailable_fields"]
            .as_array()
            .is_none_or(|fields| fields.iter().all(|field| {
                !matches!(
                    field["field"].as_str(),
                    Some("activity.slug" | "activity.publication")
                )
            }))
    );
    assert!(hazard.get("image").is_none());
    assert!(hazard.get("publication_license").is_none());
    assert_eq!(hazard["provenance"]["image"]["state"], "missing");
    assert_eq!(
        hazard["provenance"]["publication_license"]["state"],
        "missing"
    );
    assert_eq!(
        hazard["provenance"]["source_metadata"][0],
        serde_json::json!({
            "field": "token_name",
            "value": {
                "state": "typed",
                "source_path": "/prototypeToken/name",
                "value": "Test Hazard"
            }
        })
    );
    assert_no_empty_containers(surface);

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
async fn record_route_transports_opaque_spell_form_and_cast_rank_selection() {
    let (status, body) = route_json(
        Method::GET,
        "/api/records/spells-srd:test?spell_form_id=spell-form:test&spell_cast_rank=5",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["surface"]["metadata"]["title"],
        "selected:spell-form:test:5"
    );

    let (status, body) = route_json(
        Method::GET,
        "/api/records/spells-srd:test?spell_form_id=spell-form:test&spell_cast_rank=300",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");
}

#[tokio::test]
async fn record_route_transports_reference_limits_and_rejects_invalid_values() {
    let (status, body) = route_json(
        Method::GET,
        "/api/records/spells-srd:test?reference_outgoing_limit=0&reference_backlink_limit=8",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["surface"]["metadata"]["title"],
        "references:Some(0):Some(8)"
    );

    for query in [
        "reference_outgoing_limit=invalid",
        "reference_outgoing_limit=256",
        "reference_backlink_limit=invalid",
        "reference_backlink_limit=256",
    ] {
        let (status, body) = route_json(
            Method::GET,
            &format!("/api/records/spells-srd:test?{query}"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "query: {query}");
        assert_eq!(body["code"], "invalid_request", "query: {query}");
    }
}

#[tokio::test]
async fn record_route_preserves_typed_edition_metadata() {
    let (status, body) = route_json(
        Method::GET,
        "/api/records/pathfinder-bestiary:KDRlxdIUADWHI6Vr",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["surface"]["metadata"]["edition"]["status"], "legacy");
    assert_eq!(
        body["surface"]["metadata"]["edition"]["counterparts"][0]["role"],
        "remastered_counterpart"
    );
    assert_eq!(
        body["surface"]["metadata"]["edition"]["counterparts"][0]["record_key"],
        "pathfinder-monster-core:MSm1im7lZA5i82rz"
    );
    assert_eq!(
        body["surface"]["metadata"]["edition"]["counterparts"][0]["title"],
        "Air Scamp"
    );
}

#[tokio::test]
async fn record_route_preserves_typed_domain_failure_distinct_from_empty_omission() {
    let (status, body) = route_json(Method::GET, "/api/records/creatures:typedFailure", None).await;
    assert_eq!(status, StatusCode::OK);
    let creature = &body["surface"]["presentation"]["body"];
    assert!(creature.get("movement").is_none());
    let shield = &creature["defenses"]["shield"];
    assert_eq!(shield["armor_class_bonus"], 2);
    assert!(shield.get("broken_threshold").is_none());
    assert_eq!(shield["hardness"], 5);
    assert_eq!(shield["maximum_hit_points"], 20);
    assert_eq!(
        creature["unavailable_domains"]["defenses"]["causes"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    let frequency = &creature["activities"][0]["frequency"];
    assert_eq!(frequency["maximum"], 1);
    assert!(frequency.get("period").is_none());
    assert_eq!(frequency["display"], "1");
    assert_eq!(
        creature["unavailable_domains"]["activities"]["causes"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        creature["unavailable_domains"]["movement"]["causes"][0]["state"],
        "unsupported"
    );
    assert_eq!(
        creature["unavailable_domains"]["movement"]["causes"][0]["field"],
        "movement_mode"
    );
    assert!(
        creature["unavailable_domains"]["movement"]["causes"][0]
            .get("source_path")
            .is_none()
    );
    let unmodeled = &creature["unavailable_domains"]["skills"]["causes"][0];
    assert_eq!(unmodeled["state"], "unsupported");
    assert_eq!(unmodeled["field"], "unmodeled_skill");
    let hostile_key = "<img src=x onerror=alert(1)> ../../etc/passwd\nskill";
    assert_eq!(unmodeled["unmodeled_skill"]["authored_key"], hostile_key);
    assert_eq!(unmodeled["unmodeled_skill"]["authored_order"], 3);
    assert_eq!(
        unmodeled["unmodeled_skill"]["source_entries"][0]["authored_key"],
        hostile_key
    );
    assert_eq!(unmodeled["unmodeled_skill"]["base"]["state"], "null");
    assert_eq!(
        unmodeled["unmodeled_skill"]["reason"],
        "unknown_authored_key"
    );
    assert!(unmodeled.get("source_path").is_none());
    assert!(unmodeled.get("raw_json").is_none());
    assert!(unmodeled.get("diagnostic").is_none());
    assert_eq!(creature["unmodeled_skills"][0]["authored_key"], hostile_key);
    assert_eq!(
        creature["unmodeled_skills"][0]["component_id"],
        "unmodeled-skill-1"
    );
    assert!(creature["unmodeled_skills"][0].get("source_path").is_none());
}

#[tokio::test]
async fn record_route_serializes_owned_activity_and_spell_content_without_flattening() {
    let (status, body) =
        route_json(Method::GET, "/api/records/creatures:activityContent", None).await;
    assert_eq!(status, StatusCode::OK);
    let creature = &body["surface"]["presentation"]["body"];
    assert_eq!(
        creature["activities"][0]["provenance"]["nested_source_id"],
        "source-occurrence-plague"
    );
    assert_eq!(
        creature["spellcasting"][0]["provenance"]["stable_source_locator"],
        "items/entry-occult"
    );
    assert_eq!(
        creature["spellcasting"][0]["spells"][0]["provenance"]["nested_source_id"],
        "source-bind-soul"
    );
    assert!(
        creature["activities"][0]["provenance"]
            .get("source_path")
            .is_none()
    );
    let activity_content = &creature["activities"][0]["content"][0];
    assert_eq!(activity_content["content_key"], "item:plague:description");
    assert_eq!(
        activity_content["blocks"][0]["spans"][0]["span_type"],
        "check"
    );
    assert_eq!(
        activity_content["blocks"][0]["spans"][0]["display"],
        "Fortitude DC 28"
    );
    assert_eq!(
        activity_content["blocks"][0]["spans"][0]["statistic"],
        "fortitude"
    );
    assert_eq!(
        activity_content["blocks"][0]["spans"][0]["difficulty_class"],
        28
    );
    assert_eq!(activity_content["blocks"][1]["block_type"], "divider");
    assert!(activity_content.get("owner").is_none());
    assert!(activity_content.get("text").is_none());
    assert_eq!(
        creature["spellcasting"][0]["spells"][0]["content"][0]["content_key"],
        "bind-soul"
    );
    assert_eq!(
        creature["standalone_spells"][0]["content"][0]["content_key"],
        "control-weather"
    );
    assert_eq!(creature["content"][0]["content_key"], "heartstone");
    assert_eq!(creature["content"].as_array().map(Vec::len), Some(1));
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
            "filter": {
                "clauses": [
                    {
                        "id": "kind-include_any",
                        "field": "kind",
                        "operator": "include_any",
                        "values": ["action"]
                    }
                ]
            }
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
    assert!(body["participants"][0].get("record_view").is_some());
    assert!(body["participants"][0].get("surface").is_none());
    assert_eq!(
        body["participants"][0]["record_view"]["encounter"]["level"]["base_value"],
        5
    );
    assert_eq!(
        body["participants"][0]["record_view"]["encounter"]["level"]["adjusted_value"],
        6
    );
    assert!(
        body["participants"][0]["record_view"]["encounter"]
            .get("adjusted_level")
            .is_none()
    );
    for known_empty_collection in [
        "skills",
        "resources",
        "spellcasting",
        "activities",
        "conditions",
        "automation_limitations",
    ] {
        assert!(
            body["participants"][0]["record_view"]["encounter"]
                .get(known_empty_collection)
                .is_none()
        );
    }
    assert_no_empty_containers(&body["participants"][0]["record_view"]);

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
    assert_eq!(body["encounter"]["encounter_key"], "ambush");
    assert_eq!(body["encounter"]["slug"], "renamed-ambush");

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
    assert_eq!(body["encounter"]["encounter_key"], "ambush");

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
    assert_eq!(body["encounter"]["encounter_key"], "ambush");

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/participant_a",
        Some(json!({
            "participant_key": "ignored",
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
    assert_eq!(body["participant_key"], "participant_a");
    assert_eq!(body["display_name"], "Renamed Goblin");
    assert_eq!(
        body["record_view"]["encounter"]["action_budget"]["can_act"]["available"],
        true
    );
    assert_eq!(
        body["record_view"]["encounter"]["action_budget"]["can_react"]["available"],
        true
    );
    write_action_budget_api_sample("api-participant-active.json", &body);

    let (status, defeated_body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/participant_a",
        Some(json!({
            "participant_key": "ignored",
            "display_name": "Renamed Goblin",
            "side": "enemy",
            "participant_variant": "normal",
            "initiative": 19,
            "max_hp": 12,
            "current_hp": 0,
            "temporary_hp": 1,
            "defeated": true,
            "hidden": false,
            "note": "wounded"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let defeated_budget = &defeated_body["record_view"]["encounter"]["action_budget"];
    assert_eq!(defeated_body["defeated"], true);
    assert_eq!(defeated_budget["actions"]["adjusted_value"], 3);
    assert_eq!(defeated_budget["reactions"]["adjusted_value"], 1);
    for capability in ["can_act", "can_react"] {
        assert_eq!(defeated_budget[capability]["available"], false);
        assert_eq!(
            defeated_budget[capability]["provenance"]["source"]["source_type"],
            "participant_state"
        );
        assert_eq!(
            defeated_budget[capability]["reason"],
            "Defeated participants cannot act or react."
        );
    }
    write_action_budget_api_sample("api-participant-defeated.json", &defeated_body);

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
    assert_eq!(body["encounter"]["encounter_key"], "ambush");

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
    assert_eq!(body["current_turn_participant_key"], "participant_a");

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/conditions",
        Some(json!({
            "participant_key": "ignored",
            "name": "Clumsy",
            "value": 1,
            "duration_rounds": 2
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["participants"][0]["record_view"]["encounter"]["conditions"][0]["name"],
        "Clumsy"
    );
    assert_eq!(
        body["participants"][0]["record_view"]["encounter"]["automation_limitations"][0]["code"],
        "condition_attack_adjustment_partial"
    );
    assert_eq!(
        body["participants"][0]["record_view"]["encounter"]["automation_limitations"][0]["target"],
        json!({"target_type": "condition", "condition_id": 7})
    );
    assert!(
        body["participants"][0]["record_view"]["encounter"]
            .get("unapplied_facts")
            .is_none()
    );
    assert_no_empty_containers(&body["participants"][0]["record_view"]);

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/spell-casts",
        Some(json!({
            "spell_occurrence_id": "spell-shadow-blast",
            "spend_target": {
                "target_type": "innate_use",
                "entry_id": "entry-innate",
                "spell_occurrence_id": "spell-shadow-blast"
            },
            "operation": "cast_one"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["operation"], "cast_one");
    assert_eq!(body["participant_key"], "participant_a");
    assert_eq!(body["before"]["state"]["remaining"], 2);
    assert_eq!(body["after"]["state"]["remaining"], 1);
    assert_eq!(
        body["after"]["spend_target"]["spell_occurrence_id"],
        "spell-shadow-blast"
    );
    write_spell_cast_api_sample("api-spell-cast-before-after.json", &body);

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/spell-casts",
        Some(json!({
            "spell_occurrence_id": "spell-shadow-blast",
            "spend_target": {
                "target_type": "innate_use",
                "entry_id": "wrong-entry",
                "spell_occurrence_id": "spell-shadow-blast"
            },
            "operation": "cast_one"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/spell-casts",
        Some(json!({
            "spell_occurrence_id": "spell-shadow-blast",
            "spend_target": {
                "target_type": "spontaneous_pool",
                "entry_id": "entry-spontaneous",
                "rank": 9_007_199_254_740_992_i64
            },
            "operation": "cast_one"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");
    assert!(
        body["message"]
            .as_str()
            .expect("message should be a string")
            .contains("JavaScript safe-integer range")
    );

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/spell-casts",
        Some(json!({
            "spell_occurrence_id": "spell-shadow-blast",
            "spend_target": {
                "target_type": "innate_use",
                "entry_id": "entry-innate",
                "spell_occurrence_id": "spell-shadow-blast"
            },
            "operation": "restore_one"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["operation"], "restore_one");
    assert_eq!(body["before"]["state"]["remaining"], 1);
    assert_eq!(body["after"]["state"]["remaining"], 2);

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/reset",
        Some(json!({ "confirmation": "reset_participant" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["participant_key"], "participant_a");
    assert_eq!(
        body["reset_domains"],
        json!([
            "hit_points",
            "defeated",
            "conditions",
            "initiative_turn_state",
            "variant_adjustments",
            "action_budget",
            "spell_resources"
        ])
    );
    assert_eq!(
        body["preserved_domains"],
        json!(["display_name", "notes", "visibility", "side"])
    );
    assert_eq!(body["cleared_current_turn"], true);
    assert_eq!(body["participant"]["reset"]["available"], true);

    let (status, body) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/reset",
        Some(json!({ "confirmation": "confirmed" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/participant_a/conditions/7",
        Some(json!({
            "condition_id": 999,
            "name": "Clumsy",
            "value": 2,
            "duration_rounds": 1
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["participants"][0]["record_view"]["encounter"]["conditions"][0]["condition_id"],
        7
    );
    assert_eq!(
        body["participants"][0]["record_view"]["encounter"]["conditions"][0]["value"],
        2
    );

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/wrong_participant/conditions/7",
        Some(json!({
            "condition_id": 999,
            "name": "Clumsy",
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
    assert!(
        body["participants"][0]["record_view"]["encounter"]
            .get("conditions")
            .is_none()
    );
    assert_no_empty_containers(&body["participants"][0]["record_view"]);

    let (status, body) = route_json(
        Method::DELETE,
        "/api/encounters/ambush/participants/participant_a",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter"]["encounter_key"], "ambush");

    let (status, body) = route_json(Method::DELETE, "/api/encounters/ambush", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["encounter_key"], "ambush");
    assert_eq!(body["deleted"], true);

    let (status, body) = route_json(Method::GET, "/api/encounters/missing", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_not_found");
}

#[tokio::test]
async fn encounter_routes_accept_omitted_numeric_optionals_and_reject_unsafe_values() {
    let (status, _) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/record",
        Some(json!({
            "encounter_ref": "ignored",
            "record_ref": "actors:testCreature",
            "quantity": 1
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/manual",
        Some(json!({
            "encounter_ref": "ignored",
            "display_name": "Kyra"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/participant_a",
        Some(json!({
            "participant_key": "ignored",
            "display_name": "Goblin",
            "side": "enemy",
            "participant_variant": "normal",
            "temporary_hp": 0,
            "defeated": false,
            "hidden": false
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = route_json(
        Method::POST,
        "/api/encounters/ambush/participants/participant_a/conditions",
        Some(json!({
            "participant_key": "ignored",
            "name": "Clumsy"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/participant_a/conditions/7",
        Some(json!({
            "condition_id": 999,
            "name": "Clumsy",
            "value": 3,
            "duration_rounds": 1
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["participants"][0]["record_view"]["encounter"]["conditions"][0]["value"],
        3
    );

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/wrong_participant/conditions/7",
        Some(json!({
            "condition_id": 999,
            "name": "Clumsy",
            "value": 3
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_participant_not_found");

    let (status, body) = route_json(
        Method::PATCH,
        "/api/encounters/ambush/participants/participant_a/conditions/7",
        Some(json!({
            "condition_id": 7,
            "name": "Clumsy",
            "value": 9_007_199_254_740_992_i64
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_request");
    assert!(
        body["message"]
            .as_str()
            .expect("message should be string")
            .contains("JavaScript safe-integer range")
    );
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

    fn record_detail(
        &self,
        record_key: &str,
        request: RecordDetailRequest,
    ) -> Result<RecordDetailView, AppServiceError> {
        if record_key == "pathfinder-bestiary:KDRlxdIUADWHI6Vr" {
            return Ok(RecordDetailView {
                surface: air_mephit_surface(),
            });
        }
        if record_key == "creatures:activityContent" {
            return Ok(RecordDetailView {
                surface: activity_content_surface(),
            });
        }
        if record_key == "creatures:typedFailure" {
            return Ok(RecordDetailView {
                surface: typed_failure_surface(),
            });
        }
        if record_key == "hazards:testHazard" {
            return Ok(RecordDetailView {
                surface: hazard_record_surface(),
            });
        }
        let mut detail = RecordDetailView {
            surface: unavailable_surface(
                Some(record_key),
                "Test Action 1",
                RecordSurfaceProfileView::RecordDetail,
                None,
            ),
        };
        if request.reference_outgoing_limit.is_some() || request.reference_backlink_limit.is_some()
        {
            detail.surface.metadata.title = format!(
                "references:{:?}:{:?}",
                request.reference_outgoing_limit, request.reference_backlink_limit
            );
        }
        if let (Some(form_id), Some(cast_rank)) = (request.spell_form_id, request.spell_cast_rank) {
            detail.surface.metadata.title = format!("selected:{form_id}:{cast_rank}");
        }
        Ok(detail)
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
    ) -> Result<EncounterUpdateView, AppServiceError> {
        Ok(EncounterUpdateView {
            encounter: EncounterSummaryView {
                encounter_key: request.encounter_key,
                slug: request.slug,
                name: request.name,
                description: request.description,
                status: request.status,
                round_number: 1,
                participant_count: 2,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-02T00:00:00Z".to_string(),
            },
        })
    }

    fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        Ok(encounter_detail(&request.encounter_ref, None, false))
    }

    fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        Ok(encounter_detail(&request.encounter_ref, None, false))
    }

    fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> Result<EncounterParticipantView, AppServiceError> {
        if encounter_ref == "missing" {
            return Err(encounter_not_found(encounter_ref));
        }
        let display_name = request.display_name.clone();
        Ok(EncounterParticipantView {
            participant_key: request.participant_key,
            record_key: Some("actors:testCreature".to_string()),
            participant_kind: EncounterParticipantKindView::Creature,
            participant_variant: request.participant_variant,
            status: EncounterParticipantStatusView::Active,
            position: 1,
            display_name: request.display_name,
            side: request.side,
            initiative: request.initiative,
            initiative_order: 1,
            defeated: request.defeated,
            hidden: request.hidden,
            note: request.note.clone(),
            note_hint: request.note,
            reset: EncounterParticipantResetAvailabilityView {
                available: true,
                unavailable_reason: None,
            },
            record_view: unavailable_surface(
                Some("actors:testCreature"),
                &display_name,
                RecordSurfaceProfileView::EncounterParticipant,
                Some(test_runtime(
                    request.max_hp,
                    request.current_hp,
                    request.temporary_hp,
                    false,
                    request.defeated,
                )),
            ),
        })
    }

    fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        request: ReorderEncounterParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        let mut detail = encounter_detail(encounter_ref, None, false);
        if request.placement == ReorderEncounterParticipantPlacementView::Before {
            detail.participants.reverse();
        }
        Ok(detail)
    }

    fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        _participant_key: &str,
    ) -> Result<EncounterDetailView, AppServiceError> {
        Ok(encounter_detail(encounter_ref, None, false))
    }

    fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        Ok(encounter_detail(
            &request.encounter_ref,
            request.participant_key,
            false,
        ))
    }

    fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        Ok(encounter_detail(
            encounter_ref,
            None,
            request.name.as_deref() == Some("Clumsy"),
        ))
    }

    fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        if participant_key != "participant_a" {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                format!(
                    "encounter participant condition `{}` was not found for participant `{participant_key}`",
                    request.condition_id
                ),
            ));
        }
        let mut detail = encounter_detail(encounter_ref, None, true);
        runtime_mut(&mut detail.participants[0]).conditions[0].condition_id = request.condition_id;
        if let EncounterRuntimeAutomationLimitationTargetView::Condition { condition_id } =
            &mut runtime_mut(&mut detail.participants[0]).automation_limitations[0].target
        {
            *condition_id = request.condition_id;
        }
        runtime_mut(&mut detail.participants[0]).conditions[0].value = request.value;
        Ok(detail)
    }

    fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
    ) -> Result<EncounterDetailView, AppServiceError> {
        if participant_key != "participant_a" {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                format!(
                    "encounter participant condition `{condition_id}` was not found for participant `{participant_key}`"
                ),
            ));
        }
        Ok(encounter_detail(encounter_ref, None, false))
    }

    fn mutate_encounter_spell_cast(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: EncounterSpellCastRequest,
    ) -> Result<EncounterSpellCastResultView, AppServiceError> {
        if encounter_ref != "ambush" || participant_key != "participant_a" {
            return Err(encounter_not_found(encounter_ref));
        }
        let EncounterSpellSpendTargetView::InnateUse {
            entry_id,
            spell_occurrence_id,
        } = &request.spend_target
        else {
            return Err(AppServiceError::invalid_request(
                "fixture expects an innate spell use",
            ));
        };
        if entry_id.as_deref() != Some("entry-innate")
            || spell_occurrence_id != &request.spell_occurrence_id
        {
            return Err(AppServiceError::invalid_request(
                "fixture spell identity mismatch",
            ));
        }
        let availability = |remaining| EncounterSpellCastAvailabilityView {
            spend_target: Some(request.spend_target.clone()),
            available: remaining > 0,
            state: EncounterSpellCastStateView::Tracked {
                maximum: 2,
                initial_remaining: 2,
                remaining,
            },
            blocked_reason: None,
        };
        let (before, after) = match request.operation {
            EncounterSpellCastOperationView::CastOne => (availability(2), availability(1)),
            EncounterSpellCastOperationView::RestoreOne => (availability(1), availability(2)),
        };
        Ok(EncounterSpellCastResultView {
            operation: request.operation,
            participant_key: participant_key.to_string(),
            spell_occurrence_id: request.spell_occurrence_id,
            before,
            after,
            participant: encounter_participant(participant_key, "Goblin", Some(18), false),
        })
    }

    fn reset_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        _request: ResetEncounterParticipantRequest,
    ) -> Result<EncounterParticipantResetResultView, AppServiceError> {
        if encounter_ref != "ambush" || participant_key != "participant_a" {
            return Err(encounter_not_found(encounter_ref));
        }
        Ok(EncounterParticipantResetResultView {
            participant_key: participant_key.to_string(),
            reset_domains: vec![
                EncounterParticipantResetDomainView::HitPoints,
                EncounterParticipantResetDomainView::Defeated,
                EncounterParticipantResetDomainView::Conditions,
                EncounterParticipantResetDomainView::InitiativeTurnState,
                EncounterParticipantResetDomainView::VariantAdjustments,
                EncounterParticipantResetDomainView::ActionBudget,
                EncounterParticipantResetDomainView::SpellResources,
            ],
            preserved_domains: vec![
                EncounterParticipantPreservedDomainView::DisplayName,
                EncounterParticipantPreservedDomainView::Notes,
                EncounterParticipantPreservedDomainView::Visibility,
                EncounterParticipantPreservedDomainView::Side,
            ],
            cleared_current_turn: true,
            participant: encounter_participant(participant_key, "Goblin", Some(18), false),
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

fn air_mephit_surface() -> RecordSurfaceView {
    let mut surface = unavailable_surface(
        Some("pathfinder-bestiary:KDRlxdIUADWHI6Vr"),
        "Air Mephit",
        RecordSurfaceProfileView::RecordDetail,
        None,
    );
    surface.metadata.edition = Some(RecordSurfaceEditionView {
        status: RecordSurfaceEditionStatusView::Legacy,
        counterparts: vec![RecordSurfaceEditionCounterpartView {
            role: RecordSurfaceEditionCounterpartRoleView::RemasteredCounterpart,
            record_key: "pathfinder-monster-core:MSm1im7lZA5i82rz".to_string(),
            title: "Air Scamp".to_string(),
        }],
    });
    surface
}

fn activity_content_surface() -> RecordSurfaceView {
    let content = |content_key: &str, label: &str| CreatureSurfaceContentView {
        content_key: content_key.to_string(),
        role: CreatureSurfaceContentRoleView::EmbeddedCapability,
        authored_order: 0,
        label: Some(label.to_string()),
        blocks: vec![
            CreatureSurfaceContentBlockView::Paragraph {
                spans: vec![CreatureSurfaceContentInlineView::Check {
                    display: "Fortitude DC 28".to_string(),
                    statistic: Some("fortitude".to_string()),
                    difficulty_class: Some(28),
                }],
            },
            CreatureSurfaceContentBlockView::Divider,
        ],
        content_hash: "fixture".to_string(),
        visibility: "public".to_string(),
        provenance: CreatureSurfaceContentProvenanceView {
            source_record_key: "creatures:activityContent".to_string(),
            relative_source_path: format!("items[{content_key}].system.description.value"),
            field_family: "embedded_item_description".to_string(),
            nested_source_id: Some(content_key.to_string()),
        },
    };
    RecordSurfaceView {
        metadata: RecordSurfaceMetadataView {
            record_key: Some("creatures:activityContent".to_string()),
            title: "Activity Content".to_string(),
            kind: "creature".to_string(),
            kind_label: "Creature".to_string(),
            level: Some(9),
            rarity: None,
            traits: Vec::new(),
            edition: None,
            source: None,
        },
        profile: RecordSurfaceProfileView::RecordDetail,
        presentation: RecordSurfacePresentationView::Creature {
            body: Box::new(CreatureSurfaceView {
                teaser: None,
                size: None,
                adjustment: None,
                initiative: None,
                vitals: None,
                defenses: None,
                saves: None,
                awareness: None,
                abilities: None,
                skills: None,
                unmodeled_skills: None,
                movement: None,
                resources: None,
                spellcasting: Some(vec![CreatureSurfaceSpellcastingView {
                    occurrence_id: "entry-occult".to_string(),
                    authored_order: 0,
                    provenance: occurrence_provenance("entry-occult"),
                    label: "Occult Innate Spells".to_string(),
                    preparation: Some("innate".to_string()),
                    tradition: Some("occult".to_string()),
                    attack_modifier: Some(20),
                    difficulty_class: Some(28),
                    slots: None,
                    spells: vec![CreatureSurfaceSpellView {
                        occurrence_id: "bind-soul".to_string(),
                        authored_order: 1,
                        provenance: occurrence_provenance("bind-soul"),
                        label: "Bind Soul".to_string(),
                        target_record_key: None,
                        rank: Some(9),
                        context: None,
                        traits: vec!["spell".to_string()],
                        content: Some(vec![content("bind-soul", "Bind Soul")]),
                    }],
                }]),
                standalone_spells: Some(vec![CreatureSurfaceSpellView {
                    occurrence_id: "control-weather".to_string(),
                    authored_order: 2,
                    provenance: occurrence_provenance("control-weather"),
                    label: "Control Weather".to_string(),
                    target_record_key: Some("spells:control-weather".to_string()),
                    rank: Some(8),
                    context: None,
                    traits: vec!["spell".to_string()],
                    content: Some(vec![content("control-weather", "Control Weather")]),
                }]),
                activities: Some(vec![CreatureSurfaceActivityView {
                    occurrence_id: "occurrence:plague".to_string(),
                    authored_order: 0,
                    provenance: occurrence_provenance("occurrence-plague"),
                    activity_type: CreatureSurfaceActivityTypeView::Action,
                    label: "Abyssal Plague".to_string(),
                    traits: Vec::new(),
                    action_cost: None,
                    attack_effects: None,
                    category: None,
                    frequency: None,
                    requirements: None,
                    cost: None,
                    uses: None,
                    self_effect: None,
                    rolls: Vec::new(),
                    damage: Vec::new(),
                    content: Some(vec![content("item:plague:description", "Abyssal Plague")]),
                }]),
                rituals: None,
                equipment: None,
                lore: None,
                content: Some(vec![content("heartstone", "Heartstone")]),
                relationships: None,
                unavailable_domains: None,
                provenance: None,
            }),
        },
        issues: None,
        references: None,
        encounter: None,
    }
}

fn occurrence_provenance(id: &str) -> CreatureSurfaceOccurrenceProvenanceView {
    CreatureSurfaceOccurrenceProvenanceView {
        identity_stability: CreatureSurfaceOccurrenceIdentityStabilityView::StableNestedSourceId,
        nested_source_id: Some(format!("source-{id}")),
        stable_source_locator: Some(format!("items/{id}")),
        source_locators: Some(vec![CreatureSurfaceSourceLocatorView {
            locator: format!("items/{id}"),
            precedence: 0,
        }]),
    }
}

fn hostile_unmodeled_skill() -> CreatureSurfaceUnmodeledSkillView {
    let authored_key = "<img src=x onerror=alert(1)> ../../etc/passwd\nskill".to_string();
    CreatureSurfaceUnmodeledSkillView {
        component_id: "unmodeled-skill-1".to_string(),
        authored_order: 3,
        source_entries: Some(vec![CreatureSurfaceSkillSourceEntryView {
            authored_order: 0,
            authored_key: authored_key.clone(),
            modifier: CreatureSurfaceIntegerPresenceView::Null,
        }]),
        source_item_id: Some("source-unmodeled-skill-1".to_string()),
        authored_key,
        base: CreatureSurfaceIntegerPresenceView::Null,
        reason: CreatureSurfaceUnmodeledSkillReasonView::UnknownAuthoredKey,
    }
}

fn typed_failure_surface() -> RecordSurfaceView {
    RecordSurfaceView {
        metadata: RecordSurfaceMetadataView {
            record_key: Some("creatures:typedFailure".to_string()),
            title: "Typed Failure".to_string(),
            kind: "creature".to_string(),
            kind_label: "Creature".to_string(),
            level: Some(1),
            rarity: None,
            traits: Vec::new(),
            edition: None,
            source: None,
        },
        profile: RecordSurfaceProfileView::RecordDetail,
        presentation: RecordSurfacePresentationView::Creature {
            body: Box::new(CreatureSurfaceView {
                teaser: None,
                size: None,
                adjustment: None,
                initiative: None,
                vitals: None,
                defenses: Some(CreatureSurfaceDefensesView {
                    armor_class: None,
                    armor_class_details: None,
                    hardness: None,
                    shield: Some(CreatureSurfaceShieldView {
                        armor_class_bonus: Some(2),
                        broken_threshold: None,
                        hardness: Some(5),
                        maximum_hit_points: Some(20),
                    }),
                    immunities: Vec::new(),
                    resistances: Vec::new(),
                    weaknesses: Vec::new(),
                    provenance: CreatureSurfaceFactProvenanceView {
                        owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
                        field: CreatureSurfaceSourceFieldView::Defenses,
                    },
                }),
                saves: None,
                awareness: None,
                abilities: None,
                skills: None,
                unmodeled_skills: Some(vec![hostile_unmodeled_skill()]),
                movement: None,
                resources: None,
                spellcasting: None,
                standalone_spells: None,
                activities: Some(vec![CreatureSurfaceActivityView {
                    occurrence_id: "partial-frequency".to_string(),
                    authored_order: 0,
                    provenance: occurrence_provenance("partial-frequency"),
                    activity_type: CreatureSurfaceActivityTypeView::Action,
                    label: "Partial Frequency".to_string(),
                    traits: Vec::new(),
                    action_cost: None,
                    attack_effects: None,
                    category: None,
                    frequency: Some(CreatureSurfaceFrequencyView {
                        maximum: Some(1),
                        period: None,
                        display: Some("1".to_string()),
                    }),
                    requirements: None,
                    cost: None,
                    uses: None,
                    self_effect: None,
                    rolls: Vec::new(),
                    damage: Vec::new(),
                    content: None,
                }]),
                rituals: None,
                equipment: None,
                lore: None,
                content: None,
                relationships: None,
                unavailable_domains: Some(CreatureSurfaceUnavailableDomainsView {
                    classification: None,
                    initiative: None,
                    vitals: None,
                    defenses: Some(CreatureSurfaceDomainUnavailableView {
                        causes: vec![CreatureSurfaceUnavailableCauseView {
                            state: CreatureSurfaceUnavailableStateView::Null,
                            field: CreatureSurfaceUnavailableFieldView::ShieldBrokenThreshold,
                            component_id: None,
                            provenance: CreatureSurfaceFactProvenanceView {
                                owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
                                field: CreatureSurfaceSourceFieldView::Defenses,
                            },
                            unmodeled_skill: None,
                            message: "Display only.".to_string(),
                        }],
                    }),
                    saves: None,
                    awareness: None,
                    abilities: None,
                    skills: Some(CreatureSurfaceDomainUnavailableView {
                        causes: vec![CreatureSurfaceUnavailableCauseView {
                            state: CreatureSurfaceUnavailableStateView::Unsupported,
                            field: CreatureSurfaceUnavailableFieldView::UnmodeledSkill,
                            component_id: Some("unmodeled-skill-1".to_string()),
                            provenance: CreatureSurfaceFactProvenanceView {
                                owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
                                field: CreatureSurfaceSourceFieldView::Skills,
                            },
                            unmodeled_skill: Some(hostile_unmodeled_skill()),
                            message: "The source supplied an unrecognized skill key.".to_string(),
                        }],
                    }),
                    movement: Some(CreatureSurfaceDomainUnavailableView {
                        causes: vec![CreatureSurfaceUnavailableCauseView {
                            state: CreatureSurfaceUnavailableStateView::Unsupported,
                            field: CreatureSurfaceUnavailableFieldView::MovementMode,
                            component_id: Some("speed-1".to_string()),
                            provenance: CreatureSurfaceFactProvenanceView {
                                owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
                                field: CreatureSurfaceSourceFieldView::Movement,
                            },
                            unmodeled_skill: None,
                            message: "Display only.".to_string(),
                        }],
                    }),
                    resources: None,
                    spellcasting: None,
                    activities: Some(CreatureSurfaceDomainUnavailableView {
                        causes: vec![CreatureSurfaceUnavailableCauseView {
                            state: CreatureSurfaceUnavailableStateView::Missing,
                            field: CreatureSurfaceUnavailableFieldView::ActionFrequencyPeriod,
                            component_id: Some("partial-frequency".to_string()),
                            provenance: CreatureSurfaceFactProvenanceView {
                                owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
                                field: CreatureSurfaceSourceFieldView::EmbeddedEntities,
                            },
                            unmodeled_skill: None,
                            message: "Display only.".to_string(),
                        }],
                    }),
                    equipment: None,
                    lore: None,
                    relationships: None,
                }),
                provenance: Some(CreatureSurfaceProvenanceView {
                    source_path: "packs/creatures/typed-failure.json".to_string(),
                    source_contract_version: "test".to_string(),
                    source_system_version: "test".to_string(),
                    source_upstream_commit: "test".to_string(),
                }),
            }),
        },
        issues: None,
        references: None,
        encounter: None,
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
        defeated: false,
        hidden: false,
        note: Some("wounded".to_string()),
        note_hint: Some("wounded".to_string()),
        reset: EncounterParticipantResetAvailabilityView {
            available: true,
            unavailable_reason: None,
        },
        record_view: unavailable_surface(
            Some("actors:testCreature"),
            display_name,
            RecordSurfaceProfileView::EncounterParticipant,
            Some(test_runtime(Some(12), Some(6), 0, include_condition, false)),
        ),
    }
}

fn test_runtime(
    maximum_hp: Option<i64>,
    current_hp: Option<i64>,
    temporary_hp: i64,
    include_condition: bool,
    defeated: bool,
) -> EncounterRuntimeView {
    let participant_provenance = || RuntimeFactProvenanceView {
        source: RuntimeFactSourceView::ParticipantState,
        canonical_target: None,
    };
    EncounterRuntimeView {
        hazard: None,
        level: Some(RuntimeNumberView {
            label: "Level".to_string(),
            base_value: 5,
            adjusted_value: 6,
            modifiers: Vec::new(),
            suppressed_modifiers: Vec::new(),
            provenance: RuntimeFactProvenanceView {
                source: RuntimeFactSourceView::CanonicalRecord,
                canonical_target: Some(RuntimeCanonicalTargetView::Level),
            },
        }),
        vitals: Some(EncounterRuntimeVitalsView {
            maximum_hp: maximum_hp.map(|value| RuntimeNumberView {
                label: "Maximum HP".to_string(),
                base_value: value,
                adjusted_value: value,
                modifiers: Vec::new(),
                suppressed_modifiers: Vec::new(),
                provenance: participant_provenance(),
            }),
            current_hp,
            temporary_hp,
        }),
        defenses: None,
        saves: None,
        awareness: None,
        abilities: None,
        skills: Vec::new(),
        movement: None,
        resources: Vec::new(),
        spellcasting: Vec::new(),
        standalone_spells: Vec::new(),
        activities: Vec::new(),
        action_budget: Some(test_action_budget(defeated)),
        conditions: include_condition
            .then(|| EncounterRuntimeConditionView {
                condition_id: 7,
                condition_key: None,
                name: "Clumsy".to_string(),
                value: Some(1),
                source_participant_key: None,
                duration_rounds: Some(2),
                note: None,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
                provenance: RuntimeFactProvenanceView {
                    source: RuntimeFactSourceView::Condition {
                        condition_id: 7,
                        condition_ref: "clumsy".to_string(),
                        label: "Clumsy 1".to_string(),
                    },
                    canonical_target: None,
                },
            })
            .into_iter()
            .collect(),
        automation_limitations: include_condition
            .then(|| EncounterRuntimeAutomationLimitationView {
                code:
                    EncounterRuntimeAutomationLimitationCodeView::ConditionAttackAdjustmentPartial,
                target: EncounterRuntimeAutomationLimitationTargetView::Condition {
                    condition_id: 7,
                },
                message:
                    "Only structured activity attack rolls receive this Dexterity-based penalty."
                        .to_string(),
            })
            .into_iter()
            .collect(),
    }
}

fn test_action_budget(defeated: bool) -> EncounterRuntimeActionBudgetView {
    let runtime_rule_provenance = || RuntimeFactProvenanceView {
        source: RuntimeFactSourceView::RuntimeRule {
            rule: RuntimeRuleView::ActionBudget,
        },
        canonical_target: None,
    };
    let count = |label: &str, value| RuntimeCountView {
        label: label.to_string(),
        base_value: value,
        adjusted_value: value,
        segments: vec![RuntimeCountSegmentView {
            label: "Base".to_string(),
            value,
            restricted: false,
            reason: None,
        }],
        adjustments: Vec::new(),
        suppressed_adjustments: Vec::new(),
        provenance: runtime_rule_provenance(),
    };
    let capability = RuntimeCapabilityView {
        available: !defeated,
        provenance: defeated.then_some(RuntimeFactProvenanceView {
            source: RuntimeFactSourceView::ParticipantState,
            canonical_target: None,
        }),
        reason: defeated.then(|| "Defeated participants cannot act or react.".to_string()),
    };
    EncounterRuntimeActionBudgetView {
        actions: count("Actions", 3),
        reactions: count("Reactions", 1),
        can_act: capability.clone(),
        can_react: capability,
        notes: Vec::new(),
    }
}

fn write_action_budget_api_sample(file_name: &str, value: &Value) {
    let Ok(root) = std::env::var("F2_ACTION_BUDGET_SAMPLE_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    fs::create_dir_all(&root).expect("action-budget sample root should be creatable");
    let mut bytes = serde_json::to_vec_pretty(value).expect("API sample should serialize");
    bytes.push(b'\n');
    fs::write(root.join(file_name), bytes).expect("API sample should write");
}

fn write_spell_cast_api_sample(file_name: &str, value: &Value) {
    let Ok(root) = std::env::var("F2_SPELL_CAST_SAMPLE_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    fs::create_dir_all(&root).expect("spell-cast sample root should be creatable");
    let mut bytes = serde_json::to_vec_pretty(value).expect("API sample should serialize");
    bytes.push(b'\n');
    fs::write(root.join(file_name), bytes).expect("API sample should write");
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
        surface: unavailable_surface(
            Some("actions:testAction1"),
            "Test Action 1",
            RecordSurfaceProfileView::SearchCompact,
            None,
        ),
    }
}

fn hazard_record_surface() -> RecordSurfaceView {
    let activity = |occurrence_id: &str,
                    activity_type: atlas_app_model::HazardSurfaceActivityTypeView,
                    authored_order| atlas_app_model::HazardSurfaceActivityView {
        occurrence_id: occurrence_id.to_string(),
        entity_id: format!("entity-{authored_order}"),
        authored_order,
        source_ordinal: authored_order,
        identity_stability:
            atlas_app_model::HazardSurfaceOccurrenceIdentityStabilityView::StableSourceIdentity,
        label: format!("Activity {authored_order}"),
        activity_type,
        child_type: None,
        traits: None,
        action_cost: None,
        attack_mode: None,
        frequency: None,
        category: None,
        death_note: None,
        self_effect: None,
        attack_bonus: None,
        attack_effects: None,
        damage: None,
        rules: (authored_order == 0).then(|| {
            vec![atlas_app_model::HazardSurfaceRuleView::Aura {
                authored_order: 0,
                radius: Some(5),
                slug: Some("fixture-aura".to_string()),
                traits: Vec::new(),
            }]
        }),
        content: None,
    };
    let mut surface = unavailable_surface(
        Some("hazards:testHazard"),
        "Test Hazard",
        RecordSurfaceProfileView::RecordDetail,
        None,
    );
    surface.metadata.kind = "hazard".to_string();
    surface.metadata.kind_label = "Hazard".to_string();
    surface.metadata.source = Some(RecordSurfaceSourceView {
        publication_title: None,
        pack_label: "Hazards".to_string(),
        document_type: "Actor".to_string(),
        record_type: "hazard".to_string(),
        source_path: None,
        source_contract_version: None,
        source_system_version: None,
        source_upstream_commit: None,
    });
    surface.presentation = RecordSurfacePresentationView::Hazard {
        body: Box::new(atlas_app_model::HazardSurfaceView {
            teaser: None,
            complexity: Some(atlas_app_model::HazardSurfaceComplexityView::Complex),
            size: None,
            emits_sound: None,
            detection: Some(atlas_app_model::HazardSurfaceDetectionView {
                stealth_modifier: Some(12),
                difficulty_class: Some(22),
                details: None,
            }),
            defenses: Some(atlas_app_model::HazardSurfaceDefensesView {
                applicability: atlas_app_model::HazardSurfaceDefenseApplicabilityView {
                    health: atlas_app_model::HazardSurfaceApplicabilityStateView::Unknown,
                    structure: atlas_app_model::HazardSurfaceApplicabilityStateView::Unknown,
                    rule_id: "pf2e-hazard-structural-applicability".into(),
                    rule_version: 1,
                },
                armor_class: Some(22),
                hardness: None,
                hit_points: None,
                saves: Some(atlas_app_model::HazardSurfaceSavesView {
                    fortitude: Some(0),
                    reflex: Some(8),
                    will: Some(4),
                }),
                immunities: None,
                weaknesses: None,
                resistances: None,
            }),
            lifecycle: None,
            activities: Some(vec![
                activity(
                    "occurrence-action",
                    atlas_app_model::HazardSurfaceActivityTypeView::Action,
                    0,
                ),
                activity(
                    "occurrence-strike",
                    atlas_app_model::HazardSurfaceActivityTypeView::Strike,
                    1,
                ),
                activity(
                    "occurrence-unsupported",
                    atlas_app_model::HazardSurfaceActivityTypeView::UnsupportedChild,
                    2,
                ),
            ]),
            content: None,
            relationships: None,
            unavailable_fields: None,
            provenance: atlas_app_model::HazardSurfaceProvenanceView {
                source_path: "packs/hazards/test-hazard.json".to_string(),
                source_contract_version: "test".to_string(),
                source_system_version: "test".to_string(),
                source_upstream_commit: "test".to_string(),
                convenience_rule_id: "pf2e-hazard-derived-conveniences".to_string(),
                convenience_rule_version: 1,
                image: atlas_app_model::HazardSurfaceProvenanceTextView::Missing,
                publication_license: atlas_app_model::HazardSurfaceProvenanceTextView::Missing,
                source_metadata: vec![
                    atlas_app_model::HazardSurfaceSourceMetadataFactView::TokenName {
                        value: atlas_app_model::HazardSurfaceSourceFactView::Typed {
                            source_path: "/prototypeToken/name".to_string(),
                            value: "Test Hazard".to_string(),
                        },
                    },
                ],
            },
        }),
    };
    surface
}

fn unavailable_surface(
    record_key: Option<&str>,
    title: &str,
    profile: RecordSurfaceProfileView,
    encounter: Option<EncounterRuntimeView>,
) -> RecordSurfaceView {
    RecordSurfaceView {
        metadata: RecordSurfaceMetadataView {
            record_key: record_key.map(str::to_string),
            title: title.to_string(),
            kind: "rule".to_string(),
            kind_label: "Rule".to_string(),
            level: None,
            rarity: None,
            traits: Vec::new(),
            edition: None,
            source: Some(RecordSurfaceSourceView {
                publication_title: None,
                pack_label: "Actions".to_string(),
                document_type: "Item".to_string(),
                record_type: "action".to_string(),
                source_path: None,
                source_contract_version: None,
                source_system_version: None,
                source_upstream_commit: None,
            }),
        },
        profile,
        presentation: RecordSurfacePresentationView::Unavailable {
            unavailable: SurfaceUnavailableView {
                reason: SurfaceUnavailableReasonView::RecordFamilyNotMigrated,
                requested_kind: "rule".to_string(),
                message: "Typed record presentation is unavailable for this record family."
                    .to_string(),
            },
        },
        issues: None,
        references: None,
        encounter,
    }
}

fn runtime_mut(participant: &mut EncounterParticipantView) -> &mut EncounterRuntimeView {
    participant
        .record_view
        .encounter
        .as_mut()
        .expect("fixture participant should expose encounter runtime")
}

fn assert_no_empty_containers(value: &Value) {
    assert_no_empty_containers_at(value, "$");
}

fn assert_no_empty_containers_at(value: &Value, path: &str) {
    match value {
        Value::Object(object) => {
            assert!(
                !object.is_empty(),
                "transport JSON must not contain empty objects at {path}"
            );
            for (key, child) in object {
                assert_no_empty_containers_at(child, &format!("{path}/{key}"));
            }
        }
        Value::Array(values) => {
            assert!(
                !values.is_empty(),
                "transport JSON must not contain empty arrays at {path}"
            );
            for (index, child) in values.iter().enumerate() {
                assert_no_empty_containers_at(child, &format!("{path}/{index}"));
            }
        }
        _ => {}
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
