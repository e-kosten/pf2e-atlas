use super::*;
use rusqlite::params;

#[test]
fn restore_reserves_deleted_condition_ids_and_preserves_creation_baseline()
-> crate::LocalStateResult<()> {
    let source = fixture("export-condition-history")?;
    let participant = source
        .encounters()
        .add_participant("test", creature("Goblin", Some(12), 24))?;
    let live = source
        .encounters()
        .add_condition(event_condition(&participant.participant_key, "live"))?
        .condition_id;
    let deleted = source
        .encounters()
        .add_condition(event_condition(&participant.participant_key, "deleted"))?
        .condition_id;
    source
        .encounters()
        .remove_condition(&participant.participant_key, deleted)?;
    let source_db = source.connection()?;
    source_db.execute(
        "INSERT INTO encounter_condition_applications(encounter_id,participant_key,application_id,grant_slot,condition_id)
         SELECT id,?1,'deleted','slot',?2 FROM encounters WHERE slug='test'",
        params![participant.participant_key, deleted],
    )?;
    let mut document = EncountersRead::from_connection(&source_db, 1).export_document("test")?;
    let bytes = document.encode()?;
    let target = TestStore::open(temp_path("import-condition-history"))?;
    let mut db = target.connection()?;
    let tx = db.transaction()?;
    let writer = EncountersWrite::from_connection(&tx, 1);
    let imported = writer.import_document(EncounterDocument::read(bytes.as_slice())?, None)?;
    let restored = writer
        .get_with_participants(&imported.encounter_key)?
        .unwrap();
    let copy = &restored.participants[0];
    assert_ne!(copy.participant_key, participant.participant_key);
    assert_eq!(copy.current_hp, Some(24));
    let baseline = writer.participant_baseline(&copy.participant_key)?.unwrap();
    assert_eq!(baseline.current_hp, Some(24));
    assert!(baseline.conditions.is_empty());
    let (history, availability): (i64, String) = tx.query_row(
        "SELECT a.condition_id,r.availability FROM encounter_condition_applications a
         JOIN encounter_participants p USING(participant_key) JOIN record_identity r ON r.key=p.record_key",
        [], |row| Ok((row.get(0)?, row.get(1)?)))?;
    assert_eq!(availability, "unavailable");
    assert_eq!(copy.conditions.len(), 1);
    assert_ne!(copy.conditions[0].condition_id, history);
    let next = writer
        .add_condition(event_condition(&copy.participant_key, "later"))?
        .condition_id;
    assert!(next > history);
    tx.commit()?;
    assert_eq!(
        source
            .encounters()
            .get_with_participants("test")?
            .unwrap()
            .participants[0]
            .conditions[0]
            .condition_id,
        live
    );
    Ok(())
}

#[test]
fn import_failure_leaves_no_identity_or_encounter_and_can_be_retried() -> crate::LocalStateResult<()>
{
    let source = fixture("export-rollback")?;
    source
        .encounters()
        .add_participant("test", creature("Goblin", None, 24))?;
    let source_db = source.connection()?;
    let bytes = EncountersRead::from_connection(&source_db, 1)
        .export_document("test")?
        .encode()?;
    let target = TestStore::open(temp_path("import-rollback"))?;
    let mut db = target.connection()?;
    let tx = db.transaction()?;
    tx.execute_batch(
        "CREATE TRIGGER reject_baseline BEFORE INSERT ON encounter_participant_baselines
        BEGIN SELECT RAISE(ABORT, 'injected portability failure'); END;",
    )?;
    let writer = EncountersWrite::from_connection(&tx, 1);
    assert!(
        writer
            .import_document(EncounterDocument::read(bytes.as_slice())?, None)
            .is_err()
    );
    assert!(writer.list()?.is_empty());
    assert_eq!(
        tx.query_row("SELECT COUNT(*) FROM record_identity", [], |row| row
            .get::<_, i64>(0))?,
        0
    );
    tx.execute_batch("DROP TRIGGER reject_baseline;")?;
    writer.import_document(EncounterDocument::read(bytes.as_slice())?, None)?;
    tx.commit()?;
    Ok(())
}

