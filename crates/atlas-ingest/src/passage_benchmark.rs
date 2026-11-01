//! Explicit full-corpus experiment; no inference or writes to the selected index.
use crate::{
    BuildArtifactOptions,
    index_build_input::DocumentAttribution,
    product_preparation::{PreparedSourceBuild, RawOccurrence},
};
use atlas_domain::RecordKey;
use atlas_embedding::{PreparedEmbeddingBatch, PreparedNativeInput, TextEmbeddingTokenizer};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};

pub(super) struct NativeEvidence {
    pub signature: String,
    pub token_count: usize,
}

pub(super) fn native_signature(input: &PreparedNativeInput) -> String {
    format!("{:x}", Sha256::digest(serde_json::to_vec(input).unwrap()))
}

/// Retain only hashes/counts; model text is released after each stream callback.
pub(super) fn native_evidence(prepared: &PreparedSourceBuild) -> Vec<NativeEvidence> {
    let mut output = Vec::new();
    prepared
        .native_inputs
        .visit(&mut |input| {
            output.push(NativeEvidence {
                signature: native_signature(&input),
                token_count: input.token_count,
            });
            Ok(())
        })
        .unwrap();
    output
}

pub(super) fn update_document_signature(
    hash: &mut Sha256,
    attribution: &DocumentAttribution,
    input_signature: &str,
) {
    let metadata = serde_json::to_vec(&(attribution, input_signature)).unwrap();
    hash.update((metadata.len() as u64).to_le_bytes());
    hash.update(metadata);
}

pub(super) fn signatures(prepared: &PreparedSourceBuild) -> BTreeMap<RecordKey, String> {
    let inputs = native_evidence(prepared);
    let mut records = BTreeMap::<RecordKey, Sha256>::new();
    prepared
        .occurrences
        .visit(&mut |occurrence| {
            if let RawOccurrence::Document { input, attribution } = occurrence {
                let hash = records.entry(attribution.record.clone()).or_default();
                update_document_signature(hash, &attribution, &inputs[input].signature);
            }
            Ok(())
        })
        .unwrap();
    records
        .into_iter()
        .map(|(key, hash)| (key, format!("{:x}", hash.finalize())))
        .collect()
}

/// Independently reselect each source record before calling this fresh, layout-free oracle.
pub(super) fn fresh_record_signature(
    selection: &atlas_record::source_record::SourceSearchSelection,
    tokenizer: &TextEmbeddingTokenizer,
) -> Option<String> {
    let mut hash = Sha256::new();
    let mut count = 0usize;
    crate::index_build_input::semantic_units(
        selection,
        tokenizer,
        &mut |attribution, input| {
            assert!(input.token_count <= 512);
            assert!(
                input.body_token_count <= 256
                    || matches!(
                        attribution.address,
                        atlas_domain::SourcePassageAddress::Identity {}
                    )
            );
            update_document_signature(&mut hash, &attribution, &native_signature(&input.into()));
            count += 1;
            Ok(())
        },
        &mut 0,
        &[],
    )
    .unwrap();
    (count > 0).then(|| format!("{:x}", hash.finalize()))
}

