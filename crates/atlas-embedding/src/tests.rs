use crate::*;
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use std::path::PathBuf;
use tokenizers::{Tokenizer, models::wordlevel::WordLevel, pre_tokenizers::whitespace::Whitespace};

pub(crate) fn observed_progress<T>(
    action: impl FnOnce() -> T,
) -> (T, Vec<std::collections::BTreeMap<String, String>>) {
    #[derive(Default)]
    struct Fields(std::collections::BTreeMap<String, String>);
    impl tracing::field::Visit for Fields {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            self.0.insert(field.name().into(), format!("{value:?}"));
        }
        fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
            self.0.insert(field.name().into(), value.into());
        }
    }
    struct Observer(
        std::sync::Arc<std::sync::Mutex<Vec<std::collections::BTreeMap<String, String>>>>,
    );
    impl tracing::Subscriber for Observer {
        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }
        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
        fn event(&self, event: &tracing::Event<'_>) {
            if event.metadata().target() == "atlas_progress" {
                let mut fields = Fields::default();
                event.record(&mut fields);
                self.0.lock().unwrap().push(fields.0);
            }
        }
        fn enter(&self, _: &tracing::span::Id) {}
        fn exit(&self, _: &tracing::span::Id) {}
    }
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let result =
        tracing::subscriber::with_default(Observer(std::sync::Arc::clone(&events)), action);
    let events = std::mem::take(&mut *events.lock().unwrap());
    (result, events)
}

pub(crate) fn tokenizer() -> TextEmbeddingTokenizer {
    let vocab = [("[UNK]".to_owned(), 0), ("one".to_owned(), 1)]
        .into_iter()
        .collect();
    let model = WordLevel::builder()
        .vocab(vocab)
        .unk_token("[UNK]".to_owned())
        .build()
        .unwrap();
    let mut tokenizer = Tokenizer::new(model);
    tokenizer.with_pre_tokenizer(Some(Whitespace));
    TextEmbeddingTokenizer::from_tokenizer(
        crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15),
        tokenizer,
    )
    .unwrap()
}
pub(crate) fn plain<'a>(text: &'a str, context: &'a str) -> PassageSection<'a> {
    PassageSection::Plain {
        text,
        context,
        selection_version: "test/v1",
        section_ordinal: 4,
        label: Some("Fever"),
    }
}

fn layouts(inputs: &[PreparedEmbeddingInput]) -> Vec<PassageLayout> {
    inputs
        .iter()
        .map(|input| PassageLayout {
            address: input.address.clone(),
            input_sha256: input.input_sha256.clone(),
        })
        .collect()
}

#[test]
fn published_layout_replay_matches_fresh_plain_and_html_preparation() {
    let tokenizer = tokenizer();
    let text = "one 🐉 one\n".repeat(400);
    let long_context = "one ".repeat(100);
    for context in ["", "Root / owner", long_context.as_str()] {
        for section in [
            plain(&text, context),
            PassageSection::Html {
                text: &text,
                context,
                prepared_html_sha256: "html-hash",
                selection_version: "test/v1",
                section_ordinal: 7,
                label: Some("Heading"),
            },
        ] {
            let fresh = prepare_embedding_section(&tokenizer, section).unwrap();
            assert!(fresh.len() > 2);
            assert_eq!(
                reuse_embedding_section(&tokenizer, section, &layouts(&fresh)).unwrap(),
                Some(fresh)
            );
        }
    }
}

