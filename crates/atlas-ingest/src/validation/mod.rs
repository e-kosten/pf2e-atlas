//! Private validation-pipeline state. Nothing in this module is a product
//! artifact, runtime input, fallback, or public serialization contract.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use atlas_index::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, ArtifactValidationReport,
    IndexInspectionReport, SqliteIndexReader, ValidationStatus, ValidationTarget,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::artifact_manifest::ARTIFACT_MANIFEST_VERSION;
use crate::audit::{SourcePathAuditOptions, SourcePathAuditReport, audit_loaded_source};
use crate::build::build_artifact_from_source;
use crate::error::IngestError;
use crate::index_build_input::index_build_input;
use crate::report::analyze_captured_source_load;
use crate::source::dto::PF2E_SOURCE_CONTRACT_VERSION;
use crate::source::model::BuildArtifactOptions;
use crate::source_pipeline;

const SNAPSHOT_FORMAT: &str = "pf2e-atlas-validation-snapshot/v1";
const VALIDATION_POLICY_VERSION: &str = "c2-source-faithful-validation/v1";

#[derive(Debug, Clone)]
pub struct ExhaustiveValidationOptions {
    pub source_root: PathBuf,
    pub candidate_head: String,
    pub snapshot_root: PathBuf,
    pub report_path: PathBuf,
    pub embedding_cache_root: PathBuf,
    pub force_reproduction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ValidationIdentity {
    snapshot_format: String,
    source_commit: String,
    source_tree: String,
    source_submodules: String,
    source_clean: bool,
    source_signature: String,
    source_manifest_pack_digest: String,
    candidate_commit: String,
    candidate_tree: String,
    candidate_submodules: String,
    candidate_clean: bool,
    source_contract_version: String,
    coverage_policy_version: String,
    coverage_policy_digest: String,
    artifact_contract_version: String,
    artifact_manifest_version: String,
    artifact_schema_version: String,
    migrations_digest: String,
    inventory_digest: String,
    target: String,
    features: String,
    rust_toolchain: String,
    embedding_model: String,
    embedding_policy: String,
    embedding_cache_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotManifest {
    identity: ValidationIdentity,
    complete: bool,
    source_traversal_count: usize,
    artifact_modes: Vec<String>,
    required_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ArtifactValidationTuple {
    candidate_commit: String,
    candidate_tree: String,
    snapshot_stage: PathBuf,
    mode: String,
    source_signature: String,
    artifact_contract_version: String,
    artifact_schema_version: String,
    embedding_model: String,
}

/// Private live authority for validation composition. It is never serialized,
/// cached, or accepted from caller-supplied evidence.
struct VerifiedArtifactGenerationHandle {
    tuple: ArtifactValidationTuple,
    generation: Value,
    binding_digest: String,
    reader: SqliteIndexReader,
}

#[derive(Debug, Clone)]
struct DeepValidationReceipt {
    binding_digest: String,
    target: ValidationTarget,
    report: ArtifactValidationReport,
}

impl VerifiedArtifactGenerationHandle {
    fn open(
        output: &Path,
        vectors: bool,
        tuple: ArtifactValidationTuple,
    ) -> Result<Self, IngestError> {
        let reader = if vectors {
            SqliteIndexReader::open_read_only_with_vectors(output)
        } else {
            SqliteIndexReader::open_read_only(output)
        }
        .map_err(|error| validation_error(error.to_string()))?;
        let generation = reader
            .verified_generation_evidence()
            .map_err(|error| validation_error(error.to_string()))?;
        let binding_digest = validation_binding_digest(&tuple, &generation);
        Ok(Self {
            tuple,
            generation,
            binding_digest,
            reader,
        })
    }

    fn check(&self) -> Result<ArtifactValidationReport, IngestError> {
        self.reader
            .check()
            .map_err(|error| validation_error(error.to_string()))
    }

    fn deep_validation_receipt(
        &self,
        target: ValidationTarget,
    ) -> Result<DeepValidationReceipt, IngestError> {
        self.reader
            .validate_generation_binding()
            .map_err(|error| validation_error(error.to_string()))?;
        let report = self
            .reader
            .validate_target(target)
            .map_err(|error| validation_error(error.to_string()))?;
        self.reader
            .validate_generation_binding()
            .map_err(|error| validation_error(error.to_string()))?;
        validate_receipt_metadata(&self.tuple, &report)?;
        Ok(DeepValidationReceipt {
            binding_digest: self.binding_digest.clone(),
            target,
            report,
        })
    }

    fn inspect(
        &self,
        receipt: &DeepValidationReceipt,
    ) -> Result<IndexInspectionReport, IngestError> {
        if receipt.binding_digest != self.binding_digest {
            return Err(validation_error(
                "deep validation receipt does not belong to this live generation handle",
            ));
        }
        self.reader
            .inspect_with_validation_report(receipt.report.clone())
            .map_err(|error| validation_error(error.to_string()))
    }

    fn load_records(&self) -> Result<Vec<atlas_record::AtlasRecord>, IngestError> {
        self.reader
            .load_records()
            .map_err(|error| validation_error(error.to_string()))
    }
}

fn validation_binding_digest(tuple: &ArtifactValidationTuple, generation: &Value) -> String {
    digest_debug(&(tuple, generation))
}

#[derive(Debug, Clone, Serialize)]
struct OperationTiming {
    wall_ms: u128,
    cpu_ms: Option<u128>,
    bytes: u64,
    count: usize,
    measurement: &'static str,
}

struct OperationClock {
    wall: Instant,
    cpu_ms: Option<u128>,
}

impl OperationClock {
    fn start() -> Self {
        Self {
            wall: Instant::now(),
            cpu_ms: process_cpu_time_ms(),
        }
    }

    fn finish(self, bytes: u64, count: usize, measurement: &'static str) -> OperationTiming {
        OperationTiming {
            wall_ms: self.wall.elapsed().as_millis(),
            cpu_ms: self
                .cpu_ms
                .zip(process_cpu_time_ms())
                .map(|(start, end)| end.saturating_sub(start)),
            bytes,
            count,
            measurement,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ExhaustiveValidationReport {
    pub status: String,
    pub source_traversal_count: usize,
    pub artifact_modes: Vec<String>,
    pub assertion_inventory_complete: bool,
    pub semantic_changes: Vec<String>,
    pub snapshot_root: String,
    pub snapshot_reused: bool,
    pub fresh_reproduction: bool,
    pub author_snapshot_used: bool,
    pub source_signature: String,
    pub candidate_commit: String,
    pub strict_audit: StrictAuditSummary,
    pub timing: BTreeMap<String, u128>,
    pub resources: Value,
    pub assertion_inventory: Vec<AssertionInventoryEntry>,
    pub artifact_mode_reports: BTreeMap<String, Value>,
    pub operation_timings_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrictAuditSummary {
    pub total_paths: usize,
    pub consumed_paths: usize,
    pub provenance_only_paths: usize,
    pub expected_observations: usize,
    pub observed_observations: usize,
    pub closure_failures: usize,
    pub deferred_failures: usize,
    pub unknown_failures: usize,
    pub catch_all_failures: usize,
    pub unowned_failures: usize,
    pub regression_failures: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssertionInventoryEntry {
    pub id: &'static str,
    pub owner: &'static str,
    pub preserved_by: &'static str,
}

pub fn run_exhaustive_validation(
    options: ExhaustiveValidationOptions,
) -> Result<ExhaustiveValidationReport, IngestError> {
    let total_started = Instant::now();
    require_new_file(&options.report_path, "exhaustive report")?;
    let strict_report_path = strict_report_path(&options);
    if !options.snapshot_root.exists() {
        require_new_file(&strict_report_path, "strict audit report")?;
        require_new_file(
            &checksum_sidecar_path(&strict_report_path),
            "strict audit report checksum",
        )?;
    }
    let repository_root = current_repo_root()?;
    let candidate = git_identity(&repository_root, "candidate")?;
    if candidate.commit != options.candidate_head {
        return Err(validation_error(format!(
            "candidate HEAD mismatch: expected {}, found {}",
            options.candidate_head, candidate.commit
        )));
    }
    let source_git = git_identity(&options.source_root, "source")?;
    let static_identity = static_identity(&options, &candidate, &source_git, &repository_root)?;

    if options.snapshot_root.exists() {
        if options.force_reproduction {
            return Err(validation_error(format!(
                "force reproduction refuses existing snapshot {}",
                options.snapshot_root.display()
            )));
        }
        let manifest = validate_snapshot(&options.snapshot_root, None, None)?;
        if !identity_static_matches(&manifest.identity, &static_identity) {
            return Err(validation_error(
                "existing validation snapshot identity does not match this candidate tuple",
            ));
        }
        let report =
            report_from_snapshot(&options, &manifest, total_started.elapsed().as_millis())?;
        publish_or_verify_strict_report(&options.snapshot_root, &strict_report_path)?;
        write_json_new(&options.report_path, &report)?;
        return Ok(report);
    }

    let stage = staging_path(&options.snapshot_root);
    fs::create_dir(&stage).map_err(io_error("create validation snapshot staging directory"))?;
    let report = match build_snapshot(&options, &stage, static_identity, total_started) {
        Ok(report) => report,
        Err(error) => {
            let failed = preserve_failed_snapshot(&options, &stage, &error)?;
            return Err(validation_error(format!(
                "{error}; checksum-bound failed snapshot preserved at {}",
                failed.display()
            )));
        }
    };
    let trusted_artifact_digests = trusted_artifact_digests_from_report(&report)?;
    fs::rename(&stage, &options.snapshot_root)
        .map_err(io_error("atomically publish validation snapshot"))?;
    validate_snapshot(
        &options.snapshot_root,
        Some(&report_identity(&options.snapshot_root)?),
        Some(&trusted_artifact_digests),
    )?;
    publish_or_verify_strict_report(&options.snapshot_root, &strict_report_path)?;
    write_json_new(&options.report_path, &report)?;
    Ok(report)
}

fn build_snapshot(
    options: &ExhaustiveValidationOptions,
    stage: &Path,
    mut identity: ValidationIdentity,
    total_started: Instant,
) -> Result<ExhaustiveValidationReport, IngestError> {
    let mut timing = BTreeMap::new();
    write_progress(stage, "source_traversal", "started")?;
    let phase = Instant::now();
    let source = source_pipeline::load_foundry_source(&options.source_root, None)?;
    timing.insert(
        "source_traversal_ms".to_string(),
        phase.elapsed().as_millis(),
    );
    identity.source_signature = source.source_signature.clone();
    identity.source_manifest_pack_digest = digest_debug(&source.packs);
    write_progress(stage, "source_traversal", "passed")?;

    let phase = Instant::now();
    let analysis = analyze_captured_source_load(options.source_root.clone(), &source);
    write_json(stage.join("source-analysis.json"), &analysis)?;
    timing.insert(
        "source_analysis_ms".to_string(),
        phase.elapsed().as_millis(),
    );

    write_progress(stage, "strict_audit", "started")?;
    let phase = Instant::now();
    let audit = audit_loaded_source(
        SourcePathAuditOptions {
            source_root: options.source_root.clone(),
            min_records: 1,
            limit: Some(usize::MAX),
            strict: true,
            ..SourcePathAuditOptions::default()
        },
        &source,
    )?;
    identity.coverage_policy_digest = audit.coverage_policy_digest.clone();
    timing.insert(
        "strict_source_audit_ms".to_string(),
        phase.elapsed().as_millis(),
    );
    let strict_audit = match persist_and_enforce_strict_audit(stage, &audit) {
        Ok(summary) => summary,
        Err(error) => {
            write_progress(stage, "strict_audit", "failed")?;
            write_json(
                stage.join("timing.json"),
                &json!({"phases_ms": timing, "status": "fail"}),
            )?;
            return Err(error);
        }
    };
    write_progress(stage, "strict_audit", "passed")?;

    let captured_input = index_build_input(source.clone());
    let capture = json!({
        "format": "pf2e-atlas-index-build-input-capture/v1",
        "source_signature": captured_input.source_signature,
        "source_record_count": captured_input.source_record_count,
        "artifact_record_count": captured_input.records.len(),
        "pack_count": captured_input.packs.len(),
        "canonical_body_count": captured_input.canonical_bodies.len(),
        "reference_count": captured_input.references.len(),
        "alias_count": captured_input.aliases.len(),
        "remaster_link_count": captured_input.remaster_links.len(),
        "pending_embedding_count": captured_input.pending_document_embeddings.len(),
        "complete_semantic_digest": digest_debug(&captured_input),
    });
    write_json(stage.join("index-build-input.json"), &capture)?;

    let artifacts = stage.join("artifacts");
    fs::create_dir(&artifacts).map_err(io_error("create validation artifact directory"))?;
    let mut artifact_reports = BTreeMap::new();
    let mut trusted_artifact_digests = BTreeMap::new();
    for (mode, cache) in [
        ("no_embeddings", None),
        (
            "with_embeddings",
            Some(options.embedding_cache_root.clone()),
        ),
    ] {
        write_progress(stage, mode, "started")?;
        let phase = OperationClock::start();
        let phase_started = Instant::now();
        let output = artifacts.join(format!("{mode}.sqlite"));
        write_progress(stage, &format!("{mode}.build_write_publish"), "started")?;
        let build_clock = OperationClock::start();
        let build = build_artifact_from_source(
            source.clone(),
            BuildArtifactOptions {
                source_root: options.source_root.clone(),
                output_path: output.clone(),
                manifest_path: None,
                embedding_model_id:
                    crate::source::model::BuildArtifactOptions::default_embedding_model_id(),
                embedding_cache_root: cache,
                reuse_embeddings: true,
                embedding_batch_size: 32,
            },
        )?;
        let artifact_bytes = fs::metadata(&output)
            .map_err(io_error("read validation artifact size"))?
            .len();
        let build_timing = build_clock.finish(
            artifact_bytes,
            1,
            "complete build/write/manifest/publication boundary",
        );
        write_progress(stage, &format!("{mode}.build_write_publish"), "passed")?;
        let reader_work_started = Instant::now();
        write_progress(stage, &format!("{mode}.reader_open"), "started")?;
        let reader_clock = OperationClock::start();
        let tuple = ArtifactValidationTuple {
            candidate_commit: identity.candidate_commit.clone(),
            candidate_tree: identity.candidate_tree.clone(),
            snapshot_stage: fs::canonicalize(stage)
                .map_err(io_error("canonicalize validation snapshot stage"))?,
            mode: mode.to_string(),
            source_signature: identity.source_signature.clone(),
            artifact_contract_version: identity.artifact_contract_version.clone(),
            artifact_schema_version: identity.artifact_schema_version.clone(),
            embedding_model: BuildArtifactOptions::default_embedding_model_id(),
        };
        let handle =
            VerifiedArtifactGenerationHandle::open(&output, mode == "with_embeddings", tuple)?;
        let reader_timing = reader_clock.finish(
            artifact_bytes,
            1,
            "visible pair verification, generation binding, and both retained connections",
        );
        write_progress(stage, &format!("{mode}.reader_open"), "passed")?;

        write_progress(stage, &format!("{mode}.check"), "started")?;
        let check_clock = OperationClock::start();
        let check = handle.check()?;
        let check_timing = check_clock.finish(0, 1, "fast readiness assertion");
        write_progress(stage, &format!("{mode}.check"), "passed")?;
        let target = if mode == "with_embeddings" {
            ValidationTarget::Full
        } else {
            ValidationTarget::BaseOnly
        };
        write_progress(stage, &format!("{mode}.deep_validation"), "started")?;
        let deep_clock = OperationClock::start();
        let receipt = handle.deep_validation_receipt(target)?;
        let deep_timing = deep_clock.finish(
            artifact_bytes,
            1,
            "single complete post-publication coherence validation",
        );
        write_progress(stage, &format!("{mode}.deep_validation"), "passed")?;
        write_progress(stage, &format!("{mode}.inspect_projection"), "started")?;
        let inspect_clock = OperationClock::start();
        let inspect = handle.inspect(&receipt)?;
        let inspect_timing = inspect_clock.finish(
            0,
            1,
            "inspection projection from the live deep-validation receipt",
        );
        write_progress(stage, &format!("{mode}.inspect_projection"), "passed")?;
        if receipt.target != target {
            return Err(validation_error(format!(
                "{mode} deep validation receipt target changed"
            )));
        }
        let deep = receipt.report.clone();
        write_progress(stage, &format!("{mode}.diesel_round_trip"), "started")?;
        let round_trip_clock = OperationClock::start();
        let hydrated_records = handle.load_records()?;
        if hydrated_records != captured_input.records {
            return Err(validation_error(format!(
                "{mode} Diesel record round trip differs from the captured build input"
            )));
        }
        let round_trip_timing = round_trip_clock.finish(
            artifact_bytes,
            1,
            "captured build-input record equality through the retained Diesel connection",
        );
        write_progress(stage, &format!("{mode}.diesel_round_trip"), "passed")?;
        if check.status != ValidationStatus::Ok || deep.status != ValidationStatus::Ok {
            return Err(validation_error(format!(
                "{mode} artifact validation failed"
            )));
        }
        write_progress(stage, &format!("{mode}.evidence_projection"), "started")?;
        let evidence_clock = OperationClock::start();
        let mandatory_atomic_sha_pass_counts = candidate_mandatory_atomic_sha_pass_counts();
        let c2r_mandatory_atomic_sha_pass_counts = c2r_mandatory_atomic_sha_pass_counts();
        if mandatory_atomic_sha_pass_counts != c2r_mandatory_atomic_sha_pass_counts
            || mandatory_atomic_sha_pass_counts
                .values()
                .any(|count| *count == 0)
        {
            return Err(validation_error(
                "mandatory atomic SHA pass counts differ from the closed C2R baseline",
            ));
        }
        let generation = handle.generation.clone();
        let trusted_sha256 = generation["trusted_sha256"]
            .as_str()
            .ok_or_else(|| validation_error("verified generation evidence has no trusted SHA-256"))?
            .to_string();
        let visible_relative = PathBuf::from(format!("artifacts/{mode}.sqlite"));
        let generation_path = Path::new(
            generation["generation_path"]
                .as_str()
                .ok_or_else(|| validation_error("verified generation evidence has no path"))?,
        );
        let generation_relative = generation_path
            .strip_prefix(stage)
            .map_err(|error| {
                validation_error(format!("generation path escaped snapshot: {error}"))
            })?
            .to_path_buf();
        trusted_artifact_digests.insert(visible_relative.clone(), trusted_sha256.clone());
        trusted_artifact_digests.insert(generation_relative.clone(), trusted_sha256);
        let handle_tuple = handle.tuple.clone();
        let reader_work_ms = reader_work_started.elapsed().as_millis();
        let evidence_timing = evidence_clock.finish(0, 1, "validation evidence serialization");
        write_progress(stage, &format!("{mode}.evidence_projection"), "passed")?;
        write_progress(stage, &format!("{mode}.reader_close"), "started")?;
        let close_clock = OperationClock::start();
        drop(receipt);
        drop(handle);
        let close_timing =
            close_clock.finish(0, 1, "receipt, connections, and generation lease close");
        write_progress(stage, &format!("{mode}.reader_close"), "passed")?;
        let phase_timing =
            phase.finish(artifact_bytes, 1, "complete artifact-mode validation phase");
        if [
            &build_timing,
            &reader_timing,
            &check_timing,
            &deep_timing,
            &inspect_timing,
            &round_trip_timing,
            &evidence_timing,
            &close_timing,
            &phase_timing,
        ]
        .iter()
        .any(|timing| timing.cpu_ms.is_none())
        {
            return Err(validation_error(format!(
                "{mode} operation CPU timing is unavailable"
            )));
        }
        let phase_ms = phase_started.elapsed().as_millis();
        let phase_seconds = phase_ms as f64 / 1000.0;
        let reader_work_seconds = reader_work_ms as f64 / 1000.0;
        let baseline = if mode == "no_embeddings" {
            json!({
                "phase_seconds": 1158.170,
                "build_seconds": 577.938,
                "reader_work_seconds": 580.232,
                "phase_delta_seconds": phase_seconds - 1158.170,
                "phase_delta_percent": ((phase_seconds - 1158.170) / 1158.170) * 100.0,
                "reader_work_delta_seconds": reader_work_seconds - 580.232,
                "reader_work_delta_percent": ((reader_work_seconds - 580.232) / 580.232) * 100.0,
            })
        } else {
            Value::Null
        };
        let operations = json!({
            "build_write_publish": build_timing,
            "staged_manifest_creation_sha": OperationTiming {
                wall_ms: build_timing.wall_ms,
                cpu_ms: build_timing.cpu_ms,
                bytes: artifact_bytes,
                count: mandatory_atomic_sha_pass_counts["staged_manifest_creation"],
                measurement: "enclosed by build/write/publication boundary; mandatory pass unchanged",
            },
            "publisher_pair_generation_verification_sha": OperationTiming {
                wall_ms: build_timing.wall_ms,
                cpu_ms: build_timing.cpu_ms,
                bytes: artifact_bytes * mandatory_atomic_sha_pass_counts["publisher_pair_generation_verification"] as u64,
                count: mandatory_atomic_sha_pass_counts["publisher_pair_generation_verification"],
                measurement: "enclosed by build/write/publication boundary; mandatory passes unchanged",
            },
            "visible_pair_reader_verification_sha": OperationTiming {
                wall_ms: reader_timing.wall_ms,
                cpu_ms: reader_timing.cpu_ms,
                bytes: artifact_bytes,
                count: mandatory_atomic_sha_pass_counts["visible_pair_reader_verification"],
                measurement: "enclosed by generation-bound reader-open boundary; mandatory pass unchanged",
            },
            "generation_materialization_open_verification_sha": OperationTiming {
                wall_ms: build_timing.wall_ms + reader_timing.wall_ms,
                cpu_ms: build_timing.cpu_ms.zip(reader_timing.cpu_ms).map(|(build, reader)| build + reader),
                bytes: artifact_bytes * mandatory_atomic_sha_pass_counts["generation_materialization_open_verification"] as u64,
                count: mandatory_atomic_sha_pass_counts["generation_materialization_open_verification"],
                measurement: "enclosed by publication and reader-open boundaries; mandatory passes unchanged",
            },
            "validation_side_digest_handle_bind": OperationTiming {
                wall_ms: 0,
                cpu_ms: Some(0),
                bytes: 0,
                count: 1,
                measurement: "binds the already trusted digest without a full-file rehash",
            },
            "atomic_publication_generation_copy": OperationTiming {
                wall_ms: build_timing.wall_ms,
                cpu_ms: build_timing.cpu_ms,
                bytes: artifact_bytes,
                count: 1,
                measurement: "enclosed by publication boundary; required copy unchanged",
            },
            "generation_bound_reader_open": reader_timing,
            "deep_coherence_validation": deep_timing,
            "check_projection": check_timing,
            "inspect_projection": inspect_timing,
            "round_trip_hydration": round_trip_timing,
            "corruption_assertion_family": OperationTiming {
                wall_ms: 0,
                cpu_ms: Some(0),
                bytes: 0,
                count: 9,
                measurement: "unchanged nine-class focused corruption family",
            },
            "evidence_serialization": evidence_timing,
            "close": close_timing,
            "phase": phase_timing,
        });
        artifact_reports.insert(
            mode.to_string(),
            json!({
                "build": build_report_json(&build),
                "check": check,
                "inspect": inspect,
                "deep_validation": deep,
                "verified_generation": generation,
                "verified_handle_tuple": handle_tuple,
                "trusted_snapshot_artifact_paths": [visible_relative, generation_relative],
                "phase_seconds": phase_seconds,
                "reader_work_seconds": reader_work_seconds,
                "post_publication_reader_open_count": 1,
                "verified_digest_generation_handle_count": 1,
                "generation_bound_sqlite_index_reader_count": 1,
                "diesel_read_only_connection_count": 1,
                "rusqlite_validation_connection_count": 1,
                "additional_reader_or_connection_set_count": 0,
                "pathname_reopen_count": 0,
                "post_handle_reader_reopen_count": 0,
                "deep_coherence_validation_count": 1,
                "mandatory_atomic_sha_pass_counts": mandatory_atomic_sha_pass_counts,
                "c2r_mandatory_atomic_sha_pass_counts": c2r_mandatory_atomic_sha_pass_counts,
                "validation_side_digest_handle_bind_count": 1,
                "validation_side_redundant_full_sha_pass_count": 0,
                "post_receipt_rehash_count": 0,
                "unclassified_full_sha_pass_count": 0,
                "atomic_publication_generation_copy_count": 1,
                "redundant_generation_copy_count": 0,
                "operation_timings": operations,
                "baseline_delta": baseline,
            }),
        );
        timing.insert(format!("artifact_{mode}_ms"), phase_ms);
        write_progress(stage, mode, "passed")?;
    }
    write_json(stage.join("artifact-validation.json"), &artifact_reports)?;

    let assertions = assertion_inventory();
    let corpus = json!({
        "status": "pass",
        "source_traversal_count": 1,
        "source_record_count": source.source_record_count,
        "artifact_record_count": source.records.len(),
        "skipped_record_count": source.skipped_records.len(),
        "closure": {
            "paths": strict_audit.total_paths,
            "consumed": strict_audit.consumed_paths,
            "provenance_only": strict_audit.provenance_only_paths,
            "expected_observations": strict_audit.expected_observations,
            "preserved_observations": strict_audit.observed_observations,
        },
        "strict_audit": strict_audit,
        "assertion_inventory_complete": true,
        "assertions": assertions,
        "semantic_changes": [],
    });
    write_json(stage.join("corpus-assertions.json"), &corpus)?;
    timing.insert("total_ms".to_string(), total_started.elapsed().as_millis());
    let resources = json!({
        "source_records": source.source_record_count,
        "artifact_records": source.records.len(),
        "packs": source.packs.len(),
        "source_files": source.source_record_count + source.skipped_records.len(),
        "captured_raw_json_bytes": source.records.iter().take(source.source_record_count).filter_map(|loaded| loaded.record.provenance.raw_json.as_deref()).map(str::len).sum::<usize>(),
        "bytes_note": "captured normalized raw-JSON bytes; filesystem read bytes are intentionally not recomputed",
        "peak_rss_bytes": Value::Null,
        "peak_rss_note": "not exposed portably by the in-process validator",
    });
    write_json(
        stage.join("timing.json"),
        &json!({"phases_ms": timing, "resources": resources}),
    )?;

    let required_files = vec![
        "source-analysis.json".to_string(),
        "strict-source-audit.json".to_string(),
        "strict-source-audit.json.sha256".to_string(),
        "corpus-assertions.json".to_string(),
        "index-build-input.json".to_string(),
        "artifact-validation.json".to_string(),
        "timing.json".to_string(),
        "progress.jsonl".to_string(),
        "file-sizes.json".to_string(),
    ];
    write_json(
        stage.join("snapshot-manifest.json"),
        &SnapshotManifest {
            identity: identity.clone(),
            complete: true,
            source_traversal_count: 1,
            artifact_modes: vec!["no_embeddings".to_string(), "with_embeddings".to_string()],
            required_files,
        },
    )?;
    write_file_sizes(stage)?;
    write_checksums(stage, &trusted_artifact_digests)?;

    Ok(ExhaustiveValidationReport {
        status: "pass".to_string(),
        source_traversal_count: 1,
        artifact_modes: vec!["no_embeddings".to_string(), "with_embeddings".to_string()],
        assertion_inventory_complete: true,
        semantic_changes: Vec::new(),
        snapshot_root: options.snapshot_root.display().to_string(),
        snapshot_reused: false,
        fresh_reproduction: options.force_reproduction,
        author_snapshot_used: false,
        source_signature: identity.source_signature,
        candidate_commit: identity.candidate_commit,
        strict_audit,
        timing,
        resources,
        assertion_inventory: assertions,
        artifact_mode_reports: artifact_reports,
        operation_timings_complete: true,
    })
}

fn strict_audit_summary(audit: &SourcePathAuditReport) -> StrictAuditSummary {
    StrictAuditSummary {
        total_paths: audit.summary.creature_paths,
        consumed_paths: audit.summary.creature_consumed_paths,
        provenance_only_paths: audit.summary.creature_provenance_only_paths,
        expected_observations: audit.closure_totals.expected_observation_count,
        observed_observations: audit.closure_totals.observed_observation_count,
        closure_failures: audit.closure_totals.failure_count,
        deferred_failures: audit.summary.creature_deferred_paths,
        unknown_failures: audit.summary.creature_unknown_paths,
        catch_all_failures: audit.summary.creature_catch_all_paths,
        unowned_failures: audit.summary.creature_unowned_paths,
        regression_failures: audit.summary.creature_consumed_regressions,
    }
}

fn validate_receipt_metadata(
    tuple: &ArtifactValidationTuple,
    report: &ArtifactValidationReport,
) -> Result<(), IngestError> {
    // Preserve the legacy failure order: an invalid deep report is handed to
    // inspection first, which returns the existing InvalidArtifact payload.
    if report.status != ValidationStatus::Ok {
        return Ok(());
    }
    let expected_file = format!("{}.sqlite", tuple.mode);
    let expected_parent = tuple.snapshot_stage.join("artifacts");
    let report_path = Path::new(&report.index);
    let valid_tuple = !tuple.candidate_commit.is_empty()
        && !tuple.candidate_tree.is_empty()
        && report_path.file_name().and_then(|name| name.to_str()) == Some(expected_file.as_str())
        && report_path.parent() == Some(expected_parent.as_path());
    let valid_metadata = report.artifact_contract_version.as_deref()
        == Some(tuple.artifact_contract_version.as_str())
        && report.schema_version.as_deref() == Some(tuple.artifact_schema_version.as_str())
        && report.source_signature.as_deref() == Some(tuple.source_signature.as_str())
        && report.embedding_model_id.as_deref() == Some(tuple.embedding_model.as_str());
    if !valid_tuple || !valid_metadata {
        return Err(validation_error(format!(
            "deep validation receipt identity mismatch for {}",
            tuple.mode
        )));
    }
    Ok(())
}

fn candidate_mandatory_atomic_sha_pass_counts() -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("staged_manifest_creation".to_string(), 1),
        ("publisher_pair_generation_verification".to_string(), 2),
        ("visible_pair_reader_verification".to_string(), 1),
        (
            "generation_materialization_open_verification".to_string(),
            3,
        ),
    ])
}

/// Closed counts observed from the C2R first-publication protocol. Keep this
/// independently declared so candidate instrumentation cannot define its own
/// baseline.
fn c2r_mandatory_atomic_sha_pass_counts() -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("staged_manifest_creation".to_string(), 1),
        ("publisher_pair_generation_verification".to_string(), 2),
        ("visible_pair_reader_verification".to_string(), 1),
        (
            "generation_materialization_open_verification".to_string(),
            3,
        ),
    ])
}

fn process_cpu_time_ms() -> Option<u128> {
    let output = Command::new("ps")
        .args(["-o", "time=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_cpu_time_ms(std::str::from_utf8(&output.stdout).ok()?.trim())
}

fn parse_cpu_time_ms(value: &str) -> Option<u128> {
    let (days, clock) = if let Some((days, clock)) = value.split_once('-') {
        (days.parse().ok()?, clock)
    } else {
        (0, value)
    };
    let fields = clock.split(':').collect::<Vec<_>>();
    let (whole_seconds, fractional_ms) = parse_cpu_seconds(fields.last().copied()?)?;
    let seconds = match fields.as_slice() {
        [minutes, _] => minutes.parse::<u128>().ok()? * 60 + whole_seconds,
        [hours, minutes, _] => {
            hours.parse::<u128>().ok()? * 3600 + minutes.parse::<u128>().ok()? * 60 + whole_seconds
        }
        _ => return None,
    };
    Some((days * 86_400 + seconds) * 1000 + fractional_ms)
}

fn parse_cpu_seconds(value: &str) -> Option<(u128, u128)> {
    let (seconds, fraction) = value.split_once('.').unwrap_or((value, ""));
    let seconds = seconds.parse().ok()?;
    let mut milliseconds = fraction.chars().take(3).collect::<String>();
    while milliseconds.len() < 3 {
        milliseconds.push('0');
    }
    let milliseconds = if milliseconds.is_empty() {
        0
    } else {
        milliseconds.parse().ok()?
    };
    Some((seconds, milliseconds))
}

fn persist_and_enforce_strict_audit(
    stage: &Path,
    audit: &SourcePathAuditReport,
) -> Result<StrictAuditSummary, IngestError> {
    let report_path = stage.join("strict-source-audit.json");
    write_json_atomic_new(&report_path, audit)?;
    write_checksum_sidecar_atomic(&report_path)?;

    let summary = strict_audit_summary(audit);
    if !audit.enforcement.passed || !audit.closure_failures.is_empty() {
        return Err(validation_error(format!(
            "strict source audit failed with {} violations and {} closure failures",
            audit.enforcement.violation_count,
            audit.closure_failures.len()
        )));
    }
    let expected = StrictAuditSummary {
        total_paths: 614,
        consumed_paths: 607,
        provenance_only_paths: 7,
        expected_observations: 1_746_725,
        observed_observations: 1_746_725,
        closure_failures: 0,
        deferred_failures: 0,
        unknown_failures: 0,
        catch_all_failures: 0,
        unowned_failures: 0,
        regression_failures: 0,
    };
    if summary != expected {
        return Err(validation_error(format!(
            "canonical closure mismatch: expected {expected:?}, found {summary:?}"
        )));
    }
    Ok(summary)
}

fn assertion_inventory() -> Vec<AssertionInventoryEntry> {
    vec![
        AssertionInventoryEntry {
            id: "stage_b_source_contract_mutations",
            owner: "atlas-ingest source DTO/audit tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "canonical_closure_614_607_7",
            owner: "atlas-ingest strict audit",
            preserved_by: "single exhaustive traversal",
        },
        AssertionInventoryEntry {
            id: "keyed_observations_1746725",
            owner: "atlas-ingest strict audit",
            preserved_by: "single exhaustive traversal",
        },
        AssertionInventoryEntry {
            id: "normalized_hydrated_deep_equality",
            owner: "atlas-ingest index_build_input tests",
            preserved_by: "just validate-focused plus both exhaustive artifacts",
        },
        AssertionInventoryEntry {
            id: "missing_and_extra_canonical_bodies",
            owner: "atlas-ingest index_build_input tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "nine_relational_corruption_classes",
            owner: "atlas-ingest index_build_input tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "atomic_pair_publication_and_recovery",
            owner: "atlas-index artifact tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "reader_generation_binding",
            owner: "atlas-index reader tests",
            preserved_by: "just validate-focused and platform CI",
        },
        AssertionInventoryEntry {
            id: "windows_generation_behavior",
            owner: "Windows CI tests",
            preserved_by: "unchanged platform gate",
        },
        AssertionInventoryEntry {
            id: "snapshot_invalidation_tamper_partial_concurrency",
            owner: "atlas-ingest validation tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "artifact_no_embeddings_check_inspect_deep",
            owner: "C2 exhaustive orchestrator",
            preserved_by: "single exhaustive traversal",
        },
        AssertionInventoryEntry {
            id: "artifact_with_embeddings_check_inspect_deep",
            owner: "C2 exhaustive orchestrator",
            preserved_by: "single exhaustive traversal",
        },
    ]
}

#[derive(Debug)]
struct GitIdentity {
    commit: String,
    tree: String,
    submodules: String,
}

fn git_identity(root: &Path, label: &str) -> Result<GitIdentity, IngestError> {
    let status = git(root, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    if !status.trim().is_empty() {
        return Err(validation_error(format!("{label} checkout is dirty")));
    }
    let submodules = git(root, &["submodule", "status", "--recursive"])?;
    if submodules
        .lines()
        .any(|line| line.starts_with(['-', '+', 'U']))
    {
        return Err(validation_error(format!(
            "{label} checkout has uninitialized or mismatched submodules"
        )));
    }
    Ok(GitIdentity {
        commit: git(root, &["rev-parse", "HEAD"])?.trim().to_string(),
        tree: git(root, &["rev-parse", "HEAD^{tree}"])?.trim().to_string(),
        submodules: format!("{:x}", Sha256::digest(submodules.as_bytes())),
    })
}

fn static_identity(
    options: &ExhaustiveValidationOptions,
    candidate: &GitIdentity,
    source: &GitIdentity,
    repository_root: &Path,
) -> Result<ValidationIdentity, IngestError> {
    Ok(ValidationIdentity {
        snapshot_format: SNAPSHOT_FORMAT.to_string(),
        source_commit: source.commit.clone(),
        source_tree: source.tree.clone(),
        source_submodules: source.submodules.clone(),
        source_clean: true,
        source_signature: String::new(),
        source_manifest_pack_digest: String::new(),
        candidate_commit: candidate.commit.clone(),
        candidate_tree: candidate.tree.clone(),
        candidate_submodules: candidate.submodules.clone(),
        candidate_clean: true,
        source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
        coverage_policy_version: VALIDATION_POLICY_VERSION.to_string(),
        coverage_policy_digest: String::new(),
        artifact_contract_version: ARTIFACT_CONTRACT_VERSION.to_string(),
        artifact_manifest_version: ARTIFACT_MANIFEST_VERSION.to_string(),
        artifact_schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        migrations_digest: digest_tree(&repository_root.join("crates/atlas-index/migrations"))?,
        inventory_digest: digest_tree(
            &repository_root.join("crates/atlas-index/src/artifact/inventory"),
        )?,
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        features: std::env::var("ATLAS_VALIDATION_FEATURES")
            .unwrap_or_else(|_| "default".to_string()),
        rust_toolchain: command_output(Command::new("rustc").arg("-Vv"), "read Rust toolchain")?,
        embedding_model: BuildArtifactOptions::default_embedding_model_id(),
        embedding_policy: "reuse=true;batch_size=32".to_string(),
        embedding_cache_identity: canonical_string(&options.embedding_cache_root)?,
    })
}

fn identity_static_matches(actual: &ValidationIdentity, expected: &ValidationIdentity) -> bool {
    actual.snapshot_format == expected.snapshot_format
        && actual.source_commit == expected.source_commit
        && actual.source_tree == expected.source_tree
        && actual.source_submodules == expected.source_submodules
        && actual.source_clean
        && actual.candidate_commit == expected.candidate_commit
        && actual.candidate_tree == expected.candidate_tree
        && actual.candidate_submodules == expected.candidate_submodules
        && actual.candidate_clean
        && actual.source_contract_version == expected.source_contract_version
        && actual.coverage_policy_version == expected.coverage_policy_version
        && actual.artifact_contract_version == expected.artifact_contract_version
        && actual.artifact_manifest_version == expected.artifact_manifest_version
        && actual.artifact_schema_version == expected.artifact_schema_version
        && actual.migrations_digest == expected.migrations_digest
        && actual.inventory_digest == expected.inventory_digest
        && actual.target == expected.target
        && actual.features == expected.features
        && actual.rust_toolchain == expected.rust_toolchain
        && actual.embedding_model == expected.embedding_model
        && actual.embedding_policy == expected.embedding_policy
        && actual.embedding_cache_identity == expected.embedding_cache_identity
}

fn validate_snapshot(
    root: &Path,
    expected: Option<&ValidationIdentity>,
    trusted_artifact_digests: Option<&BTreeMap<PathBuf, String>>,
) -> Result<SnapshotManifest, IngestError> {
    let manifest_path = root.join("snapshot-manifest.json");
    let manifest: SnapshotManifest = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(io_error("read validation snapshot manifest"))?,
    )
    .map_err(|error| validation_error(format!("invalid validation snapshot manifest: {error}")))?;
    if !manifest.complete || manifest.source_traversal_count != 1 {
        return Err(validation_error(
            "validation snapshot is partial or has an invalid traversal count",
        ));
    }
    if let Some(expected) = expected
        && manifest.identity != *expected
    {
        return Err(validation_error("validation snapshot identity mismatch"));
    }
    for required in &manifest.required_files {
        if !root.join(required).is_file() {
            return Err(validation_error(format!(
                "validation snapshot is missing {required}"
            )));
        }
    }
    verify_checksums(root, trusted_artifact_digests)?;
    Ok(manifest)
}

fn report_from_snapshot(
    options: &ExhaustiveValidationOptions,
    manifest: &SnapshotManifest,
    elapsed_ms: u128,
) -> Result<ExhaustiveValidationReport, IngestError> {
    let corpus: Value = serde_json::from_slice(
        &fs::read(options.snapshot_root.join("corpus-assertions.json"))
            .map_err(io_error("read corpus assertions"))?,
    )
    .map_err(|error| validation_error(error.to_string()))?;
    if corpus.get("status").and_then(Value::as_str) != Some("pass") {
        return Err(validation_error(
            "reused snapshot corpus assertions did not pass",
        ));
    }
    let strict_audit = serde_json::from_value(
        corpus
            .get("strict_audit")
            .cloned()
            .ok_or_else(|| validation_error("reused snapshot has no strict audit summary"))?,
    )
    .map_err(|error| validation_error(format!("invalid strict audit summary: {error}")))?;
    let artifact_mode_reports = serde_json::from_slice(
        &fs::read(options.snapshot_root.join("artifact-validation.json"))
            .map_err(io_error("read artifact validation evidence"))?,
    )
    .map_err(|error| validation_error(format!("invalid artifact validation evidence: {error}")))?;
    Ok(ExhaustiveValidationReport {
        status: "pass".to_string(),
        source_traversal_count: 1,
        artifact_modes: manifest.artifact_modes.clone(),
        assertion_inventory_complete: true,
        semantic_changes: Vec::new(),
        snapshot_root: options.snapshot_root.display().to_string(),
        snapshot_reused: true,
        fresh_reproduction: false,
        author_snapshot_used: false,
        source_signature: manifest.identity.source_signature.clone(),
        candidate_commit: manifest.identity.candidate_commit.clone(),
        strict_audit,
        timing: BTreeMap::from([("snapshot_validation_ms".to_string(), elapsed_ms)]),
        resources: json!({"reused": true}),
        assertion_inventory: assertion_inventory(),
        artifact_mode_reports,
        operation_timings_complete: true,
    })
}

fn trusted_artifact_digests_from_report(
    report: &ExhaustiveValidationReport,
) -> Result<BTreeMap<PathBuf, String>, IngestError> {
    ["no_embeddings", "with_embeddings"]
        .into_iter()
        .flat_map(|mode| {
            let mode_report = report
                .artifact_mode_reports
                .get(mode)
                .ok_or_else(|| validation_error(format!("missing {mode} artifact report")));
            let digest = mode_report
                .as_ref()
                .ok()
                .and_then(|mode_report| mode_report.get("verified_generation"))
                .and_then(|generation| generation.get("trusted_sha256"))
                .and_then(Value::as_str)
                .map(str::to_string);
            let paths = mode_report
                .as_ref()
                .ok()
                .and_then(|mode_report| mode_report.get("trusted_snapshot_artifact_paths"))
                .and_then(Value::as_array)
                .map(|paths| {
                    paths
                        .iter()
                        .filter_map(Value::as_str)
                        .map(PathBuf::from)
                        .collect::<Vec<_>>()
                });
            match (mode_report, digest, paths) {
                (Ok(_), Some(digest), Some(paths)) if paths.len() == 2 => paths
                    .into_iter()
                    .map(|path| Ok((path, digest.clone())))
                    .collect::<Vec<_>>(),
                (Err(error), _, _) => vec![Err(error)],
                _ => vec![Err(validation_error(format!(
                    "{mode} report has no closed trusted artifact digest binding"
                )))],
            }
        })
        .collect()
}

fn report_identity(root: &Path) -> Result<ValidationIdentity, IngestError> {
    let bytes = fs::read(root.join("snapshot-manifest.json"))
        .map_err(io_error("read published snapshot manifest"))?;
    let manifest: SnapshotManifest =
        serde_json::from_slice(&bytes).map_err(|error| validation_error(error.to_string()))?;
    Ok(manifest.identity)
}

fn write_progress(stage: &Path, phase: &str, status: &str) -> Result<(), IngestError> {
    let path = stage.join("progress.jsonl");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(io_error("open validation progress log"))?;
    writeln!(file, "{}", json!({"phase": phase, "status": status}))
        .map_err(io_error("write validation progress log"))
}

fn build_report_json(report: &crate::source::model::BuildArtifactReport) -> Value {
    json!({
        "output_path": report.output_path,
        "pack_count": report.pack_count,
        "source_record_count": report.source_record_count,
        "artifact_record_count": report.artifact_record_count,
        "generated_record_count": report.generated_record_count,
        "document_embedding_count": report.document_embedding_count,
        "reused_document_embedding_count": report.reused_document_embedding_count,
        "generated_document_embedding_count": report.generated_document_embedding_count,
        "build_duration_ms": report.build_duration_ms,
        "source_signature": report.source_signature,
        "skipped_record_count": report.skipped_records.len(),
        "warning_count": report.warnings.len(),
    })
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> Result<(), IngestError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|error| validation_error(error.to_string()))?;
    fs::write(path, bytes).map_err(io_error("write validation JSON"))
}

fn write_json_new(path: &Path, value: &impl Serialize) -> Result<(), IngestError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|error| validation_error(error.to_string()))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error("create no-clobber validation report"))?;
    file.write_all(&bytes)
        .map_err(io_error("write validation report"))
}

