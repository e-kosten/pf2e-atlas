//! Database-independent records backed by the complete admitted Foundry DTO.
//! Views borrow source fields; enrichment carries identity and interpreted facts.
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

pub use availability::{FieldAvailability, SourceFieldView};
pub use content::{
    SourceContentFormat, SourceContentOutcome, SourceContentRole, SourceContentStatus,
    enrich_source_record,
};
pub use identity::{SourceIdentityError, source_record_key};
pub use model::{OwnedCollectionFact, OwnedNodeFact, SourceBackedRecord, SourceRecordEnrichment};
pub use nodes::{ItemSourceView, SourceNodeEntry, SourceNodeView, source_nodes};
pub use query::{ActorQueryView, SourceQueryView, SourceTextKind, SourceTextSource};
pub use relationships::{SourceRelationshipKind, SourceRelationshipOccurrence};
pub use resolver::SourceReferenceIndex;
