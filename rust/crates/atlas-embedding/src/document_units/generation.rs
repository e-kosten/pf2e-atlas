use std::collections::BTreeMap;

use tracing::info;

use crate::error::EmbeddingError;
use crate::minilm::TextEmbedder;

use super::model::{
    GeneratedDocumentEmbedding, GeneratedDocumentEmbeddings, PendingDocumentEmbedding,
    ReusableDocumentEmbedding,
};

pub fn generate_document_embeddings(
    pending: &[PendingDocumentEmbedding],
    embedder: &mut TextEmbedder,
) -> Result<Vec<GeneratedDocumentEmbedding>, EmbeddingError> {
    Ok(
        generate_document_embeddings_with_reuse_using(pending, None, |input| {
            embedder.embed_document(input)
        })?
        .embeddings,
    )
}

pub fn generate_document_embeddings_with_reuse(
    pending: &[PendingDocumentEmbedding],
    reusable_embeddings: Option<&BTreeMap<String, ReusableDocumentEmbedding>>,
    embedder: &mut TextEmbedder,
) -> Result<GeneratedDocumentEmbeddings, EmbeddingError> {
    generate_document_embeddings_with_reuse_using_batch(pending, reusable_embeddings, 1, |inputs| {
        embedder.embed_documents(inputs)
    })
}

pub fn generate_document_embeddings_with_reuse_using<E>(
    pending: &[PendingDocumentEmbedding],
    reusable_embeddings: Option<&BTreeMap<String, ReusableDocumentEmbedding>>,
    mut embed_document: impl FnMut(&str) -> Result<Vec<f32>, E>,
) -> Result<GeneratedDocumentEmbeddings, E> {
    generate_document_embeddings_with_reuse_using_batch(pending, reusable_embeddings, 1, |inputs| {
        inputs
            .iter()
            .map(|input| embed_document(input))
            .collect::<Result<Vec<_>, _>>()
    })
}

pub fn generate_document_embeddings_with_reuse_using_batch<E>(
    pending: &[PendingDocumentEmbedding],
    reusable_embeddings: Option<&BTreeMap<String, ReusableDocumentEmbedding>>,
    batch_size: usize,
    mut embed_documents: impl FnMut(&[&str]) -> Result<Vec<Vec<f32>>, E>,
) -> Result<GeneratedDocumentEmbeddings, E> {
    let total = pending.len();
    let progress_interval = embedding_progress_interval(total);
    let batch_size = batch_size.max(1);
    let mut generated = vec![None; total];
    let mut pending_generation_indices = Vec::new();
    let mut reused_count = 0;
    let mut generated_count = 0;
    let mut last_reported = 0;
    for (index, entry) in pending.iter().enumerate() {
        if let Some(reusable) = reusable_embeddings
            .and_then(|lookup| lookup.get(&entry.embedding_unit_key))
            .filter(|reusable| reusable.input_hash == entry.input_hash)
        {
            reused_count += 1;
            generated[index] = Some(GeneratedDocumentEmbedding {
                embedding_unit_key: entry.embedding_unit_key.clone(),
                record_key: entry.record_key.clone(),
                unit_kind: entry.unit_kind,
                label: entry.label.clone(),
                ordinal: entry.ordinal,
                input_hash: entry.input_hash.clone(),
                dimensions: reusable.dimensions,
                vector: reusable.vector.clone(),
            });
            continue;
        }
        pending_generation_indices.push(index);
    }

    for chunk in pending_generation_indices.chunks(batch_size) {
        let current = chunk.last().copied().unwrap_or(0) + 1;
        let record_key = pending[current - 1].record_key.as_str();
        let inputs = chunk
            .iter()
            .map(|index| pending[*index].input_text.as_str())
            .collect::<Vec<_>>();
        let vectors = embed_documents(&inputs)?;
        debug_assert_eq!(vectors.len(), chunk.len());
        for (chunk_index, vector) in vectors.into_iter().enumerate() {
            let index = chunk[chunk_index];
            let entry = &pending[index];
            generated_count += 1;
            generated[index] = Some(GeneratedDocumentEmbedding {
                embedding_unit_key: entry.embedding_unit_key.clone(),
                record_key: entry.record_key.clone(),
                unit_kind: entry.unit_kind,
                label: entry.label.clone(),
                ordinal: entry.ordinal,
                input_hash: entry.input_hash.clone(),
                dimensions: vector.len(),
                vector,
            });
        }
        if last_reported == 0 || current - last_reported >= progress_interval || current == total {
            info!(target: "atlas_progress",
                phase = "document_embeddings",
                current = current as u64,
                total = total as u64,
                "Prepared document embedding batch through: {record_key}",
                record_key = record_key
            );
            last_reported = current;
        }
    }
    if pending_generation_indices.is_empty() && total > 0 {
        info!(target: "atlas_progress",
            phase = "document_embeddings",
            current = total as u64,
            total = total as u64,
            "Prepared document embeddings from reusable cache"
        );
    }

    Ok(GeneratedDocumentEmbeddings {
        embeddings: generated
            .into_iter()
            .map(|entry| entry.expect("every pending embedding is generated or reused"))
            .collect(),
        reused_count,
        generated_count,
    })
}

fn embedding_progress_interval(total: usize) -> usize {
    (total / 100).clamp(25, 500)
}
