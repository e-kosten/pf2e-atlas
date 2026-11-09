//! Real HTTP contracts over isolated source-free glossary fixtures.
use super::*;

#[tokio::test]
async fn glossary_reading_and_typed_associations_share_generation_and_identity() {
    let fixture = Fixture::with_glossary();
    let (status, detail) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:chthonian"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["definition_key"], "language:chthonian");
    assert_eq!(detail["label"], "Chthonian");
    assert!(
        detail["content"]["body"]["html"]
            .as_str()
            .unwrap()
            .contains("Typically spoken by demons")
    );
    assert!(
        detail["notices"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Synthetic test content")
    );
    assert_eq!(
        detail["publications"][0],
        json!({
            "title": "Fixture rulebook", "page": 89, "license": "CC0-1.0"
        })
    );
    for removed in [
        "attribution",
        "references",
        "references_availability",
        "references_truncated",
    ] {
        assert!(detail.get(removed).is_none(), "{removed}");
    }
    assert!(detail["notices"][0].get("sha256").is_none());
    assert_eq!(
        detail["content"]["section_rendering"]["selection_version"],
        "reference-definition/v1"
    );
    let inline = &detail["content"]["references"][0];
    assert_eq!(inline["destination"]["kind"], "resolved_record");
    assert_eq!(
        inline["destination"]["target"]["record_key"],
        "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6"
    );
    let generation = detail["artifact_generation"].clone();
    let (status, sense) = fixture.json(Method::POST, "/api/definitions/detail", Some(json!({"definition_key":"sense:darkvision","expected_artifact_generation":generation}))).await;
    assert_eq!(status, StatusCode::OK, "{sense}");
    assert!(
        sense["content"].is_null(),
        "record content must retain its existing owner"
    );
    assert_eq!(sense["help"]["destination"]["kind"], "record");
    assert_eq!(
        sense["help"]["destination"]["navigation"]["record_key"],
        "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6"
    );
    assert!(
        sense["help"]["destination"]
            .get("reading_context")
            .is_none()
    );
    let (status, editor) = fixture
        .json(
            Method::POST,
            "/api/filters/editor",
            Some(json!({"expected_artifact_generation":generation})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{editor}");
    assert_eq!(
        editor["sense_qualifiers"],
        json!([{"sense":"darkvision","acuity":null,"has_range":false}])
    );
    assert_eq!(
        detail["related_filters"][0],
        json!({
            "label":"With this language",
            "predicate":{"kind":"set_match","field":"languages","op":"includes","values":["chthonian"]}
        })
    );
    let (status, page) = fixture.json(Method::POST, "/api/search/page", Some(json!({
        "criteria":{"kind":"list_records","filter":detail["related_filters"][0]["predicate"]},
        "page":{"number":1,"size":20},
        "expected_artifact_generation":generation
    }))).await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(page["rows"][0]["record"]["record_key"], ACTOR);
    let (status, error) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(
                json!({"definition_key":"language:chthonian","expected_artifact_generation":"old"}),
            ),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert_eq!(error["code"], "artifact_generation_changed");
}

#[tokio::test]
async fn glossary_unknown_and_legacy_coverage_remain_distinct_without_breaking_records() {
    let fixture = Fixture::with_glossary();
    let (status, unknown) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:custom-unpublished"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{unknown}");
    assert_eq!(unknown.get("help"), Some(&serde_json::Value::Null));
    assert!(unknown["content"].is_null());
    let (status, bodyless) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:draconic"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{bodyless}");
    assert_eq!(bodyless["label"], "Draconic");
    assert_eq!(bodyless.get("help"), Some(&serde_json::Value::Null));
    assert_eq!(bodyless["related_filters"], json!([]));
    let legacy = Fixture::with_legacy_glossary();
    let (status, record) = legacy
        .json(
            Method::POST,
            "/api/records/detail",
            Some(json!({"record_key":ACTOR})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{record}");
    let language = &record["presentation"]["body"]["value"]["languages"]["value"][0];
    assert_eq!(language["value"], "common");
    assert!(language.get("help").is_none());
    let (status, error) = legacy
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:chthonian"})),
        )
        .await;
    assert!(status.is_client_error(), "{error}");
    assert_eq!(error["code"], "setup_required");
    let (status, stale) = legacy
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(
                json!({"definition_key":"language:chthonian","expected_artifact_generation":"old"}),
            ),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale}");
    assert_eq!(stale["code"], "artifact_generation_changed");
    assert_eq!(stale["recoverable_action"], "reload_record");
    let (status, trait_detail) = legacy
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"trait:atlas-custom"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{trait_detail}");
    assert_eq!(trait_detail["definition_key"], "trait:atlas-custom");
    assert!(trait_detail["help"].is_null());
    assert!(trait_detail["content"].is_null());
}

#[tokio::test]
async fn glossary_http_requests_distinguish_omitted_null_defaults_and_unknown_fields() {
    let fixture = Fixture::with_glossary();
    let (status, omitted) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(json!({"definition_key":"language:chthonian"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, explicit_null) = fixture
        .json(
            Method::POST,
            "/api/definitions/detail",
            Some(
                json!({"definition_key":"language:chthonian","expected_artifact_generation":null}),
            ),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(omitted, explicit_null);
    for (path, request) in [
        ("/api/definitions/detail", json!({})),
        ("/api/definitions/detail", json!({"definition_key":null})),
        (
            "/api/definitions/detail",
            json!({"definition_key":"language:chthonian","unknown":true}),
        ),
        (
            "/api/records/detail",
            json!({"record_key":ACTOR,"reading_context":"invented"}),
        ),
    ] {
        let (status, _) = fixture.json(Method::POST, path, Some(request)).await;
        assert!(status.is_client_error(), "{path}: {status}");
    }
}

#[tokio::test]
async fn removed_generic_reference_conditions_are_rejected() {
    let fixture = Fixture::with_glossary();
    let response = fixture
        .router()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/filters/definitions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    let (status, error) = fixture
        .json(
            Method::POST,
            "/api/search/page",
            Some(json!({
                "criteria": {"kind": "list_records", "filter": {
                    "kind": "definition_reference",
                    "reference": {
                        "target": {"kind": "definition", "key": "sense:darkvision"},
                        "include_contents": true
                    }
                }},
                "page": {"number": 1, "size": 20}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{error}");
    assert_eq!(error["code"], "invalid_request");
}

#[tokio::test]
async fn non_glossary_filter_values_and_vocabulary_keep_readable_backend_labels() {
    let fixture = Fixture::with_glossary();
    for request in [
        json!({"kind":"vocabulary","scope":{"kind":"artifact"},"field_id":"physical.base_type","text":"chain-mail","limit":10,"selected_values":["chain-mail"]}),
        json!({"kind":"contextual","context":{"kind":"filtered","filter":null,"text":null,"search_profile":"bge-base"},"field_id":"physical.base_type","text":"chain-mail","limit":10}),
    ] {
        let (status, values) = fixture
            .json(Method::POST, "/api/filters/values", Some(request))
            .await;
        assert_eq!(status, StatusCode::OK, "{values}");
        let data = if values["kind"] == "vocabulary" {
            &values["values"]
        } else {
            assert_eq!(values["outcome"]["status"], "available", "{values}");
            &values["outcome"]["value"]
        };
        let option = data["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|option| option["value"] == "chain-mail")
            .unwrap();
        assert_eq!(option["label"], "Chain Mail", "{values}");
        assert!(
            option.get("help").is_none(),
            "non-glossary values do not invent glossary help"
        );
    }
}

#[tokio::test]
async fn selected_action_keeps_its_record_label_outside_the_vocabulary_page() {
    let fixture = Fixture::with_glossary();
    let key = "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6";
    let (status, values) = fixture
        .json(
            Method::POST,
            "/api/filters/values",
            Some(json!({
                "kind":"vocabulary", "scope":{"kind":"artifact"}, "field_id":"actions",
                "text":"missing page", "limit":1, "selected_values":[key]
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{values}");
    let values = &values["values"];
    assert_eq!(values["total_values"], 0);
    assert_eq!(values["options"][0]["value"], key);
    assert_eq!(values["options"][0]["label"], "Darkvision");
    assert_eq!(
        values["options"][0]["help"]["destination"]["navigation"]["record_key"],
        key
    );
}
