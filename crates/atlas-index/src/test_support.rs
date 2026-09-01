use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::{AtlasRecord, ProductRetrievalPolicy, RemasterLink};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::SqliteIndexReader;
use crate::artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, artifact_metadata_keys,
};
use crate::artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;
use crate::schema;

/// Inserts a minimal canonical NPC body for cross-crate retrieval fixtures.
pub fn insert_minimal_canonical_npc_body(
    connection: &Connection,
    record_key: &str,
    ac: i64,
    hp_value: i64,
    hp_maximum: i64,
    perception: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    use atlas_record::{CreatureFact, CreatureSourceField, FactValue};

    macro_rules! missing {
        ($field:expr) => {
            CreatureFact::source(FactValue::Missing, $field)
        };
    }
    let key = RecordKey::parse(record_key)?;
    let source_id = key.id().as_str().to_string();
    let name = connection.query_row(
        "SELECT name FROM records WHERE record_key = ?1",
        [record_key],
        |row| row.get::<_, String>(0),
    )?;
    let creature = atlas_record::CreatureRecord {
        identity: atlas_record::CreatureIdentity {
            record_key: key,
            source_id: atlas_record::CreatureSourceId::new(&source_id)
                .map_err(|_| "fixture source id is invalid")?,
            name: name.clone(),
            family: atlas_record::CreatureFamily::Npc,
        },
        level: missing!(CreatureSourceField::Level),
        rarity: missing!(CreatureSourceField::Rarity),
        traits: missing!(CreatureSourceField::Traits),
        size: missing!(CreatureSourceField::Size),
        publication: missing!(CreatureSourceField::Publication),
        adjustment: missing!(CreatureSourceField::Adjustment),
        source_alliance: missing!(CreatureSourceField::SourceAlliance),
        perception: CreatureFact::source(
            FactValue::Value(atlas_record::CreaturePerception {
                modifier: FactValue::Value(perception),
                details: FactValue::Missing,
                has_vision: FactValue::Missing,
                senses: FactValue::Value(Vec::new()),
            }),
            CreatureSourceField::Perception,
        ),
        initiative: missing!(CreatureSourceField::Initiative),
        languages: missing!(CreatureSourceField::Languages),
        skills: missing!(CreatureSourceField::Skills),
        legacy_abilities: missing!(CreatureSourceField::LegacyAbilities),
        defenses: CreatureFact::source(
            FactValue::Value(atlas_record::CreatureDefenses {
                armor_class: FactValue::Value(atlas_record::CreatureArmorClass {
                    value: FactValue::Value(ac),
                    details: FactValue::Missing,
                }),
                hit_points: FactValue::Value(atlas_record::CreatureHitPoints {
                    value: FactValue::Value(atlas_record::CreatureNumber::Integer(hp_value)),
                    maximum: FactValue::Value(hp_maximum),
                    temporary: FactValue::Missing,
                    temporary_maximum: FactValue::Missing,
                    details: FactValue::Missing,
                }),
                hardness: FactValue::Missing,
                shield: FactValue::Missing,
                saves: FactValue::Missing,
                all_saves_note: FactValue::Missing,
                immunities: FactValue::Missing,
                resistances: FactValue::Missing,
                weaknesses: FactValue::Missing,
            }),
            CreatureSourceField::Defenses,
        ),
        movement: missing!(CreatureSourceField::Movement),
        resources: missing!(CreatureSourceField::Resources),
        embedded_entities: missing!(CreatureSourceField::EmbeddedEntities),
        content: atlas_record::OwnedRichContent::default(),
        provenance: atlas_record::CreatureProvenance {
            source_path: format!("packs/actors/{source_id}.json"),
            source_contract_version: "fixture".to_string(),
            source_system_version: "fixture".to_string(),
            source_upstream_commit: "fixture".to_string(),
        },
    };
    let metrics = atlas_record::project_creature_facts(&creature).metrics;
    let metric_count = i64::try_from(metrics.len())?;
    let metric_digest = crate::read::records::children::metric_order_digest(&metrics)
        .map_err(|error| format!("fixture metric digest failed: {error}"))?;
    connection.execute(
        "UPDATE records
         SET metric_count = ?2, metric_order_sha256 = ?3
         WHERE record_key = ?1",
        (record_key, metric_count, metric_digest),
    )?;
    let body = atlas_record::RecordBody::Creature(creature);
    let canonical_json = crate::artifact::canonical_json::encode(&body)?;
    connection.execute(
        "INSERT INTO canonical_creature_records
         (record_key, source_id, name, family, canonical_json)
         VALUES (?1, ?2, ?3, 'npc', ?4)",
        (record_key, source_id, name, canonical_json),
    )?;
    Ok(())
}

