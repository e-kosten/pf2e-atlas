use atlas_record::{ContentDocument, ContentSourceKind, RecordPresentationDocument};

use crate::document_renderer::{EmbeddingInputChunk, EmbeddingInputSection};
use crate::tokenization::EmbeddingChunkBudgetOutcome;
use crate::unit_kind::EmbeddingUnitKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingSource {
    pub record_key: String,
    pub record_name: String,
    pub document: RecordPresentationDocument,
    pub aliases: Vec<String>,
    pub content_documents: Vec<DocumentEmbeddingContentSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingContentSource {
    pub source_kind: ContentSourceKind,
    pub label: Option<String>,
    pub document: ContentDocument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingDocumentEmbedding {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: EmbeddingUnitKind,
    pub label: Option<String>,
    pub ordinal: usize,
    pub input_chunks: Vec<EmbeddingInputChunk>,
    pub input_text: String,
    pub input_hash: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedDocumentEmbedding {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: EmbeddingUnitKind,
    pub label: Option<String>,
    pub ordinal: usize,
    pub input_hash: String,
    pub dimensions: usize,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReusableDocumentEmbedding {
    pub input_hash: String,
    pub dimensions: usize,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedDocumentEmbeddings {
    pub embeddings: Vec<GeneratedDocumentEmbedding>,
    pub reused_count: usize,
    pub generated_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentEmbeddingTokenizationTelemetry {
    pub document_count: usize,
    pub truncated_document_count: usize,
    pub max_token_count: Option<usize>,
    pub max_observed_token_count: usize,
    pub total_observed_token_count: usize,
    pub total_tokens_over_limit: usize,
    pub unit_kind_truncations: Vec<DocumentEmbeddingUnitKindTruncation>,
    pub record_truncation_coverage: DocumentEmbeddingRecordTruncationCoverage,
    pub section_truncations: Vec<DocumentEmbeddingSectionTruncation>,
    pub truncated_examples: Vec<DocumentEmbeddingTruncationExample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingTruncationExample {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: EmbeddingUnitKind,
    pub label: Option<String>,
    pub token_count: usize,
    pub max_token_count: usize,
    pub truncated_sections: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentEmbeddingUnitKindTruncation {
    pub unit_kind: String,
    pub unit_count: usize,
    pub record_count: usize,
    pub total_tokens_over_limit: usize,
    pub max_observed_token_count: usize,
    pub examples: Vec<DocumentEmbeddingTruncationExample>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentEmbeddingRecordTruncationCoverage {
    pub record_count: usize,
    pub records_with_child_units: usize,
    pub records_with_any_truncated_unit: usize,
    pub records_with_truncated_parent_unit: usize,
    pub records_with_truncated_child_unit: usize,
    pub records_with_truncated_parent_and_child_units: usize,
    pub records_with_truncated_parent_and_all_child_units_fit: usize,
    pub records_with_truncated_parent_without_child_units: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingSectionTruncation {
    pub section: String,
    pub document_count: usize,
    pub dropped_chunk_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingChunkBudgetDiagnostic {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: EmbeddingUnitKind,
    pub label: Option<String>,
    pub original_token_count: usize,
    pub final_token_count: usize,
    pub max_token_count: usize,
    pub original_chunk_count: usize,
    pub final_chunk_count: usize,
    pub chunks: Vec<DocumentEmbeddingChunkBudgetDiagnosticChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingChunkBudgetDiagnosticChunk {
    pub section: EmbeddingInputSection,
    pub outcome: EmbeddingChunkBudgetOutcome,
    pub original_text: String,
    pub final_text: Option<String>,
}
