//! Shared Foundry content interpretation and derived projections.
//!
//! Authored markup remains in the source DTO. Preparation does not persist its
//! transient parser tree or evaluate gameplay expressions. The existing artifact
//! pipeline also calls this owner for parsing; its storage format is unchanged.

mod model;
mod parse_diagnostics;
mod parser;
#[cfg(test)]
mod parser_tests;
mod preparation;
#[cfg(test)]
mod tests;
mod text_projection;

pub use model::{
    CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentDiagnosticCode, ContentInteraction,
    ContentInteractionKind, ContentInterpretationDiagnostic, ContentReferenceKind,
    ContentReferenceOccurrence, ContentReferenceResolution, ContentReferenceResolver,
    ContentReferenceTarget, ContentVisibilityRule, OwnedContentIdentity, OwnedContentLocator,
    PreparedSourceContent, ResolvedContentReference, SourceContentLocator,
};
pub use parse_diagnostics::{ContentParseDiagnostics, DroppedContentMacro};
pub use parser::{
    LocalizationResolver, ParsedContentDocument, parse_foundry_content,
    parse_foundry_content_with_localization,
};
pub use preparation::prepare_source_content;
