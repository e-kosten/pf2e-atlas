use super::*;
use atlas_domain::{
    SourcePassageAddress,
    query::{QueryFieldState, QueryPredicate},
};
use atlas_foundry_model::{
    SNAPSHOT_VERSION, SOURCE_CONTRACT_ID, SourceContext, admit_document_source,
};
use atlas_record::{
    source_content::{CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentVisibilityRule},
    source_record::{SOURCE_CONTENT_SELECTION_VERSION, SourceBackedRecord, prepare_record_content},
};
use rusqlite::Connection;
use serde_json::{Value, json};

#[test]
#[ignore = "requires the completed corpus artifact under ancestor scratch/ingest-validation"]
fn corpus_projection_and_selected_read_plans() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| {
            p.join("scratch/ingest-validation/full-lexical.sqlite")
                .is_file()
        })
        .ok_or("corpus artifact unavailable")?;
    let db = Connection::open_with_flags(
        root.join("scratch/ingest-validation/full-lexical.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    let mut evidence = vec![];
    for (name, expression) in [
        (
            "actor-saves-iwr",
            "defenses.saves.reflex>=15 && defenses.resistances.exists(x,x.type=='fire')",
        ),
        (
            "same-child-spell",
            "embedded.spells.exists(i,i.rank>=3 && 'fire' in i.traits)",
        ),
    ] {
        let query = parse_where(expression)?;
        let (sql, parameters) = crate::read_units::eligible_sql(&query, None)?;
        let rows = db
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))?
            .query_map(rusqlite::params_from_iter(parameters), |r| {
                r.get::<_, String>(3)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        evidence.push(serde_json::json!({"name":name,"expression":expression,"plan":rows}));
    }
    for (name, sql, parameter) in [
        (
            "lexical-definition",
            "SELECT lu.unit_id FROM lexical_fts JOIN lexical_units lu ON lu.unit_id=lexical_fts.rowid JOIN records r ON r.record_id=lu.record_id WHERE lexical_fts MATCH ?1 AND r.document_kind<>'Macro' ORDER BY bm25(lexical_fts,10.0,8.0,1.0,6.0),lu.unit_id",
            "\"Ghoul Fever\"",
        ),
        ("exact-name", crate::read_units::NAME_LOOKUP_SQL, "ghoul"),
        (
            "selected-content",
            "SELECT c.html_gzip,c.interactions_json FROM prepared_content c JOIN records r USING(record_id) WHERE r.key=?1 AND c.owners_json='[]' AND c.field_path='/system/details/publicNotes' AND r.document_kind<>'Macro'",
            "bestiary-effects:1234567890123456",
        ),
    ] {
        let rows = db
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))?
            .query_map([parameter], |r| r.get::<_, String>(3))?
            .collect::<Result<Vec<_>, _>>()?;
        evidence.push(serde_json::json!({"name":name,"plan":rows}));
    }
    std::fs::write(
        root.join("scratch/ingest-validation/query-plans.json"),
        serde_json::to_vec_pretty(&evidence)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&evidence)?);
    Ok(())
}
fn audience() -> ContentAudience {
    ContentAudience {
        include_gm: true,
        include_owner: true,
        implicit_check_dc: ContentVisibilityRule::All,
    }
}
fn context(n: usize, semantic: bool) -> SourceArtifactBuildContext {
    SourceArtifactBuildContext {
        trait_policy: "trait-definition-centroid/v1".into(),
        format_version: ARTIFACT_FORMAT_VERSION,
        snapshot_version: SNAPSHOT_VERSION,
        source_contract: SOURCE_CONTRACT_ID.into(),
        source_revision: None,
        source_fingerprint: "a".repeat(64),
        source_file_count: n,
        indexing_locale: "en".into(),
        locale_catalog_sha256: "b".repeat(64),
        english_catalog_sha256: "b".repeat(64),
        used_trait_labels: std::collections::BTreeMap::new(),
        localization_policy: LOCALIZATION_POLICY_VERSION.into(),
        audience: audience(),
        content_interpretation_version: CONTENT_INTERPRETATION_VERSION.into(),
        content_selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
        asset_policy: ASSET_POLICY_VERSION.into(),
        relationship_policy: RELATIONSHIP_POLICY_VERSION.into(),
        query_projection_version: SOURCE_PROJECTION_VERSION.into(),
        query_catalog_version: query_capabilities().unwrap().version,
        lookup_version: SOURCE_LOOKUP_VERSION.into(),
        lexical_selection_version: LEXICAL_SELECTION_VERSION.into(),
        fts_tokenizer: "unicode61 remove_diacritics 2".into(),
        search_profiles: if semantic {
            vec![SourceSearchProfileIdentity::from_spec(
                atlas_embedding::SEARCH_PROFILES[0],
            )]
        } else {
            vec![]
        },
    }
}
pub(super) fn record(pack: &str, kind: &str, v: Value) -> SourceArtifactRecordInput {
    let source = admit_document_source(
        kind,
        SourceContext::new(pack, "fixture", "$"),
        &serde_json::to_vec(&v).unwrap(),
    )
    .unwrap()
    .model
    .unwrap();
    let record = SourceBackedRecord::new(pack, source).unwrap();
    let content = prepare_record_content(&record, audience(), None, None);
    let relationships = atlas_record::source_record::resolve_source_relationships(&record, None);
    SourceArtifactRecordInput {
        record,
        source_path: format!("packs/{pack}/{}.json", v["_id"].as_str().unwrap()),
        content_hash: crate::codec::sha256(&serde_json::to_vec(&v).unwrap()),
        preparation_context_hash: String::new(),
        content,
        relationships,
        diagnostics: vec![],
    }
}
pub(super) fn fixture(semantic: bool) -> IndexBuildInput {
    let records = vec![
        record(
            "actors",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Dragon Guard","type":"npc","system":{"details":{"level":{"value":9}},"attributes":{"hp":{"max":80},"ac":{"value":24}},"traits":{"value":["dragon"],"rarity":"rare","size":{"value":"lg"}}}}),
        ),
        record(
            "items",
            "Item",
            json!({"_id":"bbbbbbbbbbbbbbbb","name":"Dragon Staff","type":"weapon","system":{"level":{"value":0},"traits":{"value":[],"rarity":"common"},"price":{"value":{"gp":2},"per":1},"bulk":{"value":1},"description":{"value":"<h2>Flame</h2><p>@Check[reflex|dc:20] Dragon breath.</p>"}}}),
        ),
        record(
            "items",
            "Item",
            json!({"_id":"cccccccccccccccc","name":"Dragon Spell","type":"spell","system":{"level":{"value":3},"traits":{"value":["fire"],"rarity":"common","traditions":["arcane"]},"duration":{"sustained":false,"value":"1 minute"},"area":{"value":30,"type":"burst"}}}),
        ),
        record(
            "macros",
            "Macro",
            json!({"_id":"dddddddddddddddd","name":"Hidden Dragon","type":"script","command":"throw new Error()"}),
        ),
    ];
    let mut input = IndexBuildInput {
        traits: vec![],
        trait_vectors: vec![],
        context: context(records.len(), semantic),
        records,
        packs: vec![
            IndexBuildPack {
                pack_id: "actors".into(),
                label: "Actors".into(),
                document_kind: "Actor".into(),
                declared_path: "packs/actors".into(),
            },
            IndexBuildPack {
                pack_id: "items".into(),
                label: "Items".into(),
                document_kind: "Item".into(),
                declared_path: "packs/items".into(),
            },
            IndexBuildPack {
                pack_id: "macros".into(),
                label: "Macros".into(),
                document_kind: "Macro".into(),
                declared_path: "packs/macros".into(),
            },
        ],
        aliases: vec![],
        remaster_pairs: vec![],
        lexical_units: vec![],
        semantic_units: vec![],
    };
    let hash = input.context.preparation_sha256().unwrap();
    for (i, r) in input.records.iter_mut().enumerate() {
        r.preparation_context_hash = hash.clone();
        if i == 3 {
            continue;
        }
        let name = ["Dragon Guard", "Dragon Staff", "Dragon Spell"][i];
        input.lexical_units.push(SourceLexicalUnitInput {
            record: r.record.key().clone(),
            owners: vec![],
            field: None,
            address: None,
            kind: SourceLexicalUnitKind::RootName,
            identity_terms: name.into(),
            alias_terms: String::new(),
            structured_terms: String::new(),
            definition_terms: String::new(),
        });
        if semantic {
            let mut vector = vec![0.0; 768];
            vector[i] = 1.0;
            input.semantic_units.push(SourceSemanticUnitInput {
                profile_id: atlas_domain::SearchProfileId::new("bge-base"),
                record: r.record.key().clone(),
                owners: vec![],
                field: None,
                address: SourcePassageAddress::Identity {},
                chunk_ordinal: 0,
                input_token_count: 8,
                input_hash: crate::codec::sha256(name.as_bytes()),
                vector,
            });
        }
    }
    for r in &input.records {
        let selection = atlas_record::source_record::select_record_search(
            &r.record,
            &r.content,
            audience(),
            "Fixture",
            &std::collections::BTreeMap::new(),
        )
        .unwrap();
        for field in &selection.fields {
            for section in &field.sections {
                let range = atlas_domain::SourceByteRange {
                    start: 0,
                    end: section.text.len(),
                };
                let address = if let Some(hash) = &field.prepared_html_sha256 {
                    SourcePassageAddress::HtmlSection {
                        prepared_html_sha256: hash.clone(),
                        canonical_text_sha256: crate::codec::sha256(section.text.as_bytes()),
                        selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                        section_ordinal: section.section_ordinal,
                        label: section.label.clone(),
                        chunk_bytes: range,
                    }
                } else {
                    SourcePassageAddress::PlainSection {
                        source_text_sha256: crate::codec::sha256(section.text.as_bytes()),
                        selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                        section_ordinal: section.section_ordinal,
                        label: section.label.clone(),
                        chunk_bytes: range,
                    }
                };
                if section.is_lexical_definition() {
                    input.lexical_units.push(SourceLexicalUnitInput{record:r.record.key().clone(),owners:field.locator.owners.clone(),field:Some(field.locator.field.clone()),address:Some(address.clone()),kind:match section.kind{atlas_record::source_record::SourceSelectedSectionKind::Heading=>SourceLexicalUnitKind::Heading,atlas_record::source_record::SourceSelectedSectionKind::DefinitionLabel=>SourceLexicalUnitKind::DefinitionLabel,_=>unreachable!()},identity_terms:selection.identity_required.clone(),alias_terms:String::new(),structured_terms:String::new(),definition_terms:section.label.clone().unwrap()});
                }
                if semantic {
                    input.semantic_units.push(SourceSemanticUnitInput {
                        profile_id: atlas_domain::SearchProfileId::new("bge-base"),
                        record: r.record.key().clone(),
                        owners: field.locator.owners.clone(),
                        field: Some(field.locator.field.clone()),
                        address,
                        chunk_ordinal: 0,
                        input_token_count: 8,
                        input_hash: crate::codec::sha256(section.text.as_bytes()),
                        vector: input
                            .semantic_units
                            .iter()
                            .find(|u| u.record == *r.record.key())
                            .unwrap()
                            .vector
                            .clone(),
                    });
                }
            }
        }
    }
    synchronize_fixture_terms(&mut input);
    input
}
fn synchronize_fixture_terms(input: &mut IndexBuildInput) {
    for r in &input.records {
        let selection = atlas_record::source_record::select_record_search(
            &r.record,
            &r.content,
            audience(),
            &input
                .packs
                .iter()
                .find(|p| p.pack_id == r.record.key().pack().as_str())
                .unwrap()
                .label,
            &input.context.used_trait_labels,
        )
        .unwrap();
        let aliases = input
            .aliases
            .iter()
            .filter(|alias| alias.record == *r.record.key())
            .map(|alias| alias.alias.clone())
            .collect::<Vec<_>>();
        input
            .lexical_units
            .retain(|unit| unit.record != *r.record.key());
        input
            .lexical_units
            .extend(crate::project_lexical_units(&selection, &aliases));
    }
}
pub(super) fn build(input: &IndexBuildInput) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("artifact.sqlite");
    crate::test_support::publish_input(&path, input).unwrap();
    (directory, path)
}