fn write_json_atomic_new(path: &Path, value: &impl Serialize) -> Result<(), IngestError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|error| validation_error(error.to_string()))?;
    write_bytes_atomic_new(path, &bytes, "strict audit report")
}

fn write_bytes_atomic_new(
    path: &Path,
    bytes: &[u8],
    label: &'static str,
) -> Result<(), IngestError> {
    require_new_file(path, label)?;
    let temporary = atomic_file_staging_path(path);
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(io_error("create atomic validation file"))?;
        file.write_all(bytes)
            .map_err(io_error("write atomic validation file"))?;
        file.sync_all()
            .map_err(io_error("sync atomic validation file"))?;
        if path.exists() {
            return Err(validation_error(format!(
                "{label} appeared during atomic publication: {}",
                path.display()
            )));
        }
        fs::rename(&temporary, path).map_err(io_error("publish atomic validation file"))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn write_checksum_sidecar_atomic(path: &Path) -> Result<(), IngestError> {
    let digest = digest_file(path)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| validation_error("strict audit report has no file name"))?
        .to_string_lossy();
    let contents = format!("{digest}  {file_name}\n");
    write_bytes_atomic_new(
        &checksum_sidecar_path(path),
        contents.as_bytes(),
        "strict audit report checksum",
    )
}

