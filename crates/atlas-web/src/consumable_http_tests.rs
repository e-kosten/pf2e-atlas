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
    let app = crate::router(service);
    let response = app
        .clone()
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
        Some(&Value::String("1".into()))
    );
    assert_eq!(
        body.pointer("/surface/presentation/body/spell_child/target_record_key"),
        Some(&Value::String("spells-srd:rfZpqmj0AIIdkVIs".into()))
    );
    assert!(
        body.pointer("/surface/presentation/body/use_action")
            .is_none()
    );
    let child_id = body
        .pointer("/surface/presentation/body/spell_child/child_id")
        .and_then(Value::as_str)
        .expect("exact child ID");
    assert_eq!(
        body.pointer("/surface/presentation/body/spell_child/parent_record_key"),
        Some(&Value::String("equipment-srd:eOtQtVRLeGH39dNx".into()))
    );
    let child_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/records/equipment-srd:eOtQtVRLeGH39dNx?consumable_child_id={child_id}"
                ))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(child_response.status(), StatusCode::OK);
    let child: Value =
        serde_json::from_slice(&to_bytes(child_response.into_body(), usize::MAX).await?)?;
    assert_eq!(
        child.pointer("/surface/presentation/presentation_type"),
        Some(&Value::String("spell".into()))
    );
    assert!(
        child
            .pointer("/surface/presentation/body/source_state")
            .is_none()
    );
    let wrong = app
        .oneshot(
            Request::builder()
                .uri("/api/records/equipment-srd:eOtQtVRLeGH39dNx?consumable_child_id=wrong-child")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(wrong.status(), StatusCode::BAD_REQUEST);
    Ok(())
}

