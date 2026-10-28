use super::*;
use crate::test_support::{hash, input, open_input_fixture, record};
use atlas_domain::{
    QueryPredicate, SimilarityCandidateParts, SimilarityRole, SourcePassageAddress,
};
use serde_json::json;

#[test]
fn actual_work_limits_keep_exact_boundaries_and_explain_how_to_reduce_work() {
    let mut vectors = CandidateWorkBudget::default();
    vectors
        .include(MAX_CANDIDATE_UNITS, 1, 1, 128, true)
        .unwrap();
    assert!(
        vectors
            .remaining_vectors()
            .unwrap_err()
            .to_string()
            .contains(LIMIT_RECOVERY)
    );
    let error = vectors.include(1, 1, 1, 1, true).unwrap_err();
    assert!(error.to_string().contains("65536"));
    assert!(error.to_string().contains(LIMIT_RECOVERY));

    let mut passages = CandidateWorkBudget::default();
    passages.include(62_500, 128, 128, 128, true).unwrap();
    assert_eq!(passages.comparisons, MAX_COMPARISONS);
    let error = passages.include(1, 128, 128, 1, true).unwrap_err();
    assert!(error.to_string().contains("8000000"));
    assert!(error.to_string().contains(LIMIT_RECOVERY));

    let mut averages = CandidateWorkBudget::default();
    for _ in 0..242 {
        averages.include(128, 128, 128, 128, false).unwrap();
    }
    let error = averages.include(128, 128, 128, 128, false).unwrap_err();
    assert!(error.to_string().contains("8000000"));
    assert!(error.to_string().contains(LIMIT_RECOVERY));
}

#[test]
fn complete_candidate_union_above_4096_is_scored_when_actual_work_fits() {
    let source = (0..18)
        .map(|i| format!("<h2>Section {i}</h2><p>Seed description {i}</p>"))
        .collect::<String>();
    let mut records = vec![record(
        "actors",
        "Actor",
        json!({
            "_id":"aaaaaaaaaaaaaaaa", "name":"Seed", "type":"npc",
            "system":{"details":{"publicNotes":source}}
        }),
    )];
    for i in 0..4320 {
        records.push(record(
            "actors",
            "Actor",
            json!({
                "_id":format!("b{i:015}"), "name":format!("Candidate {i}"), "type":"npc",
                "system":{"details":{"publicNotes":"<p>Candidate description</p>"}}
            }),
        ));
    }
    let mut data = input(records, true);
    for unit in &mut data.semantic_units {
        let axis = match &unit.address {
            SourcePassageAddress::Identity {} => 0,
            SourcePassageAddress::HtmlSection {
                section_ordinal, ..
            } if unit.record.id().as_str() == "aaaaaaaaaaaaaaaa" => section_ordinal + 1,
            _ => unit.record.id().as_str()[1..].parse::<usize>().unwrap() / 240 + 1,
        };
        unit.vector.fill(0.);
        unit.vector[axis] = 1.;
    }
    let (service, _artifact) = open_input_fixture(data).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let options = SimilarityOptions::Passages {
        parts: None,
        candidate_parts: SimilarityCandidateParts::Corresponding,
        decay: 0.25,
    };
    let result = service
        .configurable_similar(ConfigurableSimilarRequest {
            seed: &seed,
            options: &options,
            source_sections: Some(SimilaritySourceSections::All),
            content_scope: SearchContentScope::Core,
            search_profile: None,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::first(10).unwrap(),
            sort: ResultSort::Relevance {},
            prefer_remaster: false,
            cancelled: None,
        })
        .unwrap()
        .unwrap();
    assert_eq!(result.candidate_keys.as_ref().unwrap().len(), 4320);
    assert_eq!(result.results.page.total, 4320);
    assert_eq!(result.results.records.len(), 10);
    assert_eq!(result.diagnostics.seed_units, 19);
    assert!(result.diagnostics.knn_queries <= 2 * result.diagnostics.seed_units);
    assert!(
        result.candidate_keys.as_ref().unwrap().len()
            <= result.diagnostics.knn_queries * CANDIDATE_WINDOW
    );
    assert!(result.diagnostics.candidate_vectors < MAX_CANDIDATE_UNITS);
    assert!(result.diagnostics.pairwise_comparisons < MAX_COMPARISONS);
}

