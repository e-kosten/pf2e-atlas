mod http;
mod local;
mod selection;

use atlas_app_model::{
    AddSavedListItemRequest, AppError, BatchAddSavedListItemsRequest,
    BatchSavedListItemMutationView, CreateSavedListRequest, DeleteSavedListView,
    ImportSavedListRequest, ImportSavedListView, RemoveSavedListItemRequest, SavedListCreateView,
    SavedListDetailView, SavedListExportDocumentView, SavedListIndexView,
    SavedListItemMutationView, SavedListUpdateView, UpdateSavedListRequest,
};
use atlas_app_service::RawFilterValuesRequest;
use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_search::{
    GraphContextRequest, GraphContextResult, ListRecordsResult, RecordListSort,
    RecordRefResolutionResult, RecordResolutionResult, RemasterLinksResult, SearchPage,
    SimilarRecordRefResult, SimilarScoreWeights, TextSearchResult, TextSearchTuning,
    VariantGroupRefResolutionResult,
};

pub(crate) use http::{HttpAtlasClient, HttpAtlasClientOptions};
pub(crate) use local::{LocalAtlasClient, LocalAtlasClientOptions};
pub(crate) use selection::{AtlasClientConfig, AtlasClientHandle, connect};

pub(crate) type ClientResult<T> = Result<T, AppError>;

pub(crate) trait AtlasClient {
    fn local_state_path(&self) -> Option<&str>;

    fn get_records(
        &self,
        record_keys: Vec<RecordKey>,
    ) -> ClientResult<Vec<atlas_record::RetrievedRecord>>;

    fn resolve_record(
        &self,
        query: String,
        filter: Option<SearchFilterNode>,
    ) -> ClientResult<Vec<RecordResolutionResult>>;

    fn resolve_record_ref(
        &self,
        record_ref: String,
        filter: Option<SearchFilterNode>,
    ) -> ClientResult<RecordRefResolutionResult>;

    fn list_records(
        &self,
        filter: Option<SearchFilterNode>,
        sort: RecordListSort,
        page: SearchPage,
    ) -> ClientResult<ListRecordsResult>;

    fn search_text(
        &self,
        query: String,
        exclude: Option<String>,
        filter: Option<SearchFilterNode>,
        page: SearchPage,
        tuning: Option<TextSearchTuning>,
        explain: bool,
    ) -> ClientResult<TextSearchResult>;

    fn similar_records_for_ref(
        &self,
        record_ref: String,
        filter: Option<SearchFilterNode>,
        limit: u32,
        candidate_limit: u32,
        weights: SimilarScoreWeights,
    ) -> ClientResult<SimilarRecordRefResult>;

    fn graph_context(
        &self,
        request: GraphContextRequest,
    ) -> ClientResult<Option<GraphContextResult>>;

    fn resolve_variant_group_ref(
        &self,
        variant_group_ref: String,
    ) -> ClientResult<VariantGroupRefResolutionResult>;

    fn remaster_links(&self, record_key: RecordKey) -> ClientResult<Option<RemasterLinksResult>>;

    fn discover_raw_filter_fields(
        &self,
        filter: Option<SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
    ) -> ClientResult<FilterFieldDiscovery>;

    fn discover_raw_filter_values(
        &self,
        request: RawFilterValuesRequest,
    ) -> ClientResult<FilterValueDiscovery>;

    fn saved_lists(&self) -> ClientResult<SavedListIndexView>;

    fn saved_list(&self, slug: &str) -> ClientResult<SavedListDetailView>;

    fn create_saved_list(
        &self,
        request: CreateSavedListRequest,
    ) -> ClientResult<SavedListCreateView>;

    fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> ClientResult<SavedListItemMutationView>;

    fn add_saved_list_items(
        &self,
        request: BatchAddSavedListItemsRequest,
    ) -> ClientResult<BatchSavedListItemMutationView>;

    fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> ClientResult<SavedListItemMutationView>;

    fn update_saved_list(
        &self,
        request: UpdateSavedListRequest,
    ) -> ClientResult<SavedListUpdateView>;

    fn export_saved_list(&self, slug: &str) -> ClientResult<SavedListExportDocumentView>;

    fn import_saved_list(
        &self,
        request: ImportSavedListRequest,
    ) -> ClientResult<ImportSavedListView>;

    fn delete_saved_list(&self, slug: &str) -> ClientResult<DeleteSavedListView>;
}
