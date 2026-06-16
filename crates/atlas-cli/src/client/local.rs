use std::path::PathBuf;

use atlas_app_model::{
    AddSavedListItemRequest, CreateSavedListRequest, DeleteSavedListView,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView, SavedListIndexView,
    SavedListItemMutationView,
};
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_runtime::AtlasPathMode;

use super::{AtlasClient, ClientResult};

pub(crate) struct LocalAtlasClient {
    service: AtlasAppService,
    local_state_path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct LocalAtlasClientOptions {
    pub(crate) path_mode: AtlasPathMode,
    pub(crate) index_path: Option<PathBuf>,
}

impl LocalAtlasClient {
    pub(crate) fn connect(options: LocalAtlasClientOptions) -> ClientResult<Self> {
        let service = AtlasAppService::start(AtlasAppServiceOptions {
            path_mode: options.path_mode,
            source_root: None,
            embedding_cache_root: None,
            index_path: options.index_path,
            retrieval_mode: AppServiceRetrievalMode::OnDemandNoEmbeddings,
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

    fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> ClientResult<SavedListItemMutationView> {
        self.service
            .remove_saved_list_item(request)
            .map_err(|error| error.into_app_error())
    }

    fn delete_saved_list(&self, slug: &str) -> ClientResult<DeleteSavedListView> {
        self.service
            .delete_saved_list(slug)
            .map_err(|error| error.into_app_error())
    }
}
