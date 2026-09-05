use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use atlas_index::SqliteIndexReader;
use atlas_index::test_support::{
    create_minimal_artifact_schema, insert_artifact_metadata_entries, insert_minimal_artifact_rows,
    insert_minimal_canonical_npc_projection, legacy_minilm_metadata_entries,
    refresh_fixture_metric_summary, write_bound_test_manifest,
};
use rusqlite::Connection;

use crate::AtlasRetrievalService;

static FIXTURE_ARTIFACT_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct FixtureArtifact {
    path: PathBuf,
}

impl FixtureArtifact {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for FixtureArtifact {
    fn drop(&mut self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}

pub fn minimal_fixture_retrieval_service_without_embeddings()
-> Result<(AtlasRetrievalService, FixtureArtifact), Box<dyn std::error::Error>> {
    let artifact = FixtureArtifact {
        path: fixture_artifact_path(),
    };
    let connection = Connection::open(&artifact.path)?;
    create_minimal_artifact_schema(&connection)?;
    insert_artifact_metadata_entries(&connection, legacy_minilm_metadata_entries(), None)?;
    insert_minimal_artifact_rows(&connection)?;
    drop(connection);
    write_bound_test_manifest(&artifact.path)?;

    let reader = SqliteIndexReader::open_read_only(&artifact.path)?;
    Ok((
        AtlasRetrievalService::from_prepared_index_without_embeddings(reader),
        artifact,
    ))
}

pub fn encounter_fixture_retrieval_service_without_embeddings()
-> Result<(AtlasRetrievalService, FixtureArtifact), Box<dyn std::error::Error>> {
    let artifact = FixtureArtifact {
        path: fixture_artifact_path(),
    };
    let connection = Connection::open(&artifact.path)?;
    create_minimal_artifact_schema(&connection)?;
    insert_artifact_metadata_entries(&connection, legacy_minilm_metadata_entries(), None)?;
    insert_minimal_artifact_rows(&connection)?;
    insert_encounter_fixture_rows(&connection)?;
    drop(connection);
    write_bound_test_manifest(&artifact.path)?;

    let reader = SqliteIndexReader::open_read_only(&artifact.path)?;
    Ok((
        AtlasRetrievalService::from_prepared_index_without_embeddings(reader),
        artifact,
    ))
}

fn fixture_artifact_path() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "atlas-search-fixture-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    if root.exists() {
        let _ = fs::remove_dir_all(&root);
    }
    fs::create_dir_all(&root).expect("fixture artifact directory");
    root.join("pf2e-index.sqlite")
}

fn unique_suffix() -> u64 {
    FIXTURE_ARTIFACT_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn insert_encounter_fixture_rows(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute(
        "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
             VALUES ('actors', 'Actors', 'Actor', 'packs/actors', 'packs/actors', 1)",
        [],
    )?;
    connection.execute(
        "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
             VALUES ('hazards', 'Hazards', 'Actor', 'packs/hazards', 'packs/hazards', 1)",
        [],
    )?;
    connection.execute(
        "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
             VALUES ('conditionitems', 'Conditions', 'Item', 'packs/conditionitems', 'packs/conditionitems', 1)",
        [],
    )?;
    insert_record(
        connection,
        FixtureRecord {
            record_key: "actors:testCreature",
            id: "testCreature",
            name: "Test Creature",
            record_kind: "creature",
            pack_name: "actors",
            pack_label: "Actors",
            document_type: "Actor",
            foundry_record_type: "npc",
        },
    )?;
    insert_record(
        connection,
        FixtureRecord {
            record_key: "hazards:testHazard",
            id: "testHazard",
            name: "Test Hazard",
            record_kind: "hazard",
            pack_name: "hazards",
            pack_label: "Hazards",
            document_type: "Actor",
            foundry_record_type: "hazard",
        },
    )?;
    insert_record(
        connection,
        FixtureRecord {
            record_key: "conditionitems:testCondition",
            id: "testCondition",
            name: "Test Condition",
            record_kind: "rule",
            pack_name: "conditionitems",
            pack_label: "Conditions",
            document_type: "Item",
            foundry_record_type: "condition",
        },
    )?;
    insert_minimal_canonical_npc_projection(connection, "actors:testCreature", 19, 17, 25, 9)?;
    connection.execute(
        "UPDATE records SET level = 5 WHERE record_key = 'actors:testCreature'",
        [],
    )?;
    insert_number_metric(connection, "hazards:testHazard", "hp.max", 30.0)?;
    refresh_fixture_metric_summary(connection, "hazards:testHazard")?;
    Ok(())
}

struct FixtureRecord<'a> {
    record_key: &'a str,
    id: &'a str,
    name: &'a str,
    record_kind: &'a str,
    pack_name: &'a str,
    pack_label: &'a str,
    document_type: &'a str,
    foundry_record_type: &'a str,
}

fn insert_record(
    connection: &Connection,
    record: FixtureRecord<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let normalized_name = record.name.to_lowercase();
    let source_path = format!("packs/{}/{}.json", record.pack_name, record.id);
    connection.execute(
        "INSERT INTO records (
              record_key, id, name, normalized_name, record_kind, pack_name, pack_label,
              foundry_document_type, foundry_record_type, traits_json, prerequisites_json,
              publication_remaster, publication_family, taxonomy_families_json,
              variant_axes_json, variant_source, source_path, is_default_visible, raw_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
              '[]', '[]', 0, 'unknown', '[]', '[]', 'none', ?10, 1, '{}')",
        (
            record.record_key,
            record.id,
            record.name,
            normalized_name.as_str(),
            record.record_kind,
            record.pack_name,
            record.pack_label,
            record.document_type,
            record.foundry_record_type,
            source_path.as_str(),
        ),
    )?;
    connection.execute(
        "INSERT INTO records_fts (
             record_key, title, aliases, traits, taxonomy_terms, constraint_terms, mechanic_terms,
              source_terms, metric_terms, headings, body, facts, reference_terms, embedded_content
             ) VALUES (?1, ?2, '', '', '', '', '', '', '', '', ?2, '', '', '')",
        (record.record_key, record.name),
    )?;
    Ok(())
}

fn insert_number_metric(
    connection: &Connection,
    record_key: &str,
    metric_key: &str,
    value: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute(
        "INSERT INTO record_metrics (
             record_key, metric_domain, metric_key, value_type, number_value
         ) VALUES (?1, 'actor', ?2, 'number', ?3)",
        (record_key, metric_key, value),
    )?;
    Ok(())
}
