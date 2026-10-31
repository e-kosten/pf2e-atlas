//! Retained-runtime availability snapshots; native loading still verifies bytes.
use atlas_domain::SearchProfileId;
use atlas_embedding::{EmbeddingRuntimeConfig, required_embedding_model_cache_files};
use std::{collections::BTreeMap, fs, sync::Mutex, time::SystemTime};

#[derive(PartialEq, Eq)]
struct AssetWitness {
    len: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    #[cfg(unix)]
    inode: u64,
    #[cfg(unix)]
    changed: (i64, i64),
}
impl AssetWitness {
    fn read(path: &std::path::Path) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        if !metadata.is_file() {
            return None;
        }
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Some(Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            #[cfg(unix)]
            inode: metadata.ino(),
            #[cfg(unix)]
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        })
    }
}
struct VerifiedPackage {
    witness: Vec<Option<AssetWitness>>,
    ready: bool,
}
#[derive(Default)]
pub(crate) struct ProfileAssets(Mutex<BTreeMap<SearchProfileId, VerifiedPackage>>);
impl ProfileAssets {
    pub(crate) fn ready(&self, id: &SearchProfileId, config: &EmbeddingRuntimeConfig) -> bool {
        self.check(id, config, || {
            let started = std::time::Instant::now();
            let ready = atlas_embedding::validate_embedding_model_cache(config).is_ok();
            // This completed observation must not start an open-ended CLI progress phase.
            tracing::info!(
                phase = "verify_query_assets",
                profile = %id,
                elapsed_seconds = started.elapsed().as_secs_f64(),
                ready,
                "Checked query model assets"
            );
            ready
        })
    }
    fn check(
        &self,
        id: &SearchProfileId,
        config: &EmbeddingRuntimeConfig,
        verify: impl FnOnce() -> bool,
    ) -> bool {
        let witness = required_embedding_model_cache_files(config)
            .iter()
            .map(|file| AssetWitness::read(&file.local_path))
            .collect::<Vec<_>>();
        let Ok(mut packages) = self.0.lock() else {
            return false;
        };
        if let Some(package) = packages.get(id)
            && package.witness == witness
        {
            return package.ready;
        }
        let complete = witness.iter().all(Option::is_some);
        let ready = complete && verify();
        // A repair racing verification must be observed again, rather than
        // blessing a new package with the previous files' verification result.
        let after = required_embedding_model_cache_files(config)
            .iter()
            .map(|file| AssetWitness::read(&file.local_path))
            .collect::<Vec<_>>();
        if after != witness {
            return false;
        }
        packages.insert(id.clone(), VerifiedPackage { witness, ready });
        ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, sync::Arc};

    struct EventTargets(Arc<Mutex<Vec<&'static str>>>);

    impl tracing::Subscriber for EventTargets {
        fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
            true
        }

        fn new_span(&self, _attributes: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }

        fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

        fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

        fn event(&self, event: &tracing::Event<'_>) {
            self.0.lock().unwrap().push(event.metadata().target());
        }

        fn enter(&self, _span: &tracing::span::Id) {}

        fn exit(&self, _span: &tracing::span::Id) {}
    }

    #[test]
    fn completed_asset_checks_emit_diagnostics_without_starting_cli_progress() {
        let directory = tempfile::tempdir().unwrap();
        let config = EmbeddingRuntimeConfig::new(
            atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
            directory.path(),
        );
        for file in required_embedding_model_cache_files(&config) {
            fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
            fs::write(file.local_path, b"invalid fixture asset").unwrap();
        }
        let targets = Arc::new(Mutex::new(Vec::new()));
        let subscriber = EventTargets(Arc::clone(&targets));
        tracing::subscriber::with_default(subscriber, || {
            let assets = ProfileAssets::default();
            let id = SearchProfileId::new("bge-base");
            assert!(!assets.ready(&id, &config));
            assert!(!assets.ready(&id, &config));
        });
        assert_eq!(
            *targets.lock().unwrap(),
            vec!["atlas_runtime::profile_assets"],
            "one completed diagnostic, with no atlas_progress event or rehash on a cache hit"
        );
    }

