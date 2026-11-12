//! Bounded-decoder oracles use synthetic values, no source or models.
use super::*;
use crate::{test_support::*, *};
use atlas_domain::{QueryPredicate, SearchContentScope, SourceByteRange};
use atlas_storage::Workspace;
use rusqlite::Connection;

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
fn description_filter_matches_own_readable_text_and_journal_pages() {
    use serde_json::json;

    let input = crate::core_tests::fixture(
        vec![
            (
                "actors",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Described creature","type":"npc","system":{"details":{"publicNotes":"<p>Dwells in forests.</p>"}}}),
            ),
            (
                "actors",
                "Actor",
                json!({"_id":"bbbbbbbbbbbbbbbb","name":"Mechanics only","type":"npc","system":{"details":{"privateNotes":"<p>GM only.</p>"}},"items":[{"_id":"xxxxxxxxxxxxxxxx","name":"Carried","type":"equipment","system":{"description":{"value":"<p>Owned prose.</p>"}}}]}),
            ),
            (
                "actors",
                "Actor",
                json!({"_id":"cccccccccccccccc","name":"Routine only","type":"hazard","system":{"details":{"routine":"<p>Trigger twice.</p>"}}}),
            ),
            (
                "items",
                "Item",
                json!({"_id":"dddddddddddddddd","name":"Described feat","type":"feat","system":{"description":{"value":"<p>Gain a benefit.</p>"}}}),
            ),
            (
                "items",
                "Item",
                json!({"_id":"eeeeeeeeeeeeeeee","name":"Empty effect","type":"effect","system":{"description":{"value":"<p><br></p>"}}}),
            ),
            (
                "items",
                "Item",
                json!({"_id":"hhhhhhhhhhhhhhhh","name":"GM notes only","type":"feat","system":{"description":{"value":"","gm":"<p>Only the GM sees this.</p>"}}}),
            ),
            (
                "journals",
                "JournalEntry",
                json!({"_id":"ffffffffffffffff","name":"Readable journal","pages":[{"_id":"yyyyyyyyyyyyyyyy","name":"Page","type":"text","text":{"format":1,"content":"<p>Local customs.</p>"}}]}),
            ),
            (
                "journals",
                "JournalEntry",
                json!({"_id":"gggggggggggggggg","name":"Empty journal","pages":[]}),
            ),
        ],
        false,
    );
    let (_directory, path) = crate::foundation_tests::build(&input);
    let connection = Connection::open(&path).unwrap();
    let matches = |expression: &str| {
        let query = crate::query::parse_where(expression).unwrap();
        let (sql, parameters) = crate::read_units::eligible_sql(&query, None).unwrap();
        connection
            .prepare(&format!("WITH eligible AS ({sql}) SELECT r.name FROM eligible e JOIN records r USING(record_id) ORDER BY r.name"))
            .unwrap()
            .query_map(rusqlite::params_from_iter(parameters), |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };
    assert_eq!(
        matches("has_description == true"),
        ["Described creature", "Described feat", "Readable journal"]
    );
    assert_eq!(
        matches("has_description == false"),
        [
            "Empty effect",
            "Empty journal",
            "GM notes only",
            "Mechanics only",
            "Routine only"
        ]
    );
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
fn profile_specific_ranges_share_whole_sections_without_collapsing_locations() {
    let mut input = sample();
    let gemma = atlas_domain::SearchProfileId::new("embeddinggemma");
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
    // Different profile ranges are a current projection contract. Historical
    // migration preservation and source-required recovery are covered above.
    let (_directory, path) = crate::foundation_tests::build(&input);
    let workspace = Workspace::at(&path).unwrap();
    assert!(!ensure_reference_projections(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
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
            discovery_scope: Default::default(),
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
                    discovery_scope: Default::default(),
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
