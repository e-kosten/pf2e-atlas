use super::*;
use crate::test_support::publish_input;
use atlas_app_model::{CreateEncounterRequest, SearchPageRequest};
use atlas_search::test_support::{FixtureArtifact, input, open_input_fixture, record};
use serde_json::json;

fn command(artifact: &FixtureArtifact, mode: AppServiceRetrievalMode) -> AtlasAppService {
    atlas_runtime::initialize_workspace(artifact.path()).unwrap();
    AtlasAppService::start(AtlasAppServiceOptions {
        index_path: Some(artifact.path()),
        embedding_cache_root: Some(artifact.path().with_file_name("missing-model-assets")),
        retrieval_mode: mode,
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn command_services_are_send_and_sync_without_retaining_snapshots() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<AtlasAppService>();
    assert_sync::<AtlasAppService>();
}

#[test]
fn command_reads_see_new_revision_and_preserve_saved_state() {
    let source = |name: &str| {
        input(
            vec![record(
                "items",
                "Item",
                json!({"_id":"testAction100000","type":"action","name":name}),
            )],
            false,
        )
    };
    let (reader, artifact) = open_input_fixture(source("Original")).unwrap();
    drop(reader);
    let service = command(&artifact, AppServiceRetrievalMode::OnDemandNoEmbeddings);
    let saved = service
        .create_encounter(CreateEncounterRequest {
            name: "Saved".into(),
            description: None,
            note: None,
        })
        .unwrap();
    let before = service.record_detail("items:testAction100000").unwrap();
    publish_input(artifact.path(), &source("Replacement")).unwrap();
    let after = service.record_detail("items:testAction100000").unwrap();
    assert_eq!(after.record.title, "Replacement");
    assert_ne!(before.artifact_generation, after.artifact_generation);
    assert_eq!(
        service
            .encounter(&saved.encounter.encounter_key)
            .unwrap()
            .encounter
            .name,
        "Saved"
    );
}

#[test]
fn source_free_commands_work_in_empty_workspace_without_model_assets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.sqlite");
    atlas_runtime::initialize_workspace(&path).unwrap();
    let service = AtlasAppService::start(AtlasAppServiceOptions {
        index_path: Some(path),
        retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
        ..Default::default()
    })
    .unwrap();
    service
        .create_encounter(CreateEncounterRequest {
            name: "Empty source".into(),
            description: None,
            note: None,
        })
        .unwrap();
    assert!(service.saved_lists().unwrap().lists.is_empty());
}

#[test]
fn sole_gemma_manifest_resolves_omitted_requests_and_readiness_without_model_assets() {
    let mut input = atlas_search::test_support::input_with_both_profiles(
        (0..2)
            .map(|i| {
                record(
                    "items",
                    "Item",
                    json!({"_id":format!("{i:016}"),"name":format!("Record {i}"),"type":"effect"}),
                )
            })
            .collect(),
    );
    input
        .context
        .search_profiles
        .retain(|p| p.profile_id.as_str() == "embeddinggemma");
    input
        .semantic_units
        .retain(|p| p.profile_id.as_str() == "embeddinggemma");
    let (reader, artifact) = open_input_fixture(input).unwrap();
    drop(reader);
    let service = command(&artifact, AppServiceRetrievalMode::OnDemandStoredVectors);
    let readiness = service.readiness();
    assert_eq!(readiness.status, atlas_app_model::AppReadinessStatus::Ready);
    assert_eq!(readiness.default_search_profile.as_str(), "embeddinggemma");
    assert_eq!(readiness.search_profiles.len(), 1);
    assert_eq!(readiness.search_profiles[0].id.as_str(), "embeddinggemma");
    assert!(readiness.search_profiles[0].stored_vectors);
    assert!(!readiness.search_profiles[0].assets_ready);
    let page = service
        .search_page(
            atlas_app_model::SearchResultsRequest {
                ranking: None,
                search_profile: None,
                criteria: atlas_app_model::SearchCriteria::SimilarRecords {
                    seed_record_key: "items:0000000000000000".into(),
                    filter: None,
                },
                page: SearchPageRequest {
                    number: 1,
                    size: 10,
                },
                expected_artifact_generation: readiness.artifact_generation.clone(),
            },
            &ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_eq!(page.search_profile.as_str(), "embeddinggemma");
    assert!(!page.rows.is_empty());
    assert_eq!(
        Some(page.artifact_generation),
        readiness.artifact_generation
    );
}
