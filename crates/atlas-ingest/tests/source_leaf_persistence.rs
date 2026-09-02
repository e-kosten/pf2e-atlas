use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use atlas_domain::RecordKey;
use atlas_index::SqliteIndexReader;
use atlas_ingest::{
    BuildArtifactOptions, SourceIdentity, SourcePresence, build_artifact, parse_npc_source,
    pinned_source_version_metadata,
};
use atlas_record::{FactValue, RecordBody};
use rusqlite::Connection;

const SOURCE_ROOT: &str = "tests/fixtures/source-leaf-coverage/actor-npc/persistence";
const RECORD_KEY: &str = "pathfinder-bestiary:WQy7HBUcgDLsfVJd";
const SOURCE_PATH: &str = "packs/pathfinder-bestiary/night-hag.json";

struct TemporaryOutput(PathBuf);

impl TemporaryOutput {
    fn new() -> Result<Self, std::io::Error> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "pf2e-atlas-a2-creature-persistence-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root)?;
        Ok(Self(root))
    }

    fn artifact(&self) -> PathBuf {
        self.0.join("index.sqlite")
    }
}

impl Drop for TemporaryOutput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn single_actor_npc_no_embedding_persistence_proves_current_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(SOURCE_ROOT);
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(source_root.join(SOURCE_PATH))?)?;

    let dto = parse_npc_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(RECORD_KEY, SOURCE_PATH),
        raw,
    )?;
    let SourcePresence::Value(abilities) = dto.source.core.abilities else {
        panic!("Night Hag has an authored abilities map");
    };
    for parsed_value in [
        abilities.strength,
        abilities.dexterity,
        abilities.constitution,
        abilities.intelligence,
        abilities.wisdom,
        abilities.charisma,
    ] {
        let SourcePresence::Value(parsed) = parsed_value else {
            panic!("each authored ability object must parse");
        };
        assert!(
            matches!(parsed.value, SourcePresence::Missing),
            "current parser incorrectly looks for .value and must not pass authored .mod"
        );
    }

    let temporary = TemporaryOutput::new()?;
    let output_path = temporary.artifact();
    let report = build_artifact(BuildArtifactOptions {
        source_root,
        output_path: output_path.clone(),
        manifest_path: None,
        embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
        embedding_cache_root: None,
        reuse_embeddings: true,
        embedding_batch_size: 8,
    })?;
    assert_eq!(report.source_record_count, 1);
    assert_eq!(report.artifact_record_count, 1);
    assert_eq!(report.document_embedding_count, 0);
    assert_eq!(report.generated_document_embedding_count, 0);
    assert_eq!(report.reused_document_embedding_count, 0);
    assert_eq!(report.embedding_timing.model_load_duration_ms, 0);
    assert_eq!(report.embedding_timing.generation_duration_ms, 0);
    assert_eq!(report.embedding_timing.batch_count, 0);

    let reader = SqliteIndexReader::open_read_only(&output_path)?;
    let key = RecordKey::parse(RECORD_KEY)?;
    let hydrated = reader.load_hydrated_records_by_key(&[key])?;
    assert_eq!(hydrated.len(), 1);
    let Some(RecordBody::Creature(creature)) = hydrated[0].body.as_ref() else {
        panic!("persisted Night Hag must hydrate as a creature");
    };
    let FactValue::Value(abilities) = &creature.legacy_abilities.value else {
        panic!("current canonical projection retains the ability container");
    };
    for persisted_leaf in [
        &abilities.strength,
        &abilities.dexterity,
        &abilities.constitution,
        &abilities.intelligence,
        &abilities.wisdom,
        &abilities.charisma,
    ] {
        assert!(
            matches!(persisted_leaf, FactValue::Missing),
            "a transient or container-level metric cannot satisfy the canonical persisted leaf owner"
        );
    }
    drop(reader);

    let connection = Connection::open(&output_path)?;
    let ability_metric_count: usize = connection.query_row(
        "SELECT COUNT(*) FROM record_metrics WHERE record_key = ?1 AND metric_key LIKE 'ability.%.mod'",
        [RECORD_KEY],
        |row| row.get(0),
    )?;
    assert_eq!(
        ability_metric_count, 0,
        "missing .value inputs must not masquerade as persisted authored .mod metrics"
    );
    Ok(())
}