#[test]
fn changed_text_context_and_attribution_miss_published_layouts() {
    let tokenizer = tokenizer();
    let text = "one ".repeat(400);
    let original = plain(&text, "Root");
    let previous = layouts(&prepare_embedding_section(&tokenizer, original).unwrap());
    let changed_text = format!("{text}changed");
    for section in [
        plain(&changed_text, "Root"),
        plain(&text, "Renamed root / pack / localized name"),
        PassageSection::Plain {
            text: &text,
            context: "Root",
            selection_version: "changed/v2",
            section_ordinal: 4,
            label: Some("Fever"),
        },
        PassageSection::Plain {
            text: &text,
            context: "Root",
            selection_version: "test/v1",
            section_ordinal: 5,
            label: Some("Fever"),
        },
        PassageSection::Html {
            text: &text,
            context: "Root",
            prepared_html_sha256: "changed-html",
            selection_version: "test/v1",
            section_ordinal: 4,
            label: Some("Fever"),
        },
    ] {
        assert!(
            reuse_embedding_section(&tokenizer, section, &previous)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn incomplete_reordered_or_unfaithful_layouts_never_supply_partial_passages() {
    let tokenizer = tokenizer();
    let text = "one 🐉 ".repeat(400);
    let section = plain(&text, "Root");
    let original = layouts(&prepare_embedding_section(&tokenizer, section).unwrap());
    let mut corruptions = vec![
        vec![],
        original[1..].to_vec(),
        original[..original.len() - 1].to_vec(),
    ];
    let mut duplicate = original.clone();
    duplicate.insert(1, duplicate[0].clone());
    corruptions.push(duplicate);
    let mut reversed = original.clone();
    reversed.reverse();
    corruptions.push(reversed);
    let mut wrong_hash = original.clone();
    wrong_hash[0].input_sha256 = "0".repeat(64);
    corruptions.push(wrong_hash);
    let mut bad_utf8 = original.clone();
    if let SourcePassageAddress::PlainSection { chunk_bytes, .. } = &mut bad_utf8[0].address {
        chunk_bytes.end = text.find('🐉').unwrap() + 1;
    }
    corruptions.push(bad_utf8);
    for invalid in corruptions {
        assert!(
            reuse_embedding_section(&tokenizer, section, &invalid)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn replay_rechecks_body_and_overlap_budgets_even_with_matching_hashes() {
    let tokenizer = tokenizer();
    for (words, ranges) in [(400, vec![(0, 400)]), (300, vec![(0, 200), (100, 300)])] {
        let text = "one ".repeat(words);
        let section = plain(&text, "");
        let hash = hash_document_embedding_input(&text);
        let layouts = ranges
            .into_iter()
            .map(|(start, end)| PassageLayout {
                address: section
                    .address(
                        &hash,
                        SourceByteRange {
                            start: start * 4,
                            end: end * 4,
                        },
                    )
                    .unwrap(),
                input_sha256: hash_document_embedding_input(&text[start * 4..end * 4]),
            })
            .collect::<Vec<_>>();
        assert!(
            reuse_embedding_section(&tokenizer, section, &layouts)
                .unwrap()
                .is_none()
        );
    }
}
fn assert_coverage(
    tokenizer: &TextEmbeddingTokenizer,
    text: &str,
    units: &[PreparedEmbeddingInput],
) {
    let mut covered = 0;
    for unit in units {
        let SourcePassageAddress::PlainSection {
            chunk_bytes: range,
            source_text_sha256,
            selection_version,
            section_ordinal,
            label,
        } = &unit.address
        else {
            panic!("expected plain section")
        };
        assert_eq!(source_text_sha256, &hash_document_embedding_input(text));
        assert_eq!(selection_version, "test/v1");
        assert_eq!(*section_ordinal, 4);
        assert_eq!(label.as_deref(), Some("Fever"));
        assert!(range.start <= covered);
        assert!(range.end > covered);
        assert!(text.is_char_boundary(range.start) && text.is_char_boundary(range.end));
        let overlap = &text[range.start..covered];
        assert!(tokenizer.count_tokens(overlap, false).unwrap() <= OVERLAP_TOKEN_BUDGET);
        assert!(unit.body_token_count <= BODY_TOKEN_BUDGET);
        assert!(unit.token_count <= 512);
        let body = &text[range.start..range.end];
        assert!(unit.input.ends_with(body));
        covered = range.end;
    }
    assert_eq!(covered, text.len());
}
#[test]
fn splitting_covers_all_source_bytes_without_rewriting_unicode() {
    let tokenizer = tokenizer();
    for text in [
        "First paragraph.\n\nSecond paragraph.\n".repeat(100),
        "• Résumé ≥ ½; 2d6[fire] + @item.level.\n火 👩‍🚀 e\u{301}\n".repeat(120),
        "Name | Result\nFailure | Damage\nSuccess | No effect\n".repeat(80),
        "one ".repeat(2100) + "完整 tail",
    ] {
        let units = prepare_embedding_section(&tokenizer, plain(&text, "root owner")).unwrap();
        assert!(units.len() > 1);
        assert_coverage(&tokenizer, &text, &units);
        for unit in &units {
            assert_eq!(
                unit.input_sha256,
                hash_document_embedding_input(&unit.input)
            );
            assert_eq!(
                unit.reuse_key,
                crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15)
                    .document_reuse_key(&unit.input)
            );
        }
    }
}
#[test]
fn context_shortening_is_explicit_and_never_drops_body() {
    let tokenizer = tokenizer();
    let context = "one ".repeat(300);
    let body = "one ".repeat(800);
    let units = prepare_embedding_section(&tokenizer, plain(&body, &context)).unwrap();
    assert!(units.iter().all(|unit| unit.context_shortened));
    assert_coverage(&tokenizer, &body, &units);
    assert!(
        units
            .iter()
            .all(|unit| unit.token_count <= BODY_TOKEN_BUDGET + CONTEXT_TOKEN_BUDGET + 2)
    );
}
#[test]
fn identity_has_no_invented_byte_range_and_never_silently_clips() {
    let tokenizer = tokenizer();
    let units = prepare_embedding_section(
        &tokenizer,
        PassageSection::Identity {
            text: "Ghoul — creature",
        },
    )
    .unwrap();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].address, SourcePassageAddress::Identity {});
    let too_long = "one ".repeat(IDENTITY_TOKEN_BUDGET + 1);
    assert!(matches!(
        prepare_embedding_section(&tokenizer, PassageSection::Identity { text: &too_long }),
        Err(EmbeddingError::TokenBudgetExceeded {
            max: IDENTITY_TOKEN_BUDGET,
            ..
        })
    ));
}
#[test]
fn empty_content_is_explicitly_skipped_and_direct_inference_rejects_it() {
    let tokenizer = tokenizer();
    assert!(
        prepare_embedding_section(&tokenizer, plain("", ""))
            .unwrap()
            .is_empty()
    );
    assert!(
        prepare_embedding_section(&tokenizer, plain(" \n\t", ""))
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        tokenizer.validate_document_input(""),
        Err(EmbeddingError::EmptyInput)
    ));
}
#[test]
fn vector_validation_rejects_bad_shapes_nonfinite_and_unnormalized_data() {
    assert!(validate_embedding_vector(&[1.0, 0.0], 2).is_ok());
    for (name, vector) in [
        ("zero", &[0.0, 0.0][..]),
        ("nan", &[f32::NAN, 0.0][..]),
        ("infinity", &[f32::INFINITY, 0.0][..]),
        ("wrong dimensions", &[1.0][..]),
        ("nonunit", &[2.0, 0.0][..]),
    ] {
        assert!(validate_embedding_vector(vector, 2).is_err(), "{name}");
    }
}
#[test]
fn cache_requires_all_immutable_assets_and_only_verified_models() {
    let config = EmbeddingRuntimeConfig::new(crate::EmbeddingModelId::BgeBaseEnV15, "/not/a/cache");
    let files = required_embedding_model_cache_files(&config);
    assert_eq!(files.len(), 5);
    assert!(files.iter().all(|file| file.source_revision
        == crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15).model_revision
        && file.sha256.len() == 64));
    assert_eq!(ALL_EMBEDDING_MODELS.len(), 2);
    assert!("minilm-l12-v2".parse::<EmbeddingModelId>().is_err());
    assert_eq!(
        crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15).pooling,
        PoolingStrategy::Cls
    );
    assert_eq!(SourceByteRange { start: 0, end: 0 }.extract(""), Ok(""));
}