#[test]
fn index_vector_read_limits_keep_actionable_section_recovery() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let error = service
        .index
        .similarity_units(
            &SearchProfileId::new("bge-base"),
            &["hazards:aaaaaaaaaaaaaaaa".parse().unwrap()],
            SearchContentScope::All,
            true,
            1,
        )
        .unwrap_err();
    let error = SearchError::from(error);
    assert_eq!(error.kind(), crate::SearchErrorKind::InvalidOptions);
    assert!(error.to_string().contains("Vector limit: 1"));
    assert!(error.to_string().contains(LIMIT_RECOVERY));
}

#[test]
fn compact_explanations_keep_first_32_rows_but_score_and_rank_all_65_components() {
    let prose = (0..64)
        .map(|i| format!("<h2>Section {i}</h2><p>Source passage {i}</p>"))
        .collect::<String>();
    let mut data = input(
        vec![
            record(
                "actors",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Source","type":"npc","system":{"details":{"publicNotes":prose}}}),
            ),
            record(
                "actors",
                "Actor",
                json!({"_id":"bbbbbbbbbbbbbbbb","name":"Candidate A","type":"npc","system":{"details":{"publicNotes":"<p>Candidate A description</p>"}}}),
            ),
            record(
                "actors",
                "Actor",
                json!({"_id":"cccccccccccccccc","name":"Candidate B","type":"npc","system":{"details":{"publicNotes":"<p>Candidate B description</p>"}}}),
            ),
        ],
        true,
    );
    for unit in &mut data.semantic_units {
        unit.vector.fill(0.);
        match &unit.address {
            SourcePassageAddress::Identity {} => unit.vector[0] = 1.,
            SourcePassageAddress::HtmlSection {
                section_ordinal, ..
            } if unit.record.id().as_str() == "aaaaaaaaaaaaaaaa" => {
                if *section_ordinal < 31 {
                    unit.vector[1] = 1.;
                } else {
                    unit.vector[0] = 0.8;
                    unit.vector[1] = 0.6;
                }
            }
            _ if unit.record.id().as_str() == "bbbbbbbbbbbbbbbb" => unit.vector[1] = 1.,
            _ => {
                unit.vector[0] = 0.6;
                unit.vector[1] = 0.8;
            }
        }
    }
    let (service, _artifact) = open_input_fixture(data).unwrap();
    let key: RecordKey = "actors:aaaaaaaaaaaaaaaa".parse().unwrap();
    let seed = SimilaritySeed::Record {
        record_key: key.clone(),
    };
    let options = SimilarityOptions::Passages {
        parts: None,
        candidate_parts: SimilarityCandidateParts::Corresponding,
        decay: 1.,
    };
    let result = service
        .configurable_similar(ConfigurableSimilarRequest {
            seed: &seed,
            options: &options,
            source_sections: Some(SimilaritySourceSections::All),
            content_scope: SearchContentScope::All,
            search_profile: None,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::first(10).unwrap(),
            sort: ResultSort::Relevance {},
            prefer_remaster: false,
            cancelled: None,
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        result
            .results
            .records
            .iter()
            .map(|row| row.record.key.id().as_str())
            .collect::<Vec<_>>(),
        ["cccccccccccccccc", "bbbbbbbbbbbbbbbb"]
    );
    assert_eq!(result.results.page.total, 2);
    for (id, expected_score, section_score, section_range) in [
        ("bbbbbbbbbbbbbbbb", 51.8 / 65., 1., 0..31),
        ("cccccccccccccccc", 57.48 / 65., 0.96, 31..64),
    ] {
        let evidence = &result
            .evidence
            .iter()
            .find(|(key, _)| key.id().as_str() == id)
            .unwrap()
            .1;
        assert!(
            (evidence.score - expected_score).abs() < 1e-6,
            "{id}: {}",
            evidence.score
        );
        assert_eq!(evidence.selected_parts, 65);
        assert_eq!(evidence.comparable_parts, 65);
        assert!(evidence.truncated);
        assert_eq!(evidence.parts.len(), 32);
        assert_eq!(
            evidence
                .parts
                .iter()
                .filter(|part| part.part.is_identity())
                .count(),
            1
        );
        assert_eq!(
            evidence
                .parts
                .iter()
                .map(|part| part.part.selector())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            32
        );
        for part in &evidence.parts {
            assert!(part.comparable);
            assert!((part.weight - 1. / 65.).abs() < 1e-10);
            let expected = if let Some(section) = part.part.part() {
                assert_eq!(section.field, "/system/details/publicNotes");
                assert!(section_range.contains(&section.section_ordinal));
                assert_eq!(
                    section.label,
                    format!("Section {}", section.section_ordinal)
                );
                section_score
            } else {
                1.
            };
            assert!((part.score - expected).abs() < 1e-6, "{id}: {}", part.score);
        }
    }
}

#[test]
fn compact_scores_preserve_source_bindings_and_bound_the_allocation_at_32_rows() {
    for (count, retained, truncated) in [
        (0, 0, false),
        (31, 31, false),
        (32, 32, false),
        (33, 32, true),
        (65, 32, true),
    ] {
        let components = (0..count)
            .map(|index| atlas_domain::SimilarityComponent::Section {
                part: atlas_domain::SimilarityPart {
                    selector: atlas_domain::SimilarityPartSelector(format!("part-{index}")),
                    label: format!("Section {index}"),
                    owner_label: "Source".into(),
                    owners: vec![],
                    field: "/system/details/publicNotes".into(),
                    section_ordinal: index,
                    role: SimilarityRole::Description,
                    has_vectors: true,
                    label_only: false,
                },
            })
            .collect::<Vec<_>>();
        let source_indices = components
            .iter()
            .enumerate()
            .map(|(index, part)| (part.selector(), count - index - 1))
            .collect();
        let scores = components
            .iter()
            .enumerate()
            .map(|(index, part)| crate::similarity_scoring::PartScore {
                part: part.clone(),
                score: (count - index) as f64 / count as f64,
                weight: 1. / count as f64,
                comparable: true,
            })
            .collect();
        let (compact, actual_truncated) = compact_part_scores(scores, &source_indices).unwrap();
        assert_eq!(actual_truncated, truncated, "{count} parts");
        assert_eq!(compact.len(), retained, "{count} parts");
        assert_eq!(compact.capacity(), retained, "{count} parts");
        assert_eq!(
            compact
                .iter()
                .map(|part| part.source_part)
                .collect::<Vec<_>>(),
            (count - retained..count).rev().collect::<Vec<_>>()
        );
        for (index, part) in compact.iter().enumerate() {
            assert_eq!(part.score, (count - index) as f64 / count as f64);
            assert_eq!(part.weight, 1. / count as f64);
            assert!(part.comparable);
        }
    }
}

#[test]
fn section_navigation_combines_overlapping_chunks_across_profiles_without_hydrating_content() {
    let text = "Long description ".repeat(20);
    let mut data = crate::test_support::input_with_both_profiles(vec![record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Seed","type":"npc","system":{"details":{"publicNotes":format!("<p>{text}</p>")}}}),
    )]);
    let mut chunks = vec![];
    data.semantic_units.retain(|unit| {
        if unit.field.as_deref() != Some("/system/details/publicNotes") {
            return true;
        }
        let SourcePassageAddress::HtmlSection { chunk_bytes, .. } = &unit.address else {
            unreachable!()
        };
        let length = chunk_bytes.end;
        for (ordinal, range) in [
            atlas_domain::SourceByteRange { start: 0, end: 100 },
            atlas_domain::SourceByteRange {
                start: 80,
                end: length,
            },
        ]
        .into_iter()
        .enumerate()
        {
            let mut chunk = unit.clone();
            chunk.chunk_ordinal = ordinal;
            chunk.input_hash = hash(format!("chunk {ordinal}").as_bytes());
            let SourcePassageAddress::HtmlSection { chunk_bytes, .. } = &mut chunk.address else {
                unreachable!()
            };
            *chunk_bytes = range;
            chunks.push(chunk);
        }
        false
    });
    data.semantic_units.extend(chunks);
    let (service, _artifact) = open_input_fixture(data).unwrap();
    let key = "actors:aaaaaaaaaaaaaaaa".parse().unwrap();
    service.reset_read_metrics();
    let addresses = service.similarity_part_addresses(&key).unwrap();
    assert_eq!(addresses.len(), 1);
    let metrics = service.read_metrics();
    assert_eq!(metrics.sql_statements, 1);
    assert_eq!(metrics.source_body_decodes, 0);
    assert_eq!(metrics.prepared_content_batches, 0);
    let address = addresses.values().next().unwrap();
    let SourcePassageAddress::HtmlSection {
        chunk_bytes, label, ..
    } = address
    else {
        panic!("expected a checked HTML section")
    };
    assert_eq!(
        *chunk_bytes,
        atlas_domain::SourceByteRange {
            start: 0,
            end: text.trim().len()
        }
    );
    assert!(label.is_none());
    let seed = SimilaritySeed::Record { record_key: key };
    for profile in ["bge-base", "embeddinggemma"] {
        let inventory = service
            .similarity_parts(&seed, Some(&SearchProfileId::new(profile)))
            .unwrap();
        assert!(
            inventory
                .parts
                .iter()
                .filter_map(|component| component.part())
                .all(|part| addresses.contains_key(&part.selector))
        );
    }
}

