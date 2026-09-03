use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    LOCAL_STATE_CONTRACT_VERSION, LOCAL_STATE_SCHEMA_VERSION, LocalStateError, LocalStateResult,
};

const METADATA_TABLE: &str = "local_state_metadata";
const SAVED_LISTS_TABLE: &str = "saved_lists";
const SAVED_LIST_ITEMS_TABLE: &str = "saved_list_items";
const SAVED_LIST_TAGS_TABLE: &str = "saved_list_tags";
const ENCOUNTERS_TABLE: &str = "encounters";
const ENCOUNTER_PARTICIPANTS_TABLE: &str = "encounter_participants";
const ENCOUNTER_PARTICIPANT_ADJUSTMENTS_TABLE: &str = "encounter_participant_adjustments";
const ENCOUNTER_PARTICIPANT_CONDITIONS_TABLE: &str = "encounter_participant_conditions";
const ENCOUNTER_PARTICIPANT_BASELINES_TABLE: &str = "encounter_participant_baselines";
const ENCOUNTER_PARTICIPANT_SPELL_STATE_TABLE: &str = "encounter_participant_spell_state";
const ENCOUNTER_PARTICIPANT_SPELL_RESOURCES_TABLE: &str = "encounter_participant_spell_resources";
const METADATA_CONTRACT_VERSION: &str = "local_state_contract_version";
const METADATA_SCHEMA_VERSION: &str = "schema_version";

pub(crate) fn initialize(connection: &Connection) -> LocalStateResult<()> {
    if table_exists(connection, METADATA_TABLE)? {
        validate_contract_metadata(connection)?;
        migrate_to_current_schema(connection)?;
        validate_metadata(connection)?;
        validate_v1_tables(connection)?;
        validate_v2_tables(connection)?;
        validate_v3_tables(connection)?;
        validate_v6_tables(connection)?;
        validate_v7_tables(connection)?;
        return Ok(());
    }
    if table_exists(connection, SAVED_LISTS_TABLE)?
        || table_exists(connection, SAVED_LIST_ITEMS_TABLE)?
    {
        return Err(LocalStateError::IncompatibleSchema(
            "saved-list tables exist without local-state metadata".to_string(),
        ));
    }
    create_v7_schema(connection)?;
    write_current_metadata(connection)?;
    Ok(())
}

