use std::fs;
use std::mem::size_of;
use std::path::PathBuf;

use atlas_artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, artifact_metadata_keys,
};
use atlas_artifact::schema::{record_vector_index_create_sql, record_vector_index_insert_sql};
use atlas_artifact::storage::encode_f32_vector_blob;
use atlas_artifact::test_support::{
    create_minimal_contract_schema, insert_contract_metadata_omitting, insert_minimal_contract_rows,
};
use atlas_domain::metadata::{
    MetadataNumberField, MetadataNumberMatch, MetadataPredicate, MetadataSetField, MetadataSetMatch,
};
use atlas_domain::{MetricMatch, NumericMatch, RecordFamily, RecordKey};
use atlas_embedding::{
    EMBEDDING_UNIT_POLICY_VERSION, EmbeddingModelId, default_embedding_model_spec,
    embedding_model_spec,
};
use rusqlite::{Connection, params_from_iter};

use crate::filters::{
    EligibleRecordsQuery, FilterCompileError, FilteredRecordKeysQuery, FilteredRecordSort,
    compile_eligible_records_query, compile_filtered_record_keys_query,
};
use crate::vector::compile_vector_knn_query;
use crate::{
    ArtifactContractFamily, AtlasIndex, ReferenceEdgeDirection, ValidationCode, ValidationStatus,
    VectorQueryError,
};

mod fts;

#[test]
fn reports_valid_artifact_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("valid");
    create_contract_database(&path)?;

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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
        "unknown/model",
    )?;

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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
fn reports_embedding_unit_policy_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("embedding-unit-policy-mismatch");
    create_contract_database_with_override(
        &path,
        artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION,
        "legacy-child-sections/v0",
    )?;

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::EmbeddingMismatch);
    assert_eq!(
        report.diagnostics[0].key.as_deref(),
        Some(artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION)
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn accepts_known_non_default_embedding_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("known-non-default-embedding");
    let connection = Connection::open(&path)?;
    create_minimal_contract_schema(&connection)?;
    let embedding_spec = embedding_model_spec(EmbeddingModelId::BgeSmallEnV15);
    insert_contract_metadata_entries(
        &connection,
        valid_metadata_entries_for_embedding(embedding_spec),
        None,
    )?;
    insert_minimal_contract_rows(&connection)?;
    drop(connection);

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Ok);
    assert_eq!(report.code, ValidationCode::Ok);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_unsupported_schema_version() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("unsupported-schema");
    create_contract_database_with_override(&path, artifact_metadata_keys::SCHEMA_VERSION, "2")?;

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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
        "INSERT INTO document_embedding_cache (
           embedding_unit_key, record_key, unit_kind, label, ordinal,
           semantic_input_hash, dimensions, vector_blob
         )
         VALUES ('actions:testAction1#parent', 'actions:testAction1', 'parent', NULL, 0,
           'fixture-hash', 384, zeroblob(1536))",
        [],
    )?;
    drop(connection);

    let report = AtlasIndex::open_read_only(&path)?.validate()?;

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
fn vector_validation_reports_missing_vector_table() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("vector-table-missing");
    create_contract_database(&path)?;

    let report = AtlasIndex::open_read_only_with_vectors(&path)?.validate_vector_index()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Schema
            && diagnostic.key.as_deref() == Some("table:record_vector_index")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn check_embedding_readiness_skips_deep_vector_coverage() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("vector-check-skips-coverage");
    create_contract_database(&path)?;
    crate::vector::register_sqlite_vec_extension()?;
    let connection = Connection::open(&path)?;
    insert_document_embedding_cache_rows(&connection, 384, 384 * size_of::<f32>())?;
    connection.execute_batch(&record_vector_index_create_sql(384))?;
    connection.execute(
        &record_vector_index_insert_sql(),
        (1_i64, encode_f32_vector_blob(&vec![0.0_f32; 384])),
    )?;
    drop(connection);

    let check_report =
        AtlasIndex::open_read_only_with_vectors(&path)?.check_embedding_readiness_report();
    let validate_report =
        AtlasIndex::open_read_only_with_vectors(&path)?.validate_vector_index()?;

    assert_eq!(check_report.status, ValidationStatus::Ok);
    assert_eq!(validate_report.status, ValidationStatus::Error);
    assert!(validate_report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Embedding
            && diagnostic.key.as_deref()
                == Some("record_vector_index:document_embedding_cache_count")
    }));

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn loads_persisted_records_from_artifact_tables() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-records");
    create_contract_database(&path)?;

    let records = AtlasIndex::open_read_only(&path)?.load_records()?;

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

