use atlas_app_model::{
    AddEncounterParticipantConditionRequest, AddEncounterRecordParticipantRequest, AppErrorCode,
    CreateEncounterRequest, EncounterParticipantStatusView, EncounterStatusView,
    ReorderEncounterParticipantPlacementView, ReorderEncounterParticipantRequest,
    SetEncounterTurnRequest, UpdateEncounterParticipantConditionRequest,
    UpdateEncounterParticipantRequest, UpdateEncounterRequest,
};
use atlas_domain::RecordKey;
use atlas_local_state::{
    AddEncounterParticipant, EncounterParticipant, ParticipantKind, ParticipantSide,
};

use crate::test_support::{encounter_fixture_worker, fixture_worker};

use super::projection::{participant_side_view, participant_variant_view};

#[test]
fn set_encounter_turn_starts_advances_and_wraps_rounds() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Turn Test".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let store = fixture
        .worker
        .local_state_store()
        .expect("local state should open");
    store
        .encounters()
        .add_participant(&encounter.slug, pc("Unset", None))
        .expect("unset participant should add");
    let high = store
        .encounters()
        .add_participant(&encounter.slug, pc("High", Some(20)))
        .expect("high participant should add");
    let low = store
        .encounters()
        .add_participant(&encounter.slug, pc("Low", Some(10)))
        .expect("low participant should add");

    let started = fixture
        .worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: encounter.slug.clone(),
            participant_key: None,
        })
        .expect("encounter should start");
    let advanced = fixture
        .worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: encounter.slug.clone(),
            participant_key: None,
        })
        .expect("encounter should advance");
    let wrapped = fixture
        .worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: encounter.slug,
            participant_key: None,
        })
        .expect("encounter should wrap");

    assert_eq!(
        started.current_turn_participant_key.as_deref(),
        Some(high.participant_key.as_str())
    );
    assert_eq!(started.encounter.round_number, 1);
    assert_eq!(
        advanced.current_turn_participant_key.as_deref(),
        Some(low.participant_key.as_str())
    );
    assert_eq!(advanced.encounter.round_number, 1);
    assert_eq!(
        wrapped.current_turn_participant_key.as_deref(),
        Some(high.participant_key.as_str())
    );
    assert_eq!(wrapped.encounter.round_number, 2);
}

#[test]
fn set_encounter_turn_falls_back_to_pcs_when_everyone_is_defeated() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Defeated Test".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let store = fixture
        .worker
        .local_state_store()
        .expect("local state should open");
    let enemy = store
        .encounters()
        .add_participant(&encounter.slug, creature("Enemy", Some(18)))
        .expect("enemy participant should add");
    let pc = store
        .encounters()
        .add_participant(&encounter.slug, pc("PC", Some(12)))
        .expect("pc participant should add");
    fixture
        .worker
        .update_encounter_participant(&encounter.slug, participant_update(&enemy, true))
        .expect("enemy should update");
    fixture
        .worker
        .update_encounter_participant(&encounter.slug, participant_update(&pc, true))
        .expect("pc should update");

    let started = fixture
        .worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: encounter.slug,
            participant_key: None,
        })
        .expect("encounter should still start on pc");

    assert_eq!(
        started.current_turn_participant_key.as_deref(),
        Some(pc.participant_key.as_str())
    );
}

#[test]
fn removing_current_turn_advances_to_next_participant() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Remove Current".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let store = fixture
        .worker
        .local_state_store()
        .expect("local state should open");
    let first = store
        .encounters()
        .add_participant(&encounter.slug, pc("First", Some(20)))
        .expect("first should add");
    let second = store
        .encounters()
        .add_participant(&encounter.slug, pc("Second", Some(10)))
        .expect("second should add");
    fixture
        .worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: encounter.slug.clone(),
            participant_key: Some(first.participant_key.clone()),
        })
        .expect("turn should set");

    let detail = fixture
        .worker
        .remove_encounter_participant(&encounter.slug, &first.participant_key)
        .expect("participant should remove");

    assert_eq!(
        detail.current_turn_participant_key.as_deref(),
        Some(second.participant_key.as_str())
    );
}

