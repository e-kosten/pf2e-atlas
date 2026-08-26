//! Private validation-pipeline state. Nothing in this module is a product
//! artifact, runtime input, fallback, or public serialization contract.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use atlas_index::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, SqliteIndexReader, ValidationStatus,
    ValidationTarget,
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
        let manifest = validate_snapshot(&options.snapshot_root, None)?;
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
    fs::rename(&stage, &options.snapshot_root)
        .map_err(io_error("atomically publish validation snapshot"))?;
    validate_snapshot(
        &options.snapshot_root,
        Some(&report_identity(&options.snapshot_root)?),
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
    for (mode, cache) in [
        ("no_embeddings", None),
        (
            "with_embeddings",
            Some(options.embedding_cache_root.clone()),
        ),
    ] {
        write_progress(stage, mode, "started")?;
        let phase = Instant::now();
        let output = artifacts.join(format!("{mode}.sqlite"));
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
        let reader = if mode == "with_embeddings" {
            SqliteIndexReader::open_read_only_with_vectors(&output)
        } else {
            SqliteIndexReader::open_read_only(&output)
        }
        .map_err(|error| validation_error(error.to_string()))?;
        let check = reader
            .check()
            .map_err(|error| validation_error(error.to_string()))?;
        let inspect = reader
            .inspect()
            .map_err(|error| validation_error(error.to_string()))?;
        let target = if mode == "with_embeddings" {
            ValidationTarget::Full
        } else {
            ValidationTarget::BaseOnly
        };
        let deep = reader
            .validate_target(target)
            .map_err(|error| validation_error(error.to_string()))?;
        if check.status != ValidationStatus::Ok || deep.status != ValidationStatus::Ok {
            return Err(validation_error(format!(
                "{mode} artifact validation failed"
            )));
        }
        artifact_reports.insert(
            mode,
            json!({
                "build": build_report_json(&build),
                "check": check,
                "inspect": inspect,
                "deep_validation": deep,
            }),
        );
        timing.insert(format!("artifact_{mode}_ms"), phase.elapsed().as_millis());
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
    write_checksums(stage)?;

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
    verify_checksums(root)?;
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
    })
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
    write_checksums(stage)?;
    verify_checksums(stage)?;
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

fn write_checksums(root: &Path) -> Result<(), IngestError> {
    let mut entries = collect_files(root)?;
    entries.retain(|path| path != Path::new("checksums.sha256"));
    let mut output = String::new();
    for relative in entries {
        let digest = digest_file(&root.join(&relative))?;
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

fn verify_checksums(root: &Path) -> Result<(), IngestError> {
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
        if !path.is_file() || digest_file(&path)? != expected {
            return Err(validation_error(format!(
                "validation snapshot checksum mismatch: {relative}"
            )));
        }
        listed.push(PathBuf::from(relative));
    }
    listed.sort();
    let mut actual = collect_files(root)?;
    actual.retain(|path| path != Path::new("checksums.sha256"));
    if listed != actual {
        return Err(validation_error(
            "validation snapshot checksum inventory has missing, duplicate, or extra entries",
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
        write_checksums(&root).expect("fixture checksums");
        verify_checksums(&root).expect("valid checksums");
        fs::write(root.join("payload"), b"tampered").expect("tamper payload");
        assert!(verify_checksums(&root).is_err());
        fs::remove_file(root.join("payload")).expect("remove payload");
        assert!(verify_checksums(&root).is_err());
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
}
