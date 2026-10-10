//! New checked-source fixtures. No normalized-record adapter.
use crate::{
    executor::RetrievalExecutor,
    service::{AtlasAppService, RetrievalBackend},
};
use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntimeOptions};
use std::{
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) struct FixtureWorker {
    pub(super) worker: AtlasAppService,
}
pub(super) fn fixture_worker() -> FixtureWorker {
    fixture_worker_with_workers(1)
}
pub(super) fn fixture_worker_with_workers(count: usize) -> FixtureWorker {
    let _guard = fixture_creation_lock().lock().unwrap();
    fixture_worker_with_executor(RetrievalExecutor::from_fixture_workers(count, 16))
}
pub(super) fn encounter_fixture_worker() -> FixtureWorker {
    fixture_worker()
}
pub(super) fn fixture_worker_with_executor(executor: RetrievalExecutor) -> FixtureWorker {
    FixtureWorker {
        worker: AtlasAppService::new(
            RetrievalBackend::Pooled(executor),
            AtlasRuntimeOptions {
                path_mode: AtlasPathMode::Global,
                overrides: AtlasPathOverrides::default(),
            },
            fixture_local_state_path(),
        )
        .unwrap(),
    }
}
fn fixture_creation_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
pub(super) fn fixture_local_state_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "atlas-app-source-state-{}-{}.sqlite",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
pub(super) fn fixture_retrieval() -> Result<
    (
        atlas_search::AtlasRetrievalService,
        atlas_search::test_support::FixtureArtifact,
    ),
    Box<dyn std::error::Error>,
> {
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;
    Ok(open_source_fixture(
        vec![
            record(
                "actions",
                "Item",
                json!({"_id":"testAction100000","type":"action","name":"Test Action 1","system":{"description":{"value":"<p>Useful action @Check[type:reflex|dc:20]</p>"},"traits":{"value":["attack"]}}}),
            ),
            record(
                "actions",
                "Item",
                json!({"_id":"testAction200000","type":"action","name":"Test Action 2","system":{"description":{"value":"<p>Second useful action</p>"}}}),
            ),
            record(
                "actors",
                "Actor",
                json!({"_id":"testCreature1000","type":"npc","name":"Test Creature 1","system":{"details":{"level":{"value":5},"publicNotes":"<p>Creature prose</p>"},"attributes":{"adjustment":null,"ac":{"value":22},"hp":{"max":60},"speed":{"value":25,"otherSpeeds":[]}},"saves":{"fortitude":{"value":15},"reflex":{"value":12},"will":{"value":12}},"perception":{"mod":13},"abilities":{"str":{"mod":4},"dex":{"mod":3},"con":{"mod":2},"int":{"mod":1},"wis":{"mod":2},"cha":{"mod":0}},"skills":{"athletics":{"base":9},"stealth":{"base":8},"arcana":{"base":7,"special":[{"base":11,"label":"Identifying spells","predicate":["identify-magic"]}]}}},"items":[{"_id":"claw000000000000","type":"melee","name":"Claw","system":{"bonus":{"value":12},"damageRolls":{"main":{"damage":"1d6+4","damageType":"slashing"}},"description":{"value":"<p>Claw prose</p>"}}},{"_id":"breath0000000000","type":"action","name":"Breath Weapon","system":{"description":{"value":"<h2>Ghoul Fever</h2><p>Affliction stages</p>"}}}]}),
            ),
            record(
                "hazards",
                "Actor",
                json!({"_id":"testHazard100000","type":"hazard","name":"Test Hazard 1","system":{"details":{"level":{"value":3},"description":"<p>Trap</p>"},"attributes":{"ac":{"value":18},"hp":{"max":40},"hardness":8,"stealth":{"value":12}},"saves":{"reflex":{"value":9}}},"items":[]}),
            ),
        ],
        false,
    )?)
}
