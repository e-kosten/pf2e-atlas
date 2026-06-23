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
        source_note: None,
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
            source_note: None,
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
        source_note: None,
    })?;
    assert!(condition.condition_id > 0);
    assert!(encounters.delete("conditions")?);
    Ok(())
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
