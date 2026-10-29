use std::fs;
use std::path::PathBuf;

use atlas_domain::RecordKey;
use time::OffsetDateTime;

use super::*;
use crate::TestStore;

#[test]
fn concurrent_initiative_retries_commit_one_receipt() -> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("concurrent-roll")?;
    store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles = (0..2)
        .map(|_| {
            let store = store.clone();
            let snapshot = snapshot.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let participant = &snapshot.participants[0];
                barrier.wait();
                store.encounters().roll_initiative(
                    &snapshot.encounter.encounter_key,
                    "same-request",
                    &serde_json::json!({"initiative":true}),
                    snapshot.encounter.revision,
                    &[ResolvedInitiativeRoll {
                        participant_key: participant.participant_key.clone(),
                        expected_revision: participant.revision,
                        basis: "Perception".into(),
                        formula: "1d20+4".into(),
                    }],
                )
            })
        })
        .collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(results[0], results[1]);
    let saved = store.encounters().get_with_participants("test")?.unwrap();
    assert_eq!(saved.encounter.revision, snapshot.encounter.revision + 1);
    assert_eq!(
        saved.participants[0].revision,
        snapshot.participants[0].revision + 1
    );
    Ok(())
}

#[test]
fn initiative_receipts_recover_without_reroll_and_conflict_on_changed_intent()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("roll-retry")?;
    store
        .encounters()
        .add_participant("test", creature("Twin", Some(10), 20))?;
    store
        .encounters()
        .add_participant("test", creature("Twin", Some(10), 20))?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let key = &before.encounter.encounter_key;
    let resolved = before
        .participants
        .iter()
        .map(|p| ResolvedInitiativeRoll {
            participant_key: p.participant_key.clone(),
            expected_revision: p.revision,
            basis: "Perception".into(),
            formula: "1d20+5".into(),
        })
        .collect::<Vec<_>>();
    let intent = serde_json::json!({"participants": ["first", "second"]});
    let receipt = store.encounters().roll_initiative(
        key,
        "request-1",
        &intent,
        before.encounter.revision,
        &resolved,
    )?;
    let saved = store.encounters().get_with_participants(key)?.unwrap();
    assert_eq!(receipt.rolls.len(), 2);
    assert_ne!(
        receipt.rolls[0].participant_key,
        receipt.rolls[1].participant_key
    );
    for roll in &receipt.rolls {
        let participant = saved
            .participants
            .iter()
            .find(|p| p.participant_key == roll.participant_key)
            .unwrap();
        assert_eq!(participant.initiative, Some(i64::from(roll.outcome.total)));
        assert_eq!(participant.current_hp, Some(20));
    }
    assert_eq!(
        saved.encounter.current_turn_participant_key,
        before.encounter.current_turn_participant_key
    );
    assert_eq!(saved.encounter.round_number, before.encounter.round_number);
    // Reopen to prove persistence; the old revision and even absent source inputs
    // cannot force another roll once this request has committed.
    let reopened = TestStore::open(store.path())?;
    assert_eq!(
        reopened
            .encounters()
            .roll_initiative(key, "request-1", &intent, -1, &[])?,
        receipt
    );
    assert_eq!(
        reopened.encounters().get_with_participants(key)?.unwrap(),
        saved
    );
    assert!(matches!(
        reopened.encounters().initiative_roll_receipt(
            key,
            "request-1",
            &serde_json::json!({"changed":true})
        ),
        Err(crate::LocalStateError::RollRequestConflict)
    ));
    reopened.encounters().delete(key)?;
    assert!(
        reopened
            .encounters()
            .initiative_roll_receipt(key, "request-1", &intent)?
            .is_none()
    );
    Ok(())
}

#[test]
fn initiative_batch_and_receipt_rollback_together() -> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("roll-rollback")?;
    store
        .encounters()
        .add_participant("test", creature("First", Some(10), 20))?;
    store
        .encounters()
        .add_participant("test", creature("Second", Some(10), 20))?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let key = &before.encounter.encounter_key;
    let mut resolved = before
        .participants
        .iter()
        .map(|p| ResolvedInitiativeRoll {
            participant_key: p.participant_key.clone(),
            expected_revision: p.revision,
            basis: "Manual".into(),
            formula: "20".into(),
        })
        .collect::<Vec<_>>();
    let intent = serde_json::json!({"roll":"both"});
    resolved[1].expected_revision += 1;
    assert!(matches!(
        store.encounters().roll_initiative(
            key,
            "retry",
            &intent,
            before.encounter.revision,
            &resolved
        ),
        Err(crate::LocalStateError::StaleParticipantRevision { .. })
    ));
    assert_eq!(
        store.encounters().get_with_participants(key)?.unwrap(),
        before
    );
    resolved[1].expected_revision -= 1;
    store.connection()?.execute_batch("CREATE TRIGGER fail_receipt BEFORE INSERT ON encounter_roll_receipts BEGIN SELECT RAISE(ABORT,'receipt failure'); END;")?;
    assert!(
        store
            .encounters()
            .roll_initiative(key, "retry", &intent, before.encounter.revision, &resolved)
            .is_err()
    );
    assert_eq!(
        store.encounters().get_with_participants(key)?.unwrap(),
        before
    );
    assert!(
        store
            .encounters()
            .initiative_roll_receipt(key, "retry", &intent)?
            .is_none()
    );
    store
        .connection()?
        .execute_batch("DROP TRIGGER fail_receipt")?;
    let receipt = store.encounters().roll_initiative(
        key,
        "retry",
        &intent,
        before.encounter.revision,
        &resolved,
    )?;
    assert_eq!(receipt.rolls.len(), 2);
    assert!(
        store
            .encounters()
            .get_with_participants(key)?
            .unwrap()
            .participants
            .iter()
            .all(|p| p.initiative == Some(20))
    );
    Ok(())
}

#[test]
fn encounters_allow_duplicate_record_participants() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("encounter-duplicates"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "crypt-fight".to_string(),
        name: "Crypt Fight".to_string(),
        description: None,
        note: None,
    })?;

    let first =
        encounters.add_participant("crypt-fight", creature("Goblin Warrior 1", Some(22), 6))?;
    let second =
        encounters.add_participant("crypt-fight", creature("Goblin Warrior 2", Some(22), 6))?;

    assert_ne!(first.participant_key, second.participant_key);
    assert_eq!(first.record_key, second.record_key);
    let detail = encounters
        .get_with_participants("crypt-fight")?
        .expect("encounter should exist");
    assert_eq!(detail.participants.len(), 2);
    Ok(())
}

#[test]
fn encounters_support_manual_pc_participants() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("encounter-pcs"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "boss-fight".to_string(),
        name: "Boss Fight".to_string(),
        description: None,
        note: Some("Session finale".to_string()),
    })?;

    let pc = encounters.add_participant(
        "boss-fight",
        AddEncounterParticipant {
            level_override: None,
            spell_resources: Vec::new(),
            hp_origin: crate::ParticipantHpOrigin::Explicit,
            variant_origin: crate::ParticipantVariantOrigin::Explicit,
            participant_variant: crate::ParticipantVariant::Normal,
            record_key: None,
            participant_kind: ParticipantKind::Pc,
            display_name: "Asta".to_string(),
            record_title_snapshot: None,
            record_kind_snapshot: None,
            side: ParticipantSide::Pc,
            initiative: Some(28),
            max_hp: Some(77),
            current_hp: Some(77),
            temporary_hp: 0,
            note: Some("Blessed".to_string()),
        },
    )?;

    assert_eq!(pc.participant_kind, ParticipantKind::Pc);
    assert_eq!(pc.record_key, None);
    assert_eq!(pc.side, ParticipantSide::Pc);
    assert_eq!(pc.current_hp, Some(77));
    Ok(())
}

#[test]
fn encounters_order_set_initiative_before_unset_and_preserve_tie_order()
-> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("encounter-initiative"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "initiative".to_string(),
        name: "Initiative".to_string(),
        description: None,
        note: None,
    })?;

    encounters.add_participant("initiative", creature("Unset", None, 1))?;
    encounters.add_participant("initiative", creature("High", Some(24), 2))?;
    encounters.add_participant("initiative", creature("Tie One", Some(18), 3))?;
    encounters.add_participant("initiative", creature("Tie Two", Some(18), 4))?;

    let detail = encounters
        .get_with_participants("initiative")?
        .expect("encounter should exist");
    assert_eq!(
        detail
            .participants
            .iter()
            .map(|participant| participant.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["High", "Tie One", "Tie Two", "Unset"]
    );
    assert_eq!(detail.participants[2].initiative_order, 2);
    Ok(())
}