#[test]
fn modified_cached_asset_is_rejected_before_model_initialization() {
    let root =
        std::env::temp_dir().join(format!("atlas-embedding-checksum-{}", std::process::id()));
    let config = EmbeddingRuntimeConfig::new(crate::EmbeddingModelId::BgeBaseEnV15, &root);
    let model_dir = config.model_dir();
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("tokenizer.json"), b"modified model metadata").unwrap();
    assert!(matches!(
        crate::model_cache::read_verified_asset(config.model_spec(), &model_dir, "tokenizer.json"),
        Err(EmbeddingError::AssetChecksumMismatch { .. })
    ));
    assert!(matches!(
        validate_embedding_model_cache(&config),
        Err(EmbeddingError::AssetChecksumMismatch { .. })
    ));
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn pinned_profiles_preserve_research_identity_prefixes_and_projection() {
    let base = embedding_model_spec(EmbeddingModelId::BgeBaseEnV15);
    let gemma = embedding_model_spec(EmbeddingModelId::EmbeddingGemma300M);
    assert_eq!(base.dimensions, 768);
    assert_eq!(gemma.dimensions, 768);
    assert_eq!(
        base.model_revision,
        "a5beb1e3e68b9ab74eb54cfd186867f64f240e1a"
    );
    assert_eq!(
        gemma.model_revision,
        "5090578d9565bb06545b4552f76e6bc2c93e4a66"
    );
    assert_eq!(base.pooling, PoolingStrategy::Cls);
    assert_eq!(gemma.pooling, PoolingStrategy::ExportedSentenceEmbedding);
    assert_eq!(gemma.output_name, "sentence_embedding");
    let text = "  Résumé\n火 👩‍🚀\n";
    assert_eq!(base.document_input(text), text);
    assert_eq!(
        gemma.document_input(text),
        format!("title: none | text: {text}")
    );
    assert_eq!(
        gemma.query_input(text),
        format!("task: search result | query: {text}")
    );
    for model in ALL_EMBEDDING_MODELS {
        let spec = embedding_model_spec(*model);
        assert_eq!(model.as_str().parse::<EmbeddingModelId>().unwrap(), *model);
        assert_eq!(embedding_model_for_model_id(spec.model_id), Some(*model));
    }
}