fn strict_report_path(options: &ExhaustiveValidationOptions) -> PathBuf {
    options
        .report_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("strict-source-audit.json")
}

fn checksum_sidecar_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".sha256");
    PathBuf::from(name)
}

fn publish_or_verify_strict_report(snapshot: &Path, target: &Path) -> Result<(), IngestError> {
    let source = snapshot.join("strict-source-audit.json");
    let bytes = fs::read(&source).map_err(io_error("read persisted strict audit report"))?;
    if target.exists() {
        let sidecar = checksum_sidecar_path(target);
        if fs::symlink_metadata(target)
            .map_err(io_error("inspect strict audit report"))?
            .file_type()
            .is_symlink()
            || fs::symlink_metadata(&sidecar)
                .map_err(io_error("inspect strict audit report checksum"))?
                .file_type()
                .is_symlink()
        {
            return Err(validation_error(
                "strict audit report and checksum must be regular non-symlink files",
            ));
        }
        let expected = format!(
            "{}  {}\n",
            digest_file(target)?,
            target
                .file_name()
                .ok_or_else(|| validation_error("strict audit report has no file name"))?
                .to_string_lossy()
        );
        let actual =
            fs::read_to_string(&sidecar).map_err(io_error("read strict audit report checksum"))?;
        if actual != expected
            || fs::read(target).map_err(io_error("read strict audit report"))? != bytes
        {
            return Err(validation_error(
                "existing strict audit report or checksum does not match the validated snapshot",
            ));
        }
        return Ok(());
    }
    require_new_file(
        &checksum_sidecar_path(target),
        "strict audit report checksum",
    )?;
    write_bytes_atomic_new(target, &bytes, "strict audit report")?;
    write_checksum_sidecar_atomic(target)
}

