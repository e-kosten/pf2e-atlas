#![deny(unsafe_code)]

mod content;
mod json_projection;
pub mod metrics;
mod model;
mod presentation;
mod presentation_content;
mod presentation_format;
mod presentation_recipe;
#[cfg(test)]
mod presentation_recipe_tests;
mod reference_policy;

pub use content::{
    ContentFtsField, ContentSectionNode, ContentSectionOrigin, ContentSourceKind,
    ContentVisibility, DamagePart, FoundryLink, FoundryLinkBehavior, FoundryLinkIter,
    FoundryLinkMacroKind, FoundryLinkSource, FoundryNode, RecordContentDocument,
    RecordFtsProjection, ReferenceRelationKind, RichDocument, RichLinkTarget, RichNode,
    build_content_section_tree, build_record_fts_projection, iter_foundry_links,
    render_markdown_like, render_plain_text, visit_foundry_links_mut,
};
pub use json_projection::{
    RecordBlockJson, RecordJson, RecordJsonOptions, RecordSectionJson, record_json,
};
pub use metrics::{
    MetricCapture, MetricDefinition, MetricDefinitionMatch, MetricDisplayLabel, MetricGroup,
    MetricKeyDefinition, MetricKeyPattern, MetricKeySegment, MetricLabelTemplate,
    MetricVariableVocabulary, all_definitions, definition_for, is_known_key, label_for_row,
};
pub use model::{
    ActivationTimeSourceField, ActorMechanics, AliasSource, AtlasRecord, AtlasRecordSet,
    DefaultRetrievalVisibility, DurationTimeSourceField, FoundryDocumentMechanics,
    FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, ItemMechanics, ItemTypeMechanics,
    MetricRow, MetricValue, NormalizedTime, RecordActivationTiming, RecordAlias,
    RecordClassification, RecordContent, RecordDurationTiming, RecordIdentity, RecordMechanics,
    RecordProvenance, RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming,
    RecordVariantMembership, RecordVisibility, RecordVisibilityReason, ReferenceEdge, RemasterLink,
    SpellArea, SpellDefense, SpellMechanics, SpellRange, SpellTarget, VariantSource,
};
pub use presentation::{
    PresentationBadge, PresentationBadgeKind, PresentationBlock, PresentationContent,
    PresentationContentBlock, PresentationFact, PresentationInline, PresentationListItem,
    PresentationRelationship, PresentationRelationshipKind, PresentationSection,
    PresentationSectionKind, PresentationTableRow, PresentationText, RecordPresentationDocument,
};
pub use presentation_content::render_presentation_content_plain_text;
pub use presentation_format::format_size;
pub use presentation_recipe::{
    build_record_presentation_document, build_record_presentation_document_with_content_filter,
};
pub use reference_policy::{
    DEFAULT_EXCLUDED_SOURCE_KINDS, ReferenceEdgeFacts, ReferenceGraphMode, ReferenceGraphPolicy,
    ReferenceVisibilityPolicy, reference_edge_matches_mode, reference_graph_policy,
};
