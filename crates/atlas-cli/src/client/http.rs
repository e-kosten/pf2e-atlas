use atlas_app_model::{
    AddSavedListItemRequest, AppError, AppErrorCode, CreateSavedListRequest, DeleteSavedListView,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView, SavedListIndexView,
    SavedListItemMutationView,
};
use atlas_app_service::RawFilterValuesRequest;
use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_search::{
    GraphContextRequest, GraphContextResult, ListRecordsResult, RecordListSort,
    RecordRefResolutionResult, RecordResolutionResult, RemasterLinksResult, SearchPage,
    SimilarRecordRefResult, SimilarScoreWeights, TextSearchResult, TextSearchTuning,
    VariantGroupRefResolutionResult,
};

use super::{AtlasClient, ClientResult};

pub(crate) struct HttpAtlasClient {
    #[allow(dead_code)]
    base_url: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HttpAtlasClientOptions {
    pub(crate) base_url: String,
}

impl HttpAtlasClient {
    pub(crate) fn connect(options: HttpAtlasClientOptions) -> Self {
        Self {
            base_url: options.base_url,
        }
    }
}

impl AtlasClient for HttpAtlasClient {
    fn local_state_path(&self) -> Option<&str> {
        None
    }

    fn get_records(
        &self,
        _record_keys: Vec<RecordKey>,
    ) -> ClientResult<Vec<atlas_record::AtlasRecord>> {
        Err(not_implemented())
    }

    fn resolve_record(
        &self,
        _query: String,
        _filter: Option<SearchFilterNode>,
    ) -> ClientResult<Vec<RecordResolutionResult>> {
        Err(not_implemented())
    }

    fn resolve_record_ref(
        &self,
        _record_ref: String,
        _filter: Option<SearchFilterNode>,
    ) -> ClientResult<RecordRefResolutionResult> {
        Err(not_implemented())
    }

    fn list_records(
        &self,
        _filter: Option<SearchFilterNode>,
        _sort: RecordListSort,
        _page: SearchPage,
    ) -> ClientResult<ListRecordsResult> {
        Err(not_implemented())
    }

    fn search_text(
        &self,
        _query: String,
        _exclude: Option<String>,
        _filter: Option<SearchFilterNode>,
        _page: SearchPage,
        _tuning: Option<TextSearchTuning>,
        _explain: bool,
    ) -> ClientResult<TextSearchResult> {
        Err(not_implemented())
    }

    fn similar_records_for_ref(
        &self,
        _record_ref: String,
        _filter: Option<SearchFilterNode>,
        _limit: u32,
        _candidate_limit: u32,
        _weights: SimilarScoreWeights,
    ) -> ClientResult<SimilarRecordRefResult> {
        Err(not_implemented())
    }

    fn graph_context(
        &self,
        _request: GraphContextRequest,
    ) -> ClientResult<Option<GraphContextResult>> {
        Err(not_implemented())
    }

    fn resolve_variant_group_ref(
        &self,
        _variant_group_ref: String,
    ) -> ClientResult<VariantGroupRefResolutionResult> {
        Err(not_implemented())
    }

    fn remaster_links(&self, _record_key: RecordKey) -> ClientResult<Option<RemasterLinksResult>> {
        Err(not_implemented())
    }

    fn discover_raw_filter_fields(
        &self,
        _filter: Option<SearchFilterNode>,
        _filter_json: Option<serde_json::Value>,
    ) -> ClientResult<FilterFieldDiscovery> {
        Err(not_implemented())
    }

    fn discover_raw_filter_values(
        &self,
        _request: RawFilterValuesRequest,
    ) -> ClientResult<FilterValueDiscovery> {
        Err(not_implemented())
    }

    fn saved_lists(&self) -> ClientResult<SavedListIndexView> {
        Err(not_implemented())
    }

    fn saved_list(&self, _slug: &str) -> ClientResult<SavedListDetailView> {
        Err(not_implemented())
    }

    fn create_saved_list(
        &self,
        _request: CreateSavedListRequest,
    ) -> ClientResult<SavedListCreateView> {
        Err(not_implemented())
    }

    fn add_saved_list_item(
        &self,
        _request: AddSavedListItemRequest,
    ) -> ClientResult<SavedListItemMutationView> {
        Err(not_implemented())
    }

    fn remove_saved_list_item(
        &self,
        _request: RemoveSavedListItemRequest,
    ) -> ClientResult<SavedListItemMutationView> {
        Err(not_implemented())
    }

    fn delete_saved_list(&self, _slug: &str) -> ClientResult<DeleteSavedListView> {
        Err(not_implemented())
    }
}

fn not_implemented() -> AppError {
    AppError::new(
        AppErrorCode::InternalError,
        "remote Atlas HTTP client is not implemented",
    )
}
