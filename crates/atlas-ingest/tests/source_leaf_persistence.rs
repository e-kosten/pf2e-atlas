use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use atlas_domain::{DetailLevel, RecordKey};
use atlas_index::SqliteIndexReader;
use atlas_ingest::{
    BuildArtifactOptions, SourceIdentity, SourcePresence, build_artifact, parse_npc_source,
    pinned_source_version_metadata,
};
use atlas_record::{
    CreatureIntegerPresenceJson, CreatureSkillKind, CreatureUnmodeledSkillReason, FactValue,
    RecordBody, RecordJsonOptions, RecordPresentationJson, record_json,
};
use rusqlite::Connection;
use sha2::{Digest, Sha256};

const SOURCE_ROOT: &str = "tests/fixtures/source-leaf-coverage/actor-npc/persistence";
const RECORD_KEY: &str = "pathfinder-bestiary:WQy7HBUcgDLsfVJd";
const SOURCE_PATH: &str = "packs/pathfinder-bestiary/night-hag.json";
const GRAY_MASTER_KEY: &str = "curtain-call-bestiary:1eX4Csnv3psAsfLf";
const GRAY_MASTER_PATH: &str = "packs/curtain-call-bestiary/gray-master.json";
const SKILL_STATES_KEY: &str = "pathfinder-bestiary:b1SkillStates001";
const GRAY_MASTER_EXCERPT_SHA256: &str =
    "5c856483c6f3eec58ce754dadfd91d23f98df1b57ea631a63ad6dc98e7bc9b19";

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
fn single_b1_no_embedding_sqlite_build_proves_ability_and_skill_ownership()
-> Result<(), Box<dyn std::error::Error>> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(SOURCE_ROOT);
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(source_root.join(SOURCE_PATH))?)?;
    let gray_persistence_bytes = std::fs::read(source_root.join(GRAY_MASTER_PATH))?;
    let gray_excerpt_bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/source-leaf-coverage/actor-npc/excerpts/gray-master.json"),
    )?;
    assert_eq!(gray_persistence_bytes, gray_excerpt_bytes);
    let gray_excerpt: serde_json::Value = serde_json::from_slice(&gray_excerpt_bytes)?;
    assert_eq!(
        format!("{:x}", Sha256::digest(serde_json::to_vec(&gray_excerpt)?)),
        GRAY_MASTER_EXCERPT_SHA256
    );

    let dto = parse_npc_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(RECORD_KEY, SOURCE_PATH),
        raw,
    )?;
    let SourcePresence::Value(abilities) = dto.source.core.abilities else {
        panic!("Night Hag has an authored abilities map");
    };
    for (parsed_value, expected) in [
        abilities.strength,
        abilities.dexterity,
        abilities.constitution,
        abilities.intelligence,
        abilities.wisdom,
        abilities.charisma,
    ]
    .into_iter()
    .zip([5, 4, 6, 4, 5, 3])
    {
        let SourcePresence::Value(parsed) = parsed_value else {
            panic!("each authored ability object must parse");
        };
        assert_eq!(parsed.r#mod, SourcePresence::Value(expected));
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
    assert_eq!(report.source_record_count, 3);
    assert_eq!(report.artifact_record_count, 3);
    assert_eq!(report.document_embedding_count, 0);
    assert_eq!(report.generated_document_embedding_count, 0);
    assert_eq!(report.reused_document_embedding_count, 0);
    assert_eq!(report.embedding_timing.model_load_duration_ms, 0);
    assert_eq!(report.embedding_timing.generation_duration_ms, 0);
    assert_eq!(report.embedding_timing.batch_count, 0);

    let reader = SqliteIndexReader::open_read_only(&output_path)?;
    let hydrated = reader.load_hydrated_records_by_key(&[
        RecordKey::parse(RECORD_KEY)?,
        RecordKey::parse(GRAY_MASTER_KEY)?,
        RecordKey::parse(SKILL_STATES_KEY)?,
    ])?;
    assert_eq!(hydrated.len(), 3);
    let night_hag = hydrated
        .iter()
        .find(|record| record.record.identity.key.to_string() == RECORD_KEY)
        .expect("persisted Night Hag");
    let Some(RecordBody::Creature(creature)) = night_hag.body.as_ref() else {
        panic!("persisted Night Hag must hydrate as a creature");
    };
    let FactValue::Value(abilities) = &creature.legacy_abilities.value else {
        panic!("current canonical projection retains the ability container");
    };
    for (persisted_leaf, expected) in [
        &abilities.strength,
        &abilities.dexterity,
        &abilities.constitution,
        &abilities.intelligence,
        &abilities.wisdom,
        &abilities.charisma,
    ]
    .into_iter()
    .zip([5, 4, 6, 4, 5, 3])
    {
        assert_eq!(persisted_leaf, &FactValue::Value(expected));
    }

    let gray_master = hydrated
        .iter()
        .find(|record| record.record.identity.key.to_string() == GRAY_MASTER_KEY)
        .expect("persisted Gray Master");
    let Some(RecordBody::Creature(gray_creature)) = gray_master.body.as_ref() else {
        panic!("persisted Gray Master must hydrate as a creature");
    };
    let gray_intimidation = gray_creature
        .skills
        .value
        .as_value()
        .expect("Gray Master skills")
        .iter()
        .find(|skill| skill.kind == CreatureSkillKind::Intimidation)
        .expect("persisted Intimidation skill");
    assert_eq!(gray_intimidation.modifier, FactValue::Value(38));
    assert_eq!(
        gray_intimidation
            .source_entries
            .iter()
            .map(|entry| (entry.authored_key.as_str(), &entry.modifier))
            .collect::<Vec<_>>(),
        [
            ("intimidation", &FactValue::Value(38)),
            ("intimidate", &FactValue::Value(38)),
        ]
    );

    let skill_states = hydrated
        .iter()
        .find(|record| record.record.identity.key.to_string() == SKILL_STATES_KEY)
        .expect("persisted skill-state fixture");
    let Some(RecordBody::Creature(skill_state_creature)) = skill_states.body.as_ref() else {
        panic!("persisted skill-state fixture must hydrate as a creature");
    };
    let unmodeled = skill_state_creature
        .skills
        .value
        .as_value()
        .expect("unmodeled skill states");
    for (authored_key, expected) in [
        ("missing-base", FactValue::Missing),
        ("null-base", FactValue::Null),
        ("value-base", FactValue::Value(27)),
    ] {
        let skill = unmodeled
            .iter()
            .find(|skill| {
                skill.kind == CreatureSkillKind::Unmodeled
                    && skill
                        .unmodeled
                        .as_value()
                        .is_some_and(|fact| fact.authored_key == authored_key)
            })
            .unwrap_or_else(|| panic!("persisted unmodeled skill {authored_key}"));
        let fact = skill.unmodeled.as_value().expect("unmodeled fact");
        assert_eq!(fact.authored_key, authored_key);
        assert_eq!(fact.base, expected);
        assert_eq!(
            fact.reason,
            CreatureUnmodeledSkillReason::UnknownAuthoredKey
        );
        assert_eq!(skill.source_entries.len(), 1);
        assert_eq!(skill.source_entries[0].authored_key, authored_key);
        assert_eq!(skill.source_entries[0].modifier, expected);
    }

    let gray_json = record_json(
        gray_master,
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: false,
        },
    )?;
    let RecordPresentationJson::Creature {
        skills: Some(gray_skills),
        ..
    } = &gray_json.presentation
    else {
        panic!("Gray Master CLI creature projection must include skills");
    };
    let gray_cli_intimidation = gray_skills
        .iter()
        .find(|skill| skill.slug == "intimidation")
        .expect("CLI Intimidation skill");
    assert_eq!(gray_cli_intimidation.modifier, Some(38));
    assert_eq!(
        gray_cli_intimidation
            .source_entries
            .iter()
            .map(|entry| (entry.authored_key.as_str(), &entry.modifier))
            .collect::<Vec<_>>(),
        [
            ("intimidation", &CreatureIntegerPresenceJson::Value(38)),
            ("intimidate", &CreatureIntegerPresenceJson::Value(38)),
        ]
    );

    let skill_states_json = record_json(
        skill_states,
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: false,
        },
    )?;
    let RecordPresentationJson::Creature {
        skills: Some(cli_skills),
        ..
    } = &skill_states_json.presentation
    else {
        panic!("skill-state CLI creature projection must include skills");
    };
    for (authored_key, expected) in [
        ("missing-base", CreatureIntegerPresenceJson::Missing),
        ("null-base", CreatureIntegerPresenceJson::Null),
        ("value-base", CreatureIntegerPresenceJson::Value(27)),
    ] {
        let skill = cli_skills
            .iter()
            .find(|skill| {
                skill
                    .unmodeled
                    .as_ref()
                    .is_some_and(|fact| fact.authored_key == authored_key)
            })
            .unwrap_or_else(|| panic!("CLI unmodeled skill {authored_key}"));
        let fact = skill.unmodeled.as_ref().expect("CLI unmodeled fact");
        assert_eq!(fact.base, expected);
        assert_eq!(fact.reason, "unknown_authored_key");
        assert_eq!(skill.source_entries.len(), 1);
        assert_eq!(skill.source_entries[0].authored_key, authored_key);
        assert_eq!(skill.source_entries[0].modifier, expected);
        assert_eq!(skill.modifier, None);
    }
    drop(reader);

    let connection = Connection::open(&output_path)?;
    let ability_metric_count: usize = connection.query_row(
        "SELECT COUNT(*) FROM record_metrics WHERE record_key = ?1 AND metric_key LIKE 'ability.%.mod'",
        [RECORD_KEY],
        |row| row.get(0),
    )?;
    assert_eq!(ability_metric_count, 6);
    let strength_modifier: f64 = connection.query_row(
        "SELECT number_value FROM record_metrics WHERE record_key = ?1 AND metric_key = 'ability.str.mod'",
        [RECORD_KEY],
        |row| row.get(0),
    )?;
    assert_eq!(strength_modifier, 5.0);
    Ok(())
}
