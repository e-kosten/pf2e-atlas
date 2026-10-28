//! Composition of domain-owned schemas into one ordered workspace history.
use atlas_storage::{Migration, StorageResult, Workspace};
use std::path::Path;

pub const WORKSPACE_SCHEMA_VERSION: u32 = 19;
const MIGRATIONS: &[Migration] = &[
    Migration {
        backfill: None,
        version: 1,
        name: "reference-baseline",
        sql: atlas_index::REFERENCE_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 2,
        name: "saved-state-baseline",
        sql: atlas_local_state::LOCAL_STATE_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 3,
        name: "lexical-alias-rows",
        sql: atlas_index::LEXICAL_ALIAS_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 4,
        name: "consumable-filters",
        sql: atlas_index::CONSUMABLE_FILTER_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 5,
        name: "encounter-status",
        sql: atlas_local_state::ENCOUNTER_STATUS_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 6,
        name: "hazard-participation",
        sql: atlas_local_state::HAZARD_PARTICIPATION_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 7,
        name: "encounter-parties",
        sql: atlas_local_state::PARTIES_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 8,
        name: "trait-vocabulary",
        sql: atlas_index::TRAIT_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 9,
        name: "condition-system",
        sql: atlas_local_state::CONDITION_SYSTEM_SCHEMA_SQL,
    },
    Migration {
        version: 10,
        name: "embedding-storage",
        sql: atlas_index::EMBEDDING_STORAGE_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            id: "native-embedding-storage/v1",
            apply: atlas_index::backfill_embedding_storage,
        }),
    },
    Migration {
        version: 11,
        name: "persistent-preparation",
        sql: atlas_local_state::PERSISTENT_PREPARATION_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 12,
        name: "turn-resources",
        sql: atlas_local_state::TURN_RESOURCES_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 13,
        name: "consumed-condition-applications",
        sql: atlas_local_state::CONSUMED_CONDITION_APPLICATIONS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 14,
        name: "party-member-levels",
        sql: atlas_local_state::PARTY_LEVELS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 15,
        name: "encounter-difficulty",
        sql: atlas_local_state::ENCOUNTER_DIFFICULTY_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 16,
        name: "encounter-ability-state",
        sql: atlas_local_state::ABILITY_STATE_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 17,
        name: "stateless-strike-rolls",
        sql: atlas_local_state::STATELESS_STRIKE_ROLLS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 18,
        name: "reference-numeric-filters",
        sql: atlas_index::REFERENCE_NUMERIC_FILTER_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 19,
        name: "physical-base-type",
        sql: atlas_index::PHYSICAL_BASE_TYPE_SCHEMA_SQL,
        backfill: None,
    },
];

/// Initialize or upgrade one workspace. Reference readiness is independent:
/// an empty reference corpus must not prevent creating saved lists or encounters.
pub fn initialize_workspace(path: impl AsRef<Path>) -> StorageResult<Workspace> {
    register_extensions()?;
    let workspace = Workspace::at(path)?;
    workspace.migrate(MIGRATIONS, |_| Ok(()))?;
    Ok(workspace)
}

pub(crate) fn open_workspace_index(
    path: &Path,
) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexError> {
    register_extensions().map_err(|e| atlas_index::IndexError::Unavailable(e.to_string()))?;
    let workspace =
        Workspace::at(path).map_err(|e| atlas_index::IndexError::Unavailable(e.to_string()))?;
    let session = workspace
        .read(WORKSPACE_SCHEMA_VERSION)
        .map_err(|e| atlas_index::IndexError::Unavailable(e.to_string()))?;
    atlas_index::SqliteIndexReader::from_session(session)
}

