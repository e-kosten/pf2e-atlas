use std::sync::atomic::AtomicU64;
use std::sync::{Mutex, OnceLock};

use atlas_search::test_support::{
    FixtureArtifact, minimal_fixture_retrieval_service_without_embeddings,
};

use crate::service::AppServiceWorker;
use crate::windows::{MAX_RESULT_WINDOWS, ResultWindowStore};

pub(super) struct FixtureWorker {
    pub(super) worker: AppServiceWorker,
    _artifact: FixtureArtifact,
}

pub(super) fn fixture_worker() -> FixtureWorker {
    let _guard = fixture_creation_lock()
        .lock()
        .expect("fixture creation lock should not be poisoned");
    let (retrieval, artifact) = minimal_fixture_retrieval_service_without_embeddings()
        .expect("fixture retrieval service should build");
    FixtureWorker {
        worker: AppServiceWorker {
            retrieval,
            windows: ResultWindowStore::new(MAX_RESULT_WINDOWS),
            next_window_id: AtomicU64::new(1),
        },
        _artifact: artifact,
    }
}

fn fixture_creation_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
