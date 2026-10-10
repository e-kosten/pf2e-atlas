use crate::test_support::{input, record};
use crate::*;
use atlas_domain::{RecordKey, SourcePassageAddress};
use atlas_index::{IndexBuildInput, SourceAliasInput, SourceRemasterPairInput, SqliteIndexReader};
use serde_json::json;
fn effect(id: &str, name: &str, level: i32, html: &str) -> atlas_index::SourceArtifactRecordInput {
    record(
        "items",
        "Item",
        json!({"_id":id,"name":name,"type":"effect","system":{"level":{"value":level},"description":{"value":html},"traits":{"value":[],"rarity":"common"}}}),
    )
}
fn service(input: IndexBuildInput) -> (test_support::FixtureArtifact, AtlasRetrievalService) {
    let (service, artifact) = test_support::open_input_fixture(input).unwrap();
    (artifact, service)
}
fn key(id: &str) -> RecordKey {
    RecordKey::parse(&format!("items:{id}")).unwrap()
}
fn search(service: &mut AtlasRetrievalService, text: &str) -> TextSearchResult {
    service
        .search_text(TextSearchRequest {
            query: text,
            mode: RetrievalMode::Fts,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            prefer_remaster: true,
        })
        .unwrap()
}
#[test]
fn strict_identity_alias_ambiguity_and_macro_exclusion() {
    let mut fixture = input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Dragon", 1, ""),
            effect("bbbbbbbbbbbbbbbb", "Dragon", 2, ""),
            record(
                "macros",
                "Macro",
                json!({"_id":"cccccccccccccccc","name":"Dragon","type":"script","command":""}),
            ),
        ],
        false,
    );
    fixture.aliases.push(SourceAliasInput {
        record: key("aaaaaaaaaaaaaaaa"),
        alias: "Verified dragon".into(),
        evidence: "source redirect".into(),
    });
    fixture
        .lexical_units
        .iter_mut()
        .find(|u| u.record == key("aaaaaaaaaaaaaaaa") && u.owners.is_empty() && u.field.is_none())
        .unwrap()
        .alias_terms = "Verified dragon".into();
    let (_dir, mut service) = service(fixture);
    assert_eq!(
        service
            .resolve_record(ResolveRecordRequest {
                query: " DRAGON ",
                filter: None
            })
            .unwrap()
            .len(),
        2
    );
    assert!(matches!(
        service
            .resolve_record_ref(ResolveRecordRefRequest {
                record_ref: "Dragon",
                filter: None
            })
            .unwrap(),
        RecordRefResolutionResult::Ambiguous(_)
    ));
    let aliases = service
        .resolve_record(ResolveRecordRequest {
            query: "verified dragon",
            filter: None,
        })
        .unwrap();
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].match_kind, RecordResolutionMatchKind::Alias);
    assert!(aliases[0].evidence.is_some());
    assert_eq!(
        search(&mut service, "Verified dragon").records[0]
            .record
            .key,
        key("aaaaaaaaaaaaaaaa")
    );
    assert!(
        service
            .resolve_record(ResolveRecordRequest {
                query: "Dragn",
                filter: None
            })
            .unwrap()
            .is_empty()
    );
    assert_eq!(search(&mut service, "Dragon").records.len(), 2);
    let macro_key = RecordKey::parse("macros:cccccccccccccccc").unwrap();
    assert!(matches!(
        service
            .resolve_record_ref(ResolveRecordRefRequest {
                record_ref: &macro_key.to_string(),
                filter: None,
            })
            .unwrap(),
        RecordRefResolutionResult::Miss
    ));
    assert!(
        service
            .get_record(GetRecordRequest {
                record_key: &macro_key,
                selected_content: &[]
            })
            .unwrap()
            .is_none()
    );
    assert!(
        service
            .similar_records(SimilarRecordRequest {
                record_key: &macro_key,
                filter: None,
                scope: RecordScope::All,
                page: SearchPage::default(),
                prefer_remaster: true
            })
            .unwrap()
            .is_none()
    );
    assert!(
        service
            .graph_context(GraphContextRequest::new(macro_key))
            .unwrap()
            .is_none()
    );
}
#[test]
fn exact_key_resolution_obeys_filters_without_suppressing_legacy_identity() {
    let legacy = key("aaaaaaaaaaaaaaaa");
    let mut fixture = input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Legacy flame", 1, ""),
            effect("bbbbbbbbbbbbbbbb", "Remaster flame", 2, ""),
        ],
        false,
    );
    fixture.remaster_pairs.push(SourceRemasterPairInput {
        legacy: legacy.clone(),
        remaster: key("bbbbbbbbbbbbbbbb"),
        evidence: "verified source relation".into(),
    });
    let (_dir, service) = service(fixture);
    let legacy_ref = legacy.to_string();
    for (expression, accepted) in [("item.level >= 1", true), ("item.level == 2", false)] {
        let filter = service.parse_where(expression).unwrap();
        let result = service
            .resolve_record_ref(ResolveRecordRefRequest {
                record_ref: &legacy_ref,
                filter: Some(filter.predicate()),
            })
            .unwrap();
        if accepted {
            assert!(matches!(result, RecordRefResolutionResult::Key(key) if key == legacy));
        } else {
            assert!(matches!(result, RecordRefResolutionResult::Miss));
        }
    }
    let invalid = atlas_domain::QueryPredicate::new(atlas_domain::QueryExpression::StateMatch {
        field: "nonexistent.field".into(),
        state: atlas_domain::QueryFieldState::Value,
    });
    for record_ref in [legacy_ref.as_str(), "items:cccccccccccccccc"] {
        assert!(
            service
                .resolve_record_ref(ResolveRecordRefRequest {
                    record_ref,
                    filter: Some(&invalid),
                })
                .is_err()
        );
    }
}
#[test]
fn pair_preference_is_conditional_and_precedes_paging() {
    let mut fixture = input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Legacy flame", 1, ""),
            effect("bbbbbbbbbbbbbbbb", "Remaster flame", 2, ""),
            effect("cccccccccccccccc", "Other flame", 3, ""),
        ],
        false,
    );
    fixture.remaster_pairs.push(SourceRemasterPairInput {
        legacy: key("aaaaaaaaaaaaaaaa"),
        remaster: key("bbbbbbbbbbbbbbbb"),
        evidence: "verified source relation".into(),
    });
    let (_dir, mut service) = service(fixture);
    let both = search(&mut service, "flame");
    assert_eq!(both.page.total, 2);
    assert!(
        !both
            .records
            .iter()
            .any(|r| r.record.key == key("aaaaaaaaaaaaaaaa"))
    );
    let only = search(&mut service, "Legacy");
    assert_eq!(only.records[0].record.key, key("aaaaaaaaaaaaaaaa"));
    let filter = service.parse_where("item.level == 1").unwrap();
    let filtered = service
        .search_text(TextSearchRequest {
            query: "flame",
            mode: RetrievalMode::Fts,
            filter: Some(filter.predicate()),
            scope: RecordScope::All,
            page: SearchPage::first(1).unwrap(),
            prefer_remaster: true,
        })
        .unwrap();
    assert_eq!(filtered.page.total, 1);
    assert_eq!(filtered.records[0].record.key, key("aaaaaaaaaaaaaaaa"));
    let page2 = service
        .search_text(TextSearchRequest {
            query: "flame",
            mode: RetrievalMode::Fts,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::new(2, 1).unwrap(),
            prefer_remaster: true,
        })
        .unwrap();
    assert_eq!(page2.page.total, 2);
    assert_eq!(page2.records.len(), 1);
    assert_eq!(
        service
            .remaster_links(RemasterLinksRequest {
                record_key: &key("aaaaaaaaaaaaaaaa")
            })
            .unwrap()
            .unwrap()
            .links
            .len(),
        1
    );
    assert!(
        service
            .get_record(GetRecordRequest {
                record_key: &key("aaaaaaaaaaaaaaaa"),
                selected_content: &[]
            })
            .unwrap()
            .is_some()
    );
}
#[test]
fn exact_owned_names_and_definition_witnesses_stay_navigable() {
    let actor = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Ghoul","type":"npc","items":[{"_id":"bbbbbbbbbbbbbbbb","name":"Ghoul Fever","type":"action","system":{"description":{"value":"<h2>Ghoul Fever</h2><p>Drink water. @Check[fortitude|dc:17]</p>"}}}],"system":{"details":{"publicNotes":"<h2>Soul Degradation</h2><p>Some souls lose vitality.</p>"}}}),
    );
    let (_dir, mut service) = service(input(vec![actor], false));
    let result = search(&mut service, "\"Ghoul Fever\"");
    assert_eq!(result.records.len(), 1);
    let witness = result.records[0]
        .matches
        .iter()
        .find(|w| !w.location.owners.is_empty())
        .unwrap();
    assert_eq!(witness.label.as_deref(), Some("Ghoul Fever"));
    let detail = service
        .get_record(GetRecordRequest {
            record_key: &result.records[0].record.key,
            selected_content: &[],
        })
        .unwrap()
        .unwrap();
    assert!(detail.source.node_at(&witness.location.owners).is_some());
    let result = search(&mut service, "\"Soul Degradation\"");
    let witness = result.records[0]
        .matches
        .iter()
        .find(|w| {
            matches!(
                w.location.address,
                Some(SourcePassageAddress::HtmlSection { .. })
            )
        })
        .unwrap();
    assert!(witness.snippet.as_ref().unwrap().contains("souls"));
    assert!(
        service
            .recover_passage(&witness.location)
            .unwrap()
            .unwrap()
            .contains("souls")
    );
    assert!(
        search(&mut service, "\"lose vitality\"").records.is_empty(),
        "generic prose is semantic-only"
    );
}
#[test]
fn similar_uses_max_passage_score_excludes_seed_and_applies_key_scope_before_knn() {
    let mut fixture = input(
        vec![
            effect(
                "aaaaaaaaaaaaaaaa",
                "Seed",
                1,
                "<h2>Seed lore</h2><p>Seed prose.</p>",
            ),
            effect(
                "bbbbbbbbbbbbbbbb",
                "Best passage",
                2,
                "<h2>Best lore</h2><p>Strong prose.</p>",
            ),
            effect("cccccccccccccccc", "Other", 3, ""),
        ],
        true,
    );
    // Root B identity is orthogonal; its passage is closest to root A's seed.
    for unit in &mut fixture.semantic_units {
        if unit.record == key("bbbbbbbbbbbbbbbb") && unit.field.is_some() {
            unit.vector = vec![0.0; 384];
            unit.vector[0] = 1.0;
        }
    }
    let (_dir, service) = service(fixture);
    let seed = key("aaaaaaaaaaaaaaaa");
    let result = service
        .similar_records(SimilarRecordRequest {
            record_key: &seed,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            prefer_remaster: true,
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        result.results.records[0].record.key,
        key("bbbbbbbbbbbbbbbb")
    );
    assert_eq!(result.results.records[0].semantic_similarity, Some(1.0));
    assert!(!result.results.coverage.exhaustive);
    assert_eq!(result.results.coverage.semantic_unit_window, Some(4096));
    assert!(result.results.records.iter().all(|r| r.record.key != seed));
    let keys = [key("cccccccccccccccc")];
    let scoped = service
        .similar_records(SimilarRecordRequest {
            record_key: &seed,
            filter: None,
            scope: RecordScope::Keys(&keys),
            page: SearchPage::default(),
            prefer_remaster: true,
        })
        .unwrap()
        .unwrap();
    assert_eq!(scoped.results.records.len(), 1);
    assert_eq!(scoped.results.records[0].record.key, keys[0]);
    assert!(
        scoped.results.records[0].semantic_similarity.unwrap().abs() < 1e-6,
        "no arbitrary cosine cutoff"
    );
}
#[test]
fn source_summaries_batch_without_a_saved_list_key_limit() {
    let (_dir, service) = service(input(vec![effect("aaaaaaaaaaaaaaaa", "One", 1, "")], false));
    let keys = vec![key("aaaaaaaaaaaaaaaa"); 1100];
    let results = service
        .get_records(GetRecordsRequest { record_keys: &keys })
        .unwrap();
    assert_eq!(results.records.len(), 1100);
    assert!(results.missing.is_empty());
}
#[test]
fn suggested_variants_report_ambiguity_without_alias_or_pair_evidence() {
    let (_dir, service) = service(input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Potion (Lesser)", 1, ""),
            effect("bbbbbbbbbbbbbbbb", "Potion (Greater)", 3, ""),
            effect("cccccccccccccccc", "Potion (Greater)", 4, ""),
        ],
        false,
    ));
    let result = service
        .variant_group(VariantGroupRequest {
            record_key: &key("aaaaaaaaaaaaaaaa"),
        })
        .unwrap()
        .unwrap();
    assert_eq!(result.variants.len(), 3);
    assert!(result.ambiguous);
    assert!(
        result
            .evidence
            .iter()
            .all(|e| e.naming_convention == "trailing_parenthetical")
    );
    assert!(
        service
            .resolve_record(ResolveRecordRequest {
                query: "Potion",
                filter: None
            })
            .unwrap()
            .is_empty()
    );
}
#[test]
fn suggested_physical_variants_require_known_category_and_reuse_seed_body() {
    let weapon = |id, name, category, group| {
        record(
            "items",
            "Item",
            json!({
                "_id":id,"name":name,"type":"weapon","system":{
                    "category":category,"group":group,"traits":{"value":[],"rarity":"common"}
                }
            }),
        )
    };
    let (service, _artifact) = test_support::open_source_fixture(
        vec![
            weapon("aaaaaaaaaaaaaaaa", "Lesser Blade", "martial", "sword"),
            weapon("bbbbbbbbbbbbbbbb", "Greater Blade", "martial", "sword"),
            weapon("cccccccccccccccc", "Major Blade", "simple", "sword"),
        ],
        false,
    )
    .unwrap();
    service.reset_read_metrics();
    let result = service
        .variant_group(VariantGroupRequest {
            record_key: &key("aaaaaaaaaaaaaaaa"),
        })
        .unwrap()
        .unwrap();
    assert_eq!(result.variants.len(), 2);
    assert!(
        result.ambiguous,
        "incompatible suggestion must remain explicit"
    );
    assert!(
        result
            .evidence
            .iter()
            .all(|e| e.naming_convention == "leading_grade")
    );
    assert_eq!(
        service.read_metrics().source_body_decodes,
        3,
        "decode the seed only once"
    );
}
#[test]
#[ignore = "requires pinned local PF2e corpus"]
fn real_ghoul_fever_and_soul_degradation_open_exact_owned_and_gm_sources() {
    let corpus = std::path::Path::new(
        "/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/scratch/source-contracts-full/pf2e",
    );
    let ghoul = serde_json::from_slice(
        &std::fs::read(corpus.join("packs/pathfinder-bestiary/ghoul.json")).unwrap(),
    )
    .unwrap();
    let spirit = serde_json::from_slice(
        &std::fs::read(
            corpus.join("packs/campaign-effects/season-of-ghosts/effect-spirit-powers.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let (_dir, mut service) = service(input(
        vec![
            record("pathfinder-bestiary", "Actor", ghoul),
            record("campaign-effects", "Item", spirit),
        ],
        false,
    ));
    let ghoul = search(&mut service, "\"Ghoul Fever\"");
    assert_eq!(ghoul.records.len(), 1);
    assert_eq!(
        ghoul.records[0].record.key.to_string(),
        "pathfinder-bestiary:LHHgGSs0ELCR4CYK"
    );
    let witness = ghoul.records[0]
        .matches
        .iter()
        .find(|w| w.label.as_deref() == Some("Ghoul Fever"))
        .unwrap();
    assert!(!witness.location.owners.is_empty());
    let owner = ghoul.records[0].record.key.clone();
    let detail = service
        .get_record(GetRecordRequest {
            record_key: &owner,
            selected_content: &[],
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        detail
            .source
            .node_at(&witness.location.owners)
            .unwrap()
            .id()
            .value()
            .unwrap(),
        "oa6JQCeL4B2WQMPI"
    );
    let soul = search(&mut service, "\"Soul Degradation\"");
    assert_eq!(soul.records.len(), 1);
    let witness = soul.records[0]
        .matches
        .iter()
        .find(|w| w.location.field.as_deref() == Some("/system/description/gm"))
        .unwrap();
    assert_eq!(
        soul.records[0].matches[0].location.field.as_deref(),
        Some("/system/description/gm"),
        "actual heading definition precedes the public mention"
    );
    assert!(witness.snippet.as_ref().unwrap().contains("Curse 7"));
    let locator = atlas_record::source_content::SourceContentLocator {
        record: soul.records[0].record.key.clone(),
        owners: witness.location.owners.clone(),
        field: witness.location.field.clone().unwrap(),
    };
    let content = service.read_content(&[locator]).unwrap();
    assert_eq!(content.fields.len(), 1);
    assert!(!content.fields[0].interactions.is_empty());
    assert!(content.fields[0].html.as_ref().unwrap().contains("Stage 6"));
}
#[test]
fn contextual_numeric_bounds_restore_target_clause_and_apply_complete_text_scope() {
    let (_dir, mut service) = service(input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Flame one", 1, ""),
            effect("bbbbbbbbbbbbbbbb", "Flame two", 2, ""),
            effect("cccccccccccccccc", "Cold three", 3, ""),
        ],
        false,
    ));
    let mut predicate = service
        .parse_where("item.level == 1")
        .unwrap()
        .predicate()
        .clone();
    predicate.clause_id = Some("level-choice".into());
    let counts = service
        .discover_filter_counts(DiscoverFilterCountsRequest {
            field: "item.level",
            clause_id: Some("level-choice"),
            context: FilterDiscoveryContext {
                text: Some("flame"),
                mode: RetrievalMode::Fts,
                scope: RecordScope::All,
                prefer_remaster: true,
                filter: Some(&predicate),
            },
        })
        .unwrap();
    assert!(counts.exhaustive);
    assert_eq!(counts.minimum.unwrap().as_i64(), Some(1));
    assert_eq!(counts.maximum.unwrap().as_i64(), Some(2));
    assert_eq!(
        counts.states.iter().map(|s| s.distinct_roots).sum::<u64>(),
        2
    );
}
#[test]
fn displayed_html_witnesses_use_one_bounded_cache_batch_and_no_body_decode() {
    let html = "<h2>Flame first</h2><p>First prose.</p><h2>Flame second</h2><p>Second prose.</p>";
    let (_dir, mut service) = service(input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Flame one", 1, html),
            effect("bbbbbbbbbbbbbbbb", "Flame two", 2, html),
        ],
        false,
    ));
    service.index.reset_read_metrics();
    let result = search(&mut service, "flame");
    assert_eq!(result.records.len(), 2);
    assert!(result.records.iter().all(|r| r.matches.len() <= 3));
    let counts = service.index.read_metrics();
    assert_eq!(counts.source_body_decodes, 0);
    assert_eq!(counts.prepared_content_batches, 1);
    assert_eq!(counts.summary_batches, 1);
    assert!(counts.sql_statements <= 6, "{counts:?}");
    let owner = &result.records[0].record.key;
    let locator = atlas_record::source_content::SourceContentLocator {
        record: owner.clone(),
        owners: vec![],
        field: "/system/description/value".into(),
    };
    service.index.reset_read_metrics();
    let detail = service
        .get_record(GetRecordRequest {
            record_key: owner,
            selected_content: &[locator],
        })
        .unwrap()
        .unwrap();
    assert_eq!(detail.content.fields.len(), 1);
    let counts = service.index.read_metrics();
    assert_eq!(counts.source_body_decodes, 1);
    assert_eq!(counts.prepared_content_batches, 1);
}
#[test]
fn summary_browse_stays_body_free_and_plain_passages_decode_each_root_once() {
    let table = record(
        "journals",
        "JournalEntry",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Treasure journal","pages":[{"_id":"bbbbbbbbbbbbbbbb","name":"Ruby art","type":"image","image":{"caption":"A ruby worth a hundred pieces."}},{"_id":"cccccccccccccccc","name":"Ruby painting","type":"image","image":{"caption":"Another ruby."}}]}),
    );
    let (_dir, service) = service(input(vec![table], true));
    service.index.reset_read_metrics();
    let browse = service
        .list_records(ListRecordsRequest::new(None, SearchPage::default()))
        .unwrap();
    assert_eq!(browse.total, 1);
    assert_eq!(service.index.read_metrics().source_body_decodes, 0);
    let vector = service
        .index
        .seed_identity_vector(&browse.records[0].key)
        .unwrap()
        .unwrap();
    let query = service.predicate(None).unwrap();
    service.index.reset_read_metrics();
    let (roots, window) = service
        .rank_candidates(crate::text::CandidateRequest {
            text: "",
            mode: RetrievalMode::Vector,
            query: &query,
            scope: RecordScope::All,
            prefer: false,
            suppress: true,
            vector: Some(&vector.vector),
            exclude: None,
        })
        .unwrap();
    let results = service
        .finish_results(roots, SearchPage::default(), true, window)
        .unwrap();
    assert_eq!(results.records.len(), 1);
    assert!(results.records[0].matches.len() <= 3);
    assert_eq!(service.index.read_metrics().source_body_decodes, 1);
}
#[test]
fn root_max_score_has_no_bonus_from_siblings_or_unit_window() {
    fn hit(id: i64, distance: f64) -> atlas_index::SourceVectorHit {
        atlas_index::SourceVectorHit {
            location: atlas_index::SourceUnitLocation {
                unit_id: id,
                record: key("aaaaaaaaaaaaaaaa"),
                owners: vec![],
                field: None,
                address: Some(SourcePassageAddress::Identity {}),
            },
            distance,
        }
    }
    let one = crate::text::semantic_roots(vec![hit(1, 0.1)]);
    let mut many = (2..300).map(|id| hit(id, 0.7)).collect::<Vec<_>>();
    many.push(hit(1, 0.1));
    let many = crate::text::semantic_roots(many);
    assert_eq!(one[0].score, many[0].score);
    assert_eq!(one[0].similarity, many[0].similarity);
    assert_eq!(many[0].witnesses.len(), 3);
    assert_eq!(many[0].witnesses[0].location.unit_id, 1);
}
#[test]
#[ignore = "requires rebuilt full lexical corpus artifact"]
fn full_production_artifact_preserves_owned_and_gm_definition_navigation() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scratch/ingest-validation/full-lexical.sqlite");
    let reader = SqliteIndexReader::open_read_only(path).unwrap();
    let mut service = AtlasRetrievalService::from_prepared_index_without_embeddings(reader);
    for (pack, text, key, field) in [
        (
            "pathfinder-bestiary",
            "\"Ghoul Fever\"",
            "pathfinder-bestiary:LHHgGSs0ELCR4CYK",
            None,
        ),
        (
            "campaign-effects",
            "\"Soul Degradation\"",
            "campaign-effects:Rzlo3kvFUavFQJ3S",
            Some("/system/description/gm"),
        ),
    ] {
        let filter = service
            .parse_where(&format!("source.pack.id == '{pack}'"))
            .unwrap();
        service.index.reset_read_metrics();
        let result = service
            .search_text(TextSearchRequest {
                query: text,
                mode: RetrievalMode::Fts,
                filter: Some(filter.predicate()),
                scope: RecordScope::All,
                page: SearchPage::default(),
                prefer_remaster: true,
            })
            .unwrap();
        let root = result
            .records
            .iter()
            .find(|r| r.record.key.to_string() == key)
            .unwrap();
        assert!(result.coverage.exhaustive);
        assert!(root.matches.len() <= 3);
        assert_eq!(service.index.read_metrics().source_body_decodes, 0);
        let witness = if let Some(field) = field {
            root.matches
                .iter()
                .find(|w| w.location.field.as_deref() == Some(field))
                .unwrap()
        } else {
            root.matches
                .iter()
                .find(|w| !w.location.owners.is_empty())
                .unwrap()
        };
        let locators = if let Some(field) = &witness.location.field {
            vec![atlas_record::source_content::SourceContentLocator {
                record: root.record.key.clone(),
                owners: witness.location.owners.clone(),
                field: field.clone(),
            }]
        } else {
            vec![]
        };
        service.index.reset_read_metrics();
        let detail = service
            .get_record(GetRecordRequest {
                record_key: &root.record.key,
                selected_content: &locators,
            })
            .unwrap()
            .unwrap();
        assert!(detail.source.node_at(&witness.location.owners).is_some());
        assert_eq!(service.index.read_metrics().source_body_decodes, 1);
        if field.is_some() {
            assert!(
                detail.content.fields.iter().any(|f| f
                    .html
                    .as_ref()
                    .is_some_and(|h| h.contains("Stage 6"))
                    && !f.interactions.is_empty())
            );
        }
    }
}
#[test]
fn graph_keeps_attributed_occurrences_and_owned_targets_but_hides_macro_edges() {
    let origin = effect(
        "aaaaaaaaaaaaaaaa",
        "Origin",
        1,
        "<p>@UUID[Compendium.pf2e.actors.Actor.bbbbbbbbbbbbbbbb.Item.cccccccccccccccc]{Owned Strike} @UUID[Compendium.pf2e.macros.Macro.dddddddddddddddd]{Tool} @UUID[Compendium.pf2e.items.Item.ffffffffffffffff]{Missing}</p>",
    );
    let target = record(
        "actors",
        "Actor",
        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Target","type":"npc","items":[{"_id":"cccccccccccccccc","name":"Owned Strike","type":"action"}]}),
    );
    let tool = record(
        "macros",
        "Macro",
        json!({"_id":"dddddddddddddddd","name":"Tool","type":"script","command":""}),
    );
    let (_dir, service) = service(input(vec![origin, target, tool], false));
    let outgoing = service
        .graph_context(GraphContextRequest::new(key("aaaaaaaaaaaaaaaa")))
        .unwrap()
        .unwrap();
    assert_eq!(outgoing.outgoing.records.len(), 1);
    assert_eq!(outgoing.outgoing.occurrences.len(), 2);
    assert!(outgoing.outgoing.occurrences.iter().any(|o|matches!(&o.resolution,atlas_record::source_content::ContentReferenceResolution::Resolved(atlas_record::source_content::ContentReferenceTarget::OwnedNode{owners,..}) if !owners.is_empty())));
    assert!(outgoing.outgoing.occurrences.iter().any(|o| matches!(
        o.resolution,
        atlas_record::source_content::ContentReferenceResolution::Unresolved
    )));
    let target = RecordKey::parse("actors:bbbbbbbbbbbbbbbb").unwrap();
    let uses = service
        .graph_context(GraphContextRequest::uses(target))
        .unwrap()
        .unwrap();
    assert_eq!(uses.backlinks.records[0].key, key("aaaaaaaaaaaaaaaa"));
    assert_eq!(
        uses.backlinks.occurrences[0].locator.record,
        key("aaaaaaaaaaaaaaaa")
    );
    assert!(!uses.backlinks.occurrences[0].origin.is_empty());
}
#[test]
fn many_embedded_spell_units_expand_a_fixed_window_without_a_root_score_bonus() {
    let spells=(0..1050).map(|i|json!({"_id":format!("{i:016x}"),"name":format!("Spell {i}"),"type":"spell","system":{"description":{"value":format!("<p>Copied spell body {i}.</p>")}}})).collect::<Vec<_>>();
    let actor = record(
        "actors",
        "Actor",
        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Many spell actor","type":"npc","items":spells}),
    );
    let mut fixture = input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Seed", 1, ""),
            actor,
            effect("cccccccccccccccc", "Sparse contender", 2, ""),
        ],
        true,
    );
    let actor_key = RecordKey::parse("actors:bbbbbbbbbbbbbbbb").unwrap();
    for unit in &mut fixture.semantic_units {
        if unit.record == actor_key {
            unit.vector = vec![0.0; 384];
            unit.vector[0] = 1.0;
        }
    }
    let (_dir, service) = service(fixture);
    let result = service
        .similar_records(SimilarRecordRequest {
            record_key: &key("aaaaaaaaaaaaaaaa"),
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            prefer_remaster: true,
        })
        .unwrap()
        .unwrap();
    assert_eq!(result.results.coverage.semantic_unit_window, Some(4096));
    assert_eq!(result.results.records.len(), 2);
    assert_eq!(result.results.records[0].record.key, actor_key);
    assert_eq!(result.results.records[0].score, 1.0);
    assert_eq!(result.results.records[0].matches.len(), 3);
    assert!(
        result
            .results
            .records
            .iter()
            .any(|r| r.record.key == key("cccccccccccccccc"))
    );
    let page2 = service
        .similar_records(SimilarRecordRequest {
            record_key: &key("aaaaaaaaaaaaaaaa"),
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::new(2, 1).unwrap(),
            prefer_remaster: true,
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        page2.results.coverage.semantic_unit_window,
        result.results.coverage.semantic_unit_window
    );
    assert_eq!(page2.results.page.total, result.results.page.total);
    assert_eq!(page2.results.records[0].record.key, key("cccccccccccccccc"));
}