#[test]
fn reuse_identity_binds_model_execution_and_all_assets_independently_of_raw_hash() {
    let text = "Ghoul Fever\nSaving Throw DC 15 Fortitude";
    let base = embedding_model_spec(EmbeddingModelId::BgeBaseEnV15);
    let gemma = embedding_model_spec(EmbeddingModelId::EmbeddingGemma300M);
    let expected = base.document_reuse_key(text);
    assert_ne!(expected, gemma.document_reuse_key(text));
    assert_ne!(expected, base.document_reuse_key(&format!("{text}\n")));
    for spec in [
        EmbeddingModelSpec {
            document_prefix: "prefix: ",
            ..base
        },
        EmbeddingModelSpec {
            query_prefix: "query: ",
            ..base
        },
        EmbeddingModelSpec {
            output_name: "other",
            ..base
        },
        EmbeddingModelSpec {
            max_input_tokens: Some(511),
            ..base
        },
        EmbeddingModelSpec {
            required_assets: &[("external", "changed")],
            ..base
        },
    ] {
        assert_ne!(expected, spec.document_reuse_key(text));
        assert!(matches!(
            crate::catalog::validate_model_spec(spec),
            Err(EmbeddingError::UnsupportedModelContract)
        ));
    }
    assert_eq!(
        hash_document_embedding_input(text),
        hash_document_embedding_input(&base.document_input(text))
    );
    assert_ne!(
        hash_document_embedding_input(text),
        hash_document_embedding_input(&gemma.document_input(text))
    );
}

