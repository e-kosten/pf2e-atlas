use std::path::PathBuf;

use atlas_domain::RecordFamily;
use atlas_record::{
    PresentationBadge, PresentationBadgeKind, PresentationBlock, PresentationFact,
    PresentationRelationship, PresentationRelationshipKind, PresentationSection,
    PresentationSectionKind, PresentationText, RecordPresentationDocument,
};

use crate::document_renderer::{EmbeddingInputChunk, EmbeddingInputSection};

use super::*;

const MODEL_CACHE_ENV: &str = "ATLAS_EMBEDDING_TEST_CACHE";
const VECTOR_TOLERANCE: f32 = 0.00001;

#[test]
fn default_model_spec_matches_bge_small_contract() {
    let spec = default_embedding_model_spec();

    assert_eq!(DEFAULT_EMBEDDING_MODEL, EmbeddingModelId::BgeSmallEnV15);
    assert_eq!(spec.provider_family, "onnx-mean-pooling");
    assert_eq!(spec.model_id, "BAAI/bge-small-en-v1.5");
    assert_eq!(spec.model_revision, "main");
    assert_eq!(spec.tokenizer_id, "BAAI/bge-small-en-v1.5");
    assert_eq!(spec.max_input_tokens, Some(512));
    assert_eq!(spec.pooling.as_str(), "mean");
    assert_eq!(spec.normalization.as_str(), "l2");
    assert_eq!(spec.dimensions, 384);
    assert_eq!(spec.dtype.as_str(), "f32");
    assert_eq!(spec.distance_metric.as_str(), "cosine");
    assert_eq!(spec.document_prefix, "");
    assert_eq!(
        spec.query_prefix,
        "Represent this sentence for searching relevant passages: "
    );
}

#[test]
fn model_aliases_keep_minilm_explicit_and_default_tracks_catalog_default() {
    assert_eq!(
        "default".parse::<EmbeddingModelId>().unwrap(),
        DEFAULT_EMBEDDING_MODEL
    );
    assert_eq!(
        "minilm".parse::<EmbeddingModelId>().unwrap(),
        EmbeddingModelId::MiniLmL12V2
    );
}

#[test]
fn candidate_model_specs_declare_input_token_limits() {
    for model in ALL_EMBEDDING_MODELS {
        let spec = embedding_model_spec(*model);
        assert!(
            spec.max_input_tokens.is_some(),
            "{} should declare an ONNX-safe input token limit",
            model
        );
    }

    assert_eq!(
        embedding_model_spec(EmbeddingModelId::BgeSmallEnV15).max_input_tokens,
        Some(512)
    );
    assert_eq!(
        embedding_model_spec(EmbeddingModelId::NomicEmbedTextV15).max_input_tokens,
        Some(8192)
    );
}

#[test]
fn normalizes_queries_like_typescript_provider() {
    assert_eq!(
        normalize_embedding_text("Remove&nbsp;Frightened Condition!"),
        "remove frightened condition"
    );
}

#[test]
fn hashes_document_embedding_input_stably() {
    let input = "Name: Heal\nTraits: healing, vitality\nDescription: Restore Hit Points.";

    assert_eq!(
        hash_document_embedding_input(input),
        "4782317058a66506f1d72113b3ec9a87167ee2dff98b3ffca3039919d9024fb8"
    );
}

#[test]
fn renders_presentation_document_for_embedding_in_priority_order() {
    let document = fixture_presentation_document();

    let input = render_presentation_document_for_embedding(&document);

    assert_eq!(
        input,
        "Name: Shield Warden\n\
Family: Creature\n\
Level: 4\n\
Traits: Guardian, Shield\n\
Classification: Defender\n\
Role: Defensive guardian\n\
Description: A disciplined guardian protects nearby allies.\n\
AC: 22\n\
HP: 70\n\
Speed: Land 25 feet\n\
Attack: Shield bash +14\n\
References: Reactive Strike"
    );
}

