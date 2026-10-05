use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use atlas_app_model::{
    AddEncounterParticipantConditionRequest, AddEncounterRecordParticipantRequest, AppError,
    AppErrorCode, CreateEncounterRequest, EncounterDetailView,
    EncounterParticipantResetConfirmationView, EncounterParticipantResetUnavailableReasonView,
    EncounterParticipantSideView, EncounterParticipantVariantView, EncounterParticipantView,
    EncounterRuntimeSpellView, EncounterSpellCastBlockedReasonView,
    EncounterSpellCastOperationView, EncounterSpellCastRequest, EncounterSpellCastStateView,
    EncounterSpellCastUnavailableReasonView, EncounterSpellSpendTargetView,
    ResetEncounterParticipantRequest, SetEncounterTurnRequest, UpdateEncounterParticipantRequest,
};
use atlas_domain::{PackName, RecordKey, RecordKind};
use atlas_embedding::EmbeddingModelId;
use atlas_index::{
    IndexArtifactWriter, IndexBuildInput, IndexBuildPack, SqliteIndexReader, SqliteIndexWriter,
};
use atlas_local_state::{
    AddEncounterParticipant, LocalStateStore, NewEncounter, ParticipantKind, ParticipantSide,
};
use atlas_record::{
    AtlasRecord, CreatureActionCost, CreatureCapability, CreatureComponentId,
    CreatureEmbeddedEntities, CreatureEntity, CreatureEntityFamily, CreatureEntityId,
    CreatureEntityOccurrence, CreatureEntitySourceIdentity, CreatureEntityTarget, CreatureFact,
    CreatureFamily, CreatureIdentity, CreatureOccurrenceContext, CreatureOccurrenceId,
    CreatureOccurrenceParent, CreaturePreparedSpellSlot, CreatureProvenance, CreatureRecord,
    CreatureResource, CreatureResourceAmount, CreatureResourceKind, CreatureSourceField,
    CreatureSourceId, CreatureSourceScalar, CreatureSpellPreparation, FactValue,
    FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, OccurrenceIdentityStability,
    RecordBody, RecordClassification, RecordIdentity, RecordProvenance, ResourceCurrentPolicy,
};
use atlas_search::AtlasRetrievalService;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::executor::RetrievalExecutor;
use crate::test_support::{
    fixture_local_state_path, fixture_worker_with_executor,
    fixture_worker_with_executor_and_local_state_path,
};

const RECORD_KEY: &str = "actors:workflowSpellcaster";
const JS_SAFE_INTEGER_MAX: i64 = 9_007_199_254_740_991;

