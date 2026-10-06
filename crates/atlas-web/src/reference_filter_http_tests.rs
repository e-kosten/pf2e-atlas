use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_ingest::{BuildArtifactOptions, build_artifact};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

struct FixtureDirectory(std::path::PathBuf);
impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn post(app: &Router, uri: &str, value: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(value.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (status, serde_json::from_slice(&body).expect("json"))
}

#[tokio::test]
async fn relationship_filters_query_all_source_references_with_deduplicated_pagination()
-> Result<(), Box<dyn std::error::Error>> {
    let root = FixtureDirectory(std::env::temp_dir().join(format!(
            "atlas-ref-search-http-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        )));
    let source = root.0.join("source");
    let pack = source.join("packs/spells/3rd-rank");
    std::fs::create_dir_all(&pack)?;
    let original = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../atlas-ingest/tests/fixtures/foundry-source/fireball-source-contract");
    std::fs::copy(original.join("module.json"), source.join("module.json"))?;
    let seed: Value = serde_json::from_slice(&std::fs::read(
        original.join("packs/spells/3rd-rank/fireball.json"),
    )?)?;
    std::fs::write(pack.join("fireball.json"), serde_json::to_vec(&seed)?)?;
    for index in 0..61 {
        let mut record = seed.clone();
        record["_id"] = json!(format!("ref{index:013}"));
        record["name"] = json!(format!("Reference fixture {index:02}"));
        record["system"]["description"]["value"] = json!(
            "<p>@UUID[Compendium.pf2e.spells-srd.Item.sxQZ6yqTn0czJxVd]{Fireball} and @UUID[Compendium.pf2e.spells-srd.Item.sxQZ6yqTn0czJxVd]{again}.</p>"
        );
        std::fs::write(
            pack.join(format!("ref-{index}.json")),
            serde_json::to_vec(&record)?,
        )?;
    }
    let artifact = root.0.join("index.sqlite");
    build_artifact(BuildArtifactOptions {
        source_root: source.clone(),
        output_path: artifact.clone(),
        manifest_path: None,
        embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
        embedding_cache_root: None,
        reuse_embeddings: false,
        embedding_batch_size: 8,
    })?;
    let app = crate::router(AtlasAppService::start(AtlasAppServiceOptions {
        source_root: Some(source),
        index_path: Some(artifact),
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
        ..Default::default()
    })?);
    let seed_key = "spells-srd:sxQZ6yqTn0czJxVd";
    let filter =
        json!({"clauses":[], "relationship":{"direction":"incoming","record_key":seed_key}});
    let (status, first) = post(&app, "/api/result-windows", json!({"mode":{"kind":"list_records","filter":filter,"sort":{"kind":"record_key"}},"page":{"number":1,"size":25},"include_diagnostics":false})).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["page"]["total"], 61);
    assert_eq!(first["rows"].as_array().expect("rows").len(), 25);
    let id = first["window_id"].as_u64().expect("window id");
    let mut keys = std::collections::BTreeSet::new();
    for page in 1..=3 {
        let (status, result) = post(
            &app,
            &format!("/api/result-windows/{id}/page"),
            json!({"page":{"number":page,"size":25}}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["page"]["total"], 61);
        for row in result["rows"].as_array().expect("rows") {
            assert!(
                keys.insert(
                    row["record"]["surface"]["metadata"]["record_key"]
                        .as_str()
                        .expect("key")
                        .to_string()
                )
            );
        }
        if page == 3 {
            assert_eq!(result["page"]["has_more"], false);
        }
    }
    assert_eq!(keys.len(), 61);
    let (status, discovery) = post(
        &app,
        "/api/filters/editor",
        json!({"context":{"kind":"filtered","filter":filter},"selected_field_ids":[]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{discovery}");
    assert_eq!(discovery["matching_record_count"], 61);
    let (status, values) = post(
        &app,
        "/api/filters/values",
        json!({"context":{"kind":"filtered","filter":filter},"field_id":"kind"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{values}");
    assert_eq!(values["matching_record_count"], 61);
    let response = app.clone().oneshot(Request::builder().uri(format!("/api/records/{seed_key}?reference_backlink_limit=50&reference_outgoing_limit=50")).body(Body::empty())?).await?;
    assert_eq!(response.status(), StatusCode::OK);
    let graph: Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(
        graph["surface"]["references"]["backlinks"]["total_records"],
        first["page"]["total"]
    );
    assert_eq!(
        graph["surface"]["references"]["backlinks"]["truncated"],
        true
    );
    let (status, outgoing) = post(&app, "/api/result-windows", json!({"mode":{"kind":"list_records","filter":{"clauses":[],"relationship":{"direction":"outgoing","record_key":"spells-srd:ref0000000000000"}},"sort":{"kind":"record_key"}},"page":{"number":1,"size":25},"include_diagnostics":false})).await;
    assert_eq!(status, StatusCode::OK, "{outgoing}");
    assert_eq!(outgoing["page"]["total"], 1);
    assert_eq!(
        outgoing["rows"][0]["record"]["surface"]["metadata"]["record_key"],
        seed_key
    );
    let (status, invalid) = post(&app, "/api/result-windows", json!({"mode":{"kind":"list_records","filter":{"clauses":[],"relationship":{"direction":"incoming","record_key":"Fireball"}},"sort":{"kind":"record_key"}},"page":{"number":1,"size":25},"include_diagnostics":false})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{invalid}");
    Ok(())
}
