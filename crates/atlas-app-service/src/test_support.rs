use std::sync::{Mutex, OnceLock};

use crate::executor::RetrievalExecutor;
use crate::service::AtlasAppService;

pub(super) struct FixtureWorker {
    pub(super) worker: AtlasAppService,
}

pub(super) fn fixture_worker() -> FixtureWorker {
    fixture_worker_with_workers(1)
}

pub(super) fn fixture_worker_with_workers(worker_count: usize) -> FixtureWorker {
    let _guard = fixture_creation_lock()
        .lock()
        .expect("fixture creation lock should not be poisoned");
    FixtureWorker {
        worker: AtlasAppService::new(Ok(RetrievalExecutor::from_fixture_workers(
            worker_count,
            16,
        )))
        .expect("fixture service should build"),
    }
}

fn fixture_creation_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
