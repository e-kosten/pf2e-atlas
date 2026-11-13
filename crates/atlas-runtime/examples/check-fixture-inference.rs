//! Replays frozen document inputs without opening a predecessor workspace.
use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, TextEmbedder};
use serde::Deserialize;
use std::{error::Error, fs::File, path::PathBuf};

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
    let mut embedder = TextEmbedder::load(&EmbeddingRuntimeConfig::new(model, cache))?;
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
        if vector.len() != sample.values.len()
            || vector.iter().zip(&sample.values).any(|(candidate, old)| {
                !candidate.is_finite() || !old.is_finite() || (candidate - old).abs() > 2e-5
            })
        {
            return Err(format!("Document vector drift for {}", sample.hash).into());
        }
    }
    println!("Compared {} frozen document vectors", samples.len());
    Ok(())
}
