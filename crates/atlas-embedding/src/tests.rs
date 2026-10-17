use crate::*;
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use std::{collections::BTreeMap, path::PathBuf};
use tokenizers::{Tokenizer, models::wordlevel::WordLevel, pre_tokenizers::whitespace::Whitespace};

fn tokenizer() -> TextEmbeddingTokenizer {
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
fn plain<'a>(text: &'a str, context: &'a str) -> PassageSection<'a> {
    PassageSection::Plain {
        text,
        context,
        selection_version: "test/v1",
        section_ordinal: 4,
        label: Some("Fever"),
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
            ..
        } = &unit.address
        else {
            panic!("expected plain section")
        };
        assert_eq!(source_text_sha256, &hash_document_embedding_input(text));
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
    assert!(validate_embedding_vector(&[0.0, 0.0], 2).is_err());
    assert!(validate_embedding_vector(&[f32::NAN, 0.0], 2).is_err());
    assert!(validate_embedding_vector(&[1.0], 2).is_err());
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

fn local_model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|parent| {
            crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15)
                .model_cache_path(parent.join(".cache/hf-models"))
        })
        .find(|path| path.join("onnx/model.onnx").is_file())
        .expect("these ignored execution tests require the verified local BGE asset cache")
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
    let mut model = TextEmbedder::load_from_model_dir(
        crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15),
        local_model_dir(),
    )
    .unwrap();
    let empty = generate_prepared_embeddings(&mut model, &[], &BTreeMap::new(), 8).unwrap();
    assert!(empty.vectors.is_empty());
    assert_eq!((empty.inferred_inputs, empty.reused_inputs), (0, 0));
    let texts = [
        "Ghoul Fever — Saving Throw DC 15 Fortitude; disease",
        "Résumé ≥ ½; 2d6[fire] + @item.level.",
        "火星",
        "∞ ≠ ≤ ≥",
    ];
    let batch = model.embed_documents(&texts).unwrap();
    for (text, expected) in texts.iter().zip(&batch) {
        let single = model.embed_document(text).unwrap();
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
    let initial = generate_prepared_embeddings(&mut model, &units, &BTreeMap::new(), 16).unwrap();
    assert_eq!(initial.inferred_inputs, 1);
    assert_eq!(initial.vectors.len(), 2);
    assert_eq!(initial.vectors[0], initial.vectors[1]);
    let reusable = BTreeMap::from([(units[0].reuse_key.clone(), initial.vectors[0].clone())]);
    let reused = generate_prepared_embeddings(&mut model, &units, &reusable, 16).unwrap();
    assert_eq!(reused.inferred_inputs, 0);
    assert_eq!(reused.reused_inputs, 1);
    assert_eq!(reused.vectors, initial.vectors);
    units[0].input.push_str("changed");
    assert!(matches!(
        generate_prepared_embeddings(&mut model, &units, &reusable, 16),
        Err(EmbeddingError::PreparedInputMismatch)
    ));
}
