use super::*;
use std::sync::Barrier;

fn epoch(revision: u64) -> Epoch {
    Epoch {
        workspace: [1; 32],
        revision,
        context: [2; 32],
    }
}
fn key(store: &NativeNeighborReuse, revision: u64, value: usize) -> ReuseKey {
    store.encode(epoch(revision), &value).unwrap()
}
fn neighbors() -> SourceVectorNeighbors {
    SourceVectorNeighbors::fixture(&[(7, 0.25f64.to_bits())])
}

#[test]
fn exact_keys_lru_and_entry_limit() {
    let store = NativeNeighborReuse::default();
    store.observe(epoch(1));
    for n in 0..MAX_ENTRIES {
        store.insert(key(&store, 1, n), neighbors());
    }
    assert_eq!(store.stats().ready_entries, MAX_ENTRIES);
    let found = store.get(&key(&store, 1, 0)).unwrap();
    store.insert(key(&store, 1, 64), neighbors());
    assert!(store.get(&key(&store, 1, 0)).is_some());
    assert!(store.get(&key(&store, 1, 1)).is_none());
    let mut collision = key(&store, 1, 100);
    collision.hash = found.key.hash;
    assert!(store.get(&collision).is_none());
    assert!(store.stats().live_bytes <= MAX_BYTES);
}

#[test]
fn pinned_eviction_is_charged_until_the_last_borrow_is_released() {
    let store = NativeNeighborReuse::default();
    store.observe(epoch(1));
    let empty = store.stats().live_bytes;
    store.insert(key(&store, 1, 1), neighbors());
    let borrowed = store.get(&key(&store, 1, 1)).unwrap();
    let retained = store.stats().live_bytes;
    store.observe(epoch(2));
    assert_eq!(store.stats().ready_entries, 0);
    assert_eq!(store.stats().live_bytes, retained);
    drop(borrowed);
    assert_eq!(store.stats().live_bytes, empty);
}

#[test]
fn retired_completion_cannot_roll_back_epoch_or_republish() {
    let store = Arc::new(NativeNeighborReuse::default());
    store.observe(epoch(1));
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker = {
        let store = Arc::clone(&store);
        let entered = Arc::clone(&entered);
        let release = Arc::clone(&release);
        std::thread::spawn(move || {
            let old = key(&store, 1, 1);
            entered.wait();
            release.wait();
            store.observe(epoch(1));
            store.insert(old, neighbors());
        })
    };
    entered.wait();
    store.observe(epoch(2));
    store.insert(key(&store, 2, 2), neighbors());
    release.wait();
    worker.join().unwrap();
    assert!(store.get(&key(&store, 2, 2)).is_some());
    assert!(store.get(&key(&store, 1, 1)).is_none());
    assert_eq!(store.stats().ready_entries, 1);
}

#[test]
fn duplicate_concurrent_misses_compute_independently_and_retain_one_entry() {
    let store = Arc::new(NativeNeighborReuse::default());
    store.observe(epoch(1));
    let both_missed = Arc::new(Barrier::new(2));
    let workers = (0..2)
        .map(|_| {
            let store = Arc::clone(&store);
            let both_missed = Arc::clone(&both_missed);
            std::thread::spawn(move || {
                let key = key(&store, 1, 1);
                assert!(store.get(&key).is_none());
                both_missed.wait();
                store.insert(key, neighbors());
            })
        })
        .collect::<Vec<_>>();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(store.stats().ready_entries, 1);
}

#[test]
fn oversized_keys_entries_and_budget_pressure_bypass_without_growth() {
    let store = NativeNeighborReuse::default();
    store.observe(epoch(1));
    let base = store.stats().live_bytes;
    assert!(store.encode(epoch(1), &"x".repeat(MAX_KEY_BYTES)).is_none());
    assert_eq!(store.stats().live_bytes, base);
    store.insert(
        key(&store, 1, 1),
        SourceVectorNeighbors::fixture(&vec![(1, 0); MAX_ENTRY_BYTES / 16]),
    );
    assert_eq!(store.stats().live_bytes, base);
    let small = NativeNeighborReuse::with_limits(base + MAX_KEY_BYTES * 2 + 1024);
    small.observe(epoch(1));
    let first = key(&small, 1, 1);
    assert!(small.encode(epoch(1), &2).is_none());
    small.insert(first, neighbors());
    assert!(small.stats().peak_bytes <= small.budget.limit);
}