#[test]
fn deleting_encounter_cascades_participants() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("encounter-delete"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "delete-me".to_string(),
        name: "Delete Me".to_string(),
        description: None,
        note: None,
    })?;
    encounters.add_participant("delete-me", creature("Goblin", Some(12), 6))?;

    assert!(encounters.delete("delete-me")?);
    assert!(encounters.get_with_participants("delete-me")?.is_none());
    assert_eq!(
        store.connection()?.query_row(
            "SELECT count(*) FROM encounter_participants",
            [],
            |row| row.get::<_, i64>(0)
        )?,
        0
    );
    Ok(())
}

#[test]
fn encounters_reorder_participants_across_initiative_buckets()
-> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("encounter-reorder"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "reorder".to_string(),
        name: "Reorder".to_string(),
        description: None,
        note: None,
    })?;

    let high = encounters.add_participant("reorder", creature("High", Some(24), 2))?;
    let low_one = encounters.add_participant("reorder", creature("Low One", Some(12), 3))?;
    let low_two = encounters.add_participant("reorder", creature("Low Two", Some(12), 4))?;

    encounters.reorder_participant(ReorderEncounterParticipant {
        participant_key: high.participant_key,
        target_participant_key: low_two.participant_key,
        placement: ReorderPlacement::After,
    })?;

    let detail = encounters
        .get_with_participants("reorder")?
        .expect("encounter should exist");
    assert_eq!(
        detail
            .participants
            .iter()
            .map(|participant| (participant.display_name.as_str(), participant.initiative))
            .collect::<Vec<_>>(),
        vec![
            ("Low One", Some(12)),
            ("Low Two", Some(12)),
            ("High", Some(12))
        ]
    );
    assert_eq!(detail.participants[2].initiative_order, 3);
    assert_eq!(low_one.initiative_order, 1);
    Ok(())
}

#[test]
fn encounter_conditions_are_added_updated_removed_and_cascade()
-> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("encounter-conditions"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "conditions".to_string(),
        name: "Conditions".to_string(),
        description: None,
        note: None,
    })?;
    let participant = encounters.add_participant("conditions", creature("Goblin", Some(12), 6))?;
    let condition = encounters.add_condition(AddEncounterParticipantCondition {
        participant_key: participant.participant_key.clone(),
        condition_key: Some("conditions:frightened".to_string()),
        name: "Frightened".to_string(),
        value: Some(1),
        source_participant_key: None,
        scope: ConditionScope::default(),
        extra_action: ConditionExtraAction::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: Some(crate::ConditionDuration {
            amount: 2,
            unit: crate::ConditionDurationUnit::Rounds,
            timing: None,
        }),
        note: Some("From spell".to_string()),
    })?;
    assert_eq!(condition.name, "Frightened");

    let updated = encounters
        .update_condition(UpdateEncounterParticipantCondition {
            expected_encounter_revision: encounters.get("conditions")?.unwrap().revision,
            participant_key: participant.participant_key.clone(),
            condition_id: condition.condition_id,
            condition_key: condition.condition_key,
            name: "Frightened".to_string(),
            value: Some(2),
            source_participant_key: None,
            scope: ConditionScope::default(),
            extra_action: ConditionExtraAction::default(),
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: Some(crate::ConditionDuration {
                amount: 1,
                unit: crate::ConditionDurationUnit::Rounds,
                timing: None,
            }),
            note: Some("Raised".to_string()),
        })?
        .expect("condition should update");
    assert_eq!(updated.value, Some(2));

    let detail = encounters
        .get_with_participants("conditions")?
        .expect("encounter should exist");
    assert_eq!(detail.participants[0].conditions.len(), 1);
    assert!(encounters.remove_condition(&participant.participant_key, condition.condition_id)?);
    assert!(
        encounters
            .get_with_participants("conditions")?
            .expect("encounter should exist")
            .participants[0]
            .conditions
            .is_empty()
    );

    let condition = encounters.add_condition(AddEncounterParticipantCondition {
        participant_key: participant.participant_key,
        condition_key: None,
        name: "Custom".to_string(),
        value: None,
        source_participant_key: None,
        scope: ConditionScope::default(),
        extra_action: ConditionExtraAction::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: None,
    })?;
    assert!(condition.condition_id > 0);
    assert!(encounters.delete("conditions")?);
    Ok(())
}

#[test]
fn participant_variant_defaults_and_updates() -> Result<(), Box<dyn std::error::Error>> {
    let store = TestStore::open(temp_path("participant-variant"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "variants".to_string(),
        name: "Variants".to_string(),
        description: None,
        note: None,
    })?;
    let participant = encounters.add_participant("variants", creature("Goblin", Some(12), 6))?;
    assert_eq!(participant.participant_variant, ParticipantVariant::Normal);

    let updated = encounters
        .update_participant(UpdateEncounterParticipant {
            expected_revision: participant.revision,
            hp_origin: crate::ParticipantHpOrigin::Explicit,
            variant_origin: crate::ParticipantVariantOrigin::Explicit,
            participant_key: participant.participant_key.clone(),
            display_name: participant.display_name,
            side: participant.side,
            participant_variant: ParticipantVariant::Elite,
            initiative: participant.initiative,
            max_hp: participant.max_hp,
            current_hp: participant.current_hp,
            temporary_hp: participant.temporary_hp,
            defeated: participant.defeated,
            hazard_disabled: participant.hazard_disabled,
            hidden: participant.hidden,
            note: participant.note,
        })?
        .expect("participant should update");
    assert_eq!(updated.participant_variant, ParticipantVariant::Elite);

    let detail = encounters
        .get_with_participants("variants")?
        .expect("encounter should exist");
    assert_eq!(
        detail.participants[0].participant_variant,
        ParticipantVariant::Elite
    );
    Ok(())
}

#[test]
fn stale_complete_edits_preserve_disjoint_fields_hp_and_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("stale-edits")?;
    let second_caller = TestStore::open(store.path())?;
    let initial = store
        .encounters()
        .add_participant("test", creature("One", Some(12), 60))?;
    let mut side_edit = edit(&initial);
    side_edit.side = ParticipantSide::Ally;
    let side_saved = store.encounters().update_participant(side_edit)?.unwrap();
    assert!(side_saved.revision > initial.revision);
    let mut stale_initiative = edit(&initial);
    stale_initiative.initiative = Some(25);
    assert!(matches!(
        second_caller
            .encounters()
            .update_participant(stale_initiative),
        Err(crate::LocalStateError::StaleParticipantRevision { .. })
    ));
    assert_eq!(
        store
            .encounters()
            .get_with_participants("test")?
            .unwrap()
            .participants[0],
        side_saved
    );

    let mut damage = edit(&side_saved);
    damage.current_hp = Some(40);
    damage.hp_origin = ParticipantHpOrigin::DerivedEdited;
    let damaged = store.encounters().update_participant(damage)?.unwrap();
    let mut stale_hp = edit(&side_saved);
    stale_hp.max_hp = Some(90);
    stale_hp.current_hp = Some(90);
    stale_hp.hp_origin = ParticipantHpOrigin::Explicit;
    assert!(matches!(
        second_caller.encounters().update_participant(stale_hp),
        Err(crate::LocalStateError::StaleParticipantRevision { .. })
    ));
    assert_eq!(
        store
            .encounters()
            .get_with_participants("test")?
            .unwrap()
            .participants[0],
        damaged
    );
    Ok(())
}

#[test]
fn reorder_advances_revision_and_rejects_old_initiative_edit()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("reorder-revision")?;
    let first = store
        .encounters()
        .add_participant("test", creature("First", Some(20), 6))?;
    let target = store
        .encounters()
        .add_participant("test", creature("Target", Some(10), 6))?;
    let reordered = store
        .encounters()
        .reorder_participant(ReorderEncounterParticipant {
            participant_key: first.participant_key.clone(),
            target_participant_key: target.participant_key,
            placement: ReorderPlacement::After,
        })?
        .unwrap();
    assert_eq!(reordered.initiative, Some(10));
    assert!(reordered.revision > first.revision);
    assert!(matches!(
        store.encounters().update_participant(edit(&first)),
        Err(crate::LocalStateError::StaleParticipantRevision { .. })
    ));
    Ok(())
}

#[test]
fn quantity_failure_rolls_back_every_insert_and_encounter_touch()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("quantity-rollback")?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    store.connection()?.execute_batch(
        "CREATE TRIGGER reject_quantity BEFORE INSERT ON encounter_participants WHEN NEW.display_name='Reject'
         BEGIN SELECT RAISE(ABORT,'injected quantity failure'); END;",
    )?;
    assert!(
        store
            .encounters()
            .add_participants(
                "test",
                vec![
                    creature("First", Some(20), 6),
                    creature("Reject", Some(10), 6),
                ]
            )
            .is_err()
    );
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    store
        .connection()?
        .execute_batch("DROP TRIGGER reject_quantity;")?;
    let added = store.encounters().add_participants(
        "test",
        vec![
            creature("First", Some(20), 6),
            creature("Second", Some(10), 6),
        ],
    )?;
    assert_eq!(added.len(), 2);
    assert_eq!(added[1].position, 2);
    Ok(())
}

