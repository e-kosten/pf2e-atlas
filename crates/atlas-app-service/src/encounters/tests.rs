use atlas_app_model::{
    AddEncounterParticipantConditionRequest, AddEncounterRecordParticipantRequest, AppErrorCode,
    CreateEncounterRequest, EncounterConditionApplicabilityView,
    EncounterConditionAutomationLevelView, EncounterConditionCategoryView,
    EncounterParticipantPreservedDomainView, EncounterParticipantResetConfirmationView,
    EncounterParticipantResetDomainView, EncounterParticipantStatusView,
    EncounterParticipantVariantView, EncounterParticipantView, EncounterRuntimeView,
    EncounterStatusView, RecordSurfaceEditionStatusView, RecordSurfacePresentationView,
    ReorderEncounterParticipantPlacementView, ReorderEncounterParticipantRequest,
    ResetEncounterParticipantRequest, SetEncounterTurnRequest,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest,
};
use atlas_domain::RecordKey;
use atlas_local_state::{
    AddEncounterParticipant, EncounterParticipant, ParticipantKind, ParticipantSide,
};

use crate::test_support::{encounter_fixture_worker, fixture_worker};

use super::projection::{participant_side_view, participant_variant_view};

fn runtime(participant: &EncounterParticipantView) -> &EncounterRuntimeView {
    participant
        .record_view
        .encounter
        .as_ref()
        .expect("encounter participant should carry the typed runtime bag")
}

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
    let pc_view = started
        .participants
        .iter()
        .find(|participant| participant.participant_key == pc.participant_key)
        .expect("pc should remain in encounter");
    let pc_stats = runtime(pc_view);
    assert!(pc_stats.defenses.is_none());
    assert!(pc_stats.movement.is_none());
    let action_budget = pc_stats
        .action_budget
        .as_ref()
        .expect("manual pc should have action budget");
    assert_eq!(action_budget.actions.adjusted_value, 3);
    assert_eq!(action_budget.reactions.adjusted_value, 1);
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

    assert_eq!(
        runtime(&updated)
            .vitals
            .as_ref()
            .and_then(|vitals| vitals.current_hp),
        Some(0)
    );
    assert!(!updated.defeated);
}

