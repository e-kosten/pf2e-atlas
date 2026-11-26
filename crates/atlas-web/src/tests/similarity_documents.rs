use super::{ACTOR, Fixture, Method, StatusCode, json};

#[tokio::test]
async fn journal_source_inventory_and_ranking_agree_on_the_open_document() {
    let fixture = Fixture::with_remaster(true);
    let key = "journals:editionjournal01";
    let path = json!([{"collection":"/pages","identity":{"Stable":"editionpage00001"}}]);
    let seed = json!({"kind":"record","record_key":key,"embedded_path":path});
    let (status, source) = fixture
        .json(
            Method::POST,
            "/api/search/similarity/source",
            Some(json!({"seed":seed})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{source}");
    assert_eq!(
        source["source"],
        json!({"seed": seed, "title":"Class Features", "parent_title":"Remaster Changes"})
    );
    let generation = &source["artifact_generation"];
    let (status, inventory) = fixture
        .json(
            Method::POST,
            "/api/search/similarity/parts",
            Some(json!({"seed":seed, "expected_artifact_generation":generation})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{inventory}");
    assert_eq!(inventory["seed"], seed);
    assert_eq!(inventory["source"], source["source"]);
    assert!(inventory["core"].is_null());
    let parts = inventory["parts"].as_array().unwrap();
    assert!(!parts.is_empty());
    assert!(
        parts
            .iter()
            .all(|part| part["kind"] == "section" && part["navigation"]["embedded_path"] == path)
    );
    let criteria = json!({"kind":"similar_records","seed_record_key":key,"embedded_path":path});
    let page = json!({"criteria":criteria,"result_detail":"evidence","page":{"number":1,"size":5},"expected_artifact_generation":generation});
    let (status, ranked) = fixture
        .json(Method::POST, "/api/search/page", Some(page.clone()))
        .await;
    assert_eq!(status, StatusCode::OK, "{ranked}");
    assert_eq!(ranked["mode"]["source"], source["source"]);
    assert_eq!(ranked["mode"]["seed"]["record_key"], key);
    assert!(!ranked["rows"].as_array().unwrap().is_empty());

    let mut manual = page.clone();
    manual["criteria"]
        .as_object_mut()
        .unwrap()
        .remove("embedded_path");
    manual["criteria"]["source_sections"] = json!("custom");
    manual["criteria"]["similarity"] = json!({"method":"passages","candidate_parts":"any","decay":0.5,"parts":parts.iter().map(|p| &p["selector"]).collect::<Vec<_>>()});
    let (status, explicit) = fixture
        .json(Method::POST, "/api/search/page", Some(manual))
        .await;
    assert_eq!(status, StatusCode::OK, "{explicit}");
    assert!(explicit["mode"].get("source").is_none());
    assert_eq!(ranked["rows"], explicit["rows"]);

    for (field, value) in [
        ("similarity", json!({"method":"baseline"})),
        ("source_sections", json!("core")),
    ] {
        let mut unsupported = page.clone();
        unsupported["criteria"][field] = value;
        let (status, error) = fixture
            .json(Method::POST, "/api/search/page", Some(unsupported))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{error}");
        assert_eq!(error["code"], "similarity_method_unavailable");
    }
}

#[tokio::test]
async fn source_lookup_distinguishes_missing_stale_and_malformed_addresses() {
    let fixture = Fixture::new();
    let path = json!([{"collection":"/items","identity":{"SnapshotLocal":{"index":0}}}]);
    let local = json!({"kind":"record","record_key":ACTOR,"embedded_path":path,"source_fingerprint":fixture.source_fingerprint});
    let source_path = "/api/search/similarity/source";
    for (seed, title) in [
        (json!({"kind":"record","record_key":ACTOR}), "Test Creature"),
        (local.clone(), "Claw"),
        (
            json!({"kind":"record","record_key":super::TABLE,"embedded_path":[{"collection":"/results","identity":{"Stable":"ffffffffffffffff"}}]}),
            "Result 1–2",
        ),
    ] {
        let (status, value) = fixture
            .json(
                Method::POST,
                source_path,
                Some(json!({"seed":seed, "expected_artifact_generation":null})),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(value["source"]["seed"], seed);
        assert_eq!(value["source"]["title"], title);
    }
    for fingerprint in [json!(null), json!("outdated")] {
        let mut stale = local.clone();
        stale["source_fingerprint"] = fingerprint;
        let (status, error) = fixture
            .json(Method::POST, source_path, Some(json!({"seed":stale})))
            .await;
        assert_eq!(status, StatusCode::CONFLICT, "{error}");
        assert_eq!(error["code"], "similarity_source_stale");
    }
    let mut missing = local;
    missing["embedded_path"][0]["identity"] = json!({"SnapshotLocal":{"index":123}});
    let (status, error) = fixture
        .json(Method::POST, source_path, Some(json!({"seed":missing})))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{error}");
    assert_eq!(error["code"], "record_not_found");

    let (status, error) = fixture
        .json(
            Method::POST,
            source_path,
            Some(json!({"seed":{"kind":"record","record_key":ACTOR,"embedded_path":null}})),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{error}");
}

#[tokio::test]
async fn source_limit_empty_page_and_invalid_custom_sections_have_distinct_recovery() {
    let key = "journals:classes000000000";
    let page_path = |id: &str| json!([{"collection":"/pages","identity":{"Stable":id}}]);
    let headings = (0..129)
        .map(|i| format!("<h2>Feature {i}</h2><p>Rules for feature {i}.</p>"))
        .collect::<String>();
    let journal = atlas_search::test_support::record(
        "journals",
        "JournalEntry",
        json!({
            "_id":"classes000000000","name":"Classes","pages":[
                {"_id":"longpage00000000","name":"Class features","type":"text","text":{"content":headings,"format":1}},
                {"_id":"imagepage0000000","name":"Class portrait","type":"image"}
            ]
        }),
    );
    let fixture = Fixture::with_reference_data(false, None, None, 16, vec![journal]);
    let generation = fixture.service.readiness().artifact_generation.unwrap();
    let long = page_path("longpage00000000");
    let empty = page_path("imagepage0000000");
    let empty_seed = json!({"kind":"record","record_key":key,"embedded_path":empty});
    let (status, source) = fixture
        .json(
            Method::POST,
            "/api/search/similarity/source",
            Some(json!({"seed":empty_seed})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{source}");
    assert_eq!(source["source"]["title"], "Class portrait");
    let (status, inventory) = fixture
        .json(
            Method::POST,
            "/api/search/similarity/parts",
            Some(json!({"seed":empty_seed})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{inventory}");
    assert_eq!(inventory["parts"], json!([]));
    let base = json!({"kind":"similar_records","seed_record_key":key,"embedded_path":long});
    let mut empty_page = base.clone();
    empty_page["embedded_path"] = empty;
    let mut custom = base.clone();
    custom["source_sections"] = json!("custom");
    custom["similarity"] =
        json!({"method":"passages","candidate_parts":"any","decay":0.5,"parts":["part:unknown"]});
    for (criteria, status, code) in [
        (base, StatusCode::BAD_REQUEST, "similarity_source_limit"),
        (empty_page, StatusCode::BAD_REQUEST, "similarity_no_content"),
        (custom, StatusCode::CONFLICT, "similarity_sections_stale"),
    ] {
        let (actual, error) = fixture.json(Method::POST,"/api/search/page",Some(json!({"criteria":criteria,"page":{"number":1,"size":5},"expected_artifact_generation":generation}))).await;
        assert_eq!(actual, status, "{error}");
        assert_eq!(error["code"], code);
    }
}

#[tokio::test]
async fn candidate_vector_limit_retains_its_recovery_code_in_the_http_response() {
    use axum::response::IntoResponse;
    let search =
        atlas_search::SearchError::from(atlas_index::IndexError::SimilarityUnitLimit(65_536));
    let service = atlas_app_service::AppServiceError::from(search);
    let response = crate::error::WebError::from(service).into_response();
    let (status, body) = super::response_json(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "similarity_candidate_limit");
}
