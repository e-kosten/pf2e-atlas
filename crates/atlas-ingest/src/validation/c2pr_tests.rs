use super::*;
use atlas_index::ArtifactMetadataSummary;

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "atlas-c2pr-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

fn options(root: &Path) -> ExhaustiveValidationOptions {
    ExhaustiveValidationOptions {
        source_root: root.join("source"),
        candidate_head: "candidate".to_string(),
        snapshot_root: root.join("optimized-author-snapshot"),
        report_path: root.join("optimized-author-exhaustive.json"),
        embedding_cache_root: root.join("embedding-cache"),
        force_reproduction: false,
    }
}

fn tuple(stage: &Path, selector: &str) -> ArtifactValidationTuple {
    ArtifactValidationTuple {
        candidate_commit: "candidate".to_string(),
        candidate_tree: "tree".to_string(),
        snapshot_stage: stage.to_path_buf(),
        mode: "no_embeddings".to_string(),
        source_signature: "source-signature".to_string(),
        artifact_contract_version: ARTIFACT_CONTRACT_VERSION.to_string(),
        artifact_schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        embedding: resolve_validation_embedding_identity(selector)
            .expect("supported fixture selector"),
    }
}

fn receipt_report(
    tuple: &ArtifactValidationTuple,
    embedding_model_id: &str,
) -> ArtifactValidationReport {
    ArtifactValidationReport::ok(
        tuple
            .snapshot_stage
            .join("artifacts/no_embeddings.sqlite")
            .display()
            .to_string(),
        ArtifactMetadataSummary {
            artifact_contract_version: Some(tuple.artifact_contract_version.clone()),
            schema_version: Some(tuple.artifact_schema_version.clone()),
            source_signature: Some(tuple.source_signature.clone()),
            embedding_model_id: Some(embedding_model_id.to_string()),
            ..ArtifactMetadataSummary::default()
        },
    )
}

#[test]
fn alias_and_canonical_selectors_bind_to_one_typed_receipt_identity() {
    let stage = PathBuf::from("/validation/snapshot");
    let alias = tuple(&stage, "bge-small-en-v1.5");
    let canonical = tuple(&stage, "BAAI/bge-small-en-v1.5");
    assert_eq!(alias.embedding.model, EmbeddingModelId::BgeSmallEnV15);
    assert_eq!(alias.embedding.model, canonical.embedding.model);
    assert_eq!(alias.embedding.canonical_model_id, "BAAI/bge-small-en-v1.5");
    assert_eq!(
        alias.embedding.canonical_model_id,
        canonical.embedding.canonical_model_id
    );

    let generation = json!({
        "canonical_artifact_path": "/validation/snapshot/artifacts/no_embeddings.sqlite",
        "generation_path": "/validation/snapshot/artifacts/.generations/sha.sqlite",
        "file_identity": "dev:1:ino:2",
        "bytes": 10,
        "trusted_sha256": "sha",
    });
    assert_eq!(
        validation_binding_digest(&alias, &generation),
        validation_binding_digest(&canonical, &generation),
        "request spelling must not create a second validation identity"
    );

    let report = receipt_report(&alias, "BAAI/bge-small-en-v1.5");
    validate_receipt_metadata(&alias, &report).expect("alias request matches canonical metadata");
    validate_receipt_metadata(&canonical, &report)
        .expect("canonical request matches canonical metadata");
}

#[test]
fn genuinely_different_catalog_model_is_a_typed_mismatch() {
    let stage = PathBuf::from("/validation/snapshot");
    let expected = tuple(&stage, "bge-small-en-v1.5");
    let report = receipt_report(&expected, "BAAI/bge-base-en-v1.5");
    let mismatch = validate_receipt_metadata(&expected, &report)
        .expect_err("different catalog model must fail closed");

    assert_eq!(mismatch.error_code, "validation_receipt_identity_mismatch");
    assert_eq!(mismatch.tuple_field.as_deref(), Some("embedding_model"));
    assert_eq!(
        mismatch.semantic_enum_identity.as_deref(),
        Some("EmbeddingModelId::BgeSmallEnV15")
    );
    assert_eq!(
        mismatch
            .expected
            .as_ref()
            .and_then(|value| value.canonical_model_id.as_deref()),
        Some("BAAI/bge-small-en-v1.5")
    );
    assert_eq!(
        mismatch
            .actual
            .as_ref()
            .and_then(|value| value.semantic_enum_identity.as_deref()),
        Some("EmbeddingModelId::BgeBaseEnV15")
    );
    assert_eq!(
        mismatch
            .actual
            .as_ref()
            .and_then(|value| value.canonical_model_id.as_deref()),
        Some("BAAI/bge-base-en-v1.5")
    );
}