    #[test]
    fn unchanged_packages_do_not_rehash_and_atomic_repair_recovers_unloaded_profile() {
        let directory = tempfile::tempdir().unwrap();
        let config = EmbeddingRuntimeConfig::new(
            atlas_embedding::EmbeddingModelId::EmbeddingGemma300M,
            directory.path(),
        );
        let id = SearchProfileId::new("embeddinggemma");
        let availability = ProfileAssets::default();
        let calls = Cell::new(0);
        let verify = || {
            calls.set(calls.get() + 1);
            true
        };
        assert!(!availability.check(&id, &config, verify));
        assert_eq!(calls.get(), 0, "missing package needs no byte scan");
        for file in required_embedding_model_cache_files(&config) {
            fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
            fs::write(file.local_path, b"fixture asset").unwrap();
        }
        assert!(availability.check(&id, &config, verify));
        assert!(availability.check(&id, &config, verify));
        assert_eq!(calls.get(), 1, "unchanged metadata retains verified result");
        let file = required_embedding_model_cache_files(&config)
            .remove(0)
            .local_path;
        let replacement = file.with_extension("repaired");
        fs::write(&replacement, b"repaired fixture asset").unwrap();
        fs::rename(replacement, &file).unwrap();
        assert!(availability.check(&id, &config, verify));
        assert_eq!(
            calls.get(),
            2,
            "normal atomic repair invalidates the witness"
        );
        fs::remove_file(file).unwrap();
        assert!(!availability.check(&id, &config, verify));
        assert_eq!(calls.get(), 2);
    }
    #[test]
    fn asset_replacement_during_verification_cannot_cache_a_stale_success() {
        let directory = tempfile::tempdir().unwrap();
        let config = EmbeddingRuntimeConfig::new(
            atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
            directory.path(),
        );
        let files = required_embedding_model_cache_files(&config);
        for file in &files {
            fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
            fs::write(&file.local_path, b"original").unwrap();
        }
        let availability = ProfileAssets::default();
        let id = SearchProfileId::new("bge-base");
        assert!(!availability.check(&id, &config, || {
            fs::write(&files[0].local_path, b"changed during verification").unwrap();
            true
        }));
        let verified_again = Cell::new(false);
        assert!(availability.check(&id, &config, || {
            verified_again.set(true);
            true
        }));
        assert!(verified_again.get());
    }

    #[test]
    fn readiness_never_checks_an_unstored_model_package() {
        let directory = tempfile::tempdir().unwrap();
        let runtime = crate::AtlasRuntime::resolve(crate::AtlasRuntimeOptions {
            overrides: crate::AtlasPathOverrides {
                embedding_cache_root: Some(directory.path().into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
        let profile = atlas_index::SourceSearchProfileIdentity::from_spec(
            atlas_embedding::SEARCH_PROFILES[1],
        );
        let rows = runtime.search_profile_readiness(&[profile]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id.as_str(), "embeddinggemma");
        let checked = runtime.profile_assets.0.lock().unwrap();
        assert_eq!(
            checked
                .keys()
                .map(SearchProfileId::as_str)
                .collect::<Vec<_>>(),
            vec!["embeddinggemma"]
        );
    }

    #[test]
    fn checksum_failed_snapshot_is_rechecked_after_normal_replacement() {
        let directory = tempfile::tempdir().unwrap();
        let config = EmbeddingRuntimeConfig::new(
            atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
            directory.path(),
        );
        let files = required_embedding_model_cache_files(&config);
        for file in &files {
            fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
            fs::write(&file.local_path, b"invalid").unwrap();
        }
        let state = ProfileAssets::default();
        let id = SearchProfileId::new("bge-base");
        assert!(!state.check(&id, &config, || false));
        assert!(!state.check(&id, &config, || panic!(
            "unchanged invalid package must not rehash"
        )));
        let replacement = files[0].local_path.with_extension("repaired");
        fs::write(&replacement, b"verified replacement").unwrap();
        fs::rename(replacement, &files[0].local_path).unwrap();
        assert!(state.check(&id, &config, || true));
    }
}
