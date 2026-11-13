//! Replays frozen document inputs without opening a predecessor workspace.
use atlas_embedding::{EmbeddingError, EmbeddingModelId, EmbeddingRuntimeConfig, TextEmbedder};
use serde::Deserialize;
use std::{error::Error, fs::File, path::PathBuf, time::Instant};

#[derive(Deserialize)]
struct Sample {
    text: String,
    hash: String,
    tokens: usize,
    values: Vec<f32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let cache = PathBuf::from(args.next().ok_or("expected model cache")?);
    let model = match args.next().as_deref().and_then(std::ffi::OsStr::to_str) {
        Some("bge-base") => EmbeddingModelId::BgeBaseEnV15,
        Some("embeddinggemma") => EmbeddingModelId::EmbeddingGemma300M,
        _ => return Err("expected a supported profile".into()),
    };
    let path = PathBuf::from(args.next().ok_or("expected numerical samples")?);
    let samples: Vec<Sample> = serde_json::from_reader(File::open(path)?)?;
    if samples.is_empty() {
        return Err("empty numerical sample set".into());
    }
    let stress = args.next().is_some_and(|arg| arg == "--near-limit");
    let start = Instant::now();
    let mut embedder = TextEmbedder::load(&EmbeddingRuntimeConfig::new(model, cache))?;
    let load_seconds = start.elapsed().as_secs_f64();
    let inference_start = Instant::now();
    for sample in &samples {
        if atlas_embedding::hash_document_embedding_input(&sample.text) != sample.hash
            || embedder.tokenizer().validate_document_input(&sample.text)? != sample.tokens
        {
            return Err(
                format!("Document input or token count changed for {}", sample.hash).into(),
            );
        }
    }
    let texts: Vec<_> = samples.iter().map(|sample| sample.text.as_str()).collect();
    let vectors = embedder.embed_documents(&texts)?;
    for (sample, vector) in samples.iter().zip(vectors) {
        if !vectors_match(&vector, &sample.values) {
            return Err(format!("Document vector drift for {}", sample.hash).into());
        }
    }
    check_mixed_batches(&mut embedder, stress)?;
    println!(
        "{model:?}: compared {} frozen document vectors; session load {:.3}s; replay and batch checks {:.3}s",
        samples.len(),
        load_seconds,
        inference_start.elapsed().as_secs_f64()
    );
    Ok(())
}

fn vectors_match(candidate: &[f32], expected: &[f32]) -> bool {
    candidate.len() == expected.len()
        && candidate.iter().zip(expected).all(|(actual, expected)| {
            actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= 2e-5
        })
}

fn check_mixed_batches(embedder: &mut TextEmbedder, stress: bool) -> Result<(), Box<dyn Error>> {
    let short = "Undead";
    let medium = "Résumé @item.level: 2d8 HP. 𐐀 café 火星. ".repeat(4);
    for query in [false, true] {
        let long = if stress {
            let tokenizer = embedder.tokenizer();
            let limit = embedder
                .spec()
                .max_input_tokens
                .ok_or("missing token limit")?;
            let phrase = "Café 火星 ≥ ½ ";
            let count = |input: &str| {
                if query {
                    tokenizer.validate_query_input(input)
                } else {
                    tokenizer.validate_document_input(input)
                }
            };
            let (mut low, mut high) = (1, limit);
            while high - low > 1 {
                let middle = (low + high) / 2;
                match count(&phrase.repeat(middle)) {
                    Ok(_) => low = middle,
                    Err(EmbeddingError::TokenBudgetExceeded { .. }) => high = middle,
                    Err(error) => return Err(error.into()),
                }
            }
            let input = phrase.repeat(low);
            if count(&input)? < limit - 16
                || !matches!(
                    count(&phrase.repeat(high)),
                    Err(EmbeddingError::TokenBudgetExceeded { .. })
                )
            {
                return Err("mixed batch failed to exercise the token limit".into());
            }
            input
        } else {
            medium.clone()
        };
        let texts = [short, long.as_str()];
        let batch = if query {
            embedder.embed_queries(&texts)?
        } else {
            embedder.embed_documents(&texts)?
        };
        for (position, (text, vector)) in texts.into_iter().zip(batch).enumerate() {
            let single = if query {
                embedder.embed_query(text)?
            } else {
                embedder.embed_document(text)?
            };
            if !vectors_match(&vector, &single) {
                return Err(format!(
                    "Mixed-length batch changed vector {position} (query={query})"
                )
                .into());
            }
        }
    }
    Ok(())
}
