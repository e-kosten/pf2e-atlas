use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

fn limits() -> EligibilityLimits {
    EligibilityLimits {
        owned_bytes: EligibilityStore::base_bytes() + 16_384,
        ..EligibilityLimits::default()
    }
}
fn insert(store: &mut EligibilityStore, key: u8, ids: &[i64]) -> Option<String> {
    store
        .get_or_try_insert::<_, IndexError>(&key, |writer| {
            for id in ids {
                if !writer.push_id(*id) {
                    break;
                }
            }
            Ok(())
        })
        .unwrap()
        .map(str::to_owned)
}

#[test]
fn exact_integer_json_and_empty_success_are_retained() {
    let mut store = EligibilityStore::new();
    let ids = [
        i64::MIN,
        i64::MAX,
        9_007_199_254_740_993,
        -9_007_199_254_740_993,
        0,
        0,
        -1,
    ];
    assert_eq!(
        insert(&mut store, 0, &ids),
        Some(serde_json::to_string(&ids).unwrap())
    );
    assert_eq!(insert(&mut store, 1, &[]).as_deref(), Some("[]"));
    let before = store.stats();
    assert_eq!(
        store
            .get_or_try_insert::<_, IndexError>(&0u8, |_| panic!("hit must not populate"))
            .unwrap(),
        Some(serde_json::to_string(&ids).unwrap().as_str())
    );
    assert_eq!(store.stats().materializations, before.materializations);
    assert_eq!(store.stats().live, before.live);
    assert_eq!(store.stats().peak, before.peak);
    assert_eq!(store.stats().hits, 1);
}

#[test]
fn row_and_byte_refusals_are_remembered_once_and_do_not_publish_partial_json() {
    for configured in [
        EligibilityLimits {
            rows: 2,
            ..limits()
        },
        EligibilityLimits {
            json_bytes: 5,
            ..limits()
        },
    ] {
        let mut store = EligibilityStore::with_limits(configured);
        let base = store.stats().live;
        assert_eq!(insert(&mut store, 0, &[1, 2, 3]), None);
        assert_eq!(store.stats().ready, 0);
        assert_eq!(store.stats().live, base);
        assert_eq!(
            store
                .get_or_try_insert::<_, IndexError>(&0u8, |_| panic!("bypass must not populate"))
                .unwrap(),
            None
        );
        let stats = store.stats();
        assert_eq!(stats.materializations, 1);
        assert_eq!(stats.oversized, 1);
        assert_eq!(stats.bypasses, 2);
    }
}

#[test]
fn closing_bracket_is_reserved_before_accepting_a_token() {
    let mut store = EligibilityStore::with_limits(EligibilityLimits {
        json_bytes: 3,
        ..limits()
    });
    assert_eq!(insert(&mut store, 0, &[0]).as_deref(), Some("[0]"));
    assert_eq!(insert(&mut store, 1, &[10]), None);
    assert_eq!(store.stats().ready, 1);
}

#[test]
fn oversized_keys_bypass_before_population_and_are_remembered() {
    let mut store = EligibilityStore::with_limits(EligibilityLimits {
        key_bytes: 4,
        ..limits()
    });
    let base = store.stats().live;
    for _ in 0..2 {
        assert_eq!(
            store
                .get_or_try_insert::<_, IndexError>(&"oversized", |_| panic!(
                    "oversized key must not populate"
                ))
                .unwrap(),
            None
        );
    }
    assert_eq!(store.stats().materializations, 0);
    assert_eq!(store.stats().oversized, 1);
    assert_eq!(store.stats().bypasses, 2);
    assert_eq!(store.stats().live, base);
}

#[test]
fn growth_requires_budget_for_old_and_new_allocations_together() {
    // A one-byte key charges 65; the initial 64-byte JSON box charges128;
    // its replacement 128-byte box charges192. Final retained257 would fit,
    // but the construction peak385 must be refused under384 available bytes.
    let maximum = EligibilityStore::base_bytes() + 384;
    let mut store = EligibilityStore::with_limits(EligibilityLimits {
        owned_bytes: maximum,
        ..limits()
    });
    let base = store.stats().live;
    assert_eq!(insert(&mut store, 0, &[1; 40]), None);
    assert_eq!(store.stats().live, base);
    assert!(store.stats().peak <= maximum);
    assert_eq!(store.stats().oversized, 0);
    assert_eq!(store.stats().bypasses, 1);
    assert_eq!(store.stats().materializations, 1);
}

#[test]
fn aggregate_budget_counts_every_ready_key_and_payload_and_preserves_hits() {
    let maximum = EligibilityStore::base_bytes() + 450;
    let mut store = EligibilityStore::with_limits(EligibilityLimits {
        owned_bytes: maximum,
        ..limits()
    });
    let first = insert(&mut store, 0, &[3; 40]).unwrap();
    let retained = store.stats().live;
    assert_eq!(insert(&mut store, 1, &[4; 40]), None);
    assert_eq!(store.stats().live, retained);
    assert_eq!(store.stats().peak, maximum);
    assert_eq!(store.stats().ready, 1);
    assert_eq!(
        store
            .get_or_try_insert::<_, IndexError>(&0u8, |_| panic!(
                "ready hit survives budget bypass"
            ))
            .unwrap(),
        Some(first.as_str())
    );
    assert_eq!(
        store
            .get_or_try_insert::<_, IndexError>(&1u8, |_| panic!("budget refusal is remembered"))
            .unwrap(),
        None
    );
    assert_eq!(store.stats().materializations, 2);
}

