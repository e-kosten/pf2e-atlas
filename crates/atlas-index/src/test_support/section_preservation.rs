//! Test-only logical preservation oracle for section storage migrations.
use std::collections::BTreeMap;

type TableRows = Vec<Vec<rusqlite::types::Value>>;

fn json(value: &impl serde::Serialize) -> rusqlite::Result<String> {
    serde_json::to_string(value)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
}
fn json_bytes(value: &impl serde::Serialize) -> rusqlite::Result<Vec<u8>> {
    serde_json::to_vec(value)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
}

fn normalized_section_table(
    c: &rusqlite::Connection,
    table: &str,
) -> Result<Option<(String, TableRows)>, crate::IndexError> {
    use rusqlite::types::Value as V;
    let text = |value: Option<String>| value.map(V::Text).unwrap_or(V::Null);
    if table == "projection_allocations" {
        let mut statement = c.prepare(
            "SELECT record_id,seed,end_id FROM projection_allocations ORDER BY record_id",
        )?;
        let rows = statement
            .query_map([], |row| Ok(vec![row.get(0)?, row.get(1)?, row.get(2)?]))?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Some((table.into(), rows)));
    }
    if table == "selected_sections" {
        let mut statement = c.prepare(&format!(
            "SELECT {} FROM selected_sections ss ORDER BY ss.selector",
            crate::selected_sections::COLUMNS
        ))?;
        let rows = statement
            .query_map([], |row| {
                let section = crate::selected_sections::SelectedSection::read(row, 0)?;
                Ok(vec![
                    V::Text(section.part.selector.0.clone()),
                    V::Integer(section.record_id),
                    V::Text(json(&section.part.owners)?),
                    V::Text(section.part.field.clone()),
                    V::Integer(section.part.section_ordinal as i64),
                    V::Text(row.get("body_hash")?),
                    V::Text(
                        serde_json::to_value(section.part.role)
                            .map_err(|error| {
                                rusqlite::Error::ToSqlConversionFailure(Box::new(error))
                            })?
                            .as_str()
                            .ok_or_else(|| {
                                rusqlite::Error::ToSqlConversionFailure(Box::new(
                                    std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "section role did not serialize as text",
                                    ),
                                ))
                            })?
                            .into(),
                    ),
                    V::Text(json(&section.part)?),
                    V::Text(row.get("recipe")?),
                ])
            })?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Some(("similarity_parts".into(), rows)));
    }
    let canonical = c
        .prepare(&format!("PRAGMA table_info({table})"))?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .iter()
        .any(|column| column == "section_id");
    if table == "lexical_units" && canonical {
        let mut statement=c.prepare(&format!("SELECT lu.input_hash AS old_input_hash,lu.record_id AS old_record_id,{},lu.identity_terms,lu.alias_terms,lu.structured_terms,lu.definition_terms FROM lexical_units lu JOIN records r USING(record_id) LEFT JOIN selected_sections ss ON ss.section_id=lu.section_id ORDER BY lu.input_hash",crate::read_units::LEXICAL_LOCATION_COLUMNS))?;
        let rows = statement
            .query_map([], |row| {
                let location = crate::read_units::location_at(row, 2)?;
                Ok(vec![
                    V::Text(row.get("old_input_hash")?),
                    V::Integer(location.unit_id),
                    V::Integer(row.get("old_record_id")?),
                    V::Text(json(&location.owners)?),
                    text(location.field),
                    text(location.address.map(|address| json(&address)).transpose()?),
                    V::Text(row.get(15)?),
                    V::Text(row.get("identity_terms")?),
                    V::Text(row.get("alias_terms")?),
                    V::Text(row.get("structured_terms")?),
                    V::Text(row.get("definition_terms")?),
                ])
            })?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Some((table.into(), rows)));
    }
    if table == "semantic_locations" && canonical {
        let mut statement=c.prepare(&format!("SELECT sl.location_id AS old_location_id,sl.record_id AS old_record_id,{} FROM semantic_locations sl JOIN semantic_units su ON su.unit_id=(SELECT min(u.unit_id) FROM semantic_units u WHERE u.location_id=sl.location_id) JOIN records r ON r.record_id=su.record_id LEFT JOIN selected_sections ss ON ss.section_id=sl.section_id ORDER BY sl.location_id",crate::read_units::SEMANTIC_LOCATION_COLUMNS))?;
        let rows = statement
            .query_map([], |row| {
                let location = crate::read_units::location_at(row, 2)?;
                let record: i64 = row.get("old_record_id")?;
                let owners = json(&location.owners)?;
                let address = json(&location.address.ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Null,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "semantic location has no source address",
                        )),
                    )
                })?)?;
                let ordinal: usize = row.get("location_chunk_ordinal")?;
                let hash = crate::embedding_values::hash_bytes(&json_bytes(&(
                    record,
                    &owners,
                    &location.field,
                    &address,
                    ordinal,
                ))?);
                Ok(vec![
                    V::Integer(row.get("old_location_id")?),
                    V::Integer(record),
                    V::Blob(hash),
                    V::Text(owners),
                    text(location.field),
                    V::Text(address),
                    V::Integer(ordinal as i64),
                ])
            })?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Some((table.into(), rows)));
    }
    Ok(None)
}