#[test]
fn failed_replacement_turn_rolls_back_removal_conditions_and_order()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("remove-rollback")?;
    let rows = store.encounters().add_participants(
        "test",
        vec![
            creature("First", Some(20), 6),
            creature("Second", Some(10), 6),
        ],
    )?;
    store
        .encounters()
        .set_current_turn("test", Some(&rows[0].participant_key))?;
    store
        .encounters()
        .add_condition(AddEncounterParticipantCondition {
            participant_key: rows[0].participant_key.clone(),
            condition_key: None,
            name: "Custom".into(),
            value: None,
            source_participant_key: None,
            scope: ConditionScope::default(),
            extra_action: ConditionExtraAction::default(),
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: None,
            note: None,
        })?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    store.connection()?.execute_batch(
        "CREATE TRIGGER reject_turn BEFORE UPDATE ON encounters WHEN NEW.current_turn_participant_key IS NOT NULL
         BEGIN SELECT RAISE(ABORT,'injected turn failure'); END;",
    )?;
    let choose = |snapshot: &EncounterWithParticipants| {
        Some(EncounterTurnState {
            participant_key: Some(snapshot.participants[1].participant_key.clone()),
            round_number: snapshot.encounter.round_number,
            mark_running: true,
        })
    };
    assert!(
        store
            .encounters()
            .remove_participant_and_select_turn("test", &rows[0].participant_key, choose)
            .is_err()
    );
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    store
        .connection()?
        .execute_batch("DROP TRIGGER reject_turn;")?;
    let committed = store
        .encounters()
        .remove_participant_and_select_turn("test", &rows[0].participant_key, choose)?
        .unwrap();
    assert_eq!(
        committed.current_turn_participant_key,
        Some(rows[1].participant_key.clone())
    );
    let after = store.encounters().get_with_participants("test")?.unwrap();
    assert_eq!(after.participants.len(), 1);
    assert_eq!(after.encounter, committed);
    assert_eq!(after.participants[0].position, 1);
    Ok(())
}

#[test]
fn deleted_condition_ids_are_not_reused_for_same_or_other_participants()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("condition-no-reuse")?;
    let rows = store.encounters().add_participants(
        "test",
        vec![creature("First", None, 6), creature("Second", None, 6)],
    )?;
    let condition = |participant_key: &str| AddEncounterParticipantCondition {
        participant_key: participant_key.to_string(),
        condition_key: None,
        name: "Custom".into(),
        value: Some(1),
        source_participant_key: None,
        scope: ConditionScope::default(),
        extra_action: ConditionExtraAction::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: None,
    };
    for target in [&rows[0].participant_key, &rows[1].participant_key] {
        let old = store
            .encounters()
            .add_condition(condition(&rows[0].participant_key))?;
        store
            .encounters()
            .remove_condition(&rows[0].participant_key, old.condition_id)?;
        let new = store.encounters().add_condition(condition(target))?;
        assert!(new.condition_id > old.condition_id);
        assert!(
            store
                .encounters()
                .update_condition(UpdateEncounterParticipantCondition {
                    expected_encounter_revision: 0,
                    participant_key: rows[0].participant_key.clone(),
                    condition_id: old.condition_id,
                    condition_key: None,
                    name: "Stale".into(),
                    value: Some(9),
                    source_participant_key: None,
                    scope: ConditionScope::default(),
                    extra_action: ConditionExtraAction::default(),
                    immunity_override: false,
                    parameters: Default::default(),
                    origin: None,
                    duration: None,
                    note: None,
                })?
                .is_none()
        );
        assert!(
            !store
                .encounters()
                .remove_condition(&rows[0].participant_key, old.condition_id)?
        );
        let snapshot = store.encounters().get_with_participants("test")?.unwrap();
        let persisted = snapshot
            .participants
            .iter()
            .find(|p| &p.participant_key == target)
            .unwrap();
        assert!(persisted.conditions.contains(&new));
    }
    Ok(())
}

#[test]
fn condition_edits_bind_expected_participant_even_for_explicitly_reused_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("condition-owner")?;
    let rows = store.encounters().add_participants(
        "test",
        vec![creature("First", None, 6), creature("Second", None, 6)],
    )?;
    let original = store
        .encounters()
        .add_condition(AddEncounterParticipantCondition {
            participant_key: rows[0].participant_key.clone(),
            condition_key: None,
            name: "Original".into(),
            value: Some(1),
            source_participant_key: None,
            scope: ConditionScope::default(),
            extra_action: ConditionExtraAction::default(),
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: None,
            note: None,
        })?;
    store
        .encounters()
        .remove_condition(&rows[0].participant_key, original.condition_id)?;
    // Simulate externally reused legacy identity; SQL must still check its owner.
    store.connection()?.execute(
        "INSERT INTO encounter_participant_conditions(id,participant_id,name,value,created_at,updated_at)
         SELECT ?1,id,'Replacement',2,'now','now' FROM encounter_participants WHERE participant_key=?2",
        rusqlite::params![original.condition_id, rows[1].participant_key],
    )?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    assert!(
        store
            .encounters()
            .update_condition(UpdateEncounterParticipantCondition {
                expected_encounter_revision: 0,
                participant_key: rows[0].participant_key.clone(),
                condition_id: original.condition_id,
                condition_key: None,
                name: "Stale".into(),
                value: Some(9),
                source_participant_key: None,
                scope: ConditionScope::default(),
                extra_action: ConditionExtraAction::default(),
                immunity_override: false,
                parameters: Default::default(),
                origin: None,
                duration: None,
                note: None,
            })?
            .is_none()
    );
    assert!(
        !store
            .encounters()
            .remove_condition(&rows[0].participant_key, original.condition_id)?
    );
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    Ok(())
}

#[test]
fn concurrent_turn_advances_each_use_the_committed_predecessor()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("turn-concurrent")?;
    let rows = store.encounters().add_participants(
        "test",
        vec![
            creature("First", Some(20), 6),
            creature("Second", Some(10), 6),
        ],
    )?;
    store
        .encounters()
        .set_current_turn("test", Some(&rows[0].participant_key))?;
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles = (0..2)
        .map(|_| {
            let store = store.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                store.encounters().advance_turn("test", |snapshot| {
                    let current = snapshot
                        .participants
                        .iter()
                        .position(|p| {
                            Some(&p.participant_key)
                                == snapshot.encounter.current_turn_participant_key.as_ref()
                        })
                        .unwrap();
                    let next = (current + 1) % snapshot.participants.len();
                    EncounterTurnState {
                        participant_key: Some(snapshot.participants[next].participant_key.clone()),
                        round_number: snapshot.encounter.round_number + i64::from(next == 0),
                        mark_running: true,
                    }
                })
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().expect("turn thread panicked")?;
    }
    let committed = store.encounters().get("test")?.unwrap();
    assert_eq!(
        committed.current_turn_participant_key,
        Some(rows[0].participant_key.clone())
    );
    assert_eq!(committed.round_number, 2);
    Ok(())
}

#[test]
fn aggregate_reads_keep_encounter_and_participant_rows_in_one_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("read-snapshot")?;
    store
        .encounters()
        .add_participant("test", creature("Test", Some(10), 6))?;
    let writer = store.clone();
    let handle = std::thread::spawn(move || -> crate::LocalStateResult<()> {
        let mut connection = writer.connection()?;
        for generation in 0..100 {
            let transaction =
                connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let value = generation.to_string();
            transaction.execute("UPDATE encounters SET name=?1", [&value])?;
            transaction.execute(
                "UPDATE encounter_participants SET display_name=?1,revision=revision+1",
                [&value],
            )?;
            transaction.commit()?;
        }
        Ok(())
    });
    for _ in 0..100 {
        let snapshot = store.encounters().get_with_participants("test")?.unwrap();
        assert_eq!(
            snapshot.encounter.name,
            snapshot.participants[0].display_name
        );
    }
    handle.join().expect("writer panicked")?;
    Ok(())
}

fn fixture(name: &str) -> crate::LocalStateResult<TestStore> {
    let store = TestStore::open(temp_path(name))?;
    store.encounters().create(NewEncounter {
        slug: "test".into(),
        name: "Test".into(),
        description: None,
        note: None,
    })?;
    Ok(store)
}

fn edit(participant: &EncounterParticipant) -> UpdateEncounterParticipant {
    UpdateEncounterParticipant {
        expected_revision: participant.revision,
        participant_key: participant.participant_key.clone(),
        hp_origin: participant.hp_origin,
        variant_origin: participant.variant_origin,
        display_name: participant.display_name.clone(),
        side: participant.side,
        participant_variant: participant.participant_variant,
        initiative: participant.initiative,
        max_hp: participant.max_hp,
        current_hp: participant.current_hp,
        temporary_hp: participant.temporary_hp,
        defeated: participant.defeated,
        hazard_disabled: participant.hazard_disabled,
        hidden: participant.hidden,
        note: participant.note.clone(),
    }
}

