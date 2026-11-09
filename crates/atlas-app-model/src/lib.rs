#![deny(unsafe_code)]
mod encounter_tracker;
pub use encounter_tracker::{
    EncounterManualTrackerAction, EncounterManualTrackerView, UpdateEncounterManualTrackerRequest,
};
mod encounter_recovery;
pub use encounter_recovery::{
    ApplyEncounterRecoveryRequest, EncounterRecoveryAction, EncounterRecoveryParticipantView,
    EncounterRecoveryPreviewView, OvernightHealingMode, OvernightParticipantInput,
    PreviewEncounterRecoveryRequest,
};
mod encounter_difficulty;
pub use encounter_difficulty::{
    EncounterDifficultyAddition, EncounterDifficultyBudgetView, EncounterDifficultyOpponentView,
    EncounterDifficultyReferenceInput, EncounterDifficultyReferenceView,
    EncounterDifficultyRuleView, EncounterDifficultyView, PreviewEncounterDifficultyRequest,
    SetEncounterDifficultyRequest,
};
mod encounter_ability_state;
pub use encounter_ability_state::{
    EncounterAbilityChoiceView, EncounterAbilityStateAction, EncounterAbilityStateKindView,
    EncounterAbilityStateView, UpdateEncounterAbilityStateRequest,
};
mod turn_resources;
pub use dice::{FlatCheckRollView, RollFlatCheckRequest};
pub use turn_resources::{
    InherentActionConditionView, InherentActionConditionsView, TurnActionCalculationView,
    TurnResourceAction, TurnResourceKind, TurnResourceResultView, TurnResourcesView,
    UpdateTurnResourcesRequest,
};
mod encounter_action;
pub use encounter_action::{EncounterStrikeActionContextView, EncounterTargetingRequirementsView};
pub use encounter_action::{EncounterStrikeContextRequest, EncounterStrikeContextView};
mod action_context;
pub use action_context::{
    EncounterActionCheckSourceView, EncounterActionCheckView, EncounterActionFactView,
    EncounterActionIssueKindView, EncounterActionIssueView, EncounterCastingRemindersRequest,
    EncounterCastingRemindersView,
};
mod persistent_damage;
pub use persistent_damage::{
    ApplyPersistentDamageRequest, PersistentDamageDraft, PersistentDamageGroupInput,
    PersistentDamageInput, PersistentDamageOutcomeView, PersistentDamageProtectionView,
    PersistentDamageResolutionView, PersistentDamageResultView, PersistentDraftContext,
    PersistentRecoveryDraft, PersistentRecoveryInput, PreparePersistentDamageRequest,
    PreparedPersistentDamageDraft, PreparedPersistentDamageView, PreparedPersistentRecoveryInput,
};
pub use persistent_damage::{
    PersistentDamageGroupView, PersistentDamageSourceView, PersistentEpisodeReviewView,
    PersistentRecoveryMode, PersistentRecoveryResolution, PersistentRecoveryResultView,
    PersistentReviewView, RecoverPersistentDamageRequest,
};
mod hp_events;
pub use hp_events::{
    DamageLethalityView, HpEventContextView, HpLifecycleRequiredFactView,
    HpLifecycleResolutionView, KnockoutOptionView, PendingHpLifecycleView,
    ResolveEncounterHpLifecycleRequest,
};
mod condition_recovery;
pub use condition_recovery::{
    ApplyConditionLifecycleRequest, CheckDegreeView, ConditionLifecycleActionView,
    ConditionLifecycleActionsView, ConditionLifecycleOptionView, ConditionLifecycleResultView,
    ConditionRecoveryOfferView, FrightenedRecoveryView, LifecycleCheckFactsView,
    LifecycleCheckInputView, LifecycleCheckResultView, PendingConditionRecoveryView,
};
mod condition_events;
pub use condition_events::{
    ConditionApplicationDecisionsView, ConditionEditorKindView, ConditionEventConsequenceView,
    ConditionEventIntentView, ConditionEventPreviewView, ConditionHpChangeView,
    ConditionOriginView, ConditionParametersInputView, ConditionParametersView,
    ConditionRemovalIntentView, ConditionSubjectView, DamageMultiplierView,
    EncounterDamageTypeView, LevelOverrideInputView, MortalityStateView,
    PendingConditionConsequenceView, PersistentDamagePrecedenceEntryView,
    PersistentDamagePrecedenceView, PersistentDamageSourceRefView, PersistentRecoveryView,
    RemoveEncounterParticipantConditionRequest, ResolveDrainedConsequenceRequest,
    SetEncounterLifeSettingsRequest, WakingRuleView,
};
mod definitions;
mod traits;
pub use definitions::{
    DefinitionDetailRequest, DefinitionDetailView, DefinitionNoticeView, DefinitionPublicationView,
    ReferenceContentView, ReferenceDestinationView, ReferenceFilterActionView, ReferenceHelpView,
    ReferenceValueView,
};
pub use traits::{
    TraitAssociationKind, TraitDetailRequest, TraitDetailView, TraitRecordView, TraitRecordsPage,
    TraitRecordsRequest,
};
mod table_roll;
pub use table_roll::{RollTableRequest, TableRollView};
mod strike_roll;
pub use strike_roll::{
    EncounterStrikeOptionsRequest, EncounterStrikeOptionsView, EncounterStrikeRollView,
    RollEncounterStrikeRequest, StrikeAttackOptionView, StrikeDamageRollView,
    StrikeDamageTotalView, StrikeRollAction, StrikeRollResult,
};

