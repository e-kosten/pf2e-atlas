//! Private validation-pipeline state. Nothing in this module is a product
//! artifact, runtime input, fallback, or public serialization contract.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use atlas_embedding::{
    EmbeddingModelId, EmbeddingRuntimeConfig, embedding_model_spec,
    required_embedding_model_cache_files,
};
use atlas_index::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, ArtifactValidationReport,
    IndexInspectionReport, SqliteIndexReader, ValidationStatus, ValidationTarget,
};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::artifact_manifest::{
    ARTIFACT_MANIFEST_VERSION, adjacent_artifact_manifest_path, read_artifact_manifest,
};
use crate::build::build_artifact_from_source;
use crate::error::IngestError;
use crate::source::dto::PF2E_SOURCE_CONTRACT_VERSION;
use crate::source::model::{BuildArtifactOptions, SkippedRecord};
use crate::source_pipeline;

const SNAPSHOT_FORMAT: &str = "pf2e-atlas-validation-snapshot/v3";
const VALIDATION_POLICY_VERSION: &str = "single-embedded-production-validation/v3";
const EMBEDDED_MODE: &str = "with_embeddings";
const SELECTED_RECORD_SMOKE: [(&str, &str); 2] = [
    ("pathfinder-bestiary:WQy7HBUcgDLsfVJd", "Night Hag"),
    ("pathfinder-monster-core:iIJPJcDT8wlJ8z5M", "Giant Rat"),
];

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
    embedding: ValidationEmbeddingIdentity,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct PersistedArtifactPairEvidence {
    artifact_path: PathBuf,
    manifest_path: PathBuf,
    lock_path: PathBuf,
    artifact_sha256: String,
    artifact_bytes: u64,
    document_embedding_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationEmbeddingIdentity {
    requested_selector: String,
    model: EmbeddingModelId,
    canonical_model_id: String,
}

impl ValidationEmbeddingIdentity {
    fn semantic_enum_identity(&self) -> String {
        format!("EmbeddingModelId::{:?}", self.model)
    }
}

impl Serialize for ValidationEmbeddingIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ValidationEmbeddingIdentity", 3)?;
        state.serialize_field("requested_selector", &self.requested_selector)?;
        state.serialize_field("semantic_enum_identity", &self.semantic_enum_identity())?;
        state.serialize_field("canonical_model_id", &self.canonical_model_id)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct TypedIdentityValue {
    raw_value: Option<String>,
    semantic_enum_identity: Option<String>,
    canonical_model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ValidationFailureDetail {
    error_code: String,
    tuple_field: Option<String>,
    requested_selector: Option<String>,
    canonical_model_id: Option<String>,
    semantic_enum_identity: Option<String>,
    expected: Option<TypedIdentityValue>,
    actual: Option<TypedIdentityValue>,
}

type ValidationFailure = Box<ValidationFailureDetail>;

#[derive(Debug, Clone, Serialize)]
struct OperationEvidence {
    order: usize,
    phase: String,
    status: String,
    wall_ms: u128,
    cpu_ms: Option<u128>,
    bytes: u64,
    counters: BTreeMap<String, usize>,
}

struct ActiveOperation {
    order: usize,
    phase: String,
    started: Instant,
    cpu_ms: Option<u128>,
    bytes: u64,
    counters: BTreeMap<String, usize>,
}

struct ValidationRunJournal {
    started: Instant,
    cpu_ms: Option<u128>,
    next_order: usize,
    current_mode: Option<String>,
    completed: Vec<OperationEvidence>,
    active: BTreeMap<String, ActiveOperation>,
    failure_detail: Option<ValidationFailureDetail>,
    artifact_identities: BTreeMap<String, Value>,
    counter_state: BTreeMap<String, Value>,
}

impl ValidationRunJournal {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            cpu_ms: process_cpu_time_ms(),
            next_order: 1,
            current_mode: None,
            completed: Vec::new(),
            active: BTreeMap::new(),
            failure_detail: None,
            artifact_identities: BTreeMap::new(),
            counter_state: BTreeMap::from([
                ("source_traversal_count".to_string(), json!(0)),
                ("completed_artifact_modes".to_string(), json!([])),
            ]),
        }
    }

    fn progress(&mut self, stage: &Path, phase: &str, status: &str) -> Result<(), IngestError> {
        if status == "started" {
            if phase == EMBEDDED_MODE {
                self.current_mode = Some(phase.to_string());
            }
            let operation = ActiveOperation {
                order: self.next_order,
                phase: phase.to_string(),
                started: Instant::now(),
                cpu_ms: process_cpu_time_ms(),
                bytes: 0,
                counters: BTreeMap::new(),
            };
            self.next_order += 1;
            self.active.insert(phase.to_string(), operation);
        } else if let Some(operation) = self.active.remove(phase) {
            self.completed.push(operation.finish(status));
            if phase == EMBEDDED_MODE && status == "passed" {
                let completed = self
                    .completed
                    .iter()
                    .filter(|operation| {
                        operation.status == "passed" && operation.phase == EMBEDDED_MODE
                    })
                    .map(|operation| operation.phase.clone())
                    .collect::<Vec<_>>();
                self.counter_state
                    .insert("completed_artifact_modes".to_string(), json!(completed));
                self.current_mode = None;
            }
        }
        write_progress_entry(
            stage,
            &json!({
                "phase": phase,
                "status": status,
                "order": self.next_order.saturating_sub(1),
                "wall_ms": self.started.elapsed().as_millis(),
                "cpu_ms": self.elapsed_cpu_ms(),
                "mode": self.current_mode,
            }),
        )
    }

    fn complete_source_traversal(&mut self) {
        self.counter_state
            .insert("source_traversal_count".to_string(), json!(1));
    }

    fn annotate_operation(
        &mut self,
        phase: &str,
        bytes: u64,
        counters: impl IntoIterator<Item = (String, usize)>,
    ) {
        if let Some(operation) = self.active.get_mut(phase) {
            operation.bytes = bytes;
            operation.counters.extend(counters);
        }
    }

    fn record_artifact_identity(
        &mut self,
        mode: &str,
        visible_path: &Path,
        generation_path: &Path,
        trusted_sha256: &str,
        bytes: u64,
    ) {
        self.artifact_identities.insert(
            mode.to_string(),
            json!({
                "visible_path": visible_path,
                "generation_path": generation_path,
                "trusted_sha256": trusted_sha256,
                "bytes": bytes,
            }),
        );
    }

    fn fail_with(&mut self, detail: ValidationFailureDetail) {
        self.failure_detail = Some(detail);
    }

    fn elapsed_cpu_ms(&self) -> Option<u128> {
        self.cpu_ms
            .zip(process_cpu_time_ms())
            .map(|(start, end)| end.saturating_sub(start))
    }

    fn in_progress(&self) -> Vec<OperationEvidence> {
        let mut active = self
            .active
            .values()
            .map(ActiveOperation::in_progress)
            .collect::<Vec<_>>();
        active.sort_by_key(|operation| operation.order);
        active
    }

    fn failure_point(&self) -> Option<String> {
        self.completed
            .iter()
            .rev()
            .find(|operation| operation.status == "failed")
            .map(|operation| operation.phase.clone())
            .or_else(|| {
                self.active
                    .values()
                    .max_by_key(|operation| operation.order)
                    .map(|operation| operation.phase.clone())
            })
    }

    fn failure_order(&self) -> usize {
        self.completed
            .iter()
            .rev()
            .find(|operation| operation.status == "failed")
            .map_or_else(
                || {
                    self.active
                        .values()
                        .map(|operation| operation.order)
                        .max()
                        .unwrap_or(self.next_order)
                },
                |operation| operation.order,
            )
    }
}