fn create_v7_schema(connection: &Connection) -> LocalStateResult<()> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE local_state_metadata (
          key TEXT PRIMARY KEY,
          value TEXT NOT NULL
        );
        CREATE TABLE saved_lists (
          id INTEGER PRIMARY KEY,
          list_key TEXT NOT NULL UNIQUE,
          slug TEXT NOT NULL UNIQUE,
          name TEXT NOT NULL,
          description TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE saved_list_items (
          list_id INTEGER NOT NULL,
          record_key TEXT NOT NULL,
          position INTEGER NOT NULL,
          note TEXT,
          record_title_snapshot TEXT NOT NULL,
          record_kind_snapshot TEXT,
          added_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY (list_id, record_key),
          UNIQUE (list_id, position),
          FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
        );
        CREATE INDEX saved_list_items_position_idx
          ON saved_list_items(list_id, position);
        CREATE TABLE saved_list_tags (
          list_id INTEGER NOT NULL,
          tag TEXT NOT NULL,
          created_at TEXT NOT NULL,
          PRIMARY KEY (list_id, tag),
          FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
        );
        CREATE INDEX saved_list_tags_tag_idx
          ON saved_list_tags(tag, list_id);
        CREATE TABLE encounters (
          id INTEGER PRIMARY KEY,
          encounter_key TEXT NOT NULL UNIQUE,
          slug TEXT NOT NULL UNIQUE,
          name TEXT NOT NULL,
          description TEXT,
          note TEXT,
          status TEXT NOT NULL,
          round_number INTEGER NOT NULL DEFAULT 1,
          current_turn_participant_key TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE encounter_participants (
          id INTEGER PRIMARY KEY,
          encounter_id INTEGER NOT NULL,
          participant_key TEXT NOT NULL UNIQUE,
          record_key TEXT,
          participant_kind TEXT NOT NULL,
          participant_variant TEXT NOT NULL DEFAULT 'normal',
          position INTEGER NOT NULL,
          display_name TEXT NOT NULL,
          record_title_snapshot TEXT,
          record_kind_snapshot TEXT,
          side TEXT NOT NULL DEFAULT 'enemy',
          initiative INTEGER,
          initiative_order INTEGER NOT NULL,
          max_hp INTEGER,
          current_hp INTEGER,
          temporary_hp INTEGER NOT NULL DEFAULT 0,
          defeated INTEGER NOT NULL DEFAULT 0,
          hidden INTEGER NOT NULL DEFAULT 0,
          note TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE (encounter_id, position),
          UNIQUE (encounter_id, initiative, initiative_order),
          FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
        );
        CREATE INDEX encounter_participants_order_idx
          ON encounter_participants(encounter_id, initiative, initiative_order);
        CREATE TABLE encounter_participant_adjustments (
          participant_id INTEGER NOT NULL,
          adjustment_key TEXT NOT NULL,
          kind TEXT NOT NULL,
          value TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY (participant_id, adjustment_key),
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        CREATE TABLE encounter_participant_conditions (
          id INTEGER PRIMARY KEY,
          participant_id INTEGER NOT NULL,
          condition_key TEXT,
          name TEXT NOT NULL,
          value INTEGER,
          source_participant_key TEXT,
          duration_rounds INTEGER,
          note TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        CREATE TABLE encounter_participant_baselines (
          participant_id INTEGER PRIMARY KEY,
          participant_variant TEXT NOT NULL,
          initiative INTEGER,
          initiative_order INTEGER NOT NULL,
          max_hp INTEGER,
          current_hp INTEGER,
          temporary_hp INTEGER NOT NULL,
          defeated INTEGER NOT NULL,
          captured_at TEXT NOT NULL,
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        CREATE TABLE encounter_participant_spell_state (
          participant_id INTEGER PRIMARY KEY,
          initialized_at TEXT NOT NULL,
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        CREATE TABLE encounter_participant_spell_resources (
          participant_id INTEGER NOT NULL,
          target_kind TEXT NOT NULL,
          entry_occurrence_id TEXT NOT NULL DEFAULT '',
          spell_occurrence_id TEXT NOT NULL DEFAULT '',
          rank INTEGER NOT NULL DEFAULT -1,
          slot_id TEXT NOT NULL DEFAULT '',
          resource_id TEXT NOT NULL DEFAULT '',
          maximum INTEGER NOT NULL,
          initial_remaining INTEGER NOT NULL,
          remaining INTEGER NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY (
            participant_id, target_kind, entry_occurrence_id,
            spell_occurrence_id, rank, slot_id, resource_id
          ),
          CHECK (maximum >= 0),
          CHECK (initial_remaining >= 0 AND initial_remaining <= maximum),
          CHECK (remaining >= 0 AND remaining <= initial_remaining),
          CHECK (
            (target_kind = 'prepared_slot'
              AND entry_occurrence_id <> '' AND spell_occurrence_id <> ''
              AND rank >= 0 AND slot_id <> '' AND resource_id = '')
            OR (target_kind = 'spontaneous_pool'
              AND entry_occurrence_id <> '' AND spell_occurrence_id = ''
              AND rank >= 0 AND slot_id = '' AND resource_id = '')
            OR (target_kind = 'innate_use'
              AND spell_occurrence_id <> '' AND rank = -1
              AND slot_id = '' AND resource_id = '')
            OR (target_kind = 'focus_pool'
              AND entry_occurrence_id = '' AND spell_occurrence_id = ''
              AND rank = -1 AND slot_id = '' AND resource_id <> '')
          ),
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        ",
    )?;
    Ok(())
}

fn write_current_metadata(connection: &Connection) -> LocalStateResult<()> {
    connection.execute(
        "INSERT INTO local_state_metadata (key, value) VALUES (?1, ?2)",
        params![METADATA_CONTRACT_VERSION, LOCAL_STATE_CONTRACT_VERSION],
    )?;
    connection.execute(
        "INSERT INTO local_state_metadata (key, value) VALUES (?1, ?2)",
        params![METADATA_SCHEMA_VERSION, LOCAL_STATE_SCHEMA_VERSION],
    )?;
    Ok(())
}

fn validate_contract_metadata(connection: &Connection) -> LocalStateResult<()> {
    validate_metadata_value(
        connection,
        METADATA_CONTRACT_VERSION,
        LOCAL_STATE_CONTRACT_VERSION,
    )?;
    Ok(())
}

fn validate_metadata(connection: &Connection) -> LocalStateResult<()> {
    validate_contract_metadata(connection)?;
    validate_metadata_value(
        connection,
        METADATA_SCHEMA_VERSION,
        LOCAL_STATE_SCHEMA_VERSION,
    )?;
    Ok(())
}

fn migrate_to_current_schema(connection: &Connection) -> LocalStateResult<()> {
    let Some(schema_version) = metadata_value(connection, METADATA_SCHEMA_VERSION)? else {
        return Err(LocalStateError::IncompatibleSchema(
            "missing required local-state metadata `schema_version`".to_string(),
        ));
    };
    match schema_version.as_str() {
        LOCAL_STATE_SCHEMA_VERSION => Ok(()),
        "1" => {
            migrate_v1_to_v2(connection)?;
            migrate_v2_to_v3(connection)?;
            migrate_v3_to_v4(connection)?;
            migrate_v4_to_v5(connection)?;
            migrate_v5_to_v6(connection)?;
            migrate_v6_to_v7(connection)
        }
        "2" => {
            migrate_v2_to_v3(connection)?;
            migrate_v3_to_v4(connection)?;
            migrate_v4_to_v5(connection)?;
            migrate_v5_to_v6(connection)?;
            migrate_v6_to_v7(connection)
        }
        "3" => {
            migrate_v3_to_v4(connection)?;
            migrate_v4_to_v5(connection)?;
            migrate_v5_to_v6(connection)?;
            migrate_v6_to_v7(connection)
        }
        "4" => {
            migrate_v4_to_v5(connection)?;
            migrate_v5_to_v6(connection)?;
            migrate_v6_to_v7(connection)
        }
        "5" => {
            migrate_v5_to_v6(connection)?;
            migrate_v6_to_v7(connection)
        }
        "6" => migrate_v6_to_v7(connection),
        _ => Err(LocalStateError::UnsupportedMetadata {
            key: METADATA_SCHEMA_VERSION,
            value: schema_version,
        }),
    }
}

fn migrate_v6_to_v7(connection: &Connection) -> LocalStateResult<()> {
    connection.execute_batch(
        "
        CREATE TABLE encounter_participant_baselines (
          participant_id INTEGER PRIMARY KEY,
          participant_variant TEXT NOT NULL,
          initiative INTEGER,
          initiative_order INTEGER NOT NULL,
          max_hp INTEGER,
          current_hp INTEGER,
          temporary_hp INTEGER NOT NULL,
          defeated INTEGER NOT NULL,
          captured_at TEXT NOT NULL,
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        CREATE TABLE encounter_participant_spell_state (
          participant_id INTEGER PRIMARY KEY,
          initialized_at TEXT NOT NULL,
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        CREATE TABLE encounter_participant_spell_resources (
          participant_id INTEGER NOT NULL,
          target_kind TEXT NOT NULL,
          entry_occurrence_id TEXT NOT NULL DEFAULT '',
          spell_occurrence_id TEXT NOT NULL DEFAULT '',
          rank INTEGER NOT NULL DEFAULT -1,
          slot_id TEXT NOT NULL DEFAULT '',
          resource_id TEXT NOT NULL DEFAULT '',
          maximum INTEGER NOT NULL,
          initial_remaining INTEGER NOT NULL,
          remaining INTEGER NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY (
            participant_id, target_kind, entry_occurrence_id,
            spell_occurrence_id, rank, slot_id, resource_id
          ),
          CHECK (maximum >= 0),
          CHECK (initial_remaining >= 0 AND initial_remaining <= maximum),
          CHECK (remaining >= 0 AND remaining <= initial_remaining),
          CHECK (
            (target_kind = 'prepared_slot'
              AND entry_occurrence_id <> '' AND spell_occurrence_id <> ''
              AND rank >= 0 AND slot_id <> '' AND resource_id = '')
            OR (target_kind = 'spontaneous_pool'
              AND entry_occurrence_id <> '' AND spell_occurrence_id = ''
              AND rank >= 0 AND slot_id = '' AND resource_id = '')
            OR (target_kind = 'innate_use'
              AND spell_occurrence_id <> '' AND rank = -1
              AND slot_id = '' AND resource_id = '')
            OR (target_kind = 'focus_pool'
              AND entry_occurrence_id = '' AND spell_occurrence_id = ''
              AND rank = -1 AND slot_id = '' AND resource_id <> '')
          ),
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        UPDATE local_state_metadata
           SET value = '7'
         WHERE key = 'schema_version';
        ",
    )?;
    Ok(())
}

fn migrate_v5_to_v6(connection: &Connection) -> LocalStateResult<()> {
    connection.execute_batch(
        "
        CREATE TABLE saved_list_tags (
          list_id INTEGER NOT NULL,
          tag TEXT NOT NULL,
          created_at TEXT NOT NULL,
          PRIMARY KEY (list_id, tag),
          FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
        );
        CREATE INDEX saved_list_tags_tag_idx
          ON saved_list_tags(tag, list_id);
        UPDATE local_state_metadata
           SET value = '6'
         WHERE key = 'schema_version';
        ",
    )?;
    Ok(())
}

fn migrate_v4_to_v5(connection: &Connection) -> LocalStateResult<()> {
    connection.execute_batch(
        "
        CREATE TABLE encounter_participant_conditions_v5 (
          id INTEGER PRIMARY KEY,
          participant_id INTEGER NOT NULL,
          condition_key TEXT,
          name TEXT NOT NULL,
          value INTEGER,
          source_participant_key TEXT,
          duration_rounds INTEGER,
          note TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        INSERT INTO encounter_participant_conditions_v5 (
          id, participant_id, condition_key, name, value, source_participant_key,
          duration_rounds, note, created_at, updated_at
        )
        SELECT id, participant_id, condition_key, name, value, source_participant_key,
               duration_rounds, note, created_at, updated_at
          FROM encounter_participant_conditions;
        DROP TABLE encounter_participant_conditions;
        ALTER TABLE encounter_participant_conditions_v5
          RENAME TO encounter_participant_conditions;
        UPDATE local_state_metadata
           SET value = '5'
         WHERE key = 'schema_version';
        ",
    )?;
    Ok(())
}

fn migrate_v3_to_v4(connection: &Connection) -> LocalStateResult<()> {
    connection.execute_batch(
        "
        ALTER TABLE encounter_participants
          ADD COLUMN participant_variant TEXT NOT NULL DEFAULT 'normal';
        UPDATE local_state_metadata
           SET value = '4'
         WHERE key = 'schema_version';
        ",
    )?;
    Ok(())
}

fn migrate_v2_to_v3(connection: &Connection) -> LocalStateResult<()> {
    connection.execute_batch(
        "
        CREATE TABLE encounters (
          id INTEGER PRIMARY KEY,
          encounter_key TEXT NOT NULL UNIQUE,
          slug TEXT NOT NULL UNIQUE,
          name TEXT NOT NULL,
          description TEXT,
          note TEXT,
          status TEXT NOT NULL,
          round_number INTEGER NOT NULL DEFAULT 1,
          current_turn_participant_key TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE encounter_participants (
          id INTEGER PRIMARY KEY,
          encounter_id INTEGER NOT NULL,
          participant_key TEXT NOT NULL UNIQUE,
          record_key TEXT,
          participant_kind TEXT NOT NULL,
          position INTEGER NOT NULL,
          display_name TEXT NOT NULL,
          record_title_snapshot TEXT,
          record_kind_snapshot TEXT,
          side TEXT NOT NULL DEFAULT 'enemy',
          initiative INTEGER,
          initiative_order INTEGER NOT NULL,
          max_hp INTEGER,
          current_hp INTEGER,
          temporary_hp INTEGER NOT NULL DEFAULT 0,
          defeated INTEGER NOT NULL DEFAULT 0,
          hidden INTEGER NOT NULL DEFAULT 0,
          note TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE (encounter_id, position),
          UNIQUE (encounter_id, initiative, initiative_order),
          FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
        );
        CREATE INDEX encounter_participants_order_idx
          ON encounter_participants(encounter_id, initiative, initiative_order);
        CREATE TABLE encounter_participant_adjustments (
          participant_id INTEGER NOT NULL,
          adjustment_key TEXT NOT NULL,
          kind TEXT NOT NULL,
          value TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY (participant_id, adjustment_key),
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        CREATE TABLE encounter_participant_conditions (
          id INTEGER PRIMARY KEY,
          participant_id INTEGER NOT NULL,
          condition_key TEXT,
          name TEXT NOT NULL,
          value INTEGER,
          source_participant_key TEXT,
          duration_rounds INTEGER,
          note TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
        );
        UPDATE local_state_metadata
           SET value = '3'
         WHERE key = 'schema_version';
        ",
    )?;
    Ok(())
}

fn migrate_v1_to_v2(connection: &Connection) -> LocalStateResult<()> {
    connection.execute_batch(
        "
        ALTER TABLE saved_lists ADD COLUMN list_key TEXT;
        UPDATE saved_lists
           SET list_key = 'list_' || lower(hex(randomblob(8)))
         WHERE list_key IS NULL;
        CREATE UNIQUE INDEX saved_lists_list_key_idx ON saved_lists(list_key);
        UPDATE local_state_metadata
           SET value = '2'
         WHERE key = 'schema_version';
        ",
    )?;
    Ok(())
}

fn validate_metadata_value(
    connection: &Connection,
    key: &'static str,
    expected: &'static str,
) -> LocalStateResult<()> {
    let Some(actual) = metadata_value(connection, key)? else {
        return Err(LocalStateError::IncompatibleSchema(format!(
            "missing required local-state metadata `{key}`"
        )));
    };
    if actual == expected {
        return Ok(());
    }
    Err(LocalStateError::UnsupportedMetadata { key, value: actual })
}

fn validate_v1_tables(connection: &Connection) -> LocalStateResult<()> {
    for table in [SAVED_LISTS_TABLE, SAVED_LIST_ITEMS_TABLE] {
        if !table_exists(connection, table)? {
            return Err(LocalStateError::IncompatibleSchema(format!(
                "missing required local-state table `{table}`"
            )));
        }
    }
    Ok(())
}

fn validate_v2_tables(connection: &Connection) -> LocalStateResult<()> {
    if !column_exists(connection, SAVED_LISTS_TABLE, "list_key")? {
        return Err(LocalStateError::IncompatibleSchema(
            "missing required saved_lists.list_key column".to_string(),
        ));
    }
    Ok(())
}

fn validate_v3_tables(connection: &Connection) -> LocalStateResult<()> {
    for table in [
        ENCOUNTERS_TABLE,
        ENCOUNTER_PARTICIPANTS_TABLE,
        ENCOUNTER_PARTICIPANT_ADJUSTMENTS_TABLE,
        ENCOUNTER_PARTICIPANT_CONDITIONS_TABLE,
    ] {
        if !table_exists(connection, table)? {
            return Err(LocalStateError::IncompatibleSchema(format!(
                "missing required local-state table `{table}`"
            )));
        }
    }
    if !column_exists(
        connection,
        ENCOUNTER_PARTICIPANTS_TABLE,
        "participant_variant",
    )? {
        return Err(LocalStateError::IncompatibleSchema(
            "missing required encounter_participants.participant_variant column".to_string(),
        ));
    }
    Ok(())
}

fn validate_v6_tables(connection: &Connection) -> LocalStateResult<()> {
    if !table_exists(connection, SAVED_LIST_TAGS_TABLE)? {
        return Err(LocalStateError::IncompatibleSchema(format!(
            "missing required local-state table `{SAVED_LIST_TAGS_TABLE}`"
        )));
    }
    Ok(())
}

fn validate_v7_tables(connection: &Connection) -> LocalStateResult<()> {
    for table in [
        ENCOUNTER_PARTICIPANT_BASELINES_TABLE,
        ENCOUNTER_PARTICIPANT_SPELL_STATE_TABLE,
        ENCOUNTER_PARTICIPANT_SPELL_RESOURCES_TABLE,
    ] {
        if !table_exists(connection, table)? {
            return Err(LocalStateError::IncompatibleSchema(format!(
                "missing required local-state table `{table}`"
            )));
        }
    }
    Ok(())
}

fn table_exists(connection: &Connection, table: &str) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![table],
            |_row| Ok(()),
        )
        .optional()
        .map(|value| value.is_some())
}

fn column_exists(connection: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn metadata_value(connection: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    connection
        .query_row(
            "SELECT value FROM local_state_metadata WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use rusqlite::Connection;
    use time::OffsetDateTime;

    use super::*;
    use crate::{
        AddEncounterParticipant, LocalStateStore, NewEncounter, ParticipantKind, ParticipantSide,
    };

    #[test]
    fn fresh_database_initializes_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("metadata");
        let store = LocalStateStore::open(&path)?;
        drop(store);
        let connection = Connection::open(&path)?;
        assert_eq!(
            metadata_value(&connection, METADATA_CONTRACT_VERSION)?,
            Some(LOCAL_STATE_CONTRACT_VERSION.to_string())
        );
        assert_eq!(
            metadata_value(&connection, METADATA_SCHEMA_VERSION)?,
            Some(LOCAL_STATE_SCHEMA_VERSION.to_string())
        );
        Ok(())
    }

    #[test]
    fn unsupported_metadata_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("unsupported-metadata");
        {
            let connection = Connection::open(&path)?;
            connection.execute_batch(
                "
                CREATE TABLE local_state_metadata (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL
                );
                INSERT INTO local_state_metadata (key, value)
                  VALUES ('local_state_contract_version', 'pf2e-atlas-local-state/v2');
                INSERT INTO local_state_metadata (key, value)
                  VALUES ('schema_version', '2');
                ",
            )?;
        }

        let result = LocalStateStore::open(path);
        assert!(matches!(
            result,
            Err(LocalStateError::UnsupportedMetadata { .. })
        ));
        Ok(())
    }

    #[test]
    fn v1_database_migrates_to_list_keys() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("v1-migration");
        {
            let connection = Connection::open(&path)?;
            connection.execute_batch(
                "
                PRAGMA foreign_keys = ON;
                CREATE TABLE local_state_metadata (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL
                );
                INSERT INTO local_state_metadata (key, value)
                  VALUES ('local_state_contract_version', 'pf2e-atlas-local-state/v1');
                INSERT INTO local_state_metadata (key, value)
                  VALUES ('schema_version', '1');
                CREATE TABLE saved_lists (
                  id INTEGER PRIMARY KEY,
                  slug TEXT NOT NULL UNIQUE,
                  name TEXT NOT NULL,
                  description TEXT,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL
                );
                CREATE TABLE saved_list_items (
                  list_id INTEGER NOT NULL,
                  record_key TEXT NOT NULL,
                  position INTEGER NOT NULL,
                  note TEXT,
                  record_title_snapshot TEXT NOT NULL,
                  record_kind_snapshot TEXT,
                  added_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL,
                  PRIMARY KEY (list_id, record_key),
                  UNIQUE (list_id, position),
                  FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
                );
                CREATE INDEX saved_list_items_position_idx
                  ON saved_list_items(list_id, position);
                INSERT INTO saved_lists (
                  slug, name, description, created_at, updated_at
                )
                VALUES (
                  'legacy-list',
                  'Legacy List',
                  NULL,
                  '2026-01-01T00:00:00Z',
                  '2026-01-01T00:00:00Z'
                );
                ",
            )?;
        }

        let store = LocalStateStore::open(&path)?;
        let list = store
            .saved_lists()
            .get("legacy-list")?
            .expect("legacy list should survive migration");
        assert!(list.list_key.starts_with("list_"));
        drop(store);

        let connection = Connection::open(&path)?;
        assert_eq!(
            metadata_value(&connection, METADATA_SCHEMA_VERSION)?,
            Some(LOCAL_STATE_SCHEMA_VERSION.to_string())
        );
        assert!(column_exists(&connection, SAVED_LISTS_TABLE, "list_key")?);
        Ok(())
    }

    #[test]
    fn v6_database_migrates_to_typed_spell_resource_state() -> Result<(), Box<dyn std::error::Error>>
    {
        let path = temp_path("v6-spell-resource-migration");
        let participant_key;
        {
            let store = LocalStateStore::open(&path)?;
            store.encounters().create(NewEncounter {
                slug: "legacy".to_string(),
                name: "Legacy".to_string(),
                description: None,
                note: None,
            })?;
            participant_key = store
                .encounters()
                .add_participant(
                    "legacy",
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
            drop(store);
            let connection = Connection::open(&path)?;
            connection.execute_batch(
                "DROP TABLE encounter_participant_spell_resources;
                 DROP TABLE encounter_participant_spell_state;
                 DROP TABLE encounter_participant_baselines;
                 UPDATE local_state_metadata SET value = '6' WHERE key = 'schema_version';",
            )?;
        }

        let store = LocalStateStore::open(&path)?;
        assert!(
            !store
                .encounters()
                .participant_reset_available(&participant_key)?
        );
        assert!(matches!(
            store.encounters().reset_participant(&participant_key),
            Err(LocalStateError::ParticipantResetBaselineUnavailable(key)) if key == participant_key
        ));
        drop(store);
        let connection = Connection::open(&path)?;
        assert_eq!(
            metadata_value(&connection, METADATA_SCHEMA_VERSION)?,
            Some(LOCAL_STATE_SCHEMA_VERSION.to_string())
        );
        assert!(table_exists(
            &connection,
            ENCOUNTER_PARTICIPANT_SPELL_STATE_TABLE
        )?);
        assert!(table_exists(
            &connection,
            ENCOUNTER_PARTICIPANT_SPELL_RESOURCES_TABLE
        )?);
        assert!(table_exists(
            &connection,
            ENCOUNTER_PARTICIPANT_BASELINES_TABLE
        )?);
        assert_eq!(
            connection.query_row(
                "SELECT COUNT(*) FROM encounter_participant_baselines",
                [],
                |row| row.get::<_, i64>(0),
            )?,
            0
        );
        Ok(())
    }

    #[test]
    fn existing_tables_without_metadata_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("no-metadata");
        {
            let connection = Connection::open(&path)?;
            connection.execute_batch("CREATE TABLE saved_lists (id INTEGER PRIMARY KEY);")?;
        }

        let result = LocalStateStore::open(path);
        assert!(matches!(
            result,
            Err(LocalStateError::IncompatibleSchema(_))
        ));
        Ok(())
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
}