#[test]
fn gemma_cache_requires_external_weights_and_verified_special_tokens() {
    let config = EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, "/not/a/cache");
    let files = required_embedding_model_cache_files(&config);
    assert_eq!(files.len(), 6);
    assert!(
        files
            .iter()
            .any(|file| file.source_path == "onnx/model.onnx_data"
                && file.sha256
                    == "ef835ae565d8695236652475903078e8ed794c7c35faf1164d78ec3238e8a88d")
    );
    assert!(
        files
            .iter()
            .any(|file| file.source_path == "special_tokens_map.json"
                && file.sha256
                    == "2f7b0adf4fb469770bb1490e3e35df87b1dc578246c5e7e6fc76ecf33213a397")
    );
    for file in &files {
        assert_eq!(file.source_repo, config.model_spec().model_id);
        assert_eq!(file.source_revision, config.model_spec().model_revision);
        assert_eq!(file.sha256.len(), 64);
    }
    let root = std::env::temp_dir().join(format!("atlas-gemma-assets-{}", std::process::id()));
    std::fs::create_dir_all(root.join("onnx")).unwrap();
    for source_path in ["onnx/model.onnx_data", "special_tokens_map.json"] {
        std::fs::write(root.join(source_path), b"changed").unwrap();
        assert!(matches!(
            crate::model_cache::read_verified_asset(config.model_spec(), &root, source_path),
            Err(EmbeddingError::AssetChecksumMismatch { .. })
        ));
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn final_document_and_query_budgets_count_official_prefixes_without_clipping() {
    let native = tokenizer().tokenizer().clone();
    for model in [
        EmbeddingModelId::BgeBaseEnV15,
        EmbeddingModelId::EmbeddingGemma300M,
    ] {
        let spec = embedding_model_spec(model);
        let tokenizer = TextEmbeddingTokenizer::from_tokenizer(spec, native.clone()).unwrap();
        let max = spec.max_input_tokens.unwrap();
        let prefix_tokens = tokenizer.count_tokens(spec.document_prefix, false).unwrap();
        let fits = "one ".repeat(max - prefix_tokens);
        assert_eq!(tokenizer.validate_document_input(&fits).unwrap(), max);
        assert!(
            matches!(tokenizer.validate_document_input(&(fits + "one")), Err(EmbeddingError::TokenBudgetExceeded { max: actual_max, .. }) if actual_max == max)
        );
        assert!(matches!(
            tokenizer.validate_query_input(&"one ".repeat(max)),
            Err(EmbeddingError::TokenBudgetExceeded { .. })
        ));
        assert!(matches!(
            tokenizer.validate_query_input(" \n"),
            Err(EmbeddingError::EmptyInput)
        ));
    }
}

#[test]
fn prepared_reuse_identity_uses_the_selected_model() {
    let native = tokenizer().tokenizer().clone();
    let spec = embedding_model_spec(EmbeddingModelId::EmbeddingGemma300M);
    let tokenizer = TextEmbeddingTokenizer::from_tokenizer(spec, native).unwrap();
    let units = prepare_embedding_section(&tokenizer, plain("one one", "one")).unwrap();
    assert_eq!(units[0].input, "one\none one");
    assert_eq!(units[0].reuse_key, spec.document_reuse_key(&units[0].input));
    assert_ne!(
        units[0].reuse_key,
        crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15)
            .document_reuse_key(&units[0].input)
    );
}

fn local_model_config() -> EmbeddingRuntimeConfig {
    if let Some(cache_root) = std::env::var_os("ATLAS_TEST_MODEL_CACHE") {
        let config = EmbeddingRuntimeConfig::new(EmbeddingModelId::BgeBaseEnV15, cache_root);
        assert!(
            config.model_dir().join("onnx/model.onnx").is_file(),
            "ATLAS_TEST_MODEL_CACHE must contain pinned BGE assets at {}",
            config.model_dir().display()
        );
        return config;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|parent| {
            EmbeddingRuntimeConfig::new(
                EmbeddingModelId::BgeBaseEnV15,
                parent.join(".cache/hf-models"),
            )
        })
        .find(|config| config.model_dir().join("onnx/model.onnx").is_file())
        .expect("these ignored execution tests require the verified local BGE asset cache")
}
fn local_model_dir() -> PathBuf {
    local_model_config().model_dir()
}

#[test]
#[ignore = "requires the pinned local BGE model and native runtime; run explicitly"]
fn actual_bge_tokenizer_covers_unicode_long_fields_and_tail() {
    let tokenizer = TextEmbeddingTokenizer::load_from_model_dir(
        crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15),
        local_model_dir(),
    )
    .unwrap();
    for body in [
        "Résumé ≥ ½; 2d6[fire] + @item.level.\n".repeat(100),
        "火".repeat(2000),
        "q".repeat(20000),
        "👩‍🚀 e\u{301} ".repeat(1000),
        "First paragraph.\n\nSecond paragraph.\n".repeat(200),
    ] {
        let units = prepare_embedding_section(
            &tokenizer,
            plain(&body, &"Ghoul — Ghoul Fever — ".repeat(100)),
        )
        .unwrap();
        assert_coverage(&tokenizer, &body, &units);
    }
}