#[test]
fn rejects_out_of_range_order_before_creating_a_copy() -> crate::LocalStateResult<()> {
    let source = fixture("export-invalid-order")?;
    source
        .encounters()
        .add_participant("test", creature("Goblin", Some(10), 24))?;
    let mut db = source.connection()?;
    let tx = db.transaction()?;
    let writer = EncountersWrite::from_connection(&tx, 1);
    let bytes = writer.export_document("test")?.encode()?;
    let mut json: serde_json::Value = serde_json::from_slice(&bytes)?;
    json["snapshot"]["participants"][0]["initiative_order"] = i64::MAX.into();
    let malformed = serde_json::to_vec(&json)?;
    assert!(EncounterDocument::read(malformed.as_slice()).is_err());
    assert_eq!(writer.list()?.len(), 1);
    Ok(())
}

#[test]
fn portability_inventory_covers_current_encounter_tables() -> crate::LocalStateResult<()> {
    let source = fixture("encounter-portability-schema")?;
    let db = source.connection()?;
    let mut query = db.prepare(
        "SELECT name FROM sqlite_schema WHERE type='table'
        AND (name='encounters' OR name GLOB 'encounter_*') ORDER BY name",
    )?;
    let names = query
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        names,
        [
            "encounter_ability_states",
            "encounter_boundary_claims",
            "encounter_command_receipts",
            "encounter_condition_applications",
            "encounter_expended_spells",
            "encounter_manual_trackers",
            "encounter_outgoing_turns",
            "encounter_participant_baselines",
            "encounter_participant_conditions",
            "encounter_participants",
            "encounter_pending_consequences",
            "encounter_persistent_precedence",
            "encounter_spell_resources",
            "encounters",
        ],
        "review exchange coverage when a saved encounter table changes"
    );
    Ok(())
}

#[test]
fn copy_remaps_pending_decisions_and_preserves_source_bound_receipts() -> crate::LocalStateResult<()>
{
    let source = fixture("portable-pending")?;
    let actor = source
        .encounters()
        .add_participant("test", creature("Actor", Some(12), 24))?;
    let enemy = source
        .encounters()
        .add_participant("test", creature("Enemy", Some(10), 15))?;
    let snapshot = source.encounters().get_with_participants("test")?.unwrap();
    let key = &snapshot.encounter.encounter_key;
    let mut actor_change = ParticipantEventChange::preserving(&actor);
    actor_change.current_hp = Some(0);
    let pending = SavedPendingConsequence {
        consequence_key: "original:hp-lifecycle".into(),
        consequence: PendingConsequence::HpLifecycle {
            participant_key: actor.participant_key.clone(),
            before_hp: 24,
            after_hp: 0,
            damage_amount: 24,
            was_dying: 0,
            was_wounded: 0,
            was_doomed: 0,
            context: SavedHpEventContext {
                responsible_participant_key: Some(enemy.participant_key.clone()),
                ..Default::default()
            },
        },
    };
    let intent =
        serde_json::json!({"action":"persistent_damage","draft_intent_v1":"opaque-source-hash"});
    source.encounters().apply_condition_event(
        key,
        "original",
        &intent,
        snapshot.encounter.revision,
        PreparedConditionEvent {
            participant_changes: vec![actor_change],
            pending: vec![pending],
            ..Default::default()
        },
    )?;
    let mut db = source.connection()?;
    let tx = db.transaction()?;
    let writer = EncountersWrite::from_connection(&tx, 1);
    let bytes = writer.export_document(key)?.encode()?;
    let copy = writer.import_document(
        EncounterDocument::read(bytes.as_slice())?,
        Some("copy".into()),
    )?;
    let copy_key = &copy.encounter_key;
    let restored = writer.get_with_participants(copy_key)?.unwrap();
    let saved = writer.pending_condition_consequences(copy_key)?;
    let [decision] = saved.as_slice() else {
        panic!("expected pending HP decision")
    };
    let PendingConsequence::HpLifecycle {
        participant_key,
        context,
        ..
    } = &decision.consequence
    else {
        panic!("expected HP lifecycle");
    };
    assert_eq!(participant_key, &restored.participants[0].participant_key);
    assert_ne!(participant_key, &actor.participant_key);
    assert_eq!(
        context.responsible_participant_key.as_ref(),
        Some(&restored.participants[1].participant_key)
    );
    let recovered = writer
        .condition_event_receipt(copy_key, "original", &intent)?
        .unwrap();
    assert_eq!(recovered.encounter_key, *copy_key);
    assert_eq!(recovered.pending, saved);
    assert!(
        writer
            .condition_event_receipt(
                copy_key,
                "original",
                &serde_json::json!({"action":"different"})
            )
            .is_err()
    );
    assert_eq!(
        writer
            .participant_baseline(participant_key)?
            .unwrap()
            .current_hp,
        Some(24)
    );
    assert_eq!(restored.participants[0].current_hp, Some(0));
    tx.commit()?;
    Ok(())
}

