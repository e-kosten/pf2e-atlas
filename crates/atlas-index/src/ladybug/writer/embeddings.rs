use crate::IndexBuildInput;

#[derive(Debug, Clone)]
pub(crate) struct LadybugEmbedding {
    pub(crate) embedding_unit_key: String,
    pub(crate) record_key: String,
    pub(crate) unit_kind: String,
    pub(crate) label: Option<String>,
    pub(crate) ordinal: i64,
    pub(crate) semantic_input_hash: String,
    pub(crate) dimensions: usize,
    pub(crate) vector: Vec<f32>,
}

pub(crate) fn ladybug_embeddings(input: &IndexBuildInput<'_>) -> Vec<LadybugEmbedding> {
    input
        .document_embeddings
        .iter()
        .map(|embedding| LadybugEmbedding {
            embedding_unit_key: embedding.embedding_unit_key.clone(),
            record_key: embedding.record_key.clone(),
            unit_kind: embedding.unit_kind.as_str().to_string(),
            label: embedding.label.clone(),
            ordinal: embedding.ordinal as i64,
            semantic_input_hash: embedding.input_hash.clone(),
            dimensions: embedding.dimensions,
            vector: embedding.vector.clone(),
        })
        .collect()
}
