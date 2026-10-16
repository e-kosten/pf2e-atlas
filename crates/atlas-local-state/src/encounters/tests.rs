use std::fs;
use std::path::PathBuf;

use atlas_domain::RecordKey;
use time::OffsetDateTime;

use super::*;
use crate::LocalStateStore;

#[test]
fn encounters_allow_duplicate_record_participants() -> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("encounter-duplicates"))?;
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
    let store = LocalStateStore::open(temp_path("encounter-pcs"))?;
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
    let store = LocalStateStore::open(temp_path("encounter-initiative"))?;
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
    let store = LocalStateStore::open(temp_path("encounter-delete"))?;
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
    Ok(())
}

#[test]
fn encounters_reorder_participants_across_initiative_buckets()
-> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("encounter-reorder"))?;
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
    let store = LocalStateStore::open(temp_path("encounter-conditions"))?;
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
        duration: None,
        note: None,
    })?;
    assert!(condition.condition_id > 0);
    assert!(encounters.delete("conditions")?);
    Ok(())
}

#[test]
fn participant_variant_defaults_and_updates() -> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("participant-variant"))?;
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
    let second_caller = LocalStateStore::open(store.path())?;
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

fn fixture(name: &str) -> crate::LocalStateResult<LocalStateStore> {
    let store = LocalStateStore::open(temp_path(name))?;
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
        hidden: participant.hidden,
        note: participant.note.clone(),
    }
}

fn creature(name: &str, initiative: Option<i64>, hp: i64) -> AddEncounterParticipant {
    AddEncounterParticipant {
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
        .map(|_| {
            let store = store.clone();
            let barrier = barrier.clone();
            let key = p.participant_key.clone();
            let id = condition.condition_id;
            std::thread::spawn(move || {
                barrier.wait();
                store
                    .encounters()
                    .advance_reviewed_turn("test", revision, |_| crate::EncounterTurnUpdate {
                        turn: EncounterTurnState {
                            participant_key: Some(key),
                            round_number: 2,
                            mark_running: true,
                        },
                        durations: vec![(id, 1)],
                    })
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
        store
            .encounters()
            .advance_reviewed_turn("test", revision, |_| panic!("stale callback must not run")),
        Err(crate::LocalStateError::StaleEncounterRevision { .. })
    ));
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
            .advance_reviewed_turn("test", before.encounter.revision, |_| {
                crate::EncounterTurnUpdate {
                    turn: EncounterTurnState {
                        participant_key: Some(p.participant_key),
                        round_number: 2,
                        mark_running: true,
                    },
                    durations: vec![(condition.condition_id, 0)],
                }
            })
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
            duration: Some(crate::ConditionDuration {
                amount: 3,
                unit: crate::ConditionDurationUnit::Rounds,
                timing: None,
            }),
            note: None,
        })?;
    let revision = store.encounters().get("test")?.unwrap().revision;
    store
        .encounters()
        .advance_reviewed_turn("test", revision, |_| crate::EncounterTurnUpdate {
            turn: EncounterTurnState {
                participant_key: Some(p.participant_key.clone()),
                round_number: 1,
                mark_running: true,
            },
            durations: vec![(c.condition_id, 2)],
        })?;
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
