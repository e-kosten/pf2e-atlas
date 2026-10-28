use std::collections::BTreeMap;
use std::path::PathBuf;

use atlas_domain::{QueryPredicate, RecordKey, SourcePassageAddress};
use atlas_embedding::{
    EmbeddingModelId, EmbeddingRuntimeConfig, TextEmbedder, hash_document_embedding_input,
};
use atlas_record::source_record::select_record_search;
use serde_json::json;

use crate::test_support::{audience, input, open_input_fixture, record};
use crate::{RecordScope, RetrievalMode, SearchPage, SimilarRecordRequest, TextSearchRequest};

fn local_model_config() -> EmbeddingRuntimeConfig {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|parent| {
            EmbeddingRuntimeConfig::new(
                EmbeddingModelId::BgeBaseEnV15,
                parent.join(".cache/hf-models"),
            )
        })
        .find(|config| config.model_dir().join("onnx/model.onnx").is_file())
        .expect("this ignored native test requires the verified local BGE asset cache")
}

#[test]
#[ignore = "requires the pinned local BGE model and native runtime; run explicitly"]
fn actual_bge_pages_invoke_query_embedding_once_each_without_cross_page_reuse() {
    let config = local_model_config();
    let model_dir = config.model_dir();
    let mut model = TextEmbedder::load_from_model_dir(config.model_spec(), &model_dir)
        .expect("pinned assets must validate before native inference");
    let mut fixture = input(
        (0..4)
            .map(|level| {
                record(
                    "items",
                    "Item",
                    json!({
                        "_id": format!("{level:016}"),
                        "name": format!("Ghoul fixture {level}"),
                        "type": "effect",
                        "system": {"level": {"value": level}, "traits": {"value": []}}
                    }),
                )
            })
            .collect(),
        true,
    );
    // This fixture has only root identity units. Replace synthetic fixture
    // vectors and token metadata with actual pinned document inference.
    assert_eq!(fixture.semantic_units.len(), 4);
    for unit in &mut fixture.semantic_units {
        assert!(matches!(unit.address, SourcePassageAddress::Identity {}));
        let source = fixture
            .records
            .iter()
            .find(|record| record.record.key() == &unit.record)
            .unwrap();
        let text = select_record_search(
            &source.record,
            &source.content,
            audience(),
            "items",
            &BTreeMap::new(),
        )
        .unwrap()
        .identity_required;
        unit.input_token_count = model.tokenizer().validate_document_input(&text).unwrap();
        unit.input_hash = hash_document_embedding_input(&text);
        unit.vector = model.embed_document(&text).unwrap();
    }
    let (service, artifact) = open_input_fixture(fixture).unwrap();
    drop(service);
    let inference = atlas_embedding::SharedQueryInference::new(&config.cache_root);
    let filter: QueryPredicate = serde_json::from_value(json!({
        "kind":"compare", "field":"level", "op":"gte", "value":1
    }))
    .unwrap();
    let eligible = (1..4)
        .map(|id| RecordKey::parse(&format!("items:{id:016}")).unwrap())
        .collect::<Vec<_>>();
    eprintln!(
        "pinned BGE fixture: assets={}; identity_units=4; embed_query invocations=0",
        model_dir.display()
    );

    let mut expected_invocations = 0;
    for mode in [RetrievalMode::Vector, RetrievalMode::Hybrid] {
        let mut previous_key = None;
        for number in [1, 2] {
            let profile = atlas_domain::SearchProfileId::new("bge-base");
            let prepared =
                crate::PreparedQueryEmbedding::prepare(&inference, &profile, "Ghoul").unwrap();
            let mut service = crate::AtlasRetrievalService::from_index(
                atlas_index::test_support::open_reader(artifact.path()).unwrap(),
                profile,
                Some(prepared),
            )
            .unwrap();
            let result = service
                .search_text(TextSearchRequest {
                    content_scope: atlas_domain::SearchContentScope::All,
                    sort: atlas_domain::ResultSort::Relevance {},
                    ranking: Default::default(),
                    search_profile: None,
                    query: "Ghoul",
                    mode,
                    filter: Some(&filter),
                    scope: RecordScope::All,
                    page: SearchPage::new(number, 1).unwrap(),
                    prefer_remaster: true,
                })
                .unwrap();
            expected_invocations += 1;
            assert_eq!(result.records.len(), 1);
            assert_eq!(result.page.total, 3);
            assert_eq!(result.page.next_page, Some(number + 1));
            assert!(!result.coverage.exhaustive);
            assert_eq!(result.coverage.candidate_roots, 3);
            assert!(eligible.contains(&result.records[0].record.key));
            assert_ne!(previous_key.as_ref(), Some(&result.records[0].record.key));
            previous_key = Some(result.records[0].record.key.clone());
            eprintln!(
                "{} page{}: embed_query invocations={}; eligible_total={}; selected={}",
                mode.as_str(),
                number,
                expected_invocations,
                result.page.total,
                result.records[0].record.key
            );
        }
    }

    let mut service = crate::AtlasRetrievalService::from_index(
        atlas_index::test_support::open_reader(artifact.path()).unwrap(),
        atlas_domain::SearchProfileId::new("bge-base"),
        None,
    )
    .unwrap();
    let lexical = service
        .search_text(TextSearchRequest {
            content_scope: atlas_domain::SearchContentScope::All,
            sort: atlas_domain::ResultSort::Relevance {},
            ranking: Default::default(),
            search_profile: None,
            query: "Ghoul",
            mode: RetrievalMode::Fts,
            filter: Some(&filter),
            scope: RecordScope::All,
            page: SearchPage::first(10).unwrap(),
            prefer_remaster: true,
        })
        .unwrap();
    assert_eq!(lexical.page.total, 3);
    assert!(lexical.coverage.exhaustive);
    assert!(
        lexical
            .records
            .iter()
            .all(|row| eligible.contains(&row.record.key))
    );
    assert_eq!(expected_invocations, 4);
    eprintln!("fts: embed_query invocations=4; eligible_total=3; no query embedding");

    let seed = &eligible[0];
    let similar = service
        .similar_records(SimilarRecordRequest {
            content_scope: atlas_domain::SearchContentScope::All,
            sort: atlas_domain::ResultSort::Relevance {},
            search_profile: None,
            record_key: seed,
            filter: Some(&filter),
            scope: RecordScope::All,
            page: SearchPage::first(10).unwrap(),
            prefer_remaster: true,
        })
        .unwrap()
        .unwrap();
    assert_eq!(&similar.seed.key, seed);
    assert_eq!(similar.results.page.total, 2);
    assert!(!similar.results.coverage.exhaustive);
    assert!(
        similar
            .results
            .records
            .iter()
            .all(|row| row.record.key != *seed && eligible.contains(&row.record.key))
    );
    assert_eq!(expected_invocations, 4);
    eprintln!(
        "stored-vector Similar: embed_query invocations=4; eligible_total=2; seed excluded; no query embedding"
    );
}
