use std::sync::Arc;

use atlas_app_model::{
    AddSavedListItemRequest, AppError, AppErrorCode, AppReadinessView, CreateSavedListRequest,
    DeleteSavedListView, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    FilterEditorView, FilterValueListView, OpenResultWindowRequest, ReadResultWindowPageRequest,
    RecordDetailView, RemoveSavedListItemRequest, ResultWindowPage, SavedListCreateView,
    SavedListDetailView, SavedListIndexView, SavedListItemMutationView,
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

    fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError>;

    fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError>;

    fn saved_list(&self, slug: &str) -> Result<SavedListDetailView, AppServiceError>;

    fn create_saved_list(
        &self,
        request: CreateSavedListRequest,
    ) -> Result<SavedListCreateView, AppServiceError>;

    fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError>;

    fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> Result<SavedListItemMutationView, AppServiceError>;

    fn delete_saved_list(&self, slug: &str) -> Result<DeleteSavedListView, AppServiceError>;
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

    fn record_detail(&self, record_key: &str) -> Result<RecordDetailView, AppServiceError> {
        self.record_detail(record_key)
    }

    fn saved_lists(&self) -> Result<SavedListIndexView, AppServiceError> {
        self.saved_lists()
    }

    fn saved_list(&self, slug: &str) -> Result<SavedListDetailView, AppServiceError> {
        self.saved_list(slug)
    }

    fn create_saved_list(
        &self,
        request: CreateSavedListRequest,
    ) -> Result<SavedListCreateView, AppServiceError> {
        self.create_saved_list(request)
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

    fn delete_saved_list(&self, slug: &str) -> Result<DeleteSavedListView, AppServiceError> {
        self.delete_saved_list(slug)
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
