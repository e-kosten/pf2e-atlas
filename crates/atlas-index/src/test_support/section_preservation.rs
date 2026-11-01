//! Test-only logical preservation oracle for section storage migrations.
use std::collections::BTreeMap;

fn normalized_section_table(
    c: &rusqlite::Connection,
    table: &str,
) -> Option<(String, Vec<Vec<rusqlite::types::Value>>)> {
    use rusqlite::types::Value as V;
    let text = |value: Option<String>| value.map(V::Text).unwrap_or(V::Null);
    if table == "selected_sections" {
        let mut statement = c
            .prepare(&format!(
                "SELECT {} FROM selected_sections ss ORDER BY ss.selector",
                crate::selected_sections::COLUMNS
            ))
            .unwrap();
        let rows = statement
            .query_map([], |row| {
                let section = crate::selected_sections::SelectedSection::read(row, 0)?;
                Ok(vec![
                    V::Text(section.part.selector.0.clone()),
                    V::Integer(section.record_id),
                    V::Text(serde_json::to_string(&section.part.owners).unwrap()),
                    V::Text(section.part.field.clone()),
                    V::Integer(section.part.section_ordinal as i64),
                    V::Text(row.get("body_hash")?),
                    V::Text(
                        serde_json::to_value(section.part.role)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .into(),
                    ),
                    V::Text(serde_json::to_string(&section.part).unwrap()),
                    V::Text(row.get("recipe")?),
                ])
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        return Some(("similarity_parts".into(), rows));
    }
    let canonical = c
        .prepare(&format!("PRAGMA table_info({table})"))
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .any(|column| column.unwrap() == "section_id");
    if table == "lexical_units" && canonical {
        let mut statement=c.prepare(&format!("SELECT lu.input_hash AS old_input_hash,lu.record_id AS old_record_id,{},lu.identity_terms,lu.alias_terms,lu.structured_terms,lu.definition_terms FROM lexical_units lu JOIN records r USING(record_id) LEFT JOIN selected_sections ss ON ss.section_id=lu.section_id ORDER BY lu.input_hash",crate::read_units::LEXICAL_LOCATION_COLUMNS)).unwrap();
        let rows = statement
            .query_map([], |row| {
                let location = crate::read_units::location_at(row, 2)?;
                Ok(vec![
                    V::Text(row.get("old_input_hash")?),
                    V::Integer(location.unit_id),
                    V::Integer(row.get("old_record_id")?),
                    V::Text(serde_json::to_string(&location.owners).unwrap()),
                    text(location.field),
                    text(
                        location
                            .address
                            .map(|address| serde_json::to_string(&address).unwrap()),
                    ),
                    V::Text(row.get(15)?),
                    V::Text(row.get("identity_terms")?),
                    V::Text(row.get("alias_terms")?),
                    V::Text(row.get("structured_terms")?),
                    V::Text(row.get("definition_terms")?),
                ])
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        return Some((table.into(), rows));
    }
    if table == "semantic_locations" && canonical {
        let mut statement=c.prepare(&format!("SELECT sl.location_id AS old_location_id,sl.record_id AS old_record_id,{} FROM semantic_locations sl JOIN semantic_units su ON su.unit_id=(SELECT min(u.unit_id) FROM semantic_units u WHERE u.location_id=sl.location_id) JOIN records r ON r.record_id=su.record_id LEFT JOIN selected_sections ss ON ss.section_id=sl.section_id ORDER BY sl.location_id",crate::read_units::SEMANTIC_LOCATION_COLUMNS)).unwrap();
        let rows = statement
            .query_map([], |row| {
                let location = crate::read_units::location_at(row, 2)?;
                let record: i64 = row.get("old_record_id")?;
                let owners = serde_json::to_string(&location.owners).unwrap();
                let address = serde_json::to_string(&location.address.unwrap()).unwrap();
                let ordinal: usize = row.get("location_chunk_ordinal")?;
                let hash = crate::embedding_values::hash_bytes(
                    &serde_json::to_vec(&(record, &owners, &location.field, &address, ordinal))
                        .unwrap(),
                );
                Ok(vec![
                    V::Integer(row.get("old_location_id")?),
                    V::Integer(record),
                    V::Blob(hash),
                    V::Text(owners),
                    text(location.field),
                    V::Text(address),
                    V::Integer(ordinal as i64),
                ])
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        return Some((table.into(), rows));
    }
    None
}

pub fn reference_preservation_rows(
    c: &rusqlite::Connection,
) -> BTreeMap<String, Vec<Vec<rusqlite::types::Value>>> {
    let tables=c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'core_%' AND name NOT IN ('artifact_context','workspace','schema_migrations') ORDER BY name").unwrap().query_map([],|row|row.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    tables
        .into_iter()
        .map(|table| {
            if let Some(normalized) = normalized_section_table(c, &table) {
                return normalized;
            }
            let quoted = format!("\"{}\"", table.replace('"', "\"\""));
            let columns = c
                .prepare(&format!("PRAGMA table_info({quoted})"))
                .unwrap()
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let order = (1..=columns.len())
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let mut statement = c
                .prepare(&format!("SELECT * FROM {quoted} ORDER BY {order}"))
                .unwrap();
            let rows = statement
                .query_map([], |row| {
                    (0..columns.len())
                        .map(|i| row.get(i))
                        .collect::<Result<Vec<rusqlite::types::Value>, _>>()
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, rows)
        })
        .collect()
}
