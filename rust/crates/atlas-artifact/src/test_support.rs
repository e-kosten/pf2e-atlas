use rusqlite::Connection;

use crate::metadata::{ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, artifact_metadata_keys};
use crate::schema::create_artifact_schema_sql;

pub fn create_minimal_contract_schema(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute_batch(&create_artifact_schema_sql())?;
    Ok(())
}

pub fn insert_contract_metadata_entries(
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

pub fn insert_contract_metadata_omitting(
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

pub fn insert_minimal_contract_rows(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
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
              source_path, is_default_visible, raw_json
            ) VALUES (?1, ?2, ?3, ?4, 'rule', 'actions', 'Actions', 'Item', 'action',
              '[]', 0, 'unknown', '[]', '[]', 'none', ?5, 1, '{}')",
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
              record_key, title, aliases, traits, headings, body, facts, reference_terms, embedded_content
             ) VALUES (?1, ?2, '', '', '', ?2, '', '', '')",
            [record_key.as_str(), name.as_str()],
        )?;
    }
    Ok(())
}

pub fn legacy_minilm_metadata_entries() -> Vec<(&'static str, &'static str)> {
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
            "transformers-js-minilm",
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            "Xenova/all-MiniLM-L12-v2",
        ),
        (artifact_metadata_keys::EMBEDDING_MODEL_REVISION, "main"),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            "Xenova/all-MiniLM-L12-v2",
        ),
        (artifact_metadata_keys::EMBEDDING_POOLING, "mean"),
        (artifact_metadata_keys::EMBEDDING_NORMALIZATION, "l2"),
        (artifact_metadata_keys::EMBEDDING_DIMENSIONS, "384"),
        (artifact_metadata_keys::EMBEDDING_DTYPE, "f32"),
        (artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC, "cosine"),
        (artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX, ""),
        (artifact_metadata_keys::EMBEDDING_QUERY_PREFIX, ""),
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
