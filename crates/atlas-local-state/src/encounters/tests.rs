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
        duration_rounds: Some(2),
        note: Some("From spell".to_string()),
    })?;
    assert_eq!(condition.name, "Frightened");

    let updated = encounters
        .update_condition(UpdateEncounterParticipantCondition {
            condition_id: condition.condition_id,
            condition_key: condition.condition_key,
            name: "Frightened".to_string(),
            value: Some(2),
            source_participant_key: None,
            duration_rounds: Some(1),
            note: Some("Raised".to_string()),
        })?
        .expect("condition should update");
    assert_eq!(updated.value, Some(2));

    let detail = encounters
        .get_with_participants("conditions")?
        .expect("encounter should exist");
    assert_eq!(detail.participants[0].conditions.len(), 1);
    assert!(encounters.remove_condition(condition.condition_id)?);
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
        duration_rounds: None,
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
            participant_key: participant.participant_key.clone(),
            display_name: participant.display_name,
            side: participant.side,
            participant_variant: ParticipantVariant::Elite,
            hazard_state: participant.hazard_state,
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
fn participant_reset_restores_creation_mechanics_and_preserves_authored_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("participant-reset"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "reset".to_string(),
        name: "Reset".to_string(),
        description: None,
        note: None,
    })?;
    let target = EncounterSpellResourceTarget::InnateUse {
        entry_id: Some("entry-innate".to_string()),
        spell_occurrence_id: "spell-shadow-blast".to_string(),
    };
    let original = encounters.add_participant_with_spell_resources(
        "reset",
        creature("Original Name", Some(18), 30),
        &[spell_resource(target.clone(), 2, 2)],
    )?;
    encounters.add_participant("reset", creature("Tie Peer", Some(18), 10))?;
    encounters.add_condition(AddEncounterParticipantCondition {
        participant_key: original.participant_key.clone(),
        condition_key: Some("conditions:slowed".to_string()),
        name: "Slowed".to_string(),
        value: Some(1),
        source_participant_key: None,
        duration_rounds: None,
        note: None,
    })?;
    encounters.mutate_spell_resource(
        &original.participant_key,
        &target,
        EncounterSpellResourceOperation::CastOne,
    )?;
    encounters.set_current_turn("reset", Some(&original.participant_key))?;
    encounters.update_participant(UpdateEncounterParticipant {
        participant_key: original.participant_key.clone(),
        display_name: "Custom Name".to_string(),
        side: ParticipantSide::Ally,
        participant_variant: ParticipantVariant::Elite,
        hazard_state: ParticipantHazardState::Disabled,
        initiative: Some(12),
        max_hp: Some(42),
        current_hp: Some(3),
        temporary_hp: 7,
        defeated: true,
        hidden: true,
        note: Some("Preserve this note".to_string()),
    })?;

    assert!(encounters.participant_reset_available(&original.participant_key)?);
    let reset = encounters.reset_participant(&original.participant_key)?;
    assert!(reset.cleared_current_turn);
    assert_eq!(reset.participant.display_name, "Custom Name");
    assert_eq!(reset.participant.side, ParticipantSide::Ally);
    assert!(reset.participant.hidden);
    assert_eq!(
        reset.participant.note.as_deref(),
        Some("Preserve this note")
    );
    assert_eq!(
        reset.participant.participant_variant,
        ParticipantVariant::Normal
    );
    assert_eq!(
        reset.participant.hazard_state,
        ParticipantHazardState::Active
    );
    assert!(
        reset
            .reset_domains
            .contains(&EncounterParticipantResetDomain::HazardState)
    );
    assert_eq!(reset.participant.initiative, Some(18));
    assert_eq!(
        reset.participant.initiative_order,
        original.initiative_order
    );
    assert_eq!(reset.participant.max_hp, Some(30));
    assert_eq!(reset.participant.current_hp, Some(30));
    assert_eq!(reset.participant.temporary_hp, 0);
    assert!(!reset.participant.defeated);
    assert!(reset.participant.conditions.is_empty());
    let detail = encounters
        .get_with_participants("reset")?
        .expect("encounter should exist");
    assert_eq!(detail.encounter.current_turn_participant_key, None);
    let spell_state = encounters.spell_state(&original.participant_key)?;
    assert_eq!(spell_state.resources[0].remaining, 2);
    assert_eq!(spell_state.resources[0].initial_remaining, 2);
    Ok(())
}

