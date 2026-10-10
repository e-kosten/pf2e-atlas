//! Local product composition. Command modules call the concrete app facade.
use atlas_app_model::AppError;
use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_runtime::AtlasPathMode;
use std::path::PathBuf;
pub(crate) struct ClientOptions {
    pub(crate) path_mode: AtlasPathMode,
    pub(crate) index: Option<PathBuf>,
    pub(crate) embedding_cache: Option<PathBuf>,
    pub(crate) retrieval_mode: AppServiceRetrievalMode,
}
pub(crate) fn connect(options: ClientOptions) -> Result<AtlasAppService, AppError> {
    AtlasAppService::start(AtlasAppServiceOptions {
        path_mode: options.path_mode,
        source_root: None,
        index_path: options.index,
        embedding_cache_root: options.embedding_cache,
        retrieval_mode: options.retrieval_mode,
    })
    .map_err(|e| e.into_app_error())
}