#[test]
fn record_backed_v7_spell_cast_reset_and_legacy_dto_workflow()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        spell_fixture_retrieval_service,
    ));
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Record-backed spell workflow".to_string(),
            description: None,
            note: None,
        })?
        .encounter;
    let created =
        fixture
            .worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: encounter.slug.clone(),
                record_ref: RECORD_KEY.to_string(),
                quantity: 1,
                initiative: Some(20),
            })?;
    let created_participant = created
        .participants
        .first()
        .expect("record-backed participant")
        .clone();
    assert!(
        created_participant.reset.available,
        "v7 creation must capture baseline"
    );
    assert_eq!(created_participant.record_key.as_deref(), Some(RECORD_KEY));
    let prepared_target = spell(&created_participant, "prepared-spell")
        .cast
        .spend_target
        .clone()
        .expect("prepared target");
    assert!(matches!(
        prepared_target,
        EncounterSpellSpendTargetView::PreparedSlot {
            ref entry_id,
            rank: 4,
            ref slot_id,
        } if entry_id == "prepared-entry" && slot_id == "slot4:0"
    ));
    let prepared_cast = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "prepared-spell",
        prepared_target.clone(),
        EncounterSpellCastOperationView::CastOne,
    )?;
    assert_remaining(&prepared_cast.after.state, 0);
    let prepared_reload = fixture.worker.encounter(&encounter.slug)?;
    let reloaded_prepared = spell(participant(&prepared_reload), "prepared-spell");
    assert_remaining(&reloaded_prepared.cast.state, 0);
    assert_eq!(
        reloaded_prepared.cast.blocked_reason,
        Some(EncounterSpellCastBlockedReasonView::Exhausted)
    );
    let exhausted_error = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "prepared-spell",
        prepared_target.clone(),
        EncounterSpellCastOperationView::CastOne,
    )
    .expect_err("prepared slot must exhaust")
    .into_app_error();
    assert_eq!(exhausted_error.code, AppErrorCode::InvalidRequest);
    let prepared_restore = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "prepared-spell",
        prepared_target.clone(),
        EncounterSpellCastOperationView::RestoreOne,
    )?;
    assert_remaining(&prepared_restore.after.state, 1);
    assert_remaining(
        &spell(
            participant(&fixture.worker.encounter(&encounter.slug)?),
            "prepared-spell",
        )
        .cast
        .state,
        1,
    );

    let spontaneous_target = spell(&created_participant, "spontaneous-spell")
        .cast
        .spend_target
        .clone()
        .expect("spontaneous target");
    let spontaneous_cast = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "spontaneous-spell",
        spontaneous_target.clone(),
        EncounterSpellCastOperationView::CastOne,
    )?;
    assert_remaining(&spontaneous_cast.after.state, 1);
    assert_remaining(
        &spell(
            participant(&fixture.worker.encounter(&encounter.slug)?),
            "spontaneous-spell",
        )
        .cast
        .state,
        1,
    );
    let spontaneous_restore = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "spontaneous-spell",
        spontaneous_target.clone(),
        EncounterSpellCastOperationView::RestoreOne,
    )?;
    assert_remaining(&spontaneous_restore.after.state, 2);
    assert_remaining(
        &spell(
            participant(&fixture.worker.encounter(&encounter.slug)?),
            "spontaneous-spell",
        )
        .cast
        .state,
        2,
    );
    cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "spontaneous-spell",
        spontaneous_target,
        EncounterSpellCastOperationView::CastOne,
    )?;

    let innate_target = spell(&created_participant, "innate-spell")
        .cast
        .spend_target
        .clone()
        .expect("innate target");
    let innate_cast = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "innate-spell",
        innate_target.clone(),
        EncounterSpellCastOperationView::CastOne,
    )?;
    assert_remaining(&innate_cast.after.state, 0);
    assert_remaining(
        &spell(
            participant(&fixture.worker.encounter(&encounter.slug)?),
            "innate-spell",
        )
        .cast
        .state,
        0,
    );
    let innate_restore = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "innate-spell",
        innate_target.clone(),
        EncounterSpellCastOperationView::RestoreOne,
    )?;
    assert_remaining(&innate_restore.after.state, 1);
    assert_remaining(
        &spell(
            participant(&fixture.worker.encounter(&encounter.slug)?),
            "innate-spell",
        )
        .cast
        .state,
        1,
    );
    cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "innate-spell",
        innate_target,
        EncounterSpellCastOperationView::CastOne,
    )?;

    let focus = spell(&created_participant, "focus-spell");
    assert!(matches!(
        focus.cast.state,
        EncounterSpellCastStateView::Unavailable {
            reason: EncounterSpellCastUnavailableReasonView::MissingCurrent
        }
    ));
    let focus_resource = runtime(&created_participant)
        .resources
        .iter()
        .find(|resource| resource.resource_id == "resource:focus")
        .expect("typed focus consumer retains its runtime resource");
    assert_eq!(focus_resource.maximum.adjusted_value, 3);
    assert!(focus_resource.current.is_none());
    let focus_error = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "focus-spell",
        focus.cast.spend_target.clone().expect("typed focus target"),
        EncounterSpellCastOperationView::CastOne,
    )
    .expect_err("provenance-only focus points are unavailable")
    .into_app_error();
    assert_eq!(focus_error.code, AppErrorCode::InvalidRequest);

    let at_will_target = spell(&created_participant, "at-will-spell")
        .cast
        .spend_target
        .clone()
        .expect("at-will target");
    assert_eq!(at_will_target, EncounterSpellSpendTargetView::AtWill);
    let at_will_cast = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "at-will-spell",
        at_will_target.clone(),
        EncounterSpellCastOperationView::CastOne,
    )?;
    assert!(matches!(
        at_will_cast.after.state,
        EncounterSpellCastStateView::AtWill
    ));
    assert!(
        spell(
            participant(&fixture.worker.encounter(&encounter.slug)?),
            "at-will-spell"
        )
        .cast
        .available
    );
    let at_will_restore_error = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "at-will-spell",
        at_will_target,
        EncounterSpellCastOperationView::RestoreOne,
    )
    .expect_err("at-will has no mutable use to restore")
    .into_app_error();
    assert_eq!(at_will_restore_error.code, AppErrorCode::InvalidRequest);

    let owner_mismatch_error = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "prepared-spell",
        EncounterSpellSpendTargetView::PreparedSlot {
            entry_id: "wrong-owner".to_string(),
            rank: 4,
            slot_id: "slot4:0".to_string(),
        },
        EncounterSpellCastOperationView::CastOne,
    )
    .expect_err("owner mismatch must fail closed")
    .into_app_error();
    assert_eq!(owner_mismatch_error.code, AppErrorCode::InvalidRequest);

    let unsafe_json = format!(
        r#"{{"spell_occurrence_id":"prepared-spell","spend_target":{{"target_type":"prepared_slot","entry_id":"prepared-entry","rank":{},"slot_id":"slot4:0"}},"operation":"cast_one"}}"#,
        JS_SAFE_INTEGER_MAX + 1
    );
    assert!(serde_json::from_str::<EncounterSpellCastRequest>(&unsafe_json).is_err());

    let active = participant(&fixture.worker.encounter(&encounter.slug)?).clone();
    fixture
        .worker
        .update_encounter_participant(&encounter.slug, participant_update(&active, true))?;
    let defeated_reload = fixture.worker.encounter(&encounter.slug)?;
    let defeated = participant(&defeated_reload);
    let defeated_at_will = spell(defeated, "at-will-spell");
    assert_eq!(
        defeated_at_will.cast.blocked_reason,
        Some(EncounterSpellCastBlockedReasonView::ParticipantDefeated)
    );
    let defeated_error = cast(
        &fixture.worker,
        &encounter.slug,
        &created_participant.participant_key,
        "at-will-spell",
        EncounterSpellSpendTargetView::AtWill,
        EncounterSpellCastOperationView::CastOne,
    )
    .expect_err("defeated participant must not cast")
    .into_app_error();
    assert_eq!(defeated_error.code, AppErrorCode::InvalidRequest);

    fixture.worker.add_encounter_participant_condition(
        &encounter.slug,
        AddEncounterParticipantConditionRequest {
            participant_key: created_participant.participant_key.clone(),
            condition_ref: None,
            name: Some("Slowed".to_string()),
            value: Some(1),
            source_participant_key: None,
            duration_rounds: Some(2),
            note: None,
        },
    )?;
    fixture.worker.set_encounter_turn(SetEncounterTurnRequest {
        encounter_ref: encounter.slug.clone(),
        participant_key: Some(created_participant.participant_key.clone()),
    })?;
    let mut reset_update = participant_update(
        participant(&fixture.worker.encounter(&encounter.slug)?),
        true,
    );
    reset_update.display_name = "Custom spellcaster".to_string();
    reset_update.side = EncounterParticipantSideView::Ally;
    reset_update.participant_variant = EncounterParticipantVariantView::Elite;
    reset_update.initiative = Some(7);
    reset_update.max_hp = Some(45);
    reset_update.current_hp = Some(0);
    reset_update.temporary_hp = 5;
    reset_update.hidden = true;
    reset_update.note = Some("Preserve authored note".to_string());
    let reset_before = fixture
        .worker
        .update_encounter_participant(&encounter.slug, reset_update)?;
    let reset_result = fixture.worker.reset_encounter_participant(
        &encounter.slug,
        &created_participant.participant_key,
        ResetEncounterParticipantRequest {
            confirmation: EncounterParticipantResetConfirmationView::ResetParticipant,
        },
    )?;
    let reset_reload = fixture.worker.encounter(&encounter.slug)?;
    let reset_participant = participant(&reset_reload);
    assert_eq!(reset_reload.current_turn_participant_key, None);
    assert_eq!(reset_participant.display_name, "Custom spellcaster");
    assert_eq!(
        reset_participant.note.as_deref(),
        Some("Preserve authored note")
    );
    assert!(reset_participant.hidden);
    assert_eq!(reset_participant.side, EncounterParticipantSideView::Ally);
    assert_eq!(
        reset_participant.participant_variant,
        EncounterParticipantVariantView::Normal
    );
    assert_eq!(reset_participant.initiative, Some(20));
    assert!(!reset_participant.defeated);
    assert_remaining(&spell(reset_participant, "spontaneous-spell").cast.state, 2);
    assert_remaining(&spell(reset_participant, "innate-spell").cast.state, 1);
    assert!(runtime(reset_participant).conditions.is_empty());

    let legacy = migrated_v6_public_detail()?;
    let legacy_participant = participant(&legacy);
    assert!(!legacy_participant.reset.available);
    assert_eq!(
        legacy_participant.reset.unavailable_reason,
        Some(EncounterParticipantResetUnavailableReasonView::MissingCreationBaseline)
    );

    write_candidate_samples(
        &prepared_cast,
        &prepared_reload,
        &prepared_restore,
        &spontaneous_cast,
        &spontaneous_restore,
        &innate_cast,
        &innate_restore,
        &at_will_cast,
        &[
            exhausted_error,
            focus_error,
            at_will_restore_error,
            owner_mismatch_error,
            defeated_error,
        ],
        &reset_before,
        &reset_result,
        &reset_reload,
        &legacy,
    )?;
    Ok(())
}

