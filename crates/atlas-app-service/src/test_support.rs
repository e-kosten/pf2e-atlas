use std::sync::{Mutex, OnceLock};

use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntimeOptions};

use crate::executor::RetrievalExecutor;
use crate::service::{AtlasAppService, RetrievalBackend};

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
        worker: AtlasAppService::new(
            RetrievalBackend::Pooled(RetrievalExecutor::from_fixture_workers(worker_count, 16)),
            fixture_runtime_options(),
            fixture_local_state_path(),
        )
        .expect("fixture service should build"),
    }
}

fn fixture_creation_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub(super) fn fixture_local_state_path() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "atlas-app-service-local-state-{}-{}.sqlite",
        std::process::id(),
        unique_suffix()
    ));
    let _ = std::fs::remove_file(&path);
    path
}

fn fixture_runtime_options() -> AtlasRuntimeOptions {
    AtlasRuntimeOptions {
        path_mode: AtlasPathMode::Global,
        overrides: AtlasPathOverrides::default(),
    }
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos()
}