fn fixture() -> atlas_index::IndexBuildInput {
    let mut input = input(
        vec![
            record(
                "hazards",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Seed","type":"hazard","system":{"details":{"description":"<p>Ordinary appearance</p>","reset":"<p>Returns at dawn</p>"}}}),
            ),
            record(
                "spells",
                "Item",
                json!({"_id":"bbbbbbbbbbbbbbbb","name":"Spell","type":"spell","system":{"description":{"value":"<p>Returns at dawn</p>"}}}),
            ),
            record(
                "hazards",
                "Actor",
                json!({"_id":"cccccccccccccccc","name":"Candidate","type":"hazard","system":{"details":{"description":"<p>Ordinary appearance</p>","reset":"<p>Returns at midnight</p>"}}}),
            ),
        ],
        true,
    );
    for unit in &mut input.semantic_units {
        unit.vector.fill(0.);
        unit.vector[0] = 1.;
        if unit.record.id().as_str() == "cccccccccccccccc"
            && unit.field.as_deref() == Some("/system/details/reset")
        {
            unit.vector[0] = 0.8;
            unit.vector[1] = 0.6;
        }
    }
    input
}

#[test]
fn corresponding_roles_filter_before_the_bounded_knn_and_other_never_corresponds() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let profile = SearchProfileId::new("bge-base");
    let mut vector = vec![0.; 768];
    vector[0] = 1.;
    let query = service
        .validate_filter(&QueryPredicate::boolean(true))
        .unwrap();
    let exclude = "hazards:aaaaaaaaaaaaaaaa".parse().unwrap();
    let hits = service
        .index
        .similarity_candidates(atlas_index::SourceSimilarityCandidateRequest {
            profile: &profile,
            vector: &vector,
            query: &query,
            keys: None,
            exclude: Some(&exclude),
            comparison_key: Some(SimilarityComparisonKey::Role(SimilarityRole::Reset)),
            content_scope: SearchContentScope::All,
            include_identity: false,
            limit: 1,
        })
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits[0].location.record.to_string(),
        "hazards:cccccccccccccccc"
    );
    assert_eq!(
        hits[0].location.field.as_deref(),
        Some("/system/details/reset")
    );
    assert!(
        service
            .index
            .similarity_candidates(atlas_index::SourceSimilarityCandidateRequest {
                profile: &profile,
                vector: &vector,
                query: &query,
                keys: None,
                exclude: None,
                comparison_key: Some(SimilarityComparisonKey::Role(SimilarityRole::Other)),
                content_scope: SearchContentScope::All,
                include_identity: false,
                limit: 1,
            })
            .unwrap()
            .is_empty()
    );
}

