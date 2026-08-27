use std::any::type_name;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;
use std::time::{Duration, Instant};

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
const CAPTURE_LIMIT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RecordRoundTripStage {
    RequiredEnvironment,
    SourceLoad,
    BuildInput,
    ReaderOpen,
    GenerationBefore,
    PersistedProjection,
    PublicHydration,
    Comparison,
    GenerationAfter,
    Serialization,
    ProcessCapture,
}

impl RecordRoundTripStage {
    const ALL: [Self; 11] = [
        Self::RequiredEnvironment,
        Self::SourceLoad,
        Self::BuildInput,
        Self::ReaderOpen,
        Self::GenerationBefore,
        Self::PersistedProjection,
        Self::PublicHydration,
        Self::Comparison,
        Self::GenerationAfter,
        Self::Serialization,
        Self::ProcessCapture,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecordRoundTripEnvelopeError {
    RequiredEnvironment {
        message: String,
    },
    SourceLoad {
        message: String,
    },
    BuildInput {
        message: String,
    },
    ReaderOpen {
        message: String,
    },
    GenerationBefore {
        message: String,
    },
    PersistedProjection {
        message: String,
    },
    PublicHydration {
        message: String,
    },
    Comparison {
        message: String,
    },
    GenerationAfter {
        message: String,
    },
    Serialization {
        message: String,
    },
    UnstructuredProcessFailure {
        message: String,
        stdout_sha256: String,
        stderr_sha256: String,
    },
}

impl RecordRoundTripEnvelopeError {
    fn for_stage(stage: RecordRoundTripStage, message: impl Into<String>) -> Self {
        let message = message.into();
        match stage {
            RecordRoundTripStage::RequiredEnvironment => Self::RequiredEnvironment { message },
            RecordRoundTripStage::SourceLoad => Self::SourceLoad { message },
            RecordRoundTripStage::BuildInput => Self::BuildInput { message },
            RecordRoundTripStage::ReaderOpen => Self::ReaderOpen { message },
            RecordRoundTripStage::GenerationBefore => Self::GenerationBefore { message },
            RecordRoundTripStage::PersistedProjection => Self::PersistedProjection { message },
            RecordRoundTripStage::PublicHydration => Self::PublicHydration { message },
            RecordRoundTripStage::Comparison => Self::Comparison { message },
            RecordRoundTripStage::GenerationAfter => Self::GenerationAfter { message },
            RecordRoundTripStage::Serialization => Self::Serialization { message },
            RecordRoundTripStage::ProcessCapture => Self::UnstructuredProcessFailure {
                message,
                stdout_sha256: hash_bytes(&[]),
                stderr_sha256: hash_bytes(&[]),
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
struct RecordRoundTripCounters {
    required_environment_count: usize,
    source_load_count: usize,
    build_input_count: usize,
    reader_open_count: usize,
    generation_before_count: usize,
    persisted_projection_count: usize,
    public_hydration_count: usize,
    comparison_count: usize,
    generation_after_count: usize,
    serialization_count: usize,
    process_capture_count: usize,
}

impl RecordRoundTripCounters {
    fn record_stage(&mut self, stage: RecordRoundTripStage) {
        match stage {
            RecordRoundTripStage::RequiredEnvironment => self.required_environment_count += 1,
            RecordRoundTripStage::SourceLoad => self.source_load_count += 1,
            RecordRoundTripStage::BuildInput => self.build_input_count += 1,
            RecordRoundTripStage::ReaderOpen => self.reader_open_count += 1,
            RecordRoundTripStage::GenerationBefore => self.generation_before_count += 1,
            RecordRoundTripStage::PersistedProjection => self.persisted_projection_count += 1,
            RecordRoundTripStage::PublicHydration => self.public_hydration_count += 1,
            RecordRoundTripStage::Comparison => self.comparison_count += 1,
            RecordRoundTripStage::GenerationAfter => self.generation_after_count += 1,
            RecordRoundTripStage::Serialization => self.serialization_count += 1,
            RecordRoundTripStage::ProcessCapture => self.process_capture_count += 1,
        }
    }
}

#[derive(Debug, Clone)]
struct RecordRoundTripFailure {
    stage: RecordRoundTripStage,
    error: RecordRoundTripEnvelopeError,
    counters: RecordRoundTripCounters,
    elapsed_ms: u128,
    details: Vec<Value>,
    evidence: Value,
}

impl RecordRoundTripFailure {
    fn at(
        stage: RecordRoundTripStage,
        message: impl Into<String>,
        counters: &RecordRoundTripCounters,
        started: &Instant,
    ) -> Self {
        Self::with_elapsed(stage, message, counters, started.elapsed())
    }

    fn with_elapsed(
        stage: RecordRoundTripStage,
        message: impl Into<String>,
        counters: &RecordRoundTripCounters,
        elapsed: Duration,
    ) -> Self {
        Self {
            stage,
            error: RecordRoundTripEnvelopeError::for_stage(stage, message),
            counters: counters.clone(),
            elapsed_ms: elapsed.as_millis(),
            details: Vec::new(),
            evidence: json!({}),
        }
    }

    fn with_details(mut self, details: Vec<Value>) -> Self {
        self.details = details;
        self
    }

    fn with_evidence(mut self, evidence: Value) -> Self {
        self.evidence = evidence;
        self
    }

    fn failure_row(&self) -> Value {
        json!({
            "kind": "record_round_trip_failure",
            "stage": self.stage,
            "error": self.error,
            "counters": self.counters,
            "timing": { "elapsed_ms": self.elapsed_ms },
        })
    }

    fn summary_row(&self) -> Value {
        json!({
            "kind": "record_round_trip_summary",
            "status": "fail",
            "stage": self.stage,
            "error": self.error,
            "counters": self.counters,
            "timing": { "elapsed_ms": self.elapsed_ms },
            "evidence": self.evidence,
            "success_row_count": 0,
            "mismatch_row_count": self.details.len(),
            "error_row_count": 1,
            "summary_row_count": 1,
            "candidate_owner": null,
        })
    }
}

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
    let started = Instant::now();
    match run_record_key_aligned_round_trip(&started) {
        Ok(success) => match serde_json::to_string(&success.summary) {
            Ok(summary) => {
                println!("{summary}");
                Ok(())
            }
            Err(error) => {
                let failure = RecordRoundTripFailure::at(
                    RecordRoundTripStage::Serialization,
                    error.to_string(),
                    &success.counters,
                    &started,
                );
                emit_failure_rows(&failure);
                diagnostic_failure_result(&failure)
            }
        },
        Err(failure) => {
            emit_failure_rows(&failure);
            diagnostic_failure_result(&failure)
        }
    }
}

struct RecordRoundTripSuccess {
    summary: Value,
    counters: RecordRoundTripCounters,
}

fn run_record_key_aligned_round_trip(
    started: &Instant,
) -> Result<RecordRoundTripSuccess, RecordRoundTripFailure> {
    let mut counters = RecordRoundTripCounters::default();

    counters.required_environment_count += 1;
    let source_root = required_path(SOURCE_ROOT_ENV).map_err(|message| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::RequiredEnvironment,
            message,
            &counters,
            started,
        )
    })?;
    counters.required_environment_count += 1;
    let artifact_path = required_path(ARTIFACT_PATH_ENV).map_err(|message| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::RequiredEnvironment,
            message,
            &counters,
            started,
        )
    })?;

    counters.source_load_count += 1;
    let source = source_pipeline::load_foundry_source(&source_root, None).map_err(|error| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::SourceLoad,
            error.to_string(),
            &counters,
            started,
        )
    })?;

    counters.build_input_count += 1;
    let input = index_build_input(source);
    let (expected_records, expected_duplicate_keys) = records_by_key(&input.records);
    if !expected_duplicate_keys.is_empty() {
        return Err(duplicate_failure(
            "source",
            &expected_duplicate_keys,
            &counters,
            started,
        ));
    }

    counters.reader_open_count += 1;
    let reader = SqliteIndexReader::open_read_only(&artifact_path).map_err(|error| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::ReaderOpen,
            error.to_string(),
            &counters,
            started,
        )
    })?;

    counters.generation_before_count += 1;
    let generation = reader.verified_generation_evidence().map_err(|error| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::GenerationBefore,
            error.to_string(),
            &counters,
            started,
        )
    })?;
    reader.validate_generation_binding().map_err(|error| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::GenerationBefore,
            error.to_string(),
            &counters,
            started,
        )
    })?;

    let expected_projection = match record_round_trip_expected_retrieval_projection(
        &input.records,
        &input.remaster_links,
    ) {
        Ok(projection) => projection,
        Err(error) => {
            return Err(projection_failure("expected", error, &counters, started));
        }
    };

    counters.persisted_projection_count += 1;
    let persisted_projection = match record_round_trip_persisted_retrieval_projection(&reader) {
        Ok(projection) => projection,
        Err(error) => {
            return Err(projection_failure("persisted", error, &counters, started));
        }
    };

    counters.public_hydration_count += 1;
    let hydrated = RecordReadIndex::load_record_set(&reader).map_err(|error| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::PublicHydration,
            error.to_string(),
            &counters,
            started,
        )
    })?;

    counters.generation_after_count += 1;
    reader.validate_generation_binding().map_err(|error| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::GenerationAfter,
            error.to_string(),
            &counters,
            started,
        )
    })?;

    let (hydrated_records, hydrated_duplicate_keys) = records_by_key(&hydrated.records);
    if !hydrated_duplicate_keys.is_empty() {
        return Err(duplicate_failure(
            "hydrated",
            &hydrated_duplicate_keys,
            &counters,
            started,
        ));
    }

    counters.comparison_count += 1;
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
    let expected_policy_hash = hash_policy_stream(&expected_policy).map_err(|error| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::Comparison,
            error.to_string(),
            &counters,
            started,
        )
    })?;
    let persisted_policy_hash = hash_policy_stream(&persisted_policy).map_err(|error| {
        RecordRoundTripFailure::at(
            RecordRoundTripStage::Comparison,
            error.to_string(),
            &counters,
            started,
        )
    })?;
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
    counters.serialization_count += 1;
    let evidence = json!({
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
    });

    if !passed {
        return Err(RecordRoundTripFailure::at(
            RecordRoundTripStage::Comparison,
            "record round-trip comparison found mismatches",
            &counters,
            started,
        )
        .with_details(mismatches)
        .with_evidence(evidence));
    }

    Ok(RecordRoundTripSuccess {
        summary: json!({
            "kind": "record_round_trip_summary",
            "status": "pass",
            "stage": RecordRoundTripStage::Serialization,
            "counters": counters.clone(),
            "timing": { "elapsed_ms": started.elapsed().as_millis() },
            "evidence": evidence,
            "success_row_count": 0,
            "mismatch_row_count": 0,
            "error_row_count": 0,
            "summary_row_count": 1,
            "candidate_owner": "atlas-ingest-validation",
        }),
        counters,
    })
}