#[test]
fn explicit_turn_selection_does_not_change_encounter_status() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Manual Turn".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let store = fixture
        .worker
        .local_state_store()
        .expect("local state should open");
    let participant = store
        .encounters()
        .add_participant(&encounter.slug, pc("Hero", Some(20)))
        .expect("participant should add");
    fixture
        .worker
        .update_encounter(UpdateEncounterRequest {
            encounter_key: encounter.encounter_key,
            slug: encounter.slug.clone(),
            name: encounter.name,
            description: encounter.description,
            note: None,
            status: EncounterStatusView::Complete,
        })
        .expect("encounter status should update");

    let detail = fixture
        .worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: encounter.slug,
            participant_key: Some(participant.participant_key.clone()),
        })
        .expect("turn should set");

    assert_eq!(detail.encounter.status, EncounterStatusView::Complete);
    assert_eq!(
        detail.current_turn_participant_key.as_deref(),
        Some(participant.participant_key.as_str())
    );
}

#[test]
fn zero_hp_participant_can_be_marked_active() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Active At Zero".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let participant = fixture
        .worker
        .local_state_store()
        .expect("local state should open")
        .encounters()
        .add_participant(&encounter.slug, pc("Persistent Hero", Some(20)))
        .expect("participant should add");
    let mut update = participant_update(&participant, false);
    update.current_hp = Some(0);

    let updated = fixture
        .worker
        .update_encounter_participant(&encounter.slug, update)
        .expect("participant should update");

    assert_eq!(updated.current_hp, Some(0));
    assert!(!updated.defeated);
}

#[test]
fn record_participant_add_rejects_invalid_quantity_without_mutating() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Quantity Test".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;

    for quantity in [0, 51] {
        let error = fixture
            .worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: encounter.slug.clone(),
                record_ref: "actions:testAction1".to_string(),
                quantity,
                initiative: None,
            })
            .expect_err("invalid quantity should fail");

        assert!(error.into_app_error().message.contains("quantity must be"));
    }

    let detail = fixture
        .worker
        .encounter(&encounter.slug)
        .expect("encounter should remain readable");
    assert!(detail.participants.is_empty());
}

#[test]
fn record_participant_add_rejects_unsupported_record_kind() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Unsupported Kind".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;

    let error = fixture
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: "actions:testAction1".to_string(),
            quantity: 1,
            initiative: None,
        })
        .expect_err("actions are not encounter participants");

    assert!(
        error
            .into_app_error()
            .message
            .contains("encounter participants must be creatures")
    );
    let detail = fixture
        .worker
        .encounter(&encounter.slug)
        .expect("encounter should remain readable");
    assert!(detail.participants.is_empty());
}

#[test]
fn record_participant_add_hydrates_creature_instances_and_hazard_defaults() {
    let fixture = encounter_fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Record-backed".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;

    let creature_detail = fixture
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:testCreature".to_string(),
            quantity: 2,
            initiative: Some(18),
        })
        .expect("creatures should add");
    assert_eq!(creature_detail.participants.len(), 2);
    assert_eq!(
        creature_detail
            .participants
            .iter()
            .map(|participant| participant.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["Test Creature 1", "Test Creature 2"]
    );
    for participant in &creature_detail.participants {
        assert_eq!(
            participant.record_key.as_deref(),
            Some("actors:testCreature")
        );
        assert_eq!(participant.max_hp, Some(25));
        assert_eq!(participant.current_hp, Some(17));
        assert_eq!(
            participant.side,
            atlas_app_model::EncounterParticipantSideView::Enemy
        );
        assert!(participant.record.is_some());
    }

    let hazard_detail = fixture
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug,
            record_ref: "hazards:testHazard".to_string(),
            quantity: 1,
            initiative: None,
        })
        .expect("hazard should add");
    let hazard = hazard_detail
        .participants
        .iter()
        .find(|participant| participant.record_key.as_deref() == Some("hazards:testHazard"))
        .expect("hazard participant should exist");
    assert_eq!(hazard.display_name, "Test Hazard");
    assert_eq!(
        hazard.side,
        atlas_app_model::EncounterParticipantSideView::Hazard
    );
    assert_eq!(hazard.max_hp, Some(30));
    assert_eq!(hazard.current_hp, Some(30));
    assert!(hazard.record.is_some());
}