#[test]
fn embedding_renderer_omits_backlinks() {
    let mut document = fixture_presentation_document();
    document.sections.push(PresentationSection::new(
        PresentationSectionKind::Backlinks,
        vec![PresentationBlock::Relationships(vec![
            PresentationRelationship {
                kind: PresentationRelationshipKind::Backlink,
                label: "Some scenario record".to_string(),
                record_key: None,
            },
        ])],
    ));

    let input = render_presentation_document_for_embedding(&document);

    assert!(!input.contains("Some scenario record"));
}

#[test]
fn embedding_renderer_omits_display_only_prerequisites() {
    let mut document = fixture_presentation_document();
    document.sections[0] = PresentationSection::new(
        PresentationSectionKind::Summary,
        vec![PresentationBlock::FactList(vec![
            PresentationFact {
                key: "role".to_string(),
                label: "Role".to_string(),
                value: "Defensive guardian".to_string(),
            },
            PresentationFact {
                key: "prerequisites".to_string(),
                label: "Prerequisites".to_string(),
                value: "trained in Medicine, Battle Medicine".to_string(),
            },
        ])],
    );

    let input = render_presentation_document_for_embedding(&document);

    assert!(input.contains("Role: Defensive guardian"));
    assert!(!input.contains("Prerequisites"));
    assert!(!input.contains("trained in Medicine"));
    assert!(!input.contains("Battle Medicine"));
}

#[test]
fn minilm_tokenization_reports_catalog_limit_when_model_cache_exists() {
    let Some(config) =
        model_backed_test_config(EmbeddingModelId::MiniLmL12V2, "minilm tokenization limit")
    else {
        return;
    };

    let tokenizer = TextEmbeddingTokenizer::load(&config)
        .expect("local MiniLM cache should load from test cache");
    let long_input = std::iter::repeat_n("poison sickened slowed persistent damage", 200)
        .collect::<Vec<_>>()
        .join(" ");
    let telemetry = tokenizer
        .analyze_document_inputs(&[long_input.as_str()])
        .expect("document tokenization should succeed");

    assert_eq!(telemetry.len(), 1);
    assert_eq!(telemetry[0].max_token_count, Some(512));
    assert!(telemetry[0].token_count > 512);
    assert!(telemetry[0].truncated);
}