mod dice;
mod encounter;
pub use dice::{
    InitiativeOptionsView, InitiativeParticipantOptionsView, InitiativeParticipantRequest,
    InitiativeRollView, InitiativeStatisticView, ParticipantRollView, RollInitiativeRequest,
    RollMode,
};
mod embedded_reading;
mod error;
mod field_edit;
pub use field_edit::{FieldEdit, SavedEditPayload};
mod filter;
mod remaster;
pub use remaster::{
    RemasterExplanationLinkView, RemasterInlinePartView, RemasterRelationshipView,
    RemasterRelationshipsView, RemasterSourceContextView,
};
mod graph;
mod list;
mod party;
pub use party::{
    AddEncounterPartyRequest, CreatePartyRequest, DeletePartyRequest, PartyIndexView,
    PartyMemberInput, PartyMemberView, PartyMutationView, PartyView, UpdatePartyRequest,
};
mod prepared_content;
mod presentation;
mod reference_profiles;
pub use reference_profiles::{
    AncestryReferenceView, ArmyReferenceView, AttributeChoiceView, ClassReferenceView,
    DeityReferenceView, DeitySanctificationView, HeritageReferenceView,
    ObjectDefensesReferenceView, ReferenceGrantView, VehicleReferenceView,
};
mod readiness;
mod record;
mod search;
mod similarity;
pub use similarity::{
    SimilarityComponentView, SimilarityConfigurationView, SimilarityDiagnosticsView,
    SimilarityEvidenceView, SimilarityPairView, SimilarityPartScoreView, SimilarityPartView,
    SimilarityPartsRequest, SimilarityPartsView, SimilarityRoleScoreView,
};