pub fn native_preservation_rows(c: &rusqlite::Connection) -> Result<TableRows, crate::IndexError> {
    use rusqlite::types::Value as V;
    let models = c
        .prepare("SELECT model_id,identity_json FROM embedding_models ORDER BY model_id")?
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut values = BTreeMap::<(i64, i64), Vec<u8>>::new();
    for (model_id, json) in models {
        let model: crate::SourceSearchProfileIdentity = serde_json::from_str(&json)?;
        let current = crate::schema::native_vector_table(&model)?;
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?)",
            [&current],
            |row| row.get(0),
        )?;
        let mut collected = Vec::new();
        if exists {
            collected.extend(
                c.prepare(&format!(
                    "SELECT input_id,embedding FROM {current} ORDER BY input_id"
                ))?
                .query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?,
            );
        } else {
            let old = crate::schema::vector_table(&model)?;
            let old_exists: bool = c.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?)",
                [&old],
                |row| row.get(0),
            )?;
            if old_exists {
                collected.extend(c.prepare(&format!("SELECT u.input_id,v.embedding FROM {old} v LEFT JOIN semantic_units u USING(unit_id) ORDER BY u.input_id"))?
                    .query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,Vec<u8>>(1)?)))?.collect::<Result<Vec<_>,_>>()?);
            }
            let fallback_exists: bool = c.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='trait_native_values')",
                [],
                |row| row.get(0),
            )?;
            if fallback_exists {
                collected.extend(c.prepare("SELECT t.input_id,t.vector FROM trait_native_values t JOIN embedding_inputs i USING(input_id) WHERE i.model_id=? ORDER BY t.input_id")?
                .query_map([model_id],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,Vec<u8>>(1)?)))?.collect::<Result<Vec<_>,_>>()?);
            }
        }
        for (input, bytes) in collected {
            if let Some(old) = values.insert((model_id, input), bytes.clone())
                && old != bytes
            {
                return Err(crate::IndexError::Invalid(
                    "preservation fixture has conflicting values for one native input".into(),
                ));
            }
        }
        let expected: usize = c.query_row(
            "SELECT count(*) FROM embedding_inputs WHERE model_id=? AND published_hash IS NOT NULL",
            [model_id],
            |row| row.get(0),
        )?;
        if values.keys().filter(|key| key.0 == model_id).count() != expected {
            return Err(crate::IndexError::Invalid(
                "preservation fixture native registry/value coverage differs".into(),
            ));
        }
    }
    Ok(values
        .into_iter()
        .map(|((model, input), bytes)| vec![V::Integer(model), V::Integer(input), V::Blob(bytes)])
        .collect())
}