#[test]
fn exact_part_inventory_is_profile_independent_and_models_are_not_loaded() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let inventory = service.similarity_parts(&seed, None).unwrap();
    let reset = inventory
        .parts
        .iter()
        .filter_map(|component| component.part())
        .find(|part| part.role == SimilarityRole::Reset)
        .unwrap();
    let options = SimilarityOptions::Passages {
        decay: 0.5,
        parts: Some(vec![reset.selector.clone(), reset.selector.clone()]),
        candidate_parts: SimilarityCandidateParts::Any,
    };
    let result = service
        .configurable_similar(ConfigurableSimilarRequest {
            source_sections: None,
            content_scope: atlas_domain::SearchContentScope::All,
            seed: &seed,
            options: &options,
            search_profile: None,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            sort: ResultSort::Relevance {},
            prefer_remaster: false,
            cancelled: None,
        })
        .unwrap()
        .unwrap();
    assert_eq!(result.evidence[0].1.selected_parts, 1);
    assert!(!result.evidence[0].1.pairs[0].seed_text.is_empty());
    assert!(
        result
            .results
            .records
            .iter()
            .all(|row| row.record.key.to_string() != "hazards:aaaaaaaaaaaaaaaa")
    );
}

#[test]
fn selecting_a_small_part_of_a_large_seed_reads_only_selected_vectors() {
    let source = format!("<p>{}</p>", "abcdefghij".repeat(140));
    let mut input = input(
        vec![
            record(
                "hazards",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Long seed","type":"hazard","system":{"details":{"description":source,"reset":"<p>Returns at dawn</p>"}}}),
            ),
            record(
                "spells",
                "Item",
                json!({"_id":"bbbbbbbbbbbbbbbb","name":"Candidate","type":"spell","system":{"description":{"value":"<p>Returns at dawn</p>"}}}),
            ),
        ],
        true,
    );
    let mut split = Vec::new();
    input.semantic_units.retain(|unit| {
        if unit.record.to_string() != "hazards:aaaaaaaaaaaaaaaa"
            || unit.field.as_deref() != Some("/system/details/description")
        {
            return true;
        }
        let length = match &unit.address {
            SourcePassageAddress::HtmlSection { chunk_bytes, .. } => chunk_bytes.end,
            _ => unreachable!(),
        };
        for (ordinal, start) in (0..length).step_by(10).enumerate() {
            let mut chunk = unit.clone();
            chunk.chunk_ordinal = ordinal;
            chunk.input_hash = hash(format!("long chunk {ordinal}").as_bytes());
            if let SourcePassageAddress::HtmlSection { chunk_bytes, .. } = &mut chunk.address {
                chunk_bytes.start = start;
                chunk_bytes.end = (start + 10).min(length);
            }
            split.push(chunk);
        }
        false
    });
    input.semantic_units.extend(split);
    for unit in &mut input.semantic_units {
        unit.vector.fill(0.);
        unit.vector[0] = 1.;
    }
    let (service, _artifact) = open_input_fixture(input).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let reset = service
        .similarity_parts(&seed, None)
        .unwrap()
        .parts
        .into_iter()
        .filter_map(|component| match component {
            atlas_domain::SimilarityComponent::Section { part } => Some(part),
            atlas_domain::SimilarityComponent::Identity { .. } => None,
        })
        .find(|part| part.role == SimilarityRole::Reset)
        .unwrap();
    let run = |options: &SimilarityOptions| {
        service.configurable_similar(ConfigurableSimilarRequest {
            source_sections: None,
            content_scope: atlas_domain::SearchContentScope::All,
            seed: &seed,
            options,
            search_profile: None,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            sort: ResultSort::Relevance {},
            prefer_remaster: false,
            cancelled: None,
        })
    };
    assert!(
        run(&SimilarityOptions::default()).unwrap_err().kind()
            == crate::SearchErrorKind::InvalidOptions
    );
    let options = SimilarityOptions::Passages {
        decay: 0.5,
        parts: Some(vec![reset.selector]),
        candidate_parts: SimilarityCandidateParts::Any,
    };
    assert_eq!(run(&options).unwrap().unwrap().diagnostics.seed_units, 1);
}

