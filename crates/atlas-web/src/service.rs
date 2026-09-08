use std::sync::Arc;

use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AddSavedListItemRequest, AppError, AppErrorCode,
    AppReadinessView, CreateEncounterRequest, CreateSavedListRequest, DeleteEncounterView,
    DeleteSavedListView, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    EncounterConditionCatalogView, EncounterCreateView, EncounterDetailView, EncounterIndexView,
    EncounterParticipantResetResultView, EncounterParticipantView, EncounterSpellCastRequest,
    EncounterSpellCastResultView, EncounterUpdateView, FilterEditorView, FilterSavedListRequest,
    FilterValueListView, OpenResultWindowRequest, ReadResultWindowPageRequest, RecordDetailRequest,
    RecordDetailView, RemoveSavedListItemRequest, ReorderEncounterParticipantRequest,
    ResetEncounterParticipantRequest, ResultWindowPage, SavedListCreateView, SavedListDetailView,
    SavedListIndexView, SavedListItemMutationView, SavedListUpdateView, SetEncounterTurnRequest,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest, UpdateSavedListRequest,
};
use atlas_app_service::{AppServiceError, AtlasAppService};
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
    fn readiness(&self) -> AppReadinessView;

    fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> Result<FilterEditorView, AppServiceError>;

    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
    ) -> Result<FilterValueListView, AppServiceError>;

    fn open_result_window(
        &self,
        request: OpenResultWindowRequest,
    ) -> Result<ResultWindowPage, AppServiceError>;

    fn read_result_window_page(
        &self,
        window_id: u64,
        request: ReadResultWindowPageRequest,
    ) -> Result<ResultWindowPage, AppServiceError>;

    fn record_detail(
        &self,
        record_key: &str,
        request: RecordDetailRequest,
    ) -> Result<RecordDetailView, AppServiceError>;

    fn encounters(&self) -> Result<EncounterIndexView, AppServiceError>;

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
    ) -> Result<EncounterUpdateView, AppServiceError>;

    fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError>;

    fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError>;

    fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> Result<EncounterParticipantView, AppServiceError>;

    fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        request: ReorderEncounterParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError>;

    fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> Result<EncounterDetailView, AppServiceError>;

    fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> Result<EncounterDetailView, AppServiceError>;

    fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> Result<EncounterDetailView, AppServiceError>;

    fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> Result<EncounterDetailView, AppServiceError>;

    fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
    ) -> Result<EncounterDetailView, AppServiceError>;

    fn mutate_encounter_spell_cast(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: EncounterSpellCastRequest,
    ) -> Result<EncounterSpellCastResultView, AppServiceError>;

    fn reset_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: ResetEncounterParticipantRequest,
    ) -> Result<EncounterParticipantResetResultView, AppServiceError>;

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
    ) -> Result<FilterValueListView, AppServiceError> {
        self.discover_filter_values(request)
    }

    fn open_result_window(
        &self,
        request: OpenResultWindowRequest,
    ) -> Result<ResultWindowPage, AppServiceError> {
        self.open_result_window(request)
    }

    fn read_result_window_page(
        &self,
        window_id: u64,
        request: ReadResultWindowPageRequest,
    ) -> Result<ResultWindowPage, AppServiceError> {
        self.read_result_window_page(window_id, request)
    }

    fn record_detail(
        &self,
        record_key: &str,
        request: RecordDetailRequest,
    ) -> Result<RecordDetailView, AppServiceError> {
        self.record_detail(record_key, request)
    }

    fn encounters(&self) -> Result<EncounterIndexView, AppServiceError> {
        self.encounters()
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
    ) -> Result<EncounterUpdateView, AppServiceError> {
        self.update_encounter(request)
    }

    fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        self.add_encounter_record_participant(request)
    }

    fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        self.add_encounter_manual_participant(request)
    }

    fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> Result<EncounterParticipantView, AppServiceError> {
        self.update_encounter_participant(encounter_ref, request)
    }

    fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        request: ReorderEncounterParticipantRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        self.reorder_encounter_participant(encounter_ref, request)
    }

    fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> Result<EncounterDetailView, AppServiceError> {
        self.remove_encounter_participant(encounter_ref, participant_key)
    }

    fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        self.set_encounter_turn(request)
    }

    fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        self.add_encounter_participant_condition(encounter_ref, request)
    }

    fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> Result<EncounterDetailView, AppServiceError> {
        self.update_encounter_participant_condition(encounter_ref, participant_key, request)
    }

    fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
    ) -> Result<EncounterDetailView, AppServiceError> {
        self.remove_encounter_participant_condition(encounter_ref, participant_key, condition_id)
    }

    fn mutate_encounter_spell_cast(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: EncounterSpellCastRequest,
    ) -> Result<EncounterSpellCastResultView, AppServiceError> {
        self.mutate_encounter_spell_cast(encounter_ref, participant_key, request)
    }

    fn reset_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: ResetEncounterParticipantRequest,
    ) -> Result<EncounterParticipantResetResultView, AppServiceError> {
        self.reset_encounter_participant(encounter_ref, participant_key, request)
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
