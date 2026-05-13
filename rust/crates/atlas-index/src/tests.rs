use std::fs;
use std::mem::size_of;
use std::path::PathBuf;

use atlas_artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, artifact_metadata_keys,
};
use atlas_artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;
use atlas_domain::metadata::{
    CollectionOperator, MetadataNumberField, MetadataPredicate, MetadataSetField, NumberOperator,
};
use atlas_domain::{MetricOperator, NumericMatch, RecordFamily, RecordKey, ScalarValue};
use atlas_embedding::default_embedding_model_spec;
use rusqlite::{Connection, params_from_iter};

use crate::filters::{
    EligibleRecordsQuery, FilterCompileError, FilteredRecordKeysQuery, FilteredRecordSort,
    compile_eligible_records_query, compile_filtered_record_keys_query,
};
use crate::records::{load_persisted_record_set, load_persisted_records};
use crate::{
    ArtifactContractFamily, ValidationCode, ValidationStatus, validate_index,
    validate_vector_index, validate_vector_index_with_loader, write_vector_index,
    write_vector_index_with_loader,
};

#[test]
fn reports_valid_artifact_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("valid");
    create_contract_database(&path)?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Ok);
    assert_eq!(report.code, ValidationCode::Ok);
    assert_eq!(
        report.artifact_contract_version.as_deref(),
        Some(ARTIFACT_CONTRACT_VERSION)
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_legacy_metadata_without_accepting_it_as_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("legacy");
    let connection = Connection::open(&path)?;
    connection.execute(
        "CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )?;
    connection.execute(
        "INSERT INTO metadata (key, value) VALUES ('schema_version', '25')",
        [],
    )?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::MissingArtifactMetadata);
    assert_eq!(report.legacy_schema_version.as_deref(), Some("25"));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_missing_required_metadata_key() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("missing-key");
    create_contract_database_without(&path, artifact_metadata_keys::EMBEDDING_DTYPE)?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::MissingRequiredMetadata);
    assert_eq!(
        report.missing_keys,
        vec![artifact_metadata_keys::EMBEDDING_DTYPE.to_string()]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_stale_source_signature() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("stale-source");
    create_contract_database_with_override(
        &path,
        artifact_metadata_keys::SOURCE_SIGNATURE,
        "stale:fixture",
    )?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::StaleSourceSignature);
    assert_eq!(report.diagnostics.len(), 1);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_embedding_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("embedding-mismatch");
    create_contract_database_with_override(
        &path,
        artifact_metadata_keys::EMBEDDING_MODEL_ID,
        "BAAI/bge-small-en-v1.5",
    )?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::EmbeddingMismatch);
    assert_eq!(
        report.diagnostics[0].key.as_deref(),
        Some(artifact_metadata_keys::EMBEDDING_MODEL_ID)
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_unsupported_schema_version() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("unsupported-schema");
    create_contract_database_with_override(&path, artifact_metadata_keys::SCHEMA_VERSION, "2")?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::UnsupportedSchemaVersion);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_missing_required_artifact_table() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("missing-contract-table");
    let connection = Connection::open(&path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata(&connection, None)?;
    connection.execute("DROP TABLE item_records", [])?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].family, ArtifactContractFamily::Schema);
    assert_eq!(
        report.diagnostics[0].key.as_deref(),
        Some("table:item_records")
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_fts_rows_for_hidden_records() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("hidden-fts");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records SET is_default_visible = 0 WHERE record_key = 'actions:testAction1'",
        [],
    )?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Fts
            && diagnostic.key.as_deref() == Some("records_fts:hidden_rows")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn accepts_complete_document_embedding_cache() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("document-embedding-cache");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_document_embedding_cache_rows(&connection, 384, 384 * size_of::<f32>())?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Ok);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_document_embedding_cache_dimension_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("document-embedding-cache-dimensions");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_document_embedding_cache_rows(&connection, 383, 384 * size_of::<f32>())?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Embedding
            && diagnostic.key.as_deref() == Some("document_embedding_cache:dimensions")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_incomplete_document_embedding_cache_coverage() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("document-embedding-cache-coverage");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO document_embedding_cache (record_key, semantic_input_hash, dimensions, vector_blob)
         VALUES ('actions:testAction1', 'fixture-hash', 384, zeroblob(1536))",
        [],
    )?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Embedding
            && diagnostic.key.as_deref() == Some("document_embedding_cache:default_visible_count")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn vector_validation_reports_sqlite_vec_unavailable() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("vector-extension-unavailable");
    create_contract_database(&path)?;

    let report = validate_vector_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::VectorExtensionUnavailable);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Embedding
            && diagnostic.key.as_deref() == Some("sqlite_vec")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn vector_validation_reports_loader_failure() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("vector-loader-failure");
    create_contract_database(&path)?;

    let report =
        validate_vector_index_with_loader(&path, |_| Err("fixture loader failed".to_string()))?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::VectorExtensionUnavailable);
    assert_eq!(
        report.diagnostics[0].actual.as_deref(),
        Some("fixture loader failed")
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn vector_write_reports_sqlite_vec_unavailable() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("vector-write-extension-unavailable");
    create_contract_database(&path)?;

    let report = write_vector_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::VectorExtensionUnavailable);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Embedding
            && diagnostic.key.as_deref() == Some("sqlite_vec")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn vector_write_reports_loader_failure() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("vector-write-loader-failure");
    create_contract_database(&path)?;

    let report =
        write_vector_index_with_loader(&path, |_| Err("fixture loader failed".to_string()))?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::VectorExtensionUnavailable);
    assert_eq!(
        report.diagnostics[0].actual.as_deref(),
        Some("fixture loader failed")
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn loads_persisted_records_from_artifact_tables() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-records");
    create_contract_database(&path)?;

    let records = load_persisted_records(&path)?;

    assert_eq!(records.len(), 3);
    assert_eq!(records[0].key.to_string(), "actions:testAction1");
    assert_eq!(records[0].record_family.as_str(), "rule");
    assert_eq!(records[0].pack_name.as_str(), "actions");
    assert_eq!(records[0].traits, Vec::<String>::new());
    assert!(records[0].is_default_visible);
    assert_eq!(records[0].source_path, "packs/actions/test-action-1.json");
    fs::remove_file(path)?;
    Ok(())
}

