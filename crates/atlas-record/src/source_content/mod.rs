//! Shared Foundry content interpretation and derived projections.
//!
//! Authored markup remains in the source DTO. Preparation does not persist its
//! transient parser tree or evaluate gameplay expressions. The private interpreter tree exists only during field preparation.

mod marker_validation;
mod model;
mod parse_diagnostics;
mod parsed_markup;
#[cfg(test)]
mod parsed_text;
mod parser;
#[cfg(test)]
mod parser_tests;
mod preparation;
#[cfg(test)]
mod tests;
mod text_projection;

pub use marker_validation::{
    PreparedMarkerError, validate_prepared_markers, validate_prepared_reference_bindings,
};
pub use model::{
    CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentDiagnosticCode, ContentInteraction,
    ContentInteractionKind, ContentInterpretationDiagnostic, ContentReferenceKind,
    ContentReferenceOccurrence, ContentReferenceResolution, ContentReferenceResolver,
    ContentReferenceTarget, ContentVisibilityRule, OwnedContentIdentity, OwnedContentLocator,
    PreparedSourceContent, ResolvedContentReference, SourceContentLocator,
};
pub use parse_diagnostics::{ContentParseDiagnostics, DroppedContentMacro};
pub use parser::LocalizationResolver;
pub use preparation::{is_safe_content_url, prepare_source_content};
pub use text_projection::plain_text as prepared_plain_text;
