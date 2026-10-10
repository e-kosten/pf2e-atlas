use crate::SourceEnrichmentReport;
use atlas_embedding::EmbeddingRuntimeConfig;
use std::path::PathBuf;
#[derive(Debug, Clone)]
pub struct BuildArtifactOptions {
    pub source_root: PathBuf,
    pub output_path: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub locale: String,
    /// None builds a complete lexical artifact without native vector activation.
    pub embedding: Option<EmbeddingRuntimeConfig>,
    pub reuse_embeddings: bool,
    pub embedding_batch_size: usize,
}
#[derive(Debug)]
pub struct BuildArtifactReport {
    pub output_path: PathBuf,
    pub pack_count: usize,
    pub record_count: usize,
    pub product_record_count: usize,
    pub semantic_unit_count: usize,
    pub inferred_inputs: usize,
    pub reused_inputs: usize,
    pub context_shortened_sections: usize,
    pub source_fingerprint: String,
    pub build_duration_ms: u128,
    pub source_report: SourceEnrichmentReport,
}