#[test]
#[ignore = "requires the pinned local BGE model and native runtime; run explicitly"]
fn actual_bge_batches_reuse_preserves_attribution_and_utf8() {
    let config = local_model_config();
    let mut model = TextEmbedder::load_from_model_dir(
        crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15),
        local_model_dir(),
    )
    .unwrap();
    let empty_batch = PreparedEmbeddingBatch::prepare(model.tokenizer(), vec![]).unwrap();
    let mut runner = NativeInferenceRunner::new(&config, 8).unwrap();
    assert_eq!(
        runner
            .infer(&empty_batch, |_| panic!("empty inference checkpoint"))
            .unwrap(),
        0
    );
    assert!(!runner.is_loaded());
    let texts = [
        "Ghoul Fever — Saving Throw DC 15 Fortitude; disease",
        "Résumé ≥ ½; 2d6[fire] + @item.level.",
        "火星",
        "∞ ≠ ≤ ≥",
    ];
    let batch = model.embed_documents(&texts).unwrap();
    assert_eq!(batch.len(), texts.len());
    for (text, expected) in texts.iter().zip(&batch) {
        let single = model.embed_document(text).unwrap();
        assert_eq!(expected.len(), 768, "{text}");
        assert_eq!(single.len(), 768, "{text}");
        assert!(
            single
                .iter()
                .zip(expected)
                .all(|(left, right)| (left - right).abs() < 2e-5)
        );
    }
    assert!(matches!(
        model.embed_document(&"the creature ".repeat(1000)),
        Err(EmbeddingError::TokenBudgetExceeded { .. })
    ));
    assert!(matches!(
        model.embed_query(""),
        Err(EmbeddingError::EmptyInput)
    ));
    let query = model.embed_query("disease from a ghoul").unwrap();
    validate_embedding_vector(&query, 768).unwrap();
    let mut units = prepare_embedding_section(model.tokenizer(), plain(texts[0], "Ghoul")).unwrap();
    let mut duplicate = units[0].clone();
    if let SourcePassageAddress::PlainSection {
        section_ordinal,
        label,
        ..
    } = &mut duplicate.address
    {
        *section_ordinal = 19;
        *label = Some("A separately attributed occurrence".to_owned());
    }
    assert_ne!(duplicate.address, units[0].address);
    units.push(duplicate);
    // Attribution remains with the metadata occurrences. The numerical runner
    // receives the one distinct missing input selected by the ingest owner.
    let batch =
        PreparedEmbeddingBatch::prepare(model.tokenizer(), vec![units[0].clone().into()]).unwrap();
    let mut checkpoints = 0;
    for _ in 0..2 {
        assert_eq!(
            runner
                .infer(&batch, |values| {
                    assert_eq!(values.len(), 1);
                    assert_eq!(values[0].0.descriptor(), batch.inputs()[0].descriptor());
                    validate_embedding_vector(values[0].1, 768)?;
                    checkpoints += 1;
                    Ok(())
                })
                .unwrap(),
            1
        );
    }
    assert!(runner.is_loaded());
    assert_eq!(runner.inferred_inputs(), 2);
    assert_eq!(checkpoints, 2);
    units[0].input.push_str("changed");
    assert!(matches!(
        PreparedEmbeddingBatch::prepare(
            model.tokenizer(),
            units.into_iter().map(Into::into).collect()
        ),
        Err(EmbeddingError::PreparedInputMismatch)
    ));
}

#[test]
fn lazy_runner_needs_no_assets_until_actual_missing_inference() {
    let tokenizer = tokenizer();
    let input = prepare_embedding_section(&tokenizer, plain("one one", "one"))
        .unwrap()
        .remove(0);
    let empty = PreparedEmbeddingBatch::prepare(&tokenizer, vec![]).unwrap();
    let batch = PreparedEmbeddingBatch::prepare(&tokenizer, vec![input.clone().into()]).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let config = EmbeddingRuntimeConfig::new(EmbeddingModelId::BgeBaseEnV15, directory.path());
    let mut runner = NativeInferenceRunner::new(&config, 16).unwrap();
    assert_eq!(
        runner
            .infer(&empty, |_| panic!("empty checkpoint"))
            .unwrap(),
        0
    );
    assert!(!runner.is_loaded());
    assert_eq!(runner.inferred_inputs(), 0);
    let wrong = EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, directory.path());
    let mut wrong_runner = NativeInferenceRunner::new(&wrong, 16).unwrap();
    assert!(matches!(
        wrong_runner.infer(&batch, |_| Ok(())),
        Err(EmbeddingError::PreparedInputMismatch)
    ));
    assert!(!wrong_runner.is_loaded());
    assert!(
        runner.infer(&batch, |_| Ok(())).is_err(),
        "missing input must load verified assets"
    );
    assert_eq!(runner.inferred_inputs(), 0);
    assert!(!runner.is_loaded());
    assert!(NativeInferenceRunner::new(&config, 0).is_err());
    assert!(matches!(
        PreparedEmbeddingBatch::prepare(
            &tokenizer,
            vec![input.into(); NATIVE_INFERENCE_BATCH_LIMIT + 1]
        ),
        Err(EmbeddingError::PreparedInputMismatch)
    ));
}
