//! Stable reference identity, independent of the current source projections.
use crate::IndexError;
use atlas_domain::RecordKey;
use rusqlite::{Connection, OptionalExtension};

pub(crate) fn retain_source(
    connection: &Connection,
    key: &RecordKey,
    name: Option<&str>,
) -> Result<i64, IndexError> {
    if let Some((id, previous_name, available)) = connection
        .query_row(
            "SELECT id,last_known_name,availability FROM record_identity WHERE key=?",
            [key.to_string()],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?
    {
        if previous_name.as_deref() != name || available != "available" {
            connection.execute(
                "UPDATE record_identity SET last_known_name=?,availability='available' WHERE id=?",
                rusqlite::params![name, id],
            )?;
        }
        return Ok(id);
    }
    Ok(connection.query_row(
        "INSERT INTO record_identity(key,last_known_name,availability) VALUES(?,?,'available')
         ON CONFLICT(key) DO UPDATE SET last_known_name=excluded.last_known_name,availability='available'
         RETURNING id",
        rusqlite::params![key.to_string(), name],
        |row| row.get(0),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_reappearance_reuses_imported_and_removed_identities() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(include_str!(
                "../migrations/00000000000001_create_artifact/up.sql"
            ))
            .unwrap();
        let key = RecordKey::parse("spells:missing").unwrap();
        connection.execute("INSERT INTO record_identity(key,last_known_name,availability) VALUES('spells:missing','Saved title','unavailable')", []).unwrap();
        let initial = connection.last_insert_rowid();
        assert_eq!(
            retain_source(&connection, &key, Some("Current title")).unwrap(),
            initial
        );
        connection
            .execute(
                "UPDATE record_identity SET availability='unavailable' WHERE id=?",
                [initial],
            )
            .unwrap();
        let other = RecordKey::parse("spells:another").unwrap();
        assert_ne!(
            retain_source(&connection, &other, Some("Another")).unwrap(),
            initial
        );
        assert_eq!(
            retain_source(&connection, &key, Some("Returned title")).unwrap(),
            initial
        );
        let actual: (String, String) = connection
            .query_row(
                "SELECT last_known_name,availability FROM record_identity WHERE id=?",
                [initial],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(actual, ("Returned title".into(), "available".into()));
    }
}
