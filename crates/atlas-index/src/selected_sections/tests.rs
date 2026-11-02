//! Migration and bounded-decoder oracles use synthetic values, no source or models.
use super::*;
use crate::{test_support::*, *};
use atlas_domain::{QueryPredicate, SearchContentScope, SourceByteRange};
use atlas_storage::Workspace;
use rusqlite::Connection;
use std::path::PathBuf;

fn historical(input: &IndexBuildInput) -> (tempfile::TempDir, PathBuf, Workspace) {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sections.sqlite");
    let workspace = Workspace::at(&path).unwrap();
    workspace
        .migrate(&TEST_MIGRATIONS[..26], |_| Ok(()))
        .unwrap();
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner
        .maintenance(26, "authentic pre-section fixture")
        .unwrap();
    publish_pre_section_fixture(&maintenance, input).unwrap();
    drop(maintenance);
    drop(owner);
    let write = workspace.write(26).unwrap();
    atlas_local_state::LocalStateWrite::new(&write)
        .saved_lists()
        .create(atlas_local_state::NewSavedList {
            slug: "canonical-kept".into(),
            name: "Retained actual saved list".into(),
            description: Some("Retained user description".into()),
            tags: vec!["kept".into()],
        })
        .unwrap();
    write.commit().unwrap();
    (directory, path, workspace)
}

fn digest(c: &Connection, tables: &[&str]) -> String {
    let mut bytes = Vec::new();
    for table in tables {
        bytes.extend_from_slice(table.as_bytes());
        let mut statement = c
            .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY 1"))
            .unwrap();
        let columns = statement.column_count();
        let mut rows = statement.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for i in 0..columns {
                let value = format!("{:?}", row.get_ref(i).unwrap());
                bytes.extend_from_slice(&value.len().to_le_bytes());
                bytes.extend_from_slice(value.as_bytes());
            }
        }
    }
    crate::codec::sha256(&bytes)
}
fn saved(c: &Connection) -> String {
    let tables=c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND (name LIKE 'saved_%' OR name LIKE 'encounter%' OR name LIKE 'party%' OR name='parties') ORDER BY name").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    digest(c, &tables.iter().map(String::as_str).collect::<Vec<_>>())
}
fn numerical(c: &Connection, _: &IndexBuildInput) -> String {
    let mut output = Vec::new();
    for (name, sql) in [
        (
            "semantic_units",
            "SELECT * FROM semantic_units ORDER BY unit_id",
        ),
        (
            "embedding_models",
            "SELECT * FROM embedding_models ORDER BY model_id",
        ),
        (
            "embedding_inputs",
            "SELECT * FROM embedding_inputs ORDER BY input_id",
        ),
        (
            "pending_embedding_values",
            "SELECT input_id,vector,vector_hash FROM pending_embedding_values ORDER BY input_id",
        ),
        (
            "trait_vectors",
            "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id",
        ),
        (
            "trait_seed_chunks",
            "SELECT * FROM trait_seed_chunks ORDER BY trait_key,profile_id,ordinal",
        ),
    ] {
        output.extend_from_slice(name.as_bytes());
        let mut statement = c.prepare(sql).unwrap();
        let count = statement.column_count();
        let mut rows = statement.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for column in 0..count {
                let value = format!("{:?}", row.get_ref(column).unwrap());
                output.extend_from_slice(&value.len().to_le_bytes());
                output.extend_from_slice(value.as_bytes());
            }
        }
    }
    output.extend_from_slice(format!("{:?}", native_preservation_rows(c).unwrap()).as_bytes());
    crate::codec::sha256(&output)
}

