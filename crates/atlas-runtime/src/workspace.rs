//! Composition of domain-owned schemas into one ordered workspace history.
use atlas_storage::{Migration, StorageResult, Workspace};
use std::path::Path;

pub const WORKSPACE_SCHEMA_VERSION: u32 = 17;
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "reference-baseline",
        sql: atlas_index::REFERENCE_SCHEMA_SQL,
    },
    Migration {
        version: 2,
        name: "saved-state-baseline",
        sql: atlas_local_state::LOCAL_STATE_SCHEMA_SQL,
    },
    Migration {
        version: 3,
        name: "lexical-alias-rows",
        sql: atlas_index::LEXICAL_ALIAS_SCHEMA_SQL,
    },
    Migration {
        version: 4,
        name: "consumable-filters",
        sql: atlas_index::CONSUMABLE_FILTER_SCHEMA_SQL,
    },
    Migration {
        version: 5,
        name: "encounter-status",
        sql: atlas_local_state::ENCOUNTER_STATUS_SCHEMA_SQL,
    },
    Migration {
        version: 6,
        name: "hazard-participation",
        sql: atlas_local_state::HAZARD_PARTICIPATION_SCHEMA_SQL,
    },
    Migration {
        version: 7,
        name: "encounter-parties",
        sql: atlas_local_state::PARTIES_SCHEMA_SQL,
    },
    Migration {
        version: 8,
        name: "condition-context",
        sql: atlas_local_state::CONDITION_CONTEXT_SCHEMA_SQL,
    },
    Migration {
        version: 9,
        name: "condition-immunity-override",
        sql: atlas_local_state::CONDITION_IMMUNITY_OVERRIDE_SCHEMA_SQL,
    },
    Migration {
        version: 10,
        name: "condition-events",
        sql: atlas_local_state::CONDITION_EVENTS_SCHEMA_SQL,
    },
    Migration {
        version: 11,
        name: "action-attempts",
        sql: atlas_local_state::ACTION_ATTEMPTS_SCHEMA_SQL,
    },
    Migration {
        version: 12,
        name: "persistent-source-selections",
        sql: atlas_local_state::PERSISTENT_SELECTIONS_SCHEMA_SQL,
    },
    Migration {
        version: 13,
        name: "encounter-boundaries",
        sql: atlas_local_state::ENCOUNTER_BOUNDARIES_SCHEMA_SQL,
    },
    Migration {
        version: 14,
        name: "participant-facts",
        sql: atlas_local_state::PARTICIPANT_FACTS_SCHEMA_SQL,
    },
    Migration {
        version: 15,
        name: "encounter-relationships",
        sql: atlas_local_state::ENCOUNTER_RELATIONSHIPS_SCHEMA_SQL,
    },
    Migration {
        version: 16,
        name: "outgoing-turns",
        sql: atlas_local_state::OUTGOING_TURNS_SCHEMA_SQL,
    },
    Migration {
        version: 17,
        name: "current-turn-notifications",
        sql: atlas_local_state::TURN_NOTIFICATIONS_SCHEMA_SQL,
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
    use atlas_local_state::{LocalStateRead, LocalStateWrite, NewEncounter, NewSavedList};

    #[test]
    fn turn_notification_migration_preserves_receipts_and_encounter_data() {
        let directory = tempfile::tempdir().unwrap();
        register_extensions().unwrap();
        let workspace = Workspace::at(directory.path().join("atlas.sqlite")).unwrap();
        let index = MIGRATIONS
            .iter()
            .position(|migration| migration.name == "current-turn-notifications")
            .unwrap();
        workspace.migrate(&MIGRATIONS[..index], |_| Ok(())).unwrap();
        let write = workspace.write(MIGRATIONS[index - 1].version).unwrap();
        write.connection().execute("INSERT INTO encounters(id,encounter_key,slug,name,note,status,created_at,updated_at) VALUES(1,'encounter','encounter','Encounter','Keep this note','draft','now','now')", []).unwrap();
        write.connection().execute(r#"INSERT INTO encounter_outgoing_turns(encounter_id,boundary_key,request_id,intent_json,payload_json,handled_at_table,handled_request_id,handled_intent_json) VALUES(1,'turn','advance','{}','{"frozen":"context"}',1,'handled','{}')"#, []).unwrap();
        write.commit().unwrap();
        let upgraded = initialize_workspace(workspace.path()).unwrap();
        let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let row: (String, String, String) = read.connection().query_row("SELECT e.note,t.request_id,t.payload_json FROM encounters e JOIN encounter_outgoing_turns t ON t.encounter_id=e.id", [], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
        assert_eq!(
            row,
            (
                "Keep this note".into(),
                "advance".into(),
                r#"{"frozen":"context"}"#.into()
            )
        );
        let retired: i64 = read.connection().query_row("SELECT count(*) FROM pragma_table_info('encounter_outgoing_turns') WHERE name LIKE 'handled_%'", [], |row| row.get(0)).unwrap();
        assert_eq!(retired, 0);
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
        state
            .encounters()
            .create(NewEncounter {
                slug: "ambush".into(),
                name: "Ambush".into(),
                description: None,
                note: None,
            })
            .unwrap();
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
    fn immunity_override_upgrade_preserves_notes_and_relative_scope() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("condition-inputs.sqlite");
        register_extensions().unwrap();
        let workspace = Workspace::at(&path).unwrap();
        // Historical preview schema, intentionally before the override-intent migration.
        let index = MIGRATIONS
            .iter()
            .position(|migration| migration.name == "condition-immunity-override")
            .unwrap();
        workspace.migrate(&MIGRATIONS[..index], |_| Ok(())).unwrap();
        let write = workspace.write(MIGRATIONS[index - 1].version).unwrap();
        write.connection().execute_batch(r#"
            INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at) VALUES(1,'e','e','Encounter','draft','before','before');
            INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,created_at,updated_at) VALUES(1,1,'p','creature',0,'Creature',0,'before','before');
            INSERT INTO encounter_participant_conditions(participant_id,name,note,gm_override_reason,scope_json,created_at,updated_at) VALUES(1,'Unconscious','Existing note','GM exception','{"kind":"selected_opponents","participant_keys":["p"]}','before','before');
        "#).unwrap();
        write.commit().unwrap();
        let upgraded = initialize_workspace(&path).unwrap();
        let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let detail = LocalStateRead::new(&read)
            .encounters()
            .get_with_participants("e")
            .unwrap()
            .unwrap();
        let condition = &detail.participants[0].conditions[0];
        assert!(condition.immunity_override);
        assert_eq!(
            condition.note.as_deref(),
            Some("Existing note\nGM exception")
        );
        assert_eq!(
            condition.scope,
            atlas_local_state::ConditionScope::SelectedParticipants {
                participant_keys: vec!["p".into()]
            }
        );
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
            .position(|migration| migration.name == "condition-context")
            .unwrap();
        workspace.migrate(&MIGRATIONS[..index], |_| Ok(())).unwrap();
        let write = workspace.write(MIGRATIONS[index - 1].version).unwrap();
        let party = LocalStateWrite::new(&write)
            .parties()
            .create(
                "Kept party",
                &[atlas_local_state::PartyMemberInput {
                    member_key: None,
                    name: "Kept PC".into(),
                }],
            )
            .unwrap();
        write.connection().execute_batch("INSERT INTO encounters(id,encounter_key,slug,name,status,note,created_at,updated_at) VALUES(1,'kept','kept','Kept encounter','running','Keep encounter note','before','before');
            INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,max_hp,current_hp,created_at,updated_at) VALUES(1,1,'pc','creature',0,'Kept PC',0,20,13,'before','before');
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
    }
}