#[test]
fn publication_validation_observes_uncommitted_projection_changes() {
    let (_directory, path) = build(&fixture(false));
    let mut connection = Connection::open(&path).unwrap();
    let transaction = connection.transaction().unwrap();
    transaction
        .execute(
            "UPDATE records SET name='incorrect projection' WHERE record_id=1",
            [],
        )
        .unwrap();
    assert!(matches!(
        validate_reference(&transaction),
        Err(IndexError::Invalid(_))
    ));
    transaction.rollback().unwrap();
    validate_reference(&connection).unwrap();
}
#[test]
fn roundtrip_summary_content_and_macro_scope() {
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    for r in &input.records {
        let decoded = reader
            .read_source_record_for_inspection(r.record.key())
            .unwrap()
            .unwrap();
        assert_eq!(
            atlas_foundry_model::encode_snapshot(decoded.source()).unwrap(),
            atlas_foundry_model::encode_snapshot(r.record.source()).unwrap()
        );
    }
    let actor = reader
        .read_summary(input.records[0].record.key())
        .unwrap()
        .unwrap();
    assert_eq!(actor.level.as_value().unwrap().as_i64(), Some(9));
    assert_eq!(
        actor.traits.as_value().unwrap(),
        &vec!["dragon".to_string()]
    );
    let weapon = reader
        .read_summary(input.records[1].record.key())
        .unwrap()
        .unwrap();
    assert_eq!(weapon.level.as_value().unwrap().as_i64(), Some(0));
    assert_eq!(weapon.traits.state, QueryFieldState::Value);
    assert!(weapon.traits.as_value().unwrap().is_empty());
    let spell = reader
        .read_summary(input.records[2].record.key())
        .unwrap()
        .unwrap();
    assert_eq!(
        spell.level_basis,
        Some(atlas_domain::SourceLevelBasis::SpellRank)
    );
    assert_eq!(spell.level.as_value().unwrap().as_i64(), Some(3));
    assert!(
        reader
            .read_source_record(input.records[3].record.key())
            .unwrap()
            .is_none()
    );
    assert!(
        reader
            .read_summary(input.records[3].record.key())
            .unwrap()
            .is_none()
    );
    let content = reader
        .read_content_for_record(input.records[1].record.key(), 256)
        .unwrap();
    assert!(!content.truncated);
    assert!(content.fields.iter().any(|f| {
        f.html
            .as_deref()
            .is_some_and(|h| h.contains("data-atlas-interaction"))
    }));
    let all = validate_query(&QueryPredicate::boolean(true)).unwrap();
    assert_eq!(
        reader
            .eligible_keys(&all, None, false, atlas_domain::ResultSort::NameAsc, 0, 100)
            .unwrap()
            .keys
            .len(),
        3
    );
    assert_eq!(
        reader
            .lexical_root_candidates("dragon", &all, None)
            .unwrap()
            .len(),
        3
    );
}
#[test]
fn filtering_happens_before_vector_top_k_and_uses_numbered_bindings() {
    let input = fixture(true);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(path).unwrap();
    let query = parse_where("kind == 'spell'").unwrap();
    let hits = reader
        .vector_candidates(
            &atlas_domain::SearchProfileId::new("bge-base"),
            &input.semantic_units[0].vector,
            &query,
            None,
            1,
        )
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].location.record, *input.records[2].record.key());
    let lexical = reader
        .lexical_candidates("dragon", &query, None, 10)
        .unwrap();
    assert_eq!(lexical.len(), 1);
    assert_eq!(lexical[0].location.record, *input.records[2].record.key());
    let all = validate_query(&QueryPredicate::boolean(true)).unwrap();
    let scoped = reader
        .vector_candidates(
            &atlas_domain::SearchProfileId::new("bge-base"),
            &input.semantic_units[0].vector,
            &all,
            Some(std::slice::from_ref(input.records[1].record.key())),
            1,
        )
        .unwrap();
    assert_eq!(scoped[0].location.record, *input.records[1].record.key());
    assert!(
        reader
            .vector_candidates(
                &atlas_domain::SearchProfileId::new("bge-base"),
                &input.semantic_units[0].vector,
                &all,
                None,
                4097
            )
            .is_err()
    );
    let seed = reader
        .seed_identity_vector(
            &atlas_domain::SearchProfileId::new("bge-base"),
            &input.semantic_units[0].record,
        )
        .unwrap()
        .unwrap();
    assert_eq!(seed.vector, input.semantic_units[0].vector);
}
#[test]
fn corrupted_snapshot_does_not_force_summary_or_batched_content_decode() {
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    drop(reader);
    let c = Connection::open(&path).unwrap();
    c.execute("UPDATE record_bodies SET snapshot=x'00'", [])
        .unwrap();
    let reader = crate::test_support::open_reader(&path).unwrap();
    let batch = reader
        .read_summaries(
            &input
                .records
                .iter()
                .map(|r| r.record.key().clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
    assert_eq!(batch.records.len(), 3);
    assert_eq!(batch.missing.len(), 1);
    let fields = reader
        .read_content_for_record(input.records[1].record.key(), 256)
        .unwrap()
        .fields;
    let batch = reader
        .read_content(&fields.iter().map(|f| f.locator.clone()).collect::<Vec<_>>())
        .unwrap();
    assert_eq!(batch.fields.len(), fields.len());
    assert!(
        reader
            .read_source_record(input.records[1].record.key())
            .is_err()
    );
    assert!(validate_artifact(path).is_err());
}
#[test]
fn reference_validation_rejects_missing_reference_sidecar() {
    let mut input = fixture(false);
    let mut staff = record(
        "items",
        "Item",
        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Dragon Staff","type":"weapon","system":{"level":{"value":0},"traits":{"value":[],"rarity":"common"},"description":{"value":"<p>See @UUID[Compendium.pf2e.items.Item.cccccccccccccccc]{Dragon Spell}.</p>"}}}),
    );
    staff.preparation_context_hash = input.context.preparation_sha256().unwrap();
    let key = staff.record.key().clone();
    input.records[1] = staff;
    input.lexical_units.retain(|u| u.field.is_none());
    synchronize_fixture_terms(&mut input);
    let (_directory, path) = build(&input);
    let db = Connection::open(&path).unwrap();
    let records = crate::read_persisted_inputs(&db, &input.context).unwrap();
    assert!(records.iter().any(|r| r.record.key() == &key));
    let deleted = db.execute(
        "DELETE FROM relationship_occurrences WHERE origin='content' AND record_id=(SELECT record_id FROM records WHERE key=?)",
        [key.to_string()],
    ).unwrap();
    assert!(deleted > 0);
    assert!(crate::validate_artifact(&path).is_err());
}
#[test]
fn failed_build_preserves_published_artifact() {
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let before = validate_artifact(&path).unwrap();
    let mut broken = fixture(true);
    broken.semantic_units.pop();
    assert!(crate::test_support::publish_input(&path, &broken).is_err());
    assert_eq!(validate_artifact(&path).unwrap(), before);
    let mut bad = fixture(false);
    bad.records[1].preparation_context_hash = "0".repeat(64);
    assert!(crate::test_support::publish_input(&path, &bad).is_err());
    assert_eq!(validate_artifact(&path).unwrap(), before);
    assert!(crate::test_support::open_reader(&path).is_err());
    crate::test_support::publish_input(&path, &input).unwrap();
    assert!(crate::test_support::open_reader(&path).is_ok());
}
#[test]
fn stale_contract_and_projection_corruption_require_rebuild() {
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let c = Connection::open(&path).unwrap();
    c.execute(
        "UPDATE actor_projection SET hp_maximum=81 WHERE record_id=1",
        [],
    )
    .unwrap();
    assert!(validate_artifact(&path).is_err());
    c.execute_batch("PRAGMA user_version=1").unwrap();
    assert!(matches!(
        crate::test_support::open_reader(path),
        Err(IndexError::Storage(_))
    ));
}
#[test]
fn numeric_domain_keeps_integer_precision_and_rejects_unrepresentable_unsigned() {
    use rusqlite::types::Value as Sql;
    for n in [i64::MIN, i64::MAX, 9_007_199_254_740_993i64] {
        assert_eq!(
            crate::numeric::numeric_value(&n.into()).unwrap(),
            Sql::Integer(n)
        );
    }
    for n in [i64::MAX as u64 + 1, u64::MAX] {
        assert!(crate::numeric::numeric_value(&n.into()).is_err());
    }
    assert_eq!(
        crate::numeric::numeric_value(&serde_json::Number::from_f64(0.25).unwrap()).unwrap(),
        Sql::Real(0.25)
    );
    let mut input = fixture(false);
    input.records[0] = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Huge Level","type":"npc","system":{"details":{"level":{"value":9_007_199_254_740_993i64}}}}),
    );
    input.records[0].preparation_context_hash = input.context.preparation_sha256().unwrap();
    input.lexical_units[0].identity_terms = "Huge Level".into();
    synchronize_fixture_terms(&mut input);
    let (_directory, path) = build(&input);
    assert_eq!(
        crate::test_support::open_reader(path)
            .unwrap()
            .read_summary(input.records[0].record.key())
            .unwrap()
            .unwrap()
            .level
            .as_value()
            .unwrap()
            .as_i64(),
        Some(9_007_199_254_740_993)
    );
    for vector in [
        vec![f32::NAN; 768],
        vec![f32::INFINITY; 768],
        vec![0.0; 768],
        vec![1.0; 2],
    ] {
        assert!(crate::validation::check_vector(&vector, 768).is_err());
    }
}
#[test]
fn bounded_reader_reopens_after_maintenance_publication() {
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let original = reader.artifact_instance().clone();
    let original_context = reader.context().clone();
    assert_eq!(
        reader
            .read_summary(input.records[0].record.key())
            .unwrap()
            .unwrap()
            .name
            .as_value()
            .unwrap(),
        "Dragon Guard"
    );
    drop(reader);
    let mut replacement = fixture(false);
    replacement.records[0] = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Replacement","type":"npc"}),
    );
    replacement.records[0].preparation_context_hash =
        replacement.context.preparation_sha256().unwrap();
    replacement.lexical_units[0].identity_terms = "Replacement".into();
    synchronize_fixture_terms(&mut replacement);
    crate::test_support::publish_input(&path, &replacement).unwrap();
    let reader = crate::test_support::open_reader(&path).unwrap();
    assert_eq!(reader.context(), &original_context);
    assert_ne!(reader.artifact_instance(), &original);
    assert_eq!(
        reader
            .read_summary(input.records[0].record.key())
            .unwrap()
            .unwrap()
            .name
            .as_value()
            .unwrap(),
        "Replacement"
    );
}