#[test]
fn condition_add_resolves_condition_records_and_rejects_other_records() {
    let fixture = encounter_fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Resolved Conditions".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let participant = fixture
        .worker
        .local_state_store()
        .expect("local state should open")
        .encounters()
        .add_participant(&encounter.slug, pc("Hero", Some(20)))
        .expect("participant should add");

    let with_condition = fixture
        .worker
        .add_encounter_participant_condition(
            &encounter.slug,
            AddEncounterParticipantConditionRequest {
                participant_key: participant.participant_key.clone(),
                condition_ref: Some("conditionitems:testCondition".to_string()),
                name: None,
                value: Some(1),
                source_participant_key: None,
                duration_rounds: Some(2),
                note: None,
                source_note: None,
            },
        )
        .expect("condition record should add");
    let condition = &with_condition.participants[0].conditions[0];
    assert_eq!(
        condition.condition_key.as_deref(),
        Some("conditionitems:testCondition")
    );
    assert_eq!(condition.name, "Test Condition");
    assert_eq!(condition.value, Some(1));

    let error = fixture
        .worker
        .add_encounter_participant_condition(
            &encounter.slug,
            AddEncounterParticipantConditionRequest {
                participant_key: participant.participant_key,
                condition_ref: Some("actors:testCreature".to_string()),
                name: None,
                value: None,
                source_participant_key: None,
                duration_rounds: None,
                note: None,
                source_note: None,
            },
        )
        .expect_err("non-condition record should reject");

    assert!(error.into_app_error().message.contains("condition records"));
}

#[test]
fn condition_update_preserves_and_replaces_resolved_condition_keys() {
    let fixture = encounter_fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Condition Updates".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let participant = fixture
        .worker
        .local_state_store()
        .expect("local state should open")
        .encounters()
        .add_participant(&encounter.slug, pc("Hero", Some(20)))
        .expect("participant should add");

    let added = fixture
        .worker
        .add_encounter_participant_condition(
            &encounter.slug,
            AddEncounterParticipantConditionRequest {
                participant_key: participant.participant_key.clone(),
                condition_ref: Some("conditionitems:testCondition".to_string()),
                name: None,
                value: Some(1),
                source_participant_key: None,
                duration_rounds: Some(2),
                note: None,
                source_note: None,
            },
        )
        .expect("record-backed condition should add");
    let condition = &added.participants[0].conditions[0];
    let condition_id = condition.condition_id;
    assert_eq!(
        condition.condition_key.as_deref(),
        Some("conditionitems:testCondition")
    );

    let manual_update = fixture
        .worker
        .update_encounter_participant_condition(
            &encounter.slug,
            &participant.participant_key,
            UpdateEncounterParticipantConditionRequest {
                condition_id,
                condition_ref: None,
                name: "Renamed Condition".to_string(),
                value: Some(2),
                source_participant_key: None,
                duration_rounds: Some(1),
                note: None,
                source_note: None,
            },
        )
        .expect("manual condition update should preserve key");
    let condition = &manual_update.participants[0].conditions[0];
    assert_eq!(condition.name, "Renamed Condition");
    assert_eq!(
        condition.condition_key.as_deref(),
        Some("conditionitems:testCondition")
    );

    let resolved_update = fixture
        .worker
        .update_encounter_participant_condition(
            &encounter.slug,
            &participant.participant_key,
            UpdateEncounterParticipantConditionRequest {
                condition_id,
                condition_ref: Some("conditionitems:testCondition".to_string()),
                name: "Ignored Name".to_string(),
                value: Some(3),
                source_participant_key: None,
                duration_rounds: Some(4),
                note: None,
                source_note: None,
            },
        )
        .expect("condition ref update should resolve stored key and name");
    let condition = &resolved_update.participants[0].conditions[0];
    assert_eq!(condition.name, "Test Condition");
    assert_eq!(
        condition.condition_key.as_deref(),
        Some("conditionitems:testCondition")
    );

    let error = fixture
        .worker
        .update_encounter_participant_condition(
            &encounter.slug,
            &participant.participant_key,
            UpdateEncounterParticipantConditionRequest {
                condition_id,
                condition_ref: Some("actors:testCreature".to_string()),
                name: "Invalid".to_string(),
                value: None,
                source_participant_key: None,
                duration_rounds: None,
                note: None,
                source_note: None,
            },
        )
        .expect_err("non-condition ref should reject on update");
    assert!(error.into_app_error().message.contains("condition records"));
}

