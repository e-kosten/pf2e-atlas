use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_ingest::{BuildArtifactOptions, build_artifact};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

struct FixtureDirectory(std::path::PathBuf);
impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn pinned_spells_resolve_through_artifact_service_and_http()
-> Result<(), Box<dyn std::error::Error>> {
    for (fixture, key, cases) in [
        (
            "fireball-source-contract",
            "spells-srd:sxQZ6yqTn0czJxVd",
            vec![("Base", 3, "6d6"), ("Base", 5, "10d6")],
        ),
        (
            "spell-source-contract",
            "spells-srd:rfZpqmj0AIIdkVIs",
            vec![
                ("Base", 3, "3d8"),
                ("Heal (vs. Undead)", 3, "3d8"),
                ("Heal (vs. Living)", 3, "3d8+24"),
            ],
        ),
    ] {
        let root = FixtureDirectory(std::env::temp_dir().join(format!(
                "atlas-ux8-http-{}-{fixture}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos()
            )));
        std::fs::create_dir(&root.0)?;
        let artifact = root.0.join("index.sqlite");
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../atlas-ingest/tests/fixtures/foundry-source")
            .join(fixture);
        build_artifact(BuildArtifactOptions {
            source_root: source.clone(),
            output_path: artifact.clone(),
            manifest_path: None,
            embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
            embedding_cache_root: None,
            reuse_embeddings: false,
            embedding_batch_size: 8,
        })?;
        let service = AtlasAppService::start(AtlasAppServiceOptions {
            source_root: Some(source),
            index_path: Some(artifact),
            retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
            ..Default::default()
        })?;
        let app = crate::router(service);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/records/{key}"))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let base: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
        let forms = base
            .pointer("/surface/presentation/body/forms")
            .and_then(Value::as_array)
            .expect("form catalog");
        for (label, rank, expected) in cases {
            let id = forms
                .iter()
                .find(|form| form["label"] == label)
                .expect("known form")["id"]
                .as_str()
                .expect("opaque id");
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!(
                            "/api/records/{key}?spell_form_id={id}&spell_cast_rank={rank}"
                        ))
                        .body(Body::empty())?,
                )
                .await?;
            assert_eq!(response.status(), StatusCode::OK);
            let selected: Value =
                serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
            let effective = selected
                .pointer("/surface/presentation/body/effective_form")
                .expect("effective form");
            assert_eq!(effective["id"], id);
            assert_eq!(effective["cast_rank"], rank);
            let member = effective
                .pointer("/result/definition/damage/value/value/0")
                .expect("resolved damage");
            assert_eq!(member["formula"]["value"], expected, "{label} rank {rank}");
            assert!(member.get("key").is_none());
        }
        for query in [
            "spell_form_id=unknown&spell_cast_rank=5",
            "spell_cast_rank=5",
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/records/{key}?{query}"))
                        .body(Body::empty())?,
                )
                .await?;
            if query.starts_with("spell_form_id=unknown") {
                assert_eq!(response.status(), StatusCode::OK);
                let failure: Value =
                    serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
                assert_eq!(
                    failure.pointer("/surface/presentation/body/effective_form/result/state"),
                    Some(&Value::String("unavailable".into()))
                );
            } else {
                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            }
        }
    }
    Ok(())
}