fn required_path(name: &'static str) -> Result<PathBuf, String> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("required environment variable `{name}` is not set"))
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

fn duplicate_details(side: &str, duplicate_keys: &DuplicateKeyCounts) -> Vec<Value> {
    duplicate_keys
        .iter()
        .map(|(record_key, occurrences)| {
            json!({
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
            })
        })
        .collect()
}

fn duplicate_failure(
    side: &str,
    duplicate_keys: &DuplicateKeyCounts,
    counters: &RecordRoundTripCounters,
    started: &Instant,
) -> RecordRoundTripFailure {
    RecordRoundTripFailure::at(
        RecordRoundTripStage::Comparison,
        format!("{side} records contain duplicate RecordKey values"),
        counters,
        started,
    )
    .with_details(duplicate_details(side, duplicate_keys))
    .with_evidence(json!({
        "duplicate_side": side,
        "duplicate_key_count": duplicate_keys.len(),
    }))
}

fn projection_failure_detail(side: &str, error: &RecordRoundTripDiagnosticError) -> Option<Value> {
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
    mismatch
}

fn projection_failure(
    side: &str,
    error: RecordRoundTripDiagnosticError,
    counters: &RecordRoundTripCounters,
    started: &Instant,
) -> RecordRoundTripFailure {
    let stage = if side == "persisted" {
        RecordRoundTripStage::PersistedProjection
    } else {
        RecordRoundTripStage::Comparison
    };
    let details = projection_failure_detail(side, &error)
        .into_iter()
        .collect();
    RecordRoundTripFailure::at(stage, error.to_string(), counters, started)
        .with_details(details)
        .with_evidence(json!({ "projection_side": side }))
}

fn failure_rows(failure: &RecordRoundTripFailure) -> Vec<Value> {
    let mut rows = failure.details.clone();
    rows.push(failure.failure_row());
    rows.push(failure.summary_row());
    rows
}

fn serialize_failure_rows(
    failure: &RecordRoundTripFailure,
) -> Result<Vec<String>, serde_json::Error> {
    failure_rows(failure)
        .iter()
        .map(serde_json::to_string)
        .collect()
}

fn emit_failure_rows(failure: &RecordRoundTripFailure) {
    match serialize_failure_rows(failure) {
        Ok(rows) => {
            for row in rows {
                println!("{row}");
            }
        }
        Err(_) => {
            println!(
                "{}",
                r#"{"kind":"record_round_trip_failure","stage":"serialization","error":{"kind":"serialization","message":"failure envelope serialization failed"},"counters":{},"timing":{"elapsed_ms":0}}"#
            );
            println!(
                "{}",
                r#"{"kind":"record_round_trip_summary","status":"fail","stage":"serialization","error":{"kind":"serialization","message":"failure envelope serialization failed"},"counters":{},"timing":{"elapsed_ms":0},"evidence":{},"success_row_count":0,"mismatch_row_count":0,"error_row_count":1,"summary_row_count":1,"candidate_owner":null}"#
            );
        }
    }
}

fn diagnostic_failure_result(failure: &RecordRoundTripFailure) -> Result<(), Box<dyn Error>> {
    Err(format!(
        "record round-trip diagnostic failed at {:?}: {:?}",
        failure.stage, failure.error
    )
    .into())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RecordRoundTripCaptureDisposition {
    Retained,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RecordRoundTripStreamCapture {
    byte_count: usize,
    sha256: String,
    retained_byte_count: usize,
    retained_sha256: String,
    disposition: RecordRoundTripCaptureDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RecordRoundTripCaptureEvidence {
    stage: RecordRoundTripStage,
    stdout: RecordRoundTripStreamCapture,
    stderr: RecordRoundTripStreamCapture,
    structured_row_count: usize,
    summary_row_count: usize,
    error: Option<RecordRoundTripEnvelopeError>,
}

fn capture_process_output(stdout: &[u8], stderr: &[u8]) -> RecordRoundTripCaptureEvidence {
    let stdout_capture = capture_stream(stdout);
    let stderr_capture = capture_stream(stderr);
    let structured_rows = std::str::from_utf8(stdout)
        .ok()
        .map(|output| {
            output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(serde_json::from_str::<Value>)
                .collect::<Result<Vec<_>, _>>()
        })
        .and_then(Result::ok);
    let structured_row_count = structured_rows.as_ref().map_or(0, Vec::len);
    let summary_row_count = structured_rows.as_ref().map_or(0, |rows| {
        rows.iter()
            .filter(|row| row.get("kind") == Some(&json!("record_round_trip_summary")))
            .count()
    });
    let structured = structured_row_count > 0 && summary_row_count == 1;
    let error = (!structured).then(
        || RecordRoundTripEnvelopeError::UnstructuredProcessFailure {
            message: "diagnostic stdout did not contain one structured summary".to_string(),
            stdout_sha256: stdout_capture.sha256.clone(),
            stderr_sha256: stderr_capture.sha256.clone(),
        },
    );

    RecordRoundTripCaptureEvidence {
        stage: RecordRoundTripStage::ProcessCapture,
        stdout: stdout_capture,
        stderr: stderr_capture,
        structured_row_count,
        summary_row_count,
        error,
    }
}

fn capture_stream(stream: &[u8]) -> RecordRoundTripStreamCapture {
    let retained = &stream[..stream.len().min(CAPTURE_LIMIT_BYTES)];
    RecordRoundTripStreamCapture {
        byte_count: stream.len(),
        sha256: hash_bytes(stream),
        retained_byte_count: retained.len(),
        retained_sha256: hash_bytes(retained),
        disposition: if retained.len() == stream.len() {
            RecordRoundTripCaptureDisposition::Retained
        } else {
            RecordRoundTripCaptureDisposition::Truncated
        },
    }
}

#[test]
fn synthetic_success() {
    assert_smoke_environment_is_unset();
    let mut counters = RecordRoundTripCounters::default();
    for stage in RecordRoundTripStage::ALL {
        counters.record_stage(stage);
    }
    let summary = json!({
        "kind": "record_round_trip_summary",
        "status": "pass",
        "stage": RecordRoundTripStage::Serialization,
        "counters": counters,
        "timing": { "elapsed_ms": 7 },
        "evidence": {
            "key_sets": { "equal": true },
            "records": { "equal": true },
            "policy": { "equal": true },
        },
        "success_row_count": 0,
        "mismatch_row_count": 0,
        "error_row_count": 0,
        "summary_row_count": 1,
        "candidate_owner": "atlas-ingest-validation",
    });
    let line = serde_json::to_string(&summary).expect("synthetic success serializes");
    let persisted: Value = serde_json::from_str(&line).expect("synthetic success persists");

    assert_eq!(persisted["status"], "pass");
    assert_eq!(persisted["summary_row_count"], 1);
    assert_eq!(persisted["mismatch_row_count"], 0);
    assert_eq!(persisted["candidate_owner"], "atlas-ingest-validation");
}

#[test]
fn synthetic_failure_stages() {
    assert_smoke_environment_is_unset();
    for stage in RecordRoundTripStage::ALL {
        let mut counters = RecordRoundTripCounters::default();
        counters.record_stage(stage);
        let failure = RecordRoundTripFailure::with_elapsed(
            stage,
            format!("synthetic {stage:?} failure"),
            &counters,
            Duration::from_millis(7),
        );
        let rows = serialize_failure_rows(&failure).expect("synthetic failure rows serialize");
        assert_eq!(rows.len(), 2);

        let failure_row: Value =
            serde_json::from_str(&rows[0]).expect("synthetic failure row persists");
        let summary_row: Value =
            serde_json::from_str(&rows[1]).expect("synthetic failure summary persists");
        assert_eq!(
            failure_row["stage"],
            serde_json::to_value(stage).expect("stage serializes")
        );
        assert_eq!(failure_row["timing"]["elapsed_ms"], 7);
        assert_eq!(summary_row["status"], "fail");
        assert_eq!(summary_row["summary_row_count"], 1);
        assert_eq!(summary_row["candidate_owner"], Value::Null);
        assert!(diagnostic_failure_result(&failure).is_err());
    }
}

#[test]
fn synthetic_error_persistence() {
    assert_smoke_environment_is_unset();
    let mut counters = RecordRoundTripCounters::default();
    counters.record_stage(RecordRoundTripStage::SourceLoad);
    let failure = RecordRoundTripFailure::with_elapsed(
        RecordRoundTripStage::SourceLoad,
        "synthetic source load failure",
        &counters,
        Duration::from_millis(7),
    );
    let stdout = format!(
        "{}\n",
        serialize_failure_rows(&failure)
            .expect("structured failure rows serialize")
            .join("\n")
    );
    let stderr = b"synthetic stderr\n";
    let structured = capture_process_output(stdout.as_bytes(), stderr);
    assert_eq!(structured.summary_row_count, 1);
    assert_eq!(structured.structured_row_count, 2);
    assert_eq!(structured.stdout.sha256, hash_bytes(stdout.as_bytes()));
    assert_eq!(structured.stderr.sha256, hash_bytes(stderr));
    assert_eq!(
        structured.stdout.disposition,
        RecordRoundTripCaptureDisposition::Retained
    );
    assert!(structured.error.is_none());

    let unstructured_stdout = b"native loader terminated before diagnostic JSON\n";
    let unstructured_stderr = b"synthetic native-library failure\n";
    let unstructured = capture_process_output(unstructured_stdout, unstructured_stderr);
    assert_eq!(unstructured.structured_row_count, 0);
    assert_eq!(unstructured.summary_row_count, 0);
    assert_eq!(unstructured.stdout.sha256, hash_bytes(unstructured_stdout));
    assert_eq!(unstructured.stderr.sha256, hash_bytes(unstructured_stderr));
    assert!(matches!(
        unstructured.error,
        Some(RecordRoundTripEnvelopeError::UnstructuredProcessFailure { .. })
    ));

    let oversized = vec![b'x'; CAPTURE_LIMIT_BYTES + 1];
    let truncated = capture_stream(&oversized);
    assert_eq!(truncated.byte_count, CAPTURE_LIMIT_BYTES + 1);
    assert_eq!(truncated.retained_byte_count, CAPTURE_LIMIT_BYTES);
    assert_eq!(
        truncated.disposition,
        RecordRoundTripCaptureDisposition::Truncated
    );
}

fn assert_smoke_environment_is_unset() {
    for variable in [SOURCE_ROOT_ENV, ARTIFACT_PATH_ENV] {
        assert!(
            env::var_os(variable).is_none(),
            "synthetic smoke requires `{variable}` to be unset"
        );
    }
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

fn hash_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
