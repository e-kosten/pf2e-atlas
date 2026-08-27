use std::any::type_name;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;

use atlas_domain::RecordKey;
use atlas_index::test_support::{
    RecordRoundTripDiagnosticError, RecordRoundTripPersistedProjectionRow,
    RecordRoundTripRecordRole, RecordRoundTripRetrievalDisposition,
    RecordRoundTripRetrievalRationale, record_round_trip_expected_retrieval_projection,
    record_round_trip_persisted_retrieval_projection,
};
use atlas_index::{RecordReadIndex, SqliteIndexReader};
use atlas_record::AtlasRecord;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::index_build_input::index_build_input;
use crate::source_pipeline;

const SOURCE_ROOT_ENV: &str = "PF2E_SOURCE_ROOT";
const ARTIFACT_PATH_ENV: &str = "ATLAS_RECORD_ROUND_TRIP_ARTIFACT";
const FIELD_REGISTRY: [&str; 11] = [
    "AtlasRecord.identity",
    "AtlasRecord.classification",
    "AtlasRecord.foundry",
    "AtlasRecord.provenance",
    "AtlasRecord.publication",
    "AtlasRecord.requirements",
    "AtlasRecord.timing",
    "AtlasRecord.mechanics",
    "AtlasRecord.content",
    "AtlasRecord.variant",
    "AtlasRecord.visibility",
];
const EXPECTED_POLICY_TUPLE_COUNT: usize = 1_543;
const EXPECTED_CANONICAL_COUNT: usize = 462;
const EXPECTED_SOURCE_INSTANCE_COUNT: usize = 911;
const EXPECTED_DIRECT_ONLY_COUNT: usize = 20;
const EXPECTED_INSPECTION_ONLY_COUNT: usize = 150;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct RecordRoundTripPolicyTuple {
    record_role: RecordRoundTripRecordRole,
    retrieval_disposition: RecordRoundTripRetrievalDisposition,
    retrieval_rationale: RecordRoundTripRetrievalRationale,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
struct RecordRoundTripPolicyCategoryCounts {
    canonical: usize,
    source_instance: usize,
    direct_only: usize,
    inspection_only: usize,
}

impl RecordRoundTripPolicyCategoryCounts {
    fn expected() -> Self {
        Self {
            canonical: EXPECTED_CANONICAL_COUNT,
            source_instance: EXPECTED_SOURCE_INSTANCE_COUNT,
            direct_only: EXPECTED_DIRECT_ONLY_COUNT,
            inspection_only: EXPECTED_INSPECTION_ONLY_COUNT,
        }
    }

    fn total(self) -> usize {
        self.canonical + self.source_instance + self.direct_only + self.inspection_only
    }
}

#[test]
#[ignore]
fn record_key_aligned_round_trip() -> Result<(), Box<dyn Error>> {
    let source_root = required_path(SOURCE_ROOT_ENV)?;
    let artifact_path = required_path(ARTIFACT_PATH_ENV)?;

    let source = source_pipeline::load_foundry_source(&source_root, None)?;
    let input = index_build_input(source);
    let (expected_records, expected_duplicate_keys) = records_by_key(&input.records);
    if !expected_duplicate_keys.is_empty() {
        return emit_duplicate_failure("source", &expected_duplicate_keys);
    }

    let reader = SqliteIndexReader::open_read_only(&artifact_path)?;
    let generation = reader.verified_generation_evidence()?;
    reader.validate_generation_binding()?;

    let expected_projection = match record_round_trip_expected_retrieval_projection(
        &input.records,
        &input.remaster_links,
    ) {
        Ok(projection) => projection,
        Err(error) => return emit_projection_failure("expected", error),
    };
    let persisted_projection = match record_round_trip_persisted_retrieval_projection(&reader) {
        Ok(projection) => projection,
        Err(error) => return emit_projection_failure("persisted", error),
    };
    let hydrated = RecordReadIndex::load_record_set(&reader)?;

    reader.validate_generation_binding()?;

    let (hydrated_records, hydrated_duplicate_keys) = records_by_key(&hydrated.records);
    if !hydrated_duplicate_keys.is_empty() {
        return emit_duplicate_failure("hydrated", &hydrated_duplicate_keys);
    }

    let expected_keys = expected_records.keys().cloned().collect::<BTreeSet<_>>();
    let persisted_keys = persisted_projection
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let hydrated_keys = hydrated_records.keys().cloned().collect::<BTreeSet<_>>();
    let mut mismatches = Vec::new();

    push_key_set_differences(
        &mut mismatches,
        "source",
        &expected_keys,
        "persisted",
        &persisted_keys,
    );
    push_key_set_differences(
        &mut mismatches,
        "source",
        &expected_keys,
        "hydrated",
        &hydrated_keys,
    );

    let mut aligned_record_count = 0usize;
    let mut equal_record_count = 0usize;
    let mut unequal_record_count = 0usize;
    let mut field_comparison_count = 0usize;
    let mut field_difference_count = 0usize;
    for record_key in expected_keys.intersection(&hydrated_keys) {
        aligned_record_count += 1;
        let expected = expected_records[record_key];
        let actual = hydrated_records[record_key];
        if expected == actual {
            equal_record_count += 1;
        } else {
            unequal_record_count += 1;
        }
        field_difference_count += push_record_field_differences(
            &mut mismatches,
            record_key,
            expected,
            actual,
            &mut field_comparison_count,
        );
    }

    let expected_policy = policy_tuples(&expected_projection);
    let persisted_policy = policy_tuples(&persisted_projection);
    let policy_keys = expected_policy
        .keys()
        .chain(persisted_policy.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut policy_difference_count = 0usize;
    for record_key in policy_keys {
        let expected = expected_policy.get(&record_key);
        let actual = persisted_policy.get(&record_key);
        if expected != actual {
            policy_difference_count += 1;
            mismatches.push(json!({
                "kind": "persisted_policy_difference",
                "record_key": record_key,
                "field_path": "RecordRoundTripPolicyTuple",
                "expected": typed_policy_value(expected),
                "actual": typed_policy_value(actual),
            }));
        }
    }

    let expected_key_hash = hash_key_stream(expected_records.keys());
    let persisted_key_hash = hash_key_stream(persisted_projection.keys());
    let hydrated_key_hash = hash_key_stream(hydrated_records.keys());
    let expected_record_hash = hash_record_stream(&expected_records);
    let hydrated_record_hash = hash_record_stream(&hydrated_records);
    let expected_policy_hash = hash_policy_stream(&expected_policy)?;
    let persisted_policy_hash = hash_policy_stream(&persisted_policy)?;
    let field_registry_hash = hash_string_stream(FIELD_REGISTRY);
    let expected_policy_categories = policy_category_counts(&expected_policy);
    let persisted_policy_categories = policy_category_counts(&persisted_policy);
    let required_policy_categories = RecordRoundTripPolicyCategoryCounts::expected();

    let key_hashes_equal =
        expected_key_hash == persisted_key_hash && expected_key_hash == hydrated_key_hash;
    let key_sets_equal =
        expected_keys == persisted_keys && expected_keys == hydrated_keys && key_hashes_equal;
    let records_equal = key_sets_equal
        && aligned_record_count == expected_records.len()
        && equal_record_count == expected_records.len()
        && unequal_record_count == 0
        && field_difference_count == 0
        && field_comparison_count == aligned_record_count * FIELD_REGISTRY.len()
        && expected_record_hash == hydrated_record_hash;
    let policy_closure_equal = expected_policy == persisted_policy
        && expected_policy.len() == EXPECTED_POLICY_TUPLE_COUNT
        && persisted_policy.len() == EXPECTED_POLICY_TUPLE_COUNT
        && expected_policy_categories == required_policy_categories
        && persisted_policy_categories == required_policy_categories
        && expected_policy_categories.total() == EXPECTED_POLICY_TUPLE_COUNT
        && persisted_policy_categories.total() == EXPECTED_POLICY_TUPLE_COUNT
        && policy_difference_count == 0
        && expected_policy_hash == persisted_policy_hash;
    let passed = key_sets_equal && records_equal && policy_closure_equal && mismatches.is_empty();

    mismatches.sort_by(|left, right| mismatch_sort_key(left).cmp(&mismatch_sort_key(right)));
    for mismatch in &mismatches {
        println!("{}", serde_json::to_string(mismatch)?);
    }

    println!(
        "{}",
        serde_json::to_string(&json!({
            "kind": "record_round_trip_summary",
            "status": if passed { "pass" } else { "fail" },
            "generation": generation,
            "duplicate_counts": {
                "source": expected_duplicate_keys.len(),
                "persisted": 0,
                "hydrated": hydrated_duplicate_keys.len(),
            },
            "key_sets": {
                "source_count": expected_records.len(),
                "persisted_count": persisted_projection.len(),
                "hydrated_count": hydrated_records.len(),
                "source_sha256": expected_key_hash,
                "persisted_sha256": persisted_key_hash,
                "hydrated_sha256": hydrated_key_hash,
                "hashes_equal": key_hashes_equal,
                "equal": key_sets_equal,
            },
            "records": {
                "aligned_count": aligned_record_count,
                "equal_count": equal_record_count,
                "unequal_count": unequal_record_count,
                "source_sha256": expected_record_hash,
                "hydrated_sha256": hydrated_record_hash,
                "equal": records_equal,
            },
            "field_registry": {
                "top_level_count": FIELD_REGISTRY.len(),
                "comparison_count": field_comparison_count,
                "difference_count": field_difference_count,
                "sha256": field_registry_hash,
                "complete": FIELD_REGISTRY.len() == 11
                    && field_comparison_count == aligned_record_count * FIELD_REGISTRY.len(),
            },
            "policy": {
                "expected_tuple_count": expected_policy.len(),
                "persisted_tuple_count": persisted_policy.len(),
                "required_tuple_count": EXPECTED_POLICY_TUPLE_COUNT,
                "required_category_counts": required_policy_categories,
                "expected_category_counts": expected_policy_categories,
                "persisted_category_counts": persisted_policy_categories,
                "difference_count": policy_difference_count,
                "expected_sha256": expected_policy_hash,
                "persisted_sha256": persisted_policy_hash,
                "equal": policy_closure_equal,
            },
            "success_row_count": 0,
            "mismatch_row_count": mismatches.len(),
            "summary_row_count": 1,
            "candidate_owner": passed.then_some("atlas-ingest-validation"),
        }))?
    );

    if passed {
        Ok(())
    } else {
        Err("record round-trip diagnostic found mismatches".into())
    }
}

fn required_path(name: &'static str) -> Result<PathBuf, Box<dyn Error>> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("required environment variable `{name}` is not set").into())
}

type RecordMap<'a> = BTreeMap<RecordKey, &'a AtlasRecord>;
type DuplicateKeyCounts = BTreeMap<RecordKey, usize>;

fn records_by_key(records: &[AtlasRecord]) -> (RecordMap<'_>, DuplicateKeyCounts) {
    let mut by_key = BTreeMap::new();
    let mut duplicate_keys = BTreeMap::new();
    for record in records {
        let record_key = record.identity.key.clone();
        if by_key.contains_key(&record_key) {
            *duplicate_keys.entry(record_key).or_insert(1) += 1;
        } else {
            by_key.insert(record_key, record);
        }
    }
    (by_key, duplicate_keys)
}

fn emit_duplicate_failure(
    side: &str,
    duplicate_keys: &DuplicateKeyCounts,
) -> Result<(), Box<dyn Error>> {
    for (record_key, occurrences) in duplicate_keys {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "kind": "duplicate_key",
                "record_key": record_key,
                "field_path": "RecordKey",
                "expected": {
                    "rust_type": "DuplicateFreeRecordKey",
                    "occurrences": 1,
                },
                "actual": {
                    "rust_type": "DuplicateFreeRecordKey",
                    "side": side,
                    "occurrences": occurrences,
                },
            }))?
        );
    }
    println!(
        "{}",
        serde_json::to_string(&json!({
            "kind": "record_round_trip_summary",
            "status": "fail",
            "duplicate_side": side,
            "duplicate_key_count": duplicate_keys.len(),
            "success_row_count": 0,
            "mismatch_row_count": duplicate_keys.len(),
            "summary_row_count": 1,
        }))?
    );
    Err(format!("{side} records contain duplicate RecordKey values").into())
}