fn insert_document_embedding_cache_rows(
    connection: &Connection,
    dimensions: usize,
    byte_len: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for index in 1..=3 {
        let record_key = format!("actions:testAction{index}");
        let semantic_input_hash = format!("fixture-hash-{index}");
        connection.execute(
            "INSERT INTO document_embedding_cache (record_key, semantic_input_hash, dimensions, vector_blob)
             VALUES (?1, ?2, ?3, zeroblob(?4))",
            rusqlite::params![
                record_key,
                semantic_input_hash,
                dimensions as i64,
                byte_len as i64,
            ],
        )?;
    }
    Ok(())
}

#[test]
fn loads_persisted_record_set_relationship_tables() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-record-set");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO reference_edges (from_record_key, to_record_key, display_text, reference_text)
             VALUES ('actions:testAction1', 'actions:testAction2', 'Test Action 2', '@UUID[Compendium.pf2e.actions.Item.testAction2]')",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_aliases (canonical_record_key, alias_text, normalized_alias, source_kind, source_ref)
             VALUES ('actions:testAction1', 'Test Alias', 'test alias', 'compendium_source', 'fixture')",
        [],
    )?;
    connection.execute(
        "INSERT INTO remaster_links (remaster_record_key, legacy_record_key, source_kind, source_ref)
             VALUES ('actions:testAction1', 'actions:testAction3', 'migration', 'fixture')",
        [],
    )?;
    drop(connection);

    let record_set = load_persisted_record_set(&path)?;

    assert_eq!(record_set.records.len(), 3);
    assert_eq!(record_set.reference_edges.len(), 1);
    assert_eq!(record_set.aliases.len(), 1);
    assert_eq!(record_set.remaster_links.len(), 1);
    assert_eq!(
        record_set.reference_edges[0].to_record_key.to_string(),
        "actions:testAction2"
    );
    assert_eq!(record_set.aliases[0].source.as_str(), "compendium_source");
    assert_eq!(record_set.remaster_links[0].source.as_str(), "migration");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn compiles_empty_filter_to_default_visible_keyset() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-empty");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records SET is_default_visible = 0 WHERE record_key = 'actions:testAction3'",
        [],
    )?;

    let compiled = compile_eligible_records_query(None)?;
    let keys = query_eligible_keys(&connection, &compiled)?;

    assert_eq!(keys, vec!["actions:testAction1", "actions:testAction2"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn compiles_boolean_and_basic_record_filters() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-basic");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET level = CASE record_key WHEN 'actions:testAction1' THEN 1 ELSE 4 END,
             rarity = CASE record_key WHEN 'actions:testAction2' THEN 'rare' ELSE 'common' END",
        [],
    )?;

    let filter = atlas_domain::SearchFilterNode::all_of(vec![
        atlas_domain::SearchFilterNode::pack("actions"),
        atlas_domain::SearchFilterNode::record_family(RecordFamily::Rule),
        atlas_domain::SearchFilterNode::level(NumericMatch::Gte { value: 2.0 }),
    ]);
    let compiled = compile_eligible_records_query(Some(&filter))?;
    let keys = query_eligible_keys(&connection, &compiled)?;

    assert_eq!(keys, vec!["actions:testAction2", "actions:testAction3"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn composes_filtered_record_key_queries_from_eligible_records()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-record-query");
    create_contract_database(&path)?;
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

    let filter = atlas_domain::SearchFilterNode::record_family(RecordFamily::Rule);
    let compiled = compile_filtered_record_keys_query(
        Some(&filter),
        FilteredRecordSort::LevelDesc,
        Some(2),
        Some(0),
    )?;
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
fn compiles_reference_trait_metric_and_spell_filters() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("filter-side-tables");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO reference_edges (from_record_key, to_record_key, reference_text)
             VALUES ('actions:testAction1', 'actions:testAction2', '@UUID[fixture]')",
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
            op: CollectionOperator::Includes,
            value: Some("healing".to_string()),
        }),
        atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Set {
            field: MetadataSetField::Traditions,
            op: CollectionOperator::Includes,
            value: Some("primal".to_string()),
        }),
        atlas_domain::SearchFilterNode::metric(
            "defense.ac",
            MetricOperator::Gte,
            ScalarValue::Number(18.0),
        ),
    ]);
    let compiled = compile_eligible_records_query(Some(&filter))?;
    let keys = query_eligible_keys(&connection, &compiled)?;

    assert_eq!(keys, vec!["actions:testAction1"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_filters_that_cannot_be_lowered_authoritatively() {
    let derived_tags = atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Set {
        field: MetadataSetField::DerivedTags,
        op: CollectionOperator::Includes,
        value: Some("area-damage".to_string()),
    });
    let hands = atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Number {
        field: MetadataNumberField::Hands,
        op: NumberOperator::Eq,
        value: Some(2.0),
        min: None,
        max: None,
    });

    assert_eq!(
        compile_eligible_records_query(Some(&derived_tags)).unwrap_err(),
        FilterCompileError::Unsupported {
            filter: "metadata.set.derived_tags".to_string(),
        }
    );
    assert_eq!(
        compile_eligible_records_query(Some(&hands)).unwrap_err(),
        FilterCompileError::Unsupported {
            filter: "metadata.number.hands".to_string(),
        }
    );
}