#[test]
fn stale_parts_and_cancellation_are_explicit() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let options = SimilarityOptions::Passages {
        decay: 0.5,
        parts: Some(vec![SimilarityPartSelector("stale".into())]),
        candidate_parts: SimilarityCandidateParts::Any,
    };
    let request = ConfigurableSimilarRequest {
        source_sections: None,
        content_scope: atlas_domain::SearchContentScope::All,
        seed: &seed,
        options: &options,
        search_profile: None,
        filter: None,
        scope: RecordScope::All,
        page: SearchPage::default(),
        sort: ResultSort::Relevance {},
        prefer_remaster: false,
        cancelled: None,
    };
    assert_eq!(
        service.configurable_similar(request).unwrap_err().kind(),
        crate::SearchErrorKind::InvalidOptions
    );
    let cancelled = || true;
    let request = ConfigurableSimilarRequest {
        source_sections: None,
        content_scope: atlas_domain::SearchContentScope::All,
        seed: &seed,
        options: &SimilarityOptions::Baseline {},
        search_profile: None,
        filter: None,
        scope: RecordScope::All,
        page: SearchPage::default(),
        sort: ResultSort::Relevance {},
        prefer_remaster: false,
        cancelled: Some(&cancelled),
    };
    assert_eq!(
        service.configurable_similar(request).unwrap_err().kind(),
        crate::SearchErrorKind::OperationCancelled
    );
}