fn emit_projection_failure(
    side: &str,
    error: RecordRoundTripDiagnosticError,
) -> Result<(), Box<dyn Error>> {
    let mismatch = match &error {
        RecordRoundTripDiagnosticError::DuplicateRecordKey { record_key } => Some(json!({
            "kind": "duplicate_key",
            "record_key": record_key,
            "field_path": "RecordKey",
            "expected": {
                "rust_type": "DuplicateFreeRecordKey",
                "occurrences": 1,
            },
            "actual": {
                "rust_type": "DuplicateFreeRecordKey",
                "side": side,
                "minimum_occurrences": 2,
            },
        })),
        RecordRoundTripDiagnosticError::UnknownRecordRole { record_key, value } => Some(json!({
            "kind": "persisted_policy_difference",
            "record_key": record_key,
            "field_path": "RecordRoundTripPolicyTuple.record_role",
            "expected": {
                "rust_type": type_name::<RecordRoundTripRecordRole>(),
                "closed_values": ["source", "source_instance", "canonical"],
            },
            "actual": {
                "rust_type": type_name::<String>(),
                "side": side,
                "value": value,
            },
        })),
        RecordRoundTripDiagnosticError::UnknownRetrievalDisposition { record_key, value } => {
            Some(json!({
                "kind": "persisted_policy_difference",
                "record_key": record_key,
                "field_path": "RecordRoundTripPolicyTuple.retrieval_disposition",
                "expected": {
                    "rust_type": type_name::<RecordRoundTripRetrievalDisposition>(),
                    "closed_values": ["ordinary", "direct_only", "inspection_only"],
                },
                "actual": {
                    "rust_type": type_name::<String>(),
                    "side": side,
                    "value": value,
                },
            }))
        }
        RecordRoundTripDiagnosticError::UnknownRetrievalRationale { record_key, value } => {
            Some(json!({
                "kind": "persisted_policy_difference",
                "record_key": record_key,
                "field_path": "RecordRoundTripPolicyTuple.retrieval_rationale",
                "expected": {
                    "rust_type": type_name::<RecordRoundTripRetrievalRationale>(),
                    "closed_values": [
                        "tooling_no_addressable_product_meaning",
                        "canonical_edition_duplicate",
                        "duplicate_source_instance",
                        "generated_canonical",
                        "source_record",
                    ],
                },
                "actual": {
                    "rust_type": type_name::<String>(),
                    "side": side,
                    "value": value,
                },
            }))
        }
        RecordRoundTripDiagnosticError::PersistedProjectionQuery { .. }
        | RecordRoundTripDiagnosticError::InvalidRecordKey { .. } => None,
    };
    let Some(mismatch) = mismatch else {
        return Err(Box::new(error));
    };

    println!("{}", serde_json::to_string(&mismatch)?);
    println!(
        "{}",
        serde_json::to_string(&json!({
            "kind": "record_round_trip_summary",
            "status": "fail",
            "projection_side": side,
            "success_row_count": 0,
            "mismatch_row_count": 1,
            "summary_row_count": 1,
        }))?
    );
    Err(Box::new(error))
}

