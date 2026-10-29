//! Explicit full-corpus experiment; no inference or writes to the selected index.
use crate::{BuildArtifactOptions, index_build_input::PreparedBuildInput};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};

fn signatures(prepared: &PreparedBuildInput) -> BTreeMap<atlas_domain::RecordKey, String> {
    let mut records: BTreeMap<_, Sha256> = BTreeMap::new();
    for pending in &prepared.pending {
        let hash = records.entry(pending.unit.record.clone()).or_default();
        let metadata = serde_json::to_vec(&(
            &pending.unit,
            &pending.input.reuse_key,
            pending.input.token_count,
            pending.input.body_token_count,
            pending.input.overlap_token_count,
            pending.input.context_shortened,
        ))
        .unwrap();
        for bytes in [metadata.as_slice(), pending.input.input.as_bytes()] {
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
    }
    records
        .into_iter()
        .map(|(key, hash)| (key, format!("{:x}", hash.finalize())))
        .collect()
}

fn native_signatures(options: &BuildArtifactOptions, prepared: &PreparedBuildInput) -> Vec<String> {
    options
        .search_profiles
        .iter()
        .map(|config| {
            let tokenizer = atlas_embedding::TextEmbeddingTokenizer::load(config).unwrap();
            let batch = atlas_embedding::PreparedEmbeddingBatch::prepare(
                &tokenizer,
                prepared
                    .pending
                    .iter()
                    .map(|pending| pending.input.clone())
                    .chain(
                        prepared
                            .trait_pending
                            .iter()
                            .map(|pending| pending.input.clone()),
                    )
                    .collect(),
            )
            .unwrap();
            let mut hash = Sha256::new();
            for input in batch.inputs() {
                hash.update(
                    serde_json::to_vec(&(&input.reuse_key, &input.input_sha256, input.token_count))
                        .unwrap(),
                );
            }
            format!("{:x}", hash.finalize())
        })
        .collect()
}

#[test]
#[ignore = "full-corpus release benchmark; explicit source, models and private index required"]
fn published_layout_reuse_full_corpus_benchmark() {
    let source = PathBuf::from(std::env::var_os("ATLAS_TEST_SOURCE").expect("ATLAS_TEST_SOURCE"));
    let models =
        PathBuf::from(std::env::var_os("ATLAS_TEST_MODEL_CACHE").expect("ATLAS_TEST_MODEL_CACHE"));
    let index = PathBuf::from(std::env::var_os("ATLAS_TEST_INDEX").expect("ATLAS_TEST_INDEX"));
    let schema: u32 = std::env::var("ATLAS_TEST_SCHEMA")
        .expect("ATLAS_TEST_SCHEMA")
        .parse()
        .unwrap();
    let evidence =
        PathBuf::from(std::env::var_os("ATLAS_TEST_EVIDENCE").expect("ATLAS_TEST_EVIDENCE"));
    let edit_relative =
        std::env::var("ATLAS_TEST_EDIT_RELATIVE").expect("private source edit path required");
    let edit_path = source.join(&edit_relative);
    assert!(
        edit_path
            .canonicalize()
            .unwrap()
            .starts_with(source.canonicalize().unwrap())
    );
    struct RestoreSource(PathBuf, Vec<u8>);
    impl Drop for RestoreSource {
        fn drop(&mut self) {
            std::fs::write(&self.0, &self.1).unwrap();
        }
    }
    let original = RestoreSource(
        edit_path,
        std::fs::read(source.join(&edit_relative)).unwrap(),
    );
    let mut edited: serde_json::Value = serde_json::from_slice(&original.1).unwrap();
    let description = edited["system"]["description"]["value"]
        .as_str()
        .unwrap()
        .to_owned();
    edited["system"]["description"]["value"] = serde_json::json!(format!(
        "{description}<p>Bounded preparation benchmark update.</p>"
    ));
    let options = BuildArtifactOptions {
        source_root: source,
        output_path: index.clone(),
        manifest_path: None,
        locale: "en".into(),
        search_profiles: atlas_embedding::SEARCH_PROFILES
            .iter()
            .map(|profile| {
                atlas_embedding::EmbeddingRuntimeConfig::new(profile.model, models.clone())
            })
            .collect(),
        reuse_embeddings: true,
        embedding_batch_size: 32,
    };
    let workspace = atlas_storage::Workspace::at(&index).unwrap();
    let read = workspace.read(schema).unwrap();
    let context = atlas_index::read_stored_reference_context(read.connection())
        .unwrap()
        .unwrap();
    let mut report = vec![];
    let mut expected: Option<BTreeMap<atlas_domain::RecordKey, String>> = None;
    let mut expected_native = None;
    for case in ["forced_fresh", "unchanged_replay", "small_source_update"] {
        if case == "small_source_update" {
            std::fs::write(&original.0, serde_json::to_vec(&edited).unwrap()).unwrap();
        }
        let started = Instant::now();
        let previous = atlas_index::read_persisted_inputs(read.connection(), &context).unwrap();
        let changed_key = previous
            .iter()
            .find(|record| record.source_path == edit_relative)
            .expect("edit must address a published record")
            .record
            .key()
            .clone();
        let layouts = if case != "forced_fresh" {
            atlas_index::read_reusable_passage_layouts(&read).unwrap()
        } else {
            BTreeMap::new()
        };
        let layout_count: usize = layouts.values().map(Vec::len).sum();
        let prepared = crate::build::prepare_build(&options, previous, layouts).unwrap();
        let preparation_seconds = started.elapsed().as_secs_f64();
        let signatures = signatures(&prepared.prepared);
        let native_started = Instant::now();
        let native = native_signatures(&options, &prepared.prepared);
        let native_seconds = native_started.elapsed().as_secs_f64();
        if let Some(expected) = &expected {
            assert_eq!(signatures.len(), expected.len());
            for (key, signature) in &signatures {
                if case == "small_source_update" && key == &changed_key {
                    continue;
                }
                assert_eq!(Some(signature), expected.get(key), "{key}");
            }
            if case == "small_source_update" {
                assert_ne!(signatures[&changed_key], expected[&changed_key]);
                let record = prepared
                    .prepared
                    .index
                    .records
                    .iter()
                    .find(|record| record.record.key() == &changed_key)
                    .unwrap();
                let pack = prepared
                    .prepared
                    .index
                    .packs
                    .iter()
                    .find(|pack| pack.pack_id == changed_key.pack().as_str())
                    .unwrap();
                let mut selection = atlas_record::source_record::select_record_search(
                    &record.record,
                    &record.content,
                    prepared.prepared.index.context.audience,
                    &pack.label,
                    &prepared.prepared.index.context.used_trait_labels,
                )
                .unwrap();
                selection.identity_optional.extend(
                    prepared
                        .prepared
                        .index
                        .aliases
                        .iter()
                        .filter(|alias| alias.record == changed_key)
                        .map(|alias| alias.alias.clone()),
                );
                let tokenizer =
                    atlas_embedding::TextEmbeddingTokenizer::load_baseline(&models).unwrap();
                let mut fresh = vec![];
                crate::index_build_input::semantic_units(
                    &selection,
                    &tokenizer,
                    &mut fresh,
                    &mut 0,
                    &[],
                )
                .unwrap();
                assert_eq!(
                    fresh,
                    prepared
                        .prepared
                        .pending
                        .iter()
                        .filter(|pending| pending.unit.record == changed_key)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
        } else {
            expected = Some(signatures);
        }
        if let Some(expected) = &expected_native {
            if case == "small_source_update" {
                assert_ne!(&native, expected);
            } else {
                assert_eq!(&native, expected);
            }
        } else {
            expected_native = Some(native);
        }
        let entry = serde_json::json!({"case":case,"preparation_seconds":preparation_seconds,"native_validation_seconds":native_seconds,"layout_count":layout_count,"reused_sections":prepared.prepared.reused_sections,"records":prepared.prepared.index.records.len(),"document_inputs":prepared.prepared.pending.len(),"trait_inputs":prepared.prepared.trait_pending.len(),"source_fingerprint":prepared.fingerprint.value,"context":prepared.prepared.index.context.sha256().unwrap()});
        eprintln!("{entry}");
        report.push(entry);
        std::fs::write(&evidence, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
}
