//! Saved-domain bridge from the last pre-release schema.
use atlas_storage::{StorageError, StorageResult};
use rusqlite::Connection;
use sha2::{Digest, Sha256};

const PREDECESSOR_SHAPE: &str = "3cead4dc4850563c2603ce5ca92b0fedaf9233234a6c9e6c9709b76e596cec6d";

/// Copy the three ownership roots under storage's reconstruction transaction.
pub fn upgrade_release_owners(connection: &Connection) -> StorageResult<()> {
    let mut hash = Sha256::new();
    hash.update(b"atlas-saved-owner-schema/v1\0");
    let mut statement = connection.prepare(include_str!("release/owner_preflight.sql"))?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        for column in 0..4 {
            let field: String = row.get(column)?;
            let field = field.replace("\r\n", "\n");
            hash.update((field.len() as u64).to_be_bytes());
            hash.update(field.as_bytes());
        }
    }
    if format!("{:x}", hash.finalize()) != PREDECESSOR_SHAPE {
        return Err(StorageError::Migration(
            "schema 41 saved-owner tables or their dependencies differ from the release baseline"
                .into(),
        ));
    }
    drop(rows);
    drop(statement);
    if !connection
        .prepare("SELECT 1 FROM principals WHERE principal_key='local'")?
        .exists([])?
    {
        return Err(StorageError::Migration(
            "schema 41 workspace is missing its local principal".into(),
        ));
    }

    for (table, ddl, columns) in [
        (
            "saved_lists",
            include_str!("release/saved_lists.sql"),
            "id, owner_id, revision, list_key, slug, name, description, created_at, updated_at",
        ),
        (
            "parties",
            include_str!("release/parties.sql"),
            "party_key, owner_id, name, revision",
        ),
        (
            "encounters",
            include_str!("release/encounters.sql"),
            "id, owner_id, encounter_key, slug, name, description, note, status, round_number,
             current_turn_participant_key, created_at, updated_at, revision, difficulty_party_level,
             difficulty_party_size, round_duration_seconds",
        ),
    ] {
        let replacement = format!("{table}_explicit_owner");
        let count: usize =
            connection.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get(0)
            })?;
        connection.execute_batch(&ddl.replacen(
            &format!("CREATE TABLE {table}"),
            &format!("CREATE TABLE {replacement}"),
            1,
        ))?;
        let copied = connection.execute(
            &format!("INSERT INTO {replacement} ({columns}) SELECT {columns} FROM {table}"),
            [],
        )?;
        if count != copied {
            return Err(StorageError::Migration(format!(
                "{table} row count changed during owner reconstruction"
            )));
        }
        connection.execute_batch(&format!(
            "DROP TABLE {table}; ALTER TABLE {replacement} RENAME TO {table};"
        ))?;
    }
    Ok(())
}