fn creature(name: &str, initiative: Option<i64>, hp: i64) -> AddEncounterParticipant {
    AddEncounterParticipant {
        level_override: None,
        spell_resources: Vec::new(),
        hp_origin: crate::ParticipantHpOrigin::Explicit,
        variant_origin: crate::ParticipantVariantOrigin::Explicit,
        participant_variant: crate::ParticipantVariant::Normal,
        record_key: Some(RecordKey::parse("actors:goblinWarrior").expect("key should parse")),
        participant_kind: ParticipantKind::Creature,
        display_name: name.to_string(),
        record_title_snapshot: Some("Goblin Warrior".to_string()),
        record_kind_snapshot: Some("creature".to_string()),
        side: ParticipantSide::Enemy,
        initiative,
        max_hp: Some(hp),
        current_hp: Some(hp),
        temporary_hp: 0,
        note: None,
    }
}

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-local-state-{name}-{}-{}.sqlite",
        std::process::id(),
        OffsetDateTime::now_utc().unix_timestamp_nanos()
    ));
    let _ = fs::remove_file(&path);
    path
}

#[test]
fn reviewed_turn_revision_prevents_replay_and_concurrent_double_ticks()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("reviewed-concurrent")?;
    let p = store
        .encounters()
        .add_participant("test", creature("A", Some(20), 20))?;
    let condition = store
        .encounters()
        .add_condition(AddEncounterParticipantCondition {
            participant_key: p.participant_key.clone(),
            condition_key: None,
            name: "Timed".into(),
            value: Some(1),
            source_participant_key: None,
            scope: ConditionScope::default(),
            extra_action: ConditionExtraAction::default(),
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: Some(crate::ConditionDuration {
                amount: 2,
                unit: crate::ConditionDurationUnit::Rounds,
                timing: Some(crate::ConditionTiming {
                    participant_key: p.participant_key.clone(),
                    event: crate::ConditionTurnEvent::End,
                }),
            }),
            note: None,
        })?;
    let revision = store.encounters().get("test")?.unwrap().revision;
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(4));
    let handles = (0..4)
        .map(|request_index| {
            let store = store.clone();
            let barrier = barrier.clone();
            let key = p.participant_key.clone();
            let id = condition.condition_id;
            std::thread::spawn(move || {
                barrier.wait();
                store.encounters().advance_turn_once(
                    "test",
                    &format!("next-{request_index}"),
                    &serde_json::json!({"next":true}),
                    revision,
                    |_| {
                        Ok((
                            crate::EncounterTurnUpdate {
                                start_event: None,
                                start_claims: vec![],
                                turn: EncounterTurnState {
                                    participant_key: Some(key),
                                    round_number: 2,
                                    mark_running: true,
                                },
                                durations: vec![(id, 1)],
                            },
                            true,
                        ))
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(
                r,
                Err(crate::LocalStateError::StaleEncounterRevision { .. })
            ))
            .count(),
        3
    );
    let saved = store.encounters().get_with_participants("test")?.unwrap();
    assert_eq!(saved.encounter.round_number, 2);
    assert_eq!(
        saved.participants[0].conditions[0]
            .duration
            .as_ref()
            .unwrap()
            .amount,
        1
    );
    assert_eq!(saved.encounter.revision, revision + 1);
    assert!(matches!(
        store.encounters().advance_turn_once(
            "test",
            "stale-next",
            &serde_json::json!({"next":true}),
            revision,
            |_| panic!("stale callback must not run")
        ),
        Err(crate::LocalStateError::StaleEncounterRevision { .. })
    ));
    Ok(())
}

#[test]
fn failed_start_resource_write_rolls_back_turn_condition_changes_and_receipts()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("start-resources-rollback")?;
    let p = store
        .encounters()
        .add_participant("test", creature("A", Some(20), 20))?;
    let condition = store
        .encounters()
        .add_condition(AddEncounterParticipantCondition {
            participant_key: p.participant_key.clone(),
            condition_key: None,
            name: "Stunned".into(),
            value: Some(2),
            source_participant_key: None,
            scope: Default::default(),
            extra_action: Default::default(),
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: None,
            note: None,
        })?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    store.connection()?.execute_batch("CREATE TRIGGER reject_resources BEFORE UPDATE OF turn_resources_json ON encounter_participants BEGIN SELECT RAISE(ABORT,'resource failure'); END;")?;
    let key = p.participant_key.clone();
    let request = serde_json::json!({"next":true});
    let request_id = "x".repeat(128);
    let advance = || {
        store.encounters().advance_turn_once(
            "test",
            &request_id,
            &request,
            before.encounter.revision,
            |_| {
                Ok((
                    EncounterTurnUpdate {
                        turn: EncounterTurnState {
                            participant_key: Some(key.clone()),
                            round_number: 1,
                            mark_running: true,
                        },
                        durations: vec![],
                        start_claims: vec![],
                        start_event: Some(PreparedConditionEvent {
                            condition_changes: vec![ConditionEventChange::Remove {
                                participant_key: key.clone(),
                                condition_id: condition.condition_id,
                            }],
                            turn_resources: vec![TurnResources {
                                before_first_turn: false,
                                allocation: None,
                                actions_open: true,
                                pending_reason: None,
                                participant_key: key.clone(),
                                turn_key: "turn:test".into(),
                                round_number: 1,
                                regular_total: 1,
                                regular_remaining: 1,
                                extra_total: 0,
                                extra_remaining: 0,
                                extra_uses: vec![],
                                reactions_total: Some(1),
                                reactions_remaining: Some(1),
                                effects: vec![],
                            }],
                            ..Default::default()
                        }),
                    },
                    false,
                ))
            },
        )
    };
    assert!(advance().is_err());
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    assert_eq!(
        store.connection()?.query_row(
            "SELECT count(*) FROM encounter_outgoing_turns",
            [],
            |r| r.get::<_, i64>(0)
        )?,
        0
    );
    store
        .connection()?
        .execute_batch("DROP TRIGGER reject_resources")?;
    advance()?;
    let after = store.encounters().get_with_participants("test")?.unwrap();
    assert!(after.participants[0].conditions.is_empty());
    assert_eq!(
        after.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .regular_remaining,
        1
    );
    // The failed attempt did not leave a once-only marker that swallows retry.
    advance()?;
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        after
    );
    Ok(())
}

#[test]
fn failed_turn_write_rolls_back_duration_and_revision() -> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("timing-rollback")?;
    let p = store
        .encounters()
        .add_participant("test", creature("A", Some(20), 20))?;
    let condition = store
        .encounters()
        .add_condition(AddEncounterParticipantCondition {
            participant_key: p.participant_key.clone(),
            condition_key: None,
            name: "Timed".into(),
            value: None,
            source_participant_key: None,
            scope: ConditionScope::default(),
            extra_action: ConditionExtraAction::default(),
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: Some(crate::ConditionDuration {
                amount: 1,
                unit: crate::ConditionDurationUnit::Rounds,
                timing: None,
            }),
            note: None,
        })?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    store.connection()?.execute_batch("CREATE TRIGGER reject_turn BEFORE UPDATE ON encounters BEGIN SELECT RAISE(ABORT,'injected turn failure'); END;")?;
    assert!(
        store
            .encounters()
            .advance_turn_once(
                "test",
                "rollback-next",
                &serde_json::json!({"next":true}),
                before.encounter.revision,
                |_| {
                    Ok((
                        crate::EncounterTurnUpdate {
                            start_event: None,
                            start_claims: vec![],
                            turn: EncounterTurnState {
                                participant_key: Some(p.participant_key),
                                round_number: 2,
                                mark_running: true,
                            },
                            durations: vec![(condition.condition_id, 0)],
                        },
                        true,
                    ))
                }
            )
            .is_err()
    );
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    Ok(())
}

#[test]
fn condition_update_checks_encounter_revision_inside_write_transaction()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("condition-encounter-cas")?;
    let p = store
        .encounters()
        .add_participant("test", creature("A", Some(20), 20))?;
    let c = store
        .encounters()
        .add_condition(AddEncounterParticipantCondition {
            participant_key: p.participant_key.clone(),
            condition_key: None,
            name: "Timed".into(),
            value: None,
            source_participant_key: None,
            scope: ConditionScope::default(),
            extra_action: ConditionExtraAction::default(),
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: Some(crate::ConditionDuration {
                amount: 3,
                unit: crate::ConditionDurationUnit::Rounds,
                timing: None,
            }),
            note: None,
        })?;
    let revision = store.encounters().get("test")?.unwrap().revision;
    store.encounters().advance_turn_once(
        "test",
        "review-next",
        &serde_json::json!({"next":true}),
        revision,
        |_| {
            Ok((
                crate::EncounterTurnUpdate {
                    start_event: None,
                    start_claims: vec![],
                    turn: EncounterTurnState {
                        participant_key: Some(p.participant_key.clone()),
                        round_number: 1,
                        mark_running: true,
                    },
                    durations: vec![(c.condition_id, 2)],
                },
                true,
            ))
        },
    )?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let result = store
        .encounters()
        .update_condition(UpdateEncounterParticipantCondition {
            expected_encounter_revision: revision,
            participant_key: p.participant_key,
            condition_id: c.condition_id,
            condition_key: None,
            name: c.name,
            value: None,
            source_participant_key: None,
            scope: ConditionScope::default(),
            extra_action: ConditionExtraAction::default(),
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: c.duration,
            note: Some("old draft".into()),
        });
    assert!(matches!(
        result,
        Err(crate::LocalStateError::StaleEncounterRevision { .. })
    ));
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    Ok(())
}