impl ActiveOperation {
    fn finish(self, status: &str) -> OperationEvidence {
        OperationEvidence {
            order: self.order,
            phase: self.phase,
            status: status.to_string(),
            wall_ms: self.started.elapsed().as_millis(),
            cpu_ms: self
                .cpu_ms
                .zip(process_cpu_time_ms())
                .map(|(start, end)| end.saturating_sub(start)),
            bytes: self.bytes,
            counters: self.counters,
        }
    }

    fn in_progress(&self) -> OperationEvidence {
        OperationEvidence {
            order: self.order,
            phase: self.phase.clone(),
            status: "in_progress".to_string(),
            wall_ms: self.started.elapsed().as_millis(),
            cpu_ms: self
                .cpu_ms
                .zip(process_cpu_time_ms())
                .map(|(start, end)| end.saturating_sub(start)),
            bytes: self.bytes,
            counters: self.counters.clone(),
        }
    }
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
struct ArtifactValidationReceipt {
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

    fn validation_receipt(
        &self,
        target: ValidationTarget,
    ) -> Result<ArtifactValidationReceipt, IngestError> {
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
        Ok(ArtifactValidationReceipt {
            binding_digest: self.binding_digest.clone(),
            target,
            report,
        })
    }

    fn inspect(
        &self,
        receipt: &ArtifactValidationReceipt,
    ) -> Result<IndexInspectionReport, IngestError> {
        if receipt.binding_digest != self.binding_digest {
            return Err(validation_error(
                "validation receipt does not belong to this live generation handle",
            ));
        }
        self.reader
            .inspect_with_validation_report(receipt.report.clone())
            .map_err(|error| validation_error(error.to_string()))
    }

    fn load_hydrated_records_by_key(
        &self,
        keys: &[atlas_domain::RecordKey],
    ) -> Result<Vec<atlas_record::RetrievedRecord>, IngestError> {
        self.reader
            .load_hydrated_records_by_key(keys)
            .map_err(|error| validation_error(error.to_string()))
    }
}

fn validation_binding_digest(tuple: &ArtifactValidationTuple, generation: &Value) -> String {
    digest_debug(&(
        &tuple.candidate_commit,
        &tuple.candidate_tree,
        &tuple.snapshot_stage,
        &tuple.mode,
        &tuple.source_signature,
        &tuple.artifact_contract_version,
        &tuple.artifact_schema_version,
        tuple.embedding.model,
        &tuple.embedding.canonical_model_id,
        generation,
    ))
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
    pub source_signature: String,
    pub candidate_commit: String,
    pub timing: BTreeMap<String, u128>,
    pub resources: Value,
    pub assertion_inventory: Vec<AssertionInventoryEntry>,
    pub artifact_mode_reports: BTreeMap<String, Value>,
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
    let repository_root = current_repo_root()?;
    let candidate = git_identity(&repository_root, "candidate")?;
    if candidate.commit != options.candidate_head {
        return Err(validation_error(format!(
            "candidate HEAD mismatch: expected {}, found {}",
            options.candidate_head, candidate.commit
        )));
    }
    let source_git = git_identity(&options.source_root, "source")?;

    if options.snapshot_root.exists() {
        if options.force_reproduction {
            return Err(validation_error(format!(
                "force reproduction refuses existing snapshot {}",
                options.snapshot_root.display()
            )));
        }
        let embedding = resolve_validation_embedding_identity(
            &BuildArtifactOptions::default_embedding_model_id(),
        )
        .map_err(|detail| validation_error(failure_detail_message(&detail)))?;
        let manifest = validate_snapshot_manifest(&options.snapshot_root, None)?;
        if !identity_matches_before_embedding_hash(
            &manifest.identity,
            &options,
            &candidate,
            &source_git,
            &repository_root,
            &embedding,
        )? {
            return Err(validation_error(
                "existing validation snapshot identity does not match this candidate tuple",
            ));
        }
        let static_identity = static_identity(
            &options,
            &candidate,
            &source_git,
            &repository_root,
            &embedding,
        )?;
        if !identity_static_matches(&manifest.identity, &static_identity) {
            return Err(validation_error(
                "existing validation snapshot identity does not match this candidate tuple",
            ));
        }
        let manifest = validate_snapshot(&options.snapshot_root, Some(&manifest.identity), None)?;
        let report =
            report_from_snapshot(&options, &manifest, total_started.elapsed().as_millis())?;
        write_json_new(&options.report_path, &report)?;
        return Ok(report);
    }

    let stage = staging_path(&options.snapshot_root);
    fs::create_dir(&stage).map_err(io_error("create validation snapshot staging directory"))?;
    let mut journal = ValidationRunJournal::new();
    let mut trusted_artifact_digests = BTreeMap::new();
    let report = match (|| {
        journal.progress(&stage, "pre_artifact_identity", "started")?;
        let embedding = match resolve_validation_embedding_identity(
            &BuildArtifactOptions::default_embedding_model_id(),
        ) {
            Ok(identity) => identity,
            Err(detail) => {
                journal.fail_with((*detail).clone());
                journal.progress(&stage, "pre_artifact_identity", "failed")?;
                return Err(validation_error(failure_detail_message(&detail)));
            }
        };
        let static_identity = static_identity(
            &options,
            &candidate,
            &source_git,
            &repository_root,
            &embedding,
        )?;
        let tuple_templates =
            match pre_artifact_validation_tuples(&static_identity, &stage, &embedding) {
                Ok(tuples) => tuples,
                Err(detail) => {
                    journal.fail_with((*detail).clone());
                    journal.progress(&stage, "pre_artifact_identity", "failed")?;
                    return Err(validation_error(failure_detail_message(&detail)));
                }
            };
        journal.progress(&stage, "pre_artifact_identity", "passed")?;
        build_snapshot(
            &options,
            &stage,
            static_identity,
            total_started,
            &tuple_templates,
            &mut journal,
            &mut trusted_artifact_digests,
        )
    })() {
        Ok(report) => report,
        Err(error) => {
            let failed = preserve_failed_snapshot(
                &options,
                &stage,
                &error,
                &journal,
                &trusted_artifact_digests,
            )?;
            return Err(validation_error(format!(
                "{error}; checksum-bound failed snapshot preserved at {}",
                failed.display()
            )));
        }
    };
    let publication = (|| {
        fs::rename(&stage, &options.snapshot_root)
            .map_err(io_error("atomically publish validation snapshot"))?;
        validate_snapshot(
            &options.snapshot_root,
            None,
            Some(&trusted_artifact_digests),
        )?;
        write_json_new(&options.report_path, &report)
    })();
    if let Err(error) = publication {
        let evidence_root = if stage.exists() {
            stage.as_path()
        } else {
            options.snapshot_root.as_path()
        };
        let failed = preserve_failed_snapshot(
            &options,
            evidence_root,
            &error,
            &journal,
            &trusted_artifact_digests,
        )?;
        return Err(validation_error(format!(
            "{error}; checksum-bound failed snapshot preserved at {}",
            failed.display()
        )));
    }
    Ok(report)
}

fn validation_artifact_outputs(artifacts: &Path) -> BTreeMap<&'static str, PathBuf> {
    BTreeMap::from([(
        EMBEDDED_MODE,
        artifacts.join(EMBEDDED_MODE).join("index.sqlite"),
    )])
}

fn adjacent_artifact_lock_path(artifact: &Path) -> PathBuf {
    let manifest = adjacent_artifact_manifest_path(artifact);
    let mut lock = manifest.as_os_str().to_os_string();
    lock.push(".pair.lock");
    PathBuf::from(lock)
}