#[test]
fn current_baseline_preserves_the_direct_record_algorithm() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let key = "hazards:aaaaaaaaaaaaaaaa".parse().unwrap();
    let direct = service
        .similar_records(crate::SimilarRecordRequest {
            content_scope: atlas_domain::SearchContentScope::All,
            sort: ResultSort::Relevance {},
            search_profile: None,
            record_key: &key,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            prefer_remaster: false,
        })
        .unwrap()
        .unwrap();
    let seed = SimilaritySeed::Record { record_key: key };
    let configured = service
        .configurable_similar(ConfigurableSimilarRequest {
            source_sections: None,
            content_scope: atlas_domain::SearchContentScope::All,
            seed: &seed,
            options: &SimilarityOptions::Baseline {},
            search_profile: None,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            sort: ResultSort::Relevance {},
            prefer_remaster: false,
            cancelled: None,
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        configured
            .results
            .records
            .iter()
            .map(|row| (&row.record.key, row.semantic_similarity))
            .collect::<Vec<_>>(),
        direct
            .results
            .records
            .iter()
            .map(|row| (&row.record.key, row.semantic_similarity))
            .collect::<Vec<_>>()
    );
    assert!(configured.evidence.is_empty());
}

#[test]
fn trait_label_only_parts_use_stored_chunks_without_fabricating_record_navigation() {
    let mut input = fixture();
    let key = atlas_domain::TraitKey::new("fire").unwrap();
    input.traits.push(atlas_index::TraitDefinition {
        key: key.clone(),
        label: "Fire".into(),
        label_key: None,
        description_key: None,
        description_html: None,
        authored_markup: None,
        prepared: None,
        catalogs: vec![],
        source_locations: vec![],
        source_revision: None,
        indexing_locale: "en".into(),
    });
    let mut vector = vec![0.; 768];
    vector[0] = 1.;
    let chunk_hash = hash(b"Fire\n");
    input.trait_vectors.push(atlas_index::TraitVectorInput {
        key: key.clone(),
        profile_id: SearchProfileId::new("bge-base"),
        input_hash: hash(format!("trait-definition-centroid/v1:{chunk_hash}").as_bytes()),
        vector: vector.clone(),
        chunks: vec![atlas_index::TraitSeedChunk {
            input: "Fire\n".into(),
            input_hash: chunk_hash.clone(),
            token_count: 2,
            vector,
            address: SourcePassageAddress::PlainSection {
                source_text_sha256: chunk_hash,
                selection_version: "trait-definition-centroid/v1".into(),
                section_ordinal: 0,
                label: Some("Fire".into()),
                chunk_bytes: atlas_domain::SourceByteRange { start: 0, end: 5 },
            },
        }],
    });
    let (service, _artifact) = open_input_fixture(input).unwrap();
    let seed = SimilaritySeed::Trait { trait_key: key };
    let inventory = service.similarity_parts(&seed, None).unwrap();
    assert!(inventory.parts[0].part().unwrap().label_only);
    let result = service
        .configurable_similar(ConfigurableSimilarRequest {
            source_sections: None,
            content_scope: atlas_domain::SearchContentScope::All,
            seed: &seed,
            options: &SimilarityOptions::default(),
            search_profile: None,
            filter: None,
            scope: RecordScope::All,
            page: SearchPage::default(),
            sort: ResultSort::Relevance {},
            prefer_remaster: false,
            cancelled: None,
        })
        .unwrap()
        .unwrap();
    let pair = &result.evidence[0].1.pairs[0];
    assert_eq!(result.diagnostics.seed_units, 1);
    assert!(
        result
            .evidence
            .iter()
            .flat_map(|(_, evidence)| &evidence.parts)
            .all(|part| !part.part.is_identity())
    );
    assert!(pair.seed.is_none());
    assert_eq!(pair.seed_text, "Fire");
    let core = service
        .configurable_similar(ConfigurableSimilarRequest {
            source_sections: Some(SimilaritySourceSections::Core),
            content_scope: SearchContentScope::Core,
            seed: &seed,
            options: &SimilarityOptions::default(),
            search_profile: None,
            filter: None,
            scope: RecordScope::All,
            page: Default::default(),
            sort: ResultSort::Relevance {},
            prefer_remaster: false,
            cancelled: None,
        })
        .unwrap()
        .unwrap();
    assert_eq!(core.diagnostics.seed_units, 1);
    assert_eq!(
        core.core.unwrap().representation,
        atlas_domain::CoreSectionRepresentation::TraitLabelOnly
    );
    assert!(
        core.evidence
            .iter()
            .flat_map(|(_, evidence)| &evidence.parts)
            .all(|part| !part.part.is_identity())
    );
}

