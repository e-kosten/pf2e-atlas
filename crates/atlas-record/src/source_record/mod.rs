//! Database-independent records backed by the complete admitted Foundry DTO.
//! Views borrow source fields; explicit operations produce interpreted output.
mod availability;
mod content;
mod identity;
mod model;
mod nodes;
mod query;
mod relationships;
mod resolver;
#[cfg(test)]
mod tests;
mod text;
mod traversal;

pub use availability::{FieldAvailability, SourceFieldView};
pub use content::{
    SourceContentFormat, SourceContentOutcome, SourceContentRole, SourceContentStatus,
    prepare_record_content,
};
pub use identity::{SourceIdentityError, source_record_key};
pub use model::{SourceBackedRecord, SourceRecordConstructionError, source_owned_document_count};
pub use nodes::{ItemSourceView, SourceNodeView};
pub use query::{ActorQueryView, SourceQueryView};
pub use relationships::{
    SourceRelationshipKind, SourceRelationshipOccurrence, resolve_source_relationships,
};
pub use resolver::SourceReferenceIndex;
pub use text::{SourceTextKind, SourceTextSource};
