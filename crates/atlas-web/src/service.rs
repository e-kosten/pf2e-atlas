use std::sync::Arc;

use atlas_app_model::{AppError, AppErrorCode};
use atlas_app_service::{AppServiceError, AtlasAppService, ReadExecutionContext};
use tokio::sync::Semaphore;

use crate::error::WebError;

const MAX_BLOCKING_SERVICE_CALLS: usize = 64;

#[derive(Clone)]
pub(crate) struct AtlasWebState {
    pub(crate) service: Arc<AtlasAppService>,
    blocking_calls: Arc<Semaphore>,
    blocking_call_capacity: usize,
}

impl AtlasWebState {
    pub(crate) fn new(service: AtlasAppService) -> Self {
        Self::with_blocking_capacity(service, MAX_BLOCKING_SERVICE_CALLS)
    }

    pub(crate) fn with_blocking_capacity(
        service: AtlasAppService,
        blocking_call_capacity: usize,
    ) -> Self {
        Self {
            service: Arc::new(service),
            blocking_calls: Arc::new(Semaphore::new(blocking_call_capacity)),
            blocking_call_capacity,
        }
    }
}

struct AbandonReadOnDrop(ReadExecutionContext);

impl Drop for AbandonReadOnDrop {
    fn drop(&mut self) {
        self.0.abandon();
    }
}

pub(crate) async fn call_read_service<T: Send + 'static>(
    state: AtlasWebState,
    call: impl FnOnce(ReadExecutionContext) -> Result<T, AppServiceError> + Send + 'static,
) -> Result<T, WebError> {
    let execution = ReadExecutionContext::tracked();
    // This guard belongs to the awaiting handler, independently of the blocking
    // work which must retain its permit until it actually finishes.
    let _guard = AbandonReadOnDrop(execution.clone());
    call_service(state, move || {
        execution.check()?;
        call(execution)
    })
    .await
}

pub(crate) async fn call_service<T: Send + 'static>(
    state: AtlasWebState,
    call: impl FnOnce() -> Result<T, AppServiceError> + Send + 'static,
) -> Result<T, WebError> {
    let blocking_call_capacity = state.blocking_call_capacity;
    let permit = state.blocking_calls.try_acquire_owned().map_err(|_| {
        WebError::from(AppServiceError::service_busy(format!(
            "atlas-web blocking service call limit is full; capacity is {blocking_call_capacity}",
        )))
    })?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        call()
    })
    .await
    .map_err(|error| {
        WebError(AppError::new(
            AppErrorCode::InternalError,
            format!("app-service task failed: {error}"),
        ))
    })?
    .map_err(WebError::from)
}