#[test]
fn participant_reset_failure_rolls_back_every_mechanical_domain()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_path("participant-reset-rollback");
    let store = LocalStateStore::open(&path)?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "reset-rollback".to_string(),
        name: "Reset Rollback".to_string(),
        description: None,
        note: None,
    })?;
    let target = EncounterSpellResourceTarget::InnateUse {
        entry_id: Some("entry-innate".to_string()),
        spell_occurrence_id: "spell-shadow-blast".to_string(),
    };
    let original = encounters.add_participant_with_spell_resources(
        "reset-rollback",
        creature("Original Name", Some(18), 30),
        &[spell_resource(target.clone(), 2, 2)],
    )?;
    encounters.add_condition(AddEncounterParticipantCondition {
        participant_key: original.participant_key.clone(),
        condition_key: Some("conditions:slowed".to_string()),
        name: "Slowed".to_string(),
        value: Some(1),
        source_participant_key: None,
        duration_rounds: Some(2),
        note: Some("Must survive rollback".to_string()),
    })?;
    encounters.mutate_spell_resource(
        &original.participant_key,
        &target,
        EncounterSpellResourceOperation::CastOne,
    )?;
    encounters.set_current_turn("reset-rollback", Some(&original.participant_key))?;
    encounters.update_participant(UpdateEncounterParticipant {
        participant_key: original.participant_key.clone(),
        display_name: "Preserved Custom Name".to_string(),
        side: ParticipantSide::Ally,
        participant_variant: ParticipantVariant::Elite,
        hazard_state: ParticipantHazardState::Disabled,
        initiative: Some(12),
        max_hp: Some(42),
        current_hp: Some(3),
        temporary_hp: 7,
        defeated: true,
        hidden: true,
        note: Some("Preserved note".to_string()),
    })?;
    {
        let connection = rusqlite::Connection::open(&path)?;
        connection.execute(
            "INSERT INTO encounter_participant_adjustments (
                 participant_id, adjustment_key, kind, value, created_at, updated_at
             ) SELECT id, 'fixture-adjustment', 'fixture', 'keep', created_at, updated_at
               FROM encounter_participants WHERE participant_key = ?1",
            rusqlite::params![original.participant_key],
        )?;
    }

    let before = reset_rollback_snapshot(&path, &store, &original.participant_key)?;
    assert!(matches!(
        encounters.reset_participant_with_injected_failure(&original.participant_key),
        Err(crate::LocalStateError::InjectedResetFailure)
    ));
    let after = reset_rollback_snapshot(&path, &store, &original.participant_key)?;

    assert_eq!(
        after, before,
        "rollback must preserve the exact state bytes"
    );
    Ok(())
}

fn reset_rollback_snapshot(
    path: &std::path::Path,
    store: &LocalStateStore,
    participant_key: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let detail = store
        .encounters()
        .get_with_participants("reset-rollback")?
        .expect("encounter should exist");
    let spell_state = store.encounters().spell_state(participant_key)?;
    let connection = rusqlite::Connection::open(path)?;
    let adjustment_rows = connection.query_row(
        "SELECT COUNT(*) FROM encounter_participant_adjustments adjustment
         JOIN encounter_participants participant ON participant.id = adjustment.participant_id
         WHERE participant.participant_key = ?1",
        rusqlite::params![participant_key],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(format!("{detail:?}\n{spell_state:?}\n{adjustment_rows}").into_bytes())
}

#[test]
fn participant_creation_rolls_back_when_spell_baseline_is_invalid()
-> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("participant-baseline-atomic"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "atomic".to_string(),
        name: "Atomic".to_string(),
        description: None,
        note: None,
    })?;
    let invalid = EncounterSpellResource {
        target: EncounterSpellResourceTarget::SpontaneousPool {
            entry_id: "entry-spontaneous".to_string(),
            rank: 3,
        },
        maximum: 1,
        initial_remaining: 2,
        remaining: 2,
    };
    assert!(
        encounters
            .add_participant_with_spell_resources(
                "atomic",
                creature("Invalid", Some(12), 10),
                &[invalid],
            )
            .is_err()
    );
    assert!(
        encounters
            .get_with_participants("atomic")?
            .expect("encounter should exist")
            .participants
            .is_empty()
    );
    Ok(())
}