#[test]
fn unresolved_record_backed_participant_preserves_stored_state() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Unresolved Creature".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    fixture
        .worker
        .local_state_store()
        .expect("local state should open")
        .encounters()
        .add_participant(&encounter.slug, creature("Missing Creature", Some(16)))
        .expect("participant should add");

    let detail = fixture
        .worker
        .encounter(&encounter.slug)
        .expect("encounter should load unresolved participant");
    let participant = &detail.participants[0];

    assert_eq!(participant.display_name, "Missing Creature");
    assert_eq!(
        participant.record_key.as_deref(),
        Some("actors:testCreature")
    );
    assert_eq!(
        participant.status,
        EncounterParticipantStatusView::Unresolved
    );
    assert!(participant.record.is_none());
}

#[test]
fn encounter_conditions_and_reorder_route_through_app_service() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Condition Test".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let store = fixture
        .worker
        .local_state_store()
        .expect("local state should open");
    let high = store
        .encounters()
        .add_participant(&encounter.slug, pc("High", Some(20)))
        .expect("high should add");
    let low = store
        .encounters()
        .add_participant(&encounter.slug, pc("Low", Some(10)))
        .expect("low should add");

    let reordered = fixture
        .worker
        .reorder_encounter_participant(
            &encounter.slug,
            ReorderEncounterParticipantRequest {
                participant_key: high.participant_key.clone(),
                target_participant_key: low.participant_key.clone(),
                placement: ReorderEncounterParticipantPlacementView::After,
            },
        )
        .expect("reorder should succeed");
    assert_eq!(
        reordered.participants[0].participant_key,
        low.participant_key
    );
    assert_eq!(
        reordered.participants[1].participant_key,
        high.participant_key
    );

    let added = fixture
        .worker
        .add_encounter_participant_condition(
            &encounter.slug,
            AddEncounterParticipantConditionRequest {
                participant_key: high.participant_key.clone(),
                condition_ref: None,
                name: Some("Frightened".to_string()),
                value: Some(1),
                source_participant_key: Some(low.participant_key.clone()),
                duration_rounds: Some(2),
                note: Some("spell".to_string()),
                source_note: Some("source text".to_string()),
            },
        )
        .expect("condition should add");
    let condition = &added.participants[1].conditions[0];
    assert_eq!(condition.name, "Frightened");
    assert_eq!(
        condition.source_participant_key.as_deref(),
        Some(low.participant_key.as_str())
    );

    let updated = fixture
        .worker
        .update_encounter_participant_condition(
            &encounter.slug,
            &high.participant_key,
            UpdateEncounterParticipantConditionRequest {
                condition_id: condition.condition_id,
                condition_ref: None,
                name: "Frightened".to_string(),
                value: Some(2),
                source_participant_key: None,
                duration_rounds: Some(1),
                note: None,
                source_note: None,
            },
        )
        .expect("condition should update");
    assert_eq!(updated.participants[1].conditions[0].value, Some(2));

    let removed = fixture
        .worker
        .remove_encounter_participant_condition(
            &encounter.slug,
            &high.participant_key,
            condition.condition_id,
        )
        .expect("condition should remove");
    assert!(removed.participants[1].conditions.is_empty());
}