#[test]
fn spell_resource_writes_roll_back_and_survive_reopening() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_path("spell-resources");
    let store = TestStore::open(&path)?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "casting".into(),
        name: "Casting".into(),
        description: None,
        note: None,
    })?;
    let mut input = creature("Mage", Some(20), 6);
    input.spell_resources = vec![SpellResource {
        key: "rank3".into(),
        capacity: 3,
        remaining: 0,
        undo_remaining: None,
    }];
    let participant = encounters.add_participant("casting", input)?;
    let initial = encounters.get_with_participants("casting")?.unwrap();
    assert_eq!(initial.participants[0].spell_resources[0].remaining, 3);
    let mut resources = initial.participants[0].spell_resources.clone();
    resources[0].remaining = 2;
    resources[0].undo_remaining = Some(3);
    encounters.replace_spell_resources(
        &participant.participant_key,
        participant.revision,
        &resources,
    )?;
    let after = encounters.get_with_participants("casting")?.unwrap();
    assert_eq!(after.participants[0].current_hp, participant.current_hp);
    assert_eq!(after.encounter.revision, initial.encounter.revision + 1);
    resources[0].remaining = 4;
    assert!(
        encounters
            .replace_spell_resources(
                &participant.participant_key,
                after.participants[0].revision,
                &resources
            )
            .is_err()
    );
    let reopened = TestStore::open(&path)?
        .encounters()
        .get_with_participants("casting")?
        .unwrap();
    assert_eq!(reopened.participants[0].spell_resources[0].remaining, 2);
    assert_eq!(
        reopened.participants[0].revision,
        after.participants[0].revision
    );
    assert_eq!(reopened.encounter.revision, after.encounter.revision);
    Ok(())
}

#[test]
fn spell_expenditure_marker_survives_reopen_and_rejects_stale_restore()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_path("spell-markers");
    let store = TestStore::open(&path)?;
    store.encounters().create(NewEncounter {
        slug: "markers".into(),
        name: "Markers".into(),
        description: None,
        note: None,
    })?;
    let participant = store
        .encounters()
        .add_participant("markers", creature("Mage", Some(20), 6))?;
    store.encounters().set_spell_expended(
        &participant.participant_key,
        participant.revision,
        "spell",
        true,
    )?;
    let reopened = TestStore::open(&path)?;
    let marked = reopened
        .encounters()
        .get_with_participants("markers")?
        .unwrap();
    assert_eq!(marked.participants[0].expended_spells, vec!["spell"]);
    assert!(
        reopened
            .encounters()
            .set_spell_expended(
                &participant.participant_key,
                participant.revision,
                "spell",
                false
            )
            .is_err()
    );
    reopened.encounters().set_spell_expended(
        &participant.participant_key,
        marked.participants[0].revision,
        "spell",
        false,
    )?;
    assert!(
        reopened
            .encounters()
            .get_with_participants("markers")?
            .unwrap()
            .participants[0]
            .expended_spells
            .is_empty()
    );
    Ok(())
}

#[test]
fn caught_resource_replacement_error_cannot_commit_partial_replacement()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("resource-savepoint")?;
    let mut input = creature("Caster", None, 20);
    input.spell_resources = vec![SpellResource {
        key: "rank1".into(),
        capacity: 3,
        remaining: 3,
        undo_remaining: None,
    }];
    let participant = store.encounters().add_participant("test", input)?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let mut connection = store.connection()?;
    let tx = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let result = EncountersWrite::from_connection(&tx).replace_spell_resources(
        &participant.participant_key,
        participant.revision,
        &[
            SpellResource {
                key: "rank1".into(),
                capacity: 3,
                remaining: 1,
                undo_remaining: None,
            },
            SpellResource {
                key: "invalid".into(),
                capacity: 2,
                remaining: 9,
                undo_remaining: None,
            },
        ],
    );
    assert!(result.is_err());
    tx.commit()?;
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    Ok(())
}

#[test]
fn caught_batch_add_error_cannot_commit_earlier_participants_or_identities()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("batch-savepoint")?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let mut bad = creature("Bad", None, 20);
    bad.spell_resources = vec![SpellResource {
        key: "invalid".into(),
        capacity: -1,
        remaining: 0,
        undo_remaining: None,
    }];
    let mut connection = store.connection()?;
    let tx = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    tx.execute("INSERT INTO principals VALUES(2,'retained','Retained')", [])?;
    let result = EncountersWrite::from_connection(&tx)
        .add_participants("test", vec![creature("Good", None, 20), bad]);
    assert!(result.is_err());
    tx.commit()?;
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    assert_eq!(
        connection.query_row("SELECT count(*) FROM record_identity", [], |r| r
            .get::<_, i64>(0))?,
        0
    );
    assert_eq!(
        connection.query_row("SELECT name FROM principals WHERE id=2", [], |r| r
            .get::<_, String>(0))?,
        "Retained"
    );
    Ok(())
}

#[test]
fn caught_receipt_insert_error_cannot_commit_initiative_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("receipt-savepoint")?;
    store
        .encounters()
        .add_participant("test", creature("One", Some(10), 20))?;
    store
        .encounters()
        .add_participant("test", creature("Two", Some(10), 20))?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let mut connection = store.connection()?;
    connection.execute_batch("CREATE TRIGGER reject_receipt BEFORE INSERT ON encounter_roll_receipts BEGIN SELECT RAISE(ABORT,'receipt rejected'); END;")?;
    let tx = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let domain = EncountersWrite::from_connection(&tx);
    let result = domain.roll_initiative(
        &before.encounter.encounter_key,
        "failed-request",
        &serde_json::json!({"roll":true}),
        before.encounter.revision,
        &before
            .participants
            .iter()
            .map(|p| ResolvedInitiativeRoll {
                participant_key: p.participant_key.clone(),
                expected_revision: p.revision,
                basis: "Perception".into(),
                formula: "1d20+4".into(),
            })
            .collect::<Vec<_>>(),
    );
    assert!(result.is_err());
    tx.commit()?;
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    assert!(
        store
            .encounters()
            .initiative_roll_receipt(
                &before.encounter.encounter_key,
                "failed-request",
                &serde_json::json!({"roll":true})
            )?
            .is_none()
    );
    Ok(())
}

#[test]
fn condition_scope_restrictions_and_override_survive_independent_sources()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("condition-context")?;
    store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    store
        .encounters()
        .add_participant("test", creature("Two", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let owner = &snapshot.participants[0].participant_key;
    let opponent = &snapshot.participants[1].participant_key;
    let scope = ConditionScope::SelectedParticipants {
        participant_keys: vec![opponent.clone()],
    };
    let extra_action = ConditionExtraAction::Restricted {
        actions: vec![ConditionAction::Stride, ConditionAction::Strike],
        guidance: Some("Granted by Haste".into()),
    };
    for source in [None, Some(opponent.clone())] {
        store
            .encounters()
            .add_condition(AddEncounterParticipantCondition {
                participant_key: owner.clone(),
                condition_key: None,
                name: "Independent condition".into(),
                value: None,
                source_participant_key: source,
                scope: scope.clone(),
                extra_action: extra_action.clone(),
                immunity_override: true,
                parameters: Default::default(),
                origin: None,
                duration: None,
                note: None,
            })?;
    }
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let rows = &snapshot.participants[0].conditions;
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0].condition_id, rows[1].condition_id);
    for row in rows {
        assert_eq!(row.scope, scope);
        assert_eq!(row.extra_action, extra_action);
        assert!(row.immunity_override);
        assert_eq!(row.duration, None);
    }
    Ok(())
}

