use std::path::PathBuf;

use atlas_app_model::{
    AddSavedListItemRequest, BatchAddSavedListItemsRequest, BatchSavedListItemMutationView,
    CreateSavedListRequest, DeleteSavedListView, ImportSavedListRequest, ImportSavedListView,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView,
    SavedListExportDocumentView, SavedListIndexView, SavedListItemMutationView,
    SavedListUpdateView, UpdateSavedListRequest,
};
use atlas_app_service::{
    AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions, RawFilterValuesRequest,
};
use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_runtime::AtlasPathMode;
use atlas_search::{
    GraphContextRequest, GraphContextResult, ListRecordsResult, RecordListSort,
    RecordRefResolutionResult, RecordResolutionResult, RemasterLinksResult, SearchPage,
    SimilarRecordRefResult, SimilarScoreWeights, TextSearchResult, TextSearchTuning,
    VariantGroupRefResolutionResult,
};

use super::{AtlasClient, ClientResult};

pub(crate) struct LocalAtlasClient {
    service: AtlasAppService,
    local_state_path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct LocalAtlasClientOptions {
    pub(crate) path_mode: AtlasPathMode,
    pub(crate) index_path: Option<PathBuf>,
    pub(crate) embedding_cache_root: Option<PathBuf>,
    pub(crate) retrieval_mode: AppServiceRetrievalMode,
}

impl LocalAtlasClient {
    pub(crate) fn connect(options: LocalAtlasClientOptions) -> ClientResult<Self> {
        let service = AtlasAppService::start(AtlasAppServiceOptions {
            path_mode: options.path_mode,
            source_root: None,
            embedding_cache_root: options.embedding_cache_root,
            index_path: options.index_path,
            retrieval_mode: options.retrieval_mode,
        })
        .map_err(|error| error.into_app_error())?;
        let local_state_path = service.local_state_path().display().to_string();
        Ok(Self {
            service,
            local_state_path,
        })
    }
}

impl AtlasClient for LocalAtlasClient {
    fn local_state_path(&self) -> Option<&str> {
        Some(&self.local_state_path)
    }

    fn get_records(
        &self,
        record_keys: Vec<RecordKey>,
    ) -> ClientResult<Vec<atlas_record::RetrievedRecord>> {
        self.service
            .get_records(record_keys)
            .map_err(|error| error.into_app_error())
    }

    fn resolve_record(
        &self,
        query: String,
        filter: Option<SearchFilterNode>,
    ) -> ClientResult<Vec<RecordResolutionResult>> {
        self.service
            .resolve_record(query, filter)
            .map_err(|error| error.into_app_error())
    }

    fn resolve_record_ref(
        &self,
        record_ref: String,
        filter: Option<SearchFilterNode>,
    ) -> ClientResult<RecordRefResolutionResult> {
        self.service
            .resolve_record_ref(record_ref, filter)
            .map_err(|error| error.into_app_error())
    }

    fn list_records(
        &self,
        filter: Option<SearchFilterNode>,
        sort: RecordListSort,
        page: SearchPage,
    ) -> ClientResult<ListRecordsResult> {
        self.service
            .list_records(filter, sort, page)
            .map_err(|error| error.into_app_error())
    }

    fn search_text(
        &self,
        query: String,
        exclude: Option<String>,
        filter: Option<SearchFilterNode>,
        page: SearchPage,
        tuning: Option<TextSearchTuning>,
        explain: bool,
    ) -> ClientResult<TextSearchResult> {
        self.service
            .search_text(query, exclude, filter, page, tuning, explain)
            .map_err(|error| error.into_app_error())
    }

    fn similar_records_for_ref(
        &self,
        record_ref: String,
        filter: Option<SearchFilterNode>,
        limit: u32,
        candidate_limit: u32,
        weights: SimilarScoreWeights,
    ) -> ClientResult<SimilarRecordRefResult> {
        self.service
            .similar_records_for_ref(record_ref, filter, limit, candidate_limit, weights)
            .map_err(|error| error.into_app_error())
    }

    fn graph_context(
        &self,
        request: GraphContextRequest,
    ) -> ClientResult<Option<GraphContextResult>> {
        self.service
            .graph_context(request)
            .map_err(|error| error.into_app_error())
    }

    fn resolve_variant_group_ref(
        &self,
        variant_group_ref: String,
    ) -> ClientResult<VariantGroupRefResolutionResult> {
        self.service
            .resolve_variant_group_ref(variant_group_ref)
            .map_err(|error| error.into_app_error())
    }

    fn remaster_links(&self, record_key: RecordKey) -> ClientResult<Option<RemasterLinksResult>> {
        self.service
            .remaster_links(record_key)
            .map_err(|error| error.into_app_error())
    }

    fn discover_raw_filter_fields(
        &self,
        filter: Option<SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
    ) -> ClientResult<FilterFieldDiscovery> {
        self.service
            .discover_raw_filter_fields(filter, filter_json)
            .map_err(|error| error.into_app_error())
    }

    fn discover_raw_filter_values(
        &self,
        request: RawFilterValuesRequest,
    ) -> ClientResult<FilterValueDiscovery> {
        self.service
            .discover_raw_filter_values(request)
            .map_err(|error| error.into_app_error())
    }

    fn saved_lists(&self) -> ClientResult<SavedListIndexView> {
        self.service
            .saved_lists()
            .map_err(|error| error.into_app_error())
    }

    fn saved_list(&self, slug: &str) -> ClientResult<SavedListDetailView> {
        self.service
            .saved_list(slug)
            .map_err(|error| error.into_app_error())
    }

    fn create_saved_list(
        &self,
        request: CreateSavedListRequest,
    ) -> ClientResult<SavedListCreateView> {
        self.service
            .create_saved_list(request)
            .map_err(|error| error.into_app_error())
    }

    fn add_saved_list_item(
        &self,
        request: AddSavedListItemRequest,
    ) -> ClientResult<SavedListItemMutationView> {
        self.service
            .add_saved_list_item(request)
            .map_err(|error| error.into_app_error())
    }

    fn add_saved_list_items(
        &self,
        request: BatchAddSavedListItemsRequest,
    ) -> ClientResult<BatchSavedListItemMutationView> {
        self.service
            .add_saved_list_items(request)
            .map_err(|error| error.into_app_error())
    }

    fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> ClientResult<SavedListItemMutationView> {
        self.service
            .remove_saved_list_item(request)
            .map_err(|error| error.into_app_error())
    }

    fn update_saved_list(
        &self,
        request: UpdateSavedListRequest,
    ) -> ClientResult<SavedListUpdateView> {
        self.service
            .update_saved_list(request)
            .map_err(|error| error.into_app_error())
    }

    fn export_saved_list(&self, slug: &str) -> ClientResult<SavedListExportDocumentView> {
        self.service
            .export_saved_list(slug)
            .map_err(|error| error.into_app_error())
    }

    fn import_saved_list(
        &self,
        request: ImportSavedListRequest,
    ) -> ClientResult<ImportSavedListView> {
        self.service
            .import_saved_list(request)
            .map_err(|error| error.into_app_error())
    }

    fn delete_saved_list(&self, slug: &str) -> ClientResult<DeleteSavedListView> {
        self.service
            .delete_saved_list(slug)
            .map_err(|error| error.into_app_error())
    }
}
