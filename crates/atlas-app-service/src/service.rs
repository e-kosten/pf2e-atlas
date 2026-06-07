use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, MutexGuard};

use atlas_app_model::{AppErrorCode, AppReadinessStatus, AppReadinessView};
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};

use crate::error::{AppServiceError, AppServiceResult};
use crate::executor::RetrievalExecutor;
use crate::windows::{MAX_RESULT_WINDOWS, ResultWindowStore};

#[derive(Clone)]
pub struct AtlasAppService {
    pub(super) retrieval: RetrievalExecutor,
    pub(super) windows: Arc<Mutex<ResultWindowStore>>,
    pub(super) next_window_id: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
pub struct AtlasAppServiceOptions {
    pub path_mode: AtlasPathMode,
    pub source_root: Option<PathBuf>,
    pub embedding_cache_root: Option<PathBuf>,
    pub index_path: Option<PathBuf>,
}

impl AtlasAppService {
    pub fn start(options: AtlasAppServiceOptions) -> AppServiceResult<Self> {
        let runtime_options = runtime_options(options);
        AtlasRuntime::resolve(runtime_options.clone())?;
        Self::new(RetrievalExecutor::start(runtime_options))
    }

    pub fn readiness(&self) -> AppReadinessView {
        AppReadinessView {
            status: AppReadinessStatus::Ready,
            message: "Atlas web service is ready".to_string(),
        }
    }

    pub(super) fn new(retrieval: AppServiceResult<RetrievalExecutor>) -> AppServiceResult<Self> {
        Ok(Self {
            retrieval: retrieval?,
            windows: Arc::new(Mutex::new(ResultWindowStore::new(MAX_RESULT_WINDOWS))),
            next_window_id: Arc::new(AtomicU64::new(1)),
        })
    }

    pub(super) fn windows(&self) -> AppServiceResult<MutexGuard<'_, ResultWindowStore>> {
        self.windows.lock().map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("result-window state lock is poisoned: {error}"),
            )
        })
    }
}

impl Default for AtlasAppServiceOptions {
    fn default() -> Self {
        Self {
            path_mode: AtlasPathMode::Global,
            source_root: None,
            embedding_cache_root: None,
            index_path: None,
        }
    }
}

fn runtime_options(options: AtlasAppServiceOptions) -> AtlasRuntimeOptions {
    AtlasRuntimeOptions {
        path_mode: options.path_mode,
        overrides: AtlasPathOverrides {
            source_root: options.source_root,
            embedding_cache_root: options.embedding_cache_root,
            index_path: options.index_path,
        },
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::AppErrorCode;

    use super::*;

    #[test]
    fn app_service_start_fails_when_artifact_is_unavailable() {
        let missing_path = std::env::temp_dir().join(format!(
            "atlas-app-service-missing-{}-{}.sqlite",
            std::process::id(),
            unique_suffix()
        ));
        let result = AtlasAppService::start(AtlasAppServiceOptions {
            path_mode: AtlasPathMode::Global,
            source_root: None,
            embedding_cache_root: None,
            index_path: Some(missing_path),
        });

        let error = match result {
            Ok(_) => panic!(
                "web app service startup should fail instead of using no-embeddings fallback"
            ),
            Err(error) => error.into_app_error(),
        };
        assert_eq!(error.code, AppErrorCode::IndexUnavailable);
    }

    fn unique_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    }
}