#[test]
fn independently_opened_readers_share_one_artifact_instance() {
    let (_directory, path) = build(&fixture(false));
    let first = crate::test_support::open_reader(&path).unwrap();
    let second = crate::test_support::open_reader(&path).unwrap();
    assert_eq!(first.artifact_instance(), second.artifact_instance());
}

#[test]
fn prepared_cache_declared_policy_and_complete_coverage_are_checked() {
    for sql in [
        "DELETE FROM prepared_content WHERE record_id=2",
        "UPDATE prepared_content SET role='Notes' WHERE record_id=2",
        "UPDATE prepared_content SET visibility='None' WHERE record_id=2",
        "UPDATE prepared_content SET published=0 WHERE record_id=2",
        "UPDATE prepared_content SET outcome='unsupported_format',html_gzip=NULL,interactions_json='[]' WHERE record_id=2",
        "UPDATE prepared_content SET authored_markup_sha256=printf('%064d',0) WHERE record_id=2",
        "UPDATE prepared_content SET owners_json='[{\"collection\":\"items\",\"identity\":{\"SnapshotLocal\":{\"index\":99}}}]' WHERE record_id=2",
    ] {
        let input = fixture(false);
        let (_directory, path) = build(&input);
        Connection::open(&path).unwrap().execute_batch(sql).unwrap();
        assert!(validate_artifact(path).is_err(), "{sql}");
    }
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let c = Connection::open(&path).unwrap();
    c.execute(
        "UPDATE prepared_content SET html_gzip=? WHERE record_id=2",
        [crate::codec::gzip(b"<script>alert(1)</script>").unwrap()],
    )
    .unwrap();
    assert!(validate_artifact(path).is_err());
}
#[test]
fn fts_posting_and_semantic_tail_corruption_are_rejected() {
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let c = Connection::open(&path).unwrap();
    c.execute_batch("INSERT INTO lexical_fts(lexical_fts,rowid,identity_terms,alias_terms,structured_terms,definition_terms) SELECT 'delete',unit_id,identity_terms,alias_terms,structured_terms,definition_terms FROM lexical_units WHERE unit_id=1").unwrap();
    assert!(validate_artifact(path).is_err());
    let mut input = fixture(true);
    input.semantic_units.retain(|u| u.field.is_none());
    let directory = tempfile::tempdir().unwrap();
    assert!(
        crate::test_support::publish_input(directory.path().join("missing.sqlite"), &input)
            .is_err()
    );
    let mut input = fixture(true);
    let unit = input
        .semantic_units
        .iter_mut()
        .find(|u| u.field.is_some())
        .unwrap();
    if let SourcePassageAddress::HtmlSection { chunk_bytes, .. } = &mut unit.address {
        chunk_bytes.end -= 1;
    }
    assert!(
        crate::test_support::publish_input(directory.path().join("tail.sqlite"), &input).is_err()
    );
}

