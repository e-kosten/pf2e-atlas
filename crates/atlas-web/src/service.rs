use std::sync::Arc;

use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AddSavedListItemRequest, AdvanceEncounterTurnRequest,
    AppError, AppErrorCode, AppReadinessView, ApplyEncounterHpActionRequest,
    ApplyEncounterSpellActionRequest, CreateEncounterRequest, CreateSavedListRequest,
    DeleteEncounterView, DeleteSavedListView, DiscoverFilterEditorRequest,
    DiscoverFilterValuesRequest, EncounterConditionCatalogView, EncounterCreateView,
    EncounterDetailView, EncounterIndexView, EncounterMutationView, EncounterTurnReviewView,
    FilterEditorView, FilterSavedListRequest, FilterValueListView, RecordDetailView,
    RemoveSavedListItemRequest, ReorderEncounterParticipantRequest, SavedListCreateView,
    SavedListDetailView, SavedListIndexView, SavedListItemMutationView, SavedListUpdateView,
    SearchResultsPage, SearchResultsRequest, SetEncounterTurnRequest,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest, UpdateSavedListRequest,
};
use atlas_app_model::{
    DiscoverFilterCountsRequest, FilterCountsView, FilterValidationResult, RecordDetailRequest,
};
use atlas_app_model::{InitiativeOptionsView, InitiativeRollView, RollInitiativeRequest};
use atlas_app_service::{AppServiceError, AtlasAppService, ReadExecutionContext};
use atlas_domain::QueryPredicate;
use tokio::sync::Semaphore;

use crate::error::WebError;

const MAX_BLOCKING_SERVICE_CALLS: usize = 64;

#[derive(Clone)]
pub(crate) struct AtlasWebState {
    pub(crate) service: Arc<dyn AtlasWebService>,
    blocking_calls: Arc<Semaphore>,
    blocking_call_capacity: usize,
}

impl AtlasWebState {
    pub(crate) fn new(service: AtlasAppService) -> Self {
        Self::from_service(service)
    }

    pub(crate) fn from_service(service: impl AtlasWebService + 'static) -> Self {
        Self::from_service_with_blocking_capacity(service, MAX_BLOCKING_SERVICE_CALLS)
    }

    pub(crate) fn from_service_with_blocking_capacity(
        service: impl AtlasWebService + 'static,
        blocking_call_capacity: usize,
    ) -> Self {
        Self {
            service: Arc::new(service),
            blocking_calls: Arc::new(Semaphore::new(blocking_call_capacity)),
            blocking_call_capacity,
        }
    }
}

pub(crate) trait AtlasWebService: Send + Sync {
    fn roll_flat_check(
        &self,
        request: atlas_app_model::RollFlatCheckRequest,
    ) -> Result<atlas_app_model::FlatCheckRollView, AppServiceError>;
    fn roll_table(
        &self,
        request: atlas_app_model::RollTableRequest,
    ) -> Result<atlas_app_model::TableRollView, AppServiceError>;
    fn encounter_strike_options(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::EncounterStrikeOptionsRequest,
    ) -> Result<atlas_app_model::EncounterStrikeOptionsView, AppServiceError>;
    fn roll_encounter_strike(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::RollEncounterStrikeRequest,
    ) -> Result<atlas_app_model::EncounterStrikeRollView, AppServiceError>;
    fn readiness(&self) -> AppReadinessView;

    fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> Result<FilterEditorView, AppServiceError>;

    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
        execution: &ReadExecutionContext,
    ) -> Result<FilterValueListView, AppServiceError>;

    fn search_page(
        &self,
        request: SearchResultsRequest,
        execution: &ReadExecutionContext,
    ) -> Result<SearchResultsPage, AppServiceError>;

    fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError>;
    fn encounter_reference(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: atlas_app_model::EncounterReferenceRequest,
    ) -> Result<RecordDetailView, AppServiceError>;
    fn embedded_reading(
        &self,
        request: atlas_app_model::EmbeddedReadingRequest,
    ) -> Result<atlas_app_model::EmbeddedReadingBatchView, AppServiceError>;
    fn record_detail_at(
        &self,
        request: RecordDetailRequest,
    ) -> Result<RecordDetailView, AppServiceError>;
    fn discover_filter_counts(
        &self,
        request: DiscoverFilterCountsRequest,
        execution: &ReadExecutionContext,
    ) -> Result<FilterCountsView, AppServiceError>;
    fn validate_filter(
        &self,
        predicate: QueryPredicate,
    ) -> Result<FilterValidationResult, AppServiceError>;

    fn related_records(
        &self,
        request: atlas_app_model::RelatedRecordsRequest,
    ) -> Result<atlas_app_model::RelatedRecordsView, AppServiceError>;

    fn preview_encounter_condition_add(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::AddEncounterParticipantConditionRequest,
    ) -> Result<atlas_app_model::ConditionEventPreviewView, AppServiceError>;
    fn preview_encounter_condition_update(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: atlas_app_model::UpdateEncounterParticipantConditionRequest,
    ) -> Result<atlas_app_model::ConditionEventPreviewView, AppServiceError>;
    fn pending_encounter_condition_consequences(
        &self,
        encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingConditionConsequenceView>, AppServiceError>;
    fn resolve_encounter_drained_consequence(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ResolveDrainedConsequenceRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;
    fn set_encounter_life_settings(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: atlas_app_model::SetEncounterLifeSettingsRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn recover_persistent_damage(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::RecoverPersistentDamageRequest,
    ) -> Result<atlas_app_model::PersistentRecoveryResultView, AppServiceError>;
    fn apply_encounter_hp_event(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ApplyEncounterHpEventRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;
    fn pending_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingHpLifecycleView>, AppServiceError>;
    fn resolve_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ResolveEncounterHpLifecycleRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;
    fn apply_persistent_damage(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ApplyPersistentDamageRequest,
    ) -> Result<atlas_app_model::PersistentDamageResultView, AppServiceError>;
    fn apply_condition_lifecycle(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ApplyConditionLifecycleRequest,
    ) -> Result<atlas_app_model::ConditionLifecycleResultView, AppServiceError>;
    fn condition_lifecycle_actions(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        boundary_key: Option<&str>,
    ) -> Result<atlas_app_model::ConditionLifecycleActionsView, AppServiceError>;
    fn pending_condition_recoveries(
        &self,
        encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingConditionRecoveryView>, AppServiceError>;
    fn persistent_damage_review(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        boundary_key: Option<&str>,
    ) -> Result<atlas_app_model::PersistentReviewView, AppServiceError>;
    fn preview_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ResolveEncounterHpLifecycleRequest,
    ) -> Result<atlas_app_model::PendingHpLifecycleView, AppServiceError>;
    fn encounters(&self) -> Result<EncounterIndexView, AppServiceError>;

    fn participant_encounter_condition_definitions(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> Result<EncounterConditionCatalogView, AppServiceError>;

    fn encounter_condition_definitions(
        &self,
    ) -> Result<EncounterConditionCatalogView, AppServiceError>;

    fn encounter(&self, encounter_ref: &str) -> Result<EncounterDetailView, AppServiceError>;

    fn create_encounter(
        &self,
        request: CreateEncounterRequest,
    ) -> Result<EncounterCreateView, AppServiceError>;

    fn delete_encounter(&self, encounter_ref: &str)
    -> Result<DeleteEncounterView, AppServiceError>;

    fn update_encounter(
        &self,
        request: UpdateEncounterRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn apply_encounter_hp_action(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterHpActionRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn encounter_initiative_options(
        &self,
        encounter_ref: &str,
    ) -> Result<InitiativeOptionsView, AppServiceError>;
    fn roll_encounter_initiative(
        &self,
        encounter_ref: &str,
        request: RollInitiativeRequest,
    ) -> Result<InitiativeRollView, AppServiceError>;

    fn apply_encounter_detection_event(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ApplyEncounterDetectionEventRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;
    fn worsen_encounter_attitude(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::WorsenEncounterAttitudeRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;
    fn encounter_relationships(
        &self,
        encounter_ref: &str,
        subject_key: &str,
        other_key: &str,
    ) -> Result<atlas_app_model::EncounterRelationshipsView, AppServiceError>;
    fn set_encounter_relationship(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::SetEncounterRelationshipRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;
    fn preview_encounter_action(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::PreviewEncounterActionRequest,
    ) -> Result<atlas_app_model::EncounterActionPreviewView, AppServiceError>;
    fn preview_encounter_strike_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::PreviewEncounterStrikeAttemptRequest,
    ) -> Result<atlas_app_model::EncounterStrikeAttemptPreviewView, AppServiceError>;
    fn preview_encounter_casting_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::PreviewEncounterCastingAttemptRequest,
    ) -> Result<atlas_app_model::EncounterCastingPreviewView, AppServiceError>;
    fn begin_encounter_strike_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::BeginEncounterStrikeAttemptRequest,
    ) -> Result<atlas_app_model::EncounterActionMutationView, AppServiceError>;
    fn begin_encounter_casting_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::BeginEncounterCastingAttemptRequest,
    ) -> Result<atlas_app_model::EncounterActionMutationView, AppServiceError>;
    fn resolve_encounter_action_check(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ResolveEncounterActionCheckRequest,
    ) -> Result<atlas_app_model::EncounterActionMutationView, AppServiceError>;
    fn abandon_encounter_action_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::AbandonEncounterActionAttemptRequest,
    ) -> Result<atlas_app_model::EncounterActionMutationView, AppServiceError>;
    fn encounter_strike_attempts(
        &self,
        encounter_ref: &str,
    ) -> Result<atlas_app_model::EncounterStrikeAttemptsView, AppServiceError>;
    fn encounter_casting_attempts(
        &self,
        encounter_ref: &str,
    ) -> Result<atlas_app_model::EncounterCastingAttemptsView, AppServiceError>;
    fn apply_encounter_spell_action(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterSpellActionRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        request: ReorderEncounterParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn encounter_turn_review(
        &self,
        encounter_ref: &str,
    ) -> Result<EncounterTurnReviewView, AppServiceError>;
    fn advance_encounter_turn(
        &self,
        encounter_ref: &str,
        request: AdvanceEncounterTurnRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
        request: atlas_app_model::RemoveEncounterParticipantConditionRequest,
    ) -> Result<EncounterMutationView, AppServiceError>;

    fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError>;

    fn saved_list(&self, list_ref: &str) -> Result<SavedListDetailView, AppServiceError>;

    fn filter_saved_list(
        &self,
        request: FilterSavedListRequest,
    ) -> Result<SavedListDetailView, AppServiceError>;

    fn create_saved_list(
        &self,
        request: CreateSavedListRequest,
    ) -> Result<SavedListCreateView, AppServiceError>;

    fn update_saved_list(
        &self,
        request: UpdateSavedListRequest,
    ) -> Result<SavedListUpdateView, AppServiceError>;

    fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError>;

    fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError>;

    fn delete_saved_list(&self, list_ref: &str) -> Result<DeleteSavedListView, AppServiceError>;
}

impl AtlasWebService for AtlasAppService {
    fn roll_encounter_strike(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::RollEncounterStrikeRequest,
    ) -> Result<atlas_app_model::EncounterStrikeRollView, AppServiceError> {
        self.roll_encounter_strike(encounter_ref, request)
    }
    fn encounter_strike_options(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::EncounterStrikeOptionsRequest,
    ) -> Result<atlas_app_model::EncounterStrikeOptionsView, AppServiceError> {
        self.encounter_strike_options(encounter_ref, request)
    }
    fn roll_flat_check(
        &self,
        request: atlas_app_model::RollFlatCheckRequest,
    ) -> Result<atlas_app_model::FlatCheckRollView, AppServiceError> {
        self.roll_flat_check(request)
    }
    fn roll_table(
        &self,
        request: atlas_app_model::RollTableRequest,
    ) -> Result<atlas_app_model::TableRollView, AppServiceError> {
        self.roll_table(request)
    }
    fn encounter_reference(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: atlas_app_model::EncounterReferenceRequest,
    ) -> Result<RecordDetailView, AppServiceError> {
        self.encounter_reference(encounter_ref, participant_key, request)
    }
    fn embedded_reading(
        &self,
        request: atlas_app_model::EmbeddedReadingRequest,
    ) -> Result<atlas_app_model::EmbeddedReadingBatchView, AppServiceError> {
        self.embedded_reading(request)
    }
    fn related_records(
        &self,
        request: atlas_app_model::RelatedRecordsRequest,
    ) -> Result<atlas_app_model::RelatedRecordsView, AppServiceError> {
        self.related_records(request)
    }

    fn record_detail_at(
        &self,
        request: RecordDetailRequest,
    ) -> Result<RecordDetailView, AppServiceError> {
        self.record_detail_at(request)
    }
    fn discover_filter_counts(
        &self,
        request: DiscoverFilterCountsRequest,
        execution: &ReadExecutionContext,
    ) -> Result<FilterCountsView, AppServiceError> {
        self.discover_filter_counts(request, execution)
    }
    fn validate_filter(
        &self,
        predicate: QueryPredicate,
    ) -> Result<FilterValidationResult, AppServiceError> {
        self.validate_filter(predicate)
    }
    fn readiness(&self) -> AppReadinessView {
        self.readiness()
    }

    fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> Result<FilterEditorView, AppServiceError> {
        self.discover_filter_editor(request)
    }

    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
        execution: &ReadExecutionContext,
    ) -> Result<FilterValueListView, AppServiceError> {
        self.discover_filter_values(request, execution)
    }

    fn search_page(
        &self,
        request: SearchResultsRequest,
        execution: &ReadExecutionContext,
    ) -> Result<SearchResultsPage, AppServiceError> {
        self.search_page(request, execution)
    }

    fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError> {
        self.record_detail(record_key)
    }

    fn preview_encounter_condition_add(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::AddEncounterParticipantConditionRequest,
    ) -> Result<atlas_app_model::ConditionEventPreviewView, AppServiceError> {
        self.preview_encounter_condition_add(encounter_ref, request)
    }
    fn preview_encounter_condition_update(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: atlas_app_model::UpdateEncounterParticipantConditionRequest,
    ) -> Result<atlas_app_model::ConditionEventPreviewView, AppServiceError> {
        self.preview_encounter_condition_update(encounter_ref, participant_key, request)
    }
    fn pending_encounter_condition_consequences(
        &self,
        encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingConditionConsequenceView>, AppServiceError> {
        self.pending_encounter_condition_consequences(encounter_ref)
    }
    fn resolve_encounter_drained_consequence(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ResolveDrainedConsequenceRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.resolve_encounter_drained_consequence(encounter_ref, request)
    }
    fn set_encounter_life_settings(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: atlas_app_model::SetEncounterLifeSettingsRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.set_encounter_life_settings(encounter_ref, participant_key, request)
    }

    fn recover_persistent_damage(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::RecoverPersistentDamageRequest,
    ) -> Result<atlas_app_model::PersistentRecoveryResultView, AppServiceError> {
        self.recover_persistent_damage(encounter_ref, request)
    }
    fn apply_encounter_hp_event(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ApplyEncounterHpEventRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.apply_encounter_hp_event(encounter_ref, request)
    }
    fn pending_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingHpLifecycleView>, AppServiceError> {
        self.pending_encounter_hp_lifecycle(encounter_ref)
    }
    fn resolve_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ResolveEncounterHpLifecycleRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.resolve_encounter_hp_lifecycle(encounter_ref, request)
    }
    fn apply_persistent_damage(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ApplyPersistentDamageRequest,
    ) -> Result<atlas_app_model::PersistentDamageResultView, AppServiceError> {
        self.apply_persistent_damage(encounter_ref, request)
    }
    fn apply_condition_lifecycle(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ApplyConditionLifecycleRequest,
    ) -> Result<atlas_app_model::ConditionLifecycleResultView, AppServiceError> {
        self.apply_condition_lifecycle(encounter_ref, request)
    }
    fn condition_lifecycle_actions(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        boundary_key: Option<&str>,
    ) -> Result<atlas_app_model::ConditionLifecycleActionsView, AppServiceError> {
        self.condition_lifecycle_actions(encounter_ref, participant_key, boundary_key)
    }
    fn pending_condition_recoveries(
        &self,
        encounter_ref: &str,
    ) -> Result<Vec<atlas_app_model::PendingConditionRecoveryView>, AppServiceError> {
        self.pending_condition_recoveries(encounter_ref)
    }
    fn persistent_damage_review(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        boundary_key: Option<&str>,
    ) -> Result<atlas_app_model::PersistentReviewView, AppServiceError> {
        self.persistent_damage_review(encounter_ref, participant_key, boundary_key)
    }
    fn preview_encounter_hp_lifecycle(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ResolveEncounterHpLifecycleRequest,
    ) -> Result<atlas_app_model::PendingHpLifecycleView, AppServiceError> {
        self.preview_encounter_hp_lifecycle(encounter_ref, request)
    }
    fn encounters(&self) -> Result<EncounterIndexView, AppServiceError> {
        self.encounters()
    }

    fn participant_encounter_condition_definitions(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> Result<EncounterConditionCatalogView, AppServiceError> {
        self.participant_encounter_condition_definitions(encounter_ref, participant_key)
    }

    fn encounter_condition_definitions(
        &self,
    ) -> Result<EncounterConditionCatalogView, AppServiceError> {
        self.encounter_condition_definitions()
    }

    fn encounter(&self, encounter_ref: &str) -> Result<EncounterDetailView, AppServiceError> {
        self.encounter(encounter_ref)
    }

    fn create_encounter(
        &self,
        request: CreateEncounterRequest,
    ) -> Result<EncounterCreateView, AppServiceError> {
        self.create_encounter(request)
    }

    fn delete_encounter(
        &self,
        encounter_ref: &str,
    ) -> Result<DeleteEncounterView, AppServiceError> {
        self.delete_encounter(encounter_ref)
    }

    fn update_encounter(
        &self,
        request: UpdateEncounterRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.update_encounter(request)
    }

    fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.add_encounter_record_participant(request)
    }

    fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.add_encounter_manual_participant(request)
    }

    fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.update_encounter_participant(encounter_ref, request)
    }

    fn apply_encounter_hp_action(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterHpActionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.apply_encounter_hp_action(encounter_ref, request)
    }

    fn encounter_initiative_options(
        &self,
        encounter_ref: &str,
    ) -> Result<InitiativeOptionsView, AppServiceError> {
        self.encounter_initiative_options(encounter_ref)
    }
    fn roll_encounter_initiative(
        &self,
        encounter_ref: &str,
        request: RollInitiativeRequest,
    ) -> Result<InitiativeRollView, AppServiceError> {
        self.roll_encounter_initiative(encounter_ref, request)
    }

    fn apply_encounter_detection_event(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ApplyEncounterDetectionEventRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.apply_encounter_detection_event(encounter_ref, request)
    }
    fn worsen_encounter_attitude(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::WorsenEncounterAttitudeRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.worsen_encounter_attitude(encounter_ref, request)
    }
    fn encounter_relationships(
        &self,
        encounter_ref: &str,
        subject_key: &str,
        other_key: &str,
    ) -> Result<atlas_app_model::EncounterRelationshipsView, AppServiceError> {
        self.encounter_relationships(encounter_ref, subject_key, other_key)
    }
    fn set_encounter_relationship(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::SetEncounterRelationshipRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.set_encounter_relationship(encounter_ref, request)
    }
    fn preview_encounter_action(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::PreviewEncounterActionRequest,
    ) -> Result<atlas_app_model::EncounterActionPreviewView, AppServiceError> {
        self.preview_encounter_action(encounter_ref, request)
    }
    fn preview_encounter_strike_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::PreviewEncounterStrikeAttemptRequest,
    ) -> Result<atlas_app_model::EncounterStrikeAttemptPreviewView, AppServiceError> {
        self.preview_encounter_strike_attempt(encounter_ref, request)
    }
    fn preview_encounter_casting_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::PreviewEncounterCastingAttemptRequest,
    ) -> Result<atlas_app_model::EncounterCastingPreviewView, AppServiceError> {
        self.preview_encounter_casting_attempt(encounter_ref, request)
    }
    fn begin_encounter_strike_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::BeginEncounterStrikeAttemptRequest,
    ) -> Result<atlas_app_model::EncounterActionMutationView, AppServiceError> {
        self.begin_encounter_strike_attempt(encounter_ref, request)
    }
    fn begin_encounter_casting_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::BeginEncounterCastingAttemptRequest,
    ) -> Result<atlas_app_model::EncounterActionMutationView, AppServiceError> {
        self.begin_encounter_casting_attempt(encounter_ref, request)
    }
    fn resolve_encounter_action_check(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::ResolveEncounterActionCheckRequest,
    ) -> Result<atlas_app_model::EncounterActionMutationView, AppServiceError> {
        self.resolve_encounter_action_check(encounter_ref, request)
    }
    fn abandon_encounter_action_attempt(
        &self,
        encounter_ref: &str,
        request: atlas_app_model::AbandonEncounterActionAttemptRequest,
    ) -> Result<atlas_app_model::EncounterActionMutationView, AppServiceError> {
        self.abandon_encounter_action_attempt(encounter_ref, request)
    }
    fn encounter_strike_attempts(
        &self,
        encounter_ref: &str,
    ) -> Result<atlas_app_model::EncounterStrikeAttemptsView, AppServiceError> {
        self.encounter_strike_attempts(encounter_ref)
    }
    fn encounter_casting_attempts(
        &self,
        encounter_ref: &str,
    ) -> Result<atlas_app_model::EncounterCastingAttemptsView, AppServiceError> {
        self.encounter_casting_attempts(encounter_ref)
    }
    fn apply_encounter_spell_action(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterSpellActionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.apply_encounter_spell_action(encounter_ref, request)
    }

    fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        request: ReorderEncounterParticipantRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.reorder_encounter_participant(encounter_ref, request)
    }

    fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.remove_encounter_participant(encounter_ref, participant_key)
    }

    fn encounter_turn_review(
        &self,
        encounter_ref: &str,
    ) -> Result<EncounterTurnReviewView, AppServiceError> {
        self.encounter_turn_review(encounter_ref)
    }
    fn advance_encounter_turn(
        &self,
        encounter_ref: &str,
        request: AdvanceEncounterTurnRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.advance_encounter_turn(encounter_ref, request)
    }

    fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.set_encounter_turn(request)
    }

    fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.add_encounter_participant_condition(encounter_ref, request)
    }

    fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.update_encounter_participant_condition(encounter_ref, participant_key, request)
    }

    fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
        request: atlas_app_model::RemoveEncounterParticipantConditionRequest,
    ) -> Result<EncounterMutationView, AppServiceError> {
        self.remove_encounter_participant_condition(
            encounter_ref,
            participant_key,
            condition_id,
            request,
        )
    }

    fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError> {
        self.saved_lists()
    }

    fn saved_list(&self, list_ref: &str) -> Result<SavedListDetailView, AppServiceError> {
        self.saved_list(list_ref)
    }

    fn filter_saved_list(
        &self,
        request: FilterSavedListRequest,
    ) -> Result<SavedListDetailView, AppServiceError> {
        self.filter_saved_list(request)
    }

    fn create_saved_list(
        &self,
        request: CreateSavedListRequest,
    ) -> Result<SavedListCreateView, AppServiceError> {
        self.create_saved_list(request)
    }

    fn update_saved_list(
        &self,
        request: UpdateSavedListRequest,
    ) -> Result<SavedListUpdateView, AppServiceError> {
        self.update_saved_list(request)
    }

    fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError> {
        self.add_saved_list_item(request)
    }

    fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError> {
        self.remove_saved_list_item(request)
    }

    fn delete_saved_list(&self, list_ref: &str) -> Result<DeleteSavedListView, AppServiceError> {
        self.delete_saved_list(list_ref)
    }
}

