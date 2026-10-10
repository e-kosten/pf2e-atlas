//! Explicit local evidence through the real producer and checked public readers.
use atlas_domain::RecordKey;
use atlas_index::SqliteIndexReader;
use atlas_ingest::{BuildArtifactOptions, build_artifact, load_foundry_documents};
use atlas_search::{
    AtlasRetrievalService, RecordScope, RetrievalMode, SearchEmbeddingConfig, SearchPage,
    TextSearchRequest,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

fn primary() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("scratch/search-projection/queries.json").is_file())
        .unwrap()
        .into()
}
fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let dest = target.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &dest)
        } else {
            fs::copy(entry.path(), dest).unwrap();
        }
    }
}
#[test]
#[ignore = "requires pinned local corpus/model and judged sample; run explicitly"]
fn real_producer_judged_retrieval_and_witness_evidence() {
    let primary = primary();
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let evidence = workspace.join("scratch/retrieval-validation");
    fs::create_dir_all(&evidence).unwrap();
    let sample: Value = serde_json::from_slice(
        &fs::read(primary.join("scratch/search-projection/sample-roots.json")).unwrap(),
    )
    .unwrap();
    let selected: BTreeSet<String> = sample["roots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["key"].as_str().unwrap().into())
        .collect();
    assert_eq!(selected.len(), 220);
    let corpus = primary.join("scratch/source-contracts-full/pf2e");
    let source = evidence.join("selected-source");
    if source.exists() {
        fs::remove_dir_all(&source).unwrap();
    }
    fs::create_dir_all(&source).unwrap();
    let loaded = load_foundry_documents(&corpus, None).unwrap();
    let manifest = source.join(loaded.metadata.manifest_path.strip_prefix(&corpus).unwrap());
    fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    fs::copy(&loaded.metadata.manifest_path, manifest).unwrap();
    copy_tree(&corpus.join("static/lang"), &source.join("static/lang"));
    let migrations = corpus.join("src/module/migration");
    if migrations.is_dir() {
        copy_tree(&migrations, &source.join("src/module/migration"));
    }
    let mut copied = 0;
    for pack in loaded.packs {
        fs::create_dir_all(source.join(pack.metadata.resolved_path.strip_prefix(&corpus).unwrap()))
            .unwrap();
        for document in pack.documents {
            let raw: Value = serde_json::from_slice(&document.bytes).unwrap();
            let key = format!("{}:{}", pack.metadata.name, raw["_id"].as_str().unwrap());
            if selected.contains(&key) {
                let target = source.join(&document.provenance.source_path);
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::write(target, &document.bytes).unwrap();
                copied += 1;
            }
        }
    }
    assert_eq!(copied, 220);
    let cache = primary.join(".cache/hf-models");
    let output = evidence.join("judged-current.sqlite");
    let report = build_artifact(BuildArtifactOptions {
        source_root: source,
        output_path: output.clone(),
        manifest_path: None,
        locale: "en".into(),
        embedding: Some(atlas_embedding::EmbeddingRuntimeConfig::default_model(
            &cache,
        )),
        reuse_embeddings: true,
        embedding_batch_size: 32,
    })
    .unwrap();
    assert_eq!(report.record_count, 220);
    let validation = atlas_index::validate_artifact(&output).unwrap();
    let reader = SqliteIndexReader::open_read_only(&output).unwrap();
    let context = reader.context().clone();
    let statistics = reader.statistics().unwrap();
    let mut service = AtlasRetrievalService::from_prepared_index(
        reader,
        &SearchEmbeddingConfig::new(atlas_embedding::DEFAULT_EMBEDDING_MODEL, &cache),
    )
    .unwrap();
    let queries: Value = serde_json::from_slice(
        &fs::read(primary.join("scratch/search-projection/queries.json")).unwrap(),
    )
    .unwrap();
    let mut observations = vec![];
    for q in queries["queries"].as_array().unwrap() {
        if q.get("pair_fixture").is_some() {
            observations.push(json!({"id":q["id"],"status":"synthetic policy fixture excluded from source proof","basis":q["pair_fixture"]}));
            continue;
        }
        let keys: Vec<RecordKey> = q["scope"]["keys"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|k| k.as_str().unwrap().parse().unwrap())
            .collect();
        let mut clauses = vec![];
        if let Some(family) = q["scope"]["type"].as_str() {
            clauses.push(format!("source.type == '{family}'"));
        }
        if let Some(pack) = q["scope"]["pack"].as_str() {
            clauses.push(format!("source.pack.id == '{pack}'"));
        }
        if let Some(kind) = q["scope"]["kind"].as_str() {
            clauses.push(format!("source.document_kind == '{kind}'"));
        }
        if let Some(level) = q["scope"]["maxLevel"].as_i64() {
            clauses.push(format!("actor.level <= {level}"));
        }
        let filter = if clauses.is_empty() {
            None
        } else {
            Some(atlas_index::parse_where(&clauses.join(" && ")).unwrap())
        };
        for mode in [
            RetrievalMode::Fts,
            RetrievalMode::Vector,
            RetrievalMode::Hybrid,
        ] {
            let started = Instant::now();
            let result = service
                .search_text(TextSearchRequest {
                    query: q["text"].as_str().unwrap(),
                    mode,
                    filter: filter.as_ref().map(|f| f.predicate()),
                    scope: if keys.is_empty() {
                        RecordScope::All
                    } else {
                        RecordScope::Keys(&keys)
                    },
                    page: SearchPage::new(1, 30).unwrap(),
                    prefer_remaster: true,
                })
                .unwrap();
            let elapsed = started.elapsed().as_millis();
            let expected = q["expected"][0].as_str().unwrap();
            let rank = result
                .records
                .iter()
                .position(|r| r.record.key.to_string() == expected)
                .map(|r| r + 1);
            if mode != RetrievalMode::Fts {
                assert!(
                    rank.is_some_and(|rank| rank <= 3),
                    "{} {:?}: expected {} in first three, got {:?}",
                    q["id"],
                    mode,
                    expected,
                    rank
                );
            }
            let reader = SqliteIndexReader::open_read_only(&output).unwrap();
            let mut rows = vec![];
            for row in result.records.iter().take(3).chain(
                result
                    .records
                    .iter()
                    .filter(|r| r.record.key.to_string() == expected)
                    .filter(|r| {
                        !result
                            .records
                            .iter()
                            .take(3)
                            .any(|top| top.record.key == r.record.key)
                    }),
            ) {
                let mut witnesses = vec![];
                for witness in &row.matches {
                    let recovered = if let (Some(field), Some(address)) =
                        (&witness.location.field, &witness.location.address)
                    {
                        let locator = atlas_record::source_content::SourceContentLocator {
                            record: witness.location.record.clone(),
                            owners: witness.location.owners.clone(),
                            field: field.clone(),
                        };
                        let content = reader.read_content(std::slice::from_ref(&locator)).unwrap();
                        let html = content.fields.first().and_then(|f| f.html.as_deref());
                        let source = reader.read_source_record(&locator.record).unwrap().unwrap();
                        let text = source.authored_content_at(&locator);
                        Some(
                            atlas_record::source_record::recover_source_passage(
                                html,
                                text.value(),
                                address,
                            )
                            .unwrap()
                            .to_owned(),
                        )
                    } else {
                        None
                    };
                    witnesses.push(json!({"witness":witness,"recovered_text":recovered}));
                }
                rows.push(json!({"key":row.record.key,"score":row.score,"witnesses":witnesses}));
            }
            observations.push(json!({"id":q["id"],"query":q["text"],"mode":mode,"expected":expected,"expected_rank":rank,"elapsed_ms":elapsed,"coverage":result.coverage,"rows":rows}));
        }
    }
    let build = json!({"record_count":report.record_count,"semantic_unit_count":report.semantic_unit_count,"inferred_inputs":report.inferred_inputs,"reused_inputs":report.reused_inputs,"duration_ms":report.build_duration_ms,"source_fingerprint":report.source_fingerprint});
    fs::write(evidence.join("judged-current-evidence.json"),serde_json::to_vec_pretty(&json!({"basis":"Current production ingest, real pinned inference, checked reader/search; unjudged results are not declared irrelevant. Synthetic remaster pair remains a separate policy fixture; baseline study files are unchanged.","build":build,"validation":validation,"context":context,"statistics":statistics,"observations":observations})).unwrap()).unwrap();
    println!("retrieval evidence saved to {}", evidence.display());
}
