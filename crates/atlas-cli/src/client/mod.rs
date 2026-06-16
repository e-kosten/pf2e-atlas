mod http;
mod local;
mod selection;

use atlas_app_model::{
    AddSavedListItemRequest, AppError, CreateSavedListRequest, DeleteSavedListView,
    RemoveSavedListItemRequest, SavedListCreateView, SavedListDetailView, SavedListIndexView,
    SavedListItemMutationView,
};

pub(crate) use http::{HttpAtlasClient, HttpAtlasClientOptions};
pub(crate) use local::{LocalAtlasClient, LocalAtlasClientOptions};
pub(crate) use selection::{AtlasClientConfig, AtlasClientHandle, connect};

pub(crate) type ClientResult<T> = Result<T, AppError>;

pub(crate) trait AtlasClient {
    fn local_state_path(&self) -> Option<&str>;

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

    fn remove_saved_list_item(
        &self,
        request: RemoveSavedListItemRequest,
    ) -> ClientResult<SavedListItemMutationView>;

    fn delete_saved_list(&self, slug: &str) -> ClientResult<DeleteSavedListView>;
}