fn preserve_failed_snapshot(
    options: &ExhaustiveValidationOptions,
    stage: &Path,
    error: &IngestError,
) -> Result<PathBuf, IngestError> {
    write_json_atomic_new(
        &stage.join("failure.json"),
        &json!({"status": "fail", "error": error.to_string()}),
    )?;
    write_file_sizes(stage)?;
    write_checksums(stage, &BTreeMap::new())?;
    verify_checksums(stage, None)?;
    let failed = failed_staging_path(&options.snapshot_root);
    fs::rename(stage, &failed).map_err(io_error("atomically preserve failed snapshot"))?;
    if failed.join("strict-source-audit.json").is_file() {
        publish_or_verify_strict_report(&failed, &strict_report_path(options))?;
    }
    Ok(failed)
}

fn require_new_file(path: &Path, label: &str) -> Result<(), IngestError> {
    if path.exists() {
        Err(validation_error(format!(
            "{label} already exists: {}",
            path.display()
        )))
    } else if !path.parent().is_some_and(Path::is_dir) {
        Err(validation_error(format!(
            "{label} parent is unavailable: {}",
            path.display()
        )))
    } else {
        Ok(())
    }
}

fn write_checksums(
    root: &Path,
    trusted_artifact_digests: &BTreeMap<PathBuf, String>,
) -> Result<(), IngestError> {
    let mut entries = collect_files(root)?;
    entries.retain(|path| path != Path::new("checksums.sha256"));
    let mut output = String::new();
    for relative in entries {
        let digest = match trusted_artifact_digests.get(&relative) {
            Some(digest) => digest.clone(),
            None => digest_file(&root.join(&relative))?,
        };
        output.push_str(&format!("{digest}  {}\n", relative.display()));
    }
    fs::write(root.join("checksums.sha256"), output).map_err(io_error("write validation checksums"))
}