struct AbandonReadOnDrop(ReadExecutionContext);

impl Drop for AbandonReadOnDrop {
    fn drop(&mut self) {
        self.0.abandon();
    }
}

pub(crate) async fn call_read_service<T: Send + 'static>(
    state: AtlasWebState,
    call: impl FnOnce(ReadExecutionContext) -> Result<T, AppServiceError> + Send + 'static,
) -> Result<T, WebError> {
    let execution = ReadExecutionContext::tracked();
    // This guard belongs to the awaiting handler, independently of the blocking
    // work which must retain its permit until it actually finishes.
    let _guard = AbandonReadOnDrop(execution.clone());
    call_service(state, move || {
        execution.check()?;
        call(execution)
    })
    .await
}

pub(crate) async fn call_service<T: Send + 'static>(
    state: AtlasWebState,
    call: impl FnOnce() -> Result<T, AppServiceError> + Send + 'static,
) -> Result<T, WebError> {
    let blocking_call_capacity = state.blocking_call_capacity;
    let permit = state.blocking_calls.try_acquire_owned().map_err(|_| {
        WebError::from(AppServiceError::service_busy(format!(
            "atlas-web blocking service call limit is full; capacity is {blocking_call_capacity}",
        )))
    })?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        call()
    })
    .await
    .map_err(|error| {
        WebError(AppError::new(
            AppErrorCode::InternalError,
            format!("app-service task failed: {error}"),
        ))
    })?
    .map_err(WebError::from)
}