#[test]
fn reference_markers_and_sidecars_are_one_to_one_at_publication_and_selected_read() {
    use atlas_record::source_record::SourceReferenceIndex;
    let mut input = fixture(false);
    input.records[0] = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Dragon Guard","system":{"details":{"publicNotes":"<p>@UUID[Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb]{Staff} @UUID[Compendium.pf2e.macros.Macro.dddddddddddddddd]{Tool} <a href='javascript:alert(1)'>Blocked</a> <a href='https://example.com'>External</a> @UUID[unresolved]</p>"}}}),
    );
    input.records[0].preparation_context_hash = input.context.preparation_sha256().unwrap();
    let mut resolver = SourceReferenceIndex::default();
    for r in &input.records {
        resolver.insert_source(r.record.key(), r.record.source());
    }
    input.records[0].content =
        prepare_record_content(&input.records[0].record, audience(), None, Some(&resolver));
    synchronize_fixture_terms(&mut input);
    let locator = input.records[0].content[0].locator().clone();
    for sql in [
        "DELETE FROM relationship_occurrences WHERE record_id=1 AND origin='content' AND ordinal=0",
        "UPDATE relationship_occurrences SET target_record_id=3 WHERE record_id=1 AND origin='content' AND ordinal=0",
        "UPDATE relationship_occurrences SET ordinal=9 WHERE record_id=1 AND origin='content' AND ordinal=0",
        "UPDATE relationship_occurrences SET authored_target='different authored target' WHERE record_id=1 AND origin='content' AND ordinal=0",
        "UPDATE relationship_occurrences SET occurrence_path='different path' WHERE record_id=1 AND origin='content' AND ordinal=0",
        "INSERT INTO relationship_occurrences(record_id,owners_json,field_path,ordinal,origin,kind,authored_target,occurrence_path,details_json,resolution,target_record_id,target_owners_json,target_url) SELECT record_id,owners_json,field_path,ordinal,origin,kind,authored_target,occurrence_path,details_json,resolution,target_record_id,target_owners_json,target_url FROM relationship_occurrences WHERE record_id=1 AND origin='content' AND ordinal=0",
    ] {
        let (_directory, path) = build(&input);
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert!(reader.read_content(std::slice::from_ref(&locator)).is_ok());
        let relationships = reader
            .read_relationships(&SourceRelationshipRequest {
                record: locator.record.clone(),
                owners: None,
                field: None,
                direction: SourceRelationshipDirection::Outgoing,
                limit: 10,
            })
            .unwrap();
        assert!(!relationships.occurrences.iter().any(|edge|matches!(&edge.resolution,atlas_record::source_content::ContentReferenceResolution::Resolved(atlas_record::source_content::ContentReferenceTarget::Record{key}) if key.pack().as_str()=="macros")));
        drop(reader);
        assert_eq!(
            Connection::open(&path).unwrap().execute(sql, []).unwrap(),
            1,
            "{sql}"
        );
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert!(
            validate_artifact(&path).is_err(),
            "full validation accepted {sql}"
        );
        assert!(
            reader.read_content(std::slice::from_ref(&locator)).is_err(),
            "selected read accepted {sql}"
        );
        assert!(
            reader.read_content_for_record(&locator.record, 10).is_err(),
            "record content accepted {sql}"
        );
    }
    for (from, to) in [
        ("https://example.com", "https://different.example"),
        (
            "data-atlas-reference=\"0\"",
            "href=\"https://example.com\" data-atlas-reference=\"0\"",
        ),
        (
            "data-atlas-reference=\"4\"",
            "href=\"https://example.com\" data-atlas-reference=\"4\"",
        ),
    ] {
        let (_directory, path) = build(&input);
        let c = Connection::open(&path).unwrap();
        let compressed: Vec<u8> = c
            .query_row(
                "SELECT html_gzip FROM prepared_content WHERE record_id=1 AND outcome='prepared'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let html = String::from_utf8(crate::codec::gunzip(&compressed).unwrap()).unwrap();
        let changed = html.replace(from, to);
        assert_ne!(html, changed, "{from}");
        c.execute(
            "UPDATE prepared_content SET html_gzip=? WHERE record_id=1",
            [crate::codec::gzip(changed.as_bytes()).unwrap()],
        )
        .unwrap();
        assert!(validate_artifact(&path).is_err(), "{from}");
        assert!(
            crate::test_support::open_reader(&path)
                .unwrap()
                .read_content(std::slice::from_ref(&locator))
                .is_err(),
            "{from}"
        );
    }
    let (_directory, path) = build(&input);
    let c = Connection::open(&path).unwrap();
    let compressed: Vec<u8> = c
        .query_row(
            "SELECT html_gzip FROM prepared_content WHERE record_id=1 AND outcome='prepared'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let html = crate::codec::gunzip(&compressed).unwrap();
    let duplicated = [html.as_slice(), html.as_slice()].concat();
    c.execute(
        "UPDATE prepared_content SET html_gzip=? WHERE record_id=1",
        [crate::codec::gzip(&duplicated).unwrap()],
    )
    .unwrap();
    assert!(validate_artifact(&path).is_err());
    assert!(
        crate::test_support::open_reader(&path)
            .unwrap()
            .read_content(std::slice::from_ref(&locator))
            .is_err()
    );

    let (_directory, path) = build(&input);
    let original = validate_artifact(&path).unwrap();
    if let atlas_record::source_record::SourceContentStatus::Prepared(prepared) =
        &mut input.records[0].content[0].status
    {
        prepared.references.remove(0);
    } else {
        panic!("fixture expected prepared references");
    }
    assert!(crate::test_support::publish_input(&path, &input).is_err());
    assert_eq!(validate_artifact(&path).unwrap(), original);
}
#[test]
fn shape_and_numeric_sql_constraints_reject_text_and_incoherent_states() {
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let c = Connection::open(&path).unwrap();
    for sql in [
        "UPDATE actor_projection SET hp_maximum='80' WHERE record_id=1",
        "UPDATE actor_projection SET hp_maximum=NULL WHERE record_id=1",
        "UPDATE records SET level_state='missing' WHERE record_id=1",
        "UPDATE spell_projection SET root_record_id=NULL WHERE root_record_id=3",
    ] {
        assert!(c.execute_batch(sql).is_err(), "{sql}");
    }
    assert_eq!(
        crate::test_support::open_reader(path)
            .unwrap()
            .statistics()
            .unwrap()
            .records,
        4
    );
}

#[test]
fn world_table_result_cannot_resolve_into_the_compendium_namespace() {
    use atlas_record::source_record::{SourceReferenceIndex, resolve_source_relationships};
    let mut input = fixture(false);
    input.records.push(record("tables","RollTable",json!({"_id":"tableaaaaaaaaaaa","name":"World Table","results":[{"_id":"resultaaaaaaaaaa","type":"document","documentUuid":"Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb"},{"_id":"resultbbbbbbbbbb","type":"document","documentUuid":"pf2e.items.bbbbbbbbbbbbbbbb"}]})));
    input.packs.push(IndexBuildPack {
        pack_id: "tables".into(),
        label: "Tables".into(),
        document_kind: "RollTable".into(),
        declared_path: "packs/tables".into(),
    });
    input.context.source_file_count = input.records.len();
    let hash = input.context.preparation_sha256().unwrap();
    let mut resolver = SourceReferenceIndex::default();
    for record in &mut input.records {
        record.preparation_context_hash = hash.clone();
        resolver.insert_source(record.record.key(), record.record.source());
    }
    let table = &mut input.records[4];
    table.relationships = resolve_source_relationships(&table.record, Some(&resolver));
    assert!(matches!(
        table.relationships[0].resolution,
        atlas_record::source_content::ContentReferenceResolution::Resolved(_)
    ));
    assert_eq!(
        table.relationships[1].resolution,
        atlas_record::source_content::ContentReferenceResolution::Unresolved
    );
    input.lexical_units.push(SourceLexicalUnitInput {
        record: table.record.key().clone(),
        owners: vec![],
        field: None,
        address: None,
        kind: SourceLexicalUnitKind::RootName,
        identity_terms: String::new(),
        alias_terms: String::new(),
        structured_terms: String::new(),
        definition_terms: String::new(),
    });
    synchronize_fixture_terms(&mut input);
    for ordinal in [0, 1] {
        let (_directory, path) = build(&input);
        let connection = Connection::open(&path).unwrap();
        assert_eq!(connection.execute("UPDATE relationship_occurrences SET resolution='resolved_record',target_record_id=3 WHERE record_id=5 AND ordinal=?",[ordinal]).unwrap(),1);
        assert!(
            validate_artifact(&path).is_err(),
            "Pack/Document ordinal {ordinal}"
        );
    }
}

#[test]
fn offline_reference_reconstruction_does_not_promote_excluded_name_edges() {
    use atlas_record::source_content::ContentReferenceResolution;
    use atlas_record::source_record::{SourceReferenceIndex, resolve_source_relationships};
    let mut input = fixture(false);
    input.records[0] = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Dragon Guard","_stats":{"compendiumSource":"Compendium.pf2e.actors.Actor.Dragon Guard"}}),
    );
    input.records[0].preparation_context_hash = input.context.preparation_sha256().unwrap();
    let mut resolver = SourceReferenceIndex::default();
    for record in &input.records {
        resolver.insert_source(record.record.key(), record.record.source());
    }
    let before = resolve_source_relationships(&input.records[0].record, Some(&resolver));
    assert!(
        matches!(
            before[0].resolution,
            ContentReferenceResolution::Resolved(_)
        ),
        "{before:?}"
    );
    // An unaddressable, excluded source name can have made this alias ambiguous
    // during the original build. The artifact stores the unresolved decision,
    // but deliberately does not store the excluded source or full manifest.
    resolver.exclude_root_name("actors", "Actor", "Dragon Guard");
    input.records[0].relationships =
        resolve_source_relationships(&input.records[0].record, Some(&resolver));
    assert_eq!(
        input.records[0].relationships[0].resolution,
        ContentReferenceResolution::Unresolved
    );
    synchronize_fixture_terms(&mut input);
    let (_directory, path) = build(&input);
    assert!(validate_artifact(path).is_ok());
}
#[test]
fn missing_vectors_and_wrong_model_identity_cannot_be_published() {
    let mut input = fixture(false);
    let directory = tempfile::tempdir().unwrap();
    input.semantic_units = fixture(true).semantic_units;
    assert!(
        crate::test_support::publish_input(directory.path().join("bad.sqlite"), &input).is_err()
    );
    let mut input = fixture(true);
    input.context.search_profiles.first_mut().unwrap().revision = "other".into();
    let hash = input.context.preparation_sha256().unwrap();
    for r in &mut input.records {
        r.preparation_context_hash = hash.clone();
    }
    assert!(
        crate::test_support::publish_input(directory.path().join("bad.sqlite"), &input).is_err()
    );
    let input = fixture(true);
    let (_directory, path) = build(&input);
    let c = Connection::open(&path).unwrap();
    c.execute(
        &format!(
            "DELETE FROM {} WHERE unit_id=1",
            crate::schema::vector_table(&input.context.search_profiles[0]).unwrap()
        ),
        [],
    )
    .unwrap();
    assert!(validate_artifact(path).is_err());
}
#[test]
fn explicit_alias_and_remaster_evidence_are_collision_preserving() {
    let mut input = fixture(false);
    input.aliases = vec![
        SourceAliasInput {
            record: input.records[1].record.key().clone(),
            alias: "Old Name".into(),
            evidence: "verified migration declaration".into(),
        },
        SourceAliasInput {
            record: input.records[2].record.key().clone(),
            alias: "Old Name".into(),
            evidence: "separate declaration".into(),
        },
    ];
    input.remaster_pairs = vec![SourceRemasterPairInput {
        legacy: input.records[1].record.key().clone(),
        remaster: input.records[2].record.key().clone(),
        evidence: "typed samefamily reference fixture".into(),
    }];
    synchronize_fixture_terms(&mut input);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(path).unwrap();
    let matches = reader.lookup_name_or_alias(" OLD  NAME ").unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(
        matches[0].evidence.as_deref(),
        Some("verified migration declaration")
    );
    assert!(
        reader
            .matched_remaster_pairs(std::slice::from_ref(input.records[1].record.key()))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        reader
            .read_remaster_links(input.records[1].record.key())
            .unwrap()
            .len(),
        1
    );
    let all = validate_query(&QueryPredicate::boolean(true)).unwrap();
    assert_eq!(
        reader
            .eligible_keys(&all, None, true, atlas_domain::ResultSort::NameAsc, 0, 100)
            .unwrap()
            .total,
        2
    );
    assert_eq!(
        reader
            .eligible_keys(
                &all,
                Some(std::slice::from_ref(input.records[1].record.key())),
                true,
                atlas_domain::ResultSort::NameAsc,
                0,
                100
            )
            .unwrap()
            .total,
        1
    );
}
#[test]
fn source_exclusion_is_inside_knn_and_lexical_labels_keep_source_kind() {
    let input = fixture(true);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(path).unwrap();
    let all = validate_query(&QueryPredicate::boolean(true)).unwrap();
    let hits = reader
        .vector_candidates_excluding(
            &atlas_domain::SearchProfileId::new("bge-base"),
            &input.semantic_units[0].vector,
            &all,
            None,
            input.records[0].record.key(),
            1,
        )
        .unwrap();
    assert!(!hits.is_empty());
    assert!(
        hits.iter()
            .all(|h| h.location.record != *input.records[0].record.key())
    );
    let hits = reader.lexical_root_candidates("Flame", &all, None).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(
        hits[0]
            .witnesses
            .iter()
            .any(|w| w.exact_label && w.unit_kind == SourceLexicalUnitKind::Heading)
    );
}
#[test]
fn migration_round_trip_matches_canonical_ddl() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(include_str!(
        "../migrations/00000000000001_create_artifact/up.sql"
    ))
    .unwrap();
    let ordinary:usize=c.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'lexical_fts%'",[],|r|r.get(0)).unwrap();
    assert_eq!(ordinary, 41);
    c.execute_batch(include_str!(
        "../migrations/00000000000001_create_artifact/down.sql"
    ))
    .unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |r| r.get::<_, usize>(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn relationships_keep_source_owner_and_validate_owned_targets() {
    use atlas_record::source_content::{
        ContentReferenceResolution, ContentReferenceTarget, OwnedContentIdentity,
        OwnedContentLocator,
    };
    use atlas_record::source_record::{SourceReferenceIndex, resolve_source_relationships};
    let mut input = fixture(false);
    input.records[0] = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Dragon Guard","type":"npc","items":[{"_id":"eeeeeeeeeeeeeeee","name":"Owned Fire","type":"feat","flags":{"pf2e":{"itemGrants":{"ice":{"id":"ffffffffffffffff","onDelete":"detach"}}}},"system":{"description":{"value":"<p>@UUID[Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb]</p>"}}},{"_id":"ffffffffffffffff","name":"Owned Ice","type":"feat","flags":{"pf2e":{"grantedBy":{"id":"eeeeeeeeeeeeeeee"}}}}]}),
    );
    input.records[0].preparation_context_hash = input.context.preparation_sha256().unwrap();
    let mut references = SourceReferenceIndex::default();
    for r in &input.records {
        references.insert_source(r.record.key(), r.record.source());
    }
    input.records[0].content = prepare_record_content(
        &input.records[0].record,
        audience(),
        None,
        Some(&references),
    );
    input.records[0].relationships =
        resolve_source_relationships(&input.records[0].record, Some(&references));
    for (id, name) in [
        ("eeeeeeeeeeeeeeee", "Owned Fire"),
        ("ffffffffffffffff", "Owned Ice"),
    ] {
        input.lexical_units.push(SourceLexicalUnitInput {
            record: input.records[0].record.key().clone(),
            owners: vec![OwnedContentLocator {
                collection: "/items".into(),
                identity: OwnedContentIdentity::Stable(id.into()),
            }],
            field: None,
            address: None,
            kind: SourceLexicalUnitKind::OwnedName,
            identity_terms: "Dragon Guard".into(),
            alias_terms: String::new(),
            structured_terms: String::new(),
            definition_terms: name.into(),
        });
    }
    synchronize_fixture_terms(&mut input);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let request = SourceRelationshipRequest {
        record: input.records[0].record.key().clone(),
        owners: None,
        field: None,
        direction: SourceRelationshipDirection::Outgoing,
        limit: 100,
    };
    let bundle = reader.read_relationships(&request).unwrap();
    assert_eq!(bundle.occurrences.len(), 3);
    assert!(!bundle.truncated);
    assert!(
        bundle
            .occurrences
            .iter()
            .all(|r| r.locator.owners.len() == 1)
    );
    assert!(bundle.occurrences.iter().any(|r| matches!(
        r.resolution,
        ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode { .. })
    )));
    let incoming = reader
        .read_relationships(&SourceRelationshipRequest {
            record: input.records[1].record.key().clone(),
            owners: None,
            field: None,
            direction: SourceRelationshipDirection::Incoming,
            limit: 100,
        })
        .unwrap();
    assert_eq!(incoming.occurrences.len(), 1);
    assert_eq!(
        incoming.occurrences[0].locator.record,
        *input.records[0].record.key()
    );
    let (_directory, wrong_path) = build(&input);
    let wrong = serde_json::to_string(&vec![OwnedContentLocator {
        collection: "/items".into(),
        identity: OwnedContentIdentity::Stable("ffffffffffffffff".into()),
    }])
    .unwrap();
    assert_eq!(Connection::open(&wrong_path).unwrap().execute("UPDATE relationship_occurrences SET target_owners_json=? WHERE origin='structured' AND kind='GrantedBy'",[wrong]).unwrap(),1);
    assert!(validate_artifact(wrong_path).is_err());

    for sql in [
        "UPDATE relationship_occurrences SET resolution='blocked',target_record_id=NULL,target_owners_json=NULL WHERE origin='structured' AND kind='GrantedBy'",
        "UPDATE relationship_occurrences SET resolution='unverified_url',target_record_id=NULL,target_owners_json=NULL,target_url='https://example.com' WHERE origin='structured' AND kind='GrantedBy'",
    ] {
        let (_directory, path) = build(&input);
        assert_eq!(
            Connection::open(&path).unwrap().execute(sql, []).unwrap(),
            1
        );
        assert!(validate_artifact(path).is_err(), "{sql}");
    }

    let c = Connection::open(&path).unwrap();
    c.execute("UPDATE relationship_occurrences SET target_owners_json='[]' WHERE resolution='resolved_owned'",[]).unwrap();
    // Empty owner-chain cannot pretend that a resolved child is a root.
    assert!(validate_artifact(path).is_err());
}
#[test]
fn failure_outcomes_and_projection_diagnostics_remain_developer_only() {
    use atlas_record::source_record::{SourceContentOutcome, SourceContentStatus};
    let mut input = fixture(false);
    input.records[2] = record(
        "items",
        "Item",
        json!({"_id":"cccccccccccccccc","name":"Dragon Spell","type":"spell","system":{"area":{"value":"30","type":"burst"}}}),
    );
    input.records[2].preparation_context_hash = input.context.preparation_sha256().unwrap();
    let original = std::mem::take(&mut input.records[1].content);
    for outcome in original {
        let locator = outcome.locator().clone();
        input.records[1].content.push(SourceContentOutcome {
            role: outcome.role,
            visibility: outcome.visibility,
            visibility_availability: outcome.visibility_availability,
            status: SourceContentStatus::PreparationFailed {
                locator,
                message: "synthetic preparation failure".into(),
            },
        });
    }
    input.lexical_units.retain(|u| u.field.is_none());
    synchronize_fixture_terms(&mut input);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(path).unwrap();
    let fields = reader
        .read_content_for_record(input.records[1].record.key(), 256)
        .unwrap();
    assert_eq!(fields.fields[0].outcome, "preparation_failed");
    assert!(fields.fields[0].html.is_none());
    let diagnostics = reader
        .inspect_diagnostics(input.records[1].record.key(), 100)
        .unwrap();
    assert!(diagnostics.iter().any(|d| d.code == "preparation_failed"));
    let diagnostics = reader
        .inspect_diagnostics(input.records[2].record.key(), 100)
        .unwrap();
    assert!(diagnostics.iter().any(|d| d.stage == "projection"));
    assert!(
        serde_json::to_value(
            reader
                .read_summary(input.records[2].record.key())
                .unwrap()
                .unwrap()
        )
        .unwrap()
        .get("diagnostics")
        .is_none()
    );
}