pub use embedded_reading::{
    EmbeddedActivityAddress, EmbeddedReadingBatchView, EmbeddedReadingCursor,
    EmbeddedReadingEntryView, EmbeddedReadingParent, EmbeddedReadingRequest,
    EmbeddedReadingSelection,
};
pub use encounter::{
    ActionBudgetView, AddEncounterManualParticipantRequest,
    AddEncounterParticipantConditionRequest, AddEncounterRecordParticipantRequest,
    AdvanceEncounterTurnRequest, ApplyEncounterHpActionRequest, ApplyEncounterSpellActionRequest,
    ConditionActionView, ConditionDurationUnit, ConditionDurationView, ConditionExtraActionView,
    ConditionScopeView, ConditionTimingView, ConditionTurnEvent, CreateEncounterRequest,
    DeleteEncounterView, EncounterConditionApplicabilityView,
    EncounterConditionAutomationLevelView, EncounterConditionCatalogView,
    EncounterConditionCategoryView, EncounterConditionDefinitionView,
    EncounterConditionEligibilityView, EncounterCreateView, EncounterDetailView, EncounterHpAction,
    EncounterHpValueInput, EncounterIndexView, EncounterMutationView,
    EncounterNextTurnUnavailableReason, EncounterParticipantConditionView,
    EncounterParticipantKindView, EncounterParticipantSideView, EncounterParticipantStatusView,
    EncounterParticipantVariantView, EncounterParticipantView, EncounterReferenceRequest,
    EncounterSpellAction, EncounterSpellRankView, EncounterSpellResourceScope,
    EncounterSpellResourceView, EncounterSpellView, EncounterSpellcastingView, EncounterStatusView,
    EncounterSummaryView, EncounterTurnBoundaryView, EncounterTurnReminderView,
    EncounterTurnReviewView, ReorderEncounterParticipantRequest, RuntimeAdjustmentView,
    RuntimeCapabilityView, RuntimeCountSegmentView, RuntimeCountView, RuntimeEffectNoteView,
    SetEncounterTurnRequest, StatModifierTypeView, StatModifierView, UnappliedEffectView,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantEdit,
    UpdateEncounterRequest,
};
pub use error::{AppError, AppErrorCode, AppRecoverableAction};
pub use filter::{
    DiscoverFilterCountsRequest, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    FilterChoiceView, FilterCollectionControlView, FilterControlView, FilterCountsView,
    FilterDiscoveryContext, FilterEditorFieldView, FilterEditorGroupView, FilterEditorView,
    FilterFieldPlacement, FilterOperandView, FilterOperatorView, FilterUnitFieldView,
    FilterValidationResult, FilterValueListView, FilterValueOptionView, FilterValueOptionsView,
    FilterVocabularyScope, FilterVocabularyView, SenseQualifierView,
};
pub use graph::{
    GraphContextView, GraphContextViewRequest, GraphSectionView, RecordListView,
    RelatedCategoryView, RelatedRecordsRequest, RelatedRecordsView, SearchResultsView,
    SimilarRecordsView, VariantEvidenceView, VariantGroupView,
};
pub use list::{
    AddSavedListItemRequest, BatchAddSavedListItemsRequest, BatchSavedListItemInput,
    BatchSavedListItemMutationView, BatchSavedListItemOutcomeView, BatchSavedListItemResultView,
    CreateSavedListRequest, DeleteSavedListView, FilterSavedListRequest, ImportSavedListRequest,
    ImportSavedListView, ImportedListResolutionView, RemoveSavedListItemRequest,
    ReorderSavedListItemRequest, SavedListCreateView, SavedListDetailView,
    SavedListExportDocumentView, SavedListExportItemView, SavedListExportListView,
    SavedListIndexView, SavedListItemMutationOutcomeView, SavedListItemMutationView,
    SavedListItemNoteView, SavedListItemSnapshotView, SavedListItemStatusView, SavedListItemView,
    SavedListSummaryView, SavedListUpdateView, UpdateSavedListItemNoteRequest,
    UpdateSavedListRequest,
};
pub use prepared_content::{
    CheckDetailsView, ContentControlKindView, ContentControlView, ContentRollComponentResultView,
    ContentRollComponentView, ContentRollEncounterContext, ContentRollOffer,
    ContentRollPresentationView, ContentRollResultView, PreparedContentFieldView,
    PreparedFieldBodyView, PreparedInclusionView, PreparedReferenceView,
    PreparedSectionRenderingView, RollContentRequest, RollUnitView, RoutineHeaderView,
};
pub use presentation::{
    ActionCostView, ActionGlyphView, ActivityEffectReferenceView, ActivityFrequencyView,
    ActorAbilityGroupView, ActorActivityKindView, ActorActivityView, ActorDefenseNotesView,
    ActorModifierView, ActorMovementView, ActorPresentationView, ActorRuntimeView, ActorSavesView,
    ActorSenseView, ActorSkillView, AmmunitionReferenceView, ArmorReferenceView,
    CastingEntryContextView, ConditionalSkillView, ConsumableDamageView,
    ConsumablePresentationView, ContainerReferenceView, DamageComponentView, EffectDurationView,
    EmbeddedRecordLinkView, EncounterContextualArmorClassView, EncounterSenseView, FactView,
    HazardPresentationView, HeighteningDamageView, IwrEntryView, JournalPagePresentationView,
    JournalPresentationView, KitEntryReferenceView, KitReferenceView, NumberAdjustmentView,
    NumberFactView, PhysicalBaseTypeView, PhysicalDetailsView, PhysicalMaterialReferenceView,
    PhysicalPresentationView, PhysicalPriceView, RecordBodyView, RecordGrantLinkView,
    RecordGrantsView, RecordPresentationView, RitualCastingView, RollTablePresentationView,
    RollTableRowView, ShieldReferenceView, SpellAreaView, SpellChangeView, SpellDefenseView,
    SpellEffectChoiceView, SpellEffectFactView, SpellEffectPreviewView, SpellEffectStrikeView,
    SpellFixedLevelView, SpellFormView, SpellHeighteningChangeView, SpellHeighteningView,
    SpellPresentationView, SpellRankChangeView, SpellRankFactView, StrikeRangeView,
    TableResultPresentationView, WeaponAmmunitionReferenceView, WeaponDamageReferenceView,
    WeaponMeleeReferenceView, WeaponPersistentReferenceView, WeaponReferenceView,
    WeaponRuneReferenceView,
};
pub use readiness::{AppReadinessStatus, AppReadinessView, SearchProfileReadinessView};
pub use record::{
    ContentLocationView, EffectReferenceLabelView, RecordAddress, RecordBadgeView,
    RecordDetailRequest, RecordDetailView, RecordEditionView, RecordNavigationView,
    RecordRefResolutionView, RecordReferenceDestinationView, RecordRelationshipView,
    RecordResolutionAmbiguousView, RecordResolutionCandidateView, RecordSelection,
    RecordSummaryView,
};
pub use search::{
    ConnectionScopeUnavailableReason, ConnectionScopeView, MatchPreviewView,
    RelationshipClausePresentationView, RelationshipClauseSummaryView, RelationshipEvidenceView,
    RelationshipOccurrencesPage, RelationshipOccurrencesRequest, RetrievalModeView,
    SearchConnectionScope, SearchCoverageView, SearchCriteria, SearchLaneView,
    SearchMatchSectionView, SearchModeSummary, SearchPageRequest, SearchPageView, SearchResultRow,
    SearchResultsPage, SearchResultsRequest, SearchWitnessView, StaticSearchContext,
};
