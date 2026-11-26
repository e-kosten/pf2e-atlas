//! Compare a checked document with the same exact sections selected on its parent.
use super::*;
use crate::test_support::{input, open_input_fixture, record};
use atlas_domain::{OwnedContentIdentity, OwnedContentLocator};
use serde_json::json;

fn page(key: &RecordKey, id: &str) -> SimilaritySeed {
    SimilaritySeed::Record {
        record_key: key.clone(),
        embedded_path: vec![OwnedContentLocator {
            collection: "/pages".into(),
            identity: OwnedContentIdentity::Stable(id.into()),
        }],
        source_fingerprint: None,
    }
}

fn request<'a>(
    seed: &'a SimilaritySeed,
    options: &'a SimilarityOptions,
) -> ConfigurableSimilarRequest<'a> {
    ConfigurableSimilarRequest {
        seed,
        options,
        source_sections: None,
        content_scope: SearchContentScope::All,
        search_profile: None,
        filter: None,
        scope: RecordScope::All,
        page: SearchPage::first(5).unwrap(),
        sort: ResultSort::Relevance {},
        result_detail: atlas_domain::SearchResultDetail::Evidence,
        cancelled: None,
    }
}

fn journal() -> atlas_index::IndexBuildInput {
    let mut data = input(
        vec![
            record(
                "journals",
                "JournalEntry",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Classes","pages":[
                    {"_id":"bbbbbbbbbbbbbbbb","name":"Psychic","type":"text","text":{"format":1,"content":"<h2>Spells</h2><p>Psychic spellcasting.</p><h2>Mind</h2><p>Think deeply.</p>"}},
                    {"_id":"cccccccccccccccc","name":"Psychic","type":"text","text":{"format":1,"content":"<h2>Spells</h2><p>Different spellcasting.</p>"}},
                    {"_id":"dddddddddddddddd","name":"Empty","type":"text","text":{"format":1,"content":"<p></p>"}},
                    {"name":"Positional page","type":"text","text":{"format":1,"content":"<p>Local page.</p>"}}
                ]}),
            ),
            record(
                "items",
                "Item",
                json!({"_id":"eeeeeeeeeeeeeeee","name":"Candidate","type":"spell","system":{"description":{"value":"<p>Psychic spellcasting.</p>"}}}),
            ),
        ],
        true,
    );
    for unit in &mut data.semantic_units {
        unit.vector.fill(0.);
        unit.vector[0] = 1.;
    }
    data
}

