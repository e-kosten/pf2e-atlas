//! Typed authored-source loading, before Atlas normalization or storage.
mod loader;
mod model;
mod report;

pub use loader::load_foundry_documents;
pub use model::{
    FoundryDocumentSource, LoadedFoundryDocument, LoadedFoundryPack, LoadedFoundrySource,
    QuarantinedSourceFile, SourceFileProvenance, SourceLoadFailure, SourceLoadFailureStage,
};
pub use report::{SourceDocumentCounts, SourceLoadingReport};