#[test]
fn condition_update_and_delete_reject_wrong_participant_without_mutating() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Condition Ownership".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let store = fixture
        .worker
        .local_state_store()
        .expect("local state should open");
    let owner = store
        .encounters()
        .add_participant(&encounter.slug, pc("Owner", Some(20)))
        .expect("owner should add");
    let other = store
        .encounters()
        .add_participant(&encounter.slug, pc("Other", Some(10)))
        .expect("other should add");
    let added = fixture
        .worker
        .add_encounter_participant_condition(
            &encounter.slug,
            AddEncounterParticipantConditionRequest {
                participant_key: owner.participant_key.clone(),
                condition_ref: None,
                name: Some("Frightened".to_string()),
                value: Some(1),
                source_participant_key: None,
                duration_rounds: Some(2),
                note: Some("original".to_string()),
                source_note: None,
            },
        )
        .expect("condition should add");
    let condition_id = added.participants[0].conditions[0].condition_id;

    let update_error = fixture
        .worker
        .update_encounter_participant_condition(
            &encounter.slug,
            &other.participant_key,
            UpdateEncounterParticipantConditionRequest {
                condition_id,
                condition_ref: None,
                name: "Changed".to_string(),
                value: Some(3),
                source_participant_key: None,
                duration_rounds: None,
                note: None,
                source_note: None,
            },
        )
        .expect_err("wrong participant update should reject")
        .into_app_error();
    assert_eq!(
        update_error.code,
        AppErrorCode::EncounterParticipantNotFound
    );

    let delete_error = fixture
        .worker
        .remove_encounter_participant_condition(
            &encounter.slug,
            &other.participant_key,
            condition_id,
        )
        .expect_err("wrong participant delete should reject")
        .into_app_error();
    assert_eq!(
        delete_error.code,
        AppErrorCode::EncounterParticipantNotFound
    );

    let detail = fixture
        .worker
        .encounter(&encounter.slug)
        .expect("encounter should reload");
    let owner = detail
        .participants
        .iter()
        .find(|participant| participant.participant_key == owner.participant_key)
        .expect("owner should remain");
    assert_eq!(owner.conditions.len(), 1);
    assert_eq!(owner.conditions[0].name, "Frightened");
    assert_eq!(owner.conditions[0].value, Some(1));
    assert_eq!(owner.conditions[0].note.as_deref(), Some("original"));
}

fn pc(name: &str, initiative: Option<i64>) -> AddEncounterParticipant {
    AddEncounterParticipant {
        record_key: None,
        participant_kind: ParticipantKind::Pc,
        display_name: name.to_string(),
        record_title_snapshot: None,
        record_kind_snapshot: None,
        side: ParticipantSide::Pc,
        initiative,
        max_hp: Some(10),
        current_hp: Some(10),
        temporary_hp: 0,
        note: None,
    }
}

fn creature(name: &str, initiative: Option<i64>) -> AddEncounterParticipant {
    AddEncounterParticipant {
        record_key: Some(RecordKey::parse("actors:testCreature").expect("key should parse")),
        participant_kind: ParticipantKind::Creature,
        display_name: name.to_string(),
        record_title_snapshot: Some(name.to_string()),
        record_kind_snapshot: Some("creature".to_string()),
        side: ParticipantSide::Enemy,
        initiative,
        max_hp: Some(10),
        current_hp: Some(10),
        temporary_hp: 0,
        note: None,
    }
}

fn participant_update(
    participant: &EncounterParticipant,
    defeated: bool,
) -> UpdateEncounterParticipantRequest {
    UpdateEncounterParticipantRequest {
        participant_key: participant.participant_key.clone(),
        display_name: participant.display_name.clone(),
        side: participant_side_view(participant.side),
        participant_variant: participant_variant_view(participant.participant_variant),
        initiative: participant.initiative,
        max_hp: participant.max_hp,
        current_hp: participant.current_hp,
        temporary_hp: participant.temporary_hp,
        defeated,
        hidden: participant.hidden,
        note: participant.note.clone(),
    }
}
