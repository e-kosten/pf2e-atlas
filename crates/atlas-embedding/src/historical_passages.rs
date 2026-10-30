use crate::{PassageSection, hash_document_embedding_input, prepare_embedding_section};
use serde_json::{Value, json};

const HISTORICAL_RECIPES: &[(&str, &str)] = &[(
    include_str!("fixtures/passage-recipes/v2.json"),
    "e590aed213153c73c7e2c98bdbe19c45ad011c5db469f59a534a80e4b10f6f18",
)];

fn fixture_digest(source: &str) -> String {
    hash_document_embedding_input(&source.replace("\r\n", "\n"))
}

fn current_output(case: &Value) -> Value {
    let tokenizer = crate::TextEmbeddingTokenizer::from_tokenizer(
        crate::embedding_model_spec(crate::EmbeddingModelId::BgeBaseEnV15),
        tokenizers::Tokenizer::from_bytes(include_bytes!(
            "fixtures/passage-recipes/wordlevel-v1.json"
        ))
        .unwrap(),
    )
    .unwrap();
    let text = case["text"].as_str().unwrap();
    let context = case["context"].as_str().unwrap();
    let selection_version = case["selection_version"].as_str().unwrap();
    let section_ordinal = case["section_ordinal"].as_u64().unwrap() as usize;
    let label = case["label"].as_str();
    let section = match case["kind"].as_str().unwrap() {
        "plain" => PassageSection::Plain {
            text,
            context,
            selection_version,
            section_ordinal,
            label,
        },
        "html" => PassageSection::Html {
            text,
            context,
            selection_version,
            section_ordinal,
            label,
            prepared_html_sha256: case["prepared_html_sha256"].as_str().unwrap(),
        },
        other => panic!("unknown historical fixture kind {other}"),
    };
    json!(
        prepare_embedding_section(&tokenizer, section)
            .unwrap()
            .into_iter()
            .map(|input| json!({"address":input.address,"input_sha256":input.input_sha256}))
            .collect::<Vec<_>>()
    )
}

#[test]
fn historical_recipe_keeps_its_published_layout_and_input_identity() {
    let current = json!({
        "unit_policy_version": crate::EMBEDDING_UNIT_POLICY_VERSION,
        "segmentation_tokenizer_sha256": crate::BASELINE_SEGMENTATION_TOKENIZER_SHA256,
        "input_recipe": crate::BASELINE_INPUT_RECIPE,
    });
    let mut matched = false;
    for &(source, digest) in HISTORICAL_RECIPES {
        assert_eq!(
            fixture_digest(source),
            digest,
            "Historical fixtures are immutable. Bump the recipe identity and append a new fixture for changed behavior."
        );
        let fixture: Value = serde_json::from_str(source).unwrap();
        assert_eq!(fixture["tokenizer_fixture"], "wordlevel-whitespace/v1");
        assert_eq!(
            fixture_digest(include_str!("fixtures/passage-recipes/wordlevel-v1.json")),
            fixture["tokenizer_fixture_sha256"]
        );
        if fixture["recipe"] != current {
            continue;
        }
        matched = true;
        for case in fixture["cases"].as_array().unwrap() {
            assert_eq!(
                current_output(case),
                case["expected"],
                "{} changed under the same recipe identity; bump the recipe and add a fixture",
                case["name"]
            );
        }
    }
    assert!(
        matched,
        "The current passage recipe needs a pinned output fixture; retain previous fixtures."
    );
}
