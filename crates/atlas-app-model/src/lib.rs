#![deny(unsafe_code)]

mod encounter;
mod encounter_runtime;
mod error;
mod filter;
mod hazard_surface;
mod json_integer;
mod list;
mod readiness;
mod record;
mod result_window;
mod spell_surface;
mod surface;

pub use encounter::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, CreateEncounterRequest, DeleteEncounterView,
    EncounterConditionApplicabilityView, EncounterConditionAutomationLevelView,
    EncounterConditionCatalogView, EncounterConditionCategoryView,
    EncounterConditionDefinitionView, EncounterCreateView, EncounterDetailView, EncounterIndexView,
    EncounterParticipantKindView, EncounterParticipantPreservedDomainView,
    EncounterParticipantResetAvailabilityView, EncounterParticipantResetConfirmationView,
    EncounterParticipantResetDomainView, EncounterParticipantResetResultView,
    EncounterParticipantResetUnavailableReasonView, EncounterParticipantSideView,
    EncounterParticipantStatusView, EncounterParticipantVariantView, EncounterParticipantView,
    EncounterSpellCastOperationView, EncounterSpellCastRequest, EncounterSpellCastResultView,
    EncounterStatusView, EncounterSummaryView, EncounterUpdateView,
    ReorderEncounterParticipantPlacementView, ReorderEncounterParticipantRequest,
    ResetEncounterParticipantRequest, SetEncounterTurnRequest,
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
    FilterValueListView, FilterValueOption, MetricComparison, ReferenceSearchDirection,
    RelationshipConstraint,
};
pub use hazard_surface::*;
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
    RecordDetailRequest, RecordDetailView, RecordResolutionAmbiguousView,
    RecordResolutionCandidateView, RecordSummaryView,
};
pub use result_window::{
    OpenResultWindowRequest, ReadResultWindowPageRequest, RecordListSortView, ResultMatchSummary,
    ResultWindowMode, ResultWindowModeSummary, ResultWindowPage, ResultWindowRow,
    SearchPageRequest, SearchPageView,
};
pub use spell_surface::*;
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
        assert!(encounter_runtime.contains("standalone_spells?: Array<EncounterRuntimeSpellView>"));
        let runtime_activity = actual
            .get("EncounterRuntimeActivityView.ts")
            .expect("runtime activity binding should exist");
        assert!(runtime_activity.contains("content?: Array<CreatureSurfaceContentView>"));
        assert!(runtime_activity.contains("traits?: Array<string>"));
        let runtime_spellcasting = actual
            .get("EncounterRuntimeSpellcastingView.ts")
            .expect("runtime spellcasting binding should exist");
        for field in [
            "authored_order: number",
            "preparation?: string",
            "tradition?: string",
            "spells?: Array<EncounterRuntimeSpellView>",
        ] {
            assert!(runtime_spellcasting.contains(field), "missing `{field}`");
        }
        let runtime_spell = actual
            .get("EncounterRuntimeSpellView.ts")
            .expect("runtime spell binding should exist");
        for field in [
            "occurrence_id: string",
            "authored_order: number",
            "target_record_key?: string",
            "content?: Array<CreatureSurfaceContentView>",
            "activity?: EncounterRuntimeActivityView",
            "cast: EncounterSpellCastAvailabilityView",
        ] {
            assert!(runtime_spell.contains(field), "missing `{field}`");
        }
        let spell_cast_request = actual
            .get("EncounterSpellCastRequest.ts")
            .expect("spell-cast request binding should exist");
        for field in [
            "spell_occurrence_id: string",
            "spend_target: EncounterSpellSpendTargetView",
            "operation: EncounterSpellCastOperationView",
        ] {
            assert!(spell_cast_request.contains(field), "missing `{field}`");
        }
        let spell_cast_state = actual
            .get("EncounterSpellCastStateView.ts")
            .expect("spell-cast state binding should exist");
        for field in [
            "maximum: number",
            "initial_remaining: number",
            "remaining: number",
        ] {
            assert!(spell_cast_state.contains(field), "missing `{field}`");
        }
        let runtime_spell_slot = actual
            .get("EncounterRuntimeSpellSlotView.ts")
            .expect("runtime spell-slot binding should exist");
        assert!(runtime_spell_slot.contains("current?: RuntimeCountView"));
        let surface = actual
            .get("RecordSurfaceView.ts")
            .expect("RecordSurfaceView binding should exist");
        assert!(surface.contains("presentation: RecordSurfacePresentationView"));
        assert!(surface.contains("encounter?: EncounterRuntimeView"));
        assert!(!surface.contains(&["sec", "tions"].concat()));
        assert!(!surface.contains(&["section", "order"].join("_")));
        let presentation = actual
            .get("RecordSurfacePresentationView.ts")
            .expect("record presentation union should exist");
        assert!(presentation.contains("\"presentation_type\": \"hazard\""));
        assert!(presentation.contains("body: HazardSurfaceView"));
        let hazard = actual
            .get("HazardSurfaceView.ts")
            .expect("hazard surface binding should exist");
        for field in [
            "complexity?: HazardSurfaceComplexityView",
            "detection?: HazardSurfaceDetectionView",
            "defenses?: HazardSurfaceDefensesView",
            "lifecycle?: HazardSurfaceLifecycleView",
            "activities?: Array<HazardSurfaceActivityView>",
            "content?: Array<CreatureSurfaceContentView>",
            "relationships?: Array<HazardSurfaceRelationshipView>",
            "unavailable_fields?: Array<HazardSurfaceUnavailableView>",
            "provenance: HazardSurfaceProvenanceView",
        ] {
            assert!(hazard.contains(field), "missing hazard binding `{field}`");
        }
        let hazard_activity = actual
            .get("HazardSurfaceActivityView.ts")
            .expect("hazard activity binding should exist");
        for field in [
            "attack_mode?: HazardSurfaceAttackModeView",
            "action_cost?: CreatureSurfaceActionCostView",
        ] {
            assert!(
                hazard_activity.contains(field),
                "missing hazard activity binding `{field}`"
            );
        }
        let hazard_provenance = actual
            .get("HazardSurfaceProvenanceView.ts")
            .expect("hazard provenance binding should exist");
        assert!(
            hazard_provenance
                .contains("source_metadata: Array<HazardSurfaceSourceMetadataFactView>")
        );
        let hazard_source_metadata = actual
            .get("HazardSurfaceSourceMetadataFactView.ts")
            .expect("hazard source-metadata binding should exist");
        assert!(hazard_source_metadata.contains("HazardSurfaceSourceFactView<number>"));
        assert!(!hazard_source_metadata.contains("bigint"));
        let hazard_runtime = actual
            .get("EncounterRuntimeHazardView.ts")
            .expect("hazard runtime binding should exist");
        for field in [
            "state: EncounterRuntimeHazardStateView",
            "detection_dc?: RuntimeNumberView",
            "broken_threshold?: RuntimeNumberView",
            "initiative_suggestion?: EncounterRuntimeHazardInitiativeSuggestionView",
        ] {
            assert!(
                hazard_runtime.contains(field),
                "missing runtime binding `{field}`"
            );
        }
        assert!(presentation.contains("\"presentation_type\": \"spell\""));
        assert!(presentation.contains("body: SpellSurfaceView"));
        let spell_surface = actual
            .get("SpellSurfaceView.ts")
            .expect("SpellSurfaceView binding should exist");
        for field in [
            "family: SpellFamilyView",
            "forms: Array<SpellFormView>",
            "effective_form: SpellEffectiveFormView",
            "form_catalog_unavailable?: SpellFormCatalogUnavailableReasonView",
            "content?: Array<CreatureSurfaceContentView>",
        ] {
            assert!(spell_surface.contains(field), "missing `{field}`");
        }
        assert!(!spell_surface.contains("definition:"));
        assert!(!spell_surface.contains("selected_form"));
        let spell_form = actual
            .get("SpellFormView.ts")
            .expect("SpellFormView binding should exist");
        for field in [
            "id: string",
            "label: string",
            "order: number",
            "minimum_cast_rank: number",
        ] {
            assert!(spell_form.contains(field), "missing `{field}`");
        }
        assert!(!spell_form.contains("authored_patch"));
        assert!(!spell_form.contains("result:"));
        for removed in [
            "SpellDamagePatchView.ts",
            "SpellDefinitionSurfaceView.ts",
            "SpellPatchView.ts",
            "SpellSelectedFormView.ts",
        ] {
            assert!(!actual.contains_key(removed), "stale binding `{removed}`");
        }
        let detail_request = actual
            .get("RecordDetailRequest.ts")
            .expect("record-detail request binding should exist");
        assert!(detail_request.contains("spell_form_id?: string"));
        assert!(detail_request.contains("spell_cast_rank?: number"));
        let effective_form = actual
            .get("SpellEffectiveFormView.ts")
            .expect("effective spell-form binding should exist");
        for field in [
            "id: string",
            "cast_rank: number",
            "result: SpellFormResultView",
        ] {
            assert!(effective_form.contains(field), "missing `{field}`");
        }
        let damage = actual
            .get("SpellDamageView.ts")
            .expect("spell damage binding should exist");
        assert!(damage.contains("label: string"));
        assert!(!damage.contains("key: string"));
        assert!(!damage.contains("order: number"));
        let fixed = actual
            .get("SpellFixedHeighteningView.ts")
            .expect("fixed heightening binding should exist");
        assert!(fixed.contains("changes: Array<SpellFixedHeighteningChangeView>"));
        assert!(!fixed.contains("patch:"));
        for binding in [
            "SpellSurfaceView.ts",
            "SpellFormView.ts",
            "SpellRuleView.ts",
        ] {
            let generated = actual
                .get(binding)
                .unwrap_or_else(|| panic!("{binding} binding should exist"));
            for forbidden in [
                "authored_object_json",
                "overlay_id",
                "publication_license",
                "raw_json",
                "source_id",
                "numeric",
                "image",
            ] {
                assert!(
                    !generated.contains(forbidden),
                    "{binding} must not expose `{forbidden}`"
                );
            }
        }
        let metadata = actual
            .get("RecordSurfaceMetadataView.ts")
            .expect("RecordSurfaceMetadataView binding should exist");
        assert!(metadata.contains("edition?: RecordSurfaceEditionView"));
        let edition = actual
            .get("RecordSurfaceEditionView.ts")
            .expect("RecordSurfaceEditionView binding should exist");
        assert!(edition.contains("status: RecordSurfaceEditionStatusView"));
        assert!(edition.contains("counterparts: Array<RecordSurfaceEditionCounterpartView>"));
        let encounter_participant = actual
            .get("EncounterParticipantView.ts")
            .expect("EncounterParticipantView binding should exist");
        assert!(encounter_participant.contains("record_view: RecordSurfaceView"));
        assert!(encounter_participant.contains("reset: EncounterParticipantResetAvailabilityView"));
        assert!(
            !encounter_participant.contains("surface: RecordSurfaceView"),
            "generated participant binding must not retain the superseded public field"
        );

        let creature = actual
            .get("CreatureSurfaceView.ts")
            .expect("CreatureSurfaceView binding should exist");
        for named_domain in [
            "teaser?: string",
            "size?: CreatureSurfaceSizeView",
            "adjustment?: CreatureSurfaceAdjustmentView",
            "initiative?: CreatureSurfaceInitiativeView",
            "vitals?: CreatureSurfaceVitalsView",
            "defenses?: CreatureSurfaceDefensesView",
            "saves?: CreatureSurfaceSavesView",
            "awareness?: CreatureSurfaceAwarenessView",
            "abilities?: CreatureSurfaceAbilitiesView",
            "skills?: Array<CreatureSurfaceSkillView>",
            "unmodeled_skills?: Array<CreatureSurfaceUnmodeledSkillView>",
            "movement?: Array<CreatureSurfaceMovementView>",
            "rituals?: CreatureSurfaceRitualsView",
            "equipment?: Array<CreatureSurfaceEquipmentView>",
            "lore?: Array<CreatureSurfaceLoreView>",
            "spellcasting?: Array<CreatureSurfaceSpellcastingView>",
            "standalone_spells?: Array<CreatureSurfaceSpellView>",
            "activities?: Array<CreatureSurfaceActivityView>",
            "content?: Array<CreatureSurfaceContentView>",
            "relationships?: Array<CreatureSurfaceRelationshipView>",
            "unavailable_domains?: CreatureSurfaceUnavailableDomainsView",
        ] {
            assert!(
                creature.contains(named_domain),
                "generated creature surface should expose `{named_domain}`"
            );
        }
        let size = actual
            .get("CreatureSurfaceSizeView.ts")
            .expect("size binding should exist");
        assert!(size.contains("value: CreatureSurfaceSizeValueView"));
        assert!(size.contains("provenance: CreatureSurfaceFactProvenanceView"));
        let adjustment = actual
            .get("CreatureSurfaceAdjustmentView.ts")
            .expect("adjustment binding should exist");
        assert!(adjustment.contains("value: CreatureSurfaceAdjustmentValueView"));
        assert!(adjustment.contains("provenance: CreatureSurfaceFactProvenanceView"));
        assert!(!creature.contains(&["sec", "tions"].concat()));
        let activity = actual
            .get("CreatureSurfaceActivityView.ts")
            .expect("activity binding should exist");
        assert!(activity.contains("content?: Array<CreatureSurfaceContentView>"));
        assert!(activity.contains("provenance: CreatureSurfaceOccurrenceProvenanceView"));
        for field in [
            "attack_effects?: Array<string>",
            "category?: string",
            "frequency?: CreatureSurfaceFrequencyView",
            "requirements?: string",
            "cost?: string",
            "uses?: CreatureSurfaceUsesView",
            "self_effect?: CreatureSurfaceSelfEffectView",
        ] {
            assert!(activity.contains(field), "missing `{field}`");
        }
        let spell = actual
            .get("CreatureSurfaceSpellView.ts")
            .expect("spell binding should exist");
        assert!(spell.contains("content?: Array<CreatureSurfaceContentView>"));
        assert!(spell.contains("context?: CreatureSurfaceSpellOccurrenceContextView"));
        assert!(spell.contains("provenance: CreatureSurfaceOccurrenceProvenanceView"));
        let spellcasting = actual
            .get("CreatureSurfaceSpellcastingView.ts")
            .expect("spellcasting binding should exist");
        assert!(spellcasting.contains("slots?: Array<CreatureSurfaceSpellSlotView>"));
        assert!(spellcasting.contains("provenance: CreatureSurfaceOccurrenceProvenanceView"));
        for binding in [
            "CreatureSurfaceEquipmentView.ts",
            "CreatureSurfaceLoreView.ts",
        ] {
            assert!(
                actual
                    .get(binding)
                    .unwrap_or_else(|| panic!("{binding} should exist"))
                    .contains("provenance: CreatureSurfaceOccurrenceProvenanceView"),
                "{binding} should reuse typed occurrence provenance"
            );
        }
        let occurrence_provenance = actual
            .get("CreatureSurfaceOccurrenceProvenanceView.ts")
            .expect("occurrence provenance binding should exist");
        for field in [
            "identity_stability: CreatureSurfaceOccurrenceIdentityStabilityView",
            "nested_source_id?: string",
            "stable_source_locator?: string",
            "source_locators?: Array<CreatureSurfaceSourceLocatorView>",
        ] {
            assert!(occurrence_provenance.contains(field), "missing `{field}`");
        }
        assert!(!occurrence_provenance.contains("source_path"));
        let unmodeled = actual
            .get("CreatureSurfaceUnmodeledSkillView.ts")
            .expect("unmodeled skill binding should exist");
        for field in [
            "component_id: string",
            "authored_order: number",
            "source_entries?: Array<CreatureSurfaceSkillSourceEntryView>",
            "source_item_id?: string",
            "authored_key: string",
        ] {
            assert!(unmodeled.contains(field), "missing `{field}`");
        }
        let content = actual
            .get("CreatureSurfaceContentView.ts")
            .expect("content binding should exist");
        assert!(content.contains("blocks: Array<CreatureSurfaceContentBlockView>"));
        assert!(!content.contains("owner:"));
        assert!(!content.contains("text:"));
        assert!(!actual.contains_key(&["CreatureSurfaceContent", "OwnerView.ts"].concat()));
        let content_inline = actual
            .get("CreatureSurfaceContentInlineView.ts")
            .expect("typed content inline binding should exist");
        for field in [
            "\"span_type\": \"check\"",
            "display: string",
            "statistic?: string",
            "difficulty_class?: number",
        ] {
            assert!(content_inline.contains(field), "missing `{field}`");
        }

        for binding in [
            "AddEncounterManualParticipantRequest.ts",
            "AddEncounterParticipantConditionRequest.ts",
            "AddEncounterRecordParticipantRequest.ts",
            "CreatureSurfaceAbilitiesView.ts",
            "CreatureSurfaceAwarenessView.ts",
            "CreatureSurfaceDefensesView.ts",
            "CreatureSurfaceIwrView.ts",
            "CreatureSurfaceMovementView.ts",
            "CreatureSurfaceResourceView.ts",
            "CreatureSurfaceRollView.ts",
            "CreatureSurfaceSaveView.ts",
            "CreatureSurfaceSenseView.ts",
            "CreatureSurfaceSkillView.ts",
            "CreatureSurfaceSpellView.ts",
            "CreatureSurfaceSpellcastingView.ts",
            "CreatureSurfaceVitalsView.ts",
            "EncounterConditionDefinitionView.ts",
            "EncounterParticipantPreservedDomainView.ts",
            "EncounterParticipantResetAvailabilityView.ts",
            "EncounterParticipantResetConfirmationView.ts",
            "EncounterParticipantResetDomainView.ts",
            "EncounterParticipantResetResultView.ts",
            "EncounterParticipantResetUnavailableReasonView.ts",
            "EncounterParticipantView.ts",
            "EncounterRuntimeActionCostKindView.ts",
            "EncounterRuntimeAutomationLimitationTargetView.ts",
            "EncounterRuntimeConditionView.ts",
            "EncounterRuntimeFrequencyView.ts",
            "EncounterRuntimeSpellSlotView.ts",
            "EncounterRuntimeSpellView.ts",
            "EncounterRuntimeUsesView.ts",
            "EncounterRuntimeVitalsView.ts",
            "EncounterSpellCastRequest.ts",
            "EncounterSpellCastResultView.ts",
            "EncounterSpellCastStateView.ts",
            "EncounterSpellSpendTargetView.ts",
            "EncounterSummaryView.ts",
            "ResetEncounterParticipantRequest.ts",
            "RecordSurfaceMetadataView.ts",
            "RuntimeAdjustmentView.ts",
            "RuntimeCanonicalTargetView.ts",
            "RuntimeCountSegmentView.ts",
            "RuntimeCountView.ts",
            "RuntimeDistanceView.ts",
            "RuntimeFactSourceView.ts",
            "RuntimeModifierView.ts",
            "RuntimeNumberView.ts",
            "RuntimeRollView.ts",
            "UpdateEncounterParticipantConditionRequest.ts",
            "UpdateEncounterParticipantRequest.ts",
        ] {
            assert!(
                !actual
                    .get(binding)
                    .unwrap_or_else(|| panic!("{binding} binding should exist"))
                    .contains("bigint"),
                "{binding} must match the ordinary-number HTTP contract"
            );
        }

        let unavailable = actual
            .get("CreatureSurfaceUnavailableDomainsView.ts")
            .expect("typed unavailable-domain binding should exist");
        for domain in [
            "classification?",
            "initiative?",
            "vitals?",
            "defenses?",
            "saves?",
            "awareness?",
            "abilities?",
            "skills?",
            "movement?",
            "resources?",
            "equipment?",
            "lore?",
            "spellcasting?",
            "activities?",
            "relationships?",
        ] {
            assert!(
                unavailable.contains(domain),
                "missing typed domain `{domain}`"
            );
        }
        let cause = actual
            .get("CreatureSurfaceUnavailableCauseView.ts")
            .expect("typed unavailable cause binding should exist");
        assert!(cause.contains("state: CreatureSurfaceUnavailableStateView"));
        assert!(cause.contains("field: CreatureSurfaceUnavailableFieldView"));
        assert!(cause.contains("unmodeled_skill?: CreatureSurfaceUnmodeledSkillView"));
        assert!(!cause.contains("source_path"));

        for (binding, optional_collections) in [
            (
                "CreatureSurfaceDefensesView.ts",
                &["immunities?", "resistances?", "weaknesses?"][..],
            ),
            (
                "CreatureSurfaceAwarenessView.ts",
                &["senses?", "languages?"][..],
            ),
            (
                "CreatureSurfaceActivityView.ts",
                &["traits?", "attack_effects?", "rolls?", "damage?"][..],
            ),
            (
                "CreatureSurfaceSkillView.ts",
                &["source_entries?", "variants?"][..],
            ),
            (
                "CreatureSurfaceSpellcastingView.ts",
                &["slots?", "spells?"][..],
            ),
            (
                "EncounterRuntimeView.ts",
                &[
                    "skills?",
                    "resources?",
                    "spellcasting?",
                    "standalone_spells?",
                    "activities?",
                    "conditions?",
                    "automation_limitations?",
                ][..],
            ),
            (
                "RuntimeNumberView.ts",
                &["modifiers?", "suppressed_modifiers?"][..],
            ),
        ] {
            let generated = actual
                .get(binding)
                .unwrap_or_else(|| panic!("{binding} binding should exist"));
            for field in optional_collections {
                assert!(
                    generated.contains(field),
                    "{binding} should expose optional collection `{field}`"
                );
            }
        }

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
                edition: None,
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
            issues: None,
            references: None,
            encounter: None,
        };

        let serialized = serde_json::to_value(surface).expect("record surface should serialize");
        assert_eq!(serialized["profile"], "record_detail");
        assert_eq!(
            serialized["presentation"]["presentation_type"],
            "unavailable"
        );
        assert!(serialized.get("encounter").is_none());
        assert!(serialized["metadata"].get("traits").is_none());
        assert!(serialized["presentation"].get("body").is_none());
        assert!(serialized.get(["sec", "tions"].concat()).is_none());
        assert!(serialized.get(["section", "order"].join("_")).is_none());
    }

    #[test]
    fn record_reference_not_requested_has_no_invented_totals() {
        let references = RecordSurfaceReferencesView {
            outgoing: RecordSurfaceReferenceSectionView::Available {
                next_limit: None,
                requested_limit: 8,
                records: Vec::new(),
                edges: Vec::new(),
                total_records: 0,
                total_edges: 0,
                truncated: false,
            },
            backlinks: RecordSurfaceReferenceSectionView::NotRequested,
        };

        let serialized = serde_json::to_value(references).expect("references should serialize");
        assert_eq!(serialized["outgoing"]["state"], "available");
        assert_eq!(serialized["backlinks"]["state"], "not_requested");
        assert!(serialized["backlinks"].get("total_records").is_none());
        assert!(serialized["backlinks"].get("total_edges").is_none());
        assert!(serialized["backlinks"].get("truncated").is_none());
    }

    #[test]
    fn record_detail_request_preserves_old_shape_and_typed_reference_limits() {
        let old_request: RecordDetailRequest = serde_json::from_value(serde_json::json!({
            "spell_form_id": "spell-form:test",
            "spell_cast_rank": 5
        }))
        .expect("the pre-reference request shape should remain valid");
        assert_eq!(
            old_request.spell_form_id.as_deref(),
            Some("spell-form:test")
        );
        assert_eq!(old_request.spell_cast_rank, Some(5));
        assert_eq!(old_request.reference_outgoing_limit, None);
        assert_eq!(old_request.reference_backlink_limit, None);

        let explicit_limits: RecordDetailRequest = serde_json::from_value(serde_json::json!({
            "reference_outgoing_limit": 0,
            "reference_backlink_limit": 8
        }))
        .expect("typed reference limits should deserialize");
        assert_eq!(explicit_limits.spell_form_id, None);
        assert_eq!(explicit_limits.spell_cast_rank, None);
        assert_eq!(explicit_limits.reference_outgoing_limit, Some(0));
        assert_eq!(explicit_limits.reference_backlink_limit, Some(8));
    }

    #[test]
    fn hazard_record_surface_serializes_tagged_body_and_preserves_zero_and_presence() {
        let surface = RecordSurfaceView {
            metadata: RecordSurfaceMetadataView {
                record_key: Some("hazards:test".to_string()),
                title: "Test Hazard".to_string(),
                kind: "hazard".to_string(),
                kind_label: "Hazard".to_string(),
                level: Some(1),
                rarity: None,
                traits: Vec::new(),
                edition: None,
                source: None,
            },
            profile: RecordSurfaceProfileView::RecordDetail,
            presentation: RecordSurfacePresentationView::Hazard {
                body: Box::new(HazardSurfaceView {
                    teaser: None,
                    complexity: Some(HazardSurfaceComplexityView::Complex),
                    size: None,
                    emits_sound: None,
                    detection: Some(HazardSurfaceDetectionView {
                        stealth_modifier: Some(0),
                        difficulty_class: Some(10),
                        details: None,
                    }),
                    defenses: None,
                    lifecycle: None,
                    activities: None,
                    content: None,
                    relationships: None,
                    unavailable_fields: Some(vec![HazardSurfaceUnavailableView {
                        fact_id: None,
                        state: HazardSurfaceUnavailableStateView::Null,
                        field: "defenses.hit_points.current".to_string(),
                        component_id: None,
                        message: "This canonical hazard field was explicitly null.".to_string(),
                    }]),
                    provenance: HazardSurfaceProvenanceView {
                        source_path: "packs/hazards/test.json".to_string(),
                        source_contract_version: "v1".to_string(),
                        source_system_version: "7".to_string(),
                        source_upstream_commit: "fixture".to_string(),
                        convenience_rule_id: "pf2e-hazard-conveniences".to_string(),
                        convenience_rule_version: 1,
                        image: HazardSurfaceProvenanceTextView::Value {
                            value: "systems/pf2e/icons/test.webp".to_string(),
                        },
                        publication_license: HazardSurfaceProvenanceTextView::Unsupported,
                        source_metadata: Vec::new(),
                    },
                }),
            },
            issues: None,
            references: None,
            encounter: None,
        };

        let serialized = serde_json::to_value(surface).expect("hazard surface should serialize");
        assert_eq!(serialized["presentation"]["presentation_type"], "hazard");
        assert_eq!(
            serialized["presentation"]["body"]["detection"]["stealth_modifier"],
            0
        );
        assert_eq!(
            serialized["presentation"]["body"]["unavailable_fields"][0]["state"],
            "null"
        );
        assert_eq!(
            serialized["presentation"]["body"]["provenance"]["image"]["state"],
            "value"
        );
        assert!(serialized["presentation"]["body"].get("image").is_none());
        assert!(
            serialized["presentation"]["body"]
                .get("publication_license")
                .is_none()
        );
    }

    #[test]
    fn record_surface_edition_serializes_typed_status_and_zero_or_more_counterparts() {
        let unlinked = RecordSurfaceEditionView {
            status: RecordSurfaceEditionStatusView::Legacy,
            counterparts: Vec::new(),
        };
        let linked = RecordSurfaceEditionView {
            status: RecordSurfaceEditionStatusView::Remaster,
            counterparts: vec![RecordSurfaceEditionCounterpartView {
                role: RecordSurfaceEditionCounterpartRoleView::LegacyCounterpart,
                record_key: "pathfinder-bestiary:KDRlxdIUADWHI6Vr".to_string(),
                title: "Air Mephit".to_string(),
            }],
        };

        let unlinked = serde_json::to_value(unlinked).expect("unlinked edition should serialize");
        let linked = serde_json::to_value(linked).expect("linked edition should serialize");
        assert_eq!(unlinked["status"], "legacy");
        assert_eq!(unlinked["counterparts"], serde_json::json!([]));
        assert_eq!(linked["status"], "remaster");
        assert_eq!(linked["counterparts"][0]["role"], "legacy_counterpart");
        assert_eq!(
            linked["counterparts"][0]["record_key"],
            "pathfinder-bestiary:KDRlxdIUADWHI6Vr"
        );
    }

    #[test]
    fn record_surface_omits_known_empty_collections_but_keeps_populated_true_many_values() {
        let surface = RecordSurfaceView {
            metadata: RecordSurfaceMetadataView {
                record_key: Some("test:creature".to_string()),
                title: "Collection Contract".to_string(),
                kind: "creature".to_string(),
                kind_label: "Creature".to_string(),
                level: Some(1),
                rarity: None,
                traits: vec!["beast".to_string()],
                edition: None,
                source: None,
            },
            profile: RecordSurfaceProfileView::RecordDetail,
            presentation: RecordSurfacePresentationView::Creature {
                body: Box::new(CreatureSurfaceView {
                    teaser: None,
                    size: None,
                    adjustment: None,
                    initiative: None,
                    vitals: None,
                    defenses: Some(CreatureSurfaceDefensesView {
                        armor_class: Some(15),
                        armor_class_details: None,
                        hardness: None,
                        shield: None,
                        immunities: Vec::new(),
                        resistances: vec![CreatureSurfaceIwrView {
                            component_id: "resistance-fire".to_string(),
                            authored_order: 0,
                            kind: "fire".to_string(),
                            amount: Some(5),
                            exceptions: Vec::new(),
                            double_vs: Vec::new(),
                        }],
                        weaknesses: Vec::new(),
                        provenance: surface_fact_provenance(),
                    }),
                    saves: None,
                    awareness: Some(CreatureSurfaceAwarenessView {
                        perception: Some(5),
                        details: None,
                        has_vision: Some(true),
                        senses: Vec::new(),
                        languages: Vec::new(),
                        language_details: None,
                        provenance: surface_fact_provenance(),
                    }),
                    abilities: None,
                    skills: Some(Vec::new()),
                    unmodeled_skills: None,
                    movement: None,
                    resources: None,
                    rituals: None,
                    equipment: None,
                    lore: None,
                    spellcasting: None,
                    standalone_spells: Some(Vec::new()),
                    activities: Some(vec![CreatureSurfaceActivityView {
                        occurrence_id: "activity-bite".to_string(),
                        authored_order: 0,
                        provenance: surface_occurrence_provenance(),
                        activity_type: CreatureSurfaceActivityTypeView::Strike,
                        label: "Bite".to_string(),
                        traits: Vec::new(),
                        action_cost: Some(CreatureSurfaceActionCostView::Actions { count: 1 }),
                        attack_effects: None,
                        category: None,
                        frequency: None,
                        requirements: None,
                        cost: None,
                        uses: None,
                        self_effect: None,
                        rolls: Vec::new(),
                        damage: Vec::new(),
                        content: None,
                    }]),
                    content: Some(Vec::new()),
                    relationships: Some(Vec::new()),
                    unavailable_domains: None,
                    provenance: None,
                }),
            },
            issues: None,
            references: None,
            encounter: None,
        };

        let serialized = serde_json::to_value(surface).expect("record surface should serialize");
        assert_eq!(
            serialized["metadata"]["traits"],
            serde_json::json!(["beast"])
        );
        let body = &serialized["presentation"]["body"];
        assert!(body.get("skills").is_none());
        assert!(body.get("content").is_none());
        assert!(body.get("standalone_spells").is_none());
        assert!(body.get("relationships").is_none());
        assert!(body["defenses"].get("immunities").is_none());
        assert!(body["defenses"].get("weaknesses").is_none());
        assert_eq!(
            body["defenses"]["resistances"].as_array().map(Vec::len),
            Some(1)
        );
        assert!(
            body["defenses"]["resistances"][0]
                .get("exceptions")
                .is_none()
        );
        assert_eq!(body["activities"].as_array().map(Vec::len), Some(1));
        assert!(body["activities"][0].get("rolls").is_none());
        assert_no_empty_containers(&serialized);
    }

    #[test]
    fn typed_domain_failure_is_distinct_from_known_empty_omission() {
        let unavailable = CreatureSurfaceUnavailableDomainsView {
            classification: None,
            initiative: None,
            vitals: None,
            defenses: None,
            saves: None,
            awareness: Some(CreatureSurfaceDomainUnavailableView {
                causes: vec![CreatureSurfaceUnavailableCauseView {
                    state: CreatureSurfaceUnavailableStateView::Null,
                    field: CreatureSurfaceUnavailableFieldView::Senses,
                    component_id: None,
                    provenance: CreatureSurfaceFactProvenanceView {
                        owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
                        field: CreatureSurfaceSourceFieldView::Perception,
                    },
                    unmodeled_skill: None,
                    message: "Display-only context.".to_string(),
                }],
            }),
            abilities: None,
            skills: None,
            movement: None,
            resources: None,
            equipment: None,
            lore: None,
            spellcasting: None,
            activities: None,
            relationships: None,
        };

        let serialized = serde_json::to_value(unavailable).expect("failure should serialize");
        assert_eq!(serialized["awareness"]["causes"][0]["state"], "null");
        assert_eq!(serialized["awareness"]["causes"][0]["field"], "senses");
        assert!(serialized.get("movement").is_none());
        assert!(
            serialized["awareness"]["causes"][0]
                .get("source_path")
                .is_none()
        );
    }

    #[test]
    fn unmodeled_skill_serialization_preserves_the_exact_authored_key_without_raw_diagnostics() {
        let authored_key = " Acrobatics +13\n{\"source_path\":\"private\"} ".to_string();
        let cause = CreatureSurfaceUnavailableCauseView {
            state: CreatureSurfaceUnavailableStateView::Unsupported,
            field: CreatureSurfaceUnavailableFieldView::UnmodeledSkill,
            component_id: Some("skill-source-7".to_string()),
            provenance: CreatureSurfaceFactProvenanceView {
                owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
                field: CreatureSurfaceSourceFieldView::Skills,
            },
            unmodeled_skill: Some(CreatureSurfaceUnmodeledSkillView {
                component_id: "skill-source-7".to_string(),
                authored_order: 7,
                source_entries: Some(vec![CreatureSurfaceSkillSourceEntryView {
                    authored_order: 0,
                    authored_key: authored_key.clone(),
                    modifier: CreatureSurfaceIntegerPresenceView::Null,
                }]),
                source_item_id: None,
                authored_key: authored_key.clone(),
                base: CreatureSurfaceIntegerPresenceView::Null,
                reason: CreatureSurfaceUnmodeledSkillReasonView::UnknownAuthoredKey,
            }),
            message: "The source supplied an unrecognized skill key.".to_string(),
        };

        let serialized = serde_json::to_value(cause).expect("unmodeled skill should serialize");
        assert_eq!(serialized["state"], "unsupported");
        assert_eq!(serialized["field"], "unmodeled_skill");
        assert_eq!(serialized["unmodeled_skill"]["authored_key"], authored_key);
        assert_eq!(
            serialized["unmodeled_skill"]["component_id"],
            "skill-source-7"
        );
        assert_eq!(serialized["unmodeled_skill"]["authored_order"], 7);
        assert_eq!(
            serialized["unmodeled_skill"]["source_entries"][0]["authored_key"],
            authored_key
        );
        assert_eq!(serialized["unmodeled_skill"]["base"]["state"], "null");
        assert_eq!(
            serialized["unmodeled_skill"]["reason"],
            "unknown_authored_key"
        );
        assert!(serialized.get("source_path").is_none());
        assert!(serialized.get("diagnostic").is_none());
        assert!(serialized.get("raw_json").is_none());
    }

    #[test]
    fn activity_content_serializes_as_typed_blocks_without_owner_or_flattened_text() {
        let content = CreatureSurfaceContentView {
            content_key: "item:plague:description".to_string(),
            role: CreatureSurfaceContentRoleView::EmbeddedCapability,
            authored_order: 4,
            label: Some("Abyssal Plague".to_string()),
            blocks: vec![
                CreatureSurfaceContentBlockView::Paragraph {
                    spans: vec![
                        CreatureSurfaceContentInlineView::Strong {
                            spans: vec![CreatureSurfaceContentInlineView::Text {
                                text: "Saving Throw".to_string(),
                            }],
                        },
                        CreatureSurfaceContentInlineView::Check {
                            display: " Fortitude DC 28".to_string(),
                            statistic: Some("fortitude".to_string()),
                            difficulty_class: Some(28),
                        },
                    ],
                },
                CreatureSurfaceContentBlockView::Divider,
            ],
            content_hash: "abc123".to_string(),
            visibility: "public".to_string(),
            provenance: CreatureSurfaceContentProvenanceView {
                source_record_key: "bestiary:night-hag".to_string(),
                relative_source_path: "items[plague].system.description.value".to_string(),
                field_family: "embedded_item_description".to_string(),
                nested_source_id: Some("plague".to_string()),
            },
        };
        let activity = CreatureSurfaceActivityView {
            occurrence_id: "occurrence:plague".to_string(),
            authored_order: 4,
            provenance: surface_occurrence_provenance(),
            activity_type: CreatureSurfaceActivityTypeView::Action,
            label: "Abyssal Plague".to_string(),
            traits: Vec::new(),
            action_cost: None,
            attack_effects: None,
            category: None,
            frequency: None,
            requirements: None,
            cost: None,
            uses: None,
            self_effect: None,
            rolls: Vec::new(),
            damage: Vec::new(),
            content: Some(vec![content.clone()]),
        };
        let spell = CreatureSurfaceSpellView {
            occurrence_id: "occurrence:bind-soul".to_string(),
            authored_order: 5,
            provenance: surface_occurrence_provenance(),
            label: "Bind Soul".to_string(),
            target_record_key: None,
            rank: Some(9),
            context: None,
            traits: vec!["spell".to_string()],
            content: Some(vec![content]),
        };

        let serialized = serde_json::to_value(activity).expect("activity should serialize");
        assert_eq!(
            serialized["content"][0]["blocks"][1]["block_type"],
            "divider"
        );
        assert_eq!(
            serialized["content"][0]["blocks"][0]["spans"][0]["span_type"],
            "strong"
        );
        assert_eq!(
            serialized["content"][0]["blocks"][0]["spans"][1],
            serde_json::json!({
                "span_type": "check",
                "display": " Fortitude DC 28",
                "statistic": "fortitude",
                "difficulty_class": 28,
            })
        );
        assert!(serialized["content"][0].get("owner").is_none());
        assert!(serialized["content"][0].get("text").is_none());

        let serialized = serde_json::to_value(spell).expect("spell should serialize");
        assert_eq!(
            serialized["content"][0]["blocks"][1]["block_type"],
            "divider"
        );
        assert!(serialized["content"][0].get("owner").is_none());
    }

    #[test]
    fn app_contract_integers_enforce_the_javascript_safe_range() {
        let metadata = |level| RecordSurfaceMetadataView {
            record_key: Some("creatures:safe-integer".to_string()),
            title: "Safe Integer".to_string(),
            kind: "creature".to_string(),
            kind_label: "Creature".to_string(),
            level,
            rarity: None,
            traits: Vec::new(),
            edition: None,
            source: None,
        };

        assert_eq!(
            serde_json::to_value(metadata(Some(json_integer::JS_SAFE_INTEGER_MAX)))
                .expect("maximum safe integer should serialize")["level"],
            json_integer::JS_SAFE_INTEGER_MAX
        );
        assert_eq!(
            serde_json::to_value(metadata(Some(json_integer::JS_SAFE_INTEGER_MIN)))
                .expect("minimum safe integer should serialize")["level"],
            json_integer::JS_SAFE_INTEGER_MIN
        );
        let error = serde_json::to_value(metadata(Some(json_integer::JS_SAFE_INTEGER_MAX + 1)))
            .expect_err("unsafe integer must not serialize");
        assert!(error.to_string().contains("JavaScript safe-integer range"));
        let error = serde_json::to_value(metadata(Some(json_integer::JS_SAFE_INTEGER_MIN - 1)))
            .expect_err("negative unsafe integer must not serialize");
        assert!(error.to_string().contains("JavaScript safe-integer range"));

        let error = serde_json::from_value::<RecordSurfaceMetadataView>(serde_json::json!({
            "title": "Unsafe Integer",
            "kind": "creature",
            "kind_label": "Creature",
            "level": 9_007_199_254_740_992_i64,
        }))
        .expect_err("unsafe integer must not deserialize");
        assert!(error.to_string().contains("JavaScript safe-integer range"));
    }

    #[test]
    fn encounter_request_optional_integers_accept_omission_and_bound_present_values() {
        let record =
            serde_json::from_value::<AddEncounterRecordParticipantRequest>(serde_json::json!({
                "encounter_ref": "ambush",
                "record_ref": "actors:test-creature",
                "quantity": 1,
            }))
            .expect("omitted record-participant initiative should deserialize");
        assert_eq!(record.initiative, None);

        let manual =
            serde_json::from_value::<AddEncounterManualParticipantRequest>(serde_json::json!({
                "encounter_ref": "ambush",
                "display_name": "Kyra",
            }))
            .expect("omitted manual-participant integers should deserialize");
        assert_eq!(
            (manual.max_hp, manual.current_hp, manual.initiative),
            (None, None, None)
        );

        let participant =
            serde_json::from_value::<UpdateEncounterParticipantRequest>(serde_json::json!({
                "participant_key": "participant-a",
                "display_name": "Goblin",
                "side": "enemy",
                "participant_variant": "normal",
                "temporary_hp": 0,
                "defeated": false,
                "hidden": false,
            }))
            .expect("omitted participant-update integers should deserialize");
        assert_eq!(
            (
                participant.initiative,
                participant.max_hp,
                participant.current_hp
            ),
            (None, None, None)
        );

        let added_condition = serde_json::from_value::<AddEncounterParticipantConditionRequest>(
            serde_json::json!({"participant_key": "participant-a"}),
        )
        .expect("omitted condition-add integers should deserialize");
        assert_eq!(
            (added_condition.value, added_condition.duration_rounds),
            (None, None)
        );

        let updated_condition =
            serde_json::from_value::<UpdateEncounterParticipantConditionRequest>(
                serde_json::json!({
                    "condition_id": 7,
                    "name": "Clumsy",
                }),
            )
            .expect("omitted condition-update integers should deserialize");
        assert_eq!(
            (updated_condition.value, updated_condition.duration_rounds),
            (None, None)
        );

        let bounded_condition =
            serde_json::from_value::<UpdateEncounterParticipantConditionRequest>(
                serde_json::json!({
                    "condition_id": 7,
                    "name": "Clumsy",
                    "value": json_integer::JS_SAFE_INTEGER_MAX,
                    "duration_rounds": json_integer::JS_SAFE_INTEGER_MIN,
                }),
            )
            .expect("present safe optional integers should deserialize");
        assert_eq!(
            (bounded_condition.value, bounded_condition.duration_rounds),
            (
                Some(json_integer::JS_SAFE_INTEGER_MAX),
                Some(json_integer::JS_SAFE_INTEGER_MIN),
            )
        );

        let error = serde_json::from_value::<UpdateEncounterParticipantConditionRequest>(
            serde_json::json!({
                "condition_id": 7,
                "name": "Clumsy",
                "value": json_integer::JS_SAFE_INTEGER_MAX + 1,
            }),
        )
        .expect_err("present unsafe optional integer should be rejected");
        assert!(error.to_string().contains("JavaScript safe-integer range"));
    }

    #[test]
    fn spell_cast_request_round_trips_typed_targets_and_rejects_unsafe_ranks() {
        let request = serde_json::json!({
            "spell_occurrence_id": "spell-shadow-blast",
            "spend_target": {
                "target_type": "innate_use",
                "entry_id": "entry-occult",
                "spell_occurrence_id": "spell-shadow-blast"
            },
            "operation": "cast_one"
        });
        let decoded = serde_json::from_value::<EncounterSpellCastRequest>(request.clone())
            .expect("typed innate cast request should deserialize");
        assert_eq!(
            serde_json::to_value(decoded).expect("cast request should serialize"),
            request
        );

        let prepared = serde_json::from_value::<EncounterSpellCastRequest>(serde_json::json!({
            "spell_occurrence_id": "spell-fireball",
            "spend_target": {
                "target_type": "prepared_slot",
                "entry_id": "entry-arcane",
                "rank": json_integer::JS_SAFE_INTEGER_MAX,
                "slot_id": "slot4:0"
            },
            "operation": "restore_one"
        }))
        .expect("safe prepared rank should deserialize");
        assert!(matches!(
            prepared.spend_target,
            EncounterSpellSpendTargetView::PreparedSlot { rank, .. }
                if rank == json_integer::JS_SAFE_INTEGER_MAX
        ));

        let error = serde_json::from_value::<EncounterSpellCastRequest>(serde_json::json!({
            "spell_occurrence_id": "spell-fireball",
            "spend_target": {
                "target_type": "spontaneous_pool",
                "entry_id": "entry-arcane",
                "rank": json_integer::JS_SAFE_INTEGER_MAX + 1
            },
            "operation": "cast_one"
        }))
        .expect_err("unsafe spell rank must be rejected");
        assert!(error.to_string().contains("JavaScript safe-integer range"));
    }

    #[test]
    fn participant_reset_requires_the_typed_destructive_confirmation() {
        let request = serde_json::json!({ "confirmation": "reset_participant" });
        let decoded = serde_json::from_value::<ResetEncounterParticipantRequest>(request.clone())
            .expect("typed reset confirmation should deserialize");
        assert_eq!(
            decoded.confirmation,
            EncounterParticipantResetConfirmationView::ResetParticipant
        );
        assert_eq!(
            serde_json::to_value(decoded).expect("reset request should serialize"),
            request
        );
        assert!(
            serde_json::from_value::<ResetEncounterParticipantRequest>(serde_json::json!({
                "confirmation": "confirmed"
            }))
            .is_err()
        );
    }

    #[test]
    fn targeted_automation_limitation_remains_explicit_when_empty_runtime_arrays_are_omitted() {
        let runtime = EncounterRuntimeView {
            hazard: None,
            level: None,
            vitals: None,
            defenses: None,
            saves: None,
            awareness: None,
            abilities: None,
            skills: Vec::new(),
            movement: None,
            resources: Vec::new(),
            spellcasting: Vec::new(),
            standalone_spells: Vec::new(),
            activities: Vec::new(),
            action_budget: None,
            conditions: Vec::new(),
            automation_limitations: vec![EncounterRuntimeAutomationLimitationView {
                code: EncounterRuntimeAutomationLimitationCodeView::ActivityCheckNotAutomated,
                target: EncounterRuntimeAutomationLimitationTargetView::Activity {
                    activity_id: "activity-recall".to_string(),
                },
                message: "Resolve this check manually.".to_string(),
            }],
        };

        let serialized = serde_json::to_value(runtime).expect("runtime should serialize");
        let object = serialized.as_object().expect("runtime should be an object");
        assert_eq!(object.len(), 1);
        assert_eq!(
            serialized["automation_limitations"][0]["target"]["target_type"],
            "activity"
        );
        assert!(serialized.get("activities").is_none());
        assert_no_empty_containers(&serialized);
    }

    #[test]
    fn encounter_runtime_level_serializes_with_inner_adjustment_semantics() {
        let runtime = EncounterRuntimeView {
            hazard: None,
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
            standalone_spells: Vec::new(),
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
                "provenance": {
                    "source": { "source_type": "canonical_record" },
                    "canonical_target": { "target_type": "level" }
                }
            }))
        );
        for empty_collection in [
            "skills",
            "resources",
            "spellcasting",
            "activities",
            "conditions",
            "automation_limitations",
        ] {
            assert!(object.get(empty_collection).is_none());
        }
    }

    #[test]
    fn encounter_participant_serializes_record_view_without_surface_alias() {
        let record_view = RecordSurfaceView {
            metadata: RecordSurfaceMetadataView {
                record_key: Some("actors:test-creature".to_string()),
                title: "Test Creature".to_string(),
                kind: "creature".to_string(),
                kind_label: "Creature".to_string(),
                level: Some(5),
                rarity: None,
                traits: Vec::new(),
                edition: None,
                source: None,
            },
            profile: RecordSurfaceProfileView::EncounterParticipant,
            presentation: RecordSurfacePresentationView::Unavailable {
                unavailable: SurfaceUnavailableView {
                    reason: SurfaceUnavailableReasonView::RecordFamilyNotMigrated,
                    requested_kind: "creature".to_string(),
                    message: "Fixture presentation unavailable.".to_string(),
                },
            },
            issues: None,
            references: None,
            encounter: Some(EncounterRuntimeView {
                hazard: None,
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
                standalone_spells: Vec::new(),
                activities: Vec::new(),
                action_budget: None,
                conditions: Vec::new(),
                automation_limitations: Vec::new(),
            }),
        };
        let expected_record_view =
            serde_json::to_value(&record_view).expect("record view should serialize");
        let participant = EncounterParticipantView {
            participant_key: "participant-1".to_string(),
            record_key: Some("actors:test-creature".to_string()),
            participant_kind: EncounterParticipantKindView::Creature,
            participant_variant: EncounterParticipantVariantView::Normal,
            status: EncounterParticipantStatusView::Active,
            position: 1,
            display_name: "Test Creature".to_string(),
            side: EncounterParticipantSideView::Enemy,
            initiative: Some(18),
            initiative_order: 1,
            defeated: false,
            hidden: false,
            note: None,
            note_hint: None,
            reset: EncounterParticipantResetAvailabilityView {
                available: true,
                unavailable_reason: None,
            },
            record_view,
        };

        let serialized = serde_json::to_value(participant).expect("participant should serialize");
        let object = serialized
            .as_object()
            .expect("participant should serialize as an object");
        assert_eq!(object.get("record_view"), Some(&expected_record_view));
        assert!(object.get("surface").is_none());
        assert_eq!(
            serialized["record_view"]["presentation"]["presentation_type"],
            "unavailable"
        );
        assert_eq!(
            serialized["record_view"]["encounter"]["level"]["adjusted_value"],
            6
        );
    }

    fn surface_fact_provenance() -> CreatureSurfaceFactProvenanceView {
        CreatureSurfaceFactProvenanceView {
            owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
            field: CreatureSurfaceSourceFieldView::Defenses,
        }
    }

    fn surface_occurrence_provenance() -> CreatureSurfaceOccurrenceProvenanceView {
        CreatureSurfaceOccurrenceProvenanceView {
            identity_stability:
                CreatureSurfaceOccurrenceIdentityStabilityView::StableNestedSourceId,
            nested_source_id: Some("source-item".to_string()),
            stable_source_locator: Some("items/source-item".to_string()),
            source_locators: Some(vec![CreatureSurfaceSourceLocatorView {
                locator: "items/source-item".to_string(),
                precedence: 0,
            }]),
        }
    }

    fn assert_no_empty_containers(value: &serde_json::Value) {
        match value {
            serde_json::Value::Object(object) => {
                assert!(!object.is_empty(), "serialized object should not be empty");
                for child in object.values() {
                    assert_no_empty_containers(child);
                }
            }
            serde_json::Value::Array(values) => {
                assert!(!values.is_empty(), "serialized array should not be empty");
                for child in values {
                    assert_no_empty_containers(child);
                }
            }
            _ => {}
        }
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
        EncounterParticipantResetResultView::export_all_to(path)
            .expect("EncounterParticipantResetResultView bindings should export");
        ResetEncounterParticipantRequest::export_all_to(path)
            .expect("ResetEncounterParticipantRequest bindings should export");
        EncounterSpellCastRequest::export_all_to(path)
            .expect("EncounterSpellCastRequest bindings should export");
        EncounterSpellCastResultView::export_all_to(path)
            .expect("EncounterSpellCastResultView bindings should export");
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
        RecordDetailRequest::export_all_to(path)
            .expect("RecordDetailRequest bindings should export");
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
