mod builder;
mod generation;
mod model;
mod token_budget;

pub use builder::build_document_embedding_units;
pub use generation::{
    generate_document_embeddings, generate_document_embeddings_with_reuse,
    generate_document_embeddings_with_reuse_using,
    generate_document_embeddings_with_reuse_using_batch,
};
pub use model::{
    DocumentEmbeddingChunkBudgetDiagnostic, DocumentEmbeddingChunkBudgetDiagnosticChunk,
    DocumentEmbeddingContentSource, DocumentEmbeddingRecordTruncationCoverage,
    DocumentEmbeddingSectionTruncation, DocumentEmbeddingSource,
    DocumentEmbeddingTokenizationTelemetry, DocumentEmbeddingTruncationExample,
    DocumentEmbeddingUnitKindTruncation, GeneratedDocumentEmbedding, GeneratedDocumentEmbeddings,
    PendingDocumentEmbedding, ReusableDocumentEmbedding,
};
pub use token_budget::{
    apply_document_embedding_token_budget, apply_document_embedding_token_budget_with_diagnostics,
};

#[cfg(test)]
mod tests;