#[test]
fn read_counters_measure_batches_without_source_decodes() {
    let input = fixture(false);
    let (_directory, path) = build(&input);
    let reader = crate::test_support::open_reader(path).unwrap();
    let keys = input.records[..3]
        .iter()
        .map(|r| r.record.key().clone())
        .collect::<Vec<_>>();
    let locators = input
        .records
        .iter()
        .flat_map(|r| r.content.iter().map(|c| c.locator().clone()))
        .collect::<Vec<_>>();
    reader.reset_read_metrics();
    reader.read_summaries(&keys).unwrap();
    reader.read_content(&locators).unwrap();
    let measured = reader.read_metrics();
    assert_eq!(measured.sql_statements, 3);
    assert_eq!(measured.summary_batches, 1);
    assert_eq!(measured.prepared_content_batches, 1);
    assert_eq!(measured.source_body_decodes, 0);
    reader.read_source_record(&keys[0]).unwrap();
    assert_eq!(reader.read_metrics().sql_statements, 4);
    assert_eq!(reader.read_metrics().source_body_decodes, 1);
    reader.reset_read_metrics();
    assert_eq!(reader.read_metrics(), SourceReadMetrics::default());
}

#[test]
fn lexical_terms_and_used_trait_labels_are_checked_against_source_policy() {
    for sql in [
        "UPDATE lexical_units SET alias_terms='invented alias' WHERE unit_id=1",
        "UPDATE lexical_units SET structured_terms='invented trait' WHERE unit_id=1",
        "UPDATE lexical_units SET identity_terms='invented owner' WHERE unit_id=2",
        "UPDATE lexical_units SET definition_terms='invented definition' WHERE unit_kind='root_name'",
    ] {
        let input = fixture(false);
        let (_directory, path) = build(&input);
        let c = Connection::open(&path).unwrap();
        c.execute_batch(sql).unwrap();
        assert!(validate_artifact(path).is_err(), "{sql}");
    }
    let mut input = fixture(false);
    input
        .context
        .used_trait_labels
        .insert("fire".into(), "Flame Trait".into());
    let hash = input.context.preparation_sha256().unwrap();
    for r in &mut input.records {
        r.preparation_context_hash = hash.clone();
    }
    synchronize_fixture_terms(&mut input);
    let (_directory, path) = build(&input);
    validate_artifact(path).unwrap();
    input
        .context
        .used_trait_labels
        .insert("".into(), "Broken".into());
    assert!(crate::writer::validate_context(&input.context).is_err());
}

