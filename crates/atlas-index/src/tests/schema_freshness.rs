use std::collections::BTreeMap;

use rusqlite::Connection;

use crate::artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;

#[test]
fn checked_in_diesel_schema_matches_migration_tables_and_columns()
-> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open_in_memory()?;
    connection.execute_batch(CREATE_ARTIFACT_SCHEMA_SQL)?;

    let migration_columns = migration_table_columns(&connection)?;
    let diesel_columns = diesel_schema_columns();

    assert_eq!(diesel_columns, migration_columns);
    Ok(())
}

#[test]
fn schema_inventory_requirements_match_migration_schema() -> Result<(), Box<dyn std::error::Error>>
{
    let connection = Connection::open_in_memory()?;
    connection.execute_batch(CREATE_ARTIFACT_SCHEMA_SQL)?;

    let migration_columns = migration_table_columns(&connection)?;
    for table in crate::artifact::inventory::required_tables() {
        assert!(
            migration_columns.contains_key(table.name()),
            "required table `{}` is missing from migration schema",
            table.name()
        );
    }
    for (table, required_columns) in crate::artifact::inventory::required_columns() {
        let Some(columns) = migration_columns.get(table.name()) else {
            panic!(
                "required columns for table `{}` refer to a table missing from migration schema",
                table.name()
            );
        };
        for column in *required_columns {
            assert!(
                columns.iter().any(|name| name == column.name()),
                "required column `{}` is missing from migration table `{}`",
                column.name(),
                table.name()
            );
        }
    }
    Ok(())
}

fn migration_table_columns(
    connection: &Connection,
) -> Result<BTreeMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let mut statement = connection.prepare(
        "SELECT name
         FROM sqlite_schema
         WHERE type = 'table'
           AND name NOT LIKE 'sqlite_%'
           AND name NOT GLOB 'records_fts_*'
         ORDER BY name",
    )?;
    let table_names = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut tables = BTreeMap::new();
    for table_name in table_names {
        let pragma = format!("PRAGMA table_info({table_name})");
        let mut statement = connection.prepare(&pragma)?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;
        tables.insert(table_name, columns);
    }
    Ok(tables)
}

fn diesel_schema_columns() -> BTreeMap<String, Vec<String>> {
    let schema = include_str!("../schema.rs");
    let mut tables = BTreeMap::<String, Vec<String>>::new();
    let mut current_table = None::<String>;
    let mut pending_sql_name = None::<String>;

    for line in schema.lines() {
        let line = line.trim();
        if let Some(sql_name) = line
            .strip_prefix("#[sql_name = \"")
            .and_then(|line| line.strip_suffix("\"]"))
        {
            pending_sql_name = Some(sql_name.to_string());
            continue;
        }
        if current_table.is_none()
            && line.ends_with('{')
            && line.contains('(')
            && !line.starts_with("diesel::")
        {
            let Some((table_name, _)) = line.split_once('(') else {
                continue;
            };
            current_table = Some(table_name.trim().to_string());
            continue;
        }
        if line == "}" {
            current_table = None;
            pending_sql_name = None;
            continue;
        }
        let Some(table_name) = current_table.as_ref() else {
            continue;
        };
        let Some((column_name, _)) = line.split_once("->") else {
            continue;
        };
        let column_name = pending_sql_name
            .take()
            .unwrap_or_else(|| column_name.trim().to_string());
        tables
            .entry(table_name.clone())
            .or_default()
            .push(column_name);
    }

    tables
}
