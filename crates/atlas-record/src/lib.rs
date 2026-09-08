#![deny(unsafe_code)]

mod content;
mod creature;
mod creature_content_placement;
mod creature_entities;
mod creature_projection;
mod hazard;
mod hazard_applicability;
mod hazard_projection;
#[cfg(test)]
mod hazard_tests;
mod json_projection;
mod mechanics;
pub mod metrics;
mod model;
mod presentation;
mod presentation_content;
mod presentation_format;
mod presentation_policy;
mod presentation_recipe;
#[cfg(test)]
mod presentation_recipe_tests;
mod reference_policy;
mod retrieval_policy;
mod retrieved_record;
mod spell;

pub use content::{
    ContentDiagnostic, ContentDiagnosticKind, ContentExclusion, ContentExclusionReason,
    ContentFtsField, ContentHash, ContentId, ContentIdentityStability, ContentKey, ContentOrigin,
    ContentOwner, ContentProvenance, ContentReferenceOccurrence, ContentRole, ContentSectionNode,
    ContentSectionOrigin, ContentSemanticInputHash, ContentSourceKind, ContentVisibility,
    DamagePart, DuplicateContentStatus, FoundryLink, FoundryLinkBehavior, FoundryLinkIter,
    FoundryLinkMacroKind, FoundryLinkSource, FoundryNode, InvalidContentKey, OwnedRichContent,
    OwnedRichContentDocument, RecordContentDocument, RecordFtsProjection, ReferenceRelationKind,
    RichDocument, RichLinkTarget, RichNode, build_content_section_tree,
    build_record_fts_projection, build_search_fts_projection,
    build_search_presentation_document_with_content_filter, iter_foundry_links,
    render_markdown_like, render_plain_text, visit_foundry_links_mut,
};
pub use creature::{
    CreatureAdjustment, CreatureAllianceName, CreatureArmorClass, CreatureComponentId,
    CreatureDefenses, CreatureDerivation, CreatureFact, CreatureFactProvenance, CreatureFamily,
    CreatureHitPoints, CreatureIdentity, CreatureInitiative, CreatureInitiativeStatistic,
    CreatureIwr, CreatureIwrKind, CreatureLanguages, CreatureLegacyAbilities, CreatureMovementMode,
    CreatureNote, CreatureNumber, CreaturePerception, CreaturePredicate, CreatureProvenance,
    CreaturePublication, CreatureRecord, CreatureResource, CreatureResourceAmount,
    CreatureResourceKind, CreatureSave, CreatureSaveKind, CreatureSaves, CreatureSense,
    CreatureShield, CreatureSize, CreatureSkill, CreatureSkillKind, CreatureSkillSourceEntry,
    CreatureSkillVariant, CreatureSourceAlliance, CreatureSourceField, CreatureSourceId,
    CreatureSpeed, CreatureStatistic, CreatureTrait, CreatureUnmodeledSkill,
    CreatureUnmodeledSkillReason, CreatureUnsupportedSourceFact, CreatureUnsupportedSourceField,
    FactValue, InvalidCanonicalId, InvalidCanonicalSlug, InvalidCanonicalValue, IwrQualifier,
    IwrType, Language, PredicateTerm, PublicationLicense, RecordBody, ResourceCurrentPolicy,
    SenseAcuity, SenseType, ShieldCurrentPolicy, UnsupportedSourceReason, UnsupportedSourceShape,
    UnsupportedSourceValue,
};
pub use creature_content_placement::{
    CreatureContentAssociationFailure, CreatureContentPlacement, place_creature_content,
    place_creature_content_for_families,
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
pub use hazard::{
    HazardActionCapability, HazardActionCategory, HazardActionCount, HazardActionType,
    HazardActiveEffectLikeRule, HazardAuraRule, HazardCapability, HazardComplexity,
    HazardComponentId, HazardConditionCapability, HazardDamageCategory, HazardDamageDiceRule,
    HazardDefenseSourceMetadata, HazardDefenses, HazardDetection, HazardDiagnosticCode,
    HazardEffectCapability, HazardEmbeddedEntities, HazardEmitsSound, HazardEntity,
    HazardEntityFamily, HazardEntityId, HazardEntityOccurrence, HazardEntitySourceIdentity,
    HazardExpectedShape, HazardFact, HazardFactProvenance, HazardFlatModifierRule, HazardFrequency,
    HazardFrequencyInterval, HazardHitPointSourceMetadata, HazardHitPoints, HazardIdentity,
    HazardImmunityRule, HazardItemCommon, HazardItemLineage, HazardIwr, HazardLifecycle,
    HazardNoteRule, HazardOccurrenceId, HazardOccurrenceIdentityStability, HazardProvenance,
    HazardProvenanceValue, HazardPublication, HazardRecord, HazardRelationship,
    HazardRelationshipId, HazardRelationshipKind, HazardRelationshipTarget, HazardRuleElement,
    HazardRuleMode, HazardRuleType, HazardSaveKind, HazardSaveSourceMetadata, HazardSaves,
    HazardSelfEffect, HazardSize, HazardSourceAttackMode, HazardSourceFactIdentity, HazardSourceId,
    HazardSourceShape, HazardSourceValue, HazardStrikeCapability, HazardStrikeDamage,
    HazardStrikeSourceMetadata, HazardTokenSourceMetadata, HazardTrait,
    HazardUnsupportedChildCapability, HazardUnsupportedFact, HazardUnsupportedField,
    HazardUnsupportedOwner, HazardUnsupportedRule, HazardUnsupportedValue,
    InvalidHazardComponentId, InvalidHazardEntityId, InvalidHazardOccurrenceId,
    InvalidHazardRelationshipId, InvalidHazardSlug, InvalidHazardSourceId,
};
pub use hazard_applicability::{
    HAZARD_APPLICABILITY_RULE_ID, HAZARD_APPLICABILITY_RULE_VERSION, HazardApplicabilityState,
    HazardDefenseApplicability, project_hazard_defense_applicability,
};
pub use hazard_projection::{
    HAZARD_CONVENIENCE_RULE_ID, HAZARD_CONVENIENCE_RULE_VERSION, HazardAttackEffect,
    HazardAttackEffectsProjection, HazardAttackMode, HazardAttackModeProjection,
    HazardConvenienceProjection, HazardFactProjection, HazardHasHealthConsistency,
    HazardInitiativeStatistic, HazardInitiativeSuggestion, HazardSourceMetadataFact,
    HazardSourceMetadataField, HazardSourceMetadataIssue, HazardSourceMetadataIssueKind,
    HazardSourceMetadataProjection, HazardStrikeActionCostProjection, HazardWeaponTypeConsistency,
    PF2E_HAZARD_ATTACK_MODE_RULE_ID, PF2E_HAZARD_ATTACK_MODE_RULE_VERSION,
    PF2E_STRIKE_ACTION_COST_RULE_ID, PF2E_STRIKE_ACTION_COST_RULE_VERSION,
    build_hazard_presentation_document, project_hazard_attack_effects, project_hazard_attack_mode,
    project_hazard_conveniences, project_hazard_facts, project_hazard_has_health_consistency,
    project_hazard_source_metadata, project_hazard_strike_action_cost,
    project_hazard_weapon_type_consistency,
};
pub use json_projection::{
    CreatureAbilitiesJson, CreatureActionCostJson, CreatureActionJson, CreatureArmorClassJson,
    CreatureAvailabilityEvidenceJson, CreatureAvailabilityFieldJson, CreatureAvailabilityJson,
    CreatureAvailabilityStateJson, CreatureContentJson, CreatureContentOwnerJson,
    CreatureContentProvenanceJson, CreatureDamageJson, CreatureDefensesJson, CreatureEquipmentJson,
    CreatureFactProvenanceJson, CreatureFactProvenanceSetJson, CreatureFrequencyJson,
    CreatureHitPointsJson, CreatureInitiativeJson, CreatureIntegerPresenceJson, CreatureIwrJson,
    CreatureLoreJson, CreatureMovementJson, CreatureMovementModeJson,
    CreatureOccurrenceContextJson, CreatureOccurrenceProvenanceJson, CreaturePerceptionJson,
    CreaturePreparedSpellJson, CreatureProvenanceJson, CreatureRelationshipJson,
    CreatureRelationshipTargetJson, CreatureResourceJson, CreatureRitualsJson, CreatureRollJson,
    CreatureSaveJson, CreatureSavesJson, CreatureSenseJson, CreatureShieldJson, CreatureSkillJson,
    CreatureSkillSourceEntryJson, CreatureSkillVariantJson, CreatureSpellAreaJson,
    CreatureSpellDefenseJson, CreatureSpellDurationJson, CreatureSpellJson,
    CreatureSpellRitualJson, CreatureSpellSlotJson, CreatureSpellcastingEntryJson,
    CreatureSpellcastingJson, CreatureStrikeJson, CreatureUnmodeledSkillAvailabilityJson,
    CreatureUnmodeledSkillJson, CreatureUseLimitJson, HazardAvailabilityJson,
    HazardAvailabilityStateJson, HazardContentProvenanceJson, HazardDefenseTerminalFact,
    HazardOccurrenceProvenanceJson, HazardProvenanceJson, RecordBlockJson,
    RecordCanonicalRelationshipJson, RecordEditionContextJson, RecordEditionCounterpartJson,
    RecordEditionCounterpartLookupJson, RecordEditionCounterpartRoleJson, RecordEditionLookup,
    RecordEditionLookupError, RecordEditionStatusJson, RecordFactJson,
    RecordFactTerminalPresentation, RecordJson, RecordJsonBase, RecordJsonContext, RecordJsonError,
    RecordJsonOptions, RecordPresentationJson, RecordRelationshipContextError,
    RecordRelationshipDirectionJson, RecordRelationshipLookupJson,
    RecordRelationshipProvenanceJson, RecordSectionJson, SpellAreaJson, SpellCastingJson,
    SpellClassificationJson, SpellContentJson, SpellDamageAlterationRuleJson,
    SpellDamageDiceRuleJson, SpellDamageJson, SpellDamagePatchJson, SpellDamagePatchMemberJson,
    SpellDamagePatchOperationJson, SpellDamagePatchSetJson, SpellDefenseJson, SpellDurationJson,
    SpellEphemeralEffectRuleJson, SpellFactJson, SpellFixedHeighteningJson, SpellFormJson,
    SpellFormLabelKind, SpellFormResultJson, SpellHeighteningDamageJson, SpellHeighteningJson,
    SpellHeighteningPatchJson, SpellItemAlterationRuleJson, SpellJson, SpellMemberProvenanceJson,
    SpellPatchJson, SpellProvenanceJson, SpellRangeJson, SpellResolvedDefinitionJson,
    SpellResolvedFieldJson, SpellRitualJson, SpellRollOptionRuleJson, SpellRuleDetailJson,
    SpellRuleJson, SpellRulePredicateJson, SpellRuleSuboptionJson, SpellSaveJson,
    SpellTargetingJson, SpellTextPatchMemberJson, SpellTextPatchOperationJson,
    SpellTextPatchSetJson, SpellUnsupportedFactJson, SpellUnsupportedValueJson,
    UnmigratedRegistryJson, VerifiedRecordEditionLookup, record_json, record_json_with_context,
};
pub use mechanics::{
    AbilityKind, CanonicalMechanicActivity, CanonicalMechanicsProjection, MechanicActivityFamily,
    MechanicBaseValue, MechanicFacets, MechanicFact, MechanicSourceFamily, MechanicStatistic,
    MechanicSurface, MechanicTarget, SaveKind, UnsupportedMechanic, UnsupportedMechanicValue,
    project_creature_mechanics,
};
pub use metrics::{
    MetricCapture, MetricDefinition, MetricDefinitionMatch, MetricDisplayLabel, MetricGroup,
    MetricKeyDefinition, MetricKeyPattern, MetricKeySegment, MetricLabelTemplate,
    MetricVariableVocabulary, all_definitions, definition_for, is_known_key, label_for_row,
    normalize_metric_key_segment,
};
pub use model::{
    ActivationTimeSourceField, ActivityRollAbility, ActorMechanics, AliasSource, AtlasRecord,
    AtlasRecordSet, DamageEffectKind, DefaultRetrievalVisibility, DurationTimeSourceField,
    FoundryDocumentMechanics, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
    ItemMechanics, MetricRow, MetricValue, NormalizedTime, RecordActivationTiming, RecordAlias,
    RecordClassification, RecordContent, RecordDurationTiming, RecordIdentity, RecordMechanics,
    RecordProvenance, RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming,
    RecordVariantMembership, RecordVisibility, RecordVisibilityReason, ReferenceEdge, RemasterLink,
    VariantSource,
};
pub use presentation::{
    PresentationBadge, PresentationBadgeKind, PresentationBlock, PresentationContent,
    PresentationContentBlock, PresentationFact, PresentationInline, PresentationListItem,
    PresentationRelationship, PresentationRelationshipKind, PresentationSection,
    PresentationSectionKind, PresentationTableRow, PresentationText, RecordPresentationDocument,
};
pub use presentation_content::{
    RecordSurfaceContentContext, RecordSurfaceContentIssue, RecordSurfaceContentIssueKind,
    RecordSurfaceContentProjection, project_presentation_content, project_record_surface_content,
    project_record_surface_content_with_context, render_presentation_content_plain_text,
};
pub use presentation_format::{CreatureFrequencyPeriod, format_creature_frequency, format_size};
pub use presentation_policy::{
    FactIssueKind, FactPresentationDisposition, FactPresentationRole, FactPresentationState,
    FactRequirement, SpellPresentationIssue, SpellPresentationIssueField,
    SpellPresentationIssuePlacement, classify_fact_presentation, classify_spell_fact,
    classify_spell_json_fact, merge_spell_presentation_issues, project_resolved_spell_form_label,
    project_spell_action_cost, project_spell_form_result_presentation_issues,
    project_spell_json_presentation_issues, project_spell_presentation_issues,
};
pub use presentation_recipe::{
    build_record_presentation_document, build_record_presentation_document_with_content_filter,
};
pub use reference_policy::{
    DEFAULT_EXCLUDED_SOURCE_KINDS, ReferenceEdgeFacts, ReferenceGraphMode, ReferenceGraphPolicy,
    reference_edge_matches_mode, reference_graph_policy,
};
pub use retrieval_policy::{
    ProductRetrievalPolicy, RecordRole, RetrievalDisposition, RetrievalPolicyDecision,
    RetrievalRationale,
};
pub use retrieved_record::RetrievedRecord;
pub use spell::{
    ConsumableSpellChild, ConsumableSpellLocation, ConsumableSpellSourceContext,
    InvalidSpellIdentity, ResolvedSpellForm, SPELL_RANGE_DERIVATION_RULE, SpellAreaPatch,
    SpellAreaType, SpellAreaValue, SpellCasting, SpellCastingPatch, SpellChildId,
    SpellClassification, SpellClassificationPatch, SpellDamage, SpellDamageAlterationRule,
    SpellDamageDiceRule, SpellDamagePatch, SpellDefensePatch, SpellDefenseValue, SpellDefinition,
    SpellDuration, SpellDurationPatch, SpellEphemeralEffectRule, SpellFact,
    SpellFixedHeighteningLayer, SpellFormContext, SpellFormField, SpellFormId,
    SpellFormPatchSource, SpellFormSelectionError, SpellFormUnavailable,
    SpellFormUnavailableReason, SpellHeightening, SpellHeighteningPatch, SpellHeighteningType,
    SpellIdentity, SpellIntervalHeightening, SpellItemAlterationRule, SpellKeyedPatch,
    SpellKeyedPatchMember, SpellKeyedPatchOperation, SpellLegacyAreaType, SpellNumericRange,
    SpellNumericRangeKind, SpellOrderedMember, SpellOverlay, SpellOverlayId,
    SpellOverlayOrderError, SpellOverlayOrderFailure, SpellOverlayType, SpellPatch,
    SpellProvenance, SpellRangeValue, SpellRecord, SpellResolvedField, SpellRitual,
    SpellRollOptionRule, SpellRule, SpellRuleElement, SpellRulePredicate, SpellRuleSuboption,
    SpellSave, SpellSavePatch, SpellSourceContext, SpellSourceId, SpellSourceValue,
    SpellStandaloneTarget, SpellStatistic, SpellTargeting, SpellTargetingPatch, SpellTextPatch,
    SpellTradition, SpellTrait, SpellUnsupportedPatchField, SpellUnsupportedRule,
    SpellUnsupportedRulePredicate, SpellUnsupportedSourceFact, SpellUnsupportedSourceField,
};