fn push_key_set_differences(
    mismatches: &mut Vec<Value>,
    expected_name: &str,
    expected: &BTreeSet<RecordKey>,
    actual_name: &str,
    actual: &BTreeSet<RecordKey>,
) {
    for record_key in expected.difference(actual) {
        mismatches.push(json!({
            "kind": "missing_key",
            "record_key": record_key,
            "field_path": "RecordKeySetMembership",
            "expected": {
                "rust_type": "RecordKeySetMembership",
                "set": expected_name,
                "present": true,
            },
            "actual": {
                "rust_type": "RecordKeySetMembership",
                "set": actual_name,
                "present": false,
            },
        }));
    }
    for record_key in actual.difference(expected) {
        mismatches.push(json!({
            "kind": "extra_key",
            "record_key": record_key,
            "field_path": "RecordKeySetMembership",
            "expected": {
                "rust_type": "RecordKeySetMembership",
                "set": expected_name,
                "present": false,
            },
            "actual": {
                "rust_type": "RecordKeySetMembership",
                "set": actual_name,
                "present": true,
            },
        }));
    }
}

fn push_record_field_differences(
    mismatches: &mut Vec<Value>,
    record_key: &RecordKey,
    expected: &AtlasRecord,
    actual: &AtlasRecord,
    field_comparison_count: &mut usize,
) -> usize {
    let mut difference_count = 0usize;
    macro_rules! compare_field {
        ($field:ident) => {{
            *field_comparison_count += 1;
            if expected.$field != actual.$field {
                difference_count += 1;
                mismatches.push(json!({
                    "kind": "field_difference",
                    "record_key": record_key,
                    "field_path": concat!("AtlasRecord.", stringify!($field)),
                    "expected": typed_debug_value(&expected.$field),
                    "actual": typed_debug_value(&actual.$field),
                }));
            }
        }};
    }

    compare_field!(identity);
    compare_field!(classification);
    compare_field!(foundry);
    compare_field!(provenance);
    compare_field!(publication);
    compare_field!(requirements);
    compare_field!(timing);
    compare_field!(mechanics);
    compare_field!(content);
    compare_field!(variant);
    compare_field!(visibility);
    difference_count
}