#[test]
fn stale_condition_context_edit_keeps_saved_restriction() -> Result<(), Box<dyn std::error::Error>>
{
    let store = fixture("stale-condition-context")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let condition = store
        .encounters()
        .add_condition(AddEncounterParticipantCondition {
            participant_key: participant.participant_key.clone(),
            condition_key: None,
            name: "Quickened".into(),
            value: None,
            source_participant_key: None,
            scope: ConditionScope::Everyone,
            extra_action: ConditionExtraAction::Unspecified,
            immunity_override: false,
            parameters: Default::default(),
            origin: None,
            duration: None,
            note: None,
        })?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let request = UpdateEncounterParticipantCondition {
        expected_encounter_revision: snapshot.encounter.revision,
        participant_key: participant.participant_key,
        condition_id: condition.condition_id,
        condition_key: None,
        name: "Quickened".into(),
        value: None,
        source_participant_key: None,
        scope: ConditionScope::Everyone,
        extra_action: ConditionExtraAction::Unrestricted,
        immunity_override: true,
        parameters: Default::default(),
        origin: None,
        duration: None,
        note: None,
    };
    store.encounters().update_condition(request.clone())?;
    assert!(matches!(
        store.encounters().update_condition(request),
        Err(crate::LocalStateError::StaleEncounterRevision { .. })
    ));
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    assert_eq!(
        snapshot.participants[0].conditions[0].extra_action,
        ConditionExtraAction::Unrestricted
    );
    Ok(())
}

fn event_condition(participant: &str, application: &str) -> AddEncounterParticipantCondition {
    AddEncounterParticipantCondition {
        participant_key: participant.into(),
        condition_key: None,
        name: "Independent".into(),
        value: None,
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: Some(ConditionOrigin {
            application_id: application.into(),
            grant_slot: "condition".into(),
            participant_key: None,
            record_key: None,
            activity_id: None,
            artifact_generation: None,
        }),
        duration: None,
        note: None,
    }
}
#[test]
fn condition_event_retries_recover_before_stale_guard_and_do_not_repeat_hp_loss()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("condition-event-retry")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let key = &snapshot.encounter.encounter_key;
    let intent = serde_json::json!({"apply":"drained","value":1});
    let mut hp = ParticipantEventChange::preserving(&participant);
    hp.current_hp = Some(17);
    let changes = PreparedConditionEvent {
        condition_changes: vec![ConditionEventChange::Add(event_condition(
            &participant.participant_key,
            "application-one",
        ))],
        participant_changes: vec![hp],
        ..Default::default()
    };
    let receipt = store.encounters().apply_condition_event(
        key,
        "request-one",
        &intent,
        snapshot.encounter.revision,
        changes.clone(),
    )?;
    let repeated = store.encounters().apply_condition_event(
        key,
        "request-one",
        &intent,
        snapshot.encounter.revision,
        changes,
    )?;
    assert_eq!(receipt, repeated);
    let saved = store.encounters().get_with_participants(key)?.unwrap();
    assert_eq!(saved.participants[0].current_hp, Some(17));
    assert_eq!(saved.participants[0].conditions.len(), 1);
    assert!(
        store
            .encounters()
            .apply_condition_event(
                key,
                "request-one",
                &serde_json::json!({"apply":"different"}),
                saved.encounter.revision,
                Default::default()
            )
            .is_err()
    );
    assert!(matches!(
        store.encounters().apply_condition_event(
            key,
            "request-two",
            &intent,
            snapshot.encounter.revision,
            Default::default()
        ),
        Err(crate::LocalStateError::StaleEncounterRevision { .. })
    ));
    Ok(())
}
#[test]
fn invalid_later_condition_change_rolls_back_prior_change_and_receipt()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("condition-event-rollback")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let key = &snapshot.encounter.encounter_key;
    let intent = serde_json::json!({"apply":"batch"});
    let changes = PreparedConditionEvent {
        condition_changes: vec![
            ConditionEventChange::Add(event_condition(&participant.participant_key, "first")),
            ConditionEventChange::Remove {
                participant_key: participant.participant_key.clone(),
                condition_id: i64::MAX,
            },
        ],
        ..Default::default()
    };
    assert!(
        store
            .encounters()
            .apply_condition_event(
                key,
                "bad-batch",
                &intent,
                snapshot.encounter.revision,
                changes
            )
            .is_err()
    );
    assert_eq!(
        store.encounters().get_with_participants(key)?.unwrap(),
        snapshot
    );
    assert_eq!(
        store
            .encounters()
            .condition_event_receipt(key, "bad-batch", &intent)?,
        None
    );
    Ok(())
}
#[test]
fn consumed_application_claim_and_stock_roll_back_when_receipt_fails()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("consumed-application-rollback")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let key = &before.encounter.encounter_key;
    let origin = event_condition(&participant.participant_key, "consumed")
        .origin
        .unwrap();
    let changes = PreparedConditionEvent {
        consumed_applications: vec![ConsumedConditionApplication {
            participant_key: participant.participant_key.clone(),
            origin,
        }],
        turn_resources: vec![TurnResources {
            before_first_turn: false,
            allocation: None,
            actions_open: true,
            pending_reason: None,
            participant_key: participant.participant_key.clone(),
            turn_key: "turn:test".into(),
            round_number: 1,
            regular_total: 3,
            regular_remaining: 2,
            extra_total: 0,
            extra_remaining: 0,
            extra_uses: vec![],
            reactions_total: Some(1),
            reactions_remaining: Some(1),
            effects: vec![],
        }],
        ..Default::default()
    };
    let intent = serde_json::json!({"application":"consumed"});
    store.connection()?.execute_batch("CREATE TRIGGER reject_consumed_receipt BEFORE INSERT ON encounter_event_receipts BEGIN SELECT RAISE(ABORT,'receipt failure'); END;")?;
    // Catch the action error and commit its caller's transaction: the savepoint
    // must already have restored both the identity claim and resource write.
    let mut connection = store.connection()?;
    let transaction = connection.transaction()?;
    let domain = EncountersWrite::from_connection(&transaction);
    assert!(
        domain
            .apply_condition_event(
                key,
                "consume",
                &intent,
                before.encounter.revision,
                changes.clone()
            )
            .is_err()
    );
    transaction.commit()?;
    assert_eq!(
        store.encounters().get_with_participants(key)?.unwrap(),
        before
    );
    assert_eq!(
        store.connection()?.query_row(
            "SELECT count(*) FROM encounter_condition_applications",
            [],
            |row| row.get::<_, i64>(0),
        )?,
        0
    );
    assert!(
        store
            .encounters()
            .condition_event_receipt(key, "consume", &intent)?
            .is_none()
    );
    store
        .connection()?
        .execute_batch("DROP TRIGGER reject_consumed_receipt")?;
    let receipt = store.encounters().apply_condition_event(
        key,
        "consume",
        &intent,
        before.encounter.revision,
        changes.clone(),
    )?;
    let after = store.encounters().get_with_participants(key)?.unwrap();
    assert!(after.participants[0].conditions.is_empty());
    assert_eq!(
        after.participants[0]
            .turn_resources
            .as_ref()
            .unwrap()
            .regular_remaining,
        2
    );
    let reopened = TestStore::open(store.path())?;
    assert_eq!(
        reopened.encounters().apply_condition_event(
            key,
            "consume",
            &intent,
            before.encounter.revision,
            changes.clone(),
        )?,
        receipt
    );
    for condition_changes in [
        vec![],
        vec![ConditionEventChange::Add(event_condition(
            &participant.participant_key,
            "consumed",
        ))],
    ] {
        let duplicate = if condition_changes.is_empty() {
            changes.clone()
        } else {
            PreparedConditionEvent {
                condition_changes,
                ..Default::default()
            }
        };
        assert!(matches!(
            reopened.encounters().apply_condition_event(
                key,
                "duplicate",
                &intent,
                after.encounter.revision,
                duplicate,
            ),
            Err(crate::LocalStateError::InvalidRollRequest(
                "condition application was already committed"
            ))
        ));
        assert_eq!(
            reopened.encounters().get_with_participants(key)?.unwrap(),
            after
        );
        assert!(
            reopened
                .encounters()
                .condition_event_receipt(key, "duplicate", &intent)?
                .is_none()
        );
    }
    let marker: Option<i64> = reopened.connection()?.query_row(
        "SELECT condition_id FROM encounter_condition_applications WHERE application_id='consumed'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(marker, None);
    Ok(())
}

