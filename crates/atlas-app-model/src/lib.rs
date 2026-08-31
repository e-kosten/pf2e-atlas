#![deny(unsafe_code)]

mod encounter;
mod encounter_runtime;
mod error;
mod filter;
mod list;
mod readiness;
mod record;
mod result_window;
mod surface;

pub use encounter::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, CreateEncounterRequest, DeleteEncounterView,
    EncounterConditionApplicabilityView, EncounterConditionAutomationLevelView,
    EncounterConditionCatalogView, EncounterConditionCategoryView,
    EncounterConditionDefinitionView, EncounterCreateView, EncounterDetailView, EncounterIndexView,
    EncounterParticipantKindView, EncounterParticipantSideView, EncounterParticipantStatusView,
    EncounterParticipantVariantView, EncounterParticipantView, EncounterStatusView,
    EncounterSummaryView, EncounterUpdateView, ReorderEncounterParticipantPlacementView,
    ReorderEncounterParticipantRequest, SetEncounterTurnRequest,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest,
};
pub use encounter_runtime::*;
pub use error::{AppError, AppErrorCode, AppRecoverableAction};
pub use filter::{
    BasicSearchFilter, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest, FilterClause,
    FilterClauseOperator, FilterControlView, FilterDiscoveryContext, FilterEditorFieldView,
    FilterEditorGroupView, FilterEditorView, FilterFieldApplicability, FilterFieldPlacement,
    FilterRange, FilterValidationCode, FilterValidationMessage, FilterValidationResult,
    FilterValueListView, FilterValueOption, MetricComparison,
};
pub use list::{
    AddSavedListItemRequest, BatchAddSavedListItemsRequest, BatchSavedListItemInput,
    BatchSavedListItemMutationView, BatchSavedListItemOutcomeView, BatchSavedListItemResultView,
    CreateSavedListRequest, DeleteSavedListView, FilterSavedListRequest, ImportSavedListRequest,
    ImportSavedListView, RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView,
    SavedListExportDocumentView, SavedListExportItemView, SavedListExportListView,
    SavedListIndexView, SavedListItemMutationOutcomeView, SavedListItemMutationView,
    SavedListItemSnapshotView, SavedListItemStatusView, SavedListItemView, SavedListSummaryView,
    SavedListUpdateView, UpdateSavedListRequest,
};
pub use readiness::{AppReadinessStatus, AppReadinessView};
pub use record::{
    RecordDetailView, RecordResolutionAmbiguousView, RecordResolutionCandidateView,
    RecordSummaryView,
};
pub use result_window::{
    OpenResultWindowRequest, ReadResultWindowPageRequest, RecordListSortView, ResultMatchSummary,
    ResultWindowMode, ResultWindowModeSummary, ResultWindowPage, ResultWindowRow,
    SearchPageRequest, SearchPageView,
};
pub use surface::*;

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
        let encounter_runtime = actual
            .get("EncounterRuntimeView.ts")
            .expect("EncounterRuntimeView binding should exist");
        assert!(
            encounter_runtime.contains("level?: RuntimeNumberView"),
            "generated encounter runtime should expose the canonical outer level field"
        );
        assert!(
            !encounter_runtime.contains("adjusted_level"),
            "generated encounter runtime must not retain the superseded outer field"
        );
        let surface = actual
            .get("RecordSurfaceView.ts")
            .expect("RecordSurfaceView binding should exist");
        assert!(surface.contains("presentation: RecordSurfacePresentationView"));
        assert!(surface.contains("encounter?: EncounterRuntimeView"));
        assert!(!surface.contains(&["sec", "tions"].concat()));
        assert!(!surface.contains(&["section", "order"].join("_")));

        let creature = actual
            .get("CreatureSurfaceView.ts")
            .expect("CreatureSurfaceView binding should exist");
        for named_domain in [
            "vitals?: CreatureSurfaceVitalsView",
            "defenses?: CreatureSurfaceDefensesView",
            "saves?: CreatureSurfaceSavesView",
            "awareness?: CreatureSurfaceAwarenessView",
            "abilities?: CreatureSurfaceAbilitiesView",
            "skills?: Array<CreatureSurfaceSkillView>",
            "movement?: Array<CreatureSurfaceMovementView>",
            "spellcasting?: Array<CreatureSurfaceSpellcastingView>",
            "activities?: Array<CreatureSurfaceActivityView>",
            "content?: Array<CreatureSurfaceContentView>",
            "relationships?: Array<CreatureSurfaceRelationshipView>",
        ] {
            assert!(
                creature.contains(named_domain),
                "generated creature surface should expose `{named_domain}`"
            );
        }
        assert!(!creature.contains(&["sec", "tions"].concat()));

        fs::remove_dir_all(&temp_dir).expect("temporary binding directory should be removable");
    }

    #[test]
    fn unavailable_record_surface_serializes_without_generic_section_bag() {
        let surface = RecordSurfaceView {
            metadata: RecordSurfaceMetadataView {
                record_key: Some("actions:testAction1".to_string()),
                title: "Test Action 1".to_string(),
                kind: "rule".to_string(),
                kind_label: "Rule".to_string(),
                level: None,
                rarity: None,
                traits: Vec::new(),
                source: None,
            },
            profile: RecordSurfaceProfileView::RecordDetail,
            presentation: RecordSurfacePresentationView::Unavailable {
                unavailable: SurfaceUnavailableView {
                    reason: SurfaceUnavailableReasonView::RecordFamilyNotMigrated,
                    requested_kind: "rule".to_string(),
                    message: "Typed record presentation is unavailable for this record family."
                        .to_string(),
                },
            },
            encounter: None,
        };

        let serialized = serde_json::to_value(surface).expect("record surface should serialize");
        assert_eq!(serialized["profile"], "record_detail");
        assert_eq!(
            serialized["presentation"]["presentation_type"],
            "unavailable"
        );
        assert!(serialized.get("encounter").is_none());
        assert!(serialized.get(&["sec", "tions"].concat()).is_none());
        assert!(serialized.get(&["section", "order"].join("_")).is_none());
    }

    #[test]
    fn encounter_runtime_level_serializes_with_inner_adjustment_semantics() {
        let runtime = EncounterRuntimeView {
            level: Some(RuntimeNumberView {
                label: "Level".to_string(),
                base_value: 5,
                adjusted_value: 6,
                modifiers: Vec::new(),
                suppressed_modifiers: Vec::new(),
                provenance: RuntimeFactProvenanceView {
                    source: RuntimeFactSourceView::CanonicalRecord,
                    canonical_target: Some(RuntimeCanonicalTargetView::Level),
                },
            }),
            vitals: None,
            defenses: None,
            saves: None,
            awareness: None,
            abilities: None,
            skills: Vec::new(),
            movement: None,
            resources: Vec::new(),
            spellcasting: Vec::new(),
            activities: Vec::new(),
            action_budget: None,
            conditions: Vec::new(),
            automation_limitations: Vec::new(),
        };

        let serialized = serde_json::to_value(runtime).expect("runtime should serialize");
        let object = serialized
            .as_object()
            .expect("runtime should serialize as an object");
        assert!(object.contains_key("level"));
        assert!(!object.contains_key("adjusted_level"));
        assert_eq!(
            object.get("level"),
            Some(&serde_json::json!({
                "label": "Level",
                "base_value": 5,
                "adjusted_value": 6,
                "modifiers": [],
                "suppressed_modifiers": [],
                "provenance": {
                    "source": { "source_type": "canonical_record" },
                    "canonical_target": { "target_type": "level" }
                }
            }))
        );
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
        EncounterRuntimeView::export_all_to(path)
            .expect("EncounterRuntimeView bindings should export");
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
        EncounterParticipantVariantView::export_all_to(path)
            .expect("EncounterParticipantVariantView bindings should export");
        EncounterUpdateView::export_all_to(path)
            .expect("EncounterUpdateView bindings should export");
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
        RecordResolutionAmbiguousView::export_all_to(path)
            .expect("RecordResolutionAmbiguousView bindings should export");
        RecordSummaryView::export_all_to(path).expect("RecordSummaryView bindings should export");
        ResultWindowPage::export_all_to(path).expect("ResultWindowPage bindings should export");
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
        SavedListCreateView::export_all_to(path)
            .expect("SavedListCreateView bindings should export");
        SavedListDetailView::export_all_to(path)
            .expect("SavedListDetailView bindings should export");
        SavedListExportDocumentView::export_all_to(path)
            .expect("SavedListExportDocumentView bindings should export");
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