fn migrated_v6_public_detail() -> Result<EncounterDetailView, Box<dyn std::error::Error>> {
    let path = fixture_local_state_path();
    let participant_key;
    {
        let store = LocalStateStore::open(&path)?;
        store.encounters().create(NewEncounter {
            slug: "legacy-v6".to_string(),
            name: "Legacy v6".to_string(),
            description: None,
            note: None,
        })?;
        participant_key = store
            .encounters()
            .add_participant(
                "legacy-v6",
                AddEncounterParticipant {
                    record_key: None,
                    participant_kind: ParticipantKind::Pc,
                    display_name: "Legacy participant".to_string(),
                    record_title_snapshot: None,
                    record_kind_snapshot: None,
                    side: ParticipantSide::Pc,
                    initiative: Some(12),
                    max_hp: Some(20),
                    current_hp: Some(8),
                    temporary_hp: 0,
                    note: None,
                },
            )?
            .participant_key;
    }
    let connection = rusqlite::Connection::open(&path)?;
    connection.execute_batch(
        "DROP TABLE encounter_participant_spell_resources;
         DROP TABLE encounter_participant_spell_state;
         DROP TABLE encounter_participant_baselines;
         UPDATE local_state_metadata SET value = '6' WHERE key = 'schema_version';",
    )?;
    drop(connection);
    let fixture = fixture_worker_with_executor_and_local_state_path(
        RetrievalExecutor::from_test_fixture_factory(1, 16, spell_fixture_retrieval_service),
        path,
    );
    let detail = fixture.worker.encounter("legacy-v6")?;
    assert_eq!(participant(&detail).participant_key, participant_key);
    Ok(detail)
}