#[test]
fn pre_artifact_failure_is_atomic_structured_and_timed() {
    let root = temp_root("pre-artifact-failure");
    fs::create_dir(&root).expect("fixture root");
    let options = options(&root);
    let stage = staging_path(&options.snapshot_root);
    fs::create_dir(&stage).expect("fixture stage");
    let mut journal = ValidationRunJournal::new();
    journal
        .progress(&stage, "pre_artifact_identity", "started")
        .expect("start early validation");
    let detail = resolve_validation_embedding_identity("not-a-catalog-model")
        .expect_err("invalid selector fixture");
    journal.fail_with(*detail);
    journal
        .progress(&stage, "pre_artifact_identity", "failed")
        .expect("record early failure");

    let error = validation_error("deliberate pre-artifact identity failure");
    let failed = preserve_failed_snapshot(&options, &stage, &error, &journal, &BTreeMap::new())
        .expect("preserve early failure");
    assert!(!failed.join("artifacts").exists());
    verify_checksums(&failed, Some(&BTreeMap::new())).expect("checksum-bound failed snapshot");

    let failure: Value = serde_json::from_slice(
        &fs::read(root.join("failure.json")).expect("top-level structured failure"),
    )
    .expect("parse structured failure");
    let timing: Value = serde_json::from_slice(
        &fs::read(root.join("timing.json")).expect("top-level partial timing"),
    )
    .expect("parse partial timing");
    assert_eq!(failure["status"], json!("fail"));
    assert_eq!(failure["phase"], json!("pre_artifact_identity"));
    assert_eq!(
        failure["error_code"],
        json!("validation_embedding_selector_invalid")
    );
    assert_eq!(failure["artifact_identities"], json!({}));
    assert_eq!(failure["counter_state"]["source_traversal_count"], json!(0));
    assert_eq!(timing["status"], json!("fail"));
    assert!(timing["failure_serialization"]["wall_ms"].is_number());
    assert!(root.join("failure.json.sha256").is_file());
    assert!(root.join("timing.json.sha256").is_file());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn post_publication_failure_reuses_trusted_digests_and_retains_typed_detail() {
    let root = temp_root("post-publication-failure");
    fs::create_dir(&root).expect("fixture root");
    let options = options(&root);
    let stage = staging_path(&options.snapshot_root);
    let visible = PathBuf::from("artifacts/no_embeddings.sqlite");
    let generation = PathBuf::from("artifacts/.generations/trusted.sqlite");
    fs::create_dir_all(stage.join("artifacts/.generations")).expect("fixture artifacts");
    fs::write(stage.join(&visible), b"visible bytes").expect("visible fixture");
    fs::write(stage.join(&generation), b"generation bytes").expect("generation fixture");
    let trusted_sha = "trusted-digest-carried-from-live-handle".to_string();
    let trusted = BTreeMap::from([
        (visible.clone(), trusted_sha.clone()),
        (generation.clone(), trusted_sha.clone()),
    ]);

    let mut journal = ValidationRunJournal::new();
    journal
        .progress(&stage, "no_embeddings", "started")
        .expect("start mode");
    journal
        .progress(&stage, "no_embeddings.build_write_publish", "started")
        .expect("start build");
    journal
        .progress(&stage, "no_embeddings.build_write_publish", "passed")
        .expect("complete build");
    journal
        .progress(&stage, "no_embeddings.reader_open", "started")
        .expect("start reader");
    journal.record_artifact_identity("no_embeddings", &visible, &generation, &trusted_sha, 13);
    journal.record_c2p_counters("no_embeddings");
    journal
        .progress(&stage, "no_embeddings.reader_open", "passed")
        .expect("complete reader");
    journal
        .progress(&stage, "no_embeddings.deep_validation", "started")
        .expect("start receipt");

    let expected = tuple(&stage, "bge-small-en-v1.5");
    let report = receipt_report(&expected, "BAAI/bge-base-en-v1.5");
    let detail = validate_receipt_metadata(&expected, &report)
        .expect_err("different model must fail the receipt");
    journal.fail_with(*detail);
    journal
        .progress(&stage, "no_embeddings.deep_validation", "failed")
        .expect("record receipt failure");

    let error = validation_error("deliberate post-publication receipt failure");
    let failed = preserve_failed_snapshot(&options, &stage, &error, &journal, &trusted)
        .expect("preserve post-publication failure");
    let checksums = fs::read_to_string(failed.join("checksums.sha256"))
        .expect("failed snapshot checksum inventory");
    assert!(checksums.contains(&format!("{trusted_sha}  {}", visible.display())));
    assert!(checksums.contains(&format!("{trusted_sha}  {}", generation.display())));
    verify_checksums(&failed, Some(&trusted)).expect("trusted checksum closure");
    assert!(
        verify_checksums(&failed, None).is_err(),
        "fixture digest is deliberately not the artifact-byte digest"
    );

    let failure: Value = serde_json::from_slice(
        &fs::read(root.join("failure.json")).expect("top-level structured failure"),
    )
    .expect("parse structured failure");
    assert_eq!(failure["mode"], json!("no_embeddings"));
    assert_eq!(failure["phase"], json!("no_embeddings.deep_validation"));
    assert_eq!(failure["tuple_field"], json!("embedding_model"));
    assert_eq!(
        failure["typed_expected"]["semantic_enum_identity"],
        json!("EmbeddingModelId::BgeSmallEnV15")
    );
    assert_eq!(
        failure["typed_actual"]["semantic_enum_identity"],
        json!("EmbeddingModelId::BgeBaseEnV15")
    );
    assert_eq!(
        failure["checksum_closure"]["trusted_artifact_digest_count"],
        json!(2)
    );
    assert_eq!(
        failure["checksum_closure"]["failure_preservation_redundant_full_sha_pass_count"],
        json!(0)
    );
    assert_eq!(
        failure["checksum_closure"]["failure_preservation_unclassified_full_sha_pass_count"],
        json!(0)
    );
    assert_eq!(
        failure["counter_state"]["no_embeddings.validation_side_digest_handle_bind_count"],
        json!(1)
    );
    assert!(
        failure["in_progress_operations"]
            .as_array()
            .is_some_and(|operations| operations.iter().any(|operation| {
                operation["phase"] == json!("no_embeddings")
                    && operation["status"] == json!("in_progress")
            }))
    );

    let _ = fs::remove_dir_all(root);
}