#[test]
fn defeated_participant_action_availability_tracks_reversible_state() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Defeated Availability".to_string(),
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
        .add_participant(&encounter.slug, pc("Reversible Hero", Some(20)))
        .expect("participant should add");

    let active = fixture
        .worker
        .encounter(&encounter.slug)
        .expect("encounter should read")
        .participants
        .into_iter()
        .find(|view| view.participant_key == participant.participant_key)
        .expect("participant should remain present");
    let active_budget = runtime(&active)
        .action_budget
        .as_ref()
        .expect("action budget")
        .clone();
    assert!(active_budget.can_act.available);
    assert!(active_budget.can_react.available);

    let defeated = fixture
        .worker
        .update_encounter_participant(&encounter.slug, participant_update(&participant, true))
        .expect("participant should become defeated");
    let defeated_budget = runtime(&defeated)
        .action_budget
        .as_ref()
        .expect("action budget");
    assert_eq!(defeated_budget.actions, active_budget.actions);
    assert_eq!(defeated_budget.reactions, active_budget.reactions);
    assert_eq!(defeated_budget.notes, active_budget.notes);
    for capability in [&defeated_budget.can_act, &defeated_budget.can_react] {
        assert!(!capability.available);
        assert!(matches!(
            capability
                .provenance
                .as_ref()
                .map(|provenance| &provenance.source),
            Some(atlas_app_model::RuntimeFactSourceView::ParticipantState)
        ));
        assert_eq!(
            capability.reason.as_deref(),
            Some("Defeated participants cannot act or react.")
        );
    }

    let local = local_participant(&fixture, &encounter.slug, &participant.participant_key);
    let reactivated = fixture
        .worker
        .update_encounter_participant(&encounter.slug, participant_update(&local, false))
        .expect("participant should become active");
    assert_eq!(
        runtime(&reactivated).action_budget.as_ref(),
        Some(&active_budget)
    );
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
fn draft_variant_change_fails_closed_when_canonical_level_is_absent() {
    let fixture = encounter_fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Variant Draft".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let detail = fixture
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:testCreature".to_string(),
            quantity: 1,
            initiative: Some(18),
        })
        .expect("creature should add");
    let participant = &detail.participants[0];
    let base_stats = runtime(participant);
    assert_eq!(base_stats.level, None);

    let mut update = participant_update(
        &local_participant(&fixture, &encounter.slug, &participant.participant_key),
        false,
    );
    update.participant_variant = EncounterParticipantVariantView::Elite;
    let updated = fixture
        .worker
        .update_encounter_participant(&encounter.slug, update)
        .expect("participant should update");
    assert_eq!(
        updated.participant_variant,
        EncounterParticipantVariantView::Elite
    );
    assert_eq!(
        runtime(&updated)
            .vitals
            .as_ref()
            .and_then(|vitals| vitals.current_hp),
        Some(17)
    );
    let stats = runtime(&updated);
    assert_eq!(stats.level, None);
    let ac = &stats.defenses.as_ref().expect("defenses").armor_class;
    assert_eq!(ac.adjusted_value, 21);
    let hp = stats
        .vitals
        .as_ref()
        .and_then(|vitals| vitals.maximum_hp.as_ref())
        .expect("hp should project");
    assert_eq!(hp.base_value, 17);
    assert_eq!(hp.adjusted_value, 17);
    assert!(hp.modifiers.is_empty());

    fixture
        .worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: encounter.slug.clone(),
            participant_key: None,
        })
        .expect("encounter should start");
    let mut running_update = participant_update(
        &local_participant(&fixture, &encounter.slug, &participant.participant_key),
        false,
    );
    running_update.participant_variant = EncounterParticipantVariantView::Weak;
    let running_updated = fixture
        .worker
        .update_encounter_participant(&encounter.slug, running_update)
        .expect("participant should update");
    assert_eq!(
        running_updated.participant_variant,
        EncounterParticipantVariantView::Weak
    );
    assert_eq!(
        runtime(&running_updated)
            .vitals
            .as_ref()
            .and_then(|vitals| vitals.current_hp),
        Some(17)
    );
}

fn local_participant(
    fixture: &crate::test_support::FixtureWorker,
    encounter_ref: &str,
    participant_key: &str,
) -> EncounterParticipant {
    fixture
        .worker
        .local_state_store()
        .expect("store should open")
        .encounters()
        .get_with_participants(encounter_ref)
        .expect("encounter lookup should succeed")
        .expect("encounter should exist")
        .participants
        .into_iter()
        .find(|participant| participant.participant_key == participant_key)
        .expect("participant should exist")
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
        let vitals = runtime(participant).vitals.as_ref().expect("vitals");
        assert_eq!(
            vitals.maximum_hp.as_ref().map(|value| value.adjusted_value),
            Some(17)
        );
        assert_eq!(vitals.current_hp, Some(17));
        assert_eq!(
            participant.side,
            atlas_app_model::EncounterParticipantSideView::Enemy
        );
        assert!(matches!(
            participant.record_view.presentation,
            RecordSurfacePresentationView::Creature { .. }
        ));
        let edition = participant
            .record_view
            .metadata
            .edition
            .as_ref()
            .expect("encounter record should expose edition metadata");
        assert_eq!(edition.status, RecordSurfaceEditionStatusView::Legacy);
        assert!(edition.counterparts.is_empty());
        assert!(runtime(participant).defenses.is_some());
        assert!(runtime(participant).vitals.is_some());
        assert!(participant.reset.available);
        let spell_state = fixture
            .worker
            .local_state_store()
            .expect("local state should open")
            .encounters()
            .spell_state(&participant.participant_key)
            .expect("record-backed participant spell state should read");
        assert!(
            spell_state.initialized,
            "record-backed creation must atomically capture even a known-empty spell-resource baseline"
        );
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
    let hazard_vitals = runtime(hazard)
        .vitals
        .as_ref()
        .expect("hazard runtime vitals");
    assert_eq!(
        hazard_vitals
            .maximum_hp
            .as_ref()
            .map(|value| value.adjusted_value),
        Some(30)
    );
    assert_eq!(hazard_vitals.current_hp, Some(30));
    assert!(matches!(
        hazard.record_view.presentation,
        RecordSurfacePresentationView::Unavailable { .. }
    ));
    assert!(runtime(hazard).defenses.is_none());
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
            },
        )
        .expect("condition record should add");
    let condition = &runtime(&with_condition.participants[0]).conditions[0];
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
            },
        )
        .expect_err("non-condition record should reject");

    assert!(error.into_app_error().message.contains("condition records"));
}

