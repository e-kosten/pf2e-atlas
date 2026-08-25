#![deny(unsafe_code)]

mod content;
mod creature;
mod creature_entities;
mod creature_projection;
mod json_projection;
mod mechanics_view;
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
    ContentDiagnostic, ContentDiagnosticKind, ContentExclusion, ContentExclusionReason,
    ContentFtsField, ContentHash, ContentId, ContentIdentityStability, ContentKey, ContentOrigin,
    ContentOwner, ContentProvenance, ContentReferenceOccurrence, ContentRole, ContentSectionNode,
    ContentSectionOrigin, ContentSemanticInputHash, ContentSourceKind, ContentVisibility,
    DamagePart, DuplicateContentStatus, FoundryLink, FoundryLinkBehavior, FoundryLinkIter,
    FoundryLinkMacroKind, FoundryLinkSource, FoundryNode, InvalidContentKey, OwnedRichContent,
    OwnedRichContentDocument, RecordContentDocument, RecordFtsProjection, ReferenceRelationKind,
    RichDocument, RichLinkTarget, RichNode, build_content_section_tree,
    build_record_fts_projection, iter_foundry_links, render_markdown_like, render_plain_text,
    visit_foundry_links_mut,
};
pub use creature::{
    CreatureAdjustment, CreatureAllianceName, CreatureArmorClass, CreatureComponentId,
    CreatureDefenses, CreatureDerivation, CreatureFact, CreatureFactProvenance, CreatureFamily,
    CreatureHitPoints, CreatureIdentity, CreatureInitiative, CreatureInitiativeStatistic,
    CreatureIwr, CreatureIwrKind, CreatureLanguages, CreatureLegacyAbilities, CreatureMovementMode,
    CreatureNote, CreatureNumber, CreaturePerception, CreaturePredicate, CreatureProvenance,
    CreaturePublication, CreatureRecord, CreatureResource, CreatureResourceAmount,
    CreatureResourceKind, CreatureSave, CreatureSaveKind, CreatureSaves, CreatureSense,
    CreatureShield, CreatureSize, CreatureSkill, CreatureSkillKind, CreatureSkillVariant,
    CreatureSourceAlliance, CreatureSourceField, CreatureSourceId, CreatureSpeed,
    CreatureStatistic, CreatureTrait, CreatureUnsupportedSourceFact,
    CreatureUnsupportedSourceField, FactValue, InvalidCanonicalId, InvalidCanonicalSlug,
    InvalidCanonicalValue, IwrQualifier, IwrType, Language, PredicateTerm, PublicationLicense,
    RecordBody, ResourceCurrentPolicy, SenseAcuity, SenseType, ShieldCurrentPolicy,
    UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
};
pub use creature_entities::{
    CreatureActionCapability, CreatureActionCost, CreatureActorSpellcastingContext,
    CreatureCapability, CreatureDamage, CreatureDamageKind, CreatureDeltaDisposition,
    CreatureDeltaValue, CreatureEmbeddedEntities, CreatureEntity, CreatureEntityFamily,
    CreatureEntityId, CreatureEntityOccurrence, CreatureEntityRelationship,
    CreatureEntityRelationshipKind, CreatureEntitySourceIdentity, CreatureEntityTarget,
    CreatureEquipmentCapability, CreatureFrequency, CreatureLoreCapability,
    CreatureOccurrenceContext, CreatureOccurrenceDelta, CreatureOccurrenceId,
    CreatureOccurrenceParent, CreaturePreparedSpellSlot, CreatureRelationshipExecution,
    CreatureRelationshipTarget, CreatureRitualContext, CreatureRoll, CreatureRollKind,
    CreatureSourceLocator, CreatureSourceScalar, CreatureSpellArea, CreatureSpellCapability,
    CreatureSpellDefense, CreatureSpellDuration, CreatureSpellPreparation, CreatureSpellSave,
    CreatureSpellSlot, CreatureSpellcastingEntryCapability, CreatureStrikeCapability,
    CreatureUnsupportedCapability, CreatureUseLimit, InvalidCreatureEntityId,
    InvalidStableSourceLocator, OccurrenceIdentityStability, StableSourceLocator,
    UnsupportedMechanicNote,
};
pub use creature_projection::{CreatureFactProjection, project_creature_facts};
pub use json_projection::{
    RecordBlockJson, RecordJson, RecordJsonOptions, RecordSectionJson, record_json,
};
pub use mechanics_view::{
    AbilityKind, MechanicFacets, MechanicScalar, MechanicStatistic, MechanicSurface,
    MechanicTarget, MechanicValue, MechanicsView, MovementSpeed, SaveKind, build_mechanics_view,
};
pub use metrics::{
    MetricCapture, MetricDefinition, MetricDefinitionMatch, MetricDisplayLabel, MetricGroup,
    MetricKeyDefinition, MetricKeyPattern, MetricKeySegment, MetricLabelTemplate,
    MetricVariableVocabulary, all_definitions, definition_for, is_known_key, label_for_row,
    normalize_metric_key_segment,
};
pub use model::{
    ActivationTimeSourceField, ActivityRoll, ActivityRollAbility, ActivityRollSurface,
    ActorMechanics, AliasSource, AtlasRecord, AtlasRecordSet, DamageEffectKind, DamageExpression,
    DefaultRetrievalVisibility, DurationTimeSourceField, FoundryDocumentMechanics,
    FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, ItemMechanics, ItemTypeMechanics,
    MechanicActivity, MechanicActivityKind, MechanicActivityMode, MechanicActivityUsage, MetricRow,
    MetricValue, NormalizedTime, RecordActivationTiming, RecordAlias, RecordClassification,
    RecordContent, RecordDurationTiming, RecordIdentity, RecordMechanics, RecordProvenance,
    RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming, RecordVariantMembership,
    RecordVisibility, RecordVisibilityReason, ReferenceEdge, RemasterLink, SpellArea, SpellDefense,
    SpellMechanics, SpellRange, SpellTarget, SpellcastingEntryMechanics, SpellcastingPreparation,
    VariantSource,
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
