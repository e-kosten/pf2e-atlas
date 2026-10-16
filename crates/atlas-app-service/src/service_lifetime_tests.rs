use super::*;
use atlas_app_model::{
    AddEncounterManualParticipantRequest, CreateEncounterRequest, OwnedReadingRequest,
    OwnedReadingSelection, PreparedFieldBodyView, RecordBodyView, RecordDetailRequest,
    RetrievalModeView, SearchPageRequest,
};
use atlas_search::test_support::{FixtureArtifact, input, open_input_fixture, record};
use serde_json::json;

const ACTOR: &str = "actors:testCreature1000";

fn artifact(name: &str, semantic: bool, locale: &str) -> FixtureArtifact {
    let mut input = input(
        vec![record(
            "actors",
            "Actor",
            json!({
                "_id":"testCreature1000","type":"npc","name":name,
                "system":{"details":{"publicNotes":format!("<p>{name} creature prose</p>")}},
                "items":[
                    {"_id":"ability000000000","type":"action","name":format!("{name} ability"),
                     "system":{"description":{"value":format!("<p>{name} ability prose</p>")}}},
                    {"_id":"spell00000000000","type":"spell","name":format!("{name} spell"),
                     "system":{"description":{"value":format!("<p>{name} spell prose</p>")},
                               "level":{"value":1}}}
                ]
            }),
        )],
        semantic,
    );
    input.context.indexing_locale = locale.into();
    let hash = input.context.preparation_sha256().unwrap();
    for record in &mut input.records {
        record.preparation_context_hash = hash.clone();
    }
    let (reader, artifact) = open_input_fixture(input).unwrap();
    drop(reader);
    artifact
}

fn command(artifact: &FixtureArtifact, mode: AppServiceRetrievalMode) -> AtlasAppService {
    let mut service = AtlasAppService::start(AtlasAppServiceOptions {
        index_path: Some(artifact.path()),
        source_root: Some(artifact.path().with_file_name("absent-source")),
        embedding_cache_root: Some(artifact.path().with_file_name("absent-model")),
        retrieval_mode: mode,
        ..Default::default()
    })
    .unwrap();
    service.local_state_path = artifact.path().with_file_name("local-state.sqlite");
    service
}

fn retained_instance(service: &AtlasAppService) -> Option<atlas_search::ArtifactInstance> {
    let RetrievalBackend::Command { retrieval, .. } = &service.retrieval else {
        panic!("expected command reader")
    };
    retrieval
        .lock()
        .unwrap()
        .as_ref()
        .map(|reader| reader.artifact_instance().clone())
}

fn assert_content_contains(fields: &[atlas_app_model::PreparedContentFieldView], text: &str) {
    assert!(fields.iter().any(|field| match &field.body {
        PreparedFieldBodyView::Html { html, .. } => html.contains(text),
        PreparedFieldBodyView::Plain { text: plain } => plain.contains(text),
        _ => false,
    }));
}

#[test]
fn retained_command_types_are_send_and_app_service_is_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<AtlasRetrievalService>();
    assert_send::<AtlasAppService>();
    assert_sync::<AtlasAppService>();
}