#[test]
fn encounter_condition_definitions_expose_modeled_canonical_conditions() {
    let fixture = fixture_worker();

    let catalog = fixture
        .worker
        .encounter_condition_definitions()
        .expect("condition catalog should load");

    assert_eq!(catalog.conditions.len(), 43);

    let frightened = catalog
        .conditions
        .iter()
        .find(|condition| condition.name == "Frightened")
        .expect("frightened should be in catalog");
    assert_eq!(frightened.condition_ref, "conditionitems:TBSHQspnbcqxsmjL");
    assert_eq!(
        frightened.automation_level,
        EncounterConditionAutomationLevelView::Automated
    );
    assert_eq!(frightened.default_value, Some(1));
    assert!(
        frightened
            .categories
            .contains(&EncounterConditionCategoryView::StatModifier)
    );

    let fatigued = catalog
        .conditions
        .iter()
        .find(|condition| condition.name == "Fatigued")
        .expect("fatigued should be in catalog");
    assert_eq!(fatigued.condition_ref, "conditionitems:HL2l2VRSaQHu9lUw");
    assert_eq!(
        fatigued.automation_level,
        EncounterConditionAutomationLevelView::Automated
    );
    assert!(!fatigued.has_value);
    assert_eq!(fatigued.default_value, None);
    assert_eq!(
        fatigued.categories,
        vec![
            EncounterConditionCategoryView::StatModifier,
            EncounterConditionCategoryView::RuntimeState,
        ]
    );

    let broken = catalog
        .conditions
        .iter()
        .find(|condition| condition.name == "Broken")
        .expect("broken should be in catalog");
    assert_eq!(
        broken.automation_level,
        EncounterConditionAutomationLevelView::Tracked
    );
    assert_eq!(
        broken.applies_to,
        vec![EncounterConditionApplicabilityView::Object]
    );

    let persistent_damage = catalog
        .conditions
        .iter()
        .find(|condition| condition.name == "Persistent Damage")
        .expect("persistent damage should be in catalog");
    assert_eq!(
        persistent_damage.automation_level,
        EncounterConditionAutomationLevelView::Tracked
    );
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
            },
        )
        .expect("record-backed condition should add");
    let condition = &runtime(&added.participants[0]).conditions[0];
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
            },
        )
        .expect("manual condition update should preserve key");
    let condition = &runtime(&manual_update.participants[0]).conditions[0];
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
            },
        )
        .expect("condition ref update should resolve stored key and name");
    let condition = &runtime(&resolved_update.participants[0]).conditions[0];
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
    assert!(matches!(
        participant.record_view.presentation,
        RecordSurfacePresentationView::Unavailable { .. }
    ));
    assert!(runtime(participant).defenses.is_none());
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
            },
        )
        .expect("condition should add");
    let condition = &runtime(&added.participants[1]).conditions[0];
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
            },
        )
        .expect("condition should update");
    assert_eq!(
        runtime(&updated.participants[1]).conditions[0].value,
        Some(2)
    );

    let removed = fixture
        .worker
        .remove_encounter_participant_condition(
            &encounter.slug,
            &high.participant_key,
            condition.condition_id,
        )
        .expect("condition should remove");
    assert!(runtime(&removed.participants[1]).conditions.is_empty());
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
            },
        )
        .expect("condition should add");
    let condition_id = runtime(&added.participants[0]).conditions[0].condition_id;

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
    assert_eq!(runtime(owner).conditions.len(), 1);
    assert_eq!(runtime(owner).conditions[0].name, "Frightened");
    assert_eq!(runtime(owner).conditions[0].value, Some(1));
    assert_eq!(
        runtime(owner).conditions[0].note.as_deref(),
        Some("original")
    );
}