#[test]
fn embedded_inventory_addresses_selection_and_ranking_agree_on_exact_owner() {
    let (service, _artifact) = open_input_fixture(journal()).unwrap();
    let key: RecordKey = "journals:aaaaaaaaaaaaaaaa".parse().unwrap();
    let root = SimilaritySeed::root(key.clone());
    let psychic = page(&key, "bbbbbbbbbbbbbbbb");
    let sibling = page(&key, "cccccccccccccccc");
    let inventory = service.similarity_parts(&psychic, None).unwrap();
    assert_eq!(inventory.seed, psychic);
    assert!(inventory.core.is_none());
    assert!(inventory.parts.len() >= 2);
    let addresses = service.similarity_part_addresses(&psychic).unwrap();
    for component in &inventory.parts {
        let part = component.part().unwrap();
        assert_eq!(part.owners, psychic.embedded_path());
        assert!(addresses.contains_key(&part.selector));
    }
    assert_eq!(inventory.parts.len(), addresses.len());
    let options = SimilarityOptions::default();
    let selected = SimilarityOptions::Passages {
        decay: atlas_domain::default_similarity_decay(),
        candidate_parts: atlas_domain::SimilarityCandidateParts::Any,
        parts: Some(
            inventory
                .parts
                .iter()
                .map(|part| part.selector().clone())
                .collect(),
        ),
    };
    let result = service
        .configurable_similar(request(&psychic, &options))
        .unwrap()
        .unwrap();
    let explicit = service
        .configurable_similar(request(&root, &selected))
        .unwrap()
        .unwrap();
    assert_eq!(result.resolved_selectors, explicit.resolved_selectors);
    assert!(!result.candidate_identity);
    assert!(!result.evidence.is_empty());
    let keys = |r: &ConfigurableSimilarResult| {
        r.results
            .records
            .iter()
            .map(|row| row.record.key.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&result), keys(&explicit));
    let evidence = |result: &ConfigurableSimilarResult| {
        result
            .evidence
            .iter()
            .map(|(key, e)| {
                (
                    key.clone(),
                    e.score,
                    e.parts
                        .iter()
                        .map(|p| p.part.selector().clone())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(evidence(&result), evidence(&explicit));
    assert_eq!(
        service
            .configurable_similar(request(&sibling, &selected))
            .unwrap_err()
            .kind(),
        crate::SearchErrorKind::SimilaritySelectionStale
    );
}

#[test]
fn document_identity_and_recovery_distinguish_empty_absent_stale_and_whole_record_methods() {
    let (service, _artifact) = open_input_fixture(journal()).unwrap();
    let key: RecordKey = "journals:aaaaaaaaaaaaaaaa".parse().unwrap();
    let root = SimilaritySeed::root(key.clone());
    let empty = page(&key, "dddddddddddddddd");
    for (source, body_decodes) in [(&root, 0), (&empty, 1)] {
        service.reset_read_metrics();
        assert!(service.similarity_source(source).unwrap().is_some());
        let metrics = service.read_metrics();
        assert_eq!(metrics.source_body_decodes, body_decodes);
        assert_eq!(metrics.knn_queries, 0);
        assert_eq!(metrics.prepared_content_batches, 0);
    }
    let inv = service.similarity_parts(&empty, None).unwrap();
    assert!(inv.parts.is_empty());
    let options = SimilarityOptions::default();
    assert_eq!(
        service
            .configurable_similar(request(&empty, &options))
            .unwrap_err()
            .kind(),
        crate::SearchErrorKind::SimilarityNoContent
    );
    let missing = page(&key, "missing000000000");
    assert_eq!(
        service.similarity_parts(&missing, None).unwrap_err().kind(),
        crate::SearchErrorKind::SimilaritySourceMissing
    );
    let mut positional = SimilaritySeed::Record {
        record_key: key.clone(),
        embedded_path: vec![OwnedContentLocator {
            collection: "/pages".into(),
            identity: OwnedContentIdentity::SnapshotLocal { index: 3 },
        }],
        source_fingerprint: None,
    };
    for fingerprint in [
        None,
        Some("stale".into()),
        Some(service.source_fingerprint().to_owned()),
    ] {
        let current = fingerprint.as_deref() == Some(service.source_fingerprint());
        let SimilaritySeed::Record {
            source_fingerprint, ..
        } = &mut positional;
        *source_fingerprint = fingerprint;
        let result = service.similarity_source(&positional);
        if current {
            assert!(result.unwrap().is_some());
        } else {
            assert_eq!(
                result.unwrap_err().kind(),
                crate::SearchErrorKind::SimilaritySourceStale
            );
        }
    }
    assert_eq!(
        service
            .configurable_similar(request(&empty, &SimilarityOptions::Baseline {}))
            .unwrap_err()
            .kind(),
        crate::SearchErrorKind::SimilarityNoContent
    );
    let source = page(&key, "bbbbbbbbbbbbbbbb");
    let mut core = request(&source, &options);
    core.source_sections = Some(SimilaritySourceSections::Core);
    assert_eq!(
        service.configurable_similar(core).unwrap_err().kind(),
        crate::SearchErrorKind::SimilarityMethodUnavailable
    );
    assert_eq!(
        service
            .configurable_similar(request(&source, &SimilarityOptions::Baseline {}))
            .unwrap_err()
            .kind(),
        crate::SearchErrorKind::SimilarityMethodUnavailable
    );
}

#[test]
fn owned_item_and_its_nested_spell_use_disjoint_document_inventories() {
    let actor = record(
        "actors",
        "Actor",
        json!({
            "_id":"aaaaaaaaaaaaaaaa","name":"Wand keeper","type":"npc",
            "items":[{"_id":"bbbbbbbbbbbbbbbb","name":"Wand","type":"consumable","system":{
                "description":{"value":"<p>A slender wand.</p>"},
                "category":"wand","spell":{"_id":"cccccccccccccccc","name":"Custom spell","type":"spell",
                "system":{"description":{"value":"<p>A wave of healing.</p>"}}}
            }}]
        }),
    );
    let (service, _artifact) = open_input_fixture(input(vec![actor], true)).unwrap();
    let record_key: RecordKey = "actors:aaaaaaaaaaaaaaaa".parse().unwrap();
    let wand = OwnedContentLocator {
        collection: "/items".into(),
        identity: OwnedContentIdentity::Stable("bbbbbbbbbbbbbbbb".into()),
    };
    let spell = OwnedContentLocator {
        collection: "/system/spell".into(),
        identity: OwnedContentIdentity::Stable("cccccccccccccccc".into()),
    };
    for owners in [vec![wand.clone()], vec![wand, spell]] {
        let source = SimilaritySeed::Record {
            record_key: record_key.clone(),
            embedded_path: owners.clone(),
            source_fingerprint: None,
        };
        let parts = service.similarity_parts(&source, None).unwrap().parts;
        assert!(!parts.is_empty());
        assert!(
            parts
                .iter()
                .all(|component| component.part().is_some_and(|p| p.owners == owners))
        );
    }
}

#[test]
fn embedded_text_without_profile_vectors_cannot_offer_a_content_comparison() {
    let key = "journals:aaaaaaaaaaaaaaaa".parse().unwrap();
    let source = page(&key, "bbbbbbbbbbbbbbbb");
    let mut data = journal();
    data.semantic_units
        .retain(|u| u.owners != source.embedded_path());
    let (service, _artifact) = open_input_fixture(data).unwrap();
    let inventory = service.similarity_parts(&source, None).unwrap();
    assert!(!inventory.parts.is_empty());
    assert!(inventory.parts.iter().all(|p| !p.has_vectors()));
    for method in [SimilarityOptions::default(), SimilarityOptions::Baseline {}] {
        assert_eq!(
            service
                .configurable_similar(request(&source, &method))
                .unwrap_err()
                .kind(),
            crate::SearchErrorKind::VectorReadinessRequired
        );
    }
}