fn persisted_artifact_pair_evidence(
    stage: &Path,
    artifact: &Path,
    trusted_sha256: &str,
    expected_bytes: u64,
    expected_document_embedding_count: usize,
) -> Result<PersistedArtifactPairEvidence, IngestError> {
    let manifest_path = adjacent_artifact_manifest_path(artifact);
    let lock_path = adjacent_artifact_lock_path(artifact);
    let artifact_bytes = fs::metadata(artifact)
        .map_err(io_error("read retained validation artifact size"))?
        .len();
    if artifact_bytes != expected_bytes {
        return Err(validation_error(format!(
            "retained artifact size changed: expected {expected_bytes}, found {artifact_bytes}"
        )));
    }
    let manifest = read_artifact_manifest(&manifest_path)?;
    if manifest.build.artifact_sha256 != trusted_sha256 {
        return Err(validation_error(format!(
            "retained artifact manifest cross-binding: expected SHA-256 {trusted_sha256}, found {}",
            manifest.build.artifact_sha256
        )));
    }
    if manifest.build.document_embedding_count != expected_document_embedding_count {
        return Err(validation_error(format!(
            "retained artifact manifest embedding count changed: expected {expected_document_embedding_count}, found {}",
            manifest.build.document_embedding_count
        )));
    }
    if !lock_path.is_file() {
        return Err(validation_error(format!(
            "retained artifact pair lock is missing: {}",
            lock_path.display()
        )));
    }

    let relative = |path: &Path| {
        path.strip_prefix(stage)
            .map(Path::to_path_buf)
            .map_err(|error| {
                validation_error(format!("artifact pair path escaped snapshot: {error}"))
            })
    };
    Ok(PersistedArtifactPairEvidence {
        artifact_path: relative(artifact)?,
        manifest_path: relative(&manifest_path)?,
        lock_path: relative(&lock_path)?,
        artifact_sha256: trusted_sha256.to_string(),
        artifact_bytes,
        document_embedding_count: manifest.build.document_embedding_count,
    })
}