fn typed_debug_value<T: Debug>(value: &T) -> Value {
    json!({
        "rust_type": type_name::<T>(),
        "debug": format!("{value:#?}"),
    })
}

fn mismatch_sort_key(value: &Value) -> (&str, &str, &str) {
    let record_key = value
        .get("record_key")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let field_path = value
        .get("field_path")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default();
    (record_key, field_path, kind)
}

fn policy_tuples(
    projection: &BTreeMap<RecordKey, RecordRoundTripPersistedProjectionRow>,
) -> BTreeMap<RecordKey, RecordRoundTripPolicyTuple> {
    projection
        .iter()
        .filter_map(|(record_key, row)| {
            let tuple = RecordRoundTripPolicyTuple {
                record_role: row.record_role,
                retrieval_disposition: row.retrieval_disposition,
                retrieval_rationale: row.retrieval_rationale,
            };
            requires_policy_proof(tuple).then(|| (record_key.clone(), tuple))
        })
        .collect()
}

fn requires_policy_proof(tuple: RecordRoundTripPolicyTuple) -> bool {
    tuple.record_role != RecordRoundTripRecordRole::Source
        || tuple.retrieval_disposition != RecordRoundTripRetrievalDisposition::Ordinary
}

fn policy_category_counts(
    policy: &BTreeMap<RecordKey, RecordRoundTripPolicyTuple>,
) -> RecordRoundTripPolicyCategoryCounts {
    let mut counts = RecordRoundTripPolicyCategoryCounts::default();
    for tuple in policy.values() {
        match tuple.record_role {
            RecordRoundTripRecordRole::Canonical => counts.canonical += 1,
            RecordRoundTripRecordRole::SourceInstance => counts.source_instance += 1,
            RecordRoundTripRecordRole::Source => match tuple.retrieval_disposition {
                RecordRoundTripRetrievalDisposition::Ordinary => {}
                RecordRoundTripRetrievalDisposition::DirectOnly => counts.direct_only += 1,
                RecordRoundTripRetrievalDisposition::InspectionOnly => {
                    counts.inspection_only += 1;
                }
            },
        }
    }
    counts
}