#[test]
fn minilm_budgeting_drops_lower_priority_sections_when_model_cache_exists() {
    let Some(config) =
        model_backed_test_config(EmbeddingModelId::MiniLmL12V2, "minilm section-aware budget")
    else {
        return;
    };

    let tokenizer = TextEmbeddingTokenizer::load(&config)
        .expect("local MiniLM cache should load from test cache");
    let chunks = vec![
        EmbeddingInputChunk::line(EmbeddingInputSection::Identity, "Name: Venom Torrent"),
        EmbeddingInputChunk::line(EmbeddingInputSection::Traits, "Traits: Poison, Consumable"),
        EmbeddingInputChunk::truncatable_line(
            EmbeddingInputSection::Description,
            format!(
                "Description: {}",
                std::iter::repeat_n("poison sickened slowed persistent damage", 180)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ),
        EmbeddingInputChunk::truncatable_line(
            EmbeddingInputSection::References,
            "References: poison, sickened, persistent damage, basic Fortitude save",
        ),
    ];

    let budgeted = tokenizer
        .budget_document_input(&chunks)
        .expect("budgeting should succeed");
    let budgeted_tokenization = tokenizer
        .analyze_document_inputs(&[budgeted.text.as_str()])
        .expect("budgeted document should tokenize");

    assert!(budgeted.tokenization.truncated);
    assert_eq!(budgeted.tokenization.max_token_count, Some(512));
    assert!(budgeted_tokenization[0].token_count <= 512);
    assert!(budgeted.text.contains("Name: Venom Torrent"));
    assert!(budgeted.text.contains("Traits: Poison, Consumable"));
    assert!(
        budgeted
            .truncated_sections
            .iter()
            .any(|section| section.section == EmbeddingInputSection::Description)
    );
    assert!(budgeted.text.contains("References: poison"));
    assert!(
        !budgeted
            .truncated_sections
            .iter()
            .any(|section| section.section == EmbeddingInputSection::References)
    );
}

#[test]
fn minilm_query_vectors_match_typescript_fixture_when_model_cache_exists() {
    let Some(config) =
        model_backed_test_config(EmbeddingModelId::MiniLmL12V2, "minilm query vector parity")
    else {
        return;
    };

    let mut embedder =
        TextEmbedder::load(&config).expect("local MiniLM cache should load from test cache");
    for fixture in ts_vector_fixtures() {
        let vector = embedder
            .embed_query(fixture.query)
            .expect("query embedding should succeed");

        assert_eq!(vector.len(), 384, "query `{}`", fixture.query);
        for (index, expected) in fixture.first8.iter().enumerate() {
            let actual = vector[index];
            assert!(
                (actual - expected).abs() <= VECTOR_TOLERANCE,
                "query `{}` vector[{index}] expected {expected}, got {actual}",
                fixture.query
            );
        }
    }
}

#[test]
fn minilm_document_embedding_uses_document_prefix_when_model_cache_exists() {
    let Some(config) = model_backed_test_config(
        EmbeddingModelId::MiniLmL12V2,
        "minilm document prefix parity",
    ) else {
        return;
    };

    let mut embedder =
        TextEmbedder::load(&config).expect("local MiniLM cache should load from test cache");
    let document_vector = embedder
        .embed_document("Heal\nhealing\nRestore Hit Points.")
        .expect("document embedding should succeed");
    let query_vector = embedder
        .embed_query("Heal\nhealing\nRestore Hit Points.")
        .expect("query embedding should succeed");

    assert_eq!(document_vector.len(), 384);
    assert_eq!(query_vector.len(), 384);
    assert_eq!(document_vector, query_vector);
}

#[test]
fn minilm_batch_document_embeddings_match_single_embeddings_when_model_cache_exists() {
    let Some(config) = model_backed_test_config(
        EmbeddingModelId::MiniLmL12V2,
        "minilm batch document parity",
    ) else {
        return;
    };

    let inputs = [
        "Heal\nhealing\nRestore Hit Points.",
        "Raise a Shield\ndefense\nUse your shield to protect yourself.",
        "",
    ];
    let mut single_embedder =
        TextEmbedder::load(&config).expect("local MiniLM cache should load from test cache");
    let single_vectors = inputs
        .iter()
        .map(|input| {
            single_embedder
                .embed_document(input)
                .expect("single document embedding should succeed")
        })
        .collect::<Vec<_>>();

    let mut batch_embedder =
        TextEmbedder::load(&config).expect("local MiniLM cache should load from test cache");
    let batch_vectors = batch_embedder
        .embed_documents(&inputs)
        .expect("batch document embedding should succeed");

    assert_eq!(batch_vectors.len(), single_vectors.len());
    for (single, batch) in single_vectors.iter().zip(batch_vectors.iter()) {
        assert_eq!(batch.len(), single.len());
        for (single_value, batch_value) in single.iter().zip(batch.iter()) {
            assert!((batch_value - single_value).abs() <= VECTOR_TOLERANCE);
        }
    }
}

fn model_backed_test_config(
    model: EmbeddingModelId,
    test_name: &str,
) -> Option<EmbeddingRuntimeConfig> {
    let cache_root = model_cache_root();
    let config = EmbeddingRuntimeConfig::new(model, &cache_root);
    let model_path = config.model_dir().join("onnx").join("model.onnx");
    if model_path.exists() {
        Some(config)
    } else {
        eprintln!(
            "skipping {test_name}: {model} model cache not found at {}; set {MODEL_CACHE_ENV} to override",
            cache_root.display()
        );
        None
    }
}

fn model_cache_root() -> PathBuf {
    if let Some(path) = std::env::var_os(MODEL_CACHE_ENV) {
        return PathBuf::from(path);
    }
    repo_root().join(".cache").join("hf-models")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn fixture_presentation_document() -> RecordPresentationDocument {
    RecordPresentationDocument {
        record_key: "creatures:ShieldWarden"
            .parse()
            .expect("record key should parse"),
        record_family: RecordFamily::Creature,
        title: "Shield Warden".to_string(),
        identity: vec![
            PresentationFact {
                key: "family".to_string(),
                label: "Family".to_string(),
                value: "Creature".to_string(),
            },
            PresentationFact {
                key: "level".to_string(),
                label: "Level".to_string(),
                value: "4".to_string(),
            },
        ],
        badges: vec![
            PresentationBadge {
                kind: PresentationBadgeKind::Trait,
                label: "Trait".to_string(),
                value: "Guardian".to_string(),
            },
            PresentationBadge {
                kind: PresentationBadgeKind::Trait,
                label: "Trait".to_string(),
                value: "Shield".to_string(),
            },
            PresentationBadge {
                kind: PresentationBadgeKind::Classification,
                label: "Family".to_string(),
                value: "Defender".to_string(),
            },
        ],
        sections: vec![
            PresentationSection::new(
                PresentationSectionKind::Summary,
                vec![PresentationBlock::FactList(vec![PresentationFact {
                    key: "role".to_string(),
                    label: "Role".to_string(),
                    value: "Defensive guardian".to_string(),
                }])],
            ),
            PresentationSection::new(
                PresentationSectionKind::Defense,
                vec![PresentationBlock::FactList(vec![
                    PresentationFact {
                        key: "ac".to_string(),
                        label: "AC".to_string(),
                        value: "22".to_string(),
                    },
                    PresentationFact {
                        key: "hp".to_string(),
                        label: "HP".to_string(),
                        value: "70".to_string(),
                    },
                ])],
            ),
            PresentationSection::new(
                PresentationSectionKind::Movement,
                vec![PresentationBlock::FactList(vec![PresentationFact {
                    key: "speed".to_string(),
                    label: "Speed".to_string(),
                    value: "Land 25 feet".to_string(),
                }])],
            ),
            PresentationSection::new(
                PresentationSectionKind::Offense,
                vec![PresentationBlock::FactList(vec![PresentationFact {
                    key: "attack".to_string(),
                    label: "Attack".to_string(),
                    value: "Shield bash +14".to_string(),
                }])],
            ),
            PresentationSection::new(
                PresentationSectionKind::Description,
                vec![PresentationBlock::Prose(PresentationText {
                    text: "A disciplined guardian protects nearby allies.".to_string(),
                })],
            ),
            PresentationSection::new(
                PresentationSectionKind::References,
                vec![PresentationBlock::Relationships(vec![
                    PresentationRelationship {
                        kind: PresentationRelationshipKind::Reference,
                        label: "Reactive Strike".to_string(),
                        record_key: Some(
                            "actions:ReactiveStrike"
                                .parse()
                                .expect("record key should parse"),
                        ),
                    },
                ])],
            ),
        ],
    }
}

struct VectorFixture {
    query: &'static str,
    first8: [f32; 8],
}

fn ts_vector_fixtures() -> [VectorFixture; 5] {
    [
        VectorFixture {
            query: "low level healing spell",
            first8: [
                -0.068_654_3,
                -0.002_315_331_7,
                0.043_613_01,
                0.033_749_383,
                -0.063_630_88,
                -0.027_416_993,
                -0.015_849_806,
                -0.007_909_846,
            ],
        },
        VectorFixture {
            query: "reaction to raise a shield",
            first8: [
                -0.026_845_824,
                0.122_861_5,
                -0.011_812_944,
                0.035_734_233,
                -0.010_173_997,
                -0.065_088_47,
                0.048_870_15,
                0.004_172_891_4,
            ],
        },
        VectorFixture {
            query: "monster with grab and swim speed",
            first8: [
                -0.011_481_782,
                -0.000_345_111_77,
                -0.025_876_341,
                0.017_008_279,
                -0.015_930_57,
                -0.036_722_105,
                0.080_977_455,
                0.007_360_012_3,
            ],
        },
        VectorFixture {
            query: "fireball",
            first8: [
                -0.001_271_67,
                0.019_497_56,
                -0.011_835_683,
                -0.017_335_506,
                0.039_618_62,
                0.031_982_79,
                0.180_924_61,
                0.056_330_826,
            ],
        },
        VectorFixture {
            query: "remove frightened condition",
            first8: [
                0.093_608_31,
                -0.002_027_062,
                -0.004_421_522_4,
                0.088_750_735,
                0.101_151_95,
                -0.123_675_734,
                0.119_872_94,
                -0.047_043_37,
            ],
        },
    ]
}
