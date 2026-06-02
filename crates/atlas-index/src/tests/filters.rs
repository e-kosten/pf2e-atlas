use std::fs;

use atlas_domain::metadata::{
    MetadataNumberField, MetadataNumberMatch, MetadataPredicate, MetadataSetField, MetadataSetMatch,
};
use atlas_domain::{MetricMatch, NumericMatch, RecordKey, RecordKind};
use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

use super::{create_valid_artifact_database, temp_db_path};
use crate::read::search::filters::{
    CompiledSqliteEligibleRecordKeyset, FilterCompileError, SqliteEligibleRecordKeyset,
    SqliteFilterSqlQuery, SqliteFilteredRecordSort,
};
use crate::read::search::vector::compile_vector_knn_query;
use crate::read::sql::SqlBindValue;
use crate::{FilteredRecordSort, FtsQuery, SqliteIndexReader};

#[test]
fn compiles_empty_filter_to_default_visible_keyset() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-empty");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records SET is_default_visible = 0 WHERE record_key = 'actions:testAction3'",
        [],
    )?;

    let compiled = SqliteEligibleRecordKeyset::new(None).compile()?;
    let keys = query_eligible_keys(&connection, &compiled)?;

    assert_eq!(keys, vec!["actions:testAction1", "actions:testAction2"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn compiles_boolean_and_basic_record_filters() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-basic");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET level = CASE record_key WHEN 'actions:testAction1' THEN 1 ELSE 4 END,
             rarity = CASE record_key WHEN 'actions:testAction2' THEN 'rare' ELSE 'common' END",
        [],
    )?;

    let filter = atlas_domain::SearchFilterNode::all_of(vec![
        atlas_domain::SearchFilterNode::pack("actions"),
        atlas_domain::SearchFilterNode::record_kind(RecordKind::Rule),
        atlas_domain::SearchFilterNode::level(NumericMatch::Gte { value: 2.0 }),
    ]);
    let compiled = SqliteEligibleRecordKeyset::new(Some(&filter)).compile()?;
    let keys = query_eligible_keys(&connection, &compiled)?;

    assert_eq!(keys, vec!["actions:testAction2", "actions:testAction3"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn rejects_empty_boolean_groups_at_compile_boundary() {
    assert_eq!(
        SqliteEligibleRecordKeyset::new(Some(&atlas_domain::SearchFilterNode::any_of(Vec::new())))
            .compile()
            .unwrap_err(),
        FilterCompileError::InvalidValue(
            "filter `any_of` must contain at least one child".to_string()
        )
    );
    assert_eq!(
        SqliteEligibleRecordKeyset::new(Some(&atlas_domain::SearchFilterNode::all_of(Vec::new())))
            .compile()
            .unwrap_err(),
        FilterCompileError::InvalidValue(
            "filter `all_of` must contain at least one child".to_string()
        )
    );
}

#[test]
fn composes_filtered_record_key_queries_from_eligible_records()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-record-query");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET level = CASE record_key
             WHEN 'actions:testAction1' THEN 1
             WHEN 'actions:testAction2' THEN 4
             ELSE 2
         END",
        [],
    )?;

    let filter = atlas_domain::SearchFilterNode::record_kind(RecordKind::Rule);
    let compiled = SqliteEligibleRecordKeyset::new(Some(&filter))
        .compile()?
        .into_record_keys_query(SqliteFilteredRecordSort::LevelDesc, Some(2), Some(0));
    let keys = query_filtered_record_keys(&connection, &compiled)?;

    assert_eq!(keys, vec!["actions:testAction2", "actions:testAction3"]);
    assert!(compiled.sql.contains("WITH eligible(record_key) AS"));
    assert!(
        compiled
            .sql
            .contains("JOIN records r ON r.record_key = e.record_key")
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn shared_sqlite_keyset_applies_same_filter_to_lookup_fts_identity_and_vector()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-cross-surface");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET level = CASE record_key
             WHEN 'actions:testAction1' THEN 1
             WHEN 'actions:testAction2' THEN 2
             ELSE 3
         END,
         is_default_visible = CASE record_key
             WHEN 'actions:testAction3' THEN 0
             ELSE 1
         END",
        [],
    )?;
    drop(connection);

    let filter = atlas_domain::SearchFilterNode::level(NumericMatch::Gte { value: 2.0 });
    let reader = SqliteIndexReader::open_read_only(&path)?;
    let filtered_keys =
        reader.list_filtered_record_keys(Some(&filter), FilteredRecordSort::RecordKey, 10, 0)?;
    let fts_query = FtsQuery::from_tokens(vec!["action".to_string()]).expect("valid FTS query");
    let fts_keys = reader.query_fts_record_keys(&fts_query, Some(&filter), 10)?;
    let identity_matches =
        reader.resolve_record_identity_matches("Test Action 2", "test action 2", Some(&filter))?;

    assert_eq!(
        filtered_keys
            .record_keys
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["actions:testAction2"]
    );
    assert_eq!(
        fts_keys.iter().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["actions:testAction2"]
    );
    assert_eq!(
        identity_matches
            .iter()
            .map(|matched| matched.record_key.to_string())
            .collect::<Vec<_>>(),
        vec!["actions:testAction2"]
    );
    assert!(
        reader
            .resolve_record_identity_matches("Test Action 3", "test action 3", Some(&filter))?
            .is_empty()
    );

    let keyset = SqliteEligibleRecordKeyset::new(Some(&filter)).compile()?;
    let vector_query = compile_vector_knn_query(&[0.25, 0.5], Some(&filter), 10, false)?;
    assert!(vector_query.sql.contains(&keyset.eligible_cte_sql()));
    assert_eq!(
        vector_query.parameters[..keyset.parameters().len()],
        keyset.parameters()[..],
        "vector search should reuse the same structured-filter binds before its vector parameters"
    );

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn compiles_reference_trait_metric_and_spell_filters() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-side-tables");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO reference_edges (from_record_key, to_record_key, reference_text, source_kind, visibility)
             VALUES ('actions:testAction1', 'actions:testAction2', 'fixture', 'description', 'public')",
        [],
    )?;
    connection.execute(
        "INSERT INTO reference_edges (from_record_key, to_record_key, reference_text, source_kind, visibility)
             VALUES ('actions:testAction3', 'actions:testAction2', 'private-fixture', 'private_notes', 'private')",
        [],
    )?;
    connection.execute(
        "INSERT INTO reference_edges (from_record_key, to_record_key, reference_text, source_kind, visibility)
             VALUES ('actions:testAction2', 'actions:testAction2', 'embedded-fixture', 'embedded_item_description', 'public')",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_traits (record_key, trait) VALUES ('actions:testAction1', 'healing')",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_metrics (record_key, metric_domain, metric_key, value_type, number_value)
             VALUES ('actions:testAction1', 'item', 'defense.ac', 'number', 18)",
        [],
    )?;
    connection.execute(
        "INSERT INTO spell_records (
              record_key, traditions_json, spell_kinds_json, range_text, area_type, sustained,
              basic_save, damage_types_json
            )
            VALUES ('actions:testAction1', '[\"primal\"]', '[\"focus\"]', '30 feet', 'burst', 0, 1, '[\"vitality\"]')",
        [],
    )?;

    let filter = atlas_domain::SearchFilterNode::all_of(vec![
        atlas_domain::SearchFilterNode::links_to(RecordKey::parse("actions:testAction2")?),
        atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Set {
            field: MetadataSetField::Traits,
            r#match: MetadataSetMatch::Includes {
                value: "healing".to_string(),
            },
        }),
        atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Set {
            field: MetadataSetField::Traditions,
            r#match: MetadataSetMatch::Includes {
                value: "primal".to_string(),
            },
        }),
        atlas_domain::SearchFilterNode::metric("defense.ac", MetricMatch::Gte { value: 18.0 }),
    ]);
    let compiled = SqliteEligibleRecordKeyset::new(Some(&filter)).compile()?;
    let keys = query_eligible_keys(&connection, &compiled)?;

    assert_eq!(keys, vec!["actions:testAction1"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_filters_that_cannot_be_lowered_authoritatively() {
    let derived_tags = atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Set {
        field: MetadataSetField::DerivedTags,
        r#match: MetadataSetMatch::Includes {
            value: "area-damage".to_string(),
        },
    });
    let hands = atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Number {
        field: MetadataNumberField::Hands,
        r#match: MetadataNumberMatch::Eq { value: 2.0 },
    });

    assert_eq!(
        SqliteEligibleRecordKeyset::new(Some(&derived_tags))
            .compile()
            .unwrap_err(),
        FilterCompileError::Unsupported {
            filter: "metadata.set.derived_tags".to_string(),
        }
    );
    assert_eq!(
        SqliteEligibleRecordKeyset::new(Some(&hands))
            .compile()
            .unwrap_err(),
        FilterCompileError::Unsupported {
            filter: "metadata.number.hands".to_string(),
        }
    );
}

fn query_eligible_keys(
    connection: &Connection,
    compiled: &CompiledSqliteEligibleRecordKeyset,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut statement =
        connection.prepare(&format!("{} ORDER BY record_key", compiled.select_sql()))?;
    let parameters = rusqlite_values(compiled.parameters());
    let rows = statement.query_map(params_from_iter(parameters.iter()), |row| {
        row.get::<_, String>(0)
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn query_filtered_record_keys(
    connection: &Connection,
    compiled: &SqliteFilterSqlQuery,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut statement = connection.prepare(&compiled.sql)?;
    let parameters = rusqlite_values(&compiled.parameters);
    let rows = statement.query_map(params_from_iter(parameters.iter()), |row| {
        row.get::<_, String>(0)
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn rusqlite_values(values: &[SqlBindValue]) -> Vec<Value> {
    values
        .iter()
        .map(|value| match value {
            SqlBindValue::Integer(value) => Value::Integer(*value),
            SqlBindValue::Real(value) => Value::Real(*value),
            SqlBindValue::Text(value) => Value::Text(value.clone()),
            SqlBindValue::Blob(value) => Value::Blob(value.clone()),
        })
        .collect()
}
