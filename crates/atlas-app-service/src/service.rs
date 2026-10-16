use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use atlas_app_model::{AppErrorCode, AppReadinessStatus, AppReadinessView};
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::AtlasRetrievalService;

use crate::ReadExecutionContext;
use crate::error::{AppServiceError, AppServiceResult};
use crate::executor::RetrievalExecutor;

#[derive(Clone)]
pub struct AtlasAppService {
    pub(super) retrieval: RetrievalBackend,
    pub(super) runtime: Arc<AtlasRuntime>,
    pub(super) local_state_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AtlasAppServiceOptions {
    pub default_search_profile: Option<atlas_domain::SearchProfileId>,
    pub path_mode: AtlasPathMode,
    pub source_root: Option<PathBuf>,
    pub embedding_cache_root: Option<PathBuf>,
    pub index_path: Option<PathBuf>,
    pub retrieval_mode: AppServiceRetrievalMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppServiceRetrievalMode {
    FullPool,
    /// Lazily retain one command-scoped reader and query embedder.
    OnDemandEmbeddings,
    /// Lazily retain one command-scoped reader without query embeddings.
    OnDemandNoEmbeddings,
    /// Lazily retain one command-scoped vector reader without query embeddings.
    OnDemandStoredVectors,
}

impl AtlasAppService {
    pub fn start(options: AtlasAppServiceOptions) -> AppServiceResult<Self> {
        let runtime_options = runtime_options(&options);
        let runtime = Arc::new(AtlasRuntime::resolve(runtime_options)?);
        let retrieval = match options.retrieval_mode {
            AppServiceRetrievalMode::FullPool => {
                RetrievalBackend::Pooled(RetrievalExecutor::start(Arc::clone(&runtime))?)
            }
            AppServiceRetrievalMode::OnDemandEmbeddings => {
                RetrievalBackend::command(CommandRetrievalMode::QueryEmbeddings)
            }
            AppServiceRetrievalMode::OnDemandNoEmbeddings => {
                RetrievalBackend::command(CommandRetrievalMode::NoEmbeddings)
            }
            AppServiceRetrievalMode::OnDemandStoredVectors => {
                RetrievalBackend::command(CommandRetrievalMode::StoredVectors)
            }
        };
        let local_state_path = runtime.local_state_path().to_path_buf();
        Ok(Self {
            retrieval,
            runtime,
            local_state_path,
        })
    }

    pub fn readiness(&self) -> AppReadinessView {
        let metadata = self.with_retrieval(&ReadExecutionContext::uncancellable(), |retrieval| {
            Ok((
                retrieval.artifact_generation()?,
                retrieval.artifact_context().search_profiles.clone(),
                retrieval.default_search_profile().clone(),
            ))
        });
        let generation = metadata
            .as_ref()
            .ok()
            .map(|(generation, _, _)| generation.clone());
        let profiles = metadata
            .as_ref()
            .ok()
            .map(|(_, profiles, _)| profiles.as_slice())
            .unwrap_or(&[]);
        AppReadinessView {
            default_search_profile: metadata
                .as_ref()
                .ok()
                .map(|(_, _, id)| id.clone())
                .or_else(|| self.runtime.requested_search_profile().cloned())
                .unwrap_or_else(|| atlas_domain::SearchProfileId::new("bge-base")),
            artifact_generation: generation,
            search_profiles: self
                .runtime
                .search_profile_readiness(profiles)
                .into_iter()
                .map(|profile| atlas_app_model::SearchProfileReadinessView {
                    id: profile.id,
                    label: profile.label.into(),
                    stored_vectors: profile.stored_vectors,
                    assets_ready: profile.assets_ready,
                    usable: profile.usable,
                })
                .collect(),
            status: if metadata.is_ok() {
                AppReadinessStatus::Ready
            } else {
                AppReadinessStatus::Blocked
            },
            message: metadata.err().map_or_else(
                || "Atlas service is ready".into(),
                |error| error.to_string(),
            ),
        }
    }

    pub fn local_state_path(&self) -> &std::path::Path {
        &self.local_state_path
    }

    #[cfg(test)]
    pub(super) fn new(
        retrieval: RetrievalBackend,
        runtime_options: AtlasRuntimeOptions,
        local_state_path: PathBuf,
    ) -> AppServiceResult<Self> {
        Ok(Self {
            retrieval,
            runtime: Arc::new(AtlasRuntime::resolve(runtime_options)?),
            local_state_path,
        })
    }

    pub(super) fn with_retrieval<T>(
        &self,
        execution: &ReadExecutionContext,
        task: impl FnOnce(&mut AtlasRetrievalService) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<T>
    where
        T: Send + 'static,
    {
        execution.check()?;
        match &self.retrieval {
            RetrievalBackend::Pooled(executor) => executor.submit(execution.clone(), task),
            RetrievalBackend::Command { mode, retrieval } => {
                with_command_reader(*mode, retrieval, &self.runtime, execution, task)
            }
        }
    }
}

fn with_command_reader<T>(
    mode: CommandRetrievalMode,
    retained: &Mutex<Option<AtlasRetrievalService>>,
    runtime: &AtlasRuntime,
    execution: &ReadExecutionContext,
    task: impl FnOnce(&mut AtlasRetrievalService) -> AppServiceResult<T>,
) -> AppServiceResult<T> {
    let mut retained = retained.lock().map_err(|error| {
        AppServiceError::new(
            AppErrorCode::InternalError,
            format!("command retrieval lock is poisoned: {error}"),
        )
    })?;
    execution.check()?;
    let retrieval = match &mut *retained {
        Some(retrieval) => retrieval,
        slot @ None => slot.insert(mode.open(runtime)?),
    };
    execution.check()?;
    task(retrieval)
}

impl Default for AtlasAppServiceOptions {
    fn default() -> Self {
        Self {
            default_search_profile: None,
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
        default_search_profile: options.default_search_profile.clone(),
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
    Command {
        mode: CommandRetrievalMode,
        retrieval: Arc<Mutex<Option<AtlasRetrievalService>>>,
    },
}

impl RetrievalBackend {
    pub(super) fn command(mode: CommandRetrievalMode) -> Self {
        Self::Command {
            mode,
            retrieval: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum CommandRetrievalMode {
    QueryEmbeddings,
    NoEmbeddings,
    StoredVectors,
}

impl CommandRetrievalMode {
    fn open(self, runtime: &AtlasRuntime) -> AppServiceResult<AtlasRetrievalService> {
        match self {
            Self::QueryEmbeddings => Ok(runtime.open_retrieval_service()?),
            Self::NoEmbeddings => Ok(runtime.open_retrieval_service_no_embeddings()?),
            Self::StoredVectors => Ok(runtime.open_retrieval_service_for_stored_vectors()?),
        }
    }
}

#[cfg(test)]
#[path = "service_lifetime_tests.rs"]
mod lifetime_tests;

#[cfg(test)]
#[path = "service_abandonment_tests.rs"]
mod abandonment_tests;

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
            default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
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