fn cast(
    service: &crate::AtlasAppService,
    encounter_ref: &str,
    participant_key: &str,
    spell_occurrence_id: &str,
    spend_target: EncounterSpellSpendTargetView,
    operation: EncounterSpellCastOperationView,
) -> crate::AppServiceResult<atlas_app_model::EncounterSpellCastResultView> {
    service.mutate_encounter_spell_cast(
        encounter_ref,
        participant_key,
        EncounterSpellCastRequest {
            spell_occurrence_id: spell_occurrence_id.to_string(),
            spend_target,
            operation,
        },
    )
}

fn participant(detail: &EncounterDetailView) -> &EncounterParticipantView {
    detail.participants.first().expect("participant")
}

fn runtime(participant: &EncounterParticipantView) -> &atlas_app_model::EncounterRuntimeView {
    participant
        .record_view
        .encounter
        .as_ref()
        .expect("record-backed runtime")
}

fn spell<'a>(
    participant: &'a EncounterParticipantView,
    occurrence_id: &str,
) -> &'a EncounterRuntimeSpellView {
    let runtime = runtime(participant);
    runtime
        .spellcasting
        .iter()
        .flat_map(|entry| &entry.spells)
        .chain(runtime.standalone_spells.iter())
        .find(|spell| spell.occurrence_id == occurrence_id)
        .unwrap_or_else(|| panic!("spell {occurrence_id}"))
}