#[test]
fn source_free_commands_keep_every_lazy_mode_unopened() {
    let artifact = artifact("Unused", false, "en");
    for mode in [
        AppServiceRetrievalMode::OnDemandNoEmbeddings,
        AppServiceRetrievalMode::OnDemandStoredVectors,
        AppServiceRetrievalMode::OnDemandEmbeddings,
    ] {
        let mut service = command(&artifact, mode);
        service.runtime = Arc::new(
            AtlasRuntime::resolve(AtlasRuntimeOptions {
                overrides: AtlasPathOverrides {
                    index_path: Some(artifact.path().with_file_name("absent-artifact.sqlite")),
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap(),
        );
        assert!(retained_instance(&service).is_none());
        let encounter = service
            .create_encounter(CreateEncounterRequest {
                name: format!("Source free {mode:?}"),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter;
        let receipt = service
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: encounter.encounter_key,
                display_name: "Manual PC".into(),
                max_hp: Some(10),
                current_hp: Some(8),
                initiative: None,
            })
            .unwrap();
        let detail = service.encounter(&receipt.encounter_key).unwrap();
        assert_eq!(detail.artifact_generation, None);
        assert_eq!(detail.participants[0].current_hp, Some(8));
        assert!(service.saved_lists().unwrap().lists.is_empty());
        assert!(retained_instance(&service).is_none());
        assert_eq!(
            service
                .record_detail(ACTOR)
                .unwrap_err()
                .into_app_error()
                .code,
            AppErrorCode::IndexUnavailable
        );
        assert!(retained_instance(&service).is_none());
        assert_eq!(
            service
                .encounter(&receipt.encounter_key)
                .unwrap()
                .participants[0]
                .current_hp,
            Some(8)
        );
    }
}

#[test]
fn explicit_command_modes_preserve_model_requirements() {
    let page = SearchPageRequest {
        number: 1,
        size: 10,
    };
    let no_vectors = artifact("Original", false, "en");
    let model_free = command(&no_vectors, AppServiceRetrievalMode::OnDemandNoEmbeddings);
    assert!(retained_instance(&model_free).is_none());
    assert_eq!(
        model_free.record_detail(ACTOR).unwrap().record.title,
        "Original"
    );
    let instance = retained_instance(&model_free).unwrap();
    assert_eq!(
        model_free
            .search_text(
                "Original".into(),
                None,
                page,
                RetrievalModeView::Lexical,
                None
            )
            .unwrap()
            .rows
            .len(),
        1
    );
    assert_eq!(retained_instance(&model_free).unwrap(), instance);
    assert_eq!(
        model_free
            .search_text(
                "Original".into(),
                None,
                page,
                RetrievalModeView::Semantic,
                None
            )
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::VectorReadinessRequired
    );
    let stored = command(&no_vectors, AppServiceRetrievalMode::OnDemandStoredVectors);
    assert_eq!(
        stored
            .record_detail(ACTOR)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::IndexUnavailable
    );
    assert!(retained_instance(&stored).is_none());

    let vectors = artifact("Original", true, "en");
    let stored = command(&vectors, AppServiceRetrievalMode::OnDemandStoredVectors);
    assert_eq!(
        stored.record_detail(ACTOR).unwrap().record.title,
        "Original"
    );
    assert!(
        stored
            .similar_records(ACTOR.into(), None, page, None)
            .unwrap()
            .is_some()
    );
    assert!(!vectors.path().with_file_name("absent-model").exists());
    let full = command(&vectors, AppServiceRetrievalMode::OnDemandEmbeddings);
    assert_eq!(
        full.record_detail(ACTOR).unwrap_err().into_app_error().code,
        AppErrorCode::EmbeddingModelUnavailable
    );
    assert!(retained_instance(&full).is_none());
    assert!(!vectors.path().with_file_name("absent-model").exists());
}

#[cfg(unix)]
#[test]
fn command_clones_keep_root_owned_reading_and_stable_children_on_one_artifact() {
    let artifact = artifact("Original", false, "en");
    let service = command(&artifact, AppServiceRetrievalMode::OnDemandNoEmbeddings);
    let clone = service.clone();
    assert!(retained_instance(&service).is_none());
    let root = service.record_detail(ACTOR).unwrap();
    let instance = retained_instance(&service).unwrap();
    let RecordBodyView::Creature(actor) = &root.presentation.body else {
        panic!("expected creature")
    };
    let activity = actor
        .activities
        .value
        .as_ref()
        .unwrap()
        .iter()
        .find(|activity| activity.title == "Original ability")
        .unwrap()
        .navigation
        .clone();
    // Publish a complete, valid replacement with a different preparation context.
    let replacement = self::artifact("Replacement", false, "fr");
    std::fs::rename(replacement.path(), artifact.path()).unwrap();

    let batch = clone
        .owned_reading(OwnedReadingRequest {
            parent: root.selected.clone(),
            artifact_generation: root.artifact_generation.clone(),
            selection: OwnedReadingSelection::Abilities {
                cursor: None,
                target: None,
            },
        })
        .unwrap();
    assert_eq!(batch.artifact_generation, root.artifact_generation);
    assert_eq!(batch.entries.len(), 1);
    assert_content_contains(&batch.entries[0].content, "Original ability prose");
    let child = clone
        .record_detail_at(RecordDetailRequest {
            record_key: activity.record_key,
            owners: activity.owners,
            fields: vec![],
            passage: None,
            source_fingerprint: activity.source_fingerprint,
        })
        .unwrap();
    assert_eq!(child.presentation.identity.title, "Original ability");
    assert_eq!(child.artifact_generation, root.artifact_generation);
    assert_eq!(retained_instance(&clone).unwrap(), instance);
    assert_eq!(
        service.record_detail(ACTOR).unwrap().record.title,
        "Original"
    );

    let independent = command(&artifact, AppServiceRetrievalMode::OnDemandNoEmbeddings);
    let fresh = independent.record_detail(ACTOR).unwrap();
    assert_eq!(fresh.record.title, "Replacement");
    assert_ne!(fresh.artifact_generation, root.artifact_generation);
    assert_ne!(retained_instance(&independent).unwrap(), instance);
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