fn build_snapshot(
    options: &ExhaustiveValidationOptions,
    stage: &Path,
    mut identity: ValidationIdentity,
    total_started: Instant,
    tuple_templates: &BTreeMap<String, ArtifactValidationTuple>,
    journal: &mut ValidationRunJournal,
    trusted_artifact_digests: &mut BTreeMap<PathBuf, String>,
) -> Result<ExhaustiveValidationReport, IngestError> {
    let mut timing = BTreeMap::new();
    journal.progress(stage, "source_traversal", "started")?;
    let phase = Instant::now();
    let source = source_pipeline::load_foundry_source(&options.source_root, None)?;
    if let Err(error) = require_complete_source_admission(&source.skipped_records, &source.warnings)
    {
        journal.progress(stage, "source_traversal", "failed")?;
        return Err(error);
    }
    journal.annotate_operation(
        "source_traversal",
        0,
        [(
            "source_record_count".to_string(),
            source.source_record_count,
        )],
    );
    timing.insert(
        "source_traversal_ms".to_string(),
        phase.elapsed().as_millis(),
    );
    identity.source_signature = source.source_signature.clone();
    identity.source_manifest_pack_digest = digest_debug(&source.packs);
    let source_record_count = source.source_record_count;
    let source_file_count = source.source_record_count + source.skipped_records.len();
    let source_pack_count = source.packs.len();
    journal.complete_source_traversal();
    journal.progress(stage, "source_traversal", "passed")?;

    let artifacts = stage.join("artifacts");
    fs::create_dir(&artifacts).map_err(io_error("create validation artifact directory"))?;
    let artifact_outputs = validation_artifact_outputs(&artifacts);
    let mut artifact_reports = BTreeMap::new();
    let mode = EMBEDDED_MODE;
    let cache = Some(options.embedding_cache_root.clone());
    journal.progress(stage, mode, "started")?;
    let phase = OperationClock::start();
    let phase_started = Instant::now();
    let output = artifact_outputs
        .get(mode)
        .cloned()
        .ok_or_else(|| validation_error(format!("missing artifact output for {mode}")))?;
    fs::create_dir(
        output
            .parent()
            .ok_or_else(|| validation_error(format!("missing artifact parent for {mode}")))?,
    )
    .map_err(io_error("create validation artifact mode directory"))?;
    journal.progress(stage, &format!("{mode}.build_write_publish"), "started")?;
    let build_clock = OperationClock::start();
    let build = build_artifact_from_source(
        source,
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
    journal.annotate_operation(mode, artifact_bytes, [("artifact_count".to_string(), 1)]);
    journal.annotate_operation(
        &format!("{mode}.build_write_publish"),
        artifact_bytes,
        [("artifact_count".to_string(), 1)],
    );
    let build_timing = build_clock.finish(
        artifact_bytes,
        1,
        "complete build/write/manifest/publication boundary",
    );
    journal.progress(stage, &format!("{mode}.build_write_publish"), "passed")?;
    let reader_work_started = Instant::now();
    journal.progress(stage, &format!("{mode}.reader_open"), "started")?;
    journal.annotate_operation(
        &format!("{mode}.reader_open"),
        artifact_bytes,
        [
            ("reader_count".to_string(), 1),
            ("diesel_connection_count".to_string(), 1),
            ("rusqlite_connection_count".to_string(), 1),
        ],
    );
    let reader_clock = OperationClock::start();
    let mut tuple = tuple_templates
        .get(mode)
        .cloned()
        .ok_or_else(|| validation_error(format!("missing pre-artifact tuple for {mode}")))?;
    tuple
        .source_signature
        .clone_from(&identity.source_signature);
    let handle = VerifiedArtifactGenerationHandle::open(&output, true, tuple)?;
    let reader_timing = reader_clock.finish(
        artifact_bytes,
        1,
        "visible pair verification, generation binding, and both retained connections",
    );
    let generation = handle.generation.clone();
    let trusted_sha256 = generation["trusted_sha256"]
        .as_str()
        .ok_or_else(|| validation_error("verified generation evidence has no trusted SHA-256"))?
        .to_string();
    let visible_relative = output
        .strip_prefix(stage)
        .map_err(|error| validation_error(format!("artifact path escaped snapshot: {error}")))?
        .to_path_buf();
    let generation_path = Path::new(
        generation["generation_path"]
            .as_str()
            .ok_or_else(|| validation_error("verified generation evidence has no path"))?,
    );
    let generation_relative = generation_path
        .strip_prefix(stage)
        .map_err(|error| validation_error(format!("generation path escaped snapshot: {error}")))?
        .to_path_buf();
    trusted_artifact_digests.insert(visible_relative.clone(), trusted_sha256.clone());
    trusted_artifact_digests.insert(generation_relative.clone(), trusted_sha256.clone());
    let pair_evidence = persisted_artifact_pair_evidence(
        stage,
        &output,
        &trusted_sha256,
        artifact_bytes,
        build.document_embedding_count,
    )?;
    journal.record_artifact_identity(
        mode,
        &visible_relative,
        &generation_relative,
        &trusted_sha256,
        artifact_bytes,
    );
    journal.progress(stage, &format!("{mode}.reader_open"), "passed")?;

    let target = ValidationTarget::Full;
    journal.progress(
        stage,
        &format!("{mode}.structural_global_validation"),
        "started",
    )?;
    journal.annotate_operation(
        &format!("{mode}.structural_global_validation"),
        artifact_bytes,
        [("structural_global_receipt_count".to_string(), 1)],
    );
    let validation_clock = OperationClock::start();
    let receipt = handle.validation_receipt(target)?;
    if let Err(detail) = validate_receipt_metadata(&handle.tuple, &receipt.report) {
        journal.fail_with((*detail).clone());
        journal.progress(
            stage,
            &format!("{mode}.structural_global_validation"),
            "failed",
        )?;
        return Err(validation_error(failure_detail_message(&detail)));
    }
    let validation_timing = validation_clock.finish(
        artifact_bytes,
        1,
        "single post-publication structural and global validation",
    );
    journal.progress(
        stage,
        &format!("{mode}.structural_global_validation"),
        "passed",
    )?;
    journal.progress(stage, &format!("{mode}.inspect_projection"), "started")?;
    let inspect_clock = OperationClock::start();
    let inspect = handle.inspect(&receipt)?;
    let inspect_timing = inspect_clock.finish(
        0,
        1,
        "inspection projection from the live validation receipt",
    );
    journal.progress(stage, &format!("{mode}.inspect_projection"), "passed")?;
    if receipt.target != target {
        return Err(validation_error(format!(
            "{mode} structural/global validation receipt target changed"
        )));
    }
    let structural_global_validation = receipt.report.clone();
    journal.progress(stage, &format!("{mode}.selected_record_smoke"), "started")?;
    let selected_record_clock = OperationClock::start();
    let selected_record_smoke = validate_selected_record_smoke(&handle)?;
    let selected_record_timing = selected_record_clock.finish(
        artifact_bytes,
        selected_record_smoke.len(),
        "bounded keyed hydration through the public reader path",
    );
    journal.progress(stage, &format!("{mode}.selected_record_smoke"), "passed")?;
    if structural_global_validation.status != ValidationStatus::Ok {
        return Err(validation_error(format!(
            "{mode} artifact validation failed"
        )));
    }
    journal.progress(stage, &format!("{mode}.evidence_projection"), "started")?;
    let evidence_clock = OperationClock::start();
    let handle_tuple = handle.tuple.clone();
    let reader_work_ms = reader_work_started.elapsed().as_millis();
    let evidence_timing = evidence_clock.finish(0, 1, "validation evidence serialization");
    journal.progress(stage, &format!("{mode}.evidence_projection"), "passed")?;
    journal.progress(stage, &format!("{mode}.reader_close"), "started")?;
    let close_clock = OperationClock::start();
    drop(receipt);
    drop(handle);
    let close_timing = close_clock.finish(0, 1, "receipt, connections, and generation lease close");
    journal.progress(stage, &format!("{mode}.reader_close"), "passed")?;
    let phase_timing = phase.finish(artifact_bytes, 1, "complete artifact-mode validation phase");
    let phase_ms = phase_started.elapsed().as_millis();
    let phase_seconds = phase_ms as f64 / 1000.0;
    let reader_work_seconds = reader_work_ms as f64 / 1000.0;
    let operations = json!({
        "build_write_publish": build_timing,
        "generation_bound_reader_open": reader_timing,
        "structural_global_validation": validation_timing,
        "inspect_projection": inspect_timing,
        "selected_record_public_path_smoke": selected_record_timing,
        "evidence_serialization": evidence_timing,
        "close": close_timing,
        "phase": phase_timing,
    });
    artifact_reports.insert(
        mode.to_string(),
        json!({
            "build": build_report_json(&build),
            "inspect": inspect,
            "structural_global_validation": structural_global_validation,
            "selected_record_smoke": selected_record_smoke,
            "verified_generation": generation,
            "verified_handle_tuple": handle_tuple,
            "artifact_pair": pair_evidence,
            "trusted_snapshot_artifact_paths": [visible_relative, generation_relative],
            "phase_seconds": phase_seconds,
            "reader_work_seconds": reader_work_seconds,
            "operation_timings": operations,
        }),
    );
    timing.insert(format!("artifact_{mode}_ms"), phase_ms);
    journal.progress(stage, mode, "passed")?;
    write_json(stage.join("artifact-validation.json"), &artifact_reports)?;

    let assertions = assertion_inventory();
    let assertions_report = json!({
        "status": "pass",
        "source_traversal_count": 1,
        "artifact_modes": [EMBEDDED_MODE],
        "assertion_inventory_complete": true,
        "assertions": assertions,
        "semantic_changes": [],
    });
    write_json(stage.join("validation-assertions.json"), &assertions_report)?;
    timing.insert("total_ms".to_string(), total_started.elapsed().as_millis());
    let resources = json!({
        "source_records": source_record_count,
        "artifact_records": build.artifact_record_count,
        "packs": source_pack_count,
        "source_files": source_file_count,
        "peak_rss_bytes": Value::Null,
        "peak_rss_note": "not exposed portably by the in-process validator",
    });
    write_json(
        stage.join("timing.json"),
        &json!({"phases_ms": timing, "resources": resources}),
    )?;

    let required_files = vec![
        "validation-assertions.json".to_string(),
        "artifact-validation.json".to_string(),
        "timing.json".to_string(),
        "progress.jsonl".to_string(),
    ];
    write_json(
        stage.join("snapshot-manifest.json"),
        &SnapshotManifest {
            identity: identity.clone(),
            complete: true,
            source_traversal_count: 1,
            artifact_modes: vec![EMBEDDED_MODE.to_string()],
            required_files,
        },
    )?;
    write_optional_file_sizes(stage);
    write_checksums(stage, trusted_artifact_digests)?;

    Ok(ExhaustiveValidationReport {
        status: "pass".to_string(),
        source_traversal_count: 1,
        artifact_modes: vec![EMBEDDED_MODE.to_string()],
        assertion_inventory_complete: true,
        semantic_changes: Vec::new(),
        snapshot_root: options.snapshot_root.display().to_string(),
        snapshot_reused: false,
        source_signature: identity.source_signature,
        candidate_commit: identity.candidate_commit,
        timing,
        resources,
        assertion_inventory: assertions,
        artifact_mode_reports: artifact_reports,
    })
}

fn require_complete_source_admission(
    skipped_records: &[SkippedRecord],
    warnings: &[String],
) -> Result<(), IngestError> {
    if skipped_records.is_empty() && warnings.is_empty() {
        return Ok(());
    }

    let first_skipped = skipped_records.first().map_or_else(
        || "none".to_string(),
        |skipped| format!("{}: {}", skipped.path.display(), skipped.reason),
    );
    let first_warning = warnings.first().map_or("none", String::as_str);
    Err(validation_error(format!(
        "production validation requires complete source admission: {} skipped record(s), {} warning(s); first skipped: {first_skipped}; first warning: {first_warning}",
        skipped_records.len(),
        warnings.len(),
    )))
}

fn validate_selected_record_smoke(
    handle: &VerifiedArtifactGenerationHandle,
) -> Result<Vec<Value>, IngestError> {
    let keys = SELECTED_RECORD_SMOKE
        .iter()
        .map(|(key, _)| {
            atlas_domain::RecordKey::parse(key)
                .map_err(|error| validation_error(format!("invalid selected smoke key: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let records = handle.load_hydrated_records_by_key(&keys)?;
    if records.len() != SELECTED_RECORD_SMOKE.len() {
        return Err(validation_error(format!(
            "selected-record smoke expected {} records, found {}",
            SELECTED_RECORD_SMOKE.len(),
            records.len()
        )));
    }

    SELECTED_RECORD_SMOKE
        .iter()
        .map(|(expected_key, expected_name)| {
            let record = records
                .iter()
                .find(|record| record.record.identity.key.to_string() == *expected_key)
                .ok_or_else(|| {
                    validation_error(format!(
                        "selected-record smoke did not return `{expected_key}`"
                    ))
                })?;
            if record.record.identity.name != *expected_name {
                return Err(validation_error(format!(
                    "selected-record smoke expected `{expected_key}` to be named `{expected_name}`, found `{}`",
                    record.record.identity.name
                )));
            }
            let Some(atlas_record::RecordBody::Creature(body)) = record.body.as_ref() else {
                return Err(validation_error(format!(
                    "selected-record smoke expected a creature body for `{expected_key}`"
                )));
            };
            if body.identity.record_key.to_string() != *expected_key {
                return Err(validation_error(format!(
                    "selected-record smoke body identity diverged for `{expected_key}`"
                )));
            }
            Ok(json!({
                "record_key": expected_key,
                "name": expected_name,
                "body": "creature",
            }))
        })
        .collect()
}

fn validate_receipt_metadata(
    tuple: &ArtifactValidationTuple,
    report: &ArtifactValidationReport,
) -> Result<(), ValidationFailure> {
    // Preserve the failure order: an invalid structural/global report is handed to
    // inspection first, which returns the existing InvalidArtifact payload.
    if report.status != ValidationStatus::Ok {
        return Ok(());
    }
    let expected_file = "index.sqlite";
    let expected_parent = tuple.snapshot_stage.join("artifacts").join(&tuple.mode);
    let report_path = Path::new(&report.index);
    for (field, expected, actual) in [
        (
            "candidate_commit",
            Some("non-empty"),
            (!tuple.candidate_commit.is_empty()).then_some("non-empty"),
        ),
        (
            "candidate_tree",
            Some("non-empty"),
            (!tuple.candidate_tree.is_empty()).then_some("non-empty"),
        ),
        (
            "artifact_file_name",
            Some(expected_file),
            report_path.file_name().and_then(|name| name.to_str()),
        ),
        (
            "artifact_parent",
            expected_parent.to_str(),
            report_path.parent().and_then(Path::to_str),
        ),
        (
            "artifact_contract_version",
            Some(tuple.artifact_contract_version.as_str()),
            report.artifact_contract_version.as_deref(),
        ),
        (
            "artifact_schema_version",
            Some(tuple.artifact_schema_version.as_str()),
            report.schema_version.as_deref(),
        ),
        (
            "source_signature",
            Some(tuple.source_signature.as_str()),
            report.source_signature.as_deref(),
        ),
    ] {
        if actual != expected {
            return Err(receipt_identity_mismatch(
                tuple,
                field,
                expected.map(str::to_string),
                actual.map(str::to_string),
                None,
            ));
        }
    }

    let actual_raw = report.embedding_model_id.as_deref();
    let actual_model = actual_raw.and_then(|value| EmbeddingModelId::from_str(value).ok());
    let actual_canonical = actual_model.map(|model| embedding_model_spec(model).model_id);
    if actual_model != Some(tuple.embedding.model)
        || actual_raw != Some(tuple.embedding.canonical_model_id.as_str())
        || actual_canonical != Some(tuple.embedding.canonical_model_id.as_str())
    {
        return Err(receipt_identity_mismatch(
            tuple,
            "embedding_model",
            Some(tuple.embedding.canonical_model_id.clone()),
            actual_raw.map(str::to_string),
            actual_model,
        ));
    }
    Ok(())
}

fn resolve_validation_embedding_identity(
    selector: &str,
) -> Result<ValidationEmbeddingIdentity, ValidationFailure> {
    let model = EmbeddingModelId::from_str(selector).map_err(|_| {
        Box::new(ValidationFailureDetail {
            error_code: "validation_embedding_selector_invalid".to_string(),
            tuple_field: Some("embedding_model".to_string()),
            requested_selector: Some(selector.to_string()),
            canonical_model_id: None,
            semantic_enum_identity: None,
            expected: None,
            actual: Some(TypedIdentityValue {
                raw_value: Some(selector.to_string()),
                semantic_enum_identity: None,
                canonical_model_id: None,
            }),
        })
    })?;
    let spec = embedding_model_spec(model);
    Ok(ValidationEmbeddingIdentity {
        requested_selector: selector.to_string(),
        model,
        canonical_model_id: spec.model_id.to_string(),
    })
}

fn pre_artifact_validation_tuples(
    identity: &ValidationIdentity,
    stage: &Path,
    embedding: &ValidationEmbeddingIdentity,
) -> Result<BTreeMap<String, ArtifactValidationTuple>, ValidationFailure> {
    let fields = [
        ("candidate_commit", !identity.candidate_commit.is_empty()),
        ("candidate_tree", !identity.candidate_tree.is_empty()),
        (
            "artifact_contract_version",
            !identity.artifact_contract_version.is_empty(),
        ),
        (
            "artifact_schema_version",
            !identity.artifact_schema_version.is_empty(),
        ),
        ("snapshot_stage", stage.is_dir()),
        (
            "embedding_model",
            identity.embedding_model == embedding.canonical_model_id
                && embedding_model_spec(embedding.model).model_id
                    == embedding.canonical_model_id.as_str(),
        ),
    ];
    if let Some((field, _)) = fields.into_iter().find(|(_, valid)| !valid) {
        return Err(Box::new(ValidationFailureDetail {
            error_code: "validation_pre_artifact_identity_mismatch".to_string(),
            tuple_field: Some(field.to_string()),
            requested_selector: Some(embedding.requested_selector.clone()),
            canonical_model_id: Some(embedding.canonical_model_id.clone()),
            semantic_enum_identity: Some(embedding.semantic_enum_identity()),
            expected: None,
            actual: None,
        }));
    }
    let snapshot_stage = fs::canonicalize(stage).map_err(|_| {
        Box::new(ValidationFailureDetail {
            error_code: "validation_pre_artifact_identity_mismatch".to_string(),
            tuple_field: Some("snapshot_stage".to_string()),
            requested_selector: Some(embedding.requested_selector.clone()),
            canonical_model_id: Some(embedding.canonical_model_id.clone()),
            semantic_enum_identity: Some(embedding.semantic_enum_identity()),
            expected: None,
            actual: None,
        })
    })?;
    Ok([EMBEDDED_MODE]
        .into_iter()
        .map(|mode| {
            (
                mode.to_string(),
                ArtifactValidationTuple {
                    candidate_commit: identity.candidate_commit.clone(),
                    candidate_tree: identity.candidate_tree.clone(),
                    snapshot_stage: snapshot_stage.clone(),
                    mode: mode.to_string(),
                    source_signature: String::new(),
                    artifact_contract_version: identity.artifact_contract_version.clone(),
                    artifact_schema_version: identity.artifact_schema_version.clone(),
                    embedding: embedding.clone(),
                },
            )
        })
        .collect())
}

fn receipt_identity_mismatch(
    tuple: &ArtifactValidationTuple,
    field: &str,
    expected_raw: Option<String>,
    actual_raw: Option<String>,
    actual_model: Option<EmbeddingModelId>,
) -> ValidationFailure {
    Box::new(ValidationFailureDetail {
        error_code: "validation_receipt_identity_mismatch".to_string(),
        tuple_field: Some(field.to_string()),
        requested_selector: Some(tuple.embedding.requested_selector.clone()),
        canonical_model_id: Some(tuple.embedding.canonical_model_id.clone()),
        semantic_enum_identity: Some(tuple.embedding.semantic_enum_identity()),
        expected: Some(TypedIdentityValue {
            raw_value: expected_raw,
            semantic_enum_identity: (field == "embedding_model")
                .then(|| tuple.embedding.semantic_enum_identity()),
            canonical_model_id: (field == "embedding_model")
                .then(|| tuple.embedding.canonical_model_id.clone()),
        }),
        actual: Some(TypedIdentityValue {
            raw_value: actual_raw,
            semantic_enum_identity: actual_model
                .map(|model| format!("EmbeddingModelId::{model:?}")),
            canonical_model_id: actual_model
                .map(|model| embedding_model_spec(model).model_id.to_string()),
        }),
    })
}

fn failure_detail_message(detail: &ValidationFailureDetail) -> String {
    format!(
        "{} at {}",
        detail.error_code,
        detail
            .tuple_field
            .as_deref()
            .unwrap_or("validation_pipeline")
    )
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

fn assertion_inventory() -> Vec<AssertionInventoryEntry> {
    vec![
        AssertionInventoryEntry {
            id: "stage_b_source_contract_mutations",
            owner: "atlas-ingest source DTO/audit tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "serialized_source_admission_and_presence",
            owner: "atlas-ingest source DTO and normalization tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "canonical_writer_reader_mutation_fixtures",
            owner: "atlas-ingest and atlas-index focused tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "missing_and_extra_canonical_bodies",
            owner: "atlas-ingest index_build_input tests",
            preserved_by: "just validate-focused",
        },
        AssertionInventoryEntry {
            id: "canonical_relationship_order_identity_and_faults",
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
            id: "embedded_artifact_structural_global_and_vector_validation",
            owner: "production validation orchestrator",
            preserved_by: "single embedded production build",
        },
        AssertionInventoryEntry {
            id: "selected_record_public_path_smoke",
            owner: "atlas-index keyed hydration",
            preserved_by: "single embedded production build",
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
    embedding: &ValidationEmbeddingIdentity,
) -> Result<ValidationIdentity, IngestError> {
    static_identity_with_cache_identity(
        candidate,
        source,
        repository_root,
        embedding,
        embedding_cache_identity(&options.embedding_cache_root, embedding.model)?,
    )
}

fn static_identity_with_cache_identity(
    candidate: &GitIdentity,
    source: &GitIdentity,
    repository_root: &Path,
    embedding: &ValidationEmbeddingIdentity,
    embedding_cache_identity: String,
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
        embedding_model: embedding.canonical_model_id.clone(),
        embedding_policy: "reuse=true;batch_size=32".to_string(),
        embedding_cache_identity,
    })
}

fn identity_matches_before_embedding_hash(
    actual: &ValidationIdentity,
    options: &ExhaustiveValidationOptions,
    candidate: &GitIdentity,
    source: &GitIdentity,
    repository_root: &Path,
    embedding: &ValidationEmbeddingIdentity,
) -> Result<bool, IngestError> {
    let canonical_cache_root = canonical_string(&options.embedding_cache_root)?;
    let Some(cache_digest) = actual
        .embedding_cache_identity
        .strip_prefix(&canonical_cache_root)
        .and_then(|suffix| suffix.strip_prefix(";required_files_sha256="))
    else {
        return Ok(false);
    };
    if cache_digest.len() != 64 || !cache_digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(false);
    }
    let expected = static_identity_with_cache_identity(
        candidate,
        source,
        repository_root,
        embedding,
        actual.embedding_cache_identity.clone(),
    )?;
    Ok(identity_static_matches(actual, &expected))
}

fn embedding_cache_identity(
    cache_root: &Path,
    model: EmbeddingModelId,
) -> Result<String, IngestError> {
    let canonical_root = canonical_string(cache_root)?;
    let config = EmbeddingRuntimeConfig::new(model, cache_root);
    let mut hasher = Sha256::new();
    for file in required_embedding_model_cache_files(&config) {
        let metadata = fs::symlink_metadata(&file.local_path)
            .map_err(io_error("read required embedding cache file metadata"))?;
        if !metadata.file_type().is_file() {
            return Err(validation_error(format!(
                "required embedding cache path is not a regular file: {}",
                file.local_path.display()
            )));
        }
        hasher.update(file.source_repo.as_bytes());
        hasher.update([0]);
        hasher.update(file.source_revision.as_bytes());
        hasher.update([0]);
        hasher.update(file.source_path.as_bytes());
        hasher.update([0]);

        let mut input = fs::File::open(&file.local_path)
            .map_err(io_error("open required embedding cache file"))?;
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            let bytes_read = input
                .read(&mut buffer)
                .map_err(io_error("read required embedding cache file"))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        hasher.update([0]);
    }
    Ok(format!(
        "{canonical_root};required_files_sha256={:x}",
        hasher.finalize()
    ))
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
    let manifest = validate_snapshot_manifest(root, expected)?;
    verify_checksums(root, trusted_artifact_digests)?;
    Ok(manifest)
}

fn validate_snapshot_manifest(
    root: &Path,
    expected: Option<&ValidationIdentity>,
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
    if manifest.artifact_modes != [EMBEDDED_MODE] {
        return Err(validation_error(
            "validation snapshot must contain exactly one embedded artifact mode",
        ));
    }
    let expected_required_files = [
        "validation-assertions.json",
        "artifact-validation.json",
        "timing.json",
        "progress.jsonl",
    ];
    if manifest
        .required_files
        .iter()
        .map(String::as_str)
        .ne(expected_required_files)
    {
        return Err(validation_error(
            "validation snapshot required-file inventory does not match policy",
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
    validate_embedded_artifact_inventory(root)?;
    Ok(manifest)
}

fn validate_embedded_artifact_inventory(root: &Path) -> Result<(), IngestError> {
    let artifacts_root = Path::new("artifacts");
    let embedded_root = artifacts_root.join(EMBEDDED_MODE);
    let embedded_artifact = root.join(&embedded_root).join("index.sqlite");
    if !embedded_artifact.is_file() {
        return Err(validation_error(format!(
            "validation snapshot is missing embedded artifact {}",
            embedded_artifact.display()
        )));
    }
    for relative in collect_files(root)? {
        if relative.starts_with(artifacts_root) && !relative.starts_with(&embedded_root) {
            return Err(validation_error(format!(
                "validation snapshot contains artifact outside {EMBEDDED_MODE}: {}",
                relative.display()
            )));
        }
    }
    Ok(())
}

fn report_from_snapshot(
    options: &ExhaustiveValidationOptions,
    manifest: &SnapshotManifest,
    elapsed_ms: u128,
) -> Result<ExhaustiveValidationReport, IngestError> {
    let assertions: Value = serde_json::from_slice(
        &fs::read(options.snapshot_root.join("validation-assertions.json"))
            .map_err(io_error("read validation assertions"))?,
    )
    .map_err(|error| validation_error(error.to_string()))?;
    if assertions.get("status").and_then(Value::as_str) != Some("pass") {
        return Err(validation_error(
            "reused snapshot validation assertions did not pass",
        ));
    }
    if assertions.get("artifact_modes") != Some(&json!([EMBEDDED_MODE])) {
        return Err(validation_error(
            "reused snapshot assertion inventory has an invalid artifact mode set",
        ));
    }
    let artifact_mode_reports: BTreeMap<String, Value> = serde_json::from_slice(
        &fs::read(options.snapshot_root.join("artifact-validation.json"))
            .map_err(io_error("read artifact validation evidence"))?,
    )
    .map_err(|error| validation_error(format!("invalid artifact validation evidence: {error}")))?;
    if artifact_mode_reports
        .keys()
        .map(String::as_str)
        .ne([EMBEDDED_MODE])
    {
        return Err(validation_error(
            "reused snapshot artifact evidence must contain exactly the embedded mode",
        ));
    }
    Ok(ExhaustiveValidationReport {
        status: "pass".to_string(),
        source_traversal_count: 1,
        artifact_modes: manifest.artifact_modes.clone(),
        assertion_inventory_complete: true,
        semantic_changes: Vec::new(),
        snapshot_root: options.snapshot_root.display().to_string(),
        snapshot_reused: true,
        source_signature: manifest.identity.source_signature.clone(),
        candidate_commit: manifest.identity.candidate_commit.clone(),
        timing: BTreeMap::from([("snapshot_validation_ms".to_string(), elapsed_ms)]),
        resources: json!({"reused": true}),
        assertion_inventory: assertion_inventory(),
        artifact_mode_reports,
    })
}

fn write_progress_entry(stage: &Path, entry: &Value) -> Result<(), IngestError> {
    let path = stage.join("progress.jsonl");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(io_error("open validation progress log"))?;
    writeln!(file, "{entry}").map_err(io_error("write validation progress log"))
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

fn write_json_atomic_replace(path: &Path, value: &impl Serialize) -> Result<(), IngestError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|error| validation_error(error.to_string()))?;
    write_bytes_atomic_replace(path, &bytes)
}

fn write_bytes_atomic_replace(path: &Path, bytes: &[u8]) -> Result<(), IngestError> {
    let temporary = atomic_file_staging_path(path);
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(io_error("create atomic validation replacement"))?;
        file.write_all(bytes)
            .map_err(io_error("write atomic validation replacement"))?;
        file.sync_all()
            .map_err(io_error("sync atomic validation replacement"))?;
        fs::rename(&temporary, path).map_err(io_error("publish atomic validation replacement"))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
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
        .ok_or_else(|| validation_error("validation evidence has no file name"))?
        .to_string_lossy();
    let contents = format!("{digest}  {file_name}\n");
    write_bytes_atomic_new(
        &checksum_sidecar_path(path),
        contents.as_bytes(),
        "validation evidence checksum",
    )
}

fn checksum_sidecar_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".sha256");
    PathBuf::from(name)
}

fn preserve_failed_snapshot(
    options: &ExhaustiveValidationOptions,
    stage: &Path,
    error: &IngestError,
    journal: &ValidationRunJournal,
    trusted_artifact_digests: &BTreeMap<PathBuf, String>,
) -> Result<PathBuf, IngestError> {
    let serialization_clock = OperationClock::start();
    let untrusted_artifacts = collect_files(stage)?
        .into_iter()
        .filter(|relative| {
            relative.starts_with("artifacts")
                && !trusted_artifact_digests.contains_key(relative)
                && relative
                    .extension()
                    .is_some_and(|extension| extension == "sqlite")
        })
        .collect::<Vec<_>>();
    let untrusted_artifact_bytes =
        untrusted_artifacts
            .iter()
            .try_fold(0_u64, |total, relative| {
                fs::metadata(stage.join(relative))
                    .map(|metadata| total.saturating_add(metadata.len()))
                    .map_err(io_error("read untrusted failure artifact size"))
            })?;
    let mut counter_state = journal.counter_state.clone();
    counter_state.insert(
        "failure_preservation_new_full_sha_pass_count".to_string(),
        json!(untrusted_artifacts.len()),
    );
    counter_state.insert(
        "failure_preservation_new_full_sha_bytes".to_string(),
        json!(untrusted_artifact_bytes),
    );
    let detail = journal
        .failure_detail
        .clone()
        .unwrap_or_else(|| ValidationFailureDetail {
            error_code: "validation_pipeline_error".to_string(),
            tuple_field: None,
            requested_selector: None,
            canonical_model_id: None,
            semantic_enum_identity: None,
            expected: None,
            actual: None,
        });
    let failure = json!({
        "format": "pf2e-atlas-validation-failure/v1",
        "status": "fail",
        "mode": journal.current_mode,
        "phase": journal.failure_point(),
        "failure_point": journal.failure_point(),
        "error": error.to_string(),
        "error_code": detail.error_code,
        "error_order": journal.failure_order(),
        "tuple_field": detail.tuple_field,
        "requested_selector": detail.requested_selector,
        "canonical_model_id": detail.canonical_model_id,
        "semantic_enum_identity": detail.semantic_enum_identity,
        "typed_expected": detail.expected,
        "typed_actual": detail.actual,
        "completed_operations": journal.completed,
        "in_progress_operations": journal.in_progress(),
        "artifact_identities": journal.artifact_identities,
        "counter_state": counter_state,
        "checksum_closure": {
            "trusted_artifact_digest_count": trusted_artifact_digests.len(),
        },
    });
    write_json_atomic_replace(&stage.join("failure.json"), &failure)?;
    let failure_bytes = fs::metadata(stage.join("failure.json"))
        .map_err(io_error("read structured failure size"))?
        .len();
    let failure_serialization =
        serialization_clock.finish(failure_bytes, 1, "atomic structured failure serialization");
    let timing = json!({
        "format": "pf2e-atlas-validation-partial-timing/v1",
        "status": "fail",
        "mode": journal.current_mode,
        "phase": journal.failure_point(),
        "total_wall_ms": journal.started.elapsed().as_millis(),
        "total_cpu_ms": journal.elapsed_cpu_ms(),
        "completed_operations": journal.completed,
        "in_progress_operations": journal.in_progress(),
        "failure_serialization": failure_serialization,
        "artifact_identities": journal.artifact_identities,
        "counter_state": counter_state,
    });
    write_json_atomic_replace(&stage.join("timing.json"), &timing)?;
    write_optional_file_sizes(stage);
    let checksum_digests = write_checksums(stage, trusted_artifact_digests)?;
    verify_checksums(stage, Some(&checksum_digests))?;
    let failed = failed_staging_path(&options.snapshot_root);
    fs::rename(stage, &failed).map_err(io_error("atomically preserve failed snapshot"))?;
    let evidence_root = options
        .report_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let failure_target = evidence_root.join("failure.json");
    let timing_target = evidence_root.join("timing.json");
    let failure_bytes = fs::read(failed.join("failure.json"))
        .map_err(io_error("read preserved structured failure"))?;
    let timing_bytes =
        fs::read(failed.join("timing.json")).map_err(io_error("read preserved partial timing"))?;
    write_bytes_atomic_new(&failure_target, &failure_bytes, "structured failure report")?;
    write_checksum_sidecar_atomic(&failure_target)?;
    write_bytes_atomic_new(&timing_target, &timing_bytes, "partial timing report")?;
    write_checksum_sidecar_atomic(&timing_target)?;
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
) -> Result<BTreeMap<PathBuf, String>, IngestError> {
    let mut entries = collect_files(root)?;
    entries.retain(|path| path != Path::new("checksums.sha256"));
    let mut output = String::new();
    let mut digests = BTreeMap::new();
    for relative in entries {
        let digest = match trusted_artifact_digests.get(&relative) {
            Some(digest) => digest.clone(),
            None => digest_file(&root.join(&relative))?,
        };
        output.push_str(&format!("{digest}  {}\n", relative.display()));
        digests.insert(relative, digest);
    }
    write_bytes_atomic_replace(root.join("checksums.sha256").as_path(), output.as_bytes())?;
    Ok(digests)
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
    write_json_atomic_replace(&root.join("file-sizes.json"), &sizes)
}

fn write_optional_file_sizes(root: &Path) {
    if let Err(error) = write_file_sizes(root) {
        eprintln!("production-validation diagnostic=file-sizes status=unavailable detail={error}");
    }
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
    let sizes_path = root.join("file-sizes.json");
    if sizes_path.is_file() {
        let sizes: BTreeMap<String, u64> = serde_json::from_slice(
            &fs::read(&sizes_path).map_err(io_error("read validation file sizes"))?,
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
    fn source_admission_refuses_skipped_records_and_warnings() {
        require_complete_source_admission(&[], &[]).expect("clean source admission");

        let skipped = [SkippedRecord {
            path: PathBuf::from("packs/bestiary/broken.json"),
            reason: "invalid record".to_string(),
        }];
        let skipped_error = require_complete_source_admission(&skipped, &[])
            .expect_err("skipped record must refuse production validation");
        assert!(skipped_error.to_string().contains("1 skipped record(s)"));
        assert!(skipped_error.to_string().contains("broken.json"));

        let warning_error = require_complete_source_admission(
            &[],
            &["could not read declared pack directory".to_string()],
        )
        .expect_err("loader warning must refuse production validation");
        assert!(warning_error.to_string().contains("1 warning(s)"));
        assert!(
            warning_error
                .to_string()
                .contains("declared pack directory")
        );
    }

    #[test]
    fn embedding_cache_identity_binds_required_file_bytes() {
        let root = temp_path("embedding-cache-identity");
        fs::create_dir_all(&root).expect("create cache root");
        let config = EmbeddingRuntimeConfig::new(EmbeddingModelId::BgeSmallEnV15, &root);
        let required_files = required_embedding_model_cache_files(&config);
        for (index, file) in required_files.iter().enumerate() {
            fs::create_dir_all(file.local_path.parent().expect("cache file parent"))
                .expect("create cache file parent");
            fs::write(&file.local_path, format!("fixture-{index}")).expect("write cache fixture");
        }

        let before = embedding_cache_identity(&root, EmbeddingModelId::BgeSmallEnV15)
            .expect("initial cache identity");
        fs::write(&required_files[0].local_path, b"changed-tokenizer")
            .expect("change tokenizer fixture");
        let after = embedding_cache_identity(&root, EmbeddingModelId::BgeSmallEnV15)
            .expect("changed cache identity");
        assert_ne!(before, after);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cheap_identity_match_does_not_read_embedding_files() {
        let cache_root = temp_path("cheap-identity-cache");
        fs::create_dir_all(&cache_root).expect("create empty cache root");
        let options = ExhaustiveValidationOptions {
            source_root: cache_root.clone(),
            candidate_head: "candidate".to_string(),
            snapshot_root: cache_root.join("snapshot"),
            report_path: cache_root.join("report.json"),
            embedding_cache_root: cache_root.clone(),
            force_reproduction: false,
        };
        let candidate = GitIdentity {
            commit: "candidate".to_string(),
            tree: "candidate-tree".to_string(),
            submodules: "candidate-submodules".to_string(),
        };
        let source = GitIdentity {
            commit: "source".to_string(),
            tree: "source-tree".to_string(),
            submodules: "source-submodules".to_string(),
        };
        let embedding = resolve_validation_embedding_identity("bge-small-en-v1.5")
            .expect("fixture embedding identity");
        let repository_root = current_repo_root().expect("repository root");
        let cache_identity = format!(
            "{};required_files_sha256={}",
            canonical_string(&cache_root).expect("canonical cache root"),
            "0".repeat(64)
        );
        let mut actual = static_identity_with_cache_identity(
            &candidate,
            &source,
            &repository_root,
            &embedding,
            cache_identity,
        )
        .expect("cheap fixture identity");

        assert!(
            identity_matches_before_embedding_hash(
                &actual,
                &options,
                &candidate,
                &source,
                &repository_root,
                &embedding,
            )
            .expect("cheap identity match"),
            "empty cache proves the cheap match does not read model files"
        );
        actual.candidate_tree = "different-tree".to_string();
        assert!(
            !identity_matches_before_embedding_hash(
                &actual,
                &options,
                &candidate,
                &source,
                &repository_root,
                &embedding,
            )
            .expect("cheap identity mismatch")
        );
        let _ = fs::remove_dir_all(cache_root);
    }

    #[test]
    fn embedded_artifact_inventory_rejects_other_mode_subtrees() {
        let root = temp_path("embedded-artifact-inventory");
        let embedded = root.join("artifacts/with_embeddings/index.sqlite");
        fs::create_dir_all(embedded.parent().expect("embedded artifact parent"))
            .expect("create embedded artifact parent");
        fs::write(&embedded, b"embedded fixture").expect("write embedded fixture");
        validate_embedded_artifact_inventory(&root).expect("single embedded mode");

        let no_embeddings = root.join("artifacts/no_embeddings/index.sqlite");
        fs::create_dir_all(no_embeddings.parent().expect("other artifact parent"))
            .expect("create other artifact parent");
        fs::write(&no_embeddings, b"unexpected fixture").expect("write other fixture");
        let error = validate_embedded_artifact_inventory(&root)
            .expect_err("other artifact mode must fail closed");
        assert!(error.to_string().contains("outside with_embeddings"));
        let _ = fs::remove_dir_all(root);
    }

    fn write_artifact_pair_fixture(
        stage: &Path,
        artifact: &Path,
        bytes: &[u8],
        document_embedding_count: usize,
    ) -> PersistedArtifactPairEvidence {
        fs::create_dir_all(artifact.parent().expect("artifact parent"))
            .expect("create artifact parent");
        fs::write(artifact, bytes).expect("write artifact fixture");
        let artifact_sha256 = digest_file(artifact).expect("digest artifact fixture");
        write_json(
            adjacent_artifact_manifest_path(artifact),
            &json!({
                "manifest_version": ARTIFACT_MANIFEST_VERSION,
                "artifact_contract_version": ARTIFACT_CONTRACT_VERSION,
                "schema_version": ARTIFACT_SCHEMA_VERSION,
                "source": {
                    "kind": "foundry-pf2e",
                    "root": "/fixture/source",
                    "signature": "fixture-source-signature",
                    "record_count": 1
                },
                "build": {
                    "artifact_sha256": artifact_sha256,
                    "artifact_record_count": 1,
                    "generated_record_count": 0,
                    "document_embedding_count": document_embedding_count,
                    "embedding_model": "BAAI/bge-small-en-v1.5"
                }
            }),
        )
        .expect("write artifact manifest fixture");
        fs::write(adjacent_artifact_lock_path(artifact), b"").expect("write pair lock fixture");
        persisted_artifact_pair_evidence(
            stage,
            artifact,
            &artifact_sha256,
            bytes.len() as u64,
            document_embedding_count,
        )
        .expect("inspect matching artifact pair fixture")
    }

    #[test]
    fn embedded_artifact_retains_matching_pair() {
        let stage = temp_path("artifact-pair-layout");
        let outputs = validation_artifact_outputs(&stage.join("artifacts"));
        let with_embeddings = write_artifact_pair_fixture(
            &stage,
            &outputs[EMBEDDED_MODE],
            b"with-embedding artifact",
            7,
        );

        assert_eq!(
            with_embeddings.artifact_path,
            PathBuf::from("artifacts/with_embeddings/index.sqlite")
        );
        assert_eq!(
            with_embeddings.manifest_path,
            PathBuf::from("artifacts/with_embeddings/manifest.json")
        );
        assert_eq!(
            with_embeddings.lock_path,
            PathBuf::from("artifacts/with_embeddings/manifest.json.pair.lock")
        );
        assert_eq!(with_embeddings.artifact_bytes, 23);
        assert_eq!(with_embeddings.document_embedding_count, 7);
        let _ = fs::remove_dir_all(stage);
    }

    #[test]
    fn embedded_artifact_rejects_cross_bound_manifest() {
        let stage = temp_path("artifact-pair-cross-binding");
        let outputs = validation_artifact_outputs(&stage.join("artifacts"));
        let other_path = stage.join("artifacts/other/index.sqlite");
        let other = write_artifact_pair_fixture(&stage, &other_path, b"other artifact", 3);
        let with_embeddings = write_artifact_pair_fixture(
            &stage,
            &outputs[EMBEDDED_MODE],
            b"with-embedding artifact",
            7,
        );
        fs::copy(
            stage.join(&other.manifest_path),
            stage.join(&with_embeddings.manifest_path),
        )
        .expect("overwrite with the other variant manifest");

        let error = persisted_artifact_pair_evidence(
            &stage,
            &outputs[EMBEDDED_MODE],
            &with_embeddings.artifact_sha256,
            with_embeddings.artifact_bytes,
            with_embeddings.document_embedding_count,
        )
        .expect_err("a manifest from the other variant must fail closed");
        assert!(error.to_string().contains("manifest cross-binding"));
        let _ = fs::remove_dir_all(stage);
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
    fn checksum_validation_accepts_absent_optional_sizes_and_rejects_tamper() {
        let root = temp_path("tamper");
        fs::create_dir(&root).expect("fixture root");
        fs::write(root.join("payload"), b"complete").expect("fixture payload");
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
            mode: EMBEDDED_MODE.into(),
            source_signature: "source".into(),
            artifact_contract_version: "artifact".into(),
            artifact_schema_version: "schema".into(),
            embedding: resolve_validation_embedding_identity("bge-small-en-v1.5")
                .expect("fixture embedding identity"),
        };
        let generation = json!({
            "canonical_artifact_path": "/snapshot/artifacts/with_embeddings/index.sqlite",
            "generation_path": "/snapshot/artifacts/with_embeddings/index.sqlite.atlas-generations/sha.sqlite",
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
        rejects_change!(mode, "other_mode");
        rejects_change!(source_signature, "other-source");
        rejects_change!(artifact_contract_version, "other-artifact");
        rejects_change!(artifact_schema_version, "other-schema");
        let mut changed_embedding = base.clone();
        changed_embedding.embedding = resolve_validation_embedding_identity("bge-base-en-v1.5")
            .expect("different fixture embedding identity");
        assert_ne!(
            expected,
            validation_binding_digest(&changed_embedding, &generation)
        );

        let mut changed_generation = generation.clone();
        changed_generation["bytes"] = json!(11);
        assert_ne!(
            expected,
            validation_binding_digest(&base, &changed_generation)
        );
    }
}

#[cfg(test)]
mod c2pr_tests;

#[cfg(all(test, feature = "record-round-trip-diagnostic"))]
mod record_round_trip_diagnostic;
