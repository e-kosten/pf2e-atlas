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
    TextEmbeddingTokenizer::from_tokenizer(default_embedding_model_spec(), tokenizer).unwrap()
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
            assert_eq!(unit.reuse_key, embedding_reuse_key(&unit.input));
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
fn cache_requires_all_four_immutable_assets_and_only_verified_model() {
    let config = EmbeddingRuntimeConfig::default_model("/not/a/cache");
    let files = required_embedding_model_cache_files(&config);
    assert_eq!(files.len(), 4);
    assert!(files.iter().all(|file| file.source_revision
        == default_embedding_model_spec().model_revision
        && file.sha256.len() == 64));
    assert_eq!(ALL_EMBEDDING_MODELS.len(), 1);
    assert!("minilm-l12-v2".parse::<EmbeddingModelId>().is_err());
    assert_eq!(default_embedding_model_spec().pooling, PoolingStrategy::Cls);
    assert_eq!(SourceByteRange { start: 0, end: 0 }.extract(""), Ok(""));
}

#[test]
fn modified_cached_asset_is_rejected_before_model_initialization() {
    let root =
        std::env::temp_dir().join(format!("atlas-embedding-checksum-{}", std::process::id()));
    let config = EmbeddingRuntimeConfig::default_model(&root);
    let model_dir = config.model_dir();
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("tokenizer.json"), b"modified model metadata").unwrap();
    assert!(matches!(
        crate::model_cache::read_verified_asset(&model_dir, "tokenizer.json"),
        Err(EmbeddingError::AssetChecksumMismatch { .. })
    ));
    assert!(matches!(
        validate_embedding_model_cache(&config),
        Err(EmbeddingError::AssetChecksumMismatch { .. })
    ));
    std::fs::remove_dir_all(&root).unwrap();
}

fn local_model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|parent| parent.join(".cache/hf-models/BAAI/bge-small-en-v1.5"))
        .find(|path| path.join("onnx/model.onnx").is_file())
        .expect("these ignored execution tests require the verified local BGE asset cache")
}

#[test]
#[ignore = "requires the pinned local BGE model and native runtime; run explicitly"]
fn actual_bge_tokenizer_covers_unicode_long_fields_and_tail() {
    let tokenizer = TextEmbeddingTokenizer::load_from_model_dir(
        default_embedding_model_spec(),
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
    let mut model =
        TextEmbedder::load_from_model_dir(default_embedding_model_spec(), local_model_dir())
            .unwrap();
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
    validate_embedding_vector(&query, 384).unwrap();
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

#[test]
#[ignore = "requires the independent saved same-assets model probe and local BGE; run explicitly"]
fn actual_production_inference_matches_saved_compatible_probes() {
    use std::io::{BufRead, BufReader};
    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|parent| {
            parent.join("scratch/text-splitter-evaluation/inference/compatible/probe-vectors.jsonl")
        })
        .find(|path| path.is_file())
        .expect("independent compatible-library probe corpus is required");
    let mut model =
        TextEmbedder::load_from_model_dir(default_embedding_model_spec(), local_model_dir())
            .unwrap();
    let mut verified = 0;
    let mut deliberately_overlong = 0;
    for row in BufReader::new(std::fs::File::open(reference).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&row.unwrap()).unwrap();
        let input = row["input"].as_str().unwrap();
        let query = row["role"] == "query";
        let raw = if query {
            input
                .strip_prefix(default_embedding_model_spec().query_prefix)
                .unwrap()
        } else {
            input
        };
        let result = if query {
            model.embed_query(raw)
        } else {
            model.embed_document(raw)
        };
        if row["truncated"] == true {
            assert!(matches!(
                result,
                Err(EmbeddingError::TokenBudgetExceeded { .. })
            ));
            deliberately_overlong += 1;
            continue;
        }
        let actual = result.unwrap();
        let expected: Vec<f32> = serde_json::from_value(row["vector"].clone()).unwrap();
        assert_eq!(actual.len(), expected.len());
        assert!(
            actual
                .iter()
                .zip(expected)
                .all(|(actual, expected)| (actual - expected).abs() < 2e-5),
            "saved probe {}",
            row["id"]
        );
        verified += 1;
    }
    assert_eq!(verified, 6);
    assert_eq!(deliberately_overlong, 1);
}

#[test]
#[ignore = "requires the pinned selected corpus and BGE tokenizer; run explicitly"]
fn actual_selected_corpus_has_full_byte_coverage_and_no_clipped_inputs() {
    use std::{
        collections::BTreeSet,
        fs,
        io::{BufRead, BufReader},
    };
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|parent| parent.join("scratch/search-projection/selection/semantic-units.jsonl"))
        .find(|path| path.is_file())
        .expect("the locally selected source corpus is required");
    let tokenizer = TextEmbeddingTokenizer::load_from_model_dir(
        default_embedding_model_spec(),
        local_model_dir(),
    )
    .unwrap();
    let mut roots = BTreeSet::new();
    let mut unique = BTreeSet::new();
    let mut sections = 0;
    let mut prepared = 0;
    let mut source_bytes = 0;
    let mut input_tokens = 0;
    let mut max_tokens = 0;
    let mut shortened = 0;
    let started = std::time::Instant::now();
    for line in BufReader::new(fs::File::open(&corpus).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let body = row["body"].as_str().unwrap();
        let identity = row["unit_kind"] == "identity";
        let section = if identity {
            PassageSection::Identity { text: body }
        } else {
            plain(body, row["context"].as_str().unwrap())
        };
        let units = prepare_embedding_section(&tokenizer, section).unwrap();
        assert!(
            !units.is_empty(),
            "selected corpus has no zero-token bodies"
        );
        if !identity {
            assert_coverage(&tokenizer, body, &units);
        }
        source_bytes += body.len();
        roots.insert(row["root_key"].as_str().unwrap().to_owned());
        shortened += usize::from(units[0].context_shortened);
        for unit in units {
            prepared += 1;
            input_tokens += unit.token_count;
            max_tokens = max_tokens.max(unit.token_count);
            unique.insert(unit.reuse_key);
        }
        sections += 1;
        if sections % 10000 == 0 {
            eprintln!("prepared {sections} selected sections into {prepared} inputs");
        }
    }
    assert_eq!(sections, 123803);
    assert_eq!(roots.len(), 25560);
    assert_eq!(source_bytes, 45692869);
    assert_eq!(prepared, 130421);
    assert_eq!(input_tokens, 12170310);
    assert!(max_tokens <= 512);
    let census = serde_json::json!({
        "policy": EMBEDDING_UNIT_POLICY_VERSION, "corpus": corpus,
        "selected_sections": sections, "prepared_inputs": prepared,
        "product_roots": roots.len(), "source_body_bytes": source_bytes,
        "input_tokens": input_tokens, "max_input_tokens": max_tokens,
        "context_shortened_sections": shortened, "unique_inference_inputs": unique.len(),
        "coverage": "exact UTF8 source slices, advancing overlapping ranges, all tails, independently retokenized overlap and actual final input budgets",
        "elapsed_seconds": started.elapsed().as_secs_f64(),
        "scope": "production embedding preparation over the immutable prior selected corpus, not production record selection or full corpus inference"
    });
    let output =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scratch/embedding-validation");
    fs::create_dir_all(&output).unwrap();
    fs::write(
        output.join("production-preparation-census.json"),
        serde_json::to_vec_pretty(&census).unwrap(),
    )
    .unwrap();
}