pub(crate) fn register_extensions() -> StorageResult<()> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| atlas_storage::StorageError::Unsupported(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_local_state::{LocalStateRead, LocalStateWrite, NewSavedList};

    #[test]
    fn reference_filter_upgrade_preserves_saved_lists_from_published_schema_seventeen() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("reference-upgrade.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        workspace.migrate(&MIGRATIONS[..17], |_| Ok(())).unwrap();
        let write = workspace.write(17).unwrap();
        LocalStateWrite::new(&write)
            .saved_lists()
            .create(NewSavedList {
                slug: "campaign".into(),
                name: "Campaign".into(),
                description: Some("Keep these notes".into()),
                tags: vec!["review".into()],
            })
            .unwrap();
        write.commit().unwrap();
        let before = {
            let read = workspace.read(17).unwrap();
            serde_json::to_value(LocalStateRead::new(&read).saved_lists().list().unwrap()).unwrap()
        };
        let upgraded = initialize_workspace(&path).unwrap();
        let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let after = LocalStateRead::new(&read).saved_lists().list().unwrap();
        assert_eq!(serde_json::to_value(after).unwrap(), before);
    }

    #[test]
    fn stateless_strike_upgrade_removes_only_display_roll_receipts() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("stateless-strikes.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        workspace.migrate(&MIGRATIONS[..16], |_| Ok(())).unwrap();
        let write = workspace.write(16).unwrap();
        write.connection().execute_batch(r#"
            INSERT INTO encounters(id,encounter_key,slug,name,status,current_turn_participant_key,round_number,revision,created_at,updated_at)
            VALUES(1,'kept','kept','Kept','running','actor',4,7,'before','before');
            INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,current_hp,max_hp,temporary_hp,revision,created_at,updated_at)
            VALUES(1,1,'actor','pc',0,'PC',0,13,20,3,5,'before','before');
            INSERT INTO encounter_roll_receipts VALUES
                (1,'attack','{"action":"strike"}','{"version":1,"payload":{"action":"strike","receipt":{"roll":{"outcome":{"kind":"attack"}}}}}'),
                (1,'damage','{"action":"strike"}','{"version":1,"payload":{"action":"strike","receipt":{"roll":{"outcome":{"kind":"damage"}}}}}'),
                (1,'initiative','{"participants":["actor"]}','{"version":1,"payload":{"rolls":[17]}}'),
                (1,'unknown','{"action":"future"}','{"version":2,"payload":{"action":"future"}}'),
                (1,'malformed','not json','not json'),
                (1,'unidentified','{"action":"strike"}','{"version":2,"payload":{"action":"future"}}');
            INSERT INTO encounter_event_receipts VALUES(1,'prepared','{}','{"version":1,"payload":{"prepared":"persistent_damage"}}');
            INSERT INTO encounter_boundary_claims VALUES(1,'actor',4,'prepared_persistent_damage','fire','prepared');
        "#).unwrap();
        let before = LocalStateWrite::new(&write)
            .encounters()
            .get_with_participants("kept")
            .unwrap();
        let retained = |connection: &rusqlite::Connection| {
            let mut statement = connection.prepare("SELECT request_id,request_json,receipt_json FROM encounter_roll_receipts WHERE request_id NOT IN ('attack','damage') ORDER BY request_id").unwrap();
            statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        let receipts = retained(write.connection());
        write.commit().unwrap();
        let upgraded = initialize_workspace(&path).unwrap();
        let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let after = LocalStateRead::new(&read)
            .encounters()
            .get_with_participants("kept")
            .unwrap();
        assert_eq!(
            serde_json::to_value(before).unwrap(),
            serde_json::to_value(after).unwrap()
        );
        assert_eq!(retained(read.connection()), receipts);
        let strike_count: i64 = read.connection().query_row("SELECT COUNT(*) FROM encounter_roll_receipts WHERE request_id IN ('attack','damage')", [], |row| row.get(0)).unwrap();
        assert_eq!(strike_count, 0);
        let preparation: (String,String) = read.connection().query_row("SELECT r.receipt_json,c.request_id FROM encounter_event_receipts r JOIN encounter_boundary_claims c USING(encounter_id,request_id)", [], |row| Ok((row.get(0)?,row.get(1)?))).unwrap();
        assert_eq!(
            preparation,
            (
                r#"{"version":1,"payload":{"prepared":"persistent_damage"}}"#.into(),
                "prepared".into()
            )
        );
        drop(read);
        // Reopening an already migrated workspace preserves the surviving receipts.
        let reopened = initialize_workspace(&path).unwrap();
        let read = reopened.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(retained(read.connection()), receipts);
    }

    #[test]
    fn consumed_application_upgrade_preserves_markers_receipts_and_saved_state() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("consumed-applications.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        workspace.migrate(&MIGRATIONS[..12], |_| Ok(())).unwrap();
        let write = workspace.write(12).unwrap();
        write.connection().execute_batch(r#"
            INSERT INTO encounters(id,encounter_key,slug,name,status,current_turn_participant_key,round_number,revision,created_at,updated_at)
            VALUES(1,'kept','kept','Kept','running','actor',4,7,'before','before');
            INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,current_hp,max_hp,temporary_hp,revision,created_at,updated_at)
            VALUES(1,1,'actor','pc',0,'PC',0,13,20,3,5,'before','before');
            INSERT INTO encounter_participant_conditions(id,participant_id,name,value,note,created_at,updated_at)
            VALUES(1,1,'Slowed',1,'Keep note','before','before');
            INSERT INTO encounter_condition_applications VALUES(1,'actor','kept-application','condition',1);
            INSERT INTO encounter_condition_applications VALUES(1,'actor','removed-application','condition',99);
            INSERT INTO encounter_event_receipts VALUES(1,'kept-request','{}','{"version":1,"payload":{}}');
        "#).unwrap();
        // Read the old schema directly: current domain readers require all current columns.
        let snapshot = |connection: &rusqlite::Connection| {
            connection.query_row("SELECT name,status,round_number,current_turn_participant_key,revision FROM encounters WHERE encounter_key='kept'", [], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,i64>(2)?,row.get::<_,String>(3)?,row.get::<_,i64>(4)?))).unwrap()
        };
        let before = snapshot(write.connection());
        write.commit().unwrap();
        let upgraded = initialize_workspace(&path).unwrap();
        {
            let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
            assert_eq!(snapshot(read.connection()), before);
            let detail = LocalStateRead::new(&read)
                .encounters()
                .get_with_participants("kept")
                .unwrap()
                .unwrap();
            assert_eq!(detail.encounter.difficulty_reference, Default::default());
            assert_eq!(
                (
                    detail.participants[0].current_hp,
                    detail.participants[0].max_hp,
                    detail.participants[0].temporary_hp,
                    detail.participants[0].revision
                ),
                (Some(13), Some(20), 3, 5)
            );
            assert_eq!(
                detail.participants[0].conditions[0].note.as_deref(),
                Some("Keep note")
            );
            let mut statement = read.connection().prepare("SELECT application_id,condition_id FROM encounter_condition_applications ORDER BY application_id").unwrap();
            let markers = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?))
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(
                markers,
                vec![
                    ("kept-application".into(), Some(1)),
                    ("removed-application".into(), Some(99))
                ]
            );
            let receipt: String = read.connection().query_row("SELECT receipt_json FROM encounter_event_receipts WHERE request_id='kept-request'", [], |row| row.get(0)).unwrap();
            assert_eq!(receipt, r#"{"version":1,"payload":{}}"#);
        }
        let write = upgraded.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        write.connection().execute("INSERT INTO encounter_condition_applications VALUES(1,'actor','consumed','condition',NULL)", []).unwrap();
        assert!(write.connection().execute("INSERT INTO encounter_condition_applications VALUES(1,'actor','consumed','condition',NULL)", []).is_err());
        write
            .connection()
            .execute(
                "DELETE FROM encounter_participants WHERE participant_key='actor'",
                [],
            )
            .unwrap();
        let markers: i64 = write
            .connection()
            .query_row(
                "SELECT count(*) FROM encounter_condition_applications",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(markers, 0);
        let receipts: i64 = write
            .connection()
            .query_row("SELECT count(*) FROM encounter_event_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            receipts, 1,
            "receipts survive until the encounter is deleted"
        );
        write
            .connection()
            .execute("DELETE FROM encounters WHERE encounter_key='kept'", [])
            .unwrap();
        let receipts: i64 = write
            .connection()
            .query_row("SELECT count(*) FROM encounter_event_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(receipts, 0);
        write.commit().unwrap();
    }

    #[test]
    fn turn_resource_upgrade_preserves_state_without_inventing_an_allocation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("resources.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        workspace.migrate(&MIGRATIONS[..11], |_| Ok(())).unwrap();
        let write = workspace.write(11).unwrap();
        write.connection().execute_batch(r#"
            INSERT INTO encounters(id,encounter_key,slug,name,status,current_turn_participant_key,round_number,revision,created_at,updated_at)
            VALUES(1,'kept','kept','Kept','running','actor',4,7,'before','before');
            INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,current_hp,max_hp,temporary_hp,revision,created_at,updated_at)
            VALUES(1,1,'actor','pc',0,'PC',0,13,20,3,5,'before','before');
            INSERT INTO encounter_participant_conditions(participant_id,name,value,note,created_at,updated_at)
            VALUES(1,'Slowed',1,'Keep note','before','before');
            INSERT INTO encounter_event_receipts VALUES(1,'kept-request','{}','{"version":1,"payload":{}}');
        "#).unwrap();
        write.commit().unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let detail = LocalStateRead::new(&read)
            .encounters()
            .get_with_participants("kept")
            .unwrap()
            .unwrap();
        assert_eq!(detail.encounter.revision, 7);
        assert_eq!(detail.encounter.round_number, 4);
        assert_eq!(
            detail.encounter.current_turn_participant_key.as_deref(),
            Some("actor")
        );
        let p = &detail.participants[0];
        assert_eq!(
            (p.current_hp, p.max_hp, p.temporary_hp, p.revision),
            (Some(13), Some(20), 3, 5)
        );
        assert!(p.turn_resources.is_none());
        assert_eq!(p.conditions[0].note.as_deref(), Some("Keep note"));
        let receipt: String = read
            .connection()
            .query_row(
                "SELECT receipt_json FROM encounter_event_receipts WHERE request_id='kept-request'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(receipt, r#"{"version":1,"payload":{}}"#);
        // Older outgoing snapshots have no resource field. They must still be
        // readable, and absence must remain unknown rather than fabricated 3/1.
        let mut old_snapshot = serde_json::to_value(&detail).unwrap();
        old_snapshot["participants"][0]
            .as_object_mut()
            .unwrap()
            .remove("turn_resources");
        let decoded: atlas_local_state::EncounterWithParticipants =
            serde_json::from_value(old_snapshot).unwrap();
        assert_eq!(decoded, detail);
    }

    #[test]
    fn persistent_preparation_upgrade_preserves_old_claims_hp_and_receipts() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preparation.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        workspace.migrate(&MIGRATIONS[..10], |_| Ok(())).unwrap();
        let write = workspace.write(10).unwrap();
        write.connection().execute("INSERT INTO encounters(encounter_key,slug,name,status,round_number,created_at,updated_at) VALUES('retained-key','retained','Retained','draft',1,'now','now')",[]).unwrap();
        let encounter_key = "retained-key";
        write.connection().execute("INSERT INTO encounter_participants (encounter_id,participant_key,participant_kind,position,display_name,initiative_order,max_hp,current_hp,temporary_hp,created_at,updated_at) SELECT id,'retained-actor','pc',1,'PC',1,20,13,3,'now','now' FROM encounters WHERE encounter_key=?1",[encounter_key]).unwrap();
        write.connection().execute("INSERT INTO encounter_event_receipts SELECT id,'retained-request','{}','{\"version\":1,\"payload\":{}}' FROM encounters WHERE encounter_key=?1",[encounter_key]).unwrap();
        write.connection().execute("INSERT INTO encounter_boundary_claims SELECT id,'retained-actor',4,'persistent_damage','','retained-request' FROM encounters WHERE encounter_key=?1",[encounter_key]).unwrap();
        write.commit().unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let row=read.connection().query_row("SELECT p.current_hp,p.temporary_hp,c.round_number,c.kind,c.request_id,r.receipt_json FROM encounter_participants p JOIN encounter_boundary_claims c ON c.participant_key=p.participant_key JOIN encounter_event_receipts r ON r.encounter_id=c.encounter_id AND r.request_id=c.request_id",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?))).unwrap();
        assert_eq!(
            row,
            (
                13,
                3,
                4,
                "persistent_damage".into(),
                "retained-request".into(),
                r#"{"version":1,"payload":{}}"#.into()
            )
        );
    }

    #[test]
    fn version_two_workspace_upgrade_preserves_saved_domains() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("atlas.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        workspace.migrate(&MIGRATIONS[..2], |_| Ok(())).unwrap();
        let write = workspace.write(2).unwrap();
        let state = LocalStateWrite::new(&write);
        state
            .saved_lists()
            .create(NewSavedList {
                slug: "campaign".into(),
                name: "Campaign".into(),
                description: None,
                tags: vec![],
            })
            .unwrap();
        write.connection().execute("INSERT INTO encounters(encounter_key,slug,name,status,round_number,created_at,updated_at) VALUES('ambush-key','ambush','Ambush','draft',1,'now','now')",[]).unwrap();
        write.commit().unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let state = LocalStateRead::new(&read);
        assert_eq!(state.saved_lists().list().unwrap().len(), 1);
        assert_eq!(state.encounters().list().unwrap().len(), 1);
        assert_eq!(read.reference_revision().unwrap(), 0);
    }

    #[test]
    fn encounter_status_upgrade_preserves_existing_hazard_hp_and_defaults_active() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("hazard.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        // Version four is the historical input under test, not the current version.
        workspace.migrate(&MIGRATIONS[..4], |_| Ok(())).unwrap();
        let write = workspace.write(4).unwrap();
        write.connection().execute_batch("INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at) VALUES(1,'e','e','Encounter','draft','before','before');
            INSERT INTO encounter_participants(encounter_id,participant_key,participant_kind,position,display_name,initiative_order,max_hp,current_hp,note,created_at,updated_at)
            VALUES(1,'h','hazard',0,'Hazard',0,30,17,'Keep this','before','before');").unwrap();
        write.commit().unwrap();
        let upgraded = initialize_workspace(&path).unwrap();
        let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let detail = LocalStateRead::new(&read)
            .encounters()
            .get_with_participants("e")
            .unwrap()
            .unwrap();
        let p = &detail.participants[0];
        assert!(!p.hazard_disabled);
        assert_eq!(p.current_hp, Some(17));
        assert_eq!(p.max_hp, Some(30));
        assert_eq!(p.note.as_deref(), Some("Keep this"));
        assert_eq!(p.revision, 0);
    }

    #[test]
    fn defeated_hazard_upgrade_preserves_hp_and_creature_status() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("hazard-status.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        // Version five is the historical input under test.
        workspace.migrate(&MIGRATIONS[..5], |_| Ok(())).unwrap();
        let write = workspace.write(5).unwrap();
        write.connection().execute_batch("INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at) VALUES(1,'e','e','Encounter','draft','before','before');
            INSERT INTO encounter_participants(encounter_id,participant_key,participant_kind,position,display_name,initiative_order,max_hp,current_hp,defeated,created_at,updated_at)
            VALUES(1,'h','hazard',0,'Hazard',0,30,17,1,'before','before'),
                  (1,'c','creature',1,'Creature',1,30,0,1,'before','before');").unwrap();
        write.commit().unwrap();
        let upgraded = initialize_workspace(&path).unwrap();
        let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let detail = LocalStateRead::new(&read)
            .encounters()
            .get_with_participants("e")
            .unwrap()
            .unwrap();
        let hazard = detail
            .participants
            .iter()
            .find(|p| p.participant_key == "h")
            .unwrap();
        assert!(hazard.hazard_disabled);
        assert!(!hazard.defeated);
        assert_eq!(hazard.current_hp, Some(17));
        let creature = detail
            .participants
            .iter()
            .find(|p| p.participant_key == "c")
            .unwrap();
        assert!(creature.defeated);
        assert!(!creature.hazard_disabled);
    }

    #[test]
    fn startup_can_initialize_current_workspace_during_publisher_preparation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("atlas.sqlite");
        let workspace = initialize_workspace(&path).unwrap();
        let _publisher = workspace.update_owner().unwrap();
        let reopened = initialize_workspace(&path).unwrap();
        assert_eq!(
            reopened
                .read(WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .reference_revision()
                .unwrap(),
            0
        );
    }

    #[test]
    fn invalid_new_domain_migration_rolls_back_entire_fresh_baseline() {
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace::at(directory.path().join("atlas.sqlite")).unwrap();
        let mut migrations = MIGRATIONS.to_vec();
        migrations.push(Migration {
            backfill: None,
            version: WORKSPACE_SCHEMA_VERSION + 1,
            name: "invalid",
            sql: "INSERT INTO missing_table VALUES(1);",
        });
        assert!(workspace.migrate(&migrations, |_| Ok(())).is_err());
        // A failed fresh initialization leaves no partially usable reference-only DB.
        let recovered = initialize_workspace(workspace.path()).unwrap();
        let read = recovered.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert!(
            LocalStateRead::new(&read)
                .saved_lists()
                .list()
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn parties_upgrade_preserves_saved_encounter_state_and_durable_rosters() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("party-upgrade.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        workspace.migrate(&MIGRATIONS[..6], |_| Ok(())).unwrap();
        let write = workspace.write(6).unwrap();
        write.connection().execute_batch("INSERT INTO encounters(encounter_key,slug,name,status,created_at,updated_at) VALUES('kept','kept','Kept','running','before','before');").unwrap();
        write.commit().unwrap();
        let upgraded = initialize_workspace(&path).unwrap();
        let write = upgraded.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        let party = LocalStateWrite::new(&write)
            .parties()
            .create(
                "Party",
                &[atlas_local_state::PartyMemberInput {
                    level: None,
                    member_key: None,
                    name: "PC".into(),
                }],
            )
            .unwrap();
        write.commit().unwrap();
        drop(upgraded);
        let reopened = initialize_workspace(&path).unwrap();
        let read = reopened.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let state = LocalStateRead::new(&read);
        assert_eq!(state.parties().get(&party.party_key).unwrap(), party);
        assert_eq!(
            state.encounters().get("kept").unwrap().unwrap().status,
            atlas_local_state::EncounterStatus::Running
        );
    }
    #[test]
    fn current_main_upgrade_preserves_parties_and_existing_conditions() {
        let directory = tempfile::tempdir().unwrap();
        register_extensions().unwrap();
        let workspace = Workspace::at(directory.path().join("main-upgrade.sqlite")).unwrap();
        let index = MIGRATIONS
            .iter()
            .position(|migration| migration.name == "condition-system")
            .unwrap();
        workspace.migrate(&MIGRATIONS[..index], |_| Ok(())).unwrap();
        let write = workspace.write(MIGRATIONS[index - 1].version).unwrap();
        // Seed the old schema directly; current services require the current migration.
        write.connection().execute_batch("INSERT INTO parties(party_key,name) VALUES('kept-party','Kept party'); INSERT INTO party_members(member_key,party_key,position,name) VALUES('kept-member','kept-party',0,'Kept PC');").unwrap();
        let party = atlas_local_state::Party {
            party_key: "kept-party".into(),
            name: "Kept party".into(),
            revision: 0,
            members: vec![atlas_local_state::PartyMember {
                member_key: "kept-member".into(),
                name: "Kept PC".into(),
                level: None,
            }],
        };
        write.connection().execute_batch("INSERT INTO encounters(id,encounter_key,slug,name,status,note,created_at,updated_at) VALUES(1,'kept','kept','Kept encounter','running','Keep encounter note','before','before');
            INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,max_hp,current_hp,created_at,updated_at) VALUES(1,1,'pc','pc',0,'Kept PC',0,20,13,'before','before');
            INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,created_at,updated_at) VALUES(2,1,'creature','creature',1,'Kept creature',1,'before','before');
            INSERT INTO encounter_participant_conditions(participant_id,name,value,note,created_at,updated_at) VALUES(1,'Frightened',2,'Keep condition note','before','before');").unwrap();
        write.commit().unwrap();
        let upgraded = initialize_workspace(workspace.path()).unwrap();
        let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let state = LocalStateRead::new(&read);
        assert_eq!(state.parties().get(&party.party_key).unwrap(), party);
        let detail = state
            .encounters()
            .get_with_participants("kept")
            .unwrap()
            .unwrap();
        assert_eq!(
            detail.encounter.note.as_deref(),
            Some("Keep encounter note")
        );
        let participant = &detail.participants[0];
        assert_eq!(participant.current_hp, Some(13));
        let condition = &participant.conditions[0];
        assert_eq!(condition.name, "Frightened");
        assert_eq!(condition.value, Some(2));
        assert_eq!(condition.note.as_deref(), Some("Keep condition note"));
        assert!(participant.track_dying);
        assert!(!detail.participants[1].track_dying);
        assert_eq!(
            participant.mortality,
            atlas_local_state::MortalityState::Alive
        );
        assert_eq!(participant.base_death_threshold, 4);
        assert_eq!(condition.scope, atlas_local_state::ConditionScope::Everyone);
        assert_eq!(
            condition.extra_action,
            atlas_local_state::ConditionExtraAction::Unspecified
        );
        assert!(!condition.immunity_override);
        assert_eq!(
            condition.parameters,
            atlas_local_state::ConditionParameters::None
        );
        let obsolete: i64 = read.connection().query_row("SELECT count(*) FROM sqlite_master WHERE name IN ('encounter_action_attempts','encounter_action_receipts','encounter_relationships')", [], |row| row.get(0)).unwrap();
        assert_eq!(obsolete, 0);
        let prototype_columns: i64 = read.connection().query_row("SELECT (SELECT count(*) FROM pragma_table_info('encounter_participant_conditions') WHERE name='gm_override_reason') + (SELECT count(*) FROM pragma_table_info('encounter_outgoing_turns') WHERE name LIKE 'handled_%')", [], |row| row.get(0)).unwrap();
        assert_eq!(prototype_columns, 0);
    }
}
