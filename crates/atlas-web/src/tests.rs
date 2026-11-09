#[cfg(feature = "embedded-ui")]
use super::assets::WebAssets;
use super::router::router_with_state;
use super::service::{AtlasWebState, call_service};
use atlas_app_model::{AppError, AppErrorCode};
use atlas_app_service::AppServiceError;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use tower::ServiceExt;
mod definitions;
mod fixture;
mod read_abandonment_tests;
mod wire_conversion;
use fixture::{ACTION, ACTOR, Fixture, SECOND_ACTION, TABLE};

#[path = "../build_contract.rs"]
mod build_contract;

#[test]
fn browser_contract_uses_the_openapi_artifact_and_protocol_revision() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let contract = build_contract::load(&root).unwrap();
    assert_eq!(contract.identifier, super::compatibility::CONTRACT);
    assert_eq!(
        contract.watched_paths,
        [
            root.join(build_contract::METADATA),
            root.join(build_contract::SCHEMA)
        ]
    );
}

#[test]
fn strike_roll_schema_separates_attack_and_damage_fields() {
    let document = crate::openapi().unwrap();
    let schemas = &document["components"]["schemas"];
    let request = &schemas["RollEncounterStrikeRequest"];
    assert_eq!(request["additionalProperties"], false);
    assert!(
        request["required"]
            .as_array()
            .unwrap()
            .contains(&json!("action"))
    );
    assert!(request["properties"].get("attack_number").is_none());

    let action = &schemas["StrikeRollAction"];
    let attack = &action["oneOf"][0];
    let damage = &action["oneOf"][1];
    assert_eq!(attack["properties"]["kind"]["enum"], json!(["attack"]));
    assert_eq!(attack["properties"]["attack_number"]["minimum"], 1);
    assert_eq!(attack["properties"]["attack_number"]["maximum"], 3);
    assert_eq!(
        damage["properties"],
        json!({"kind":{"type":"string","enum":["damage"]}})
    );

    let result = &schemas["StrikeRollResult"];
    let attack = &result["oneOf"][0];
    let damage = &result["oneOf"][1];
    assert_eq!(attack["properties"]["kind"]["enum"], json!(["attack"]));
    assert_eq!(damage["properties"]["kind"]["enum"], json!(["damage"]));
    assert!(
        attack["required"]
            .as_array()
            .unwrap()
            .contains(&json!("attack_modifier"))
    );
    assert!(attack["properties"].get("damage").is_none());
    assert!(
        damage["required"]
            .as_array()
            .unwrap()
            .contains(&json!("persistent_total"))
    );
    assert!(damage["properties"].get("attack_modifier").is_none());
}