#[test]
fn full_ready_metadata_disables_new_setups_while_allowing_existing_hits() {
    let mut store = EligibilityStore::with_limits(EligibilityLimits {
        ready_entries: 2,
        ..limits()
    });
    assert_eq!(insert(&mut store, 0, &[1]).as_deref(), Some("[1]"));
    assert_eq!(insert(&mut store, 1, &[2]).as_deref(), Some("[2]"));
    assert!(store.stats().disabled);
    assert_eq!(
        store
            .get_or_try_insert::<_, IndexError>(&2u8, |_| panic!("metadata full"))
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .get_or_try_insert::<_, IndexError>(&0u8, |_| panic!("ready hit"))
            .unwrap(),
        Some("[1]")
    );
    assert_eq!(store.stats().materializations, 2);
    assert_eq!(store.stats().ready, 2);
}

#[test]
fn full_bypass_metadata_disables_all_further_new_attempts() {
    let mut store = EligibilityStore::with_limits(EligibilityLimits {
        rows: 0,
        bypass_entries: 1,
        ..limits()
    });
    assert_eq!(insert(&mut store, 0, &[1]), None);
    assert!(store.stats().disabled);
    assert_eq!(
        store
            .get_or_try_insert::<_, IndexError>(&1u8, |_| panic!(
                "global disable prevents another attempt"
            ))
            .unwrap(),
        None
    );
    assert_eq!(store.stats().materializations, 1);
}

#[test]
fn callback_error_propagates_even_after_capacity_refusal_and_cleans_partial_state() {
    let mut store = EligibilityStore::with_limits(EligibilityLimits {
        rows: 1,
        ..limits()
    });
    let base = store.stats().live;
    let result = store.get_or_try_insert::<_, IndexError>(&0u8, |writer| {
        assert!(writer.push_id(1));
        assert!(!writer.push_id(2));
        Err(IndexError::Operation("selected SQL error".into()))
    });
    assert!(
        matches!(result, Err(IndexError::Operation(message)) if message == "selected SQL error")
    );
    assert_eq!(store.stats().live, base);
    assert_eq!(store.stats().ready, 0);
    assert_eq!(store.stats().bypasses, 0);
    assert_eq!(insert(&mut store, 0, &[7]).as_deref(), Some("[7]"));
}

#[test]
fn unwind_drops_partial_growth_and_drop_releases_every_accounted_byte() {
    let mut store = EligibilityStore::with_limits(limits());
    let probe = store.accounting_probe();
    let base = probe.live();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _ = store.get_or_try_insert::<_, IndexError>(&0u8, |writer| {
                for id in 0..100 {
                    assert!(writer.push_id(id));
                }
                panic!("population unwound")
            });
        }))
        .is_err()
    );
    assert_eq!(probe.live(), base);
    assert_eq!(store.stats().ready, 0);
    assert!(probe.peak() > base);
    assert_eq!(insert(&mut store, 0, &[8]).as_deref(), Some("[8]"));
    drop(store);
    assert_eq!(probe.live(), 0);
}

#[test]
fn complete_key_equality_rejects_a_matching_digest_with_different_bytes() {
    let mut store = EligibilityStore::with_limits(limits());
    assert_eq!(insert(&mut store, 0, &[1]).as_deref(), Some("[1]"));
    let hash: [u8; 32] = Sha256::digest(b"1").into();
    store.entries[0].as_mut().unwrap().hash = hash;
    assert_eq!(insert(&mut store, 1, &[2]).as_deref(), Some("[2]"));
    assert_eq!(store.stats().hits, 0);
    assert_eq!(store.stats().materializations, 2);
}

#[test]
fn noncapacity_serialization_errors_are_errors_and_do_not_poison_admission() {
    struct Broken;
    impl Serialize for Broken {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("broken complete key"))
        }
    }
    let mut store = EligibilityStore::with_limits(limits());
    let base = store.stats().live;
    let result =
        store.get_or_try_insert::<_, IndexError>(&Broken, |_| panic!("bad key must not populate"));
    assert!(
        matches!(result, Err(IndexError::Invalid(message)) if message.contains("broken complete key"))
    );
    assert_eq!(store.stats().live, base);
    assert_eq!(store.stats().bypasses, 0);
    assert_eq!(insert(&mut store, 0, &[]).as_deref(), Some("[]"));
}

#[test]
fn invalid_json_encoding_returns_an_index_error_without_panicking() {
    let mut store = EligibilityStore::with_limits(limits());
    assert_eq!(insert(&mut store, 0, &[1]).as_deref(), Some("[1]"));
    store.entries[0].as_mut().unwrap().json.bytes[0] = 0xff;
    let result = store.get_or_try_insert::<_, IndexError>(&0u8, |_| panic!("ready entry"));
    assert!(
        matches!(result, Err(IndexError::Invalid(message)) if message.contains("JSON encoding invariant"))
    );
}