#[test]
fn spell_resources_preserve_typed_ownership_and_bounded_cast_restore_state()
-> Result<(), Box<dyn std::error::Error>> {
    let store = LocalStateStore::open(temp_path("spell-resources"))?;
    let encounters = store.encounters();
    encounters.create(NewEncounter {
        slug: "spell-resources".to_string(),
        name: "Spell Resources".to_string(),
        description: None,
        note: None,
    })?;
    let participant =
        encounters.add_participant("spell-resources", creature("Spellcaster", Some(20), 30))?;
    let prepared = EncounterSpellResourceTarget::PreparedSlot {
        entry_id: "entry-prepared".to_string(),
        spell_occurrence_id: "spell-fireball".to_string(),
        rank: 4,
        slot_id: "slot4:0".to_string(),
    };
    let spontaneous = EncounterSpellResourceTarget::SpontaneousPool {
        entry_id: "entry-spontaneous".to_string(),
        rank: 3,
    };
    let innate = EncounterSpellResourceTarget::InnateUse {
        entry_id: Some("entry-innate".to_string()),
        spell_occurrence_id: "spell-shadow-blast".to_string(),
    };
    let focus = EncounterSpellResourceTarget::FocusPool {
        resource_id: "resource:focus".to_string(),
    };
    let resources = vec![
        spell_resource(prepared.clone(), 1, 1),
        spell_resource(spontaneous.clone(), 3, 2),
        spell_resource(innate.clone(), 2, 2),
        spell_resource(focus.clone(), 1, 1),
    ];

    let initialized =
        encounters.initialize_spell_state(&participant.participant_key, &resources)?;
    assert!(initialized.initialized);
    assert_eq!(initialized.resources.len(), 4);

    let cast = encounters.mutate_spell_resource(
        &participant.participant_key,
        &innate,
        EncounterSpellResourceOperation::CastOne,
    )?;
    assert_eq!(cast.before.remaining, 2);
    assert_eq!(cast.after.remaining, 1);
    let restored = encounters.mutate_spell_resource(
        &participant.participant_key,
        &innate,
        EncounterSpellResourceOperation::RestoreOne,
    )?;
    assert_eq!(restored.after.remaining, 2);
    assert!(matches!(
        encounters.mutate_spell_resource(
            &participant.participant_key,
            &innate,
            EncounterSpellResourceOperation::RestoreOne,
        ),
        Err(crate::LocalStateError::SpellResourceAtBaseline(_))
    ));

    encounters.mutate_spell_resource(
        &participant.participant_key,
        &focus,
        EncounterSpellResourceOperation::CastOne,
    )?;
    assert!(matches!(
        encounters.mutate_spell_resource(
            &participant.participant_key,
            &focus,
            EncounterSpellResourceOperation::CastOne,
        ),
        Err(crate::LocalStateError::SpellResourceExhausted(_))
    ));

    let ignored_reinitialization = encounters.initialize_spell_state(
        &participant.participant_key,
        &[spell_resource(spontaneous, 9, 9)],
    )?;
    assert_eq!(ignored_reinitialization.resources.len(), 4);
    assert!(ignored_reinitialization.resources.iter().any(|resource| {
        resource.target == focus && resource.remaining == 0 && resource.initial_remaining == 1
    }));
    assert!(
        ignored_reinitialization
            .resources
            .iter()
            .any(|resource| { resource.target == prepared && resource.remaining == 1 })
    );

    let second = encounters.add_participant(
        "spell-resources",
        creature("Second Spellcaster", Some(10), 30),
    )?;
    encounters.initialize_spell_state(
        &second.participant_key,
        &[spell_resource(innate.clone(), 2, 2)],
    )?;
    encounters.mutate_spell_resource(
        &participant.participant_key,
        &innate,
        EncounterSpellResourceOperation::CastOne,
    )?;
    let first_state = encounters.spell_state(&participant.participant_key)?;
    let second_state = encounters.spell_state(&second.participant_key)?;
    assert!(
        first_state
            .resources
            .iter()
            .any(|resource| { resource.target == innate && resource.remaining == 1 })
    );
    assert!(
        second_state
            .resources
            .iter()
            .any(|resource| { resource.target == innate && resource.remaining == 2 })
    );
    assert!(matches!(
        encounters.mutate_spell_resource(
            &participant.participant_key,
            &EncounterSpellResourceTarget::InnateUse {
                entry_id: Some("entry-innate".to_string()),
                spell_occurrence_id: "spell-other".to_string(),
            },
            EncounterSpellResourceOperation::CastOne,
        ),
        Err(crate::LocalStateError::SpellResourceNotFound(_))
    ));
    Ok(())
}

fn spell_resource(
    target: EncounterSpellResourceTarget,
    maximum: i64,
    remaining: i64,
) -> EncounterSpellResource {
    EncounterSpellResource {
        target,
        maximum,
        initial_remaining: remaining,
        remaining,
    }
}

fn creature(name: &str, initiative: Option<i64>, hp: i64) -> AddEncounterParticipant {
    AddEncounterParticipant {
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
