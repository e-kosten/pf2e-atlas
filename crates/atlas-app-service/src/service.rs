use std::path::PathBuf;
use std::sync::Arc;

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
    /// Open bounded command reads and retain shared query inference.
    OnDemandEmbeddings,
    /// Open bounded command reads without query inference.
    OnDemandNoEmbeddings,
    /// Open bounded reads of stored vectors without query inference.
    OnDemandStoredVectors,
}

impl AtlasAppService {
    pub fn start(options: AtlasAppServiceOptions) -> AppServiceResult<Self> {
        let runtime_options = runtime_options(&options);
        let runtime = Arc::new(AtlasRuntime::resolve(runtime_options)?);
        runtime.upgrade_workspace_if_present()?;
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
        Ok(Self { retrieval, runtime })
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

    pub fn workspace_path(&self) -> &std::path::Path {
        self.runtime.index_path()
    }

    #[cfg(test)]
    pub(super) fn last_read_metrics(&self) -> atlas_search::test_support::SourceReadMetrics {
        match &self.retrieval {
            RetrievalBackend::Pooled(executor) => *executor
                .last_read_metrics
                .lock()
                .expect("operation read metrics"),
            RetrievalBackend::Command { .. } => panic!("metrics require fixture executor"),
        }
    }

    #[cfg(test)]
    pub(super) fn new(
        retrieval: RetrievalBackend,
        runtime_options: AtlasRuntimeOptions,
    ) -> AppServiceResult<Self> {
        Ok(Self {
            retrieval,
            runtime: Arc::new(AtlasRuntime::resolve(runtime_options)?),
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
        self.with_workspace(execution, move |workspace| task(workspace.retrieval_mut()))
    }

    pub(crate) fn with_workspace<T>(
        &self,
        execution: &ReadExecutionContext,
        task: impl FnOnce(&mut atlas_runtime::WorkspaceRead) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<T>
    where
        T: Send + 'static,
    {
        self.with_prepared_workspace(execution, None, task)
    }

    pub(crate) fn with_prepared_workspace<T>(
        &self,
        execution: &ReadExecutionContext,
        prepared: Option<atlas_search::PreparedQueryEmbedding>,
        task: impl FnOnce(&mut atlas_runtime::WorkspaceRead) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<T>
    where
        T: Send + 'static,
    {
        execution.check()?;
        match &self.retrieval {
            RetrievalBackend::Pooled(executor) => {
                executor.submit_prepared(execution.clone(), prepared, task)
            }
            RetrievalBackend::Command { .. } => {
                let mut workspace = self.runtime.read_workspace(prepared)?;
                execution.check()?;
                task(&mut workspace)
            }
        }
    }

    pub(crate) fn with_state<T>(
        &self,
        task: impl FnOnce(&atlas_runtime::StateRead) -> AppServiceResult<T>,
    ) -> AppServiceResult<T> {
        let state = self.runtime.read_state()?;
        task(&state)
    }

    pub(crate) fn with_write<T>(
        &self,
        task: impl FnOnce(&atlas_runtime::WorkspaceWrite) -> AppServiceResult<T>,
    ) -> AppServiceResult<T> {
        let write = self.runtime.write_workspace()?;
        let result = task(&write)?;
        write.commit()?;
        Ok(result)
    }

    pub(crate) fn prepare_query(
        &self,
        profile: Option<&atlas_domain::SearchProfileId>,
        text: &str,
        mode: atlas_search::RetrievalMode,
    ) -> AppServiceResult<Option<atlas_search::PreparedQueryEmbedding>> {
        if !mode.uses_vector() || text.trim().is_empty() {
            return Ok(None);
        }
        if matches!(
            self.retrieval,
            RetrievalBackend::Command {
                mode: CommandRetrievalMode::NoEmbeddings | CommandRetrievalMode::StoredVectors
            }
        ) {
            return Err(AppServiceError::new(
                AppErrorCode::VectorReadinessRequired,
                "query embeddings are unavailable in this runtime mode",
            ));
        }
        Ok(Some(self.runtime.prepare_query(profile, text)?))
    }
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
    Command { mode: CommandRetrievalMode },
}
impl RetrievalBackend {
    pub(super) fn command(mode: CommandRetrievalMode) -> Self {
        Self::Command { mode }
    }
}
#[derive(Clone, Copy)]
pub(super) enum CommandRetrievalMode {
    QueryEmbeddings,
    NoEmbeddings,
    StoredVectors,
}

#[cfg(test)]
#[path = "service_lifetime_tests.rs"]
mod lifetime_tests;

#[cfg(test)]
#[path = "service_abandonment_tests.rs"]
mod abandonment_tests;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn app_service_start_without_workspace_reports_blocked_readiness() {
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
            index_path: Some(missing_path.clone()),
            retrieval_mode: AppServiceRetrievalMode::FullPool,
        });

        let service = result.unwrap();
        assert_eq!(service.readiness().status, AppReadinessStatus::Blocked);
        assert!(!missing_path.exists());
        assert!(service.record_detail("items:missing").is_err());
    }

    fn unique_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    }
}