/// Recomputes the writer-bound metric summary after a cross-crate fixture has
/// inserted its ordered metric rows.
pub fn refresh_fixture_metric_summary(
    connection: &Connection,
    record_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut statement = connection.prepare(
        "SELECT metric_domain,metric_key,value_type,number_value,text_value,bool_value
         FROM record_metrics WHERE record_key = ?1 ORDER BY ordinal",
    )?;
    let rows = statement.query_map([record_key], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<f64>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<i64>>(5)?,
        ))
    })?;
    let mut metrics = Vec::new();
    for row in rows {
        let (domain, key, value_type, number, text, boolean) = row?;
        metrics.push(
            crate::read::records::children::metric_from_storage(
                &domain,
                key,
                &value_type,
                number,
                text,
                boolean.map(|value| value != 0),
            )
            .map_err(|error| format!("fixture metric decode failed: {error}"))?,
        );
    }
    let metric_count = i64::try_from(metrics.len())?;
    let metric_digest = crate::read::records::children::metric_order_digest(&metrics)
        .map_err(|error| format!("fixture metric digest failed: {error}"))?;
    connection.execute(
        "UPDATE records
         SET metric_count = ?2, metric_order_sha256 = ?3
         WHERE record_key = ?1",
        (record_key, metric_count, metric_digest),
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordRoundTripRecordRole {
    Source,
    SourceInstance,
    Canonical,
}

impl RecordRoundTripRecordRole {
    fn parse(record_key: &RecordKey, value: &str) -> Result<Self, RecordRoundTripDiagnosticError> {
        match value {
            "source" => Ok(Self::Source),
            "source_instance" => Ok(Self::SourceInstance),
            "canonical" => Ok(Self::Canonical),
            _ => Err(RecordRoundTripDiagnosticError::UnknownRecordRole {
                record_key: record_key.clone(),
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordRoundTripRetrievalDisposition {
    Ordinary,
    DirectOnly,
    InspectionOnly,
}

impl RecordRoundTripRetrievalDisposition {
    fn parse(record_key: &RecordKey, value: &str) -> Result<Self, RecordRoundTripDiagnosticError> {
        match value {
            "ordinary" => Ok(Self::Ordinary),
            "direct_only" => Ok(Self::DirectOnly),
            "inspection_only" => Ok(Self::InspectionOnly),
            _ => Err(
                RecordRoundTripDiagnosticError::UnknownRetrievalDisposition {
                    record_key: record_key.clone(),
                    value: value.to_string(),
                },
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordRoundTripRetrievalRationale {
    ToolingNoAddressableProductMeaning,
    CanonicalEditionDuplicate,
    DuplicateSourceInstance,
    GeneratedCanonical,
    SourceRecord,
}

impl RecordRoundTripRetrievalRationale {
    fn parse(record_key: &RecordKey, value: &str) -> Result<Self, RecordRoundTripDiagnosticError> {
        match value {
            "tooling_no_addressable_product_meaning" => {
                Ok(Self::ToolingNoAddressableProductMeaning)
            }
            "canonical_edition_duplicate" => Ok(Self::CanonicalEditionDuplicate),
            "duplicate_source_instance" => Ok(Self::DuplicateSourceInstance),
            "generated_canonical" => Ok(Self::GeneratedCanonical),
            "source_record" => Ok(Self::SourceRecord),
            _ => Err(RecordRoundTripDiagnosticError::UnknownRetrievalRationale {
                record_key: record_key.clone(),
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordRoundTripPersistedProjectionRow {
    pub record_key: RecordKey,
    pub record_role: RecordRoundTripRecordRole,
    pub retrieval_disposition: RecordRoundTripRetrievalDisposition,
    pub retrieval_rationale: RecordRoundTripRetrievalRationale,
    pub is_default_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecordRoundTripDiagnosticError {
    #[error("persisted retrieval projection query failed: {message}")]
    PersistedProjectionQuery { message: String },
    #[error("invalid persisted record key `{value}`: {message}")]
    InvalidRecordKey { value: String, message: String },
    #[error("record `{record_key}` has unknown record role `{value}`")]
    UnknownRecordRole {
        record_key: RecordKey,
        value: String,
    },
    #[error("record `{record_key}` has unknown retrieval disposition `{value}`")]
    UnknownRetrievalDisposition {
        record_key: RecordKey,
        value: String,
    },
    #[error("record `{record_key}` has unknown retrieval rationale `{value}`")]
    UnknownRetrievalRationale {
        record_key: RecordKey,
        value: String,
    },
    #[error("record-round-trip projection contains duplicate key `{record_key}`")]
    DuplicateRecordKey { record_key: RecordKey },
}

pub fn record_round_trip_persisted_retrieval_projection(
    reader: &SqliteIndexReader,
) -> Result<
    BTreeMap<RecordKey, RecordRoundTripPersistedProjectionRow>,
    RecordRoundTripDiagnosticError,
> {
    let rows = reader.with_diesel_connection(|connection| {
        schema::records::table
            .select((
                schema::records::record_key,
                schema::records::record_role,
                schema::records::retrieval_disposition,
                schema::records::retrieval_rationale,
                schema::records::is_default_visible,
            ))
            .order(schema::records::record_key.asc())
            .load::<(String, String, String, String, bool)>(connection)
    });
    let rows = rows.map_err(
        |error| RecordRoundTripDiagnosticError::PersistedProjectionQuery {
            message: error.to_string(),
        },
    )?;

    let mut projection = BTreeMap::new();
    for (record_key, record_role, disposition, rationale, is_default_visible) in rows {
        let record_key_value = record_key;
        let record_key = RecordKey::parse(&record_key_value).map_err(|error| {
            RecordRoundTripDiagnosticError::InvalidRecordKey {
                value: record_key_value.clone(),
                message: error.to_string(),
            }
        })?;
        let row = record_round_trip_projection_row(
            record_key.clone(),
            &record_role,
            &disposition,
            &rationale,
            is_default_visible,
        )?;
        if projection.insert(record_key.clone(), row).is_some() {
            return Err(RecordRoundTripDiagnosticError::DuplicateRecordKey { record_key });
        }
    }
    Ok(projection)
}

pub fn record_round_trip_expected_retrieval_projection(
    records: &[AtlasRecord],
    remaster_links: &[RemasterLink],
) -> Result<
    BTreeMap<RecordKey, RecordRoundTripPersistedProjectionRow>,
    RecordRoundTripDiagnosticError,
> {
    let policy = ProductRetrievalPolicy::from_remaster_links(remaster_links);
    let mut projection = BTreeMap::new();
    for record in records {
        let record_key = record.identity.key.clone();
        let decision = policy.decision(record);
        let record_role = decision.role.as_str();
        let disposition = decision.disposition.as_str();
        let rationale = decision.rationale.as_str();
        let row = record_round_trip_projection_row(
            record_key.clone(),
            record_role,
            disposition,
            rationale,
            disposition == "ordinary",
        )?;
        if projection.insert(record_key.clone(), row).is_some() {
            return Err(RecordRoundTripDiagnosticError::DuplicateRecordKey { record_key });
        }
    }
    Ok(projection)
}

fn record_round_trip_projection_row(
    record_key: RecordKey,
    record_role: &str,
    retrieval_disposition: &str,
    retrieval_rationale: &str,
    is_default_visible: bool,
) -> Result<RecordRoundTripPersistedProjectionRow, RecordRoundTripDiagnosticError> {
    Ok(RecordRoundTripPersistedProjectionRow {
        record_role: RecordRoundTripRecordRole::parse(&record_key, record_role)?,
        retrieval_disposition: RecordRoundTripRetrievalDisposition::parse(
            &record_key,
            retrieval_disposition,
        )?,
        retrieval_rationale: RecordRoundTripRetrievalRationale::parse(
            &record_key,
            retrieval_rationale,
        )?,
        record_key,
        is_default_visible,
    })
}

pub fn create_record_vector_index_sql(dimensions: usize) -> String {
    crate::read::search::sqlite_vector_index::create_sql(dimensions)
}

pub fn insert_record_vector_index_sql() -> String {
    crate::read::search::sqlite_vector_index::insert_sql()
}

pub fn encode_f32_vector_blob(vector: &[f32]) -> Vec<u8> {
    crate::artifact::storage::encode_f32_vector_blob(vector)
}

pub fn write_bound_test_manifest(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let hash = format!("{:x}", Sha256::digest(std::fs::read(path)?));
    std::fs::write(
        path.parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("manifest.json"),
        format!(
            r#"{{"manifest_version":"{}","artifact_contract_version":"{}","schema_version":"{}","build":{{"artifact_sha256":"{hash}"}}}}"#,
            crate::ARTIFACT_MANIFEST_VERSION,
            crate::ARTIFACT_CONTRACT_VERSION,
            crate::ARTIFACT_SCHEMA_VERSION,
        ),
    )?;
    Ok(())
}

pub fn create_minimal_artifact_schema(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    // Cross-crate fixtures predate the persisted metric ordinal and insert one
    // metric per statement without naming it. Keep that convenience confined
    // to test support: production writers always enumerate the canonical
    // vector explicitly, while this trigger assigns the next fixture ordinal.
    let schema = CREATE_ARTIFACT_SCHEMA_SQL.replacen(
        "ordinal INTEGER NOT NULL DEFAULT 9223372036854775807 CHECK (ordinal >= 0)",
        "ordinal INTEGER NOT NULL DEFAULT -1 CHECK (ordinal >= -1)",
        1,
    );
    connection.execute_batch(&schema)?;
    connection.execute_batch(
        "CREATE TRIGGER test_assign_record_metric_ordinal
         AFTER INSERT ON record_metrics
         FOR EACH ROW WHEN NEW.ordinal = -1
         BEGIN
           UPDATE record_metrics
           SET ordinal = COALESCE((
             SELECT MAX(ordinal) + 1
             FROM record_metrics
             WHERE record_key = NEW.record_key AND ordinal >= 0
           ), 0)
           WHERE rowid = NEW.rowid;
         END;",
    )?;
    Ok(())
}

pub fn insert_artifact_metadata_entries(
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

pub fn insert_artifact_metadata_omitting(
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

pub fn insert_minimal_artifact_rows(
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
            artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION,
            atlas_embedding::EMBEDDING_UNIT_POLICY_VERSION,
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