fn typed_policy_value(value: Option<&RecordRoundTripPolicyTuple>) -> Value {
    match value {
        Some(value) => json!({
            "rust_type": type_name::<RecordRoundTripPolicyTuple>(),
            "value": value,
        }),
        None => json!({
            "rust_type": type_name::<RecordRoundTripPolicyTuple>(),
            "value": null,
        }),
    }
}

fn hash_key_stream<'a>(keys: impl Iterator<Item = &'a RecordKey>) -> String {
    let mut hasher = Sha256::new();
    for record_key in keys {
        hash_frame(&mut hasher, &record_key.to_string());
    }
    format!("{:x}", hasher.finalize())
}

fn hash_record_stream(records: &RecordMap<'_>) -> String {
    let mut hasher = Sha256::new();
    for (record_key, record) in records {
        hash_frame(&mut hasher, &record_key.to_string());
        macro_rules! hash_field {
            ($field:ident) => {{
                hash_frame(&mut hasher, concat!("AtlasRecord.", stringify!($field)));
                hash_frame(&mut hasher, &format!("{:#?}", record.$field));
            }};
        }
        hash_field!(identity);
        hash_field!(classification);
        hash_field!(foundry);
        hash_field!(provenance);
        hash_field!(publication);
        hash_field!(requirements);
        hash_field!(timing);
        hash_field!(mechanics);
        hash_field!(content);
        hash_field!(variant);
        hash_field!(visibility);
    }
    format!("{:x}", hasher.finalize())
}

fn hash_policy_stream(
    policy: &BTreeMap<RecordKey, RecordRoundTripPolicyTuple>,
) -> Result<String, serde_json::Error> {
    let mut hasher = Sha256::new();
    for (record_key, tuple) in policy {
        hash_frame(&mut hasher, &record_key.to_string());
        hash_frame(&mut hasher, &serde_json::to_string(tuple)?);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn hash_string_stream<const N: usize>(values: [&str; N]) -> String {
    let mut hasher = Sha256::new();
    for value in values {
        hash_frame(&mut hasher, value);
    }
    format!("{:x}", hasher.finalize())
}

fn hash_frame(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}