#[tokio::test]
async fn pinned_npc_and_hazard_consumables_cross_production_readers_and_search_service()
-> Result<(), Box<dyn std::error::Error>> {
    use atlas_domain::{DetailLevel, RecordKey};
    use atlas_record::{RecordJsonOptions, record_json};
    use sha2::{Digest, Sha256};

    let repository = std::env::var_os("PF2E_SOURCE_REPOSITORY")
        .expect("TEST PREREQUISITE: accepted pinned PF2E repository");
    let root = FixtureDirectory(std::env::temp_dir().join(format!(
        "atlas-h5-pinned-http-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos(),
    )));
    std::fs::create_dir_all(root.0.join("packs/actors"))?;
    std::fs::write(
        root.0.join("module.json"),
        r#"{"packs":[{"name":"actors","label":"Actors","type":"Actor","path":"packs/actors"}]}"#,
    )?;
    let mut expected = Vec::new();
    for (path, digest, family) in [
        (
            "packs/abomination-vaults-bestiary/abomination-vaults-hardcover-compilation/nyzuros.json",
            "11f4a02fa7f9cea6ce3e14224ea2777e88900b340da5bdfa73ae53e1efe3cba3",
            "creature",
        ),
        (
            "packs/agents-of-edgewatch-bestiary/book-4-assault-on-hunting-lodge-seven/false-door-trap.json",
            "6560ba176f07c15d490b0c8eef5308d890c07150eb53dd4900524a4f903b600e",
            "hazard",
        ),
    ] {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args([
                "show",
                &format!("{}:{path}", atlas_ingest::PF2E_SOURCE_PINNED_COMMIT),
            ])
            .output()?;
        assert!(output.status.success(), "pinned fixture must exist: {path}");
        assert_eq!(format!("{:x}", Sha256::digest(&output.stdout)), digest);
        let raw: Value = serde_json::from_slice(&output.stdout)?;
        let id = raw["_id"].as_str().expect("source ID");
        let name = raw["name"].as_str().expect("source name").to_string();
        let consumables = raw["items"]
            .as_array()
            .expect("authored items")
            .iter()
            .filter(|item| item["type"] == "consumable")
            .count();
        assert!(consumables > 0);
        std::fs::write(
            root.0.join(format!("packs/actors/{id}.json")),
            output.stdout,
        )?;
        expected.push((
            RecordKey::parse(&format!("actors:{id}"))?,
            name,
            family,
            consumables,
        ));
    }
    let artifact = root.0.join("index.sqlite");
    build_artifact(BuildArtifactOptions {
        source_root: root.0.clone(),
        output_path: artifact.clone(),
        manifest_path: None,
        embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
        embedding_cache_root: None,
        reuse_embeddings: true,
        embedding_batch_size: 8,
    })?;
    let reader = atlas_index::SqliteIndexReader::open_read_only(&artifact)?;
    let all = reader.load_hydrated_records()?;
    let service = AtlasAppService::start(AtlasAppServiceOptions {
        source_root: Some(root.0.clone()),
        index_path: Some(artifact),
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
        ..Default::default()
    })?;
    let app = crate::router(service.clone());
    for (key, name, family, count) in expected {
        // Exercise the production lexical search service explicitly. The HTTP
        // text-search window defaults to hybrid retrieval and requires vectors.
        let lexical = service.search_text(
            name,
            None,
            None,
            atlas_search::SearchPage::new(1, 20)?,
            Some(
                atlas_search::TextSearchTuning::default()
                    .with_retrieval(atlas_search::RetrievalMode::Fts),
            ),
            false,
        )?;
        let search_record = lexical
            .records
            .iter()
            .find(|hit| hit.record.record.identity.key == key)
            .expect("production FTS service returns pinned actor");
        assert_eq!(
            search_record
                .record
                .consumable_occurrences
                .occurrences
                .len(),
            count
        );
        search_record
            .record
            .consumable_occurrences
            .validated_entities()?;
        let selected = reader.load_hydrated_records_by_key(std::slice::from_ref(&key))?;
        assert_eq!(&search_record.record, &selected[0]);
        assert_eq!(selected.len(), 1);
        assert_eq!(
            all.iter().find(|record| record.record.identity.key == key),
            selected.first()
        );
        let record = &selected[0];
        assert_eq!(record.consumable_occurrences.occurrences.len(), count);
        record.consumable_occurrences.validated_entities()?;
        let cli = serde_json::to_value(record_json(
            record,
            RecordJsonOptions {
                detail: DetailLevel::Full,
                include_source_json: false,
            },
        )?)?;
        assert_eq!(
            cli.pointer("/consumables")
                .and_then(Value::as_array)
                .expect("CLI occurrence destination")
                .len(),
            count
        );
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/records/{key}"))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let detail: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
        assert_eq!(
            detail
                .pointer("/surface/presentation/presentation_type")
                .and_then(Value::as_str),
            Some(family)
        );
        assert_eq!(
            detail
                .pointer("/surface/presentation/body/consumables")
                .and_then(Value::as_array)
                .expect("app-service occurrence destination")
                .len(),
            count
        );
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/result-windows")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&serde_json::json!({
                        "mode": {"kind":"list_records", "sort":{"kind":"record_key"}},
                        "page":{"number":1,"size":20}, "include_diagnostics":false
                    }))?))?,
            )
            .await?;
        let status = response.status();
        let search: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
        assert_eq!(status, StatusCode::OK, "search response: {search}");
        assert!(
            search["rows"]
                .as_array()
                .expect("search service rows")
                .iter()
                .any(|row| {
                    row.pointer("/record/surface/metadata/record_key")
                        .and_then(Value::as_str)
                        == Some(key.to_string().as_str())
                        && row
                            .pointer("/record/surface/presentation/body/consumables")
                            .and_then(Value::as_array)
                            .is_some_and(|occurrences| occurrences.len() == count)
                }),
            "search service must hydrate typed occurrences for {key}: {search}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn production_occurrence_child_navigation_distinguishes_reuse_and_retained_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    let repository = std::env::var_os("PF2E_SOURCE_REPOSITORY")
        .expect("TEST PREREQUISITE: accepted pinned PF2E repository");
    let output = std::process::Command::new("git").arg("-C").arg(repository)
        .args(["show", &format!("{}:packs/abomination-vaults-bestiary/abomination-vaults-hardcover-compilation/nyzuros.json", atlas_ingest::PF2E_SOURCE_PINNED_COMMIT)])
        .output()?;
    assert!(output.status.success());
    assert_eq!(
        format!("{:x}", Sha256::digest(&output.stdout)),
        "11f4a02fa7f9cea6ce3e14224ea2777e88900b340da5bdfa73ae53e1efe3cba3"
    );
    let mut npc: Value = serde_json::from_slice(&output.stdout)?;
    let owner_key = format!("actors:{}", npc["_id"].as_str().unwrap());
    let root = FixtureDirectory(std::env::temp_dir().join(format!(
        "atlas-h5-child-navigation-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos(),
    )));
    for pack in ["actors", "equipment", "spells"] {
        std::fs::create_dir_all(root.0.join("packs").join(pack))?;
    }
    std::fs::write(
        root.0.join("module.json"),
        r#"{"packs":[
        {"name":"actors","label":"Actors","type":"Actor","path":"packs/actors"},
        {"name":"equipment-srd","label":"Equipment","type":"Item","path":"packs/equipment"},
        {"name":"spells-srd","label":"Spells","type":"Item","path":"packs/spells"}
    ]}"#,
    )?;
    let portable = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../atlas-ingest/tests/fixtures/foundry-source/spell-source-contract/packs");
    let wand_bytes = std::fs::read(portable.join("equipment/arboreal-wand-rank-4.json"))?;
    let wand: Value = serde_json::from_slice(&wand_bytes)?;
    std::fs::write(root.0.join("packs/equipment/wand.json"), wand_bytes)?;
    std::fs::copy(
        portable.join("spells/heal.json"),
        root.0.join("packs/spells/heal.json"),
    )?;
    for (id, name, mismatch) in [
        ("h5ReusedWand", "Reused wand", false),
        ("h5LocalWand", "Retained local wand", true),
    ] {
        let mut local = wand.clone();
        local["_id"] = Value::String(id.to_string());
        local["name"] = Value::String(name.to_string());
        local["_stats"] = serde_json::json!({"compendiumSource":"Compendium.pf2e.equipment-srd.Item.eOtQtVRLeGH39dNx"});
        if mismatch {
            local["system"]["spell"]["system"]["damage"]["0"]["formula"] =
                Value::String("7d8".to_string());
            local["system"]["spell"]["system"]["description"] = serde_json::json!({
                "value":"<p>Occurrence-owned Spell narrative. @UUID[Compendium.pf2e.spells-srd.Item.rfZpqmj0AIIdkVIs]{Heal}</p>"
            });
        }
        npc["items"].as_array_mut().unwrap().push(local);
    }
    std::fs::write(
        root.0.join("packs/actors/npc.json"),
        serde_json::to_vec(&npc)?,
    )?;
    let artifact = root.0.join("index.sqlite");
    build_artifact(BuildArtifactOptions {
        source_root: root.0.clone(),
        output_path: artifact.clone(),
        manifest_path: None,
        embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
        embedding_cache_root: None,
        reuse_embeddings: true,
        embedding_batch_size: 8,
    })?;
    let reader = atlas_index::SqliteIndexReader::open_read_only(&artifact)?;
    let parent =
        reader.load_hydrated_records_by_key(&[atlas_domain::RecordKey::parse(&owner_key)?])?;
    for (name, reused) in [("Reused wand", true), ("Retained local wand", false)] {
        let occurrence = parent[0]
            .consumable_occurrences
            .occurrences
            .iter()
            .find(|occurrence| occurrence.contextual_name == name)
            .expect("authored occurrence");
        assert_eq!(
            matches!(
                &occurrence.spell_reuse,
                atlas_record::ConsumableSpellReuse::Reused { .. }
            ),
            reused
        );
        if !reused {
            if let atlas_record::ConsumableSpellReuse::Mismatch {
                local_evidence: atlas_record::ConsumableLocalSpellEvidence::Child(child),
                ..
            } = &occurrence.spell_reuse
            {
                let document = child
                    .definition
                    .content
                    .documents
                    .first()
                    .expect("retained authored Spell prose");
                assert_eq!(
                    document.provenance.nested_source_id.as_deref(),
                    Some("h5LocalWand")
                );
                assert!(
                    atlas_record::render_plain_text(&document.document)
                        .contains("Occurrence-owned Spell narrative")
                );
                assert_eq!(document.reference_occurrences.len(), 1);
            }
            assert!(
                matches!(
                    &occurrence.spell_reuse,
                    atlas_record::ConsumableSpellReuse::Mismatch {
                        local_evidence: atlas_record::ConsumableLocalSpellEvidence::Child(_),
                        ..
                    }
                ),
                "the local H2 definition must survive as typed mismatch evidence"
            );
        }
    }
    let app = crate::router(AtlasAppService::start(AtlasAppServiceOptions {
        source_root: Some(root.0.clone()),
        index_path: Some(artifact),
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
        ..Default::default()
    })?);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/records/{owner_key}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let parent: Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    let occurrences = parent
        .pointer("/surface/presentation/body/consumables")
        .and_then(Value::as_array)
        .unwrap();
    let mut selected_children = Vec::new();
    for (name, reused) in [("Reused wand", true), ("Retained local wand", false)] {
        let occurrence = occurrences
            .iter()
            .find(|occurrence| occurrence["name"] == name)
            .unwrap();
        let link = &occurrence["spell_child"];
        let parent_key = link["parent_record_key"]
            .as_str()
            .expect("exact child parent");
        assert_eq!(
            parent_key,
            if reused {
                "equipment-srd:eOtQtVRLeGH39dNx"
            } else {
                &owner_key
            }
        );
        let mut uri = format!(
            "/api/records/{parent_key}?consumable_child_id={}",
            link["child_id"].as_str().unwrap()
        );
        if reused {
            assert!(link.get("occurrence_id").is_none());
        } else {
            assert_eq!(link["occurrence_id"], occurrence["occurrence_id"]);
            uri.push_str(&format!(
                "&consumable_occurrence_id={}",
                link["occurrence_id"].as_str().unwrap()
            ));
        }
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let selected: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
        assert_eq!(
            selected
                .pointer("/surface/presentation/presentation_type")
                .and_then(Value::as_str),
            Some("spell")
        );
        selected_children.push(selected["surface"]["presentation"].clone());
    }
    assert_ne!(
        selected_children[0], selected_children[1],
        "inspection must not replace retained local mechanics with canonical mechanics"
    );
    assert!(selected_children[1].to_string().contains("7d8"));
    assert!(
        selected_children[1]
            .to_string()
            .contains("Occurrence-owned Spell narrative")
    );
    let wrong = app.oneshot(Request::builder()
        .uri(format!("/api/records/{owner_key}?consumable_child_id=wrong&consumable_occurrence_id=wrong"))
        .body(Body::empty())?).await?;
    assert_eq!(wrong.status(), StatusCode::BAD_REQUEST);
    Ok(())
}
