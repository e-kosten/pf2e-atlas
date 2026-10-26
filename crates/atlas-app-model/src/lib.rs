#![deny(unsafe_code)]
mod encounter_difficulty;
pub use encounter_difficulty::{
    EncounterDifficultyAddition, EncounterDifficultyBudgetView, EncounterDifficultyOpponentView,
    EncounterDifficultyReferenceView, EncounterDifficultyRuleView, EncounterDifficultyView,
    PreviewEncounterDifficultyRequest, SetEncounterDifficultyRequest,
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
    ApplyPersistentDamageRequest, PersistentDamageGroupInput, PersistentDamageInput,
    PersistentDamageOutcomeView, PersistentDamageProtectionView, PersistentDamageResolutionView,
    PersistentDamageResultView, PersistentRecoveryInput, PreparePersistentDamageRequest,
    PreparedPersistentDamageGroupView, PreparedPersistentDamageView,
    PreparedPersistentRecoveryInput,
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
mod traits;
pub use traits::{
    TraitAssociationKind, TraitContentView, TraitDetailRequest, TraitDetailView, TraitRecordView,
    TraitRecordsPage, TraitRecordsRequest,
};
mod table_roll;
pub use table_roll::{RollTableRequest, TableRollView};
mod strike_roll;
pub use strike_roll::{
    EncounterStrikeOptionsRequest, EncounterStrikeOptionsView, EncounterStrikeRollView,
    RollEncounterStrikeRequest, StrikeAttackOptionView, StrikeDamageRollView,
    StrikeDamageTotalView, StrikeRollKind,
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
mod filter;
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

pub use embedded_reading::{
    EmbeddedReadingBatchView, EmbeddedReadingCursor, EmbeddedReadingEntryView,
    EmbeddedReadingRequest, EmbeddedReadingSelection,
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
    EncounterNextTurnUnavailableReason, EncounterOutgoingTurnView,
    EncounterParticipantConditionView, EncounterParticipantKindView, EncounterParticipantSideView,
    EncounterParticipantStatusView, EncounterParticipantVariantView, EncounterParticipantView,
    EncounterReferenceRequest, EncounterSpellAction, EncounterSpellRankView,
    EncounterSpellResourceScope, EncounterSpellResourceView, EncounterSpellView,
    EncounterSpellcastingView, EncounterStatusView, EncounterSummaryView,
    EncounterTurnReminderView, EncounterTurnReviewView, ReorderEncounterParticipantPlacementView,
    ReorderEncounterParticipantRequest, RuntimeAdjustmentView, RuntimeCapabilityView,
    RuntimeCountSegmentView, RuntimeCountView, RuntimeEffectNoteView, SetEncounterTurnRequest,
    StatModifierTypeView, StatModifierView, UnappliedEffectView,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest,
};
pub use error::{AppError, AppErrorCode, AppRecoverableAction};
pub use filter::{
    DiscoverFilterCountsRequest, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    FilterChoiceView, FilterCollectionControlView, FilterControlView, FilterCountsView,
    FilterDiscoveryContext, FilterEditorFieldView, FilterEditorGroupView, FilterEditorView,
    FilterFieldPlacement, FilterOperandView, FilterOperatorView, FilterUnitFieldView,
    FilterValidationResult, FilterValueListView, FilterVocabularyScope,
};
pub use graph::{
    GraphContextView, GraphContextViewRequest, GraphSectionView, RecordListView,
    RelatedCategoryView, RelatedRecordsRequest, RelatedRecordsView, RemasterLinkView,
    RemasterLinksView, SearchResultsView, SimilarRecordsView, VariantEvidenceView,
    VariantGroupView,
};
pub use list::{
    AddSavedListItemRequest, BatchAddSavedListItemsRequest, BatchSavedListItemInput,
    BatchSavedListItemMutationView, BatchSavedListItemOutcomeView, BatchSavedListItemResultView,
    CreateSavedListRequest, DeleteSavedListView, FilterSavedListRequest, ImportSavedListRequest,
    ImportSavedListView, RemoveSavedListItemRequest, ReorderSavedListItemPlacementView,
    ReorderSavedListItemRequest, SavedListCreateView, SavedListDetailView,
    SavedListExportDocumentView, SavedListExportItemView, SavedListExportListView,
    SavedListIndexView, SavedListItemMutationOutcomeView, SavedListItemMutationView,
    SavedListItemNoteView, SavedListItemSnapshotView, SavedListItemStatusView, SavedListItemView,
    SavedListSummaryView, SavedListUpdateView, UpdateSavedListItemNoteRequest,
    UpdateSavedListRequest,
};
pub use prepared_content::{
    ContentControlKindView, ContentControlView, PreparedContentFieldView, PreparedFieldBodyView,
    PreparedInclusionView, PreparedReferenceView,
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
    NumberFactView, PhysicalDetailsView, PhysicalMaterialReferenceView, PhysicalPresentationView,
    PhysicalPriceView, RecordBodyView, RecordPresentationView, RitualCastingView,
    RollTablePresentationView, RollTableRowView, ShieldReferenceView, SpellAreaView,
    SpellChangeView, SpellDefenseView, SpellEffectChoiceView, SpellEffectFactView,
    SpellEffectPreviewView, SpellEffectStrikeView, SpellFixedLevelView, SpellFormView,
    SpellHeighteningChangeView, SpellHeighteningView, SpellPresentationView, SpellRankChangeView,
    SpellRankFactView, StrikeRangeView, TableResultPresentationView, WeaponAmmunitionReferenceView,
    WeaponDamageReferenceView, WeaponMeleeReferenceView, WeaponPersistentReferenceView,
    WeaponReferenceView, WeaponRuneReferenceView,
};
pub use readiness::{AppReadinessStatus, AppReadinessView, SearchProfileReadinessView};
pub use record::{
    ContentLocationView, EffectReferenceLabelView, RecordBadgeView, RecordDetailRequest,
    RecordDetailView, RecordNavigationView, RecordRefResolutionView, RecordRelationshipView,
    RecordResolutionAmbiguousView, RecordResolutionCandidateView, RecordSummaryView,
};
pub use search::{
    RelationshipClausePresentationView, RelationshipClauseSummaryView, RelationshipEvidenceView,
    RelationshipOccurrencesPage, RelationshipOccurrencesRequest, RetrievalModeView,
    SearchCoverageView, SearchCriteria, SearchLaneView, SearchModeSummary, SearchPageRequest,
    SearchPageView, SearchResultRow, SearchResultsPage, SearchResultsRequest, SearchWitnessView,
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use ts_rs::TS;

    use super::*;

    #[test]
    fn typescript_bindings_are_fresh() {
        let temp_dir = fresh_temp_dir("atlas-app-model-bindings-check");
        export_bindings_to(&temp_dir);

        let expected = read_binding_dir(&binding_dir());
        let actual = read_binding_dir(&temp_dir);
        assert_eq!(
            expected, actual,
            "generated TypeScript bindings are stale; run `cargo test -p atlas-app-model export_typescript_bindings -- --ignored`"
        );

        fs::remove_dir_all(&temp_dir).expect("temporary binding directory should be removable");
    }

    #[test]
    #[ignore = "regenerates checked-in TypeScript bindings"]
    fn export_typescript_bindings() {
        let bindings = binding_dir();
        fs::create_dir_all(&bindings).expect("binding directory should be creatable");
        for entry in fs::read_dir(&bindings).expect("bindings") {
            let path = entry.expect("binding").path();
            if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                fs::remove_file(path).expect("remove retired binding");
            }
        }
        export_bindings_to(&bindings);
    }

    fn export_bindings_to(path: &Path) {
        UpdateTurnResourcesRequest::export_all_to(path).expect("turn resource request");
        TurnResourcesView::export_all_to(path).expect("turn resources");
        EncounterStrikeContextRequest::export_all_to(path).expect("strike attempt preview request");
        FlatCheckRollView::export_all_to(path).expect("flat check result");
        RollFlatCheckRequest::export_all_to(path).expect("flat check request");
        EncounterStrikeContextView::export_all_to(path).expect("strike attempt preview");
        EncounterStrikeActionContextView::export_all_to(path).expect("action preview bindings");
        ConditionRecoveryOfferView::export_all_to(path).expect("recovery offer bindings");
        ConditionLifecycleActionsView::export_all_to(path).expect("lifecycle actions bindings");
        ApplyConditionLifecycleRequest::export_all_to(path).expect("lifecycle request bindings");
        ConditionLifecycleResultView::export_all_to(path).expect("lifecycle result bindings");
        PendingConditionRecoveryView::export_all_to(path).expect("pending recovery bindings");
        ResolveEncounterHpLifecycleRequest::export_all_to(path)
            .expect("HP lifecycle resolution bindings");
        PendingHpLifecycleView::export_all_to(path).expect("pending HP lifecycle bindings");
        ConditionEventPreviewView::export_all_to(path).expect("condition preview bindings");
        RemoveEncounterParticipantConditionRequest::export_all_to(path)
            .expect("condition removal bindings");
        ResolveDrainedConsequenceRequest::export_all_to(path)
            .expect("condition consequence resolution bindings");
        PendingConditionConsequenceView::export_all_to(path).expect("pending consequence bindings");
        SetEncounterLifeSettingsRequest::export_all_to(path).expect("life settings bindings");
        PartyIndexView::export_all_to(path).expect("party bindings");
        CreatePartyRequest::export_all_to(path).expect("party create bindings");
        UpdatePartyRequest::export_all_to(path).expect("party update bindings");
        DeletePartyRequest::export_all_to(path).expect("party delete bindings");
        PartyMutationView::export_all_to(path).expect("party mutation bindings");
        AddEncounterPartyRequest::export_all_to(path).expect("party addition bindings");
        InitiativeOptionsView::export_all_to(path).expect("initiative options bindings");
        RollInitiativeRequest::export_all_to(path).expect("initiative request bindings");
        InitiativeRollView::export_all_to(path).expect("initiative receipt bindings");
        RollTableRequest::export_all_to(path).expect("table roll request bindings");
        TableRollView::export_all_to(path).expect("table roll result bindings");
        EncounterStrikeOptionsRequest::export_all_to(path).expect("strike options request");
        EncounterStrikeOptionsView::export_all_to(path).expect("strike options view");
        RollEncounterStrikeRequest::export_all_to(path).expect("strike request");
        EncounterStrikeRollView::export_all_to(path).expect("strike result");
        fs::create_dir_all(path).expect("binding export directory should be creatable");
        PreparePersistentDamageRequest::export_all_to(path)
            .expect("persistent preparation request");
        PreparedPersistentDamageView::export_all_to(path).expect("persistent preparation result");
        ApplyPersistentDamageRequest::export_all_to(path).expect("persistent damage request");
        PersistentDamageResultView::export_all_to(path).expect("persistent damage result");
        RecoverPersistentDamageRequest::export_all_to(path).expect("persistent recovery request");
        PersistentRecoveryResultView::export_all_to(path).expect("persistent recovery result");
        PersistentReviewView::export_all_to(path).expect("persistent review");
        EncounterTurnReviewView::export_all_to(path).expect("turn review bindings");
        AdvanceEncounterTurnRequest::export_all_to(path).expect("turn advance bindings");
        ApplyEncounterSpellActionRequest::export_all_to(path).expect("spell action bindings");
        EncounterCastingRemindersRequest::export_all_to(path).expect("casting preview bindings");
        EncounterCastingRemindersView::export_all_to(path)
            .expect("casting preview response bindings");
        ApplyEncounterHpActionRequest::export_all_to(path).expect("HP action bindings");
        EmbeddedReadingRequest::export_all_to(path).expect("owned reading request bindings");
        EmbeddedReadingBatchView::export_all_to(path).expect("owned reading response bindings");
        RelatedRecordsView::export_all_to(path).expect("related bindings");
        RelatedRecordsRequest::export_all_to(path).expect("related request bindings");
        RecordListView::export_all_to(path).expect("graph bindings");
        RecordRefResolutionView::export_all_to(path).expect("resolution bindings");
        GraphContextViewRequest::export_all_to(path).expect("graph request bindings");
        GraphContextView::export_all_to(path).expect("graph bindings");
        RemasterLinksView::export_all_to(path).expect("remaster bindings");
        VariantGroupView::export_all_to(path).expect("variants bindings");
        SimilarRecordsView::export_all_to(path).expect("similar bindings");
        AppError::export_all_to(path).expect("AppError bindings should export");
        AppReadinessView::export_all_to(path).expect("AppReadinessView bindings should export");
        ActionBudgetView::export_all_to(path).expect("ActionBudgetView bindings should export");

        DiscoverFilterEditorRequest::export_all_to(path).expect("editor bindings");
        DiscoverFilterValuesRequest::export_all_to(path).expect("values bindings");
        DiscoverFilterCountsRequest::export_all_to(path).expect("counts bindings");
        FilterEditorView::export_all_to(path).expect("editor bindings");
        FilterValidationResult::export_all_to(path).expect("validation bindings");
        FilterValueListView::export_all_to(path).expect("values bindings");
        FilterCountsView::export_all_to(path).expect("counts bindings");
        RecordDetailRequest::export_all_to(path).expect("detail request bindings");

        AddEncounterManualParticipantRequest::export_all_to(path)
            .expect("AddEncounterManualParticipantRequest bindings should export");
        AddEncounterParticipantConditionRequest::export_all_to(path)
            .expect("AddEncounterParticipantConditionRequest bindings should export");
        AddEncounterRecordParticipantRequest::export_all_to(path)
            .expect("AddEncounterRecordParticipantRequest bindings should export");
        CreateEncounterRequest::export_all_to(path)
            .expect("CreateEncounterRequest bindings should export");

        DeleteEncounterView::export_all_to(path)
            .expect("DeleteEncounterView bindings should export");
        EncounterCreateView::export_all_to(path)
            .expect("EncounterCreateView bindings should export");
        EncounterConditionCatalogView::export_all_to(path)
            .expect("EncounterConditionCatalogView bindings should export");
        EncounterConditionAutomationLevelView::export_all_to(path)
            .expect("EncounterConditionAutomationLevelView bindings should export");
        EncounterConditionApplicabilityView::export_all_to(path)
            .expect("EncounterConditionApplicabilityView bindings should export");
        EncounterConditionCategoryView::export_all_to(path)
            .expect("EncounterConditionCategoryView bindings should export");
        EncounterConditionDefinitionView::export_all_to(path)
            .expect("EncounterConditionDefinitionView bindings should export");
        PreviewEncounterDifficultyRequest::export_all_to(path)
            .expect("difficulty preview bindings should export");
        SetEncounterDifficultyRequest::export_all_to(path)
            .expect("difficulty request bindings should export");
        EncounterDetailView::export_all_to(path)
            .expect("EncounterDetailView bindings should export");
        EncounterReferenceRequest::export_all_to(path)
            .expect("EncounterReferenceRequest bindings should export");
        EncounterIndexView::export_all_to(path).expect("EncounterIndexView bindings should export");
        EncounterParticipantConditionView::export_all_to(path)
            .expect("EncounterParticipantConditionView bindings should export");
        EncounterParticipantVariantView::export_all_to(path)
            .expect("EncounterParticipantVariantView bindings should export");
        EncounterMutationView::export_all_to(path)
            .expect("EncounterMutationView bindings should export");

        RuntimeAdjustmentView::export_all_to(path)
            .expect("RuntimeAdjustmentView bindings should export");
        RuntimeCapabilityView::export_all_to(path)
            .expect("RuntimeCapabilityView bindings should export");
        RuntimeCountSegmentView::export_all_to(path)
            .expect("RuntimeCountSegmentView bindings should export");
        RuntimeCountView::export_all_to(path).expect("RuntimeCountView bindings should export");
        RuntimeEffectNoteView::export_all_to(path)
            .expect("RuntimeEffectNoteView bindings should export");
        StatModifierView::export_all_to(path).expect("StatModifierView bindings should export");
        StatModifierTypeView::export_all_to(path)
            .expect("StatModifierTypeView bindings should export");

        UnappliedEffectView::export_all_to(path)
            .expect("UnappliedEffectView bindings should export");
        ReorderEncounterParticipantRequest::export_all_to(path)
            .expect("ReorderEncounterParticipantRequest bindings should export");
        SetEncounterTurnRequest::export_all_to(path)
            .expect("SetEncounterTurnRequest bindings should export");
        UpdateEncounterParticipantConditionRequest::export_all_to(path)
            .expect("UpdateEncounterParticipantConditionRequest bindings should export");
        UpdateEncounterRequest::export_all_to(path)
            .expect("UpdateEncounterRequest bindings should export");
        UpdateEncounterParticipantRequest::export_all_to(path)
            .expect("UpdateEncounterParticipantRequest bindings should export");
        SearchResultsRequest::export_all_to(path)
            .expect("SearchResultsRequest bindings should export");
        TraitDetailRequest::export_all_to(path).expect("trait request bindings");
        TraitDetailView::export_all_to(path).expect("trait detail bindings");
        TraitRecordsRequest::export_all_to(path).expect("trait associations request bindings");
        TraitRecordsPage::export_all_to(path).expect("trait associations bindings");
        RecordDetailView::export_all_to(path).expect("RecordDetailView bindings should export");

        RecordResolutionAmbiguousView::export_all_to(path)
            .expect("RecordResolutionAmbiguousView bindings should export");
        RecordSummaryView::export_all_to(path).expect("RecordSummaryView bindings should export");
        SearchResultsPage::export_all_to(path).expect("SearchResultsPage bindings should export");
        RelationshipOccurrencesRequest::export_all_to(path)
            .expect("relationship request should export");
        RelationshipOccurrencesPage::export_all_to(path).expect("relationship page should export");

        AddSavedListItemRequest::export_all_to(path)
            .expect("AddSavedListItemRequest bindings should export");
        BatchAddSavedListItemsRequest::export_all_to(path)
            .expect("BatchAddSavedListItemsRequest bindings should export");
        BatchSavedListItemMutationView::export_all_to(path)
            .expect("BatchSavedListItemMutationView bindings should export");
        CreateSavedListRequest::export_all_to(path)
            .expect("CreateSavedListRequest bindings should export");
        DeleteSavedListView::export_all_to(path)
            .expect("DeleteSavedListView bindings should export");
        FilterSavedListRequest::export_all_to(path)
            .expect("FilterSavedListRequest bindings should export");
        ImportSavedListRequest::export_all_to(path)
            .expect("ImportSavedListRequest bindings should export");
        ImportSavedListView::export_all_to(path)
            .expect("ImportSavedListView bindings should export");
        RemoveSavedListItemRequest::export_all_to(path)
            .expect("RemoveSavedListItemRequest bindings should export");
        ReorderSavedListItemRequest::export_all_to(path)
            .expect("ReorderSavedListItemRequest bindings should export");
        SavedListCreateView::export_all_to(path)
            .expect("SavedListCreateView bindings should export");
        SavedListDetailView::export_all_to(path)
            .expect("SavedListDetailView bindings should export");
        SavedListExportDocumentView::export_all_to(path)
            .expect("SavedListExportDocumentView bindings should export");
        SavedListIndexView::export_all_to(path).expect("SavedListIndexView bindings should export");
        SavedListItemMutationView::export_all_to(path)
            .expect("SavedListItemMutationView bindings should export");
        SavedListItemNoteView::export_all_to(path).expect("list note bindings");
        UpdateSavedListItemNoteRequest::export_all_to(path).expect("list note request bindings");
        SavedListUpdateView::export_all_to(path)
            .expect("SavedListUpdateView bindings should export");
        UpdateSavedListRequest::export_all_to(path)
            .expect("UpdateSavedListRequest bindings should export");

        // ts-rs can emit trailing spaces before field documentation comments.
        // Normalize at export so freshness checks and Git's whitespace gate agree.
        for (name, contents) in read_binding_dir(path) {
            let normalized = contents
                .lines()
                .map(str::trim_end)
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(path.join(name), format!("{normalized}\n"))
                .expect("normalized binding should be writable");
        }
    }

    fn read_binding_dir(path: &Path) -> BTreeMap<String, String> {
        let mut files = BTreeMap::new();
        for entry in fs::read_dir(path).expect("binding directory should be readable") {
            let entry = entry.expect("binding directory entry should be readable");
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("ts") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .expect("binding file name should be utf-8")
                .to_string();
            let contents = fs::read_to_string(&path).expect("binding file should be readable");
            files.insert(name, contents);
        }
        files
    }

    fn binding_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bindings")
    }

    fn fresh_temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path).expect("stale temporary binding directory should be removed");
        }
        path
    }
}
