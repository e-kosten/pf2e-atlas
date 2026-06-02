use std::fs;
use std::path::PathBuf;

use crate::artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, artifact_metadata_keys,
};
use crate::artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;
use atlas_embedding::{EMBEDDING_UNIT_POLICY_VERSION, default_embedding_model_spec};
use rusqlite::Connection;

mod embedding;
mod filters;
mod fts;
mod graph_product;
mod records;
mod schema_freshness;
mod validation;
mod vector;
mod vector_query;

fn create_valid_artifact_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_artifact_schema(&connection)?;
    insert_artifact_metadata(&connection, None)?;
    insert_minimal_artifact_rows(&connection)?;
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

fn create_valid_artifact_database_without(
    path: &PathBuf,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_artifact_schema(&connection)?;
    insert_artifact_metadata_omitting(&connection, valid_metadata_entries(), omitted_key)?;
    Ok(())
}

fn create_valid_artifact_database_with_override(
    path: &PathBuf,
    override_key: &str,
    override_value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_artifact_schema(&connection)?;
    insert_artifact_metadata(&connection, Some((override_key, override_value)))?;
    Ok(())
}

fn insert_artifact_metadata(
    connection: &Connection,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    insert_artifact_metadata_entries(connection, valid_metadata_entries(), override_entry)
}

fn insert_artifact_metadata_entries(
    connection: &Connection,
    entries: Vec<(&'static str, &'static str)>,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (key, mut value) in entries {
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

fn create_minimal_artifact_schema(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute_batch(CREATE_ARTIFACT_SCHEMA_SQL)?;
    Ok(())
}

fn insert_artifact_metadata_omitting(
    connection: &Connection,
    entries: Vec<(&'static str, &'static str)>,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for (key, value) in entries {
        if key != omitted_key {
            connection.execute(
                "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
                [key, value],
            )?;
        }
    }
    Ok(())
}

fn insert_minimal_artifact_rows(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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
              record_key, id, name, normalized_name, record_kind, pack_name, pack_label,
              foundry_document_type, foundry_record_type, traits_json, prerequisites_json, publication_remaster,
              publication_family, taxonomy_families_json, variant_axes_json, variant_source,
              source_path, is_default_visible, raw_json
            ) VALUES (?1, ?2, ?3, ?4, 'rule', 'actions', 'Actions', 'Item', 'action',
              '[]', '[]', 0, 'unknown', '[]', '[]', 'none', ?5, 1, '{}')",
            [
                record_key.as_str(),
                record_id.as_str(),
                name.as_str(),
                normalized_name.as_str(),
                source_path.as_str(),
            ],
        )?;
        connection.execute(
            "INSERT INTO records_fts (
              record_key, title, aliases, traits, taxonomy_terms, constraint_terms, mechanic_terms,
              source_terms, metric_terms, headings, body, facts, reference_terms, embedded_content
             ) VALUES (?1, ?2, '', '', '', '', '', '', '', '', ?2, '', '', '')",
            [record_key.as_str(), name.as_str()],
        )?;
    }
    insert_minimal_filter_discovery_rows(connection)?;
    Ok(())
}

fn insert_minimal_filter_discovery_rows(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    const CANONICAL_KINDS_JSON: &str = r#"["creature","character","companion","army","hazard","vehicle","equipment","feat","spell","affliction","rule","character_option","lore","tooling","campaign_feature"]"#;
    let fields = [
        ("record_kind", r#"["--kind"]"#, CANONICAL_KINDS_JSON),
        ("pack_name", r#"["--pack-name"]"#, CANONICAL_KINDS_JSON),
        ("pack_label", r#"["--pack-label"]"#, CANONICAL_KINDS_JSON),
        ("foundry_record_type", r#"[]"#, CANONICAL_KINDS_JSON),
        ("publication_family", r#"[]"#, CANONICAL_KINDS_JSON),
    ];
    for kind in [None, Some("rule")] {
        for (field, cli_flags, applicable_kinds) in fields {
            connection.execute(
                "INSERT INTO filter_field_catalog (
                   field, record_kind, field_type, field_group, value_policy,
                   operators_json, cli_flags_json, applicable_kinds_json,
                   value_count, matching_record_count, null_count, distinct_count,
                   singleton_count, singleton_ratio, observation_singleton_ratio, policy_reason
                 ) VALUES (
                   ?1, ?2, 'enum_string', 'record', 'enumerable',
                   '[\"eq\",\"not_eq\",\"is_null\",\"is_not_null\"]', ?3, ?4,
                   3, 3, 0, 1, 0, 0.0, 0.0, 'Enumerable'
                 )",
                (field, kind, cli_flags, applicable_kinds),
            )?;
        }
    }
    for kind in [None, Some("rule")] {
        connection.execute(
            "INSERT INTO filter_field_catalog (
               field, record_kind, field_type, field_group, value_policy,
               operators_json, cli_flags_json, applicable_kinds_json,
               value_count, matching_record_count, null_count, distinct_count,
               singleton_count, singleton_ratio, observation_singleton_ratio, policy_reason
             ) VALUES (
               'publication_remaster', ?1, 'boolean', 'record', 'boolean_counts',
               '[\"eq\",\"is_null\",\"is_not_null\"]', '[]', ?2,
               3, 3, 0, 1, 0, 0.0, 0.0, 'BooleanCounts'
             )",
            (kind, CANONICAL_KINDS_JSON),
        )?;
    }
    for kind in [None, Some("rule")] {
        for (field, value) in [
            ("record_kind", "rule"),
            ("pack_name", "actions"),
            ("pack_label", "Actions"),
            ("foundry_record_type", "action"),
            ("publication_family", "unknown"),
        ] {
            connection.execute(
                "INSERT INTO filter_value_catalog (field, record_kind, value, catalog_count)
                 VALUES (?1, ?2, ?3, 3)",
                (field, kind, value),
            )?;
        }
    }
    Ok(())
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
