use super::{
    AtlasClient, ClientResult, HttpAtlasClient, HttpAtlasClientOptions, LocalAtlasClient,
    LocalAtlasClientOptions,
};

pub(crate) enum AtlasClientConfig {
    Local(LocalAtlasClientOptions),
    #[allow(dead_code)]
    Http(HttpAtlasClientOptions),
}

pub(crate) enum AtlasClientHandle {
    Local(LocalAtlasClient),
    Http(HttpAtlasClient),
}

pub(crate) fn connect(config: AtlasClientConfig) -> ClientResult<AtlasClientHandle> {
    match config {
        AtlasClientConfig::Local(options) => Ok(AtlasClientHandle::Local(
            LocalAtlasClient::connect(options)?,
        )),
        AtlasClientConfig::Http(options) => {
            Ok(AtlasClientHandle::Http(HttpAtlasClient::connect(options)))
        }
    }
}

impl AtlasClient for AtlasClientHandle {
    fn local_state_path(&self) -> Option<&str> {
        match self {
            Self::Local(client) => client.local_state_path(),
            Self::Http(client) => client.local_state_path(),
        }
    }

    fn get_records(
        &self,
        record_keys: Vec<atlas_domain::RecordKey>,
    ) -> super::ClientResult<Vec<atlas_record::AtlasRecord>> {
        match self {
            Self::Local(client) => client.get_records(record_keys),
            Self::Http(client) => client.get_records(record_keys),
        }
    }

    fn resolve_record(
        &self,
        query: String,
        filter: Option<atlas_domain::SearchFilterNode>,
    ) -> super::ClientResult<Vec<atlas_search::RecordResolutionResult>> {
        match self {
            Self::Local(client) => client.resolve_record(query, filter),
            Self::Http(client) => client.resolve_record(query, filter),
        }
    }

    fn resolve_record_ref(
        &self,
        record_ref: String,
        filter: Option<atlas_domain::SearchFilterNode>,
    ) -> super::ClientResult<atlas_search::RecordRefResolutionResult> {
        match self {
            Self::Local(client) => client.resolve_record_ref(record_ref, filter),
            Self::Http(client) => client.resolve_record_ref(record_ref, filter),
        }
    }

    fn list_records(
        &self,
        filter: Option<atlas_domain::SearchFilterNode>,
        sort: atlas_search::RecordListSort,
        page: atlas_search::SearchPage,
    ) -> super::ClientResult<atlas_search::ListRecordsResult> {
        match self {
            Self::Local(client) => client.list_records(filter, sort, page),
            Self::Http(client) => client.list_records(filter, sort, page),
        }
    }

    fn search_text(
        &self,
        query: String,
        exclude: Option<String>,
        filter: Option<atlas_domain::SearchFilterNode>,
        page: atlas_search::SearchPage,
        tuning: Option<atlas_search::TextSearchTuning>,
        explain: bool,
    ) -> super::ClientResult<atlas_search::TextSearchResult> {
        match self {
            Self::Local(client) => {
                client.search_text(query, exclude, filter, page, tuning, explain)
            }
            Self::Http(client) => client.search_text(query, exclude, filter, page, tuning, explain),
        }
    }

    fn similar_records_for_ref(
        &self,
        record_ref: String,
        filter: Option<atlas_domain::SearchFilterNode>,
        limit: u32,
        candidate_limit: u32,
        weights: atlas_search::SimilarScoreWeights,
    ) -> super::ClientResult<atlas_search::SimilarRecordRefResult> {
        match self {
            Self::Local(client) => {
                client.similar_records_for_ref(record_ref, filter, limit, candidate_limit, weights)
            }
            Self::Http(client) => {
                client.similar_records_for_ref(record_ref, filter, limit, candidate_limit, weights)
            }
        }
    }

    fn graph_context(
        &self,
        request: atlas_search::GraphContextRequest,
    ) -> super::ClientResult<Option<atlas_search::GraphContextResult>> {
        match self {
            Self::Local(client) => client.graph_context(request),
            Self::Http(client) => client.graph_context(request),
        }
    }

    fn resolve_variant_group_ref(
        &self,
        variant_group_ref: String,
    ) -> super::ClientResult<atlas_search::VariantGroupRefResolutionResult> {
        match self {
            Self::Local(client) => client.resolve_variant_group_ref(variant_group_ref),
            Self::Http(client) => client.resolve_variant_group_ref(variant_group_ref),
        }
    }