fn assert_remaining(state: &EncounterSpellCastStateView, expected: i64) {
    assert!(matches!(
        state,
        EncounterSpellCastStateView::Tracked { remaining, .. } if *remaining == expected
    ));
}

fn participant_update(
    participant: &EncounterParticipantView,
    defeated: bool,
) -> UpdateEncounterParticipantRequest {
    UpdateEncounterParticipantRequest {
        participant_key: participant.participant_key.clone(),
        display_name: participant.display_name.clone(),
        side: participant.side,
        participant_variant: participant.participant_variant,
        hazard_state: None,
        initiative: participant.initiative,
        max_hp: Some(30),
        current_hp: Some(if defeated { 0 } else { 30 }),
        temporary_hp: 0,
        defeated,
        hidden: participant.hidden,
        note: participant.note.clone(),
    }
}

struct SpellFixtureArtifact {
    root: PathBuf,
}

impl Drop for SpellFixtureArtifact {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn spell_fixture_retrieval_service()
-> Result<(AtlasRetrievalService, SpellFixtureArtifact), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!(
        "atlas-app-service-spell-workflow-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir_all(&root)?;
    let artifact_path = root.join("pf2e-index.sqlite");
    let (record, body) = spell_fixture_record();
    let input = IndexBuildInput {
        source_signature: "foundry-pf2e:app-service-spell-workflow-fixture".to_string(),
        source_record_count: 1,
        packs: vec![IndexBuildPack {
            name: PackName::new("actors")?,
            label: "Actors".to_string(),
            document_type: "Actor".to_string(),
            declared_path: "packs/actors".to_string(),
            resolved_path: root.join("packs/actors"),
            record_count: 1,
        }],
        records: vec![record],
        canonical_bodies: vec![body],
        canonical_spell_children: Vec::new(),
        consumable_occurrence_sets: Vec::new(),
        references: Vec::new(),
        aliases: Vec::new(),
        remaster_links: Vec::new(),
        pending_document_embeddings: Vec::new(),
        document_embeddings: Vec::new(),
    };
    SqliteIndexWriter::new(artifact_path.clone()).write(&input, EmbeddingModelId::BgeSmallEnV15)?;
    atlas_index::test_support::write_bound_test_manifest(&artifact_path)?;
    let reader = SqliteIndexReader::open_read_only(&artifact_path)?;
    Ok((
        AtlasRetrievalService::from_prepared_index_without_embeddings(reader),
        SpellFixtureArtifact { root },
    ))
}

fn spell_fixture_record() -> (AtlasRecord, RecordBody) {
    let mut creature = creature();
    let owner = creature.identity.record_key.clone();
    let mut occurrences = vec![
        entry(
            &owner,
            "prepared-entry",
            0,
            CreatureSpellPreparation::Prepared,
            FactValue::Value(vec![atlas_record::CreatureSpellSlot {
                rank: 4,
                maximum: scalar(1),
                serialized_value: scalar(1),
                prepared: FactValue::Value(vec![CreaturePreparedSpellSlot::Spell {
                    id: FactValue::Value(source_id("prepared-spell")),
                    name: FactValue::Value("Prepared Bolt".to_string()),
                    expended: FactValue::Value(false),
                    prepared: FactValue::Value(true),
                    authored_order: 0,
                }]),
            }]),
        ),
        entry(
            &owner,
            "spontaneous-entry",
            1,
            CreatureSpellPreparation::Spontaneous,
            FactValue::Value(vec![atlas_record::CreatureSpellSlot {
                rank: 3,
                maximum: scalar(3),
                serialized_value: scalar(2),
                prepared: FactValue::Missing,
            }]),
        ),
        entry(
            &owner,
            "innate-entry",
            2,
            CreatureSpellPreparation::Innate,
            FactValue::Value(Vec::new()),
        ),
        entry(
            &owner,
            "focus-entry",
            3,
            CreatureSpellPreparation::Focus,
            FactValue::Value(Vec::new()),
        ),
    ];
    let mut prepared = spell_occurrence(&owner, "prepared-spell", "prepared-entry", 4, 4);
    prepared.context.slot = FactValue::Value("slot4:0".to_string());
    let spontaneous = spell_occurrence(&owner, "spontaneous-spell", "spontaneous-entry", 3, 5);
    let mut innate = spell_occurrence(&owner, "innate-spell", "innate-entry", 5, 6);
    innate.context.uses = FactValue::Value(atlas_record::CreatureUseLimit {
        maximum: FactValue::Value(2),
        serialized_value: FactValue::Value(1),
    });
    let at_will = spell_occurrence(&owner, "at-will-spell", "innate-entry", 2, 7);
    let focus = spell_occurrence(&owner, "focus-spell", "focus-entry", 1, 8);
    occurrences.extend([prepared, spontaneous, innate, at_will, focus]);
    let entities = occurrences
        .iter()
        .map(|occurrence| CreatureEntity {
            id: match &occurrence.target {
                CreatureEntityTarget::ActorOwned(id) => id.clone(),
                CreatureEntityTarget::CanonicalRecord(_) => unreachable!("fixture is actor-owned"),
            },
            family: occurrence.family,
            label: label(occurrence.id.as_str()).to_string(),
            source_identity: occurrence.source_identity.clone(),
        })
        .collect();
    creature.resources = CreatureFact::source(
        FactValue::Value(vec![CreatureResource {
            id: CreatureComponentId::new("resource:focus").expect("focus id"),
            authored_order: 0,
            kind: CreatureResourceKind::new("focus").expect("focus kind"),
            label: "Focus".to_string(),
            maximum: FactValue::Value(CreatureResourceAmount::Integer(3)),
            serialized_value: FactValue::Value(CreatureResourceAmount::Integer(1)),
            source_drift: FactValue::Missing,
            current_policy: ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
        }]),
        CreatureSourceField::Resources,
    );
    creature.embedded_entities = CreatureFact::source(
        FactValue::Value(CreatureEmbeddedEntities {
            entities,
            occurrences,
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        }),
        CreatureSourceField::EmbeddedEntities,
    );
    let body = RecordBody::Creature(creature.clone());
    let mut record = AtlasRecord::new(
        RecordIdentity::new(owner, "Workflow Spellcaster"),
        RecordClassification::new(RecordKind::Creature),
        FoundryRecordInfo::new("Actors", FoundryDocumentType::Actor, FoundryRecordType::Npc),
        RecordProvenance::new("packs/actors/workflow-spellcaster.json"),
    );
    record.classification.level = Some(10);
    (record, body)
}

fn entry(
    owner: &RecordKey,
    id: &str,
    authored_order: u32,
    preparation: CreatureSpellPreparation,
    slots: FactValue<Vec<atlas_record::CreatureSpellSlot>>,
) -> CreatureEntityOccurrence {
    occurrence(
        owner,
        id,
        authored_order,
        CreatureEntityFamily::SpellcastingEntry,
        CreatureOccurrenceParent::Creature,
        CreatureCapability::SpellcastingEntry(atlas_record::CreatureSpellcastingEntryCapability {
            preparation: FactValue::Value(preparation),
            tradition: FactValue::Value("arcane".to_string()),
            attack: FactValue::Value(18),
            dc: FactValue::Value(28),
            slots,
            unsupported_notes: Vec::new(),
        }),
    )
}

fn spell_occurrence(
    owner: &RecordKey,
    id: &str,
    entry_id: &str,
    rank: i64,
    authored_order: u32,
) -> CreatureEntityOccurrence {
    let mut occurrence = occurrence(
        owner,
        id,
        authored_order,
        CreatureEntityFamily::Spell,
        CreatureOccurrenceParent::SpellcastingEntry(
            CreatureOccurrenceId::new(entry_id).expect("entry id"),
        ),
        CreatureCapability::Spell(atlas_record::CreatureSpellCapability {
            traits: FactValue::Value(vec!["spell".to_string()]),
            base_rank: FactValue::Value(rank),
            signature: FactValue::Missing,
            traditions: FactValue::Missing,
            requirements: FactValue::Missing,
            cost: FactValue::Missing,
            counteraction: FactValue::Missing,
            ritual: FactValue::Missing,
            target: FactValue::Missing,
            area: FactValue::Missing,
            range: FactValue::Missing,
            time: FactValue::Missing,
            duration: FactValue::Missing,
            defense: FactValue::Missing,
            damage: FactValue::Value(Vec::new()),
            action_cost: CreatureActionCost::Actions(2),
            unsupported_notes: Vec::new(),
        }),
    );
    occurrence.context.rank = FactValue::Value(rank);
    occurrence
}

fn occurrence(
    owner: &RecordKey,
    id: &str,
    authored_order: u32,
    family: CreatureEntityFamily,
    parent: CreatureOccurrenceParent,
    capability: CreatureCapability,
) -> CreatureEntityOccurrence {
    CreatureEntityOccurrence {
        id: CreatureOccurrenceId::new(id).expect("occurrence id"),
        identity_stability: OccurrenceIdentityStability::StableNestedSourceId,
        owner: owner.clone(),
        target: CreatureEntityTarget::ActorOwned(
            CreatureEntityId::new(format!("entity-{id}")).expect("entity id"),
        ),
        family,
        authored_order,
        source_sort: FactValue::Missing,
        source_folder: FactValue::Missing,
        source_identity: CreatureEntitySourceIdentity {
            nested_source_id: FactValue::Value(source_id(id)),
            stable_source_locator: FactValue::Missing,
            source_locators: Vec::new(),
        },
        parent,
        context: CreatureOccurrenceContext::default(),
        capability,
        deltas: Vec::new(),
    }
}

fn creature() -> CreatureRecord {
    macro_rules! missing {
        ($field:expr) => {
            CreatureFact::source(FactValue::Missing, $field)
        };
    }
    CreatureRecord {
        identity: CreatureIdentity {
            record_key: RecordKey::parse(RECORD_KEY).expect("record key"),
            source_id: source_id("workflow-spellcaster"),
            name: "Workflow Spellcaster".to_string(),
            family: CreatureFamily::Npc,
        },
        level: CreatureFact::source(FactValue::Value(10), CreatureSourceField::Level),
        rarity: missing!(CreatureSourceField::Rarity),
        traits: missing!(CreatureSourceField::Traits),
        size: missing!(CreatureSourceField::Size),
        publication: missing!(CreatureSourceField::Publication),
        adjustment: missing!(CreatureSourceField::Adjustment),
        source_alliance: missing!(CreatureSourceField::SourceAlliance),
        perception: missing!(CreatureSourceField::Perception),
        initiative: missing!(CreatureSourceField::Initiative),
        languages: missing!(CreatureSourceField::Languages),
        skills: missing!(CreatureSourceField::Skills),
        legacy_abilities: missing!(CreatureSourceField::LegacyAbilities),
        defenses: missing!(CreatureSourceField::Defenses),
        movement: missing!(CreatureSourceField::Movement),
        resources: missing!(CreatureSourceField::Resources),
        embedded_entities: missing!(CreatureSourceField::EmbeddedEntities),
        content: atlas_record::OwnedRichContent::default(),
        provenance: CreatureProvenance {
            source_path: "packs/actors/workflow-spellcaster.json".to_string(),
            source_contract_version: "fixture-v1".to_string(),
            source_system_version: "fixture".to_string(),
            source_upstream_commit: "fixture".to_string(),
        },
    }
}

fn label(id: &str) -> &'static str {
    match id {
        "prepared-entry" => "Prepared Spells",
        "spontaneous-entry" => "Spontaneous Spells",
        "innate-entry" => "Innate Spells",
        "focus-entry" => "Focus Spells",
        "prepared-spell" => "Prepared Bolt",
        "spontaneous-spell" => "Spontaneous Burst",
        "innate-spell" => "Innate Shadow",
        "at-will-spell" => "At-Will Spark",
        "focus-spell" => "Focus Ray",
        _ => "Fixture",
    }
}

