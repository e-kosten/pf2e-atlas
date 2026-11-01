//! Schema-25 reconstruction; retired source contracts are never decoded.
use super::{SelectedSection, project, upsert};
use crate::{IndexError, ReferenceUpgrade};
use atlas_domain::SourcePassageAddress;
use rusqlite::{Connection, params};
use std::collections::BTreeMap;

pub fn backfill_selected_sections(c: &Connection) -> atlas_storage::StorageResult<()> {
    migrate(c).map_err(|e| atlas_storage::StorageError::Migration(e.to_string()))
}
fn invalid(message: &str) -> IndexError {
    IndexError::Invalid(message.into())
}
fn migrate(c: &Connection) -> Result<(), IndexError> {
    let context = crate::read_stored_reference_context(c)?;
    if context.is_none()
        && c.query_row("SELECT EXISTS(SELECT 1 FROM records)", [], |row| {
            row.get::<_, bool>(0)
        })?
    {
        return Err(invalid(
            "unpublished workspace contains partial reference data",
        ));
    }
    let compatible = context
        .as_ref()
        .map(|_| crate::classify_reference_upgrade(c))
        .transpose()?
        .is_none_or(|upgrade| {
            !matches!(
                upgrade,
                ReferenceUpgrade::SourcePreparation | ReferenceUpgrade::SourceReplacement
            )
        });
    c.execute_batch(include_str!("occurrences.sql"))?;
    if let Some(context) = context.as_ref().filter(|_| compatible) {
        let mut after = 0i64;
        loop {
            let roots=c.prepare("SELECT r.record_id,r.key,p.label FROM records r JOIN packs p USING(pack_id) WHERE r.record_id>? ORDER BY r.record_id LIMIT 128")?.query_map([after],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?)))?.collect::<Result<Vec<_>,_>>()?;
            if roots.is_empty() {
                break;
            }
            for (id, key, label) in roots {
                after = id;
                let key = crate::read_units::parse_key(key)?;
                let source = crate::persisted_inputs::read_persisted_input(c, context, &key)?
                    .ok_or_else(|| invalid("canonical section backfill lost its checked source"))?;
                let selection = atlas_record::source_record::select_record_search(
                    &source.record,
                    &source.content,
                    context.audience,
                    &label,
                    &context.used_trait_labels,
                )
                .map_err(|e| IndexError::Invalid(e.to_string()))?;
                let sections = project(
                    &source.record,
                    &selection,
                    &source.content_hash,
                    &source.preparation_context_hash,
                )?;
                let mut texts = BTreeMap::new();
                for field in &selection.fields {
                    for section in &field.sections {
                        texts.insert(
                            (
                                serde_json::to_string(&field.locator.owners)?,
                                field.locator.field.clone(),
                                section.section_ordinal,
                            ),
                            section.text.as_str(),
                        );
                    }
                }
                for section in sections {
                    upsert(c, id, &section)?;
                }
                copy_lexical(c, id, true)?;
                copy_locations(c, id, true, &texts)?;
            }
        }
        let old: usize = c.query_row("SELECT count(*) FROM core_parts", [], |r| r.get(0))?;
        let copied=c.execute("INSERT INTO canonical_core_parts SELECT ss.section_id,cp.record_id FROM core_parts cp JOIN selected_sections ss ON ss.selector=cp.selector AND ss.record_id=cp.record_id",[])?;
        if old != copied {
            return Err(invalid(
                "Core section reference has no checked canonical section",
            ));
        }
    } else {
        // These accelerators cannot be certified with the current model. Exact
        // native values remain reachable through input/unit IDs for later setup.
        c.execute_batch("DELETE FROM core_parts; DELETE FROM core_lexical_units; DELETE FROM core_semantic_locations; DELETE FROM core_roots; DELETE FROM lexical_units WHERE unit_kind IN ('heading','definition_label');")?;
        let roots = c
            .prepare("SELECT record_id FROM records ORDER BY record_id")?
            .query_map([], |r| r.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for id in roots {
            copy_lexical(c, id, false)?;
            copy_locations(c, id, false, &BTreeMap::new())?;
        }
    }
    c.execute_batch("DROP TABLE core_parts; DROP TABLE lexical_units; DROP TABLE semantic_locations; DROP TABLE similarity_parts;
ALTER TABLE canonical_core_parts RENAME TO core_parts;
ALTER TABLE canonical_lexical_units RENAME TO lexical_units;
ALTER TABLE canonical_semantic_locations RENAME TO semantic_locations;
CREATE INDEX core_parts_root ON core_parts(record_id,section_id);
CREATE INDEX lexical_units_root ON lexical_units(record_id,unit_id);
CREATE INDEX semantic_locations_root ON semantic_locations(record_id,location_id);
CREATE UNIQUE INDEX semantic_identity_location ON semantic_locations(record_id) WHERE section_id IS NULL AND chunk_ordinal=0;
CREATE INDEX semantic_locations_section ON semantic_locations(section_id,location_id);
CREATE TRIGGER lexical_insert AFTER INSERT ON lexical_units BEGIN INSERT INTO lexical_fts(rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES(new.unit_id,new.identity_terms,new.alias_terms,new.structured_terms,new.definition_terms); END;
CREATE TRIGGER lexical_delete AFTER DELETE ON lexical_units BEGIN INSERT INTO lexical_fts(lexical_fts,rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES('delete',old.unit_id,old.identity_terms,old.alias_terms,old.structured_terms,old.definition_terms); END;
CREATE TRIGGER lexical_update AFTER UPDATE ON lexical_units BEGIN INSERT INTO lexical_fts(lexical_fts,rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES('delete',old.unit_id,old.identity_terms,old.alias_terms,old.structured_terms,old.definition_terms); INSERT INTO lexical_fts(rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES(new.unit_id,new.identity_terms,new.alias_terms,new.structured_terms,new.definition_terms); END;")?;
    if c.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(invalid(
            "canonical section reconstruction has invalid foreign keys",
        ));
    }
    Ok(())
}
fn copy_lexical(c: &Connection, id: i64, compatible: bool) -> Result<(), IndexError> {
    let mut statement=c.prepare("SELECT input_hash,unit_id,owners_json,field_path,section_json,unit_kind,identity_terms,alias_terms,structured_terms,definition_terms FROM lexical_units WHERE record_id=? ORDER BY unit_id")?;
    let mut rows = statement.query([id])?;
    while let Some(row) = rows.next()? {
        let owners: String = row.get(2)?;
        let field: Option<String> = row.get(3)?;
        let address: Option<String> = row.get(4)?;
        let kind: String = row.get(5)?;
        let section = if matches!(kind.as_str(), "heading" | "definition_label") {
            if !compatible {
                return Err(invalid("retired lexical section was not removed"));
            }
            let address: SourcePassageAddress = serde_json::from_str(
                address
                    .as_deref()
                    .ok_or_else(|| invalid("lexical definition has no section address"))?,
            )?;
            let section = checked_section(c, id, &owners, field.as_deref(), &address)?;
            if section.address != address {
                return Err(invalid(
                    "lexical section does not cover its canonical whole extent",
                ));
            }
            Some(section.id)
        } else {
            if !matches!(kind.as_str(), "root_name" | "root_alias" | "owned_name")
                || field.is_some()
                || address.is_some()
            {
                return Err(invalid("invalid non-section lexical attribution"));
            }
            None
        };
        c.execute(
            "INSERT INTO canonical_lexical_units VALUES(?,?,?,?,?,?,?,?,?,?)",
            params![
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                id,
                section,
                if section.is_none() {
                    Some(owners)
                } else {
                    None
                },
                kind,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?
            ],
        )?;
    }
    Ok(())
}
fn checked_section(
    c: &Connection,
    id: i64,
    owners: &str,
    field: Option<&str>,
    address: &SourcePassageAddress,
) -> Result<SelectedSection, IndexError> {
    let ordinal = match address {
        SourcePassageAddress::HtmlSection {
            section_ordinal, ..
        }
        | SourcePassageAddress::PlainSection {
            section_ordinal, ..
        } => *section_ordinal,
        SourcePassageAddress::Identity {} => {
            return Err(invalid("identity cannot bind a canonical prose section"));
        }
    };
    let field = field.ok_or_else(|| invalid("section field is missing"))?;
    super::find(c, id, owners, field, ordinal)
}
fn copy_locations(
    c: &Connection,
    id: i64,
    compatible: bool,
    texts: &BTreeMap<(String, String, usize), &str>,
) -> Result<(), IndexError> {
    let mut statement=c.prepare("SELECT location_id,location_hash,owners_json,field_path,section_json,chunk_ordinal FROM semantic_locations l WHERE record_id=? AND EXISTS(SELECT 1 FROM semantic_units u WHERE u.location_id=l.location_id) ORDER BY location_id")?;
    let mut rows = statement.query([id])?;
    while let Some(row) = rows.next()? {
        let location: i64 = row.get(0)?;
        let old_hash: Vec<u8> = row.get(1)?;
        let owners: String = row.get(2)?;
        let field: Option<String> = row.get(3)?;
        let json: String = row.get(4)?;
        let ordinal: usize = row.get(5)?;
        if old_hash
            != crate::embedding_values::hash_bytes(&serde_json::to_vec(&(
                id, &owners, &field, &json, ordinal,
            ))?)
        {
            return Err(invalid("historical semantic location checksum mismatch"));
        }
        let identity:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM semantic_units WHERE location_id=? AND unit_kind='identity')",[location],|r|r.get(0))?;
        let passage:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM semantic_units WHERE location_id=? AND unit_kind='passage')",[location],|r|r.get(0))?;
        let (section, ordinal, start, end) = if identity {
            if passage
                || owners != "[]"
                || field.is_some()
                || ordinal != 0
                || serde_json::from_str::<SourcePassageAddress>(&json)?
                    != (SourcePassageAddress::Identity {})
            {
                return Err(invalid("invalid historical semantic identity attribution"));
            }
            (None, Some(0usize), None, None)
        } else if compatible {
            let address: SourcePassageAddress = serde_json::from_str(&json)?;
            let section = checked_section(c, id, &owners, field.as_deref(), &address)?;
            let mut copied = address.clone();
            let range = *super::range_mut(&mut copied)?;
            if section.chunk_address(range)? != address {
                return Err(invalid(
                    "historical semantic section identity disagrees with checked source",
                ));
            }
            let text = texts
                .get(&(
                    owners.clone(),
                    field
                        .clone()
                        .ok_or_else(|| invalid("passage field unavailable"))?,
                    section.part.section_ordinal,
                ))
                .ok_or_else(|| invalid("historical semantic text unavailable"))?;
            range
                .extract(text)
                .map_err(|e| IndexError::Invalid(e.to_string()))?;
            (
                Some(section.id),
                Some(ordinal),
                Some(range.start),
                Some(range.end),
            )
        } else {
            (None, None, None, None)
        };
        c.execute(
            "INSERT INTO canonical_semantic_locations VALUES(?,?,?,?,?,?)",
            params![location, id, section, ordinal, start, end],
        )?;
    }
    Ok(())
}