#[test]
fn tracker_order_survives_updates_recovery_and_export_restore() -> crate::LocalStateResult<()> {
    let store = fixture("portable-tracker-order")?;
    let actor = store
        .encounters()
        .add_participant("test", creature("Actor", Some(12), 24))?;
    let mut db = store.connection()?;
    let tx = db.transaction()?;
    let writer = EncountersWrite::from_connection(&tx, 1);
    let encounter = writer.get("test")?.unwrap();
    let key = &encounter.encounter_key;
    let tracker = |key: &str, name: &str| ManualTracker {
        tracker_key: key.into(),
        name: name.into(),
        duration: ConditionDuration {
            amount: 2,
            unit: ConditionDurationUnit::Rounds,
            timing: Some(ConditionTiming {
                participant_key: actor.participant_key.clone(),
                event: ConditionTurnEvent::End,
            }),
            remaining_seconds: None,
        },
        progress: TrackerProgress::Rounds(2),
    };
    let save = |tracker_key: &str, value: Option<ManualTracker>| {
        writer.save_manual_tracker(
            key,
            &ManualTrackerChange {
                participant_key: actor.participant_key.clone(),
                tracker_key: tracker_key.into(),
                value,
            },
        )
    };
    let trackers = |key: &str| -> crate::LocalStateResult<Vec<String>> {
        Ok(writer.get_with_participants(key)?.unwrap().participants[0]
            .manual_trackers
            .iter()
            .map(|tracker| tracker.name.clone())
            .collect())
    };

    save("z", Some(tracker("z", "First")))?;
    save("a", Some(tracker("a", "Second")))?;
    let mut renamed = tracker("z", "Renamed");
    renamed.progress = TrackerProgress::Rounds(1);
    save("z", Some(renamed))?;
    assert_eq!(trackers(key)?, ["Renamed", "Second"]);
    save("z", None)?;
    save("z", Some(tracker("z", "Re-added")))?;
    assert_eq!(trackers(key)?, ["Second", "Re-added"]);

    let snapshot = writer.get_with_participants(key)?.unwrap();
    let mut gameplay = ParticipantGameplay::from(&snapshot.participants[0]);
    gameplay.manual_trackers.reverse();
    writer.apply_recovery(ApplyEncounterRecovery {
        encounter_key: key,
        expected_revision: snapshot.encounter.revision,
        reset_encounter: false,
        end_turn: false,
        replacements: &[(actor.participant_key.clone(), gameplay)],
    })?;
    assert_eq!(trackers(key)?, ["Re-added", "Second"]);

    let bytes = writer.export_document(key)?.encode()?;
    let copied = writer.import_document(
        EncounterDocument::read(bytes.as_slice())?,
        Some("tracker-order-copy".into()),
    )?;
    assert_eq!(trackers(&copied.encounter_key)?, ["Re-added", "Second"]);
    Ok(())
}
