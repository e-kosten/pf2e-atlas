use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use atlas_index::SqliteIndexReader;
use atlas_index::test_support::{
    create_minimal_artifact_schema, insert_artifact_metadata_entries, insert_minimal_artifact_rows,
    legacy_minilm_metadata_entries,
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
        let _ = fs::remove_file(&self.path);
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

    let reader = SqliteIndexReader::open_read_only(&artifact.path)?;
    Ok((
        AtlasRetrievalService::from_prepared_index_without_embeddings(reader),
        artifact,
    ))
}

fn fixture_artifact_path() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "atlas-search-fixture-{}-{}.sqlite",
        std::process::id(),
        unique_suffix()
    ));
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    path
}

fn unique_suffix() -> u64 {
    FIXTURE_ARTIFACT_COUNTER.fetch_add(1, Ordering::Relaxed)
}