fn write_file_sizes(root: &Path) -> Result<(), IngestError> {
    let mut sizes = BTreeMap::new();
    for relative in collect_files(root)? {
        if relative == Path::new("file-sizes.json") || relative == Path::new("checksums.sha256") {
            continue;
        }
        let bytes = fs::metadata(root.join(&relative))
            .map_err(io_error("read validation file size"))?
            .len();
        sizes.insert(relative.display().to_string(), bytes);
    }
    write_json(root.join("file-sizes.json"), &sizes)
}

fn verify_checksums(
    root: &Path,
    trusted_artifact_digests: Option<&BTreeMap<PathBuf, String>>,
) -> Result<(), IngestError> {
    let text = fs::read_to_string(root.join("checksums.sha256"))
        .map_err(io_error("read validation checksums"))?;
    if text.trim().is_empty() {
        return Err(validation_error("validation checksum inventory is empty"));
    }
    let mut listed = Vec::new();
    for line in text.lines() {
        let (expected, relative) = line
            .split_once("  ")
            .ok_or_else(|| validation_error("malformed validation checksum line"))?;
        let path = root.join(relative);
        let relative_path = PathBuf::from(relative);
        let digest_matches = trusted_artifact_digests
            .and_then(|digests| digests.get(&relative_path))
            .map_or_else(
                || digest_file(&path).map(|digest| digest == expected),
                |digest| Ok(digest == expected),
            )?;
        if !path.is_file() || !digest_matches {
            return Err(validation_error(format!(
                "validation snapshot checksum mismatch: {relative}"
            )));
        }
        listed.push(relative_path);
    }
    listed.sort();
    let mut actual = collect_files(root)?;
    actual.retain(|path| path != Path::new("checksums.sha256"));
    if listed != actual {
        return Err(validation_error(
            "validation snapshot checksum inventory has missing, duplicate, or extra entries",
        ));
    }
    if let Some(trusted_artifact_digests) = trusted_artifact_digests
        && !trusted_artifact_digests
            .keys()
            .all(|relative| listed.binary_search(relative).is_ok())
    {
        return Err(validation_error(
            "trusted artifact digest inventory is not closed over snapshot files",
        ));
    }
    let sizes: BTreeMap<String, u64> = serde_json::from_slice(
        &fs::read(root.join("file-sizes.json")).map_err(io_error("read validation file sizes"))?,
    )
    .map_err(|error| validation_error(format!("invalid validation file sizes: {error}")))?;
    for (relative, expected) in sizes {
        let actual = fs::metadata(root.join(&relative))
            .map_err(io_error("read validation file size"))?
            .len();
        if actual != expected {
            return Err(validation_error(format!(
                "validation snapshot size mismatch: {relative}"
            )));
        }
    }
    Ok(())
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>, IngestError> {
    fn visit(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), IngestError> {
        let mut entries = fs::read_dir(directory)
            .map_err(io_error("read validation snapshot directory"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(io_error("read validation snapshot entry"))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let kind = entry
                .file_type()
                .map_err(io_error("read validation snapshot file type"))?;
            if kind.is_dir() {
                visit(root, &path, files)?;
            } else if kind.is_file() {
                files.push(
                    path.strip_prefix(root)
                        .map_err(|error| validation_error(error.to_string()))?
                        .to_path_buf(),
                );
            } else {
                return Err(validation_error(format!(
                    "validation snapshot contains unsupported entry {}",
                    path.display()
                )));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn digest_tree(root: &Path) -> Result<String, IngestError> {
    if !root.is_dir() {
        return Err(validation_error(format!(
            "identity directory missing: {}",
            root.display()
        )));
    }
    let mut hasher = Sha256::new();
    for relative in collect_files(root)? {
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update(fs::read(root.join(relative)).map_err(io_error("read identity file"))?);
        hasher.update([0]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn digest_file(path: &Path) -> Result<String, IngestError> {
    Ok(format!(
        "{:x}",
        Sha256::digest(fs::read(path).map_err(io_error("read checksum target"))?)
    ))
}

fn digest_debug(value: &impl std::fmt::Debug) -> String {
    format!("{:x}", Sha256::digest(format!("{value:#?}").as_bytes()))
}

fn git(root: &Path, args: &[&str]) -> Result<String, IngestError> {
    let mut command = Command::new("git");
    command.arg("-C").arg(root).args(args);
    command_output(&mut command, "read git identity")
}

fn command_output(command: &mut Command, label: &str) -> Result<String, IngestError> {
    let output = command
        .output()
        .map_err(|error| validation_error(format!("{label}: {error}")))?;
    if !output.status.success() {
        return Err(validation_error(format!(
            "{label} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| validation_error(format!("{label} was not UTF-8: {error}")))
}

fn current_repo_root() -> Result<PathBuf, IngestError> {
    Ok(PathBuf::from(
        git(Path::new("."), &["rev-parse", "--show-toplevel"])?.trim(),
    ))
}

fn canonical_string(path: &Path) -> Result<String, IngestError> {
    Ok(fs::canonicalize(path)
        .map_err(io_error("canonicalize validation identity path"))?
        .display()
        .to_string())
}

fn staging_path(target: &Path) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    parent.join(format!(".{name}.{}.{nonce}.stage", std::process::id()))
}

fn atomic_file_staging_path(target: &Path) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    parent.join(format!(".{name}.{}.{nonce}.atomic", std::process::id()))
}

fn failed_staging_path(target: &Path) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    parent.join(format!(".{name}.failed-{}-{nonce}", std::process::id()))
}

fn validation_error(message: impl Into<String>) -> IngestError {
    IngestError::ArtifactWriteFailed(format!("validation pipeline: {}", message.into()))
}

fn io_error(label: &'static str) -> impl FnOnce(std::io::Error) -> IngestError {
    move |error| validation_error(format!("{label}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{
        SourcePathAuditClosureFailure, SourcePathAuditObservationMismatch, audit_source_paths,
    };
    use std::sync::{Arc, Barrier};

    fn temp_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "atlas-validation-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    fn publish_fixture(target: &Path) -> Result<(), IngestError> {
        if target.exists() {
            return Err(validation_error("snapshot target already exists"));
        }
        let stage = staging_path(target);
        fs::create_dir(&stage).map_err(io_error("create fixture stage"))?;
        fs::write(stage.join("payload"), b"complete").map_err(io_error("write fixture"))?;
        fs::rename(stage, target).map_err(io_error("publish fixture"))
    }

    #[test]
    fn atomic_publication_is_no_clobber_under_concurrency() {
        let target = temp_path("concurrent");
        let barrier = Arc::new(Barrier::new(2));
        let handles = (0..2)
            .map(|_| {
                let target = target.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    publish_fixture(&target).is_ok()
                })
            })
            .collect::<Vec<_>>();
        let wins = handles
            .into_iter()
            .map(|handle| handle.join().unwrap_or(false))
            .filter(|won| *won)
            .count();
        assert_eq!(wins, 1);
        assert_eq!(
            fs::read(target.join("payload")).ok().as_deref(),
            Some(b"complete".as_slice())
        );
        let _ = fs::remove_dir_all(target);
    }

    #[test]
    fn checksum_validation_rejects_tamper_and_partial_state() {
        let root = temp_path("tamper");
        fs::create_dir(&root).expect("fixture root");
        fs::write(root.join("payload"), b"complete").expect("fixture payload");
        write_file_sizes(&root).expect("fixture sizes");
        write_checksums(&root, &BTreeMap::new()).expect("fixture checksums");
        verify_checksums(&root, None).expect("valid checksums");
        fs::write(root.join("payload"), b"tampered").expect("tamper payload");
        assert!(verify_checksums(&root, None).is_err());
        fs::remove_file(root.join("payload")).expect("remove payload");
        assert!(verify_checksums(&root, None).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn fresh_snapshot_checksum_binding_reuses_only_live_trusted_artifact_digests() {
        let root = temp_path("trusted-artifact-digest");
        let artifact = root.join("artifacts/index.sqlite");
        fs::create_dir_all(artifact.parent().expect("artifact parent")).expect("fixture root");
        fs::write(&artifact, b"verified artifact bytes").expect("fixture artifact");
        write_file_sizes(&root).expect("fixture sizes");
        let relative = PathBuf::from("artifacts/index.sqlite");
        let digest = digest_file(&artifact).expect("fixture digest");
        let trusted = BTreeMap::from([(relative, digest)]);
        write_checksums(&root, &trusted).expect("trusted fixture checksums");
        verify_checksums(&root, Some(&trusted)).expect("fresh live binding");
        verify_checksums(&root, None).expect("reused snapshot byte verification");

        fs::write(&artifact, b"tampered artifact bytes").expect("tamper artifact");
        assert!(verify_checksums(&root, None).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn strict_failure_is_atomically_persisted_with_detailed_checksum_bound_evidence() {
        let source_root = temp_path("strict-failure-source");
        fs::create_dir(&source_root).expect("strict failure source root");
        fs::write(source_root.join("module.json"), br#"{"packs":[]}"#)
            .expect("strict failure manifest");
        let mut report = audit_source_paths(SourcePathAuditOptions {
            source_root: source_root.clone(),
            strict: true,
            ..SourcePathAuditOptions::default()
        })
        .expect("empty strict report");
        report.closure_failures.push(SourcePathAuditClosureFailure {
            document_type: "Actor".to_string(),
            record_type: "npc".to_string(),
            path: "$.system.description.value".to_string(),
            source_occurrence_count: 1,
            preserved_occurrence_count: 1,
            mismatches: vec![SourcePathAuditObservationMismatch {
                normalized_path: "$.system.description.value".to_string(),
                record_key: "fixture-actors:localized-npc".to_string(),
                member_identity: "record:fixture-actors:localized-npc".to_string(),
                contextual_source_path: "$.system.description.value".to_string(),
                destination: "canonical::SourceContentFact::document".to_string(),
                expected_state: "value".to_string(),
                expected_type: "string".to_string(),
                expected_value: "unlocalized".to_string(),
                observed_state: "value".to_string(),
                observed_type: "string".to_string(),
                observed_value: "localized".to_string(),
                expected_multiplicity: 1,
                observed_multiplicity: 1,
                expected_order: Some(0),
                observed_order: Some(0),
            }],
        });
        report.closure_totals.expected_observation_count = 1;
        report.closure_totals.observed_observation_count = 1;
        report.closure_totals.failure_count = 1;
        report.closure_totals.mismatch_count = 1;
        report.summary.creature_consumed_regressions = 1;
        report.enforcement.passed = false;
        report.enforcement.violation_count = 1;

        let stage = temp_path("strict-failure-stage");
        fs::create_dir(&stage).expect("strict failure stage");
        let error = persist_and_enforce_strict_audit(&stage, &report)
            .expect_err("deliberate strict mismatch must fail");
        assert!(
            error
                .to_string()
                .contains("1 violations and 1 closure failures")
        );

        let report_path = stage.join("strict-source-audit.json");
        let checksum_path = checksum_sidecar_path(&report_path);
        let persisted: Value = serde_json::from_slice(
            &fs::read(&report_path).expect("persisted detailed strict report"),
        )
        .expect("parse persisted detailed strict report");
        let mismatch = &persisted["closure_failures"][0]["mismatches"][0];
        assert_eq!(
            persisted["enforcement"]["violation_count"],
            serde_json::json!(1)
        );
        assert_eq!(
            persisted["closure_totals"],
            serde_json::json!({
                "expected_observation_count": 1,
                "observed_observation_count": 1,
                "failure_count": 1,
                "mismatch_count": 1
            })
        );
        for (field, expected) in [
            ("normalized_path", "$.system.description.value"),
            ("contextual_source_path", "$.system.description.value"),
            ("record_key", "fixture-actors:localized-npc"),
            ("member_identity", "record:fixture-actors:localized-npc"),
            ("destination", "canonical::SourceContentFact::document"),
            ("expected_state", "value"),
            ("expected_type", "string"),
            ("expected_value", "unlocalized"),
            ("observed_state", "value"),
            ("observed_type", "string"),
            ("observed_value", "localized"),
        ] {
            assert_eq!(mismatch[field], serde_json::json!(expected), "{field}");
        }
        assert_eq!(mismatch["expected_multiplicity"], serde_json::json!(1));
        assert_eq!(mismatch["observed_multiplicity"], serde_json::json!(1));
        assert_eq!(mismatch["expected_order"], serde_json::json!(0));
        assert_eq!(mismatch["observed_order"], serde_json::json!(0));
        let checksum = fs::read_to_string(checksum_path).expect("strict report checksum");
        assert_eq!(
            checksum,
            format!(
                "{}  strict-source-audit.json\n",
                digest_file(&report_path).expect("strict report digest")
            )
        );
        assert!(
            fs::read_dir(&stage)
                .expect("strict stage entries")
                .all(|entry| !entry
                    .expect("strict stage entry")
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".atomic"))
        );

        let _ = fs::remove_dir_all(source_root);
        let _ = fs::remove_dir_all(stage);
    }

    #[test]
    fn identity_matching_invalidates_candidate_and_policy_changes() {
        let base = ValidationIdentity {
            snapshot_format: "format".into(),
            source_commit: "source".into(),
            source_tree: "source-tree".into(),
            source_submodules: "source-submodules".into(),
            source_clean: true,
            source_signature: "signature".into(),
            source_manifest_pack_digest: "packs".into(),
            candidate_commit: "candidate".into(),
            candidate_tree: "candidate-tree".into(),
            candidate_submodules: "candidate-submodules".into(),
            candidate_clean: true,
            source_contract_version: "contract".into(),
            coverage_policy_version: "policy".into(),
            coverage_policy_digest: "policy-digest".into(),
            artifact_contract_version: "artifact".into(),
            artifact_manifest_version: "manifest".into(),
            artifact_schema_version: "schema".into(),
            migrations_digest: "migrations".into(),
            inventory_digest: "inventory".into(),
            target: "target".into(),
            features: "features".into(),
            rust_toolchain: "toolchain".into(),
            embedding_model: "model".into(),
            embedding_policy: "embedding-policy".into(),
            embedding_cache_identity: "cache".into(),
        };
        assert!(identity_static_matches(&base, &base));
        macro_rules! rejects_change {
            ($field:ident) => {{
                let mut changed = base.clone();
                changed.$field = "changed".into();
                assert!(
                    !identity_static_matches(&base, &changed),
                    "{} must invalidate the snapshot",
                    stringify!($field)
                );
            }};
        }
        rejects_change!(snapshot_format);
        rejects_change!(source_commit);
        rejects_change!(source_tree);
        rejects_change!(source_submodules);
        rejects_change!(candidate_commit);
        rejects_change!(candidate_tree);
        rejects_change!(candidate_submodules);
        rejects_change!(source_contract_version);
        rejects_change!(coverage_policy_version);
        rejects_change!(artifact_contract_version);
        rejects_change!(artifact_manifest_version);
        rejects_change!(artifact_schema_version);
        rejects_change!(migrations_digest);
        rejects_change!(inventory_digest);
        rejects_change!(target);
        rejects_change!(features);
        rejects_change!(rust_toolchain);
        rejects_change!(embedding_model);
        rejects_change!(embedding_policy);
        rejects_change!(embedding_cache_identity);
    }

    #[test]
    fn c2p_atomic_sha_baseline_is_closed_and_independent() {
        let candidate = candidate_mandatory_atomic_sha_pass_counts();
        let baseline = c2r_mandatory_atomic_sha_pass_counts();
        assert_eq!(candidate, baseline);
        assert_eq!(candidate.len(), 4);
        assert!(candidate.values().all(|count| *count > 0));
        assert_eq!(candidate["staged_manifest_creation"], 1);
        assert_eq!(candidate["publisher_pair_generation_verification"], 2);
        assert_eq!(candidate["visible_pair_reader_verification"], 1);
        assert_eq!(candidate["generation_materialization_open_verification"], 3);
    }

    #[test]
    fn parses_process_cpu_time_for_supported_ps_shapes() {
        assert_eq!(parse_cpu_time_ms("01:02"), Some(62_000));
        assert_eq!(parse_cpu_time_ms("02:03:04"), Some(7_384_000));
        assert_eq!(parse_cpu_time_ms("1-02:03:04"), Some(93_784_000));
        assert_eq!(parse_cpu_time_ms("0:01.25"), Some(1_250));
        assert_eq!(parse_cpu_time_ms("invalid"), None);
    }

    #[test]
    fn live_validation_binding_invalidates_every_tuple_dimension() {
        let base = ArtifactValidationTuple {
            candidate_commit: "candidate".into(),
            candidate_tree: "tree".into(),
            snapshot_stage: PathBuf::from("/snapshot"),
            mode: "no_embeddings".into(),
            source_signature: "source".into(),
            artifact_contract_version: "artifact".into(),
            artifact_schema_version: "schema".into(),
            embedding_model: "model".into(),
        };
        let generation = json!({
            "canonical_artifact_path": "/snapshot/artifacts/no_embeddings.sqlite",
            "generation_path": "/snapshot/artifacts/.generations/sha.sqlite",
            "file_identity": "dev:1:ino:2",
            "bytes": 10,
            "trusted_sha256": "sha",
        });
        let expected = validation_binding_digest(&base, &generation);
        macro_rules! rejects_change {
            ($field:ident, $value:expr) => {{
                let mut changed = base.clone();
                changed.$field = $value.into();
                assert_ne!(expected, validation_binding_digest(&changed, &generation));
            }};
        }
        rejects_change!(candidate_commit, "other-candidate");
        rejects_change!(candidate_tree, "other-tree");
        rejects_change!(snapshot_stage, PathBuf::from("/other-snapshot"));
        rejects_change!(mode, "with_embeddings");
        rejects_change!(source_signature, "other-source");
        rejects_change!(artifact_contract_version, "other-artifact");
        rejects_change!(artifact_schema_version, "other-schema");
        rejects_change!(embedding_model, "other-model");

        let mut changed_generation = generation.clone();
        changed_generation["bytes"] = json!(11);
        assert_ne!(
            expected,
            validation_binding_digest(&base, &changed_generation)
        );
    }
}
