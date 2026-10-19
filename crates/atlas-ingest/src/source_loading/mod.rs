//! Typed authored-source loading, before Atlas normalization or storage.
mod loader;
pub(crate) use loader::load_captured_source;
mod metadata;
mod model;
mod report;

pub use loader::{load_foundry_documents, load_foundry_documents_with_reuse};
pub use metadata::{SourceMetadata, SourcePackMetadata};
pub use model::{
    LoadedFoundryDocument, LoadedFoundryPack, LoadedFoundrySource, QuarantinedSourceFile,
    SourceFileProvenance, SourceLoadFailure, SourceLoadFailureStage,
};
pub use report::{SourceDocumentCounts, SourceLoadingReport};
