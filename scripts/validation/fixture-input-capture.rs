// This test is appended to atlas-ingest's product preparation tests in an
// isolated checkout of the pinned producer. It does not ship with Atlas.
#[test]
fn capture_compatibility_inputs() {
    use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig};

    let source = std::path::PathBuf::from(std::env::var_os("ATLAS_FIXTURE_SOURCE").unwrap());
    let cache = std::path::PathBuf::from(std::env::var_os("ATLAS_FIXTURE_MODELS").unwrap());
    let output = std::path::PathBuf::from(std::env::var_os("ATLAS_FIXTURE_INPUTS").unwrap());
    let scratch = tempfile::tempdir().unwrap();
    let profiles = [
        ("bge-base", EmbeddingModelId::BgeBaseEnV15),
        ("embeddinggemma", EmbeddingModelId::EmbeddingGemma300M),
    ];
    let configs = profiles.map(|(_, model)| EmbeddingRuntimeConfig::new(model, &cache));
    let options = BuildArtifactOptions {
        source_root: source,
        output_path: scratch.path().join("fixture.sqlite"),
        manifest_path: None,
        locale: "en".into(),
        search_profiles: configs.to_vec(),
        reuse_passage_layouts: false,
        embedding_batch_size: 16,
    };
    let prepared = prepare_fixture(&options).unwrap();
    let mut cursor = prepared.native_inputs.cursor().unwrap();
    let mut inputs = std::collections::BTreeMap::new();
    while let Some(frame) = cursor.next_frame().unwrap() {
        let kind = match frame.attribution.address {
            atlas_domain::SourcePassageAddress::Identity {} => "identity",
            _ => "passage",
        };
        inputs
            .entry(kind)
            .or_insert_with(std::collections::BTreeMap::new)
            .entry(frame.input.input_sha256)
            .or_insert(frame.input.input);
    }
    let mut samples = std::collections::BTreeMap::new();
    for ((profile, _), config) in profiles.iter().zip(configs.iter()) {
        let tokenizer = TextEmbeddingTokenizer::load(config).unwrap();
        let rows: Vec<_> = ["identity", "passage"]
            .into_iter()
            .flat_map(|kind| {
                let inputs = inputs.get(kind).unwrap();
                assert!(!inputs.is_empty());
                let tokenizer = &tokenizer;
                inputs.iter().take(4).map(move |(hash, input)| {
                    serde_json::json!({
                        "kind": kind, "hash": hash, "text": input,
                        "tokens": tokenizer.validate_document_input(input).unwrap(),
                    })
                })
            })
            .collect();
        samples.insert(profile, rows);
    }
    fs::write(output, serde_json::to_vec_pretty(&samples).unwrap()).unwrap();
}
