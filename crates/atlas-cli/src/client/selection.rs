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

    fn remove_saved_list_item(
        &self,
        request: atlas_app_model::RemoveSavedListItemRequest,
    ) -> super::ClientResult<atlas_app_model::SavedListItemMutationView> {
        match self {
            Self::Local(client) => client.remove_saved_list_item(request),
            Self::Http(client) => client.remove_saved_list_item(request),
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