#[test]
fn independent_applications_survive_and_removed_application_cannot_replay()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("condition-application-identity")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let key = snapshot.encounter.encounter_key;
    for (id, application) in [
        ("request-a", "application-a"),
        ("request-b", "application-b"),
    ] {
        let revision = store.encounters().get(&key)?.unwrap().revision;
        store.encounters().apply_condition_event(
            &key,
            id,
            &serde_json::json!({"application":application}),
            revision,
            PreparedConditionEvent {
                condition_changes: vec![ConditionEventChange::Add(event_condition(
                    &participant.participant_key,
                    application,
                ))],
                ..Default::default()
            },
        )?;
    }
    let snapshot = store.encounters().get_with_participants(&key)?.unwrap();
    assert_eq!(snapshot.participants[0].conditions.len(), 2);
    let removed = snapshot.participants[0].conditions[0].condition_id;
    store.encounters().apply_condition_event(
        &key,
        "remove",
        &serde_json::json!({"remove":removed}),
        snapshot.encounter.revision,
        PreparedConditionEvent {
            condition_changes: vec![ConditionEventChange::Remove {
                participant_key: participant.participant_key.clone(),
                condition_id: removed,
            }],
            ..Default::default()
        },
    )?;
    let revision = store.encounters().get(&key)?.unwrap().revision;
    assert!(
        store
            .encounters()
            .apply_condition_event(
                &key,
                "new-request-same-application",
                &serde_json::json!({"application":"application-a"}),
                revision,
                PreparedConditionEvent {
                    condition_changes: vec![ConditionEventChange::Add(event_condition(
                        &participant.participant_key,
                        "application-a"
                    ))],
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert_eq!(
        store
            .encounters()
            .get_with_participants(&key)?
            .unwrap()
            .participants[0]
            .conditions
            .len(),
        1
    );
    Ok(())
}

#[test]
fn boundary_claim_rejects_new_request_before_evaluation_and_saved_outcome_is_readable()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("boundary-claim")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let key = &snapshot.encounter.encounter_key;
    let claim = EncounterBoundaryClaim {
        participant_key: participant.participant_key,
        round_number: snapshot.encounter.round_number,
        kind: EncounterBoundaryKind::PersistentDamage,
        scope_key: String::new(),
    };
    let evaluations = std::cell::Cell::new(0);
    let mut connection = store.connection()?;
    let tx = connection.transaction()?;
    let domain = EncountersWrite::from_connection(&tx);
    let receipt = domain.apply_condition_event_with_claims(
        key,
        "boundary-first",
        &serde_json::json!({"boundary":true}),
        snapshot.encounter.revision,
        crate::ConditionEventBoundary {
            claims: std::slice::from_ref(&claim),
            outgoing_boundary_key: None,
        },
        || {
            evaluations.set(evaluations.get() + 1);
            Ok(Default::default())
        },
    )?;
    assert_eq!(domain.boundary_receipt(key, &claim)?, Some(receipt));
    let revision = domain.get(key)?.unwrap().revision;
    let error = domain
        .apply_condition_event_with_claims(
            key,
            "boundary-second",
            &serde_json::json!({"boundary":true}),
            revision,
            crate::ConditionEventBoundary {
                claims: std::slice::from_ref(&claim),
                outgoing_boundary_key: None,
            },
            || {
                evaluations.set(evaluations.get() + 1);
                Ok(Default::default())
            },
        )
        .unwrap_err();
    assert!(matches!(
        error,
        crate::LocalStateError::EncounterBoundaryConflict
    ));
    assert_eq!(evaluations.get(), 1);
    tx.commit()?;
    Ok(())
}
#[test]
fn failed_boundary_evaluation_does_not_consume_claim() -> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("boundary-rollback")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let key = &snapshot.encounter.encounter_key;
    let claim = EncounterBoundaryClaim {
        participant_key: participant.participant_key,
        round_number: snapshot.encounter.round_number,
        kind: EncounterBoundaryKind::TurnEnd,
        scope_key: String::new(),
    };
    let mut connection = store.connection()?;
    let tx = connection.transaction()?;
    let domain = EncountersWrite::from_connection(&tx);
    assert!(
        domain
            .apply_condition_event_with_claims(
                key,
                "failed-boundary",
                &serde_json::json!({"boundary":true}),
                snapshot.encounter.revision,
                crate::ConditionEventBoundary {
                    claims: std::slice::from_ref(&claim),
                    outgoing_boundary_key: None
                },
                || Err(crate::LocalStateError::InvalidRollRequest(
                    "evaluation failed"
                ))
            )
            .is_err()
    );
    assert!(!domain.boundary_claimed(key, &claim)?);
    assert_eq!(
        domain.get(key)?.unwrap().revision,
        snapshot.encounter.revision
    );
    tx.commit()?;
    Ok(())
}

#[test]
fn difficulty_reference_is_guarded_persistent_and_does_not_overwrite_encounter_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("difficulty-override")?;
    let initial = store.encounters().get("test")?.unwrap();
    let mut connection = store.connection()?;
    let tx = connection.transaction()?;
    let saved = EncountersWrite::from_connection(&tx).set_difficulty_reference(
        "test",
        initial.revision,
        EncounterDifficultyReference {
            party_level: Some(5),
            party_size: Some(4),
        },
    )?;
    tx.commit()?;
    assert_eq!(saved.name, initial.name);
    assert_eq!(saved.revision, initial.revision + 1);
    assert_eq!(
        store
            .encounters()
            .get("test")?
            .unwrap()
            .difficulty_reference,
        saved.difficulty_reference
    );
    let tx = connection.transaction()?;
    let state = EncountersWrite::from_connection(&tx);
    assert!(matches!(
        state.set_difficulty_reference("test", initial.revision, Default::default()),
        Err(crate::LocalStateError::StaleEncounterRevision { .. })
    ));
    for reference in [
        EncounterDifficultyReference {
            party_level: Some(0),
            party_size: None,
        },
        EncounterDifficultyReference {
            party_level: None,
            party_size: Some(101),
        },
    ] {
        assert!(matches!(
            state.set_difficulty_reference("test", saved.revision, reference),
            Err(crate::LocalStateError::InvalidDifficultyReference(_))
        ));
    }
    tx.commit()?;
    assert_eq!(store.encounters().get("test")?.unwrap(), saved);
    let tx = connection.transaction()?;
    EncountersWrite::from_connection(&tx).set_difficulty_reference(
        "test",
        saved.revision,
        Default::default(),
    )?;
    tx.commit()?;
    assert_eq!(
        store
            .encounters()
            .get("test")?
            .unwrap()
            .difficulty_reference,
        Default::default()
    );
    Ok(())
}

#[test]
fn authored_ability_state_is_participant_owned_persistent_and_retry_safe()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("ability-state-persistence")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    store
        .encounters()
        .add_participant("test", creature("Two", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let state = ParticipantAbilityState {
        owner: atlas_domain::OwnedContentLocator {
            collection: "/items".into(),
            identity: atlas_domain::OwnedContentIdentity::Stable("ability000000000".into()),
        },
        rule_ordinal: None,
        source_fingerprint: "source-one".into(),
        declaration_identity: Some("ability-declaration/v1:test".into()),
        value: AbilityStateValue::Uses { remaining: 2 },
    };
    let changes = PreparedConditionEvent {
        ability_states: vec![AbilityStateChange {
            participant_key: participant.participant_key.clone(),
            expected_revision: participant.revision,
            state: state.clone(),
        }],
        ..Default::default()
    };
    let intent = serde_json::json!({"ability":"use"});
    let receipt = store.encounters().apply_condition_event(
        &snapshot.encounter.encounter_key,
        "use-one",
        &intent,
        snapshot.encounter.revision,
        changes.clone(),
    )?;
    let retry = store.encounters().apply_condition_event(
        &snapshot.encounter.encounter_key,
        "use-one",
        &intent,
        snapshot.encounter.revision,
        changes,
    )?;
    assert_eq!(receipt, retry);
    let reopened = TestStore::open(store.path())?;
    let saved = reopened
        .encounters()
        .get_with_participants("test")?
        .unwrap();
    let first = saved
        .participants
        .iter()
        .find(|p| p.participant_key == participant.participant_key)
        .unwrap();
    assert_eq!(first.ability_states, vec![state]);
    assert_eq!(first.current_hp, Some(20));
    assert!(first.conditions.is_empty());
    assert_eq!(first.revision, participant.revision + 1);
    assert!(
        saved
            .participants
            .iter()
            .filter(|p| p.participant_key != participant.participant_key)
            .all(|p| p.ability_states.is_empty())
    );
    Ok(())
}

#[test]
fn failed_receipt_rolls_back_ability_state_and_participant_revision()
-> Result<(), Box<dyn std::error::Error>> {
    let store = fixture("ability-state-rollback")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("One", None, 20))?;
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    store.connection()?.execute_batch("CREATE TRIGGER reject_ability_receipt BEFORE INSERT ON encounter_event_receipts BEGIN SELECT RAISE(ABORT,'reject receipt'); END;")?;
    let changes = PreparedConditionEvent {
        ability_states: vec![AbilityStateChange {
            participant_key: participant.participant_key.clone(),
            expected_revision: participant.revision,
            state: ParticipantAbilityState {
                owner: atlas_domain::OwnedContentLocator {
                    collection: "/items".into(),
                    identity: atlas_domain::OwnedContentIdentity::Stable("ability000000000".into()),
                },
                rule_ordinal: Some(0),
                source_fingerprint: "source".into(),
                declaration_identity: Some("ability-declaration/v1:test".into()),
                value: AbilityStateValue::Option {
                    active: true,
                    selection: None,
                },
            },
        }],
        ..Default::default()
    };
    assert!(
        store
            .encounters()
            .apply_condition_event(
                "test",
                "failed-ability",
                &serde_json::json!({"active":true}),
                snapshot.encounter.revision,
                changes
            )
            .is_err()
    );
    let saved = store.encounters().get_with_participants("test")?.unwrap();
    assert_eq!(saved.encounter.revision, snapshot.encounter.revision);
    assert_eq!(saved.participants[0].revision, participant.revision);
    assert!(saved.participants[0].ability_states.is_empty());
    Ok(())
}