    fn remaster_links(
        &self,
        record_key: atlas_domain::RecordKey,
    ) -> super::ClientResult<Option<atlas_search::RemasterLinksResult>> {
        match self {
            Self::Local(client) => client.remaster_links(record_key),
            Self::Http(client) => client.remaster_links(record_key),
        }
    }

    fn discover_raw_filter_fields(
        &self,
        filter: Option<atlas_domain::SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
    ) -> super::ClientResult<atlas_domain::FilterFieldDiscovery> {
        match self {
            Self::Local(client) => client.discover_raw_filter_fields(filter, filter_json),
            Self::Http(client) => client.discover_raw_filter_fields(filter, filter_json),
        }
    }

    fn discover_raw_filter_values(
        &self,
        request: atlas_app_service::RawFilterValuesRequest,
    ) -> super::ClientResult<atlas_domain::FilterValueDiscovery> {
        match self {
            Self::Local(client) => client.discover_raw_filter_values(request),
            Self::Http(client) => client.discover_raw_filter_values(request),
        }
    }

    fn saved_lists(&self) -> super::ClientResult<atlas_app_model::SavedListIndexView> {
        match self {
            Self::Local(client) => client.saved_lists(),
            Self::Http(client) => client.saved_lists(),
        }
    }

    fn saved_list(&self, slug: &str) -> super::ClientResult<atlas_app_model::SavedListDetailView> {
        match self {
            Self::Local(client) => client.saved_list(slug),
            Self::Http(client) => client.saved_list(slug),
        }
    }

    fn create_saved_list(
        &self,
        request: atlas_app_model::CreateSavedListRequest,
    ) -> super::ClientResult<atlas_app_model::SavedListCreateView> {
        match self {
            Self::Local(client) => client.create_saved_list(request),
            Self::Http(client) => client.create_saved_list(request),
        }
    }

    fn add_saved_list_item(
        &self,
        request: atlas_app_model::AddSavedListItemRequest,
    ) -> super::ClientResult<atlas_app_model::SavedListItemMutationView> {
        match self {
            Self::Local(client) => client.add_saved_list_item(request),
            Self::Http(client) => client.add_saved_list_item(request),
        }
    }

    fn add_saved_list_items(
        &self,
        request: atlas_app_model::BatchAddSavedListItemsRequest,
    ) -> super::ClientResult<atlas_app_model::BatchSavedListItemMutationView> {
        match self {
            Self::Local(client) => client.add_saved_list_items(request),
            Self::Http(client) => client.add_saved_list_items(request),
        }
    }

    fn remove_saved_list_item(
        &self,
        request: atlas_app_model::RemoveSavedListItemRequest,
    ) -> super::ClientResult<atlas_app_model::SavedListItemMutationView> {
        match self {
            Self::Local(client) => client.remove_saved_list_item(request),
            Self::Http(client) => client.remove_saved_list_item(request),
        }
    }

    fn update_saved_list(
        &self,
        request: atlas_app_model::UpdateSavedListRequest,
    ) -> super::ClientResult<atlas_app_model::SavedListUpdateView> {
        match self {
            Self::Local(client) => client.update_saved_list(request),
            Self::Http(client) => client.update_saved_list(request),
        }
    }

    fn export_saved_list(
        &self,
        slug: &str,
    ) -> super::ClientResult<atlas_app_model::SavedListExportDocumentView> {
        match self {
            Self::Local(client) => client.export_saved_list(slug),
            Self::Http(client) => client.export_saved_list(slug),
        }
    }

    fn import_saved_list(
        &self,
        request: atlas_app_model::ImportSavedListRequest,
    ) -> super::ClientResult<atlas_app_model::ImportSavedListView> {
        match self {
            Self::Local(client) => client.import_saved_list(request),
            Self::Http(client) => client.import_saved_list(request),
        }
    }

    fn delete_saved_list(
        &self,
        slug: &str,
    ) -> super::ClientResult<atlas_app_model::DeleteSavedListView> {
        match self {
            Self::Local(client) => client.delete_saved_list(slug),
            Self::Http(client) => client.delete_saved_list(slug),
        }
    }
}