pub fn reference_preservation_rows(
    c: &rusqlite::Connection,
) -> Result<BTreeMap<String, Vec<Vec<rusqlite::types::Value>>>, crate::IndexError> {
    let families=c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND sql LIKE '%USING vec0%' AND (name GLOB 'semantic_vectors_*' OR name GLOB 'native_vectors_*') ORDER BY name")?
        .query_map([],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
    let tables=c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'core_%' AND name NOT IN ('artifact_context','workspace','schema_migrations','reference_update_jobs','trait_native_values') ORDER BY name")?
        .query_map([],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
    let mut output = BTreeMap::new();
    output.insert("native_values".into(), native_preservation_rows(c)?);
    for table in tables {
        if families
            .iter()
            .any(|family| table == *family || table.starts_with(&format!("{family}_")))
        {
            continue;
        }
        if let Some((name, rows)) = normalized_section_table(c, &table)? {
            output.insert(name, rows);
            continue;
        }
        let quoted = format!("\"{}\"", table.replace('"', "\"\""));
        let columns = c
            .prepare(&format!("PRAGMA table_info({quoted})"))?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        let (sql, count) = if table == "pending_embedding_values" {
            ("SELECT input_id,vector,vector_hash FROM pending_embedding_values ORDER BY input_id".to_owned(),3)
        } else {
            let order = (1..=columns.len())
                .map(|index| index.to_string())
                .collect::<Vec<_>>()
                .join(",");
            (
                format!("SELECT * FROM {quoted} ORDER BY {order}"),
                columns.len(),
            )
        };
        let mut statement = c.prepare(&sql)?;
        let mut rows = statement
            .query_map([], |row| {
                (0..count)
                    .map(|index| row.get(index))
                    .collect::<Result<Vec<rusqlite::types::Value>, _>>()
            })?
            .collect::<Result<Vec<_>, _>>()?;
        if table == "sqlite_sequence" {
            // These counters belong to the physical vec0 families excluded above.
            // Deduplication/repacking may change their allocation high-water marks;
            // every other sequence, including saved-state counters, stays exact.
            rows.retain(|row| {
                !matches!(row.first(), Some(rusqlite::types::Value::Text(name))
                    if families.iter().any(|family|
                        name == &format!("{family}_chunks")
                            || name == &format!("{family}_rowids")))
            });
        }
        output.insert(table, rows);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::reference_preservation_rows;
    use rusqlite::types::Value;

    #[test]
    fn allocation_retirement_oracle_keeps_record_identity_and_exact_bounds() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE embedding_models(model_id INTEGER PRIMARY KEY,identity_json TEXT NOT NULL);
            CREATE TABLE projection_allocations(record_id INTEGER PRIMARY KEY,seed INTEGER,end_id INTEGER,input_hash TEXT);
            INSERT INTO projection_allocations VALUES(77,10,20,'old-digest');").unwrap();
        let before = reference_preservation_rows(&c).unwrap();
        c.execute_batch("ALTER TABLE projection_allocations DROP COLUMN input_hash")
            .unwrap();
        assert_eq!(reference_preservation_rows(&c).unwrap(), before);
        for mutation in [
            "UPDATE projection_allocations SET record_id=78",
            "UPDATE projection_allocations SET seed=11",
            "UPDATE projection_allocations SET end_id=21",
        ] {
            c.execute_batch("BEGIN").unwrap();
            c.execute_batch(mutation).unwrap();
            assert_ne!(
                reference_preservation_rows(&c).unwrap(),
                before,
                "{mutation}"
            );
            c.execute_batch("ROLLBACK").unwrap();
        }
    }

    #[test]
    fn logical_oracle_excludes_only_discovered_vec0_counters_and_keeps_saved_high_water() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE embedding_models(model_id INTEGER PRIMARY KEY,identity_json TEXT NOT NULL);
             CREATE TABLE saved_lists(id INTEGER PRIMARY KEY AUTOINCREMENT);
             INSERT INTO saved_lists DEFAULT VALUES;
             CREATE TABLE native_vectors_decoy_rowids(id INTEGER PRIMARY KEY AUTOINCREMENT);
             INSERT INTO native_vectors_decoy_rowids DEFAULT VALUES;
             CREATE VIRTUAL TABLE semantic_vectors_oracle USING vec0(
                 unit_id INTEGER PRIMARY KEY,embedding FLOAT[2] distance_metric=cosine);",
        )
        .unwrap();
        let bytes = crate::read_units::vector_bytes(&[1.0, 0.0]);
        c.execute(
            "INSERT INTO semantic_vectors_oracle(unit_id,embedding) VALUES(7,?)",
            [&bytes],
        )
        .unwrap();
        let before = reference_preservation_rows(&c).unwrap();
        assert_eq!(
            before["sqlite_sequence"],
            vec![
                vec![
                    Value::Text("native_vectors_decoy_rowids".into()),
                    Value::Integer(1)
                ],
                vec![Value::Text("saved_lists".into()), Value::Integer(1)],
            ]
        );
        c.execute_batch(
            "DROP TABLE semantic_vectors_oracle;
             CREATE VIRTUAL TABLE native_vectors_oracle USING vec0(
                 input_id INTEGER PRIMARY KEY,embedding FLOAT[2] distance_metric=cosine);",
        )
        .unwrap();
        c.execute(
            "INSERT INTO native_vectors_oracle(input_id,embedding) VALUES(99,?)",
            [&bytes],
        )
        .unwrap();
        assert_eq!(reference_preservation_rows(&c).unwrap(), before);
        c.execute_batch(
            "INSERT INTO saved_lists(id) VALUES(40); DELETE FROM saved_lists WHERE id=40;",
        )
        .unwrap();
        let after = reference_preservation_rows(&c).unwrap();
        assert_eq!(after["saved_lists"], before["saved_lists"]);
        assert_ne!(after["sqlite_sequence"], before["sqlite_sequence"]);
        assert_eq!(
            after["sqlite_sequence"][1],
            vec![Value::Text("saved_lists".into()), Value::Integer(40)]
        );
    }
}