#[test]
fn recovery_restores_creation_state_preserves_identity_and_recovers_retries()
-> crate::LocalStateResult<()> {
    let store = fixture("recovery-baseline")?;
    let one = store
        .encounters()
        .add_participant("test", creature("One", Some(15), 20))?;
    let two = store
        .encounters()
        .add_participant("test", creature("Two", Some(15), 30))?;
    let mut update = edit(&one);
    update.display_name = "Renamed".into();
    update.note = Some("Keep this".into());
    update.hidden = true;
    update.side = ParticipantSide::Ally;
    update.current_hp = Some(0);
    update.temporary_hp = 6;
    update.defeated = true;
    update.participant_variant = ParticipantVariant::Elite;
    store.encounters().update_participant(update)?;
    {
        let mut c = store.connection()?;
        let tx = c.transaction()?;
        EncountersWrite::from_connection(&tx).set_turn_state(
            "test",
            Some(&one.participant_key),
            4,
            true,
        )?;
        tx.commit()?;
    }
    let snapshot = store.encounters().get_with_participants("test")?.unwrap();
    let mut c = store.connection()?;
    let tx = c.transaction()?;
    let domain = EncountersWrite::from_connection(&tx);
    let baseline = domain.participant_baseline(&one.participant_key)?.unwrap();
    domain.apply_recovery(ApplyEncounterRecovery {
        encounter_key: &snapshot.encounter.encounter_key,
        request_id: "reset-one",
        intent: &serde_json::json!({"reset":"one"}),
        expected_revision: snapshot.encounter.revision,
        reset_encounter: false,
        end_turn: true,
        replacements: &[(one.participant_key.clone(), baseline)],
    })?;
    tx.commit()?;
    let saved = store.encounters().get_with_participants("test")?.unwrap();
    let p = saved
        .participants
        .iter()
        .find(|p| p.participant_key == one.participant_key)
        .unwrap();
    assert_eq!(
        (
            p.current_hp,
            p.temporary_hp,
            p.participant_variant,
            p.defeated
        ),
        (Some(20), 0, ParticipantVariant::Normal, false)
    );
    assert_eq!(
        (&p.display_name, &p.note, p.hidden, p.side),
        (
            &"Renamed".into(),
            &Some("Keep this".into()),
            true,
            ParticipantSide::Ally
        )
    );
    assert_eq!(
        saved
            .participants
            .iter()
            .find(|p| p.participant_key == two.participant_key),
        Some(&two)
    );
    assert_eq!(
        (
            saved.encounter.current_turn_participant_key.clone(),
            saved.encounter.round_number
        ),
        (None, 4)
    );
    let mut c = store.connection()?;
    let tx = c.transaction()?;
    // A retry after subsequent state is safe even with the old encounter revision.
    let domain = EncountersWrite::from_connection(&tx);
    domain.apply_recovery(ApplyEncounterRecovery {
        encounter_key: &saved.encounter.encounter_key,
        request_id: "reset-one",
        intent: &serde_json::json!({"reset":"one"}),
        expected_revision: snapshot.encounter.revision,
        reset_encounter: false,
        end_turn: true,
        replacements: &[],
    })?;
    assert!(
        domain
            .apply_recovery(ApplyEncounterRecovery {
                encounter_key: &saved.encounter.encounter_key,
                request_id: "different",
                intent: &serde_json::json!({}),
                expected_revision: snapshot.encounter.revision,
                reset_encounter: false,
                end_turn: true,
                replacements: &[],
            })
            .is_err()
    );
    assert!(
        domain
            .recovery_applied(
                &saved.encounter.encounter_key,
                "reset-one",
                &serde_json::json!({"reset":"two"})
            )
            .is_err()
    );
    tx.commit()?;
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        saved
    );
    Ok(())
}

#[test]
fn recovery_keeps_unchanged_ties_reserved_while_restoring_changed_initiatives()
-> crate::LocalStateResult<()> {
    let store = fixture("recovery-tied-initiatives")?;
    let mut participants = vec![];
    for name in ["One", "Two", "Three"] {
        participants.push(
            store
                .encounters()
                .add_participant("test", creature(name, Some(10), 20))?,
        );
    }
    for (participant, initiative) in participants.iter().zip([20, 30]) {
        let current = store.encounters().get_with_participants("test")?.unwrap();
        let mut update = edit(
            current
                .participants
                .iter()
                .find(|p| p.participant_key == participant.participant_key)
                .unwrap(),
        );
        update.initiative = Some(initiative);
        store.encounters().update_participant(update)?;
    }
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let unchanged = before
        .participants
        .iter()
        .find(|p| p.participant_key == participants[2].participant_key)
        .unwrap();
    let mut connection = store.connection()?;
    let transaction = connection.transaction()?;
    let domain = EncountersWrite::from_connection(&transaction);
    let replacements = before
        .participants
        .iter()
        .map(|p| {
            Ok((
                p.participant_key.clone(),
                domain.participant_baseline(&p.participant_key)?.unwrap(),
            ))
        })
        .collect::<crate::LocalStateResult<Vec<_>>>()?;
    domain.apply_recovery(ApplyEncounterRecovery {
        encounter_key: &before.encounter.encounter_key,
        request_id: "restore-tied-initiatives",
        intent: &serde_json::json!({"reset":true}),
        expected_revision: before.encounter.revision,
        reset_encounter: true,
        end_turn: true,
        replacements: &replacements,
    })?;
    transaction.commit()?;
    let after = store.encounters().get_with_participants("test")?.unwrap();
    assert!(after.participants.iter().all(|p| p.initiative == Some(10)));
    assert_eq!(
        after
            .participants
            .iter()
            .map(|p| p.initiative_order)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    assert_eq!(
        after
            .participants
            .iter()
            .find(|p| p.participant_key == unchanged.participant_key)
            .unwrap()
            .initiative_order,
        unchanged.initiative_order
    );
    Ok(())
}

#[test]
fn recovery_failure_rolls_back_all_participants_and_does_not_consume_request()
-> crate::LocalStateResult<()> {
    let store = fixture("recovery-rollback")?;
    let one = store
        .encounters()
        .add_participant("test", creature("One", Some(12), 20))?;
    let two = store
        .encounters()
        .add_participant("test", creature("Two", Some(10), 30))?;
    let before = store.encounters().get_with_participants("test")?.unwrap();
    let mut c = store.connection()?;
    let tx = c.transaction()?;
    let domain = EncountersWrite::from_connection(&tx);
    let mut a = ParticipantGameplay::from(&one);
    a.current_hp = Some(3);
    let mut b = ParticipantGameplay::from(&two);
    b.spell_resources.push(SpellResource {
        key: "invalid".into(),
        capacity: 1,
        remaining: 2,
        undo_remaining: None,
    });
    assert!(
        domain
            .apply_recovery(ApplyEncounterRecovery {
                encounter_key: &before.encounter.encounter_key,
                request_id: "rollback",
                intent: &serde_json::json!({}),
                expected_revision: before.encounter.revision,
                reset_encounter: true,
                end_turn: true,
                replacements: &[(one.participant_key, a), (two.participant_key, b)],
            })
            .is_err()
    );
    assert!(!domain.recovery_applied(
        &before.encounter.encounter_key,
        "rollback",
        &serde_json::json!({})
    )?);
    // The caller may still commit unrelated work after the domain savepoint fails.
    tx.commit()?;
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        before
    );
    Ok(())
}

#[test]
fn creation_baseline_captures_final_derived_hp_and_is_not_fabricated_for_legacy_rows()
-> crate::LocalStateResult<()> {
    let store = fixture("recovery-derived")?;
    let mut input = creature("Drained source", None, 40);
    input.hp_origin = ParticipantHpOrigin::DerivedPristine;
    let p = store.encounters().add_participant("test", input)?;
    let mut c = store.connection()?;
    let tx = c.transaction()?;
    let domain = EncountersWrite::from_connection(&tx);
    domain.initialize_participant_hp(&p.participant_key, Some(30))?;
    assert_eq!(
        domain
            .participant_baseline(&p.participant_key)?
            .unwrap()
            .current_hp,
        Some(30)
    );
    tx.execute(
        "DELETE FROM encounter_participant_baselines WHERE participant_key=?1",
        [&p.participant_key],
    )?;
    assert!(domain.participant_baseline(&p.participant_key)?.is_none());
    tx.commit()?;
    Ok(())
}
