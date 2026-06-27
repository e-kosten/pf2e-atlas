#![deny(unsafe_code)]

mod encounter;
mod error;
mod filter;
mod list;
mod readiness;
mod record;
mod result_window;
mod surface;

pub use encounter::{
    ActionBudgetView, ActivityRollSurfaceView, ActivityRollView,
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, CreateEncounterRequest, DamageEffectKindView,
    DamageExpressionView, DeleteEncounterView, EncounterConditionApplicabilityView,
    EncounterConditionAutomationLevelView, EncounterConditionCatalogView,
    EncounterConditionCategoryView, EncounterConditionDefinitionView, EncounterCreateView,
    EncounterDetailView, EncounterIndexView, EncounterParticipantConditionView,
    EncounterParticipantKindView, EncounterParticipantSideView, EncounterParticipantStatusView,
    EncounterParticipantVariantView, EncounterParticipantView, EncounterStatusView,
    EncounterSummaryView, EncounterUpdateView, MechanicActivityKindView, MechanicActivityModeView,
    MechanicActivityUsageView, MechanicActivityView, MovementSpeedView,
    ReorderEncounterParticipantPlacementView, ReorderEncounterParticipantRequest,
    RuntimeAdjustmentView, RuntimeCapabilityView, RuntimeCountSegmentView, RuntimeCountView,
    RuntimeEffectNoteView, SetEncounterTurnRequest, StatBlockView, StatModifierTypeView,
    StatModifierView, StatValueView, UnappliedEffectView,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest,
};
pub use error::{AppError, AppErrorCode, AppRecoverableAction};
pub use filter::{
    BasicSearchFilter, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest, FilterClause,
    FilterClauseOperator, FilterControlView, FilterDiscoveryContext, FilterEditorFieldView,
    FilterEditorGroupView, FilterEditorView, FilterFieldApplicability, FilterFieldPlacement,
    FilterRange, FilterValidationCode, FilterValidationMessage, FilterValidationResult,
    FilterValueListView, FilterValueOption, MetricComparison,
};
pub use list::{
    AddSavedListItemRequest, CreateSavedListRequest, DeleteSavedListView, FilterSavedListRequest,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView, SavedListIndexView,
    SavedListItemMutationOutcomeView, SavedListItemMutationView, SavedListItemSnapshotView,
    SavedListItemStatusView, SavedListItemView, SavedListSummaryView, SavedListUpdateView,
    UpdateSavedListRequest,
};
pub use readiness::{AppReadinessStatus, AppReadinessView};
pub use record::{
    RecordBadgeView, RecordDetailView, RecordResolutionAmbiguousView,
    RecordResolutionCandidateView, RecordSummaryView,
};
pub use result_window::{
    OpenResultWindowRequest, ReadResultWindowPageRequest, RecordListSortView, ResultMatchSummary,
    ResultWindowMode, ResultWindowModeSummary, ResultWindowPage, ResultWindowRow,
    SearchPageRequest, SearchPageView,
};
pub use surface::{
    RecordSurfaceHeaderView, RecordSurfaceProfileView, RecordSurfaceSectionKindView,
    RecordSurfaceSectionView, RecordSurfaceView, SurfaceActivityView, SurfaceAdjustmentView,
    SurfaceBadgeView, SurfaceNoteView, SurfaceScalarView, SurfaceValueDisplayView,
    SurfaceValueGroupView, SurfaceValueView,
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
        export_bindings_to(&bindings);
    }

    fn export_bindings_to(path: &Path) {
        fs::create_dir_all(path).expect("binding export directory should be creatable");
        AppError::export_all_to(path).expect("AppError bindings should export");
        AppReadinessView::export_all_to(path).expect("AppReadinessView bindings should export");
        ActionBudgetView::export_all_to(path).expect("ActionBudgetView bindings should export");
        ActivityRollSurfaceView::export_all_to(path)
            .expect("ActivityRollSurfaceView bindings should export");
        ActivityRollView::export_all_to(path).expect("ActivityRollView bindings should export");
        BasicSearchFilter::export_all_to(path).expect("BasicSearchFilter bindings should export");
        DiscoverFilterEditorRequest::export_all_to(path)
            .expect("DiscoverFilterEditorRequest bindings should export");
        DiscoverFilterValuesRequest::export_all_to(path)
            .expect("DiscoverFilterValuesRequest bindings should export");
        FilterEditorView::export_all_to(path).expect("FilterEditorView bindings should export");
        FilterDiscoveryContext::export_all_to(path)
            .expect("FilterDiscoveryContext bindings should export");
        FilterValidationResult::export_all_to(path)
            .expect("FilterValidationResult bindings should export");
        FilterValueListView::export_all_to(path)
            .expect("FilterValueListView bindings should export");
        FilterValueOption::export_all_to(path).expect("FilterValueOption bindings should export");
        AddEncounterManualParticipantRequest::export_all_to(path)
            .expect("AddEncounterManualParticipantRequest bindings should export");
        AddEncounterParticipantConditionRequest::export_all_to(path)
            .expect("AddEncounterParticipantConditionRequest bindings should export");
        AddEncounterRecordParticipantRequest::export_all_to(path)
            .expect("AddEncounterRecordParticipantRequest bindings should export");
        CreateEncounterRequest::export_all_to(path)
            .expect("CreateEncounterRequest bindings should export");
        DamageEffectKindView::export_all_to(path)
            .expect("DamageEffectKindView bindings should export");
        DamageExpressionView::export_all_to(path)
            .expect("DamageExpressionView bindings should export");
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
        EncounterDetailView::export_all_to(path)
            .expect("EncounterDetailView bindings should export");
        EncounterIndexView::export_all_to(path).expect("EncounterIndexView bindings should export");
        EncounterParticipantConditionView::export_all_to(path)
            .expect("EncounterParticipantConditionView bindings should export");
        EncounterParticipantVariantView::export_all_to(path)
            .expect("EncounterParticipantVariantView bindings should export");
        EncounterUpdateView::export_all_to(path)
            .expect("EncounterUpdateView bindings should export");
        MechanicActivityKindView::export_all_to(path)
            .expect("MechanicActivityKindView bindings should export");
        MechanicActivityUsageView::export_all_to(path)
            .expect("MechanicActivityUsageView bindings should export");
        MechanicActivityModeView::export_all_to(path)
            .expect("MechanicActivityModeView bindings should export");
        MechanicActivityView::export_all_to(path)
            .expect("MechanicActivityView bindings should export");
        MovementSpeedView::export_all_to(path).expect("MovementSpeedView bindings should export");
        RuntimeAdjustmentView::export_all_to(path)
            .expect("RuntimeAdjustmentView bindings should export");
        RuntimeCapabilityView::export_all_to(path)
            .expect("RuntimeCapabilityView bindings should export");
        RuntimeCountSegmentView::export_all_to(path)
            .expect("RuntimeCountSegmentView bindings should export");
        RuntimeCountView::export_all_to(path).expect("RuntimeCountView bindings should export");
        RuntimeEffectNoteView::export_all_to(path)
            .expect("RuntimeEffectNoteView bindings should export");
        StatBlockView::export_all_to(path).expect("StatBlockView bindings should export");
        StatModifierView::export_all_to(path).expect("StatModifierView bindings should export");
        StatModifierTypeView::export_all_to(path)
            .expect("StatModifierTypeView bindings should export");
        StatValueView::export_all_to(path).expect("StatValueView bindings should export");
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
        OpenResultWindowRequest::export_all_to(path)
            .expect("OpenResultWindowRequest bindings should export");
        ReadResultWindowPageRequest::export_all_to(path)
            .expect("ReadResultWindowPageRequest bindings should export");
        RecordDetailView::export_all_to(path).expect("RecordDetailView bindings should export");
        RecordSurfaceHeaderView::export_all_to(path)
            .expect("RecordSurfaceHeaderView bindings should export");
        RecordSurfaceProfileView::export_all_to(path)
            .expect("RecordSurfaceProfileView bindings should export");
        RecordSurfaceSectionKindView::export_all_to(path)
            .expect("RecordSurfaceSectionKindView bindings should export");
        RecordSurfaceSectionView::export_all_to(path)
            .expect("RecordSurfaceSectionView bindings should export");
        RecordSurfaceView::export_all_to(path).expect("RecordSurfaceView bindings should export");
        RecordResolutionAmbiguousView::export_all_to(path)
            .expect("RecordResolutionAmbiguousView bindings should export");
        RecordSummaryView::export_all_to(path).expect("RecordSummaryView bindings should export");
        ResultWindowPage::export_all_to(path).expect("ResultWindowPage bindings should export");
        SurfaceActivityView::export_all_to(path)
            .expect("SurfaceActivityView bindings should export");
        SurfaceAdjustmentView::export_all_to(path)
            .expect("SurfaceAdjustmentView bindings should export");
        SurfaceBadgeView::export_all_to(path).expect("SurfaceBadgeView bindings should export");
        SurfaceNoteView::export_all_to(path).expect("SurfaceNoteView bindings should export");
        SurfaceScalarView::export_all_to(path).expect("SurfaceScalarView bindings should export");
        SurfaceValueDisplayView::export_all_to(path)
            .expect("SurfaceValueDisplayView bindings should export");
        SurfaceValueGroupView::export_all_to(path)
            .expect("SurfaceValueGroupView bindings should export");
        SurfaceValueView::export_all_to(path).expect("SurfaceValueView bindings should export");
        AddSavedListItemRequest::export_all_to(path)
            .expect("AddSavedListItemRequest bindings should export");
        CreateSavedListRequest::export_all_to(path)
            .expect("CreateSavedListRequest bindings should export");
        DeleteSavedListView::export_all_to(path)
            .expect("DeleteSavedListView bindings should export");
        FilterSavedListRequest::export_all_to(path)
            .expect("FilterSavedListRequest bindings should export");
        RemoveSavedListItemRequest::export_all_to(path)
            .expect("RemoveSavedListItemRequest bindings should export");
        SavedListCreateView::export_all_to(path)
            .expect("SavedListCreateView bindings should export");
        SavedListDetailView::export_all_to(path)
            .expect("SavedListDetailView bindings should export");
        SavedListIndexView::export_all_to(path).expect("SavedListIndexView bindings should export");
        SavedListItemMutationView::export_all_to(path)
            .expect("SavedListItemMutationView bindings should export");
        SavedListUpdateView::export_all_to(path)
            .expect("SavedListUpdateView bindings should export");
        UpdateSavedListRequest::export_all_to(path)
            .expect("UpdateSavedListRequest bindings should export");
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