#[test]
fn query_catalog_and_preparation_identity_require_current_artifacts() {
    let current = fixture(false);
    let (_directory, path) = build(&current);
    assert!(crate::test_support::open_reader(&path).is_ok());
    let mut old = current.context.clone();
    old.query_catalog_version = 1;
    assert!(crate::writer::validate_context(&old).is_err());
    old = current.context.clone();
    old.content_interpretation_version = "foundry-content/v4".into();
    assert!(crate::writer::validate_context(&old).is_err());
    let c = Connection::open(&path).unwrap();
    c.execute(
        "DELETE FROM query_field_catalog WHERE field='level_or_rank'",
        [],
    )
    .unwrap();
    assert!(matches!(
        crate::test_support::open_reader(&path),
        Err(IndexError::Unsupported(_))
    ));
}

#[test]
fn incremental_publication_preserves_unchanged_rows_and_saved_references() {
    let input = fixture(true);
    let (_directory, path) = build(&input);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TABLE write_audit(kind TEXT); CREATE TRIGGER audit_source AFTER UPDATE ON records BEGIN INSERT INTO write_audit VALUES('record'); END; CREATE TRIGGER audit_body AFTER INSERT ON record_bodies BEGIN INSERT INTO write_audit VALUES('body'); END; CREATE TRIGGER audit_unit AFTER UPDATE ON semantic_units BEGIN INSERT INTO write_audit VALUES('semantic'); END; CREATE TRIGGER audit_identity AFTER UPDATE ON record_identity BEGIN INSERT INTO write_audit VALUES('identity'); END;").unwrap();
    let removed_key = input.records[1].record.key().to_string();
    let identity: i64 = connection
        .query_row(
            "SELECT record_id FROM records WHERE key=?",
            [&removed_key],
            |r| r.get(0),
        )
        .unwrap();
    connection.execute("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(1,'saved','saved','Saved','now','now')",[]).unwrap();
    connection.execute("INSERT INTO saved_list_items(list_id,record_key,position,record_title_snapshot,added_at,updated_at) VALUES(1,?,0,'Remembered staff','now','now')",[&removed_key]).unwrap();
    drop(connection);
    crate::test_support::publish_input(&path, &input).unwrap();
    let connection = Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM write_audit", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    drop(connection);
    let mut reduced = fixture(true);
    reduced.records.remove(1);
    reduced
        .lexical_units
        .retain(|u| u.record.to_string() != removed_key);
    reduced
        .semantic_units
        .retain(|u| u.record.to_string() != removed_key);
    crate::test_support::publish_input(&path, &reduced).unwrap();
    let connection = Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT availability FROM record_identity WHERE id=?",
                [identity],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "unavailable"
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT record_title_snapshot FROM saved_list_items",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "Remembered staff"
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM write_audit WHERE kind IN ('record','body','semantic')",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    drop(connection);
    crate::test_support::publish_input(&path, &input).unwrap();
    let connection = Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT record_id FROM records WHERE key=?",
                [&removed_key],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        identity
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM saved_list_items", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn persisted_preparation_reconstructs_exact_publication_inputs_without_query_catalog() {
    let mut input = fixture(false);
    input.context.audience.include_gm = false;
    input.records[0] = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Dragon Guard","type":"npc","system":{"details":{"level":{"value":9},"privateNotes":"<p>@UUID[secret] @Check[reflex|dc:99]</p>"},"attributes":{"hp":{"max":80},"ac":{"value":24}},"traits":{"value":["dragon"],"rarity":"rare","size":{"value":"lg"}}}}),
    );
    for record in &mut input.records {
        record.preparation_context_hash = input.context.preparation_sha256().unwrap();
        record.content = prepare_record_content(&record.record, input.context.audience, None, None);
    }
    synchronize_fixture_terms(&mut input);
    let (_directory, path) = build(&input);
    let connection = Connection::open(&path).unwrap();
    let hidden: usize = connection
        .query_row(
            "SELECT count(*) FROM prepared_content WHERE published=0",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        hidden > 0,
        "fixture must exercise hidden preparation metadata"
    );
    connection
        .execute("DELETE FROM query_field_catalog", [])
        .unwrap();
    let restored = crate::read_persisted_inputs(&connection, &input.context).unwrap();
    assert_eq!(restored.len(), input.records.len());
    for original in &input.records {
        let restored = restored
            .iter()
            .find(|record| record.record.key() == original.record.key())
            .unwrap();
        let label = &input
            .packs
            .iter()
            .find(|pack| pack.pack_id == original.record.key().pack().as_str())
            .unwrap()
            .label;
        assert_eq!(
            crate::publication_delta::root_digest(original, label).unwrap(),
            crate::publication_delta::root_digest(restored, label).unwrap(),
            "{}",
            original.record.key()
        );
    }
}

#[test]
fn ammunition_projects_physical_values_for_root_and_embedded_queries() {
    for family in [
        "ammo",
        "armor",
        "backpack",
        "book",
        "consumable",
        "equipment",
        "shield",
        "treasure",
        "weapon",
    ] {
        let input = record(
            "items",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":family,"system":{"price":{"value":{"gp":2},"per":1},"bulk":{"value":1}}}),
        );
        let mut ids = crate::projections::ProjectionIds::default();
        let rows = crate::projections::root_rows(&input, "Items", 1, &mut ids).unwrap();
        let physical = rows
            .iter()
            .find(|row| row.table == "physical_projection")
            .unwrap_or_else(|| panic!("missing physical row for {family}"));
        let column = physical
            .columns
            .iter()
            .position(|name| name == "price_per_item_cp_state")
            .unwrap();
        assert_eq!(
            physical.values[column],
            rusqlite::types::Value::Text("value".into()),
            "{family}"
        );
    }
}