fn scalar(value: i64) -> FactValue<CreatureSourceScalar<i64>> {
    FactValue::Value(CreatureSourceScalar::Value(value))
}

fn source_id(value: &str) -> CreatureSourceId {
    CreatureSourceId::new(value).expect("source id")
}

#[allow(clippy::too_many_arguments)]
fn write_candidate_samples(
    prepared_cast: &impl Serialize,
    prepared_reload: &impl Serialize,
    prepared_restore: &impl Serialize,
    spontaneous_cast: &impl Serialize,
    spontaneous_restore: &impl Serialize,
    innate_cast: &impl Serialize,
    innate_restore: &impl Serialize,
    at_will_cast: &impl Serialize,
    errors: &[AppError],
    reset_before: &impl Serialize,
    reset_result: &impl Serialize,
    reset_reload: &impl Serialize,
    legacy: &impl Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    let Ok(root) = std::env::var("ATLAS_F2_REMEDIATION_SAMPLE_ROOT") else {
        return Ok(());
    };
    let candidate = required_env("ATLAS_F2_REMEDIATION_CANDIDATE")?;
    let tree = required_env("ATLAS_F2_REMEDIATION_TREE")?;
    let parent = required_env("ATLAS_F2_REMEDIATION_PARENT")?;
    let root = PathBuf::from(root);
    if root.exists() {
        return Err(format!("sample root already exists: {}", root.display()).into());
    }
    fs::create_dir_all(&root)?;
    write_json(
        &root.join("app-service-spell-cast-workflow.json"),
        &serde_json::json!({
            "prepared": { "cast": prepared_cast, "reload": prepared_reload, "restore": prepared_restore },
            "spontaneous": { "cast": spontaneous_cast, "restore": spontaneous_restore },
            "innate": { "cast": innate_cast, "restore": innate_restore },
            "at_will_cast": at_will_cast,
            "expected_errors": errors,
        }),
    )?;
    write_json(
        &root.join("api-participant-reset-before.json"),
        reset_before,
    )?;
    write_json(
        &root.join("api-participant-reset-result.json"),
        reset_result,
    )?;
    write_json(
        &root.join("api-participant-reset-reload.json"),
        reset_reload,
    )?;
    write_json(&root.join("api-v6-reset-unavailable.json"), legacy)?;

    let mut hashes = BTreeMap::new();
    for name in [
        "app-service-spell-cast-workflow.json",
        "api-participant-reset-before.json",
        "api-participant-reset-result.json",
        "api-participant-reset-reload.json",
        "api-v6-reset-unavailable.json",
    ] {
        hashes.insert(name.to_string(), sha256_file(&root.join(name))?);
    }
    write_json(
        &root.join("manifest.json"),
        &serde_json::json!({
            "candidate_commit": candidate,
            "candidate_tree": tree,
            "direct_parent": parent,
            "source": "production atlas-index writer/reader -> AtlasRetrievalService -> AtlasAppService -> v7 SQLite -> public DTO",
            "files": hashes,
        }),
    )?;
    let manifest_hash = sha256_file(&root.join("manifest.json"))?;
    let mut checksum_lines = hashes
        .iter()
        .map(|(name, hash)| format!("{hash}  {name}"))
        .collect::<Vec<_>>();
    checksum_lines.push(format!("{manifest_hash}  manifest.json"));
    checksum_lines.sort();
    fs::write(
        root.join("checksums.sha256"),
        checksum_lines.join("\n") + "\n",
    )?;
    Ok(())
}

fn required_env(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    std::env::var(name).map_err(|_| format!("{name} is required when writing samples").into())
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut digest = Sha256::new();
    digest.update(fs::read(path)?);
    Ok(format!("{:x}", digest.finalize()))
}
