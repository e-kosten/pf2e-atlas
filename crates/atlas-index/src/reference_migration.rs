//! Offline migration of retained trait content into its canonical reference owner.
use crate::{IndexError, ReferenceEntry};
use rusqlite::{Connection, params};

pub const REFERENCE_ENTRIES_SCHEMA_SQL: &str = include_str!("reference_schema.sql");

pub fn backfill_reference_entries(c: &Connection) -> atlas_storage::StorageResult<()> {
    backfill(c).map_err(|e| atlas_storage::StorageError::Migration(e.to_string()))
}

fn backfill(c: &Connection) -> Result<(), IndexError> {
    let keys = c
        .prepare("SELECT key FROM reference_entries ORDER BY key")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for key in keys {
        let (json, hash): (String, String) = c.query_row(
            "SELECT definition_json,definition_hash FROM reference_entries WHERE key=?",
            [&key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if crate::codec::sha256(json.as_bytes()) != hash {
            return Err(IndexError::Invalid(
                "retained trait definition checksum mismatch".into(),
            ));
        }
        let definition: crate::TraitDefinition = serde_json::from_str(&json)?;
        if definition.key.to_string() != key {
            return Err(IndexError::Invalid("retained trait key mismatch".into()));
        }
        let mut entry = ReferenceEntry::from_trait(&definition)?;
        if matches!(&entry.backing, crate::ReferenceBacking::None) {
            entry.availability = crate::ReferenceAvailability::UnavailableUnknown;
        }
        let json = serde_json::to_string(&entry)?;
        c.execute(
            "UPDATE reference_entries SET definition_json=?,definition_hash=? WHERE key=?",
            params![json, crate::codec::sha256(json.as_bytes()), key],
        )?;
    }
    let rows = c
        .prepare(
            "SELECT evidence FROM legacy_trait_references ORDER BY trait_key,record_key,evidence",
        )?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for json in rows {
        let reference: crate::TraitReferenceInput = serde_json::from_str(&json)?;
        crate::references::insert_reference(c, &reference.into())?;
    }
    c.execute_batch("DROP TABLE legacy_trait_references;")?;
    Ok(())
}
