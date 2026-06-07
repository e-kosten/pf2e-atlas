use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc;
use std::thread;

use atlas_app_model::{
    AppErrorCode, AppReadinessStatus, AppReadinessView, DiscoverFilterEditorRequest,
    DiscoverFilterValuesRequest, FilterEditorView, FilterValueListView, OpenResultWindowRequest,
    ReadResultWindowPageRequest, RecordDetailView, ResultWindowPage,
};
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::AtlasRetrievalService;

use crate::error::{AppServiceError, AppServiceResult};
use crate::windows::{MAX_RESULT_WINDOWS, ResultWindowStore};

#[derive(Clone)]
pub struct AtlasAppService {
    sender: mpsc::Sender<AppServiceCommand>,
}

#[derive(Debug, Clone)]
pub struct AtlasAppServiceOptions {
    pub path_mode: AtlasPathMode,
    pub source_root: Option<PathBuf>,
    pub embedding_cache_root: Option<PathBuf>,
    pub index_path: Option<PathBuf>,
}

pub(super) struct AppServiceWorker {
    pub(super) retrieval: AtlasRetrievalService,
    pub(super) windows: ResultWindowStore,
    pub(super) next_window_id: AtomicU64,
}

enum AppServiceCommand {
    Readiness {
        reply: mpsc::Sender<AppServiceResult<AppReadinessView>>,
    },
    OpenResultWindow {
        request: OpenResultWindowRequest,
        reply: mpsc::Sender<AppServiceResult<ResultWindowPage>>,
    },
    ReadResultWindowPage {
        window_id: u64,
        request: ReadResultWindowPageRequest,
        reply: mpsc::Sender<AppServiceResult<ResultWindowPage>>,
    },
    RecordDetail {
        record_key: String,
        reply: mpsc::Sender<AppServiceResult<RecordDetailView>>,
    },
    DiscoverFilterEditor {
        request: DiscoverFilterEditorRequest,
        reply: mpsc::Sender<AppServiceResult<FilterEditorView>>,
    },
    DiscoverFilterValues {
        request: DiscoverFilterValuesRequest,
        reply: mpsc::Sender<AppServiceResult<FilterValueListView>>,
    },
}

impl AtlasAppService {
    pub fn start(options: AtlasAppServiceOptions) -> AppServiceResult<Self> {
        let (sender, receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::channel();
        thread::Builder::new()
            .name("atlas-app-service".to_string())
            .spawn(move || {
                let startup = AppServiceWorker::start(options);
                match startup {
                    Ok(mut worker) => {
                        let _ = startup_sender.send(Ok(()));
                        worker.run(receiver);
                    }
                    Err(error) => {
                        let _ = startup_sender.send(Err(error));
                    }
                }
            })
            .map_err(|error| {
                AppServiceError::new(
                    AppErrorCode::InternalError,
                    format!("failed to start app-service worker: {error}"),
                )
            })?;

        startup_receiver.recv().map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("app-service worker failed to report startup: {error}"),
            )
        })??;

        Ok(Self { sender })
    }

    pub fn readiness(&self) -> AppReadinessView {
        match self.call(|reply| AppServiceCommand::Readiness { reply }) {
            Ok(view) => view,
            Err(error) => AppReadinessView {
                status: AppReadinessStatus::Blocked,
                message: error.into_app_error().message,
            },
        }
    }

    pub fn open_result_window(
        &self,
        request: OpenResultWindowRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        self.call(|reply| AppServiceCommand::OpenResultWindow { request, reply })
    }

    pub fn read_result_window_page(
        &self,
        window_id: u64,
        request: ReadResultWindowPageRequest,
    ) -> AppServiceResult<ResultWindowPage> {
        self.call(|reply| AppServiceCommand::ReadResultWindowPage {
            window_id,
            request,
            reply,
        })
    }

    pub fn record_detail(&self, record_key: &str) -> AppServiceResult<RecordDetailView> {
        self.call(|reply| AppServiceCommand::RecordDetail {
            record_key: record_key.to_string(),
            reply,
        })
    }

    pub fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> AppServiceResult<FilterEditorView> {
        self.call(|reply| AppServiceCommand::DiscoverFilterEditor { request, reply })
    }

    pub fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
    ) -> AppServiceResult<FilterValueListView> {
        self.call(|reply| AppServiceCommand::DiscoverFilterValues { request, reply })
    }

    fn call<T>(
        &self,
        build: impl FnOnce(mpsc::Sender<AppServiceResult<T>>) -> AppServiceCommand,
    ) -> AppServiceResult<T> {
        let (reply, receiver) = mpsc::channel();
        self.sender.send(build(reply)).map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("app-service worker is unavailable: {error}"),
            )
        })?;
        receiver.recv().map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("app-service worker dropped response: {error}"),
            )
        })?
    }
}

impl AppServiceWorker {
    fn start(options: AtlasAppServiceOptions) -> AppServiceResult<Self> {
        let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
            path_mode: options.path_mode,
            overrides: AtlasPathOverrides {
                source_root: options.source_root,
                embedding_cache_root: options.embedding_cache_root,
                index_path: options.index_path,
            },
        })?;
        let retrieval = runtime.open_retrieval_service()?;
        Ok(Self {
            retrieval,
            windows: ResultWindowStore::new(MAX_RESULT_WINDOWS),
            next_window_id: AtomicU64::new(1),
        })
    }

    fn run(&mut self, receiver: mpsc::Receiver<AppServiceCommand>) {
        for command in receiver {
            match command {
                AppServiceCommand::Readiness { reply } => {
                    let _ = reply.send(Ok(self.readiness()));
                }
                AppServiceCommand::OpenResultWindow { request, reply } => {
                    let _ = reply.send(self.open_result_window(request));
                }
                AppServiceCommand::ReadResultWindowPage {
                    window_id,
                    request,
                    reply,
                } => {
                    let _ = reply.send(self.read_result_window_page(window_id, request));
                }
                AppServiceCommand::RecordDetail { record_key, reply } => {
                    let _ = reply.send(self.record_detail(&record_key));
                }
                AppServiceCommand::DiscoverFilterEditor { request, reply } => {
                    let _ = reply.send(self.discover_filter_editor(request));
                }
                AppServiceCommand::DiscoverFilterValues { request, reply } => {
                    let _ = reply.send(self.discover_filter_values(request));
                }
            }
        }
    }

    fn readiness(&self) -> AppReadinessView {
        AppReadinessView {
            status: AppReadinessStatus::Ready,
            message: "Atlas web service is ready".to_string(),
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
        }
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
