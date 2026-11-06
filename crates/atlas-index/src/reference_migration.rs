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
    if let Some(context) = crate::read_stored_reference_context(c)?
        && context.snapshot_version == atlas_foundry_model::SNAPSHOT_VERSION
        && context.source_contract == atlas_foundry_model::SOURCE_CONTRACT_ID
        && !matches!(
            crate::classify_reference_upgrade(c)?,
            crate::ReferenceUpgrade::SourcePreparation
                | crate::ReferenceUpgrade::SourceReplacement
                | crate::ReferenceUpgrade::QueryProjections
        )
    {
        backfill_reference_facts(c, &context)?;
        crate::component_digests::refresh_present(c, &context)?;
    }
    Ok(())
}

fn backfill_reference_facts(
    c: &Connection,
    context: &crate::SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    crate::visit_persisted_inputs(c, context, &mut |source| {
        let (id,label,seed):(i64,String,i64)=c.query_row("SELECT r.record_id,p.label,a.seed FROM records r JOIN packs p USING(pack_id) JOIN projection_allocations a USING(record_id) WHERE r.key=?",[source.record.key().to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let mut allocation = crate::projections::ProjectionIds::seeded(seed);
        for row in crate::projections::root_rows(&source, &label, id, &mut allocation)? {
            let (key, fields): (&str, &[&str]) = match row.table {
                "actor_senses" => (
                    "id",
                    &[
                        "acuity_state",
                        "acuity",
                        "range_state",
                        "range",
                        "source_state",
                        "source",
                    ],
                ),
                "armor_projection" => ("physical_id", &["armor_group_state", "armor_group"]),
                _ => continue,
            };
            let key_index = row.columns.iter().position(|n| n == key).ok_or_else(|| {
                IndexError::Invalid("reference backfill row has no identity".into())
            })?;
            let mut values = Vec::new();
            for field in fields {
                let index = row.columns.iter().position(|n| n == field).ok_or_else(|| {
                    IndexError::Invalid("reference backfill row lacks field".into())
                })?;
                values.push(row.values[index].clone());
            }
            values.push(row.values[key_index].clone());
            let setters = fields
                .iter()
                .map(|name| format!("{name}=?"))
                .collect::<Vec<_>>()
                .join(",");
            if c.execute(
                &format!("UPDATE {} SET {setters} WHERE {key}=?", row.table),
                rusqlite::params_from_iter(values),
            )? != 1
            {
                return Err(IndexError::Invalid(
                    "reference backfill row is absent".into(),
                ));
            }
        }
        Ok(())
    })
}