#[test]
fn loads_persisted_records_by_key_scopes_detail_tables() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-records-by-key-scoped");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO record_content (
           record_key, ordinal, source_kind, visibility, contributes_to_search,
           contributes_to_references, label, content_json
         ) VALUES (
           'actions:testAction1', 0, 'description', 'public', 1, 1, NULL,
           '{\"blocks\":[]}'
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_content (
           record_key, ordinal, source_kind, visibility, contributes_to_search,
           contributes_to_references, label, content_json
         ) VALUES (
           'actions:testAction2', 0, 'description', 'public', 1, 1, NULL,
           'not json'
         )",
        [],
    )?;
    drop(connection);

    let records = AtlasIndex::open_read_only(&path)?
        .load_records_by_key(&[RecordKey::parse("actions:testAction1")?])?;

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].key.to_string(), "actions:testAction1");
    assert_eq!(records[0].supplemental_content.len(), 1);
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
        let embedding_unit_key = format!("{record_key}#parent");
        let semantic_input_hash = format!("fixture-hash-{index}");
        connection.execute(
            "INSERT INTO document_embedding_cache (
               embedding_unit_key, record_key, unit_kind, label, ordinal,
               semantic_input_hash, dimensions, vector_blob
             )
             VALUES (?1, ?2, 'parent', NULL, 0, ?3, ?4, zeroblob(?5))",
            rusqlite::params![
                embedding_unit_key,
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
        "INSERT INTO reference_edges (from_record_key, to_record_key, display_text, reference_text, source_kind, visibility)
             VALUES ('actions:testAction1', 'actions:testAction2', 'Test Action 2', 'Compendium.pf2e.actions.Item.testAction2', 'description', 'public')",
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

    let record_set = AtlasIndex::open_read_only(&path)?.load_record_set()?;

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
fn rejects_empty_boolean_groups_at_compile_boundary() {
    assert_eq!(
        compile_eligible_records_query(Some(&atlas_domain::SearchFilterNode::any_of(Vec::new())))
            .unwrap_err(),
        FilterCompileError::InvalidValue(
            "filter `any_of` must contain at least one child".to_string()
        )
    );
    assert_eq!(
        compile_eligible_records_query(Some(&atlas_domain::SearchFilterNode::all_of(Vec::new())))
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
    let compiled = compile_eligible_records_query(Some(&filter))?;
    let keys = query_eligible_keys(&connection, &compiled)?;

    assert_eq!(keys, vec!["actions:testAction1"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reference_edges_for_seed_returns_policy_visible_outgoing_edges()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-outgoing");
    create_contract_database(&path)?;
    insert_reference_edge(
        &path,
        "actions:testAction1",
        "actions:testAction2",
        Some("Visible"),
        "visible-ref",
        "description",
        "public",
    )?;
    insert_reference_edge(
        &path,
        "actions:testAction1",
        "actions:testAction3",
        Some("Private"),
        "private-ref",
        "private_notes",
        "private",
    )?;
    insert_reference_edge(
        &path,
        "actions:testAction1",
        "actions:testAction3",
        Some("Embedded"),
        "embedded-ref",
        "embedded_item_description",
        "public",
    )?;

    let index = AtlasIndex::open_read_only(&path)?;
    let edges = index.reference_edges_for_seed(
        &RecordKey::parse("actions:testAction1")?,
        ReferenceEdgeDirection::Outgoing,
    )?;

    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].to_record_key,
        RecordKey::parse("actions:testAction2")?
    );
    assert_eq!(edges[0].display_text.as_deref(), Some("Visible"));
    assert_eq!(edges[0].reference_text, "visible-ref");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reference_edges_for_seed_returns_policy_visible_backlinks()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-backlinks");
    create_contract_database(&path)?;
    insert_reference_edge(
        &path,
        "actions:testAction2",
        "actions:testAction1",
        Some("Incoming"),
        "incoming-ref",
        "description",
        "public",
    )?;
    insert_reference_edge(
        &path,
        "actions:testAction3",
        "actions:testAction1",
        Some("Private"),
        "private-ref",
        "private_notes",
        "private",
    )?;

    let index = AtlasIndex::open_read_only(&path)?;
    let edges = index.reference_edges_for_seed(
        &RecordKey::parse("actions:testAction1")?,
        ReferenceEdgeDirection::Backlink,
    )?;

    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].from_record_key,
        RecordKey::parse("actions:testAction2")?
    );
    assert_eq!(edges[0].reference_text, "incoming-ref");
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

#[test]
fn composes_vector_knn_query_from_eligible_records() -> Result<(), Box<dyn std::error::Error>> {
    let filter = atlas_domain::SearchFilterNode::all_of(vec![
        atlas_domain::SearchFilterNode::record_family(RecordFamily::Rule),
        atlas_domain::SearchFilterNode::pack("actions"),
    ]);

    let compiled = compile_vector_knn_query(&[0.25, 0.5, 0.75], Some(&filter), 12, true)?;

    assert!(compiled.sql.contains("WITH eligible(record_key) AS"));
    assert!(compiled.sql.contains("FROM record_vector_index v"));
    assert!(
        compiled
            .sql
            .contains("JOIN document_embedding_cache e ON e.rowid = v.rowid")
    );
    assert!(compiled.sql.contains("v.embedding MATCH ?3"));
    assert!(compiled.sql.contains("AND k = ?4"));
    assert!(
        compiled
            .sql
            .contains("v.rowid IN (\n             SELECT candidate.rowid")
    );
    assert_eq!(compiled.parameters.len(), 4);
    assert_eq!(
        compiled.parameters[0],
        rusqlite::types::Value::Text("rule".to_string())
    );
    assert_eq!(
        compiled.parameters[1],
        rusqlite::types::Value::Text("actions".to_string())
    );
    assert_eq!(
        compiled.parameters[2],
        rusqlite::types::Value::Blob(vec![0, 0, 128, 62, 0, 0, 0, 63, 0, 0, 64, 63])
    );
    assert_eq!(compiled.parameters[3], rusqlite::types::Value::Integer(12));
    Ok(())
}

#[test]
fn parent_only_vector_knn_query_restricts_candidate_units() -> Result<(), Box<dyn std::error::Error>>
{
    let compiled = compile_vector_knn_query(&[1.0], None, 5, false)?;

    assert!(compiled.sql.contains("candidate.unit_kind = 'parent'"));
    Ok(())
}

#[test]
fn rejects_invalid_vector_knn_queries() {
    let unsupported = atlas_domain::SearchFilterNode::metadata(MetadataPredicate::Set {
        field: MetadataSetField::DerivedTags,
        r#match: MetadataSetMatch::Includes {
            value: "area-damage".to_string(),
        },
    });

    assert_eq!(
        compile_vector_knn_query(&[1.0], None, 0, true).unwrap_err(),
        VectorQueryError::InvalidLimit
    );
    assert_eq!(
        compile_vector_knn_query(&[], None, 10, true).unwrap_err(),
        VectorQueryError::EmptyQueryVector
    );
    assert!(matches!(
        compile_vector_knn_query(&[1.0], Some(&unsupported), 10, true).unwrap_err(),
        VectorQueryError::Filter(FilterCompileError::Unsupported { .. })
    ));
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

fn insert_reference_edge(
    path: &PathBuf,
    from_record_key: &str,
    to_record_key: &str,
    display_text: Option<&str>,
    reference_text: &str,
    source_kind: &str,
    visibility: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "INSERT INTO reference_edges (
           from_record_key, to_record_key, display_text, reference_text, source_kind, visibility
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            from_record_key,
            to_record_key,
            display_text,
            reference_text,
            source_kind,
            visibility,
        ),
    )?;
    Ok(())
}

fn create_contract_database_without(
    path: &PathBuf,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata_omitting(&connection, valid_metadata_entries(), omitted_key)?;
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
    insert_contract_metadata_entries(connection, valid_metadata_entries(), override_entry)
}

fn insert_contract_metadata_entries(
    connection: &Connection,
    entries: Vec<(&'static str, &'static str)>,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    atlas_artifact::test_support::insert_contract_metadata_entries(
        connection,
        entries,
        override_entry,
    )
}

fn valid_metadata_entries() -> Vec<(&'static str, &'static str)> {
    valid_metadata_entries_for_embedding(default_embedding_model_spec())
}

fn valid_metadata_entries_for_embedding(
    embedding_spec: atlas_embedding::EmbeddingModelSpec,
) -> Vec<(&'static str, &'static str)> {
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
            artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION,
            EMBEDDING_UNIT_POLICY_VERSION,
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
