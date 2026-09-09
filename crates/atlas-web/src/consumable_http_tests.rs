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
async fn source_backed_consumable_uses_the_typed_http_surface()
-> Result<(), Box<dyn std::error::Error>> {
    let root = FixtureDirectory(std::env::temp_dir().join(format!(
        "atlas-h5-consumable-http-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    )));
    std::fs::create_dir(&root.0)?;
    let artifact = root.0.join("index.sqlite");
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../atlas-ingest/tests/fixtures/foundry-source/spell-source-contract");
    build_artifact(BuildArtifactOptions {
        source_root: source.clone(),
        output_path: artifact.clone(),
        manifest_path: None,
        embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
        embedding_cache_root: None,
        reuse_embeddings: true,
        embedding_batch_size: 8,
    })?;
    let service = AtlasAppService::start(AtlasAppServiceOptions {
        source_root: Some(source),
        index_path: Some(artifact),
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
        ..Default::default()
    })?;
    let response = crate::router(service)
        .oneshot(
            Request::builder()
                .uri("/api/records/equipment-srd:eOtQtVRLeGH39dNx")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(
        body.pointer("/surface/presentation/presentation_type"),
        Some(&Value::String("consumable".into()))
    );
    assert_eq!(
        body.pointer("/surface/presentation/body/category/state"),
        Some(&Value::String("missing".into()))
    );
    assert_eq!(
        body.pointer("/surface/presentation/body/source_state/current_uses/value"),
        Some(&Value::from(1))
    );
    assert_eq!(
        body.pointer("/surface/presentation/body/spell_child/target_record_key"),
        Some(&Value::String("spells-srd:rfZpqmj0AIIdkVIs".into()))
    );
    assert!(
        body.pointer("/surface/presentation/body/use_action")
            .is_none()
    );
    Ok(())
}
