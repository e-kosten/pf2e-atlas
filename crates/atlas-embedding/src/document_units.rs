mod builder;
mod generation;
mod model;
mod token_budget;

pub use builder::build_document_embedding_units;
pub use generation::{
    DocumentEmbeddingGenerationError, generate_document_embeddings,
    generate_document_embeddings_with_reuse, generate_document_embeddings_with_reuse_using,
    generate_document_embeddings_with_reuse_using_batch,
};
pub use model::{
    DocumentEmbeddingContentSource, DocumentEmbeddingRecordTruncationCoverage,
    DocumentEmbeddingSectionTruncation, DocumentEmbeddingSource,
    DocumentEmbeddingTokenizationTelemetry, DocumentEmbeddingTruncationExample,
    DocumentEmbeddingUnitKindTruncation, GeneratedDocumentEmbedding, GeneratedDocumentEmbeddings,
    PendingDocumentEmbedding, ReusableDocumentEmbedding,
};
pub use token_budget::apply_document_embedding_token_budget;

#[cfg(test)]
mod tests;