fn query_eligible_keys(
    connection: &Connection,
    compiled: &EligibleRecordsQuery,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut statement = connection.prepare(&format!("{} ORDER BY record_key", compiled.sql))?;
    let rows = statement.query_map(params_from_iter(compiled.parameters.iter()), |row| {
        row.get::<_, String>(0)
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn query_filtered_record_keys(
    connection: &Connection,
    compiled: &FilteredRecordKeysQuery,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut statement = connection.prepare(&compiled.sql)?;
    let rows = statement.query_map(params_from_iter(compiled.parameters.iter()), |row| {
        row.get::<_, String>(0)
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn create_contract_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata(&connection, None)?;
    insert_minimal_contract_rows(&connection)?;
    Ok(())
}

fn create_contract_database_without(
    path: &PathBuf,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    for (key, value) in valid_metadata_entries() {
        if key != omitted_key {
            connection.execute(
                "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
                [key, value],
            )?;
        }
    }
    Ok(())
}

fn create_contract_database_with_override(
    path: &PathBuf,
    override_key: &str,
    override_value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata(&connection, Some((override_key, override_value)))?;
    Ok(())
}

fn insert_contract_metadata(
    connection: &Connection,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (key, mut value) in valid_metadata_entries() {
        if let Some((override_key, override_value)) = override_entry
            && key == override_key
        {
            value = override_value;
        }
        connection.execute(
            "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
    }
    Ok(())
}

fn valid_metadata_entries() -> Vec<(&'static str, &'static str)> {
    let embedding_spec = default_embedding_model_spec();
    vec![
        (
            artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
            ARTIFACT_CONTRACT_VERSION,
        ),
        (
            artifact_metadata_keys::SCHEMA_VERSION,
            ARTIFACT_SCHEMA_VERSION,
        ),
        (artifact_metadata_keys::SOURCE_KIND, "foundry-pf2e"),
        (
            artifact_metadata_keys::SOURCE_SIGNATURE,
            "foundry-pf2e:fixture",
        ),
        (artifact_metadata_keys::SOURCE_RECORD_COUNT, "3"),
        (artifact_metadata_keys::ARTIFACT_RECORD_COUNT, "3"),
        (artifact_metadata_keys::GENERATED_RECORD_COUNT, "0"),
        (artifact_metadata_keys::CONTENT_HASH_ALGORITHM, "sha256"),
        (
            artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
            embedding_spec.provider_family,
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            embedding_spec.model_id,
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
            embedding_spec.model_revision,
        ),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            embedding_spec.tokenizer_id,
        ),
        (
            artifact_metadata_keys::EMBEDDING_POOLING,
            embedding_spec.pooling.as_str(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_NORMALIZATION,
            embedding_spec.normalization.as_str(),
        ),
        (artifact_metadata_keys::EMBEDDING_DIMENSIONS, "384"),
        (
            artifact_metadata_keys::EMBEDDING_DTYPE,
            embedding_spec.dtype.as_str(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
            embedding_spec.distance_metric.as_str(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
            embedding_spec.document_prefix,
        ),
        (
            artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
            embedding_spec.query_prefix,
        ),
        (
            artifact_metadata_keys::FTS_TOKENIZER,
            "unicode61 remove_diacritics 2",
        ),
        (
            artifact_metadata_keys::ADJACENT_MANIFEST_PATH,
            "manifest.json",
        ),
    ]
}

fn create_minimal_contract_schema(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute_batch(CREATE_ARTIFACT_SCHEMA_SQL)?;
    Ok(())
}

fn insert_minimal_contract_rows(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute(
        "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
             VALUES ('actions', 'Actions', 'Item', 'packs/actions', 'packs/actions', 3)",
        [],
    )?;
    for index in 1..=3 {
        let record_key = format!("actions:testAction{index}");
        let record_id = format!("testAction{index}");
        let name = format!("Test Action {index}");
        let normalized_name = name.to_lowercase();
        let source_path = format!("packs/actions/test-action-{index}.json");
        connection.execute(
            "INSERT INTO records (
                  record_key, id, name, normalized_name, record_family, pack_name, pack_label,
                  foundry_document_type, foundry_record_type, traits_json, publication_remaster,
                  publication_family, taxonomy_families_json, variant_axes_json, variant_source,
                  source_path, is_default_visible, search_text_projection, raw_json
                ) VALUES (?1, ?2, ?3, ?4, 'rule', 'actions', 'Actions', 'Item', 'action',
                  '[]', 0, 'unknown', '[]', '[]', 'none', ?5, 1, ?3, '{}')",
            [
                record_key.as_str(),
                record_id.as_str(),
                name.as_str(),
                normalized_name.as_str(),
                source_path.as_str(),
            ],
        )?;
        connection.execute(
            "INSERT INTO records_fts (record_key, name, search_text_projection)
                 VALUES (?1, ?2, ?2)",
            [record_key.as_str(), name.as_str()],
        )?;
    }
    Ok(())
}

fn temp_db_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-index-{name}-{}-{}.sqlite",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_file(&path);
    path
}
