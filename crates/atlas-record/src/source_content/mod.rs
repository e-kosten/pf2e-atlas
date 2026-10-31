//! Shared Foundry content interpretation and derived projections.
//!
//! Authored markup remains in the source DTO. Preparation does not persist its
//! transient parser tree or bind runtime gameplay values. Expression trees remain
//! transient during preparation and selected reading.

mod contextual_formula;
pub use contextual_formula::{
    FormulaContext, FormulaText, contextual_damage_text, contextual_roll_text,
    describe_contextual_formula, render_contextual_interactions, resolve_contextual_formula,
    resolved_contextual_damage, resolved_contextual_roll,
};
mod inline_action;
mod literal_damage;
mod literal_roll;
pub use literal_roll::{LiteralContentRoll, literal_content_roll};
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
mod read_only_text;
#[cfg(test)]
mod tests;
mod text_projection;

pub use literal_damage::{LiteralDamageComponent, LiteralDamageKind, literal_damage};
pub use marker_validation::{
    PreparedMarkerError, validate_prepared_markers, validate_prepared_reference_bindings,
    validate_prepared_trait_reference_bindings,
};
pub use model::{
    CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentDiagnosticCode, ContentInteraction,
    ContentInteractionKind, ContentInteractionLabel, ContentInterpretationDiagnostic,
    ContentReferenceKind, ContentReferenceLabel, ContentReferenceLookup,
    ContentReferenceOccurrence, ContentReferenceResolution, ContentReferenceResolver,
    ContentReferenceTarget, ContentVisibilityRule, PreparedSourceContent, PreparedTraitDefinition,
    ResolvedContentReference, SourceContentLocator,
};
pub use parse_diagnostics::{ContentParseDiagnostics, DroppedContentMacro};
pub use parser::LocalizationResolver;
pub use preparation::{is_safe_content_url, prepare_source_content, prepare_trait_definition};
pub use text_projection::plain_text as prepared_plain_text;

mod trait_references;
pub use trait_references::{TraitLocalizationReference, trait_localization_references};

mod inclusion;
pub use inclusion::{
    PreparedHtmlInclusion, PreparedInclusionSourceLink, compose_prepared_inclusions,
    escape_content_text, terminal_prepared_reference,
};

mod reading_metadata;
pub use reading_metadata::{RoutineHeader, check_traits, routine_header};