fn native_signatures(
    options: &BuildArtifactOptions,
    prepared: &PreparedSourceBuild,
) -> Vec<String> {
    options
        .search_profiles
        .iter()
        .map(|config| {
            let tokenizer = TextEmbeddingTokenizer::load(config).unwrap();
            let mut hash = Sha256::new();
            let limit = options
                .embedding_batch_size
                .clamp(1, atlas_embedding::NATIVE_INFERENCE_BATCH_LIMIT);
            let mut pending = Vec::with_capacity(limit);
            let mut flush = |pending: &mut Vec<PreparedNativeInput>| {
                let batch =
                    PreparedEmbeddingBatch::prepare(&tokenizer, std::mem::take(pending)).unwrap();
                for input in batch.inputs() {
                    let metadata = serde_json::to_vec(&(
                        &input.reuse_key,
                        &input.input_sha256,
                        input.token_count,
                    ))
                    .unwrap();
                    hash.update((metadata.len() as u64).to_le_bytes());
                    hash.update(metadata);
                }
            };
            prepared
                .native_inputs
                .visit(&mut |input| {
                    pending.push(input);
                    if pending.len() == limit {
                        flush(&mut pending);
                    }
                    Ok(())
                })
                .unwrap();
            if !pending.is_empty() {
                flush(&mut pending);
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
        reuse_passage_layouts: true,
        embedding_batch_size: 32,
    };
    let workspace = atlas_storage::Workspace::at(&index).unwrap();
    let reuse = atlas_index::ReferenceReuseReader::new(&workspace, schema).unwrap();
    let identities = reuse.identities().unwrap();
    let changed = identities
        .values()
        .filter(|record| record.source_path == edit_relative)
        .collect::<Vec<_>>();
    assert_eq!(
        changed.len(),
        1,
        "edit must address exactly one published record"
    );
    let changed_key = changed[0].key.clone();
    drop(changed);
    // Count layouts in bounded owned batches outside measured preparation.
    let keys = identities
        .values()
        .map(|record| &record.key)
        .collect::<Vec<_>>();
    let layout_count: usize = keys
        .chunks(32)
        .map(|keys| {
            let requests = keys
                .iter()
                .map(|key| atlas_index::SourceReuseRequest {
                    key,
                    record: false,
                    layouts: true,
                })
                .collect::<Vec<_>>();
            reuse
                .read_batch(&requests)
                .unwrap()
                .iter()
                .map(|r| r.layouts.len())
                .sum::<usize>()
        })
        .sum();
    drop(identities);
    let fresh_directory = tempfile::tempdir().unwrap();
    let fresh_workspace = atlas_index::test_support::initialize_workspace(
        fresh_directory.path().join("fresh.sqlite"),
    )
    .unwrap();
    let fresh_reuse = atlas_index::ReferenceReuseReader::new(
        &fresh_workspace,
        atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
    )
    .unwrap();
    let tokenizer = TextEmbeddingTokenizer::load_baseline(&models).unwrap();
    let mut report = vec![];
    let mut expected: Option<BTreeMap<RecordKey, String>> = None;
    let mut expected_native = None;
    for case in ["forced_fresh", "unchanged_replay", "small_source_update"] {
        if case == "small_source_update" {
            std::fs::write(&original.0, serde_json::to_vec(&edited).unwrap()).unwrap();
        }
        let mut case_options = options.clone();
        let selected_reuse = if case == "forced_fresh" {
            case_options.output_path = fresh_workspace.path().to_owned();
            case_options.reuse_passage_layouts = false;
            &fresh_reuse
        } else {
            &reuse
        };
        let started = Instant::now();
        let prepared = crate::product_preparation::prepare(&case_options, selected_reuse).unwrap();
        let preparation_seconds = started.elapsed().as_secs_f64();
        let signatures = signatures(&prepared);
        let native_started = Instant::now();
        let native = native_signatures(&options, &prepared);
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
                let mut aliases = Vec::new();
                prepared
                    .spool
                    .aliases
                    .visit(&mut |alias| {
                        if alias.record == changed_key {
                            aliases.push(alias.alias);
                        }
                        Ok(())
                    })
                    .unwrap();
                let mut checked = false;
                prepared
                    .spool
                    .records
                    .visit(&mut |frame| {
                        if frame.selection.root != changed_key {
                            return Ok(());
                        }
                        let (source, _, _, _) = frame.decode()?;
                        let pack = prepared
                            .spool
                            .packs
                            .iter()
                            .find(|pack| pack.pack_id == changed_key.pack().as_str())
                            .unwrap();
                        let mut selection = atlas_record::source_record::select_record_search(
                            &source.record,
                            &source.content,
                            prepared.spool.context.audience,
                            &pack.label,
                            &prepared.spool.context.used_trait_labels,
                        )
                        .unwrap();
                        selection.identity_optional.extend(aliases.iter().cloned());
                        assert_eq!(
                            fresh_record_signature(&selection, &tokenizer).as_ref(),
                            signatures.get(&changed_key)
                        );
                        checked = true;
                        Ok(())
                    })
                    .unwrap();
                assert!(checked);
            } else {
                assert!(prepared.reused_sections > 0);
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
        let entry = serde_json::json!({"case":case,"preparation_seconds":preparation_seconds,"native_validation_seconds":native_seconds,"layout_count":if case=="forced_fresh"{0}else{layout_count},"layout_count_timing":"bounded inventory before measured preparation","reused_sections":prepared.reused_sections,"records":prepared.spool.records.len(),"document_inputs":prepared.document_units,"trait_inputs":prepared.occurrences.len()-prepared.document_units,"source_fingerprint":prepared.fingerprint.value,"context":prepared.spool.context.sha256().unwrap()});
        eprintln!("{entry}");
        report.push(entry);
        std::fs::write(&evidence, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    let read = workspace.read(schema).unwrap();
    assert_eq!(
        read.workspace_key().unwrap(),
        reuse.expected().workspace_key()
    );
    assert_eq!(
        read.reference_revision().unwrap(),
        reuse.expected().reference_revision()
    );
}