fn sample() -> IndexBuildInput {
    crate::core_tests::fixture(
        vec![
            (
                "actors",
                "Actor",
                serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Creature","type":"npc","system":{"details":{"publicNotes":"<h2>Origin</h2><p>Root source prose.</p>","privateNotes":"<p>Unlabeled private prose.</p>","blurb":"Plain brief source prose"}},"items":[{"_id":"cccccccccccccccc","name":"Carried","type":"consumable","system":{"description":{"value":"<p>Owned unlabeled source prose.</p>"}}}]}),
            ),
            (
                "items",
                "Item",
                serde_json::json!({"_id":"bbbbbbbbbbbbbbbb","name":"Item","type":"equipment","system":{"description":{"value":"<h2>Heading</h2><p>One two three four five six seven eight nine ten eleven twelve.</p>"}}}),
            ),
            (
                "items",
                "Item",
                serde_json::json!({"_id":"dddddddddddddddd","name":"Empty","type":"effect","system":{"description":{"value":""}}}),
            ),
        ],
        true,
    )
}
#[test]
fn missing_marker_roundtrips_old_context_and_does_not_change_preparation_or_selector() {
    let input = sample();
    let mut old = input.context.clone();
    old.selected_sections_recipe.clear();
    let json = serde_json::to_string(&old).unwrap();
    assert!(!json.contains("selected_sections_recipe"));
    let decoded: SourceArtifactBuildContext = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&decoded).unwrap(), json);
    assert_eq!(
        decoded.sha256().unwrap(),
        crate::codec::sha256(json.as_bytes())
    );
    assert_ne!(decoded.sha256().unwrap(), input.context.sha256().unwrap());
    assert_eq!(
        decoded.preparation_sha256().unwrap(),
        input.context.preparation_sha256().unwrap()
    );
    for source in &input.records {
        let selection = atlas_record::source_record::select_record_search(
            &source.record,
            &source.content,
            old.audience,
            &input
                .packs
                .iter()
                .find(|p| p.pack_id == source.record.key().pack().as_str())
                .unwrap()
                .label,
            &old.used_trait_labels,
        )
        .unwrap();
        let original = project(
            &source.record,
            &selection,
            &source.content_hash,
            &source.preparation_context_hash,
        )
        .unwrap();
        assert!(
            !original
                .iter()
                .any(|s| s.address == SourcePassageAddress::Identity {})
        );
        assert_eq!(
            original,
            project(
                &source.record,
                &selection,
                &source.content_hash,
                &decoded.preparation_sha256().unwrap()
            )
            .unwrap()
        );
    }
}
#[test]
fn compatible_upgrade_preserves_values_saved_state_and_selectors_for_zero_one_and_two_profiles() {
    for profiles in [
        vec![],
        vec!["bge-base"],
        vec!["embeddinggemma"],
        vec!["bge-base", "embeddinggemma"],
    ] {
        let mut input = sample();
        input
            .context
            .search_profiles
            .retain(|p| profiles.contains(&p.profile_id.as_str()));
        input
            .semantic_units
            .retain(|u| profiles.contains(&u.profile_id.as_str()));
        let (_directory, path, workspace) = historical(&input);
        let write = workspace.write(26).unwrap();
        let before = numerical(write.connection(), &input);
        let saved_before = saved(write.connection());
        let selectors = write
            .connection()
            .prepare("SELECT selector FROM similarity_parts ORDER BY selector")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let revision = write.reference_revision().unwrap();
        write.commit().unwrap();
        let upgraded = initialize_workspace(&path).unwrap();
        let read = upgraded.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(read.reference_revision().unwrap(), revision);
        assert_eq!(numerical(read.connection(), &input), before);
        assert_eq!(saved(read.connection()), saved_before);
        assert_eq!(
            classify_reference_upgrade(read.connection()).unwrap(),
            ReferenceUpgrade::SectionProjections
        );
        assert!(
            read.connection()
                .prepare("SELECT * FROM similarity_parts")
                .is_err()
        );
        assert_eq!(read.connection().query_row("SELECT count(*) FROM selected_sections WHERE record_id=(SELECT record_id FROM records WHERE name='Empty')",[],|r|r.get::<_,usize>(0)).unwrap(),0);
        drop(read);
        assert!(ensure_reference_projections(&upgraded, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
        let read = upgraded.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(read.reference_revision().unwrap(), revision + 1);
        assert_eq!(numerical(read.connection(), &input), before);
        assert_eq!(saved(read.connection()), saved_before);
        crate::validate_reference(read.connection()).unwrap();
        let reader = SqliteIndexReader::from_session(read).unwrap();
        let mut actual_selectors = Vec::new();
        let mut actual = std::collections::BTreeMap::new();
        for source in &input.records {
            let addresses = reader
                .similarity_part_addresses(source.record.key())
                .unwrap();
            actual_selectors.extend(addresses.keys().map(|selector| selector.0.clone()));
            let parts = reader
                .similarity_parts(
                    &atlas_domain::SimilaritySeed::Record {
                        record_key: source.record.key().clone(),
                    },
                    None,
                )
                .unwrap();
            for component in parts.parts {
                if let atlas_domain::SimilarityComponent::Section { part } = component {
                    assert!(
                        actual
                            .insert(
                                (source.record.key().to_string(), part.field),
                                (part.owners, addresses[&part.selector].clone()),
                            )
                            .is_none()
                    );
                }
            }
        }
        actual_selectors.sort();
        assert_eq!(actual_selectors, selectors);
        let mut expected = std::collections::BTreeMap::new();
        for (record, field, owned, html, label, text) in [
            (
                "actors:aaaaaaaaaaaaaaaa",
                "/system/details/blurb",
                false,
                None,
                None,
                "Plain brief source prose",
            ),
            (
                "actors:aaaaaaaaaaaaaaaa",
                "/system/details/publicNotes",
                false,
                Some("<h2>Origin</h2><p>Root source prose.</p>"),
                Some("Origin"),
                "Origin\n\nRoot source prose.",
            ),
            (
                "actors:aaaaaaaaaaaaaaaa",
                "/system/details/privateNotes",
                false,
                Some("<p>Unlabeled private prose.</p>"),
                None,
                "Unlabeled private prose.",
            ),
            (
                "actors:aaaaaaaaaaaaaaaa",
                "/system/description/value",
                true,
                Some("<p>Owned unlabeled source prose.</p>"),
                None,
                "Owned unlabeled source prose.",
            ),
            (
                "items:bbbbbbbbbbbbbbbb",
                "/system/description/value",
                false,
                Some(
                    "<h2>Heading</h2><p>One two three four five six seven eight nine ten eleven twelve.</p>",
                ),
                Some("Heading"),
                "Heading\n\nOne two three four five six seven eight nine ten eleven twelve.",
            ),
        ] {
            let hash = crate::codec::sha256(text.as_bytes());
            let range = SourceByteRange {
                start: 0,
                end: text.len(),
            };
            let address = match html {
                Some(html) => SourcePassageAddress::HtmlSection {
                    prepared_html_sha256: crate::codec::sha256(html.as_bytes()),
                    canonical_text_sha256: hash,
                    selection_version: "structural-source-sections/v4".into(),
                    section_ordinal: 0,
                    label: label.map(str::to_owned),
                    chunk_bytes: range,
                },
                None => SourcePassageAddress::PlainSection {
                    source_text_sha256: hash,
                    selection_version: "structural-source-sections/v4".into(),
                    section_ordinal: 0,
                    label: None,
                    chunk_bytes: range,
                },
            };
            let owners = if owned {
                vec![atlas_domain::OwnedContentLocator {
                    collection: "/items".into(),
                    identity: atlas_domain::OwnedContentIdentity::Stable("cccccccccccccccc".into()),
                }]
            } else {
                vec![]
            };
            expected.insert((record.to_owned(), field.to_owned()), (owners, address));
        }
        assert_eq!(actual, expected);
        drop(reader);
        assert!(!ensure_reference_projections(&upgraded, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
    }
}
#[test]
fn profile_specific_ranges_share_whole_sections_without_collapsing_locations() {
    let mut input = sample();
    let gemma = atlas_domain::SearchProfileId::new("embeddinggemma");
    input
        .context
        .search_profiles
        .iter_mut()
        .find(|p| p.profile_id == gemma)
        .unwrap()
        .input_recipe = "retained-profile-layout/v1".into();
    let old = std::mem::take(&mut input.semantic_units);
    for unit in old {
        if unit.profile_id != gemma || matches!(unit.address, SourcePassageAddress::Identity {}) {
            input.semantic_units.push(unit);
            continue;
        }
        let length = range_mut(&mut unit.address.clone()).unwrap().end;
        let middle = length / 2;
        for (ordinal, range) in [
            (
                0,
                SourceByteRange {
                    start: 0,
                    end: middle + 2,
                },
            ),
            (
                1,
                SourceByteRange {
                    start: middle,
                    end: length,
                },
            ),
        ] {
            let mut chunk = unit.clone();
            *range_mut(&mut chunk.address).unwrap() = range;
            chunk.chunk_ordinal = ordinal;
            chunk.input_hash =
                crate::codec::sha256(&serde_json::to_vec(&(&unit.input_hash, range)).unwrap());
            input.semantic_units.push(chunk);
        }
    }
    let (_directory, path, workspace) = historical(&input);
    let read = workspace.read(26).unwrap();
    let before = numerical(read.connection(), &input);
    drop(read);
    let upgraded = initialize_workspace(&path).unwrap();
    assert!(ensure_reference_projections(&upgraded, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
    let read = upgraded.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(numerical(read.connection(), &input), before);
    assert!(read.connection().query_row("SELECT EXISTS(SELECT 1 FROM semantic_locations WHERE section_id IS NOT NULL GROUP BY section_id,chunk_ordinal HAVING count(*)>1)",[],|r|r.get::<_,bool>(0)).unwrap());
    let reader = SqliteIndexReader::from_session(read).unwrap();
    for profile in &input.context.search_profiles {
        let keys = input
            .records
            .iter()
            .map(|r| r.record.key().clone())
            .collect::<Vec<_>>();
        let mut actual = reader
            .similarity_units(
                &profile.profile_id,
                &keys,
                SearchContentScope::All,
                true,
                1000,
            )
            .unwrap()
            .into_iter()
            .map(|u| serde_json::to_string(&u.location.unwrap().address.unwrap()).unwrap())
            .collect::<Vec<_>>();
        let mut expected = input
            .semantic_units
            .iter()
            .filter(|u| u.profile_id == profile.profile_id)
            .map(|u| serde_json::to_string(&u.address).unwrap())
            .collect::<Vec<_>>();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
    }
}
#[test]
fn retired_preparation_and_predecessor_never_decode_snapshots_or_alter_pending_values() {
    for replacement in [false, true] {
        let mut input = sample();
        let current = input.context.clone();
        if replacement {
            input.context.snapshot_version = 1;
            input.context.source_contract =
                "6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8".into();
        } else {
            input.context.relationship_policy = "typed-authored-occurrences/v6".into();
        }
        let retired_preparation = input.context.preparation_sha256().unwrap();
        for source in &mut input.records {
            source.preparation_context_hash = retired_preparation.clone();
        }
        let (_directory, path, workspace) = historical(&input);
        let write = workspace.write(26).unwrap();
        let c = write.connection();
        c.execute("UPDATE record_bodies SET snapshot=x'00'", [])
            .unwrap();
        c.execute_batch("CREATE TABLE saved_probe(id INTEGER PRIMARY KEY,note TEXT); INSERT INTO saved_probe VALUES(1,'survives retired source');").unwrap();
        let model = &input.context.search_profiles[0];
        let unit = &input.semantic_units[0];
        let prepared = atlas_embedding::PreparedNativeInput {
            input: String::new(),
            input_sha256: unit.input_hash.clone(),
            token_count: unit.input_token_count,
        };
        let mut forced = vec![0.; model.dimensions];
        forced[31] = 1.;
        stage_historical_pending_native_fixture(
            &write,
            "historical-forced-retry",
            model,
            &prepared.descriptor(),
            &forced,
        )
        .unwrap();
        let before = numerical(c, &input);
        let saved_before = saved(c);
        let revision = write.reference_revision().unwrap();
        write.commit().unwrap();
        let upgraded = initialize_workspace(&path).unwrap();
        let read = upgraded.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(numerical(read.connection(), &input), before);
        assert_eq!(saved(read.connection()), saved_before);
        assert_eq!(read.reference_revision().unwrap(), revision);
        assert_eq!(
            classify_reference_upgrade(read.connection()).unwrap(),
            if replacement {
                ReferenceUpgrade::SourceReplacement
            } else {
                ReferenceUpgrade::SourcePreparation
            }
        );
        assert!(read_reusable_passage_layouts(&read).unwrap().is_empty());
        assert!(read.connection().query_row("SELECT EXISTS(SELECT 1 FROM semantic_locations WHERE section_id IS NULL AND chunk_ordinal IS NULL)",[],|r|r.get::<_,bool>(0)).unwrap());
        assert!(SqliteIndexReader::from_session(read).is_err());
        let owner = upgraded.update_owner().unwrap();
        drop(
            owner
                .maintenance(
                    TEST_WORKSPACE_SCHEMA_VERSION,
                    "interrupted retired publication",
                )
                .unwrap(),
        );
        assert!(recover_reference_publication(&owner, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
        drop(owner);
        let read = upgraded.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(read.reference_revision().unwrap(), revision);
        assert_eq!(numerical(read.connection(), &input), before);
        assert_eq!(saved(read.connection()), saved_before);
        assert!(SqliteIndexReader::from_session(read).is_err());
        assert!(!ensure_reference_projections(&upgraded, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
        // An ordinary source-backed publication must resolve every placeholder.
        input.context = current;
        let current_preparation = input.context.preparation_sha256().unwrap();
        for source in &mut input.records {
            source.preparation_context_hash = current_preparation.clone();
        }
        publish_input(&path, &input).unwrap();
        let read = upgraded.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        crate::validate_reference(read.connection()).unwrap();
        assert!(
            !read
                .connection()
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM semantic_locations WHERE chunk_ordinal IS NULL)",
                    [],
                    |r| r.get::<_, bool>(0)
                )
                .unwrap()
        );
        assert_eq!(saved(read.connection()), saved_before);
    }
}
#[test]
fn corrupt_historical_ranges_roll_back_schema_ledger_and_native_values() {
    for corruption in ["checksum", "range", "owner", "lexical whole extent"] {
        let input = sample();
        let (_directory, path, workspace) = historical(&input);
        let write = workspace.write(26).unwrap();
        let c = write.connection();
        if corruption == "lexical whole extent" {
            let (unit, json):(i64,String)=c.query_row("SELECT unit_id,section_json FROM lexical_units WHERE unit_kind='heading' LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
            let mut address: SourcePassageAddress = serde_json::from_str(&json).unwrap();
            range_mut(&mut address).unwrap().end -= 1;
            c.execute(
                "UPDATE lexical_units SET section_json=? WHERE unit_id=?",
                rusqlite::params![serde_json::to_string(&address).unwrap(), unit],
            )
            .unwrap();
        } else {
            let (location,record,mut owners,field,mut json,ordinal):(i64,i64,String,Option<String>,String,usize)=c.query_row("SELECT l.location_id,l.record_id,l.owners_json,l.field_path,l.section_json,l.chunk_ordinal FROM semantic_locations l JOIN semantic_units u USING(location_id) WHERE u.unit_kind='passage' LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).unwrap();
            if corruption == "range" {
                let mut address: SourcePassageAddress = serde_json::from_str(&json).unwrap();
                range_mut(&mut address).unwrap().end += 100;
                json = serde_json::to_string(&address).unwrap();
            }
            if corruption == "owner" {
                owners = serde_json::to_string(&vec![atlas_domain::OwnedContentLocator {
                    collection: "/items".into(),
                    identity: atlas_domain::OwnedContentIdentity::Stable("ffffffffffffffff".into()),
                }])
                .unwrap();
            }
            let checksum = if corruption == "checksum" {
                vec![0; 32]
            } else {
                crate::embedding_values::hash_bytes(
                    &serde_json::to_vec(&(record, &owners, &field, &json, ordinal)).unwrap(),
                )
            };
            c.execute("UPDATE semantic_locations SET owners_json=?,section_json=?,location_hash=? WHERE location_id=?",rusqlite::params![owners,json,checksum,location]).unwrap();
        }
        let before = numerical(c, &input);
        let saved_before = saved(c);
        let metadata = digest(
            c,
            &[
                "semantic_locations",
                "similarity_parts",
                "lexical_units",
                "schema_migrations",
            ],
        );
        let revision = write.reference_revision().unwrap();
        write.commit().unwrap();
        assert!(initialize_workspace(&path).is_err(), "{corruption}");
        let read = workspace.read(26).unwrap();
        assert_eq!(read.reference_revision().unwrap(), revision);
        assert_eq!(numerical(read.connection(), &input), before);
        assert_eq!(saved(read.connection()), saved_before);
        assert_eq!(
            digest(
                read.connection(),
                &[
                    "semantic_locations",
                    "similarity_parts",
                    "lexical_units",
                    "schema_migrations"
                ]
            ),
            metadata
        );
        assert!(
            read.connection()
                .prepare("SELECT * FROM selected_sections")
                .is_err()
        );
    }
}

#[test]
fn published_passages_and_neighbor_hydration_reject_unavailable_sections_and_identity_ordinals() {
    for corruption in [
        "missing section",
        "identity ordinal",
        "range",
        "location root",
        "section root",
        "identity section",
    ] {
        let input = sample();
        let (_directory, path) = crate::foundation_tests::build(&input);
        let reader = open_reader(&path).unwrap();
        let query = validate_query(&QueryPredicate::boolean(true)).unwrap();
        let request = SourceVectorCandidateRequest {
            profile: &input.context.search_profiles[0].profile_id,
            vector: &input.semantic_units[0].vector,
            query: &query,
            keys: None,
            exclude: None,
            content_scope: SearchContentScope::All,
            limit: 1000,
        };
        let (_, neighbors) = reader.vector_candidates_with_neighbors(request).unwrap();
        let c = reader.workspace_session().connection();
        c.execute_batch("PRAGMA ignore_check_constraints=ON;")
            .unwrap();
        // TEMP tables shadow only this admitted bounded session. They model
        // corruption without changing publication identity or durable fixtures.
        c.execute_batch(
            "CREATE TEMP TABLE semantic_locations AS SELECT * FROM main.semantic_locations",
        )
        .unwrap();
        match corruption {
            "missing section"=>c.execute_batch("UPDATE temp.semantic_locations SET section_id=NULL,chunk_ordinal=NULL,chunk_start=NULL,chunk_end=NULL WHERE location_id IN(SELECT location_id FROM semantic_units WHERE unit_kind='passage')").unwrap(),
            "identity ordinal"=>c.execute_batch("UPDATE temp.semantic_locations SET chunk_ordinal=1 WHERE location_id IN(SELECT location_id FROM semantic_units WHERE unit_kind='identity')").unwrap(),
            "location root"=>c.execute_batch("UPDATE temp.semantic_locations SET record_id=(SELECT max(record_id) FROM records)+1 WHERE section_id IS NOT NULL").unwrap(),
            "section root"=>c.execute_batch("CREATE TEMP TABLE selected_sections AS SELECT * FROM main.selected_sections; UPDATE temp.selected_sections SET record_id=(SELECT max(record_id) FROM records)+1").unwrap(),
            "identity section"=>c.execute_batch("UPDATE temp.semantic_locations SET section_id=(SELECT min(section_id) FROM selected_sections) WHERE location_id IN(SELECT location_id FROM semantic_units WHERE unit_kind='identity')").unwrap(),
            _=>c.execute_batch("UPDATE temp.semantic_locations SET chunk_end=9223372036854775807 WHERE section_id IS NOT NULL").unwrap()
        }

        assert!(reader.vector_candidates(request).is_err(), "{corruption}");
        assert!(
            reader
                .hydrate_vector_neighbors(request, &neighbors)
                .is_err(),
            "{corruption}"
        );
        let keys = input
            .records
            .iter()
            .map(|r| r.record.key().clone())
            .collect::<Vec<_>>();
        assert!(
            reader
                .similarity_units(request.profile, &keys, SearchContentScope::All, true, 1000)
                .is_err(),
            "{corruption}"
        );
        assert!(
            reader
                .similarity_candidates(SourceSimilarityCandidateRequest {
                    profile: request.profile,
                    vector: request.vector,
                    query: &query,
                    keys: None,
                    exclude: None,
                    comparison_key: None,
                    content_scope: SearchContentScope::All,
                    include_identity: true,
                    limit: 1000
                })
                .is_err(),
            "{corruption}"
        );
        assert!(crate::validate_reference(c).is_err(), "{corruption}");
    }
}
