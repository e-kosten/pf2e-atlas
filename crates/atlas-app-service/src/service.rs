use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, MutexGuard};

use atlas_app_model::{AppErrorCode, AppReadinessStatus, AppReadinessView};
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::AtlasRetrievalService;

use crate::error::{AppServiceError, AppServiceResult};
use crate::executor::{RetrievalExecutor, open_retrieval_service_no_embeddings};
use crate::windows::{MAX_RESULT_WINDOWS, ResultWindowStore};

#[derive(Clone)]
pub struct AtlasAppService {
    pub(super) retrieval: RetrievalBackend,
    pub(super) runtime_options: AtlasRuntimeOptions,
    pub(super) local_state_path: PathBuf,
    pub(super) windows: Arc<Mutex<ResultWindowStore>>,
    pub(super) next_window_id: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
pub struct AtlasAppServiceOptions {
    pub path_mode: AtlasPathMode,
    pub source_root: Option<PathBuf>,
    pub embedding_cache_root: Option<PathBuf>,
    pub index_path: Option<PathBuf>,
    pub retrieval_mode: AppServiceRetrievalMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppServiceRetrievalMode {
    FullPool,
    OnDemandNoEmbeddings,
}

impl AtlasAppService {
    pub fn start(options: AtlasAppServiceOptions) -> AppServiceResult<Self> {
        let runtime_options = runtime_options(&options);
        let runtime = AtlasRuntime::resolve(runtime_options.clone())?;
        let retrieval = match options.retrieval_mode {
            AppServiceRetrievalMode::FullPool => {
                RetrievalBackend::Pooled(RetrievalExecutor::start(runtime_options.clone())?)
            }
            AppServiceRetrievalMode::OnDemandNoEmbeddings => RetrievalBackend::OnDemandNoEmbeddings,
        };
        Self::new(
            retrieval,
            runtime_options,
            runtime.local_state_path().to_path_buf(),
        )
    }

    pub fn readiness(&self) -> AppReadinessView {
        AppReadinessView {
            status: AppReadinessStatus::Ready,
            message: "Atlas web service is ready".to_string(),
        }
    }

    pub fn local_state_path(&self) -> &std::path::Path {
        &self.local_state_path
    }

    pub(super) fn new(
        retrieval: RetrievalBackend,
        runtime_options: AtlasRuntimeOptions,
        local_state_path: PathBuf,
    ) -> AppServiceResult<Self> {
        Ok(Self {
            retrieval,
            runtime_options,
            local_state_path,
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

    pub(super) fn submit_retrieval<T>(
        &self,
        task: impl FnOnce(&mut AtlasRetrievalService) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<T>
    where
        T: Send + 'static,
    {
        match &self.retrieval {
            RetrievalBackend::Pooled(executor) => executor.submit(task),
            RetrievalBackend::OnDemandNoEmbeddings => {
                let mut retrieval =
                    open_retrieval_service_no_embeddings(self.runtime_options.clone())?;
                task(&mut retrieval)
            }
        }
    }
}

impl Default for AtlasAppServiceOptions {
    fn default() -> Self {
        Self {
            path_mode: AtlasPathMode::Global,
            source_root: None,
            embedding_cache_root: None,
            index_path: None,
            retrieval_mode: AppServiceRetrievalMode::FullPool,
        }
    }
}

fn runtime_options(options: &AtlasAppServiceOptions) -> AtlasRuntimeOptions {
    AtlasRuntimeOptions {
        path_mode: options.path_mode,
        overrides: AtlasPathOverrides {
            source_root: options.source_root.clone(),
            embedding_cache_root: options.embedding_cache_root.clone(),
            index_path: options.index_path.clone(),
        },
    }
}

#[derive(Clone)]
pub(super) enum RetrievalBackend {
    Pooled(RetrievalExecutor),
    OnDemandNoEmbeddings,
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
            retrieval_mode: AppServiceRetrievalMode::FullPool,
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