#[test]
fn reset_participant_restores_mechanics_and_reports_preserved_authored_domains() {
    let fixture = fixture_worker();
    let encounter = fixture
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Reset Contract".to_string(),
            description: None,
            note: None,
        })
        .expect("encounter should create")
        .encounter;
    let original = fixture
        .worker
        .local_state_store()
        .expect("local state should open")
        .encounters()
        .add_participant(&encounter.slug, pc("Original", Some(18)))
        .expect("participant should add");
    fixture
        .worker
        .add_encounter_participant_condition(
            &encounter.slug,
            AddEncounterParticipantConditionRequest {
                participant_key: original.participant_key.clone(),
                condition_ref: None,
                name: Some("Slowed".to_string()),
                value: Some(1),
                source_participant_key: None,
                duration_rounds: None,
                note: None,
            },
        )
        .expect("condition should add");
    fixture
        .worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: encounter.slug.clone(),
            participant_key: Some(original.participant_key.clone()),
        })
        .expect("turn should set");
    let mut update = participant_update(&original, true);
    update.display_name = "Custom Name".to_string();
    update.side = atlas_app_model::EncounterParticipantSideView::Ally;
    update.participant_variant = EncounterParticipantVariantView::Elite;
    update.initiative = Some(9);
    update.max_hp = Some(20);
    update.current_hp = Some(0);
    update.temporary_hp = 4;
    update.hidden = true;
    update.note = Some("Preserved note".to_string());
    let changed = fixture
        .worker
        .update_encounter_participant(&encounter.slug, update)
        .expect("participant should mutate");
    assert!(changed.reset.available);
    assert!(
        !runtime(&changed)
            .action_budget
            .as_ref()
            .expect("budget")
            .can_act
            .available
    );
    write_reset_api_sample("participant-reset-before.json", &changed);

    let result = fixture
        .worker
        .reset_encounter_participant(
            &encounter.slug,
            &original.participant_key,
            ResetEncounterParticipantRequest {
                confirmation: EncounterParticipantResetConfirmationView::ResetParticipant,
            },
        )
        .expect("participant should reset");
    assert_eq!(
        result.reset_domains,
        vec![
            EncounterParticipantResetDomainView::HitPoints,
            EncounterParticipantResetDomainView::Defeated,
            EncounterParticipantResetDomainView::Conditions,
            EncounterParticipantResetDomainView::InitiativeTurnState,
            EncounterParticipantResetDomainView::VariantAdjustments,
            EncounterParticipantResetDomainView::ActionBudget,
            EncounterParticipantResetDomainView::SpellResources,
        ]
    );
    assert_eq!(
        result.preserved_domains,
        vec![
            EncounterParticipantPreservedDomainView::DisplayName,
            EncounterParticipantPreservedDomainView::Notes,
            EncounterParticipantPreservedDomainView::Visibility,
            EncounterParticipantPreservedDomainView::Side,
        ]
    );
    assert!(result.cleared_current_turn);
    assert_eq!(result.participant.display_name, "Custom Name");
    assert_eq!(result.participant.note.as_deref(), Some("Preserved note"));
    assert!(result.participant.hidden);
    assert_eq!(
        result.participant.side,
        atlas_app_model::EncounterParticipantSideView::Ally
    );
    assert_eq!(
        result.participant.participant_variant,
        EncounterParticipantVariantView::Normal
    );
    assert_eq!(result.participant.initiative, Some(18));
    assert!(!result.participant.defeated);
    assert!(runtime(&result.participant).conditions.is_empty());
    assert!(
        runtime(&result.participant)
            .action_budget
            .as_ref()
            .expect("budget")
            .can_act
            .available
    );
    let detail = fixture
        .worker
        .encounter(&encounter.slug)
        .expect("encounter should reload");
    assert_eq!(detail.current_turn_participant_key, None);
    write_reset_api_sample("participant-reset-after.json", &result);
}

fn write_reset_api_sample<T: serde::Serialize>(file_name: &str, value: &T) {
    let Ok(root) = std::env::var("ATLAS_F2_BACKEND_SAMPLE_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    std::fs::create_dir_all(&root).expect("backend sample root should be creatable");
    let bytes = serde_json::to_vec_pretty(value).expect("backend sample should serialize");
    std::fs::write(root.join(file_name), bytes).expect("backend sample should write");
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
