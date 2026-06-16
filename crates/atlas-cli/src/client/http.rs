use atlas_app_model::{
    AddSavedListItemRequest, AppError, AppErrorCode, CreateSavedListRequest, DeleteSavedListView,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView, SavedListIndexView,
    SavedListItemMutationView,
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