#[test]
fn schema_preserves_wire_contract_edges() {
    let document = crate::openapi().unwrap();
    let schemas = &document["components"]["schemas"];
    for name in crate::router::CLOSED_TAGGED_SCHEMAS {
        for variant in schemas[name]["oneOf"].as_array().unwrap() {
            let keyword = if variant["allOf"].is_array() {
                "unevaluatedProperties"
            } else {
                "additionalProperties"
            };
            assert_eq!(variant[keyword], false, "{name}: {variant}");
        }
    }
    assert!(
        serde_json::from_value::<atlas_domain::ResultSort>(
            json!({"kind": "relevance", "extra": true})
        )
        .is_err()
    );
    assert_eq!(schemas["QueryPredicate"]["unevaluatedProperties"], false);
    let predicate = json!({"kind": "boolean_constant", "value": true, "clause_id": "fixture"});
    assert!(serde_json::from_value::<atlas_domain::QueryPredicate>(predicate.clone()).is_ok());
    let mut invalid = predicate;
    invalid["extra"] = json!(true);
    assert!(serde_json::from_value::<atlas_domain::QueryPredicate>(invalid).is_err());

    let fact = serde_json::to_value(atlas_app_model::FactView::<String> {
        state: atlas_domain::QueryFieldState::Missing,
        value: None,
    })
    .unwrap();
    assert_eq!(fact, json!({"state": "missing", "value": null}));
    let fact_schema = &schemas["FactView_String"];
    assert!(
        fact_schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("value"))
    );
    assert!(
        fact_schema["properties"]["value"]["anyOf"]
            .as_array()
            .unwrap()
            .iter()
            .any(|variant| variant["type"] == "null")
    );

    assert_eq!(
        schemas["CompatibilityConflict"]["additionalProperties"],
        false
    );
    assert!(
        schemas["AppError"]["required"]
            .as_array()
            .unwrap()
            .contains(&json!("code"))
    );
    for (name, fields) in [
        ("AppReadinessView", &["artifact_generation"][..]),
        ("SavedListItemNoteView", &["note"][..]),
        (
            "EncounterParticipantConditionView",
            &["scope", "extra_action", "immunity_override", "parameters"][..],
        ),
        ("ConditionLifecycleResultView", &["check"][..]),
        (
            "LifecycleCheckResultView",
            &["dc", "modifier", "roll", "table_result"][..],
        ),
    ] {
        let required = schemas[name]["required"].as_array().unwrap();
        for field in fields {
            assert!(
                required.contains(&json!(field)),
                "{name}.{field} must be present"
            );
        }
    }
    // Optional input accepts null or omission. The corresponding response
    // omits a missing level and emits a number when present.
    let input = &schemas["PartyMemberInput"];
    assert!(
        !input["required"]
            .as_array()
            .unwrap()
            .contains(&json!("level"))
    );
    assert!(
        input["properties"]["level"]["type"]
            .as_array()
            .unwrap()
            .contains(&json!("null"))
    );
    for member in [
        json!({"name": "Scout"}),
        json!({"name": "Scout", "level": null}),
    ] {
        let request = serde_json::from_value::<atlas_app_model::CreatePartyRequest>(
            json!({"name": "Party", "members": [member]}),
        )
        .unwrap();
        assert_eq!(request.members[0].level, None);
    }
    let output = &schemas["PartyMemberView"];
    assert!(
        !output["required"]
            .as_array()
            .unwrap()
            .contains(&json!("level"))
    );
    assert_eq!(output["properties"]["level"]["type"], "integer");
    assert_eq!(
        serde_json::to_value(atlas_app_model::PartyMemberView {
            member_key: "scout".into(),
            name: "Scout".into(),
            level: None,
        })
        .unwrap(),
        json!({"member_key": "scout", "name": "Scout"})
    );
    let hp_value = &schemas["EncounterHpValueInput"];
    assert!(
        hp_value["required"]
            .as_array()
            .unwrap()
            .contains(&json!("input"))
    );
    assert!(
        hp_value["properties"]["input"]["type"]
            .as_array()
            .unwrap()
            .contains(&json!("null"))
    );

    let note = &schemas["UpdateSavedListItemNoteRequest"];
    assert!(
        note["required"]
            .as_array()
            .unwrap()
            .contains(&json!("note"))
    );
    assert_eq!(
        note["properties"]["note"]["$ref"],
        "#/components/schemas/FieldEdit_Option_String"
    );
    assert!(note["properties"].get("list_ref").is_none());
    assert!(note["properties"].get("record_key").is_none());
    assert_eq!(note["additionalProperties"], false);
    let nullable_edit = &schemas["FieldEdit_Option_String"];
    assert_eq!(nullable_edit["additionalProperties"], false);
    for property in ["expected", "value"] {
        assert!(
            nullable_edit["required"]
                .as_array()
                .unwrap()
                .contains(&json!(property))
        );
        assert!(
            nullable_edit["properties"][property]["oneOf"]
                .as_array()
                .unwrap()
                .contains(&json!({"type": "null"}))
        );
    }
    let list_update = &schemas["UpdateSavedListRequest"];
    assert!(list_update.get("required").is_none());
    assert!(list_update["properties"].get("list_key").is_none());
    assert_eq!(list_update["additionalProperties"], false);
    assert_eq!(
        list_update["properties"]["description"],
        json!({"$ref": "#/components/schemas/FieldEdit_Option_String"})
    );
    for kind in ["set_current", "set_maximum"] {
        let action = schemas["EncounterHpAction"]["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|variant| variant["properties"]["kind"]["enum"] == json!([kind]))
            .unwrap();
        assert!(
            action["required"]
                .as_array()
                .unwrap()
                .contains(&json!("input"))
        );
        assert!(
            serde_json::from_value::<atlas_app_model::EncounterHpAction>(json!({"kind": kind}))
                .is_err()
        );
        assert!(
            serde_json::from_value::<atlas_app_model::EncounterHpAction>(
                json!({"kind": kind, "input": null})
            )
            .is_ok()
        );
    }
    let delete = &document["paths"]["/api/lists/{list_ref}/items"]["delete"];
    assert_eq!(
        delete["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/RemoveSavedListItemRequest"
    );
    assert_eq!(
        delete["responses"]["409"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/ConflictResponse"
    );
}

#[test]
fn shared_schema_uses_serde_input_and_output_shapes() {
    let document = crate::openapi().unwrap();
    let schemas = &document["components"]["schemas"];
    let input = &schemas["ConditionDurationViewInputSchema"];
    let output = &schemas["ConditionDurationView"];
    let nullable_duration = json!({"amount": 1, "unit": "rounds", "remaining_rounds": null});
    let duration: atlas_app_model::ConditionDurationView =
        serde_json::from_value(nullable_duration).unwrap();
    assert_eq!(
        serde_json::to_value(duration).unwrap(),
        json!({"amount": 1, "unit": "rounds", "timing": null})
    );
    assert!(
        !input["required"]
            .as_array()
            .unwrap()
            .contains(&json!("timing"))
    );
    assert!(
        output["required"]
            .as_array()
            .unwrap()
            .contains(&json!("timing"))
    );
    assert_eq!(
        input["properties"]["remaining_rounds"]["anyOf"][1]["type"],
        "null"
    );
    assert_eq!(output["properties"]["remaining_rounds"]["type"], "integer");

    let contains_ref = |schema: &serde_json::Value, name: &str| {
        schema
            .to_string()
            .contains(&format!("\"#/components/schemas/{name}\""))
    };
    assert!(contains_ref(
        &schemas["AddEncounterParticipantConditionRequest"]["properties"]["duration"],
        "ConditionDurationViewInputSchema"
    ));
    assert!(contains_ref(
        &schemas["EncounterParticipantConditionView"]["properties"]["duration"],
        "ConditionDurationView"
    ));
    assert!(contains_ref(
        &schemas["SearchCriteria"],
        "QueryPredicateInputSchema"
    ));
    assert!(contains_ref(
        &schemas["QueryExpressionInputSchema"],
        "QueryPredicateInputSchema"
    ));
    assert!(contains_ref(&schemas["QueryExpression"], "QueryPredicate"));
}

#[tokio::test]
async fn compatibility_handshake_is_available_without_service_capacity() {
    let fixture = Fixture::without_reference();
    assert_ne!(
        fixture.service.readiness().status,
        atlas_app_model::AppReadinessStatus::Ready
    );
    let app = router_with_state(fixture.state(0));
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
    let fixture = Fixture::without_reference();
    for (method, path) in [
        (Method::GET, "/api/readiness"),
        (Method::POST, "/api/lists"),
    ] {
        let app = router_with_state(fixture.state(0));
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
    let fixture = Fixture::without_reference();
    let app = router_with_state(fixture.state(0));
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

#[tokio::test]
async fn app_errors_preserve_their_identity_and_details_in_http_responses() {
    for (code, expected) in [
        (AppErrorCode::InvalidRequest, StatusCode::BAD_REQUEST),
        (AppErrorCode::RecordResolutionMiss, StatusCode::BAD_REQUEST),
        (
            AppErrorCode::RecordResolutionAmbiguous,
            StatusCode::BAD_REQUEST,
        ),
        (
            AppErrorCode::ArtifactGenerationChanged,
            StatusCode::CONFLICT,
        ),
        (AppErrorCode::SavedListNotFound, StatusCode::NOT_FOUND),
        (AppErrorCode::SavedListAlreadyExists, StatusCode::CONFLICT),
        (
            AppErrorCode::ArtifactNotReady,
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            AppErrorCode::VectorReadinessRequired,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (AppErrorCode::ServiceBusy, StatusCode::SERVICE_UNAVAILABLE),
        (
            AppErrorCode::OperationCancelled,
            StatusCode::REQUEST_TIMEOUT,
        ),
        (AppErrorCode::OperationTimeout, StatusCode::REQUEST_TIMEOUT),
        (
            AppErrorCode::InternalError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (AppErrorCode::QueryFailed, StatusCode::INTERNAL_SERVER_ERROR),
    ] {
        let mut error = atlas_app_model::AppError::new(code, "Request could not complete")
            .with_details(json!({"request_id": "request-42"}));
        error.retryable = Some(false);
        let response = super::error::WebError(error.clone()).into_response();
        assert_eq!(response.status(), expected, "{code:?}");
        assert_eq!(response.headers()["content-type"], "application/json");
        let (_, body) = response_json(response).await;
        let actual: atlas_app_model::AppError = serde_json::from_value(body).unwrap();
        assert_eq!(actual, error, "{code:?}");
    }
}

#[tokio::test]
async fn call_service_preserves_successful_result() {
    let fixture = Fixture::without_reference();
    let value = call_service(fixture.state(64), || Ok::<_, AppServiceError>("ready"))
        .await
        .expect("successful app-service call should pass through");

    assert_eq!(value, "ready");
}

#[tokio::test]
async fn call_service_maps_service_error_to_http_envelope() {
    let fixture = Fixture::without_reference();
    let error = call_service(fixture.state(64), || {
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
    let fixture = Fixture::without_reference();
    let error = call_service(fixture.state(64), || -> Result<(), AppServiceError> {
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

#[tokio::test]
async fn routes_return_service_busy_when_web_backpressure_is_full() {
    let fixture = Fixture::without_reference();
    for path in ["/api/records/actions:aaaaaaaaaaaaaaaa", "/api/readiness"] {
        let app = router_with_state(fixture.state(0));
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
    let fixture = Fixture::new();
    let (status, body) = fixture.json(Method::GET, "/api/readiness", None).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "ready");
    assert_eq!(body["search_profiles"][0]["stored_vectors"], true);
    assert_eq!(body["search_profiles"][0]["assets_ready"], false);
    assert_eq!(body["search_profiles"][0]["usable"], false);
}

#[cfg(feature = "embedded-ui")]
#[tokio::test]
async fn root_serves_embedded_frontend_index() {
    let fixture = Fixture::without_reference();
    let app = fixture.router();
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

#[cfg(feature = "embedded-ui")]
#[tokio::test]
async fn frontend_routes_fall_back_to_index_but_missing_resources_do_not() {
    let fixture = Fixture::without_reference();
    let app = fixture.router();
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

    for path in ["/api/no-such-route", "/assets/missing.js"] {
        let response = app
            .clone()
            .oneshot(
                Request::get(path)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
}

#[cfg(feature = "embedded-ui")]
#[tokio::test]
async fn embedded_frontend_assets_get_long_cache_headers() {
    let fixture = Fixture::without_reference();
    let asset_path = WebAssets::iter()
        .find(|path| path.starts_with("assets/") && path.ends_with(".js"))
        .expect("built frontend should include a JavaScript asset");
    let app = fixture.router();
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
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        body.as_ref(),
        WebAssets::get(&asset_path).unwrap().data.as_ref()
    );
}

#[tokio::test]
async fn stateless_search_route_carries_criteria_and_generation_and_rejects_stale_reads() {
    let fixture = Fixture::new();
    let (status,first)=fixture.json(Method::POST,"/api/search/page",Some(json!({"criteria":{"kind":"list_records","filter":null},"page":{"number":1,"size":2}}))).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["mode"]["kind"], "list_records");
    assert_eq!(first["rows"].as_array().unwrap().len(), 2);
    let generation = first["artifact_generation"].clone();
    let mut request = json!({"criteria":{"kind":"text_search","query":"Flame","filter":null,"mode":"lexical"},"page":{"number":2,"size":1},"expected_artifact_generation":generation});
    let (status, body) = fixture
        .json(Method::POST, "/api/search/page", Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["page"]["number"], 2);
    assert_eq!(
        body["mode"],
        json!({"kind":"text_search","query":"Flame","mode":"lexical"})
    );
    request["expected_artifact_generation"] = json!("previous-generation");
    let (status, body) = fixture
        .json(Method::POST, "/api/search/page", Some(request))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "artifact_generation_changed");
    assert_eq!(
        body["details"],
        json!({"expected_artifact_generation":"previous-generation","actual_artifact_generation":generation})
    );
    assert_eq!(body["recoverable_action"], "refresh_results");
}

#[tokio::test]
async fn search_page_keeps_trait_reference_scope_and_rejects_retired_trait_seed() {
    let fixture = Fixture::new();
    for criteria in [
        json!({"kind":"similar_traits","seed_trait_key":"trait:fire","filter":null}),
        json!({"kind":"trait_references","trait_key":"trait:fire","query":null,"filter":null}),
    ] {
        let (status, body) = fixture
            .json(
                Method::POST,
                "/api/search/page",
                Some(json!({"criteria":criteria,"page":{"number":1,"size":25}})),
            )
            .await;
        if criteria["kind"] == "similar_traits" {
            assert!(status.is_client_error(), "{status}: {body}");
            assert_eq!(body["code"], "invalid_request");
            continue;
        }
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["mode"]["kind"], criteria["kind"]);
        assert_eq!(body["mode"]["trait_key"], "trait:fire");
        assert_eq!(body["mode"]["title"], "Fire");
    }
}

#[tokio::test]
async fn search_page_rejects_malformed_criteria_and_retired_window_routes() {
    let fixture = Fixture::new();
    for criteria in [
        json!({"kind":"text_search","query":"Record","mode":"unknown"}),
        json!({"kind":"list_records","filter":null,"search_content":"core"}),
        json!({"kind":"trait_references","trait_key":"trait:fire","query":null,"filter":null,"search_content":"core"}),
        json!({"kind":"text_search","query":"fire","filter":null,"mode":"lexical","source_sections":"core"}),
    ] {
        let (status, body) = fixture
            .json(
                Method::POST,
                "/api/search/page",
                Some(json!({"criteria":criteria,"page":{"number":1,"size":25}})),
            )
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["code"], "invalid_request");
    }
    for path in ["/api/result-windows", "/api/result-windows/1/page"] {
        let response = fixture
            .router()
            .oneshot(
                Request::post(path)
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
async fn search_match_previews_use_the_structured_wire_contract() {
    let fixture = Fixture::new();
    for (label, preview) in [
        (
            "Lantern Colors",
            json!({"kind":"table","columns":["Color","Meaning"]}),
        ),
        (
            "Lantern Signals",
            json!({"kind":"text","blocks":["One flash means wait.","Two flashes mean go."],"truncated":false}),
        ),
        ("Lantern Warning", Value::Null),
    ] {
        let (status, body) = fixture.json(
            Method::POST,
            "/api/search/page",
            Some(json!({
                "criteria":{"kind":"text_search","query":format!("\"{label}\""),"mode":"lexical"},
                "page":{"number":1,"size":10},
            }))
        ).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let row = body["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["record"]["record_key"] == ACTION)
            .unwrap();
        let matched = &row["matching_sections"][0];
        assert_eq!(matched["label"], label);
        assert_eq!(matched.get("preview"), Some(&preview), "{body}");
        assert!(matched.get("snippet").is_none());
        let round_trip: atlas_app_model::SearchMatchSectionView =
            serde_json::from_value(matched.clone()).unwrap();
        assert_eq!(round_trip.navigation.record_key, ACTION);
        assert!(round_trip.navigation.passage.is_some());
    }
}

#[tokio::test]
async fn description_filter_keeps_own_prose_through_the_search_route() {
    let fixture = Fixture::new();
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/search/page",
            Some(json!({
                "criteria": {
                    "kind": "list_records",
                    "filter": {"kind":"compare","field":"has_description","op":"eq","value":true}
                },
                "page": {"number":1,"size":25}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["page"]["total"], 2);
    assert_eq!(body["rows"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn record_and_filter_routes_use_real_router_wiring() {
    let fixture = Fixture::new();
    let (status, body) = fixture
        .json(Method::GET, &format!("/api/records/{ACTION}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["record"]["record_key"], ACTION);
    assert_eq!(body["presentation"]["identity"]["title"], "Test Action 1");
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/filters/editor",
            Some(json!({"selected_field_ids":[]})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["groups"].as_array().unwrap().iter().any(|group| {
        group["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["definition"]["id"] == "kind")
    }));
    let context = json!({"kind":"filtered","filter":null,"text":null});
    let(status,body)=fixture.json(Method::POST,"/api/filters/values",Some(json!({"kind":"contextual","context":context,"field_id":"source.pack.id","clause_id":null,"text":null,"offset":0,"limit":100}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["outcome"]["value"]["field"], "source.pack.id");
    assert!(
        body["outcome"]["value"]["options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["value"] == "actions")
    );
    let(status,body)=fixture.json(Method::POST,"/api/filters/counts",Some(json!({"context":context,"field_id":"source.pack.id","clause_id":null,"values":["actions"]}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["counts"]["value"]["options"],
        json!([{"value":"actions","distinct_roots":2,"selected":false}])
    );
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/filters/validate",
            Some(json!({"kind":"compare","field":"kind","op":"eq","value":"rule"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["predicate"]["field"], "kind");
    let(status,body)=fixture.json(Method::POST,"/api/records/detail",Some(json!({"record_key":ACTION,"embedded_path":[],"fields":[],"passage":null,"source_fingerprint":null}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["record"]["record_key"], ACTION);
}

#[tokio::test]
async fn saved_list_routes_use_real_router_wiring() {
    let fixture = Fixture::new();
    let (status, body) = fixture.json(Method::GET, "/api/lists", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["lists"][0]["list_key"], fixture.list_key);
    assert_eq!(body["lists"][0]["slug"], "research");
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/lists",
            Some(json!({"slug":"encounters","name":"Encounters","description":"Session prep"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list"]["slug"], "encounters");
    assert_eq!(body["list"]["tags"], json!([]));
    let path = format!("/api/lists/{}", fixture.list_key);
    let (status, body) = fixture.json(Method::GET, &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["items"][0]["record_key"], ACTION);
    assert_eq!(body["items"][0]["status"], "active");
    let (status, body) = fixture
        .json(
            Method::POST,
            &format!("{path}/filter"),
            Some(
                json!({"filter":{"kind":"in","clause_id":"kind","field":"kind","values":["rule"]}}),
            ),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list"]["list_key"], fixture.list_key);
    assert_eq!(body["items"][0]["record_key"], ACTION);
    let(status,body)=fixture.json(Method::PATCH,&path,Some(json!({"slug":{"expected":"research","value":"renamed-research"},"name":{"expected":"Research","value":"Renamed Research"},"description":{"expected":null,"value":"Updated prep"}}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list"]["list_key"], fixture.list_key);
    assert_eq!(body["list"]["slug"], "renamed-research");
    assert_eq!(body["list"]["name"], "Renamed Research");
    assert_eq!(body["list"]["description"], "Updated prep");
    let (status, body) = fixture
        .json(
            Method::POST,
            &format!("{path}/items"),
            Some(json!({"record_ref":SECOND_ACTION,"note":null})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list_key"], fixture.list_key);
    assert_eq!(body["slug"], "renamed-research");
    assert_eq!(body["record_key"], SECOND_ACTION);
    assert_eq!(body["outcome"], "added");
    let (status, body) = fixture
        .json(
            Method::DELETE,
            &format!("{path}/items"),
            Some(json!({"record_ref":ACTION})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list_key"], fixture.list_key);
    assert_eq!(body["record_key"], ACTION);
    assert_eq!(body["outcome"], "removed");
    let (status, body) = fixture.json(Method::DELETE, &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list_key"], fixture.list_key);
    assert_eq!(body["deleted"], true);
    let (status, body) = fixture.json(Method::GET, "/api/lists/missing", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "saved_list_not_found");
}

#[tokio::test]
async fn encounter_routes_use_real_router_wiring() {
    let fixture = Fixture::new();
    let (status, body) = fixture.json(Method::GET, "/api/encounters", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["encounters"][0]["slug"], "ambush");
    let (status, body) = fixture
        .json(Method::GET, "/api/encounters/condition-definitions", None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body["conditions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["condition_ref"] == "conditionitems:TBSHQspnbcqxsmjL")
    );
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters",
            Some(json!({"name":"Boss Fight","description":"Climax","note":"Bring maps"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["encounter"]["slug"], "boss-fight");
    let path = format!("/api/encounters/{}", fixture.encounter_key);
    let (status, body) = fixture.json(Method::GET, &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["encounter"]["encounter_key"], fixture.encounter_key);
    assert_eq!(
        body["participants"][0]["participant_key"],
        fixture.participant_key
    );
    let receipt = json!({"encounter_key":fixture.encounter_key});
    let(status,body)=fixture.json(Method::PATCH,&path,Some(json!({"slug":{"expected":"ambush","value":"renamed-ambush"},"name":{"expected":"Ambush","value":"Renamed Ambush"},"description":{"expected":null,"value":"Updated"},"note":{"expected":null,"value":"Updated note"},"status":{"expected":"draft","value":"complete"}}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let path = format!("/api/encounters/{}", fixture.encounter_key);
    let (status, body) = fixture
        .json(
            Method::POST,
            &format!("{path}/participants/record"),
            Some(json!({"record_ref":ACTOR,"quantity":2,"initiative":18})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let (status, body) = fixture
        .json(
            Method::POST,
            &format!("{path}/participants/manual"),
            Some(json!({"display_name":"Valeros","max_hp":30,"current_hp":25,"initiative":20})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let participant_path = format!("{path}/participants/{}", fixture.participant_key);
    let detail = fixture.service.encounter(&fixture.encounter_key).unwrap();
    let request = participant_edit(&detail.participants[0], "Renamed Kyra");
    let (status, body) = fixture
        .json(Method::PATCH, &participant_path, Some(request))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let other = &detail.participants[1].participant_key;
    let(status,body)=fixture.json(Method::POST,&format!("{path}/participants/reorder"),Some(json!({"participant_key":other,"target_participant_key":fixture.participant_key,"placement":"before"}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let (status, body) = fixture
        .json(
            Method::POST,
            &format!("{path}/turn"),
            Some(json!({"participant_key":fixture.participant_key})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let revision = fixture
        .service
        .encounter(&fixture.encounter_key)
        .unwrap()
        .revision;
    let(status,body)=fixture.json(Method::POST,&format!("{participant_path}/conditions"),Some(json!({"request_id":"apply-frightened","expected_encounter_revision":revision,"name":"Frightened","value":1,"duration":{"amount":2,"unit":"rounds","timing":{"participant_key":fixture.participant_key,"event":"end"}}}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let detail = fixture.service.encounter(&fixture.encounter_key).unwrap();
    let condition = &detail
        .participants
        .iter()
        .find(|p| p.participant_key == fixture.participant_key)
        .unwrap()
        .conditions[0];
    let condition_path = format!("{participant_path}/conditions/{}", condition.condition_id);
    let update = json!({"request_id":"edit-frightened","expected_encounter_revision":detail.revision,"name":"Frightened","value":2,"duration":{"amount":1,"unit":"rounds","timing":{"participant_key":fixture.participant_key,"event":"end"}}});
    for condition_id in [condition.condition_id, condition.condition_id + 1] {
        let mut request = update.clone();
        request["condition_id"] = json!(condition_id);
        let (status, body) = fixture
            .json(Method::PATCH, &condition_path, Some(request))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["code"], "invalid_request");
    }
    assert_eq!(
        fixture.service.encounter(&fixture.encounter_key).unwrap(),
        detail
    );
    let (status, body) = fixture
        .json(Method::PATCH, &condition_path, Some(update.clone()))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let revision = fixture
        .service
        .encounter(&fixture.encounter_key)
        .unwrap()
        .revision;
    let mut update = update;
    update["expected_encounter_revision"] = json!(revision);
    update["request_id"] = json!("wrong-owner");
    let wrong = format!(
        "{path}/participants/wrong_participant/conditions/{}",
        condition.condition_id
    );
    let (status, body) = fixture.json(Method::PATCH, &wrong, Some(update)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_participant_not_found");
    let remove = json!({"request_id":"remove-frightened","expected_encounter_revision":revision,"intent":"remove_effect"});
    let (status, body) = fixture
        .json(Method::DELETE, &wrong, Some(remove.clone()))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_participant_not_found");
    let (status, body) = fixture
        .json(Method::DELETE, &condition_path, Some(remove))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let (status, body) = fixture.json(Method::DELETE, &participant_path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, receipt);
    let (status, body) = fixture.json(Method::DELETE, &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["encounter_key"], fixture.encounter_key);
    assert_eq!(body["deleted"], true);
    let (status, body) = fixture
        .json(Method::GET, "/api/encounters/missing", None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "encounter_not_found");
}

fn participant_edit(participant: &atlas_app_model::EncounterParticipantView, name: &str) -> Value {
    json!({"expected_revision":participant.revision,"display_name":name})
}

#[tokio::test]
async fn saved_list_reorder_route_uses_path_identity_and_strict_placement() {
    let fixture = Fixture::new();
    let path = format!("/api/lists/{}/items", fixture.list_key);
    let (status, body) = fixture
        .json(
            Method::POST,
            &path,
            Some(json!({"record_ref":SECOND_ACTION})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let request =
        json!({"record_ref":ACTION,"target_record_ref":SECOND_ACTION,"placement":"after"});
    let path = format!("{path}/reorder");
    let (status, body) = fixture
        .json(Method::POST, &path, Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list_key"], fixture.list_key);
    assert_eq!(body["record_key"], ACTION);
    assert_eq!(body["outcome"], "reordered");
    let (_, detail) = fixture
        .json(
            Method::GET,
            &format!("/api/lists/{}", fixture.list_key),
            None,
        )
        .await;
    assert_eq!(detail["items"][0]["record_key"], SECOND_ACTION);
    for invalid in [
        {
            let mut v = request.clone();
            v["placement"] = json!("above");
            v
        },
        {
            let mut v = request.clone();
            v.as_object_mut().unwrap().remove("target_record_ref");
            v
        },
        {
            let mut v = request;
            v["unexpected"] = json!(true);
            v
        },
    ] {
        let (status, body) = fixture.json(Method::POST, &path, Some(invalid)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["code"], "invalid_request");
    }
}

#[tokio::test]
async fn malformed_json_route_body_returns_app_error_envelope() {
    let fixture = Fixture::without_reference();
    let response = fixture
        .router()
        .oneshot(
            Request::post("/api/filters/editor")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{"))
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, body) = response_json(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["code"], "invalid_request");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("invalid JSON request body")
    );
}

#[tokio::test]
async fn participant_revision_is_required_and_stale_edits_return_409() {
    let fixture = Fixture::new();
    let detail = fixture.service.encounter("ambush").unwrap();
    let participant = &detail.participants[0];
    let path = format!(
        "/api/encounters/ambush/participants/{}",
        participant.participant_key
    );
    let mut request = participant_edit(participant, "Edited");
    request.as_object_mut().unwrap().remove("expected_revision");
    let (status, body) = fixture
        .json(Method::PATCH, &path, Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["code"], "invalid_request");
    request["expected_revision"] = json!(participant.revision - 1);
    let (status, body) = fixture
        .json(Method::PATCH, &path, Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_participant_conflict");
    request["expected_revision"] = json!(participant.revision);
    for body in [
        json!({"expected_revision":participant.revision,"participant_key":participant.participant_key}),
        json!({"expected_revision":participant.revision,"max_hp":99}),
        json!({"expected_revision":participant.revision,"display_name":null}),
        json!([
            participant.revision,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null
        ]),
    ] {
        let (status, body) = fixture.json(Method::PATCH, &path, Some(body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["code"], "invalid_request");
    }
    assert_eq!(fixture.service.encounter("ambush").unwrap(), detail);
    request["initiative"] = json!(19);
    request["note"] = json!("Track this");
    let (status, body) = fixture.json(Method::PATCH, &path, Some(request)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({"encounter_key":fixture.encounter_key}));
    let edited = fixture
        .service
        .encounter("ambush")
        .unwrap()
        .participants
        .remove(0);
    assert_eq!(edited.display_name, "Edited");
    assert_eq!(edited.initiative, Some(19));
    assert_eq!(edited.note.as_deref(), Some("Track this"));
    assert_eq!(edited.max_hp, participant.max_hp);
    assert_eq!(edited.current_hp, participant.current_hp);
    assert_eq!(edited.temporary_hp, participant.temporary_hp);
    assert_eq!(edited.variant_origin, participant.variant_origin);
    assert_eq!(edited.hp_origin, participant.hp_origin);
    let (status, body) = fixture
        .json(
            Method::PATCH,
            &path,
            Some(json!({"expected_revision":edited.revision,"initiative":null,"note":null})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let cleared = fixture
        .service
        .encounter("ambush")
        .unwrap()
        .participants
        .remove(0);
    assert_eq!(cleared.initiative, None);
    assert_eq!(cleared.note, None);
    assert_eq!(cleared.display_name, "Edited");
}

#[tokio::test]
async fn turn_review_and_advance_use_explicit_revision_path_and_typed_failures() {
    let fixture = Fixture::new();
    let (status, review) = fixture
        .json(Method::GET, "/api/encounters/ambush/turn-review", None)
        .await;
    assert_eq!(status, StatusCode::OK, "{review}");
    assert_eq!(review["next_participant_key"], fixture.participant_key);
    let request = json!({"request_id":"next","expected_revision":review["expected_revision"]});
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/advance",
            Some(request.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({"encounter_key":fixture.encounter_key}));
    let (status, retry) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/advance",
            Some(request.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{retry}");
    assert_eq!(retry, body);
    let mut stale = request;
    stale["request_id"] = json!("next-stale");
    let (status, body) = fixture
        .json(Method::POST, "/api/encounters/ambush/advance", Some(stale))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_turn_conflict");
    for body in [
        json!({}),
        json!({"request_id":"bad","expected_revision":null}),
        json!({"request_id":"bad","expected_revision":1.5}),
        json!({"request_id":"bad","expected_revision":7,"encounter_ref":"spoof"}),
    ] {
        let (status, body) = fixture
            .json(Method::POST, "/api/encounters/ambush/advance", Some(body))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["code"], "invalid_request");
    }
    let (status, body) = fixture
        .json(Method::POST, "/api/encounters/ambush/turn", Some(json!({})))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}

#[tokio::test]
async fn turn_resource_route_preserves_intent_and_uses_participant_path_identity() {
    let fixture = Fixture::new();
    let review = fixture.service.encounter_turn_review("ambush").unwrap();
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/advance",
            Some(json!({"request_id":"start","expected_revision":review.expected_revision})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let detail = fixture.service.encounter("ambush").unwrap();
    let turn_key = detail.participants[0]
        .turn_resources
        .as_ref()
        .unwrap()
        .turn_key
        .clone();
    let path = format!(
        "/api/encounters/ambush/participants/{}/turn-resources",
        fixture.participant_key
    );
    let mut request = json!({"request_id":"manual-reaction","turn_key":turn_key,"expected_encounter_revision":detail.revision,"action":{"kind":"spend","resource":"reaction"}});
    let (status, body) = fixture
        .json(Method::POST, &path, Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({"encounter_key":fixture.encounter_key}));
    request["request_id"] = json!("stale-reaction");
    let (status, body) = fixture
        .json(Method::POST, &path, Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_turn_conflict");
    request["action"] = json!({"kind":"spend","resource":"anything"});
    let (status, body) = fixture.json(Method::POST, &path, Some(request)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}

#[tokio::test]
async fn hp_actions_require_explicit_contract_and_use_path_identity_and_receipt() {
    let fixture = Fixture::new();
    let detail = fixture.service.encounter("ambush").unwrap();
    let path = format!(
        "/api/encounters/ambush/participants/{}/hp",
        fixture.participant_key
    );
    let mut request = json!({"request_id":"hp-action","action":{"kind":"damage","amount":"10+2"}});
    let (status, body) = fixture
        .json(Method::POST, &path, Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["code"], "invalid_request");
    request["expected_revision"] = json!(detail.participants[0].revision - 1);
    let (status, body) = fixture
        .json(Method::POST, &path, Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "encounter_participant_conflict");
    request["expected_revision"] = json!(detail.participants[0].revision);
    let (status, body) = fixture
        .json(Method::POST, &path, Some(request.clone()))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({"encounter_key":fixture.encounter_key}));
    assert_eq!(
        fixture.service.encounter("ambush").unwrap().participants[0].current_hp,
        Some(8)
    );
    request["action"] = json!({"kind":"damage","amount":12});
    let (status, body) = fixture.json(Method::POST, &path, Some(request)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["code"], "invalid_request");
}

#[tokio::test]
async fn persistent_preparation_routes_round_trip_a_browser_draft() {
    let fixture = Fixture::new();
    let participant = &fixture.participant_key;
    let revision = fixture.service.encounter("ambush").unwrap().revision;
    let (status, body) = fixture
        .json(
            Method::POST,
            &format!("/api/encounters/ambush/participants/{participant}/conditions"),
            Some(json!({
                "request_id": "persistent-condition",
                "expected_encounter_revision": revision,
                "condition_ref": "conditionitems:lDVqvLKA6eF3Df60",
                "parameters": {
                    "kind": "persistent_damage", "formula": "1d6", "damage_type": "fire",
                    "multiplier": "normal", "recovery": {"kind": "standard"}
                }
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let before = fixture.service.encounter("ambush").unwrap();
    let episode = &before.participants[0].persistent_damage_groups[0].episode_id;
    let group = json!({"episode_id": episode, "damage": {"kind": "table_result", "amount": 4}});
    let (status, preparation) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/persistent-preparation",
            Some(json!({
                "request_id": "review-recovery",
                "expected_encounter_revision": before.revision,
                "participant_key": participant,
                "groups": [group.clone()],
                "recovery": {"episode_id": episode, "dc_override": 0}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{preparation}");
    assert_eq!(
        preparation["recovery_draft"]["context"]["participant_key"],
        *participant
    );
    assert_eq!(preparation["recovery_draft"]["dc"], 0);
    assert!(preparation["recovery_draft"]["roll"].is_null());
    assert_eq!(
        fixture.service.encounter("ambush").unwrap().revision,
        before.revision
    );
    let mut apply = json!({
        "request_id": "apply-recovery-draft",
        "expected_encounter_revision": before.revision,
        "participant_key": participant,
        "groups": [group],
        "recoveries": [{"kind": "prepared", "episode_id": episode, "request_id": "review-recovery"}],
        "context": {}
    });
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/persistent-damage",
            Some(apply.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    apply["recoveries"][0] = json!({
        "kind": "prepared", "episode_id": episode, "draft": preparation["recovery_draft"]
    });
    let (status, committed) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/persistent-damage",
            Some(apply.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{committed}");
    assert_eq!(committed["total"], 4);
    let (status, replay) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/persistent-damage",
            Some(apply),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{replay}");
    assert_eq!(replay, committed);
}

#[tokio::test]
async fn dice_routes_forward_typed_requests_and_conflicts() {
    let fixture = Fixture::new();
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/rolls/flat-check",
            Some(json!({"dc":11})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["dc"], 11);
    assert_eq!(body["outcome"]["formula"], "1d20");
    assert_eq!(
        body["success"].as_bool().unwrap(),
        body["outcome"]["total"].as_i64().unwrap() >= 11
    );
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/rolls/flat-check",
            Some(json!({"dc":"bad"})),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, options) = fixture
        .json(Method::GET, "/api/encounters/ambush/initiative", None)
        .await;
    assert_eq!(status, StatusCode::OK, "{options}");
    assert_eq!(options["encounter_key"], fixture.encounter_key);
    let detail = fixture.service.encounter("ambush").unwrap();
    let(status,body)=fixture.json(Method::POST,"/api/encounters/ambush/initiative",Some(json!({"request_id":"initiative-id","artifact_generation":null,"expected_revision":detail.revision,"participants":[{"participant_key":fixture.participant_key,"expected_revision":detail.participants[0].revision,"statistic":"manual","modifier_override":2,"mode":"normal"}]}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["request_id"], "initiative-id");
    let selection = json!({"address":{"record_key":TABLE,"embedded_path":[],"source_fingerprint":null},"fields":[],"passage":null});
    let generation = fixture.service.readiness().artifact_generation.unwrap();
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/rolls/table",
            Some(json!({"selection":selection,"artifact_generation":generation})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!((1..=2).contains(&body["outcome"]["total"].as_i64().unwrap()));
    assert!(
        (1..=2).contains(
            &body["outcome"]["expression"]["dice"][0]["value"]
                .as_i64()
                .unwrap()
        )
    );
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/rolls/table",
            Some(json!({"selection":selection,"artifact_generation":"old"})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "artifact_generation_changed");
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/participants/record",
            Some(json!({"record_ref":ACTOR,"quantity":1})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let detail = fixture.service.encounter("ambush").unwrap();
    let actor = detail
        .participants
        .iter()
        .find(|p| p.record_key.as_deref() == Some(ACTOR))
        .unwrap();
    let reference = json!({"artifact_generation":generation,"selection":{"address":{"record_key":ACTOR,"embedded_path":[{"collection":"/items","identity":{"SnapshotLocal":{"index":0}}}],"source_fingerprint":fixture.source_fingerprint},"fields":[],"passage":null}});
    let (status, options) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/strike-options",
            Some(json!({"participant_key":actor.participant_key,"reference":reference})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{options}");
    assert_eq!(options["title"], "Claw");
    let mut strike = json!({"participant_key":actor.participant_key,"reference":reference,"request_id":"strike-id","expected_revision":detail.revision,"expected_participant_revision":actor.revision-1,"action":{"kind":"attack","attack_number":2,"mode":"fortune","modifier_override":null}});
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/strike-rolls",
            Some(strike.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["code"], "encounter_participant_conflict");
    strike["expected_participant_revision"] = json!(actor.revision);
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/strike-rolls",
            Some(strike.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["request_id"], "strike-id");
    assert_eq!(body["participant_key"], actor.participant_key);
    assert_eq!(body["result"]["kind"], "attack");
    assert_eq!(body["result"]["attack_modifier"], 3);
    assert_eq!(body["result"]["attack"]["formula"], "2d20kh1+3");
    assert!(body["result"].get("damage").is_none());

    let mut mixed = strike.clone();
    mixed["action"] = json!({"kind":"damage","attack_number":2,"mode":"fortune"});
    let mut old_envelope = strike.clone();
    old_envelope["roll_kind"] = json!("attack");
    let mut mixed_attack = strike.clone();
    mixed_attack["action"]["damage"] = json!([]);
    let mut incomplete_attack = strike.clone();
    incomplete_attack["action"] = json!({"kind":"attack","mode":"normal"});
    for request in [mixed, old_envelope, mixed_attack, incomplete_attack] {
        let (status, body) = fixture
            .json(
                Method::POST,
                "/api/encounters/ambush/strike-rolls",
                Some(request),
            )
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["code"], "invalid_request");
    }

    let mut bad_number = strike.clone();
    bad_number["action"]["attack_number"] = json!(4);
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/missing/strike-rolls",
            Some(bad_number),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["message"], "Choose attack 1, 2 or 3");

    let mut damage = strike;
    damage["action"] = json!({"kind":"damage"});
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/strike-rolls",
            Some(damage),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["result"]["kind"], "damage");
    assert!(body["result"].get("attack").is_none());
    assert_eq!(body["result"]["damage"][0]["formula"], "1d6+2");
    assert_eq!(
        body["result"]["damage_totals"][0]["damage_type"],
        "slashing"
    );
    assert!(body["result"]["persistent_total"].is_null());
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters/ambush/strike-rolls",
            Some(json!({"request_id":"incomplete"})),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}

#[tokio::test]
async fn filter_discovery_transport_keeps_explicit_vocabulary_and_optional_unavailability() {
    let fixture = Fixture::new();
    let(status,body)=fixture.json(Method::POST,"/api/filters/values",Some(json!({"kind":"vocabulary","scope":{"kind":"saved_list","list_ref":"research"},"field_id":"traits","text":"fire","limit":20,"selected_values":["retained"]}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["kind"], "vocabulary");
    assert_eq!(body["scope"]["list_ref"], "research");
    assert_eq!(body["values"]["selected_values"][0], "retained");
    assert!(body["values"].get("count_basis").is_none());
    let(status,body)=fixture.json(Method::POST,"/api/filters/counts",Some(json!({"context":{"kind":"filtered","filter":{"kind":"not","predicate":{"kind":"all_of","children":[{"kind":"set_match","clause_id":"negative-condition","field":"traits","op":"includes_any","values":["fire"]}]}},"text":null},"field_id":"traits","clause_id":"negative-condition","values":["fire"]}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["counts"]["status"], "unavailable");
    let(status,_)=fixture.json(Method::POST,"/api/filters/values",Some(json!({"kind":"vocabulary","scope":{"kind":"artifact"},"field_id":"traits","limit":20,"context":{"kind":"filtered","filter":null,"text":"dragon"}}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}

#[tokio::test]
async fn related_request_transports_complete_selection_and_generation() {
    let fixture = Fixture::new();
    let (_, detail) = fixture
        .json(
            Method::POST,
            "/api/records/detail",
            Some(json!({"record_key":ACTOR})),
        )
        .await;
    let selection = json!({"record_key":ACTOR,"embedded_path":[{"collection":"/items","identity":{"SnapshotLocal":{"index":0}}}],"field":"/system/description/value","passage":null,"source_fingerprint":fixture.source_fingerprint});
    let generation = detail["artifact_generation"].clone();
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/related",
            Some(json!({"selection":selection,"artifact_generation":generation})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["selection"], selection);
    assert_eq!(body["artifact_generation"], generation);
    assert_eq!(body["references"]["kind"], "empty");
    for fingerprint in [Value::Null, json!("stale-source")] {
        let mut stale = selection.clone();
        stale["source_fingerprint"] = fingerprint;
        let (status, body) = fixture
            .json(
                Method::POST,
                "/api/related",
                Some(json!({"selection":stale,"artifact_generation":generation})),
            )
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["code"], "invalid_request");
    }
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/related",
            Some(json!({"selection":selection})),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}

#[tokio::test]
async fn similar_page_transport_carries_exact_seed_and_body_free_summary() {
    let fixture = Fixture::new();
    let(status,body)=fixture.json(Method::POST,"/api/search/page",Some(json!({"criteria":{"kind":"similar_records","seed_record_key":ACTION,"filter":null},"page":{"number":2,"size":1}}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["mode"]["kind"], "similar_records");
    assert_eq!(body["mode"]["seed"]["record_key"], ACTION);
    assert!(body["mode"]["seed"].get("presentation").is_none());
    assert_eq!(body["page"]["number"], 2);
}

#[tokio::test]
async fn list_note_routes_take_identity_from_path_and_preserve_explicit_clear() {
    let fixture = Fixture::new();
    let path = format!("/api/lists/{}/items/{ACTION}/note", fixture.list_key);
    let (status, body) = fixture.json(Method::GET, &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["note"], "fixture note");
    let (status, body) = fixture
        .json(
            Method::PATCH,
            &path,
            Some(json!({"note":{"expected":"fixture note","value":"Purpose"}})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list_key"], fixture.list_key);
    assert_eq!(body["record_key"], ACTION);
    assert_eq!(body["note"], "Purpose");
    let (status, body) = fixture
        .json(
            Method::PATCH,
            &path,
            Some(json!({"note":{"expected":"fixture note","value":"Stale"}})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["code"], "saved_edit_conflict");
    assert_eq!(body["details"]["fields"], json!(["note"]));
    let (status, body) = fixture
        .json(
            Method::PATCH,
            &path,
            Some(json!({"note":{"expected":"Purpose","value":null}})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["note"].is_null());
    let (_, body) = fixture.json(Method::GET, &path, None).await;
    assert!(body["note"].is_null());
    let (status, _) = fixture
        .json(
            Method::DELETE,
            &format!("/api/lists/{}/items", fixture.list_key),
            Some(json!({"record_ref":ACTION})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    for (method, request) in [
        (Method::GET, None),
        (
            Method::PATCH,
            Some(json!({"note":{"expected":null,"value":"Removed"}})),
        ),
    ] {
        let (status, body) = fixture.json(method, &path, request).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
        assert_eq!(body["code"], "saved_list_item_not_found");
    }
}

#[tokio::test]
async fn saved_edit_routes_reject_blind_and_malformed_updates_without_a_compatibility_header() {
    let fixture = Fixture::new();
    for (path, request) in [
        (
            format!("/api/lists/{}", fixture.list_key),
            json!({"name":"Overwrite","slug":"research","tags":[]}),
        ),
        (
            format!("/api/encounters/{}", fixture.encounter_key),
            json!({"name":"Overwrite","slug":"ambush","status":"draft"}),
        ),
        (
            format!("/api/lists/{}/items/{ACTION}/note", fixture.list_key),
            json!({"note":null}),
        ),
        (
            format!("/api/lists/{}/items/{ACTION}/note", fixture.list_key),
            json!({"note":["fixture note",null]}),
        ),
        (
            format!("/api/lists/{}/items/{ACTION}/note", fixture.list_key),
            json!([{"expected":"fixture note","value":null}]),
        ),
    ] {
        let (status, body) = fixture.json(Method::PATCH, &path, Some(request)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}: {body}");
        assert_eq!(body["code"], "invalid_request");
    }
}

#[tokio::test]
async fn routes_reject_retired_body_targets_before_writing() {
    let fixture = Fixture::new();
    let list = format!("/api/lists/{}", fixture.list_key);
    let encounter = format!("/api/encounters/{}", fixture.encounter_key);
    let participant = format!("{encounter}/participants/{}", fixture.participant_key);
    let detail = fixture.service.encounter(&fixture.encounter_key).unwrap();

    for (method, path, key, matching, valid) in [
        (
            Method::PATCH,
            list.clone(),
            "list_key",
            &fixture.list_key,
            json!({"name":{"expected":"Research","value":"Changed"}}),
        ),
        (
            Method::PATCH,
            format!("{list}/items/{ACTION}/note"),
            "list_ref",
            &fixture.list_key,
            json!({"note":{"expected":"fixture note","value":null}}),
        ),
        (
            Method::PATCH,
            encounter,
            "encounter_key",
            &fixture.encounter_key,
            json!({"name":{"expected":"Ambush","value":"Changed"}}),
        ),
        (
            Method::PATCH,
            participant,
            "participant_key",
            &fixture.participant_key,
            participant_edit(&detail.participants[0], "Changed"),
        ),
    ] {
        for value in [matching.as_str(), "different-target"] {
            let mut request = valid.clone();
            request[key] = json!(value);
            let (status, body) = fixture.json(method.clone(), &path, Some(request)).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{path}: {body}");
            assert_eq!(body["code"], "invalid_request");
        }
    }

    let path = format!("{list}/items/{ACTION}/note");
    for value in [ACTION, SECOND_ACTION] {
        let request = json!({"record_key":value,"note":{"expected":"fixture note","value":null}});
        let (status, body) = fixture.json(Method::PATCH, &path, Some(request)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }

    let (_, list) = fixture.json(Method::GET, &list, None).await;
    assert_eq!(list["list"]["name"], "Research");
    let (_, note) = fixture.json(Method::GET, &path, None).await;
    assert_eq!(note["note"], "fixture note");
    assert_eq!(
        fixture.service.encounter(&fixture.encounter_key).unwrap(),
        detail
    );
}

#[tokio::test]
async fn list_settings_patches_merge_disjoint_fields_and_reject_stale_batches() {
    let fixture = Fixture::new();
    let path = format!("/api/lists/{}", fixture.list_key);
    for patch in [
        json!({"name":{"expected":"Research","value":"New name"}}),
        json!({"description":{"expected":null,"value":"New description"}}),
    ] {
        let (status, body) = fixture.json(Method::PATCH, &path, Some(patch)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (status, body) = fixture
        .json(
            Method::PATCH,
            &path,
            Some(json!({

                "name":{"expected":"Research","value":"Stale name"},
                "description":{"expected":"New description","value":null}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["details"]["fields"], json!(["name"]));
    let (status, body) = fixture.json(Method::GET, &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list"]["name"], "New name");
    assert_eq!(body["list"]["description"], "New description");
}

#[tokio::test]
async fn similarity_inventory_route_preserves_typed_seed_profile_and_generation() {
    let fixture = Fixture::new();
    let generation = fixture.service.readiness().artifact_generation.unwrap();
    let seed = json!({"kind":"record","record_key":ACTION});
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/search/similarity/parts",
            Some(json!({"seed":seed,"search_profile":"bge-base","expected_artifact_generation":generation})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["search_profile"], "bge-base");
    assert_eq!(body["seed"], seed);
    assert_eq!(body["artifact_generation"], generation);
    assert_eq!(body["supports_corresponding_parts"], true);
    let (status, body)=fixture.json(Method::POST,"/api/search/similarity/parts",Some(json!({"seed":{"kind":"record","record_key":ACTION},"search_profile":"embeddinggemma","expected_artifact_generation":generation}))).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{body}");
    assert_eq!(body["code"], "vector_readiness_required");
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/search/similarity/parts",
            Some(json!({"seed":seed,"expected_artifact_generation":"previous-generation"})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "artifact_generation_changed");
}

#[tokio::test]
async fn similarity_inventory_route_rejects_invalid_seeds() {
    let fixture = Fixture::new();
    for seed in [
        json!({"kind":"record","record_key":ACTION,"text":"invented selection"}),
        json!({"kind":"trait","trait_key":"trait:fire"}),
    ] {
        let (status, body) = fixture
            .json(
                Method::POST,
                "/api/search/similarity/parts",
                Some(json!({"seed":seed})),
            )
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }
}

#[tokio::test]
async fn content_roll_reference_roll_transport_checks_the_request_shape() {
    let fixture = Fixture::new();
    let (status, detail) = fixture
        .json(Method::GET, &format!("/api/records/{ACTION}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    let offer = detail["presentation"]["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|field| field["body"]["controls"].as_array())
        .flatten()
        .find_map(|control| {
            control["control"]
                .get("roll")
                .filter(|roll| !roll.is_null())
        })
        .unwrap();
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/rolls/content",
            Some(json!({"offer": offer})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["components"].is_array());
    let mut tampered = offer.clone();
    tampered["authored_formula"] = json!("100d100");
    let mut stale = offer.clone();
    stale["source_fingerprint"] = json!("old");
    for rejected in [
        offer.clone(),
        json!({"offer": tampered}),
        json!({"offer": stale}),
    ] {
        let (status, body) = fixture
            .json(Method::POST, "/api/rolls/content", Some(rejected))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }
}

#[tokio::test]
async fn embedded_reading_accepts_a_root_address_and_rejects_read_options_on_it() {
    let fixture = Fixture::new();
    let generation = fixture.service.readiness().artifact_generation.unwrap();
    let mut request = json!({
        "parent": {"record_key": ACTOR},
        "artifact_generation": generation,
        "selection": {"kind": "actor_content"},
    });
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/records/embedded-reading",
            Some(request.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["parent"]["record_key"], ACTOR);
    assert_eq!(body["parent"]["embedded_path"], json!([]));
    request["parent"]["field"] = json!("/system/details/publicNotes");
    let (status, body) = fixture
        .json(Method::POST, "/api/records/embedded-reading", Some(request))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}

async fn response_json(response: Response) -> (StatusCode, Value) {
    let (parts, body) = response.into_parts();
    let body = to_bytes(body, usize::MAX).await.unwrap();
    (parts.status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn source_free_saved_state_works_without_reference_readiness() {
    let fixture = Fixture::without_reference();
    let workspace = atlas_runtime::initialize_workspace(fixture.service.workspace_path()).unwrap();
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write
        .connection()
        .execute_batch(
            "UPDATE principals SET id = 7 WHERE principal_key = 'local';
         INSERT INTO principals VALUES (1, 'other', 'Other');
         INSERT INTO saved_lists(owner_id,list_key,slug,name,created_at,updated_at)
         VALUES (1,'foreign-list','offline','Other list','now','now');
         INSERT INTO encounters(owner_id,encounter_key,slug,name,status,created_at,updated_at)
         VALUES (1,'foreign-encounter','offline','Other encounter','draft','now','now');",
        )
        .unwrap();
    write.commit().unwrap();
    assert_ne!(
        fixture.service.readiness().status,
        atlas_app_model::AppReadinessStatus::Ready
    );
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/lists",
            Some(json!({"slug":"offline","name":"Offline"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["list"]["slug"], "offline");
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/encounters",
            Some(json!({"name":"Offline"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["encounter"]["slug"], "offline");
    let (status, body) = fixture
        .json(
            Method::POST,
            "/api/parties",
            Some(json!({"name":"Offline party","members":[]})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let read = workspace
        .read(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let owners: (i64, i64, i64) = read
        .connection()
        .query_row(
            "SELECT (SELECT owner_id FROM saved_lists WHERE name='Offline'),
                (SELECT owner_id FROM encounters WHERE name='Offline'),
                (SELECT owner_id FROM parties WHERE name='Offline party')",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(owners, (7, 7, 7));
    let(status,body)=fixture.json(Method::POST,"/api/search/page",Some(json!({"criteria":{"kind":"list_records","filter":null},"page":{"number":1,"size":10}}))).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{body}");
    assert_eq!(body["code"], "index_unavailable");
}

#[tokio::test]
async fn missing_local_principal_blocks_saved_access_but_not_reference_reads() {
    let fixture = Fixture::new();
    let workspace = atlas_runtime::initialize_workspace(fixture.service.workspace_path()).unwrap();
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write
        .connection()
        .execute(
            "UPDATE principals SET principal_key = 'other' WHERE principal_key = 'local'",
            [],
        )
        .unwrap();
    write.commit().unwrap();

    let (status, body) = fixture.json(Method::GET, "/api/lists", None).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{body}");
    assert_eq!(body["code"], "internal_error");
    let (status, body) = fixture
        .json(Method::GET, &format!("/api/records/{ACTION}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = fixture.json(
        Method::POST,
        "/api/search/page",
        Some(json!({"criteria":{"kind":"list_records","filter":null},"page":{"number":1,"size":10}})),
    ).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn saved_list_vocabulary_respects_ownership() {
    let fixture = Fixture::new();
    let workspace = atlas_runtime::initialize_workspace(fixture.service.workspace_path()).unwrap();
    let write = workspace
        .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write
        .connection()
        .execute_batch(
            "INSERT INTO principals VALUES (7, 'other', 'Other');
         INSERT INTO saved_lists(owner_id,list_key,slug,name,created_at,updated_at)
         VALUES (7,'foreign-list','research','Other research','now','now');",
        )
        .unwrap();
    write.commit().unwrap();

    for (list_ref, expected_status) in [
        ("research", StatusCode::OK),
        ("foreign-list", StatusCode::NOT_FOUND),
    ] {
        let (status, body) = fixture
            .json(
                Method::POST,
                "/api/filters/values",
                Some(
                    json!({"kind":"vocabulary","scope":{"kind":"saved_list","list_ref":list_ref},
                        "field_id":"traits","text":"fire","limit":20}),
                ),
            )
            .await;
        assert_eq!(status, expected_status, "{body}");
    }
}

#[tokio::test]
async fn discovery_rejects_record_text_before_model_work_and_keeps_value_name_search() {
    let fixture = Fixture::new();
    for search_content in ["core", "all"] {
        let context =
            json!({"kind":"filtered","filter":null,"text":"fire","search_content":search_content});
        let request = json!({"kind":"contextual","context":context,"field_id":"source.pack.id","clause_id":null,"text":null,"offset":0,"limit":10,"only_matching":true});
        let (status, body) = fixture
            .json(Method::POST, "/api/filters/values", Some(request.clone()))
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(
            body["outcome"],
            json!({"status":"unavailable","reason":"text_context"})
        );
        let(status,body)=fixture.json(Method::POST,"/api/filters/counts",Some(json!({"context":context,"field_id":"source.pack.id","clause_id":null,"values":["actions"]}))).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(
            body["counts"],
            json!({"status":"unavailable","reason":"text_context"})
        );
        let mut invalid = request;
        invalid["limit"] = json!(0);
        let (status, _) = fixture
            .json(Method::POST, "/api/filters/values", Some(invalid))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    let context = json!({"kind":"filtered","filter":null,"text":"  "});
    let(status,body)=fixture.json(Method::POST,"/api/filters/values",Some(json!({"kind":"contextual","context":context,"field_id":"source.pack.id","clause_id":null,"text":"actions","offset":0,"limit":10,"only_matching":true}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["outcome"]["status"], "available");
    assert_eq!(body["outcome"]["value"]["exhaustive"], true);
    assert!(
        !body["outcome"]["value"]["options"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn compact_remaster_context_crosses_record_and_related_http_with_exact_journal_navigation() {
    let fixture = Fixture::with_remaster(true);
    let (status, body) = fixture
        .json(Method::GET, &format!("/api/records/{ACTION}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let detail: atlas_app_model::RecordDetailView = serde_json::from_value(body.clone()).unwrap();
    let relationship = &body["remaster_relationships"]["content"]["relationships"][0];
    assert_eq!(relationship["label"], "Replaced by");
    assert_eq!(
        relationship["parts"][0]["navigation"]["record_key"],
        SECOND_ACTION
    );
    assert!(
        body["remaster_relationships"]["content"]
            .get("declarations")
            .is_none()
    );
    let navigation = &relationship["explanation_links"][0]["navigation"];
    assert_eq!(navigation["record_key"], "journals:editionjournal01");
    assert_eq!(navigation["field"], "/text/content");
    assert_eq!(
        navigation["embedded_path"][0]["identity"]["Stable"],
        "editionpage00001"
    );
    let (status,related)=fixture.json(Method::POST,"/api/related",Some(json!({"selection":detail.selected,"artifact_generation":detail.artifact_generation}))).await;
    assert_eq!(status, StatusCode::OK, "{related}");
    let decoded: atlas_app_model::RelatedRecordsView = serde_json::from_value(related).unwrap();
    assert!(decoded.show_versions);
    assert_eq!(
        decoded.remaster_relationships,
        detail.remaster_relationships
    );
    let (status,journal)=fixture.json(Method::POST,"/api/records/detail",Some(json!({"record_key":navigation["record_key"],"embedded_path":navigation["embedded_path"],"fields":[navigation["field"]]}))).await;
    assert_eq!(status, StatusCode::OK, "{journal}");
}

#[tokio::test]
async fn publication_edition_crosses_real_http_for_known_and_unknown_sources() {
    let fixture = Fixture::new();
    for (key, expected) in [
        (ACTION, Some("legacy")),
        (SECOND_ACTION, Some("remaster")),
        (ACTOR, None),
    ] {
        let (status, body) = fixture
            .json(Method::GET, &format!("/api/records/{key}"), None)
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        for identity in [&body["record"], &body["presentation"]["identity"]] {
            assert_eq!(identity.get("edition").and_then(Value::as_str), expected);
            if expected.is_none() {
                assert!(identity.get("edition").is_none());
            }
        }
        let _: atlas_app_model::RecordDetailView = serde_json::from_value(body).unwrap();
    }
    let paired = Fixture::with_remaster(true);
    let (status, body) = paired
        .json(Method::GET, &format!("/api/records/{ACTION}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["remaster_relationships"]["kind"], "content");
    assert_eq!(
        body["remaster_relationships"]["content"]["relationships"][0]["parts"][0]["navigation"]["record_key"],
        SECOND_ACTION
    );
    assert_eq!(
        body["remaster_relationships"]["content"]["relationships"][0]["explanation_links"][0]["navigation"]
            ["record_key"],
        "journals:editionjournal01"
    );
}

#[tokio::test]
async fn unreadable_counterpart_evidence_does_not_prevent_reading_the_record() {
    let fixture = Fixture::with_remaster(true);
    fixture.corrupt_remaster_evidence();
    let (status, body) = fixture
        .json(Method::GET, &format!("/api/records/{ACTION}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["record"]["title"], "Test Action 1");
    assert_eq!(body["presentation"]["identity"]["edition"], "legacy");
    assert_eq!(body["remaster_relationships"]["kind"], "error");
    let _: atlas_app_model::RecordDetailView = serde_json::from_value(body).unwrap();
}
