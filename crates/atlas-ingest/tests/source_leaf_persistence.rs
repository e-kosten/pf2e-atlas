use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use atlas_domain::{DetailLevel, NumericMatch, RecordKey, SimpleSearchFilter};
use atlas_index::{FilteredRecordSort, FtsColumnWeights, FtsQuery, SqliteIndexReader};
use atlas_ingest::{
    BuildArtifactOptions, SourceIdentity, SourcePresence, build_artifact, parse_npc_source,
    pinned_source_version_metadata,
};
use atlas_record::{
    CreatureIntegerPresenceJson, CreatureSkillKind, CreatureUnmodeledSkillReason, FactValue,
    RecordBody, RecordJsonContext, RecordJsonOptions, RecordPresentationJson, SpellSourceValue,
    record_json, record_json_with_context,
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
static SPELL_ARTIFACT_TEST_LOCK: Mutex<()> = Mutex::new(());

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

#[test]
fn real_spell_fixture_round_trips_canonical_body_and_consumable_child()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = SPELL_ARTIFACT_TEST_LOCK
        .lock()
        .expect("spell artifact lock");
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/foundry-source/spell-source-contract");
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
    assert_eq!(report.source_record_count, 9);

    let heal_key = RecordKey::parse("spells-srd:rfZpqmj0AIIdkVIs")?;
    let rime_key = RecordKey::parse("spells-srd:Popa5umI3H33levx")?;
    let deity_key = RecordKey::parse("spells-srd:x9RIFhquazom4p02")?;
    let planar_key = RecordKey::parse("spells-srd:HmKajQS0DP23bipp")?;
    let qi_key = RecordKey::parse("spells-srd:oo7YcRC2gcez81PV")?;
    let wand_key = RecordKey::parse("equipment-srd:eOtQtVRLeGH39dNx")?;
    let reader = SqliteIndexReader::open_read_only(&output_path)?;
    let blazing_key = RecordKey::parse("spells-srd:NacrNSvfODxpZena")?;
    let blazing = reader.load_hydrated_records_by_key(std::slice::from_ref(&blazing_key))?;
    assert_eq!(blazing.len(), 1);
    assert_eq!(blazing[0].record.identity.key, blazing_key);
    assert!(matches!(blazing[0].body, Some(RecordBody::Spell(_))));
    let hydrated = reader.load_hydrated_records_by_key(&[
        heal_key.clone(),
        rime_key.clone(),
        deity_key.clone(),
        planar_key.clone(),
        qi_key.clone(),
        wand_key.clone(),
    ])?;
    assert_eq!(hydrated.len(), 6);

    let heal = hydrated
        .iter()
        .find(|record| record.record.identity.key == heal_key)
        .expect("persisted Heal");
    assert!(heal.record.mechanics.metrics.is_empty());
    assert!(matches!(
        heal.record.mechanics.document,
        atlas_record::FoundryDocumentMechanics::None
    ));
    let heal_json = record_json(
        heal,
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: false,
        },
    )?;
    let serialized_heal = serde_json::to_value(&heal_json)?;
    assert_eq!(serialized_heal["presentation_type"], "spell");
    assert_eq!(
        serialized_heal["spell"]["targeting"]["value"]["range"]["value"]["authored_text"],
        "varies"
    );
    assert!(
        serialized_heal["spell"]["targeting"]["value"]["range"]["value"]
            .get("numeric")
            .is_none(),
        "ordinary JSON must not expose the numeric range derivative"
    );
    assert_eq!(serialized_heal["spell"]["forms"][0]["label"], "Base");
    assert!(
        serialized_heal["spell"]["forms"][0]["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("spell-form:"))
    );
    assert_eq!(serialized_heal["spell"]["forms"][1]["label"], "Overlay 1");
    assert_eq!(serialized_heal["spell"]["damage"]["value"][0]["key"], "0");
    assert_eq!(
        serialized_heal["spell"]["forms"][1]["result"]["state"],
        "available"
    );
    assert_eq!(
        serialized_heal["spell"]["forms"][1]["result"]["definition"]["targeting"]["value"]["value"]
            ["range"]["value"]["authored_text"],
        "touch"
    );
    assert!(
        serialized_heal["spell"]["content"]
            .as_array()
            .is_some_and(|v| !v.is_empty())
    );
    assert!(serialized_heal["spell"].get("provenance").is_none());

    let heal_provenance = record_json_with_context(
        heal,
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: false,
        },
        RecordJsonContext::without_lookups(&heal.record).with_provenance_evidence(),
    )?;
    let serialized_provenance = serde_json::to_value(heal_provenance)?;
    assert_eq!(
        serialized_provenance["spell"]["provenance"]["source_id"],
        "rfZpqmj0AIIdkVIs"
    );
    assert_eq!(
        serialized_provenance["spell"]["provenance"]["publication_license"]["value"],
        "ORC"
    );
    assert!(
        serialized_provenance["spell"]["provenance"]["members"]
            .as_array()
            .is_some_and(|members| members.iter().any(|member| {
                member["field"] == "overlay"
                    && member["authored_key"] == "7qdtetowq348s9oc"
                    && member["source_path"] == "system.overlays.7qdtetowq348s9oc"
            }))
    );
    let Some(RecordBody::Spell(heal)) = heal.body.as_ref() else {
        panic!("standalone Heal must hydrate from its canonical spell body");
    };
    assert_eq!(heal.identity.name, "Heal");
    assert_eq!(
        heal.definition
            .source_context
            .publication_license
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .map(|value| value.as_str()),
        Some("ORC")
    );
    let classification = heal
        .definition
        .classification
        .as_value()
        .and_then(SpellSourceValue::as_known)
        .expect("Heal classification");
    assert_eq!(
        classification
            .traditions
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("Heal traditions")
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["divine", "primal"]
    );

    for (key, assertion) in [
        (&rime_key, "fixed"),
        (&deity_key, "defense"),
        (&planar_key, "ritual"),
    ] {
        let retrieved = hydrated
            .iter()
            .find(|record| &record.record.identity.key == key)
            .expect("named spell hydrates");
        let value = serde_json::to_value(record_json(
            retrieved,
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        )?)?;
        match assertion {
            "fixed" => {
                assert_eq!(
                    value["spell"]["targeting"]["value"]["area"]["value"]["value"]["value"],
                    15
                );
                assert_eq!(
                    value["spell"]["damage"]["value"][0]["formula"]["value"],
                    "2d4"
                );
                assert_eq!(
                    value["spell"]["defense"]["value"]["save"]["value"]["basic"]["value"],
                    true
                );
                assert_eq!(value["spell"]["heightening"]["value"]["kind"], "fixed");
                let layers = value["spell"]["heightening"]["value"]["layers"]
                    .as_array()
                    .expect("fixed layers");
                assert_eq!(layers[0]["key"], "5");
                assert_eq!(
                    layers[0]["patch"]["targeting"]["value"]["area"]["value"]["value"]["value"],
                    30
                );
                assert_eq!(
                    layers[0]["patch"]["damage"]["value"]["members"][0]["key"],
                    "0"
                );
                assert_eq!(
                    layers[0]["patch"]["damage"]["value"]["members"][0]["value"]["formula"]["value"],
                    "8d4"
                );
                assert_eq!(layers[1]["key"], "8");
                assert_eq!(
                    layers[1]["patch"]["targeting"]["value"]["area"]["value"]["value"]["value"],
                    60
                );
                assert_eq!(
                    layers[1]["patch"]["damage"]["value"]["members"][0]["value"]["formula"]["value"],
                    "14d4"
                );
            }
            "defense" => {
                assert_eq!(value["spell"]["defense"]["value"]["passive"]["value"], "ac");
                assert_eq!(
                    value["spell"]["defense"]["value"]["save"]["value"]["statistic"]["value"],
                    "reflex"
                );
            }
            "ritual" => assert_eq!(
                value["spell"]["ritual"]["value"]["secondary_casters"]["value"],
                2
            ),
            _ => unreachable!(),
        }
    }

    let qi = hydrated
        .iter()
        .find(|record| record.record.identity.key == qi_key)
        .expect("persisted Qi Blast");
    let qi_json = serde_json::to_value(record_json(
        qi,
        RecordJsonOptions {
            detail: DetailLevel::Standard,
            include_source_json: false,
        },
    )?)?;
    assert_eq!(qi_json["spell"]["rules"]["value"][0]["kind"], "roll_option");
    assert_eq!(
        qi_json["spell"]["rules"]["value"][0]["authored_key"],
        "RollOption"
    );
    assert_eq!(
        qi_json["spell"]["rules"]["value"][0]["value"]["option"]["value"],
        "heavens-thunder"
    );
    let qi_provenance = serde_json::to_value(record_json_with_context(
        qi,
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: false,
        },
        RecordJsonContext::without_lookups(&qi.record).with_provenance_evidence(),
    )?)?;
    assert!(
        qi_provenance["spell"]["provenance"]["members"]
            .as_array()
            .is_some_and(|members| members.iter().any(|member| {
                member["field"] == "rule"
                    && member["authored_key"] == "RollOption"
                    && member["source_path"] == "system.rules.0"
            }))
    );

    let filter = SimpleSearchFilter {
        spell_rank: Some(NumericMatch::Eq { value: 7.0 }),
        spell_traditions: vec!["divine".to_string()],
        spell_save_types: vec!["reflex".to_string()],
        spell_basic_save: Some(true),
        spell_damage_types: vec!["force".to_string()],
        ..SimpleSearchFilter::default()
    }
    .into_filter_node()?
    .expect("spell filter exists");
    let filtered = reader.list_filtered_record_keys(
        Some(&filter),
        None,
        FilteredRecordSort::RecordKey,
        20,
        0,
    )?;
    assert_eq!(filtered.record_keys, vec![deity_key.clone()]);

    let authored_false = SimpleSearchFilter {
        spell_basic_save: Some(false),
        ..SimpleSearchFilter::default()
    }
    .into_filter_node()?
    .expect("false basic-save filter exists");
    let filtered = reader.list_filtered_record_keys(
        Some(&authored_false),
        None,
        FilteredRecordSort::RecordKey,
        20,
        0,
    )?;
    assert!(filtered.record_keys.is_empty());

    let fts = reader.query_weighted_fts_index(
        &FtsQuery::from_tokens(vec!["precious".to_string(), "chalk".to_string()])
            .expect("safe FTS query"),
        None,
        10,
        FtsColumnWeights::default(),
    )?;
    assert!(fts.iter().any(|hit| hit.record_key == planar_key));

    let wand = hydrated
        .iter()
        .find(|record| record.record.identity.key == wand_key)
        .expect("persisted Arboreal Wand");
    assert!(
        matches!(wand.body, Some(RecordBody::Consumable(_))),
        "consumables hydrate their own body without acquiring a spell body"
    );
    assert_eq!(wand.spell_children.len(), 1);
    let child = &wand.spell_children[0];
    assert_eq!(child.child_id.as_str(), "7w37duycMs4YOBeu");
    assert_eq!(child.authored_order, 0);
    assert_eq!(
        child
            .definition
            .source_context
            .publication_license
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .map(|value| value.as_str()),
        Some("ORC")
    );
    assert!(matches!(
        &child.standalone_target,
        FactValue::Value(atlas_record::SpellStandaloneTarget::Resolved(key)) if key == &heal_key
    ));
    assert_eq!(
        child
            .definition
            .source_context
            .consumable_child
            .as_value()
            .and_then(|context| context.slug.as_value())
            .and_then(SpellSourceValue::as_known)
            .map(String::as_str),
        Some("heal")
    );

    drop(reader);
    let connection = Connection::open(&output_path)?;
    connection.execute(
        "UPDATE item_records SET damage_types_json='[\"cold\"]' WHERE record_key=?1",
        [wand_key.to_string()],
    )?;
    drop(connection);
    atlas_index::test_support::write_bound_test_manifest(&output_path)?;
    let reader = SqliteIndexReader::open_read_only(&output_path)?;
    let cold_spells = SimpleSearchFilter {
        spell_damage_types: vec!["cold".to_string()],
        ..SimpleSearchFilter::default()
    }
    .into_filter_node()?
    .expect("spell damage filter exists");
    let filtered = reader.list_filtered_record_keys(
        Some(&cold_spells),
        None,
        FilteredRecordSort::RecordKey,
        20,
        0,
    )?;
    assert_eq!(filtered.record_keys, vec![rime_key]);
    drop(reader);
    let connection = Connection::open(&output_path)?;
    let stored_spell_bodies: i64 =
        connection.query_row("SELECT COUNT(*) FROM canonical_spell_records", [], |row| {
            row.get(0)
        })?;
    let stored_children: i64 = connection.query_row(
        "SELECT COUNT(*) FROM canonical_consumable_spell_children",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(stored_spell_bodies, 8);
    assert_eq!(stored_children, 1);
    let legacy_spell_item_rows: i64 = connection.query_row(
        "SELECT COUNT(*)
         FROM item_records i
         JOIN records r ON r.record_key = i.record_key
         WHERE r.record_kind = 'spell'",
        [],
        |row| row.get(0),
    )?;
    let legacy_spell_metric_rows: i64 = connection.query_row(
        "SELECT COUNT(*)
         FROM record_metrics m
         JOIN records r ON r.record_key = m.record_key
         WHERE r.record_kind = 'spell'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(legacy_spell_item_rows, 0);
    assert_eq!(legacy_spell_metric_rows, 0);
    let heal_content: i64 = connection.query_row(
        "SELECT COUNT(*) FROM record_content WHERE record_key = ?1",
        [heal_key.to_string()],
        |row| row.get(0),
    )?;
    assert_eq!(heal_content, heal.definition.content.documents.len() as i64);
    let wand_content: i64 = connection.query_row(
        "SELECT COUNT(*) FROM record_content WHERE record_key = ?1",
        [wand_key.to_string()],
        |row| row.get(0),
    )?;
    let wand_references: i64 = connection.query_row(
        "SELECT COUNT(*) FROM reference_occurrences WHERE record_key = ?1",
        [wand_key.to_string()],
        |row| row.get(0),
    )?;
    assert_eq!(wand_content, 1, "parent non-spell content remains stored");
    assert_eq!(
        wand_references, 1,
        "shared reference persistence remains active"
    );
    Ok(())
}

#[test]
fn spell_artifact_reader_rejects_grouped_body_child_and_projection_faults()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = SPELL_ARTIFACT_TEST_LOCK
        .lock()
        .expect("spell artifact lock");
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/foundry-source/spell-source-contract");
    let temporary = TemporaryOutput::new()?;
    let output_path = temporary.artifact();
    build_artifact(BuildArtifactOptions {
        source_root,
        output_path: output_path.clone(),
        manifest_path: None,
        embedding_model_id: BuildArtifactOptions::default_embedding_model_id(),
        embedding_cache_root: None,
        reuse_embeddings: true,
        embedding_batch_size: 8,
    })?;

    let heal = RecordKey::parse("spells-srd:rfZpqmj0AIIdkVIs")?;
    let wand = RecordKey::parse("equipment-srd:eOtQtVRLeGH39dNx")?;
    let faults = [
        (
            "missing-body",
            "PRAGMA foreign_keys=OFF;
             DELETE FROM canonical_spell_records WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![heal.clone()],
            "no canonical spell body",
        ),
        (
            "extra-body",
            "INSERT INTO canonical_spell_records(record_key,source_id,name,canonical_json)
             SELECT 'equipment-srd:eOtQtVRLeGH39dNx',source_id,name,canonical_json
             FROM canonical_spell_records WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![wand.clone()],
            "does not match row key",
        ),
        (
            "malformed-body",
            "UPDATE canonical_spell_records SET canonical_json='{}'
             WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![heal.clone()],
            "canonical_json",
        ),
        (
            "mismatched-body-key",
            "UPDATE canonical_spell_records
             SET canonical_json=(SELECT canonical_json FROM canonical_spell_records
                                 WHERE record_key='spells-srd:5gophZ4AOKW4VW27')
             WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![heal.clone()],
            "does not match row key",
        ),
        (
            "wrong-body-kind",
            "UPDATE canonical_spell_records
             SET canonical_json=replace(canonical_json,'\"kind\":\"spell\"','\"kind\":\"creature\"')
             WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![heal.clone()],
            "canonical_json",
        ),
        (
            "wrong-record-owner-identity",
            "UPDATE canonical_spell_records
             SET source_id='wrongSourceId', name='Wrong Heal',
                 canonical_json=json_set(canonical_json,
                   '$.value.identity.source_id', 'wrongSourceId',
                   '$.value.identity.name', 'Wrong Heal')
             WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![heal.clone()],
            "generic record owner",
        ),
        (
            "scalar-projection",
            "UPDATE spell_records SET rank=9
             WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![heal.clone()],
            "query projection",
        ),
        (
            "ordered-projection",
            "UPDATE spell_traditions SET authored_order=7
             WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs' AND authored_order=1",
            vec![heal.clone()],
            "query projection",
        ),
        (
            "child-order",
            "UPDATE canonical_consumable_spell_children SET authored_order=3
             WHERE parent_record_key='equipment-srd:eOtQtVRLeGH39dNx'",
            vec![wand.clone()],
            "identity/order",
        ),
        (
            "child-foreign-key",
            "PRAGMA foreign_keys=OFF;
             UPDATE canonical_consumable_spell_children
             SET standalone_target_record_key='spells-srd:missingTarget'
             WHERE parent_record_key='equipment-srd:eOtQtVRLeGH39dNx'",
            vec![wand.clone()],
            "standalone target",
        ),
        (
            "child-nonspell-target",
            "PRAGMA foreign_keys=OFF;
             UPDATE canonical_consumable_spell_children
             SET standalone_target_record_key='equipment-srd:eOtQtVRLeGH39dNx',
                 canonical_json=replace(canonical_json,
                   'spells-srd:rfZpqmj0AIIdkVIs',
                   'equipment-srd:eOtQtVRLeGH39dNx')
             WHERE parent_record_key='equipment-srd:eOtQtVRLeGH39dNx'",
            vec![wand.clone()],
            "not a canonical spell record",
        ),
        (
            "content-metadata",
            "UPDATE record_content SET label='corrupt canonical label'
             WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![heal.clone()],
            "content projection",
        ),
        (
            "reference-metadata",
            "UPDATE reference_occurrences
             SET target_kind='unresolved', target_record_key=NULL, provenance_json='{}'
             WHERE record_key='spells-srd:rfZpqmj0AIIdkVIs'",
            vec![heal.clone()],
            "reference projection",
        ),
    ];
    for (name, sql, keys, expected) in faults {
        let candidate = temporary.0.join(format!("{name}.sqlite"));
        std::fs::copy(&output_path, &candidate)?;
        Connection::open(&candidate)?.execute_batch(sql)?;
        atlas_index::test_support::write_bound_test_manifest(&candidate)?;
        let reader = SqliteIndexReader::open_read_only(&candidate)?;
        if matches!(
            name,
            "missing-body"
                | "extra-body"
                | "child-order"
                | "child-foreign-key"
                | "child-nonspell-target"
        ) {
            assert_eq!(
                reader.validate()?.status,
                atlas_index::ValidationStatus::Error,
                "{name}: structural validation must reject the global fault"
            );
        }
        let error = reader
            .load_hydrated_records_by_key(&keys)
            .expect_err(name)
            .to_string();
        assert!(
            error.contains(expected),
            "{name}: expected `{expected}`, got {error}"
        );
    }

    let legacy_side_row_faults = [
        (
            "spell-item-side-row",
            "INSERT INTO item_records (
               record_key,system_category,system_base_item,system_group,system_usage,
               system_price_json,price_cp,bulk_value,hands_requirement,damage_types_json
             )
             VALUES ('spells-srd:rfZpqmj0AIIdkVIs','wand',NULL,NULL,
                     'held-in-one-hand','{\"gp\":1}',100,0.1,'one_hand','[]')",
            "item_records.canonical_body_owner",
        ),
        (
            "spell-metric-side-row",
            "INSERT INTO record_metrics (
               record_key,ordinal,metric_domain,metric_key,value_type,
               number_value,text_value,bool_value
             )
             VALUES ('spells-srd:rfZpqmj0AIIdkVIs',0,'item','rank.value','number',1,NULL,NULL)",
            "record_metrics.canonical_spell_owner",
        ),
    ];
    for (name, sql, expected_key) in legacy_side_row_faults {
        let candidate = temporary.0.join(format!("{name}.sqlite"));
        std::fs::copy(&output_path, &candidate)?;
        Connection::open(&candidate)?.execute_batch(sql)?;
        atlas_index::test_support::write_bound_test_manifest(&candidate)?;

        let report = SqliteIndexReader::open_read_only(&candidate)?
            .validate_target(atlas_index::ValidationTarget::Full)?;
        assert_eq!(
            report.status,
            atlas_index::ValidationStatus::Error,
            "{name}: Full validation must reject a rebound legacy side row"
        );
        assert!(
            report.diagnostics.iter().any(|diagnostic| {
                diagnostic.family == atlas_index::ArtifactValidationFamily::Data
                    && diagnostic.key.as_deref() == Some(expected_key)
                    && diagnostic.expected.as_deref() == Some("0 invalid rows")
                    && diagnostic.actual.as_deref() == Some("1")
            }),
            "{name}: expected exact invariant `{expected_key}`, got {report:?}"
        );
    }

    let unsupported_candidate = temporary.0.join("unsupported-value.sqlite");
    std::fs::copy(&output_path, &unsupported_candidate)?;
    let connection = Connection::open(&unsupported_candidate)?;
    let unsupported_key = heal.to_string();
    connection.execute(
        "UPDATE canonical_spell_records
         SET canonical_json=replace(canonical_json,'\"kind\":\"known\"','\"kind\":\"unsupported\"')
         WHERE record_key=?1",
        [&unsupported_key],
    )?;
    drop(connection);
    let unsupported_key = RecordKey::parse(&unsupported_key)?;
    atlas_index::test_support::write_bound_test_manifest(&unsupported_candidate)?;
    let reader = SqliteIndexReader::open_read_only(&unsupported_candidate)?;
    reader
        .load_hydrated_records_by_key(&[unsupported_key])
        .expect_err("typed unsupported evidence corruption must fail closed");
    Ok(())
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
