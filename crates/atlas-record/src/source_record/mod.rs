//! Database-independent records backed by the complete admitted Foundry DTO.
//! Views borrow source fields; explicit operations produce interpreted output.
mod actor_presentation;
mod actor_query;
mod availability;
mod content;
#[cfg(test)]
mod declared_grant_tests;
mod identity;
mod item_query;
mod model;
mod nodes;
mod query;
mod relationships;
mod resolver;
mod search_selection;
mod shield_content;
#[cfg(test)]
mod tests;
mod text;
mod traversal;

pub use actor_presentation::SourceNpcAdjustment;
pub use actor_query::{
    ActorIwrEntries, ActorIwrEntry, ActorIwrKind, ActorIwrType, ActorQueryView, ActorSave,
    ActorSpeedEntry, ActorSpeeds,
};
pub use availability::{FieldAvailability, SourceFieldView};
pub use content::{
    SourceContentFormat, SourceContentOutcome, SourceContentRole, SourceContentSelection,
    SourceContentStatus, prepare_record_content, trait_record_localization_references,
};
pub use identity::{SourceIdentityError, source_record_key};
pub use item_query::spell_casting_form;
pub use model::{SourceBackedRecord, SourceRecordConstructionError, source_owned_document_count};
pub use nodes::{ItemSourceView, SourceNodeView};
pub use query::{SourceQueryView, source_family_record_kind};
pub use relationships::{
    SourceRelationshipKind, SourceRelationshipOccurrence, resolve_source_relationships,
};
pub use resolver::SourceReferenceIndex;
pub use search_selection::{
    SOURCE_CONTENT_SELECTION_VERSION, SourceSearchSelection, SourceSelectedField,
    SourceSelectedIdentity, SourceSelectedSection, SourceSelectedSectionKind, SourceSelectionError,
    recover_source_passage, select_prepared_html_sections, select_record_search,
};
pub use text::{SourceTextKind, SourceTextSource};

mod trait_memberships;
pub use trait_memberships::{SourceTraitMembership, trait_record_memberships};

mod inline_embed;
pub use inline_embed::{InlineEmbedOptions, inline_embed_fields};