#[test]
fn page_explanations_share_prepared_fields_and_plain_snapshots_and_reuse_candidate_witnesses() {
    let records = ["aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb", "cccccccccccccccc"].into_iter().map(|id| {
        record("characters", "Actor", json!({
            "_id":id,"name":id,"type":"character","system":{"details":{"biography":{
                "appearance":"<p>A tall traveler</p>","attitude":"Patient with travelers","beliefs":"Travel brings wisdom",
                "visibility":{"appearance":true,"personality":true}
            }}}
        }))
    }).collect();
    let mut input = input(records, true);
    for unit in &mut input.semantic_units {
        unit.vector.fill(0.);
        if matches!(unit.address, SourcePassageAddress::HtmlSection { .. }) {
            unit.vector[0] = 0.8;
            unit.vector[1] = 0.6;
        } else {
            unit.vector[0] = 1.;
        }
    }
    let (service, _artifact) = open_input_fixture(input).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "characters:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let options = SimilarityOptions::Passages {
        decay: 0.5,
        parts: Some(
            service
                .similarity_parts(&seed, None)
                .unwrap()
                .parts
                .iter()
                .filter(|component| !component.is_identity())
                .map(|component| component.selector().clone())
                .collect(),
        ),
        candidate_parts: SimilarityCandidateParts::Any,
    };
    let request = || ConfigurableSimilarRequest {
        source_sections: None,
        content_scope: atlas_domain::SearchContentScope::All,
        seed: &seed,
        options: &options,
        search_profile: None,
        filter: None,
        scope: RecordScope::All,
        page: SearchPage::default(),
        sort: ResultSort::Relevance {},
        prefer_remaster: false,
        cancelled: None,
    };
    service.index.reset_read_metrics();
    let result = service.configurable_similar(request()).unwrap().unwrap();
    assert_eq!(result.results.records.len(), 2);
    let metrics = service.index.read_metrics();
    assert_eq!(
        metrics.prepared_content_batches, 1,
        "the repeated HTML seed is read once for the whole page"
    );
    assert_eq!(
        metrics.source_body_decodes, 3,
        "seed and both candidates each decode once, even with several plain parts"
    );
    for row in &result.results.records {
        let evidence = &result
            .evidence
            .iter()
            .find(|(key, _)| key == &row.record.key)
            .unwrap()
            .1;
        assert_eq!(row.matches.len(), evidence.pairs.len());
        for (witness, pair) in row.matches.iter().zip(&evidence.pairs) {
            assert_eq!(witness.snippet, pair.candidate.snippet);
            assert_eq!(witness.location, pair.candidate.location);
            assert!(!pair.seed_text.is_empty());
        }
    }
    // Exercise the same request hydration with repeated seed/candidate locations,
    // as when many results explain themselves using the same selected passages.
    let originals = result
        .evidence
        .iter()
        .flat_map(|(_, evidence)| &evidence.pairs)
        .flat_map(|pair| pair.seed.iter().chain(std::iter::once(&pair.candidate)))
        .cloned()
        .collect::<Vec<_>>();
    let mut repeated = (0..40).flat_map(|_| originals.clone()).collect::<Vec<_>>();
    for witness in &mut repeated {
        witness.snippet = None;
    }
    service.index.reset_read_metrics();
    service
        .hydrate_similarity_witnesses(&mut repeated.iter_mut().collect::<Vec<_>>(), &request())
        .unwrap();
    let metrics = service.index.read_metrics();
    assert_eq!(
        metrics.sql_statements, 3,
        "two prepared-content statements plus one batched source-body statement, independent of repeated pairs"
    );
    assert_eq!(metrics.prepared_content_batches, 1);
    assert_eq!(metrics.source_body_decodes, 3);
    assert!(repeated.iter().all(|witness| {
        witness
            .snippet
            .as_ref()
            .is_some_and(|text| !text.is_empty())
    }));
}