#[test]
fn runtime_shutdown_releases_ready_entries_and_empty_windows_are_retained() {
    let store = NativeNeighborReuse::default();
    store.observe(epoch(1));
    store.insert(key(&store, 1, 1), SourceVectorNeighbors::fixture(&[]));
    let borrowed = store.get(&key(&store, 1, 1)).unwrap();
    let weak = Arc::downgrade(&borrowed);
    drop(borrowed);
    drop(store);
    assert!(weak.upgrade().is_none());
}

#[test]
fn different_workspace_or_same_revision_different_context_cannot_hit() {
    let store = NativeNeighborReuse::default();
    store.observe(epoch(1));
    store.insert(key(&store, 1, 1), neighbors());
    for changed in [
        Epoch {
            workspace: [3; 32],
            ..epoch(1)
        },
        Epoch {
            context: [3; 32],
            ..epoch(1)
        },
    ] {
        store.observe(changed);
        let key = store.encode(changed, &1).unwrap();
        assert!(store.get(&key).is_none());
        store.insert(key, neighbors());
        assert_eq!(store.stats().ready_entries, 1);
    }
    assert!(store.get(&key(&store, 1, 1)).is_some());
}

#[test]
fn semantic_identity_keeps_exact_profile_predicate_text_vector_scope_and_window() {
    let store = NativeNeighborReuse::default();
    store.observe(epoch(1));
    let context = crate::test_support::context(true);
    let profile = &context.search_profiles[0];
    let predicate = atlas_domain::QueryPredicate::boolean(true);
    let vector = [1., 0.];
    fn make(
        store: &NativeNeighborReuse,
        profile: &atlas_index::SourceSearchProfileIdentity,
        predicate: &atlas_domain::QueryPredicate,
        text: &str,
        vector: &[f32],
        content: atlas_domain::SearchContentScope,
        k: usize,
    ) -> ReuseKey {
        store
            .encode(
                epoch(1),
                &KeyInput {
                    recipe: RECIPE,
                    workspace: "fixture",
                    revision: 1,
                    context: epoch(1).context,
                    profile,
                    predicate,
                    text,
                    vector: VectorBits(vector),
                    scope: "all",
                    exclude: None,
                    content,
                    k,
                },
            )
            .unwrap()
    }
    store.insert(
        make(
            &store,
            profile,
            &predicate,
            "Ember",
            &vector,
            atlas_domain::SearchContentScope::All,
            1024,
        ),
        neighbors(),
    );
    assert!(
        store
            .get(&make(
                &store,
                profile,
                &predicate,
                "Ember",
                &vector,
                atlas_domain::SearchContentScope::All,
                1024
            ))
            .is_some()
    );
    let mut revised = profile.clone();
    revised
        .assets
        .insert("additional-verified-model-asset".into(), "c".repeat(64));
    assert!(
        store
            .get(&make(
                &store,
                &revised,
                &predicate,
                "Ember",
                &vector,
                atlas_domain::SearchContentScope::All,
                1024
            ))
            .is_none()
    );
    revised = profile.clone();
    revised.input_recipe.push_str("/revision");
    assert!(
        store
            .get(&make(
                &store,
                &revised,
                &predicate,
                "Ember",
                &vector,
                atlas_domain::SearchContentScope::All,
                1024
            ))
            .is_none()
    );
    let mut clause = predicate.clone();
    clause.clause_id = Some("another-clause".into());
    assert!(
        store
            .get(&make(
                &store,
                profile,
                &clause,
                "Ember",
                &vector,
                atlas_domain::SearchContentScope::All,
                1024
            ))
            .is_none()
    );
    for (text, vector, content, k) in [
        (
            "Ember ",
            [1., 0.],
            atlas_domain::SearchContentScope::All,
            1024,
        ),
        (
            "Ember",
            [1., -0.],
            atlas_domain::SearchContentScope::All,
            1024,
        ),
        (
            "Ember",
            [1., 0.],
            atlas_domain::SearchContentScope::Core,
            1024,
        ),
        (
            "Ember",
            [1., 0.],
            atlas_domain::SearchContentScope::All,
            2048,
        ),
        (
            "Ember",
            [1., 0.],
            atlas_domain::SearchContentScope::All,
            4096,
        ),
    ] {
        assert!(
            store
                .get(&make(
                    &store, profile, &predicate, text, &vector, content, k
                ))
                .is_none()
        );
    }
}
