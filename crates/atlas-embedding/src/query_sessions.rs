//! Shared retained native sessions. Runtime composes one owner across SQL workers.
use crate::{EmbeddingError, EmbeddingRuntimeConfig, TextEmbedder, search_profile};
use atlas_domain::SearchProfileId;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicU8, Ordering},
    },
};

#[derive(Clone)]
pub struct SharedQueryInference {
    cache_root: PathBuf,
    sessions: Arc<Mutex<BTreeMap<SearchProfileId, TextEmbedder>>>,
    loaded: Arc<AtomicU8>,
}

impl SharedQueryInference {
    pub fn new(cache_root: impl Into<PathBuf>) -> Self {
        Self {
            cache_root: cache_root.into(),
            sessions: Arc::new(Mutex::new(BTreeMap::new())),
            loaded: Arc::new(AtomicU8::new(0)),
        }
    }
    fn lock(
        &self,
    ) -> Result<MutexGuard<'_, BTreeMap<SearchProfileId, TextEmbedder>>, EmbeddingError> {
        self.sessions.lock().map_err(|e| {
            EmbeddingError::ModelRunFailed(format!("shared inference lock poisoned: {e}"))
        })
    }
    fn load<'a>(
        &self,
        sessions: &'a mut BTreeMap<SearchProfileId, TextEmbedder>,
        id: &SearchProfileId,
    ) -> Result<&'a mut TextEmbedder, EmbeddingError> {
        let spec = search_profile(id)?;
        Ok(match sessions.entry(id.clone()) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let model =
                    TextEmbedder::load(&EmbeddingRuntimeConfig::new(spec.model, &self.cache_root))?;
                let session = entry.insert(model);
                self.loaded
                    .fetch_or(model_flag(spec.model), Ordering::Release);
                session
            }
        })
    }
    pub fn preload(&self, id: &SearchProfileId) -> Result<(), EmbeddingError> {
        let mut sessions = self.lock()?;
        self.load(&mut sessions, id)?;
        Ok(())
    }
    /// The single global lane covers loading and inference, then releases before SQL ranking.
    /// Caller admission is bounded by the runtime's existing retrieval worker queue.
    pub fn embed_query(
        &self,
        id: &SearchProfileId,
        text: &str,
    ) -> Result<Vec<f32>, EmbeddingError> {
        let mut sessions = self.lock()?;
        self.load(&mut sessions, id)?.embed_query(text)
    }
    /// A retained-session snapshot must not wait behind model loading/inference.
    pub fn loaded_profiles(&self) -> Vec<SearchProfileId> {
        let loaded = self.loaded.load(Ordering::Acquire);
        crate::SEARCH_PROFILES
            .iter()
            .filter(|profile| loaded & model_flag(profile.model) != 0)
            .map(|profile| profile.profile_id())
            .collect()
    }
}
fn model_flag(model: crate::EmbeddingModelId) -> u8 {
    match model {
        crate::EmbeddingModelId::BgeBaseEnV15 => 1,
        crate::EmbeddingModelId::EmbeddingGemma300M => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readiness_snapshot_does_not_wait_for_the_native_lane() {
        let inference = SharedQueryInference::new("/missing/cache");
        let worker = inference.clone();
        let native_lane = inference.lock().unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || sender.send(worker.loaded_profiles()).unwrap());
        let observed = receiver.recv_timeout(std::time::Duration::from_secs(2));
        drop(native_lane);
        thread.join().unwrap();
        assert_eq!(observed.unwrap(), Vec::<SearchProfileId>::new());
    }
    #[test]
    fn cloned_workers_share_one_global_inference_lane_without_eager_loads() {
        let inference = SharedQueryInference::new("/missing/cache");
        let worker = inference.clone();
        assert!(Arc::ptr_eq(&inference.sessions, &worker.sessions));
        assert!(inference.loaded_profiles().is_empty());
        let guard = inference.lock().unwrap();
        assert!(worker.sessions.try_lock().is_err());
        drop(guard);
        assert!(worker.sessions.try_lock().is_ok());
    }
    #[test]
    #[ignore = "requires verified local BGE Base/Gemma assets and native runtime"]
    fn actual_shared_sessions_warm_default_load_secondary_once_and_retain_both() {
        let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .map(|p| p.join(".cache/hf-models"))
            .find(|p| {
                p.join(format!(
                    "onnx-community/embeddinggemma-300m-ONNX/{}/onnx/model.onnx_data",
                    crate::embedding_model_spec(crate::EmbeddingModelId::EmbeddingGemma300M)
                        .model_revision
                ))
                .is_file()
            })
            .unwrap();
        let inference = SharedQueryInference::new(cache);
        let base = SearchProfileId::new("bge-base");
        let gemma = SearchProfileId::new("embeddinggemma");
        assert!(inference.loaded_profiles().is_empty());
        inference.preload(&base).unwrap();
        assert_eq!(inference.loaded_profiles(), vec![base.clone()]);
        let clone = inference.clone();
        let before = clone.embed_query(&base, "forest guardian").unwrap();
        clone.preload(&base).unwrap();
        assert_eq!(inference.loaded_profiles().len(), 1);
        clone.embed_query(&gemma, "forest guardian").unwrap();
        assert_eq!(inference.loaded_profiles(), vec![base.clone(), gemma]);
        let after = inference.embed_query(&base, "forest guardian").unwrap();
        assert!(before.iter().zip(after).all(|(a, b)| (a - b).abs() < 2e-5));
        eprintln!(
            "shared sessions: default warm, secondary lazy, both retained, clone shares native lane"
        );
    }
}
