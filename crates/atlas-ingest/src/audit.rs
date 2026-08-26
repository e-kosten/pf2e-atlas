use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::PackName;
use atlas_record::{
    CreatureCapability, FactValue, RecordBody, UnsupportedSourceShape, UnsupportedSourceValue,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::IngestError;
use crate::records::LoadedSourceRecord;
use crate::source::SourceLoad;
use crate::source::dto::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, SourceDiagnostic,
    SourceDiagnosticKind, SourceIdentity, SourcePresence, parse_item_source, parse_npc_source,
    pinned_source_version_metadata,
};
use crate::source::loader::{
    default_manifest_path, json_files, parse_manifest, relative_source_path, resolve_pack_path,
};
use crate::source::normalize::normalize_record;
#[cfg(test)]
use crate::source::npc_entities::RETAINED_CAPABILITY_PATHS;
use crate::source::npc_entities::{collect_npc_embedded_candidates, convert_npc_embedded_entities};

mod coverage;
mod predicate_inventory;

pub use predicate_inventory::RetrievalPredicateInventoryEntry;

use coverage::{
    COVERAGE_POLICY_VERSION, SOURCE_COVERAGE_REGISTRY_ASSIGNMENTS, coverage_declarations,
    declaration_for,
};
use predicate_inventory::retrieval_predicate_inventory;

const SAMPLE_LIMIT: usize = 3;
const DEFAULT_PATH_LIMIT: usize = 200;

#[derive(Debug, Clone, Default)]
pub struct SourcePathAuditOptions {
    pub source_root: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub pack_name: Option<String>,
    pub document_type: Option<String>,
    pub record_type: Option<String>,
    pub min_records: usize,
    pub limit: Option<usize>,
    pub strict: bool,
    pub baseline_report: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditReport {
    pub coverage_policy_version: &'static str,
    pub coverage_policy_digest: String,
    pub source_contract_version: &'static str,
    pub source_upstream_commit: &'static str,
    pub registry_assignment_count: usize,
    pub source_root: String,
    pub manifest_path: String,
    pub pack_count: usize,
    pub record_count: usize,
    pub path_count: usize,
    pub filters: SourcePathAuditFilters,
    pub summary: SourcePathAuditSummary,
    pub enforcement: SourcePathAuditEnforcement,
    pub closure_totals: SourcePathAuditClosureTotals,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_diff: Option<SourcePathAuditDiff>,
    pub closure_failures: Vec<SourcePathAuditClosureFailure>,
    pub diagnostics: Vec<SourceCoverageDiagnostic>,
    pub retrieval_predicate_inventory: Vec<RetrievalPredicateInventoryEntry>,
    pub paths: Vec<SourcePathAuditPathReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_type: Option<String>,
    pub min_records: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    pub strict: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_report: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditSummary {
    pub consumed_paths: usize,
    pub ignored_with_rationale_paths: usize,
    pub provenance_only_paths: usize,
    pub deferred_paths: usize,
    pub unknown_paths: usize,
    pub generic_deferred_paths: usize,
    pub unowned_recursive_matches: usize,
    pub consumed_regressions: usize,
    pub creature_paths: usize,
    pub creature_consumed_paths: usize,
    pub creature_provenance_only_paths: usize,
    pub creature_deferred_paths: usize,
    pub creature_unknown_paths: usize,
    pub creature_catch_all_paths: usize,
    pub creature_unowned_paths: usize,
    pub creature_consumed_regressions: usize,
    pub type_drift_diagnostics: usize,
    pub source_diff_changes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditEnforcement {
    pub mode: SourcePathAuditMode,
    pub passed: bool,
    pub violation_count: usize,
    pub aggregate_warning_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditClosureTotals {
    pub expected_observation_count: usize,
    pub observed_observation_count: usize,
    pub failure_count: usize,
    pub mismatch_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePathAuditMode {
    Relaxed,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditPathReport {
    pub document_type: String,
    pub record_type: String,
    pub path: String,
    pub path_family: String,
    pub matched_rule_id: String,
    pub owner_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extractor_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserved_occurrence_count: Option<usize>,
    pub fixture_key: String,
    pub validation: String,
    pub checkpoint: String,
    pub recursive_match: bool,
    pub complete_family_assignment: bool,
    pub record_count: usize,
    pub occurrence_count: usize,
    pub value_types: Vec<SourcePathAuditValueType>,
    pub disposition: SourcePathCoverageDisposition,
    pub owner: String,
    pub product_rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub future_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub future_plan: Option<String>,
    pub examples: Vec<SourcePathAuditSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditValueType {
    pub kind: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SourcePathAuditSample {
    pub record_key: String,
    pub source_path: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePathCoverageDisposition {
    Consumed,
    IgnoredWithRationale,
    ProvenanceOnly,
    Deferred,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceCoverageDiagnosticKind {
    UnknownPath,
    MalformedShape,
    UnknownDiscriminator,
    InvalidParentContext,
    UnsupportedSourceVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCoverageDiagnostic {
    pub kind: SourceCoverageDiagnosticKind,
    pub document_type: String,
    pub record_type: String,
    pub json_path: String,
    pub expected_shape: String,
    pub actual_shape: String,
    pub occurrence_count: usize,
    pub examples: Vec<SourcePathAuditSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditDiff {
    pub baseline_policy_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_source_upstream_commit: Option<String>,
    pub added_paths: Vec<SourcePathAuditDiffEntry>,
    pub removed_paths: Vec<SourcePathAuditDiffEntry>,
    pub changed_dispositions: Vec<SourcePathAuditDispositionChange>,
    pub consumed_regressions: Vec<SourcePathAuditDispositionChange>,
}

impl SourcePathAuditDiff {
    pub fn change_count(&self) -> usize {
        self.added_paths.len() + self.removed_paths.len() + self.changed_dispositions.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SourcePathAuditDiffEntry {
    pub document_type: String,
    pub record_type: String,
    pub path: String,
    pub disposition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditDispositionChange {
    pub document_type: String,
    pub record_type: String,
    pub path: String,
    pub baseline_disposition: String,
    pub current_disposition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SourcePathAuditClosureFailure {
    pub document_type: String,
    pub record_type: String,
    pub path: String,
    pub source_occurrence_count: usize,
    pub preserved_occurrence_count: usize,
    pub mismatches: Vec<SourcePathAuditObservationMismatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SourcePathAuditObservationMismatch {
    pub normalized_path: String,
    pub record_key: String,
    pub member_identity: String,
    pub contextual_source_path: String,
    pub destination: String,
    pub expected_state: String,
    pub expected_type: String,
    pub expected_value: String,
    pub observed_state: String,
    pub observed_type: String,
    pub observed_value: String,
    pub expected_multiplicity: usize,
    pub observed_multiplicity: usize,
    pub expected_order: Option<usize>,
    pub observed_order: Option<usize>,
}

#[derive(Debug, Default)]
struct MutablePathStats {
    occurrence_count: usize,
    value_types: BTreeMap<String, usize>,
    record_keys: BTreeSet<String>,
    examples: Vec<SourcePathAuditSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PathKey {
    document_type: String,
    record_type: String,
    path: String,
}

#[derive(Debug)]
struct RecordContext {
    document_type: String,
    record_type: String,
    record_key: String,
    source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DiagnosticKey {
    kind: SourceCoverageDiagnosticKind,
    document_type: String,
    record_type: String,
    json_path: String,
    expected_shape: String,
    actual_shape: String,
}

#[derive(Debug, Default)]
struct MutableDiagnostic {
    occurrence_count: usize,
    examples: BTreeSet<SourcePathAuditSample>,
}

#[derive(Debug, Default)]
struct CreatureSurvivalInventory {
    canonical: BTreeMap<String, CanonicalPathStats>,
    declarations: BTreeMap<String, Option<&'static str>>,
    mismatches: BTreeMap<String, Vec<SourcePathAuditObservationMismatch>>,
    observation_ordinals: BTreeMap<(String, String, String), usize>,
    reverse_expected: BTreeMap<ReverseObservation, usize>,
    reverse_observed: BTreeMap<ReverseObservation, usize>,
    reverse_observed_owners: BTreeSet<(String, String, String)>,
    output_only_counts: BTreeMap<String, usize>,
    actual_observation_ordinals: BTreeMap<(String, String, String), usize>,
}

#[derive(Debug, Clone, Default)]
struct CanonicalPathStats {
    occurrence_count: usize,
    destinations: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ReverseObservation {
    record_key: String,
    member_identity: String,
    contextual_source_path: String,
    normalized_path: String,
    destination: String,
    state: String,
    value_type: String,
    normalized_value: String,
    order: usize,
}

pub fn audit_source_paths(
    options: SourcePathAuditOptions,
) -> Result<SourcePathAuditReport, IngestError> {
    let source_root = options.source_root.clone();
    if !source_root.is_dir() {
        return Err(IngestError::SourceUnavailable(format!(
            "{} is not a readable directory",
            source_root.display()
        )));
    }
    let manifest_path = options
        .manifest_path
        .clone()
        .unwrap_or_else(|| default_manifest_path(&source_root));
    let parsed_manifest = parse_manifest(&manifest_path)?;
    let mut stats = BTreeMap::<PathKey, MutablePathStats>::new();
    let mut typed_diagnostics = BTreeMap::<DiagnosticKey, MutableDiagnostic>::new();
    let mut creature_survival = CreatureSurvivalInventory::default();
    let mut pack_count = 0;
    let mut record_count = 0;

    for manifest_pack in parsed_manifest.manifest.packs {
        if options
            .pack_name
            .as_ref()
            .is_some_and(|pack_name| pack_name != &manifest_pack.name)
        {
            continue;
        }
        if options
            .document_type
            .as_ref()
            .is_some_and(|document_type| document_type != &manifest_pack.document_type)
        {
            continue;
        }
        let pack_name = PackName::new(manifest_pack.name.clone()).map_err(|error| {
            IngestError::ManifestParseFailed(format!("invalid pack name: {error}"))
        })?;
        let resolved_path = resolve_pack_path(&source_root, &manifest_pack);
        if !resolved_path.is_dir() {
            continue;
        }
        pack_count += 1;
        for source_file in json_files(&resolved_path)? {
            let value = read_json_value(&source_file)?;
            let record_type = value
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if options
                .record_type
                .as_ref()
                .is_some_and(|expected| expected != &record_type)
            {
                continue;
            }
            let record_key = value
                .get("_id")
                .and_then(Value::as_str)
                .map(|id| format!("{}:{id}", manifest_pack.name))
                .unwrap_or_else(|| relative_source_path(&source_root, &source_file));
            let context = RecordContext {
                document_type: manifest_pack.document_type.clone(),
                record_type,
                record_key,
                source_path: relative_source_path(&source_root, &source_file),
            };
            record_count += 1;
            collect_value_paths("$", &value, &context, &mut stats);
            let loaded = (context.document_type == "Actor" && context.record_type == "npc")
                .then(|| {
                    normalize_record(
                        &manifest_pack,
                        &pack_name,
                        &source_file,
                        &source_root,
                        value.clone(),
                        None,
                    )
                    .ok()
                })
                .flatten();
            collect_creature_survival(&value, &context, loaded.as_ref(), &mut creature_survival);
            validate_typed_source(&value, &context, &mut typed_diagnostics);
        }
    }

    finish_source_path_audit(
        options,
        source_root,
        manifest_path,
        pack_count,
        record_count,
        stats,
        typed_diagnostics,
        creature_survival,
    )
}

pub(crate) fn audit_loaded_source(
    options: SourcePathAuditOptions,
    source: &SourceLoad,
) -> Result<SourcePathAuditReport, IngestError> {
    let source_root = options.source_root.clone();
    let manifest_path = source.manifest_path.clone();
    let mut stats = BTreeMap::<PathKey, MutablePathStats>::new();
    let mut typed_diagnostics = BTreeMap::<DiagnosticKey, MutableDiagnostic>::new();
    let mut creature_survival = CreatureSurvivalInventory::default();
    let pack_count = source
        .packs
        .iter()
        .filter(|pack| !pack.declared_path.starts_with("derived://"))
        .filter(|pack| {
            options
                .pack_name
                .as_ref()
                .is_none_or(|expected| expected == pack.name.as_str())
        })
        .filter(|pack| {
            options
                .document_type
                .as_ref()
                .is_none_or(|expected| expected == &pack.document_type)
        })
        .count();
    let mut record_count = 0;

    for loaded in source.records.iter().take(source.source_record_count) {
        let document_type = loaded.record.foundry.document_type.as_str().to_string();
        let record_type = loaded.record.foundry.record_type.as_str().to_string();
        let pack_name = loaded.record.identity.key.pack().as_str().to_string();
        if options
            .pack_name
            .as_ref()
            .is_some_and(|expected| expected != &pack_name)
            || options
                .document_type
                .as_ref()
                .is_some_and(|expected| expected != &document_type)
            || options
                .record_type
                .as_ref()
                .is_some_and(|expected| expected != &record_type)
        {
            continue;
        }
        let serialized = loaded
            .record
            .provenance
            .raw_json
            .as_deref()
            .ok_or_else(|| {
                IngestError::RecordParseFailed(format!(
                    "captured source record {} has no raw JSON for strict audit",
                    loaded.record.identity.key
                ))
            })?;
        let value: Value = serde_json::from_str(serialized).map_err(|error| {
            IngestError::RecordParseFailed(format!(
                "captured source record {} has invalid raw JSON: {error}",
                loaded.record.identity.key
            ))
        })?;
        let context = RecordContext {
            document_type: document_type.clone(),
            record_type: record_type.clone(),
            record_key: loaded.record.identity.key.to_string(),
            source_path: loaded.record.provenance.source_path.clone(),
        };
        record_count += 1;
        collect_value_paths("$", &value, &context, &mut stats);
        let audit_record = legacy_audit_record_from_capture(
            &source_root,
            source,
            loaded,
            &value,
            &document_type,
            &record_type,
        )?;
        collect_creature_survival(
            &value,
            &context,
            audit_record.as_ref(),
            &mut creature_survival,
        );
        validate_typed_source(&value, &context, &mut typed_diagnostics);
    }

    finish_source_path_audit(
        options,
        source_root,
        manifest_path,
        pack_count,
        record_count,
        stats,
        typed_diagnostics,
        creature_survival,
    )
}

fn legacy_audit_record_from_capture(
    source_root: &Path,
    source: &SourceLoad,
    loaded: &LoadedSourceRecord,
    value: &Value,
    document_type: &str,
    record_type: &str,
) -> Result<Option<LoadedSourceRecord>, IngestError> {
    if document_type != "Actor" || record_type != "npc" {
        return Ok(None);
    }
    let pack_name = loaded.record.identity.key.pack();
    let pack = source
        .packs
        .iter()
        .find(|pack| {
            !pack.declared_path.starts_with("derived://")
                && &pack.name == pack_name
                && pack.document_type == document_type
        })
        .ok_or_else(|| {
            IngestError::RecordParseFailed(format!(
                "captured source record {} has no matching source pack for strict audit",
                loaded.record.identity.key
            ))
        })?;
    let manifest_pack = crate::source::ManifestPack {
        name: pack.name.as_str().to_string(),
        label: pack.label.clone(),
        document_type: pack.document_type.clone(),
        path: pack.declared_path.clone(),
    };
    let source_file = source_root.join(&loaded.record.provenance.source_path);
    normalize_record(
        &manifest_pack,
        &pack.name,
        &source_file,
        source_root,
        value.clone(),
        None,
    )
    .map(Some)
}

#[allow(clippy::too_many_arguments)]
fn finish_source_path_audit(
    options: SourcePathAuditOptions,
    source_root: PathBuf,
    manifest_path: PathBuf,
    pack_count: usize,
    record_count: usize,
    stats: BTreeMap<PathKey, MutablePathStats>,
    typed_diagnostics: BTreeMap<DiagnosticKey, MutableDiagnostic>,
    creature_survival: CreatureSurvivalInventory,
) -> Result<SourcePathAuditReport, IngestError> {
    let mut paths = stats
        .into_iter()
        .filter_map(|(key, stats)| {
            let record_count = stats.record_keys.len();
            (record_count >= options.min_records).then(|| path_report(key, stats, record_count))
        })
        .collect::<Vec<_>>();
    paths.sort_by(path_report_order);
    let path_count = paths.len();
    let closure_failures = reconcile_creature_survival(&mut paths, &creature_survival);
    let closure_totals = SourcePathAuditClosureTotals {
        expected_observation_count: paths
            .iter()
            .filter(|path| {
                is_creature_path(&path.document_type, &path.record_type)
                    && path.disposition == SourcePathCoverageDisposition::Consumed
            })
            .map(|path| path.occurrence_count)
            .sum(),
        observed_observation_count: paths
            .iter()
            .filter(|path| {
                is_creature_path(&path.document_type, &path.record_type)
                    && path.disposition == SourcePathCoverageDisposition::Consumed
            })
            .map(|path| path.preserved_occurrence_count.unwrap_or_default())
            .sum(),
        failure_count: closure_failures.len(),
        mismatch_count: closure_failures
            .iter()
            .map(|failure| failure.mismatches.len())
            .sum(),
    };
    let mut diagnostics = typed_diagnostics
        .into_iter()
        .map(|(key, diagnostic)| SourceCoverageDiagnostic {
            kind: key.kind,
            document_type: key.document_type,
            record_type: key.record_type,
            json_path: key.json_path,
            expected_shape: key.expected_shape,
            actual_shape: key.actual_shape,
            occurrence_count: diagnostic.occurrence_count,
            examples: diagnostic.examples.into_iter().take(SAMPLE_LIMIT).collect(),
        })
        .collect::<Vec<_>>();
    diagnostics.extend(unknown_path_diagnostics(&paths));
    diagnostics.sort_by(diagnostic_order);

    let mut summary = summarize_paths(&paths, &diagnostics);
    let source_diff = options
        .baseline_report
        .as_deref()
        .map(|baseline| compare_baseline(baseline, &paths))
        .transpose()?;
    summary.source_diff_changes = source_diff
        .as_ref()
        .map_or(0, SourcePathAuditDiff::change_count);
    summary.consumed_regressions = closure_failures.len()
        + source_diff
            .as_ref()
            .map_or(0, |diff| diff.consumed_regressions.len());
    summary.creature_consumed_regressions = closure_failures.len()
        + source_diff.as_ref().map_or(0, |diff| {
            diff.consumed_regressions
                .iter()
                .filter(|change| is_creature_path(&change.document_type, &change.record_type))
                .count()
        });
    let violation_count = summary.unknown_paths
        + summary.type_drift_diagnostics
        + summary.source_diff_changes
        + summary.generic_deferred_paths
        + summary.unowned_recursive_matches
        + summary.creature_deferred_paths
        + summary.creature_catch_all_paths
        + summary.creature_unowned_paths
        + summary.consumed_regressions;
    let enforcement = source_path_enforcement(options.strict, violation_count);
    let policy_digest = coverage_policy_digest()?;
    let predicate_inventory = retrieval_predicate_inventory();
    let limit = options.limit.unwrap_or(DEFAULT_PATH_LIMIT);
    paths.truncate(limit);

    Ok(SourcePathAuditReport {
        coverage_policy_version: COVERAGE_POLICY_VERSION,
        coverage_policy_digest: policy_digest,
        source_contract_version: PF2E_SOURCE_CONTRACT_VERSION,
        source_upstream_commit: PF2E_SOURCE_PINNED_COMMIT,
        registry_assignment_count: SOURCE_COVERAGE_REGISTRY_ASSIGNMENTS,
        source_root: source_root.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        pack_count,
        record_count,
        path_count,
        filters: SourcePathAuditFilters {
            pack_name: options.pack_name,
            document_type: options.document_type,
            record_type: options.record_type,
            min_records: options.min_records,
            limit: options.limit,
            strict: options.strict,
            baseline_report: options
                .baseline_report
                .map(|path| path.display().to_string()),
        },
        summary,
        enforcement,
        closure_totals,
        source_diff,
        closure_failures,
        diagnostics,
        retrieval_predicate_inventory: predicate_inventory,
        paths,
    })
}

fn source_path_enforcement(strict: bool, violation_count: usize) -> SourcePathAuditEnforcement {
    SourcePathAuditEnforcement {
        mode: if strict {
            SourcePathAuditMode::Strict
        } else {
            SourcePathAuditMode::Relaxed
        },
        passed: !strict || violation_count == 0,
        violation_count: if strict { violation_count } else { 0 },
        aggregate_warning_count: violation_count,
    }
}

fn read_json_value(path: &Path) -> Result<Value, IngestError> {
    let serialized = fs::read_to_string(path)
        .map_err(|error| IngestError::RecordParseFailed(error.to_string()))?;
    serde_json::from_str(&serialized)
        .map_err(|error| IngestError::RecordParseFailed(format!("{}: {error}", path.display())))
}

fn collect_value_paths(
    path: &str,
    value: &Value,
    context: &RecordContext,
    stats: &mut BTreeMap<PathKey, MutablePathStats>,
) {
    record_path(path, value, context, stats);
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let segment = object_segment(path, key, map.len());
                let child_path = format!("{path}.{segment}");
                collect_value_paths(&child_path, child, context, stats);
            }
        }
        Value::Array(values) => {
            let child_path = format!("{path}[]");
            for child in values {
                collect_value_paths(&child_path, child, context, stats);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn record_path(
    path: &str,
    value: &Value,
    context: &RecordContext,
    stats: &mut BTreeMap<PathKey, MutablePathStats>,
) {
    if !is_meaningful_value(value) {
        return;
    }
    let path_key = PathKey {
        document_type: context.document_type.clone(),
        record_type: context.record_type.clone(),
        path: path.to_string(),
    };
    let path_stats = stats.entry(path_key).or_default();
    path_stats.occurrence_count += 1;
    *path_stats
        .value_types
        .entry(value_type(value).to_string())
        .or_insert(0) += 1;
    path_stats.record_keys.insert(context.record_key.clone());
    if path_stats.examples.len() < SAMPLE_LIMIT
        && let Some(sample) = scalar_sample(value)
    {
        path_stats.examples.push(SourcePathAuditSample {
            record_key: context.record_key.clone(),
            source_path: context.source_path.clone(),
            value: sample,
        });
    }
}

fn is_meaningful_value(value: &Value) -> bool {
    match value {
        Value::Null | Value::Array(_) | Value::Object(_) => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
    }
}

fn value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn scalar_sample(value: &Value) -> Option<String> {
    let text = match value {
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Null | Value::Array(_) | Value::Object(_) => return None,
    };
    Some(truncate_sample(&text))
}

fn truncate_sample(value: &str) -> String {
    const MAX_CHARS: usize = 160;
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(MAX_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn object_segment(parent_path: &str, key: &str, sibling_count: usize) -> String {
    if is_dynamic_key_parent(parent_path)
        || (sibling_count > 4 && looks_like_source_id_or_hash(key))
    {
        "*".to_string()
    } else if is_simple_path_key(key) {
        key.to_string()
    } else {
        format!("[\"{}\"]", key.escape_default())
    }
}

fn is_dynamic_key_parent(path: &str) -> bool {
    [
        ".damageRolls",
        ".damage",
        ".itemGrants",
        ".overlays",
        ".resources",
        ".skills",
        ".saves",
        ".slots",
    ]
    .iter()
    .any(|suffix| path.ends_with(suffix))
}

fn looks_like_source_id_or_hash(key: &str) -> bool {
    key.len() >= 10
        && key.chars().any(|character| character.is_ascii_digit())
        && key.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn is_simple_path_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn path_report(
    key: PathKey,
    stats: MutablePathStats,
    record_count: usize,
) -> SourcePathAuditPathReport {
    if let Some(declaration) = declaration_for(&key.document_type, &key.record_type, &key.path) {
        let recursive_match = declaration.is_recursive();
        let complete_family_assignment = declaration.is_complete_family_assignment();
        let owner_family = owner_family(declaration.disposition).to_string();
        let fixture_key = format!(
            "pinned-full-corpus/{}/{}",
            key.document_type,
            if key.record_type.is_empty() {
                "no-discriminator"
            } else {
                key.record_type.as_str()
            }
        );
        let validation = validation_contract(declaration.disposition).to_string();
        let checkpoint = declaration.future_owner.unwrap_or("B2").to_string();
        return SourcePathAuditPathReport {
            document_type: key.document_type,
            record_type: key.record_type,
            path: key.path,
            path_family: declaration.path_family.to_string(),
            matched_rule_id: format!("{}@{}", declaration.id, declaration.path_family),
            owner_family,
            extractor_identity: None,
            preserved_occurrence_count: None,
            fixture_key,
            validation,
            checkpoint,
            recursive_match,
            complete_family_assignment,
            record_count,
            occurrence_count: stats.occurrence_count,
            value_types: stats
                .value_types
                .into_iter()
                .map(|(kind, count)| SourcePathAuditValueType { kind, count })
                .collect(),
            disposition: declaration.disposition,
            owner: declaration.owner.to_string(),
            product_rationale: declaration.product_rationale.to_string(),
            future_owner: declaration.future_owner.map(str::to_string),
            future_plan: declaration.future_plan.map(str::to_string),
            examples: stats.examples,
        };
    }

    let fixture_key = format!(
        "pinned-full-corpus/{}/{}",
        key.document_type,
        if key.record_type.is_empty() {
            "no-discriminator"
        } else {
            key.record_type.as_str()
        }
    );
    SourcePathAuditPathReport {
        document_type: key.document_type,
        record_type: key.record_type,
        path: key.path.clone(),
        path_family: key.path,
        matched_rule_id: "unassigned".to_string(),
        owner_family: "unassigned".to_string(),
        extractor_identity: None,
        preserved_occurrence_count: None,
        fixture_key,
        validation: "strict coverage failure pending exact classification".to_string(),
        checkpoint: "B2".to_string(),
        recursive_match: false,
        complete_family_assignment: false,
        record_count,
        occurrence_count: stats.occurrence_count,
        value_types: stats
            .value_types
            .into_iter()
            .map(|(kind, count)| SourcePathAuditValueType { kind, count })
            .collect(),
        disposition: SourcePathCoverageDisposition::Unknown,
        owner: "unassigned".to_string(),
        product_rationale: "No real extractor, reviewed exclusion, provenance owner, or exact future owner declares this meaningful path.".to_string(),
        future_owner: None,
        future_plan: None,
        examples: stats.examples,
    }
}

fn owner_family(disposition: SourcePathCoverageDisposition) -> &'static str {
    match disposition {
        SourcePathCoverageDisposition::Consumed => "real_extractor",
        SourcePathCoverageDisposition::IgnoredWithRationale => "reviewed_exclusion",
        SourcePathCoverageDisposition::ProvenanceOnly => "typed_provenance_owner",
        SourcePathCoverageDisposition::Deferred => "exact_future_owner",
        SourcePathCoverageDisposition::Unknown => "unassigned",
    }
}

fn validation_contract(disposition: SourcePathCoverageDisposition) -> &'static str {
    match disposition {
        SourcePathCoverageDisposition::Consumed => {
            "strict full-corpus audit plus consumed-regression comparison"
        }
        SourcePathCoverageDisposition::IgnoredWithRationale => {
            "strict full-corpus audit plus reviewed non-auth rationale"
        }
        SourcePathCoverageDisposition::ProvenanceOnly => {
            "strict full-corpus audit plus reviewed typed-provenance rationale"
        }
        SourcePathCoverageDisposition::Deferred => {
            "strict full-corpus audit plus exact future-owner checkpoint"
        }
        SourcePathCoverageDisposition::Unknown => "strict coverage failure",
    }
}

fn collect_creature_survival(
    value: &Value,
    context: &RecordContext,
    loaded: Option<&LoadedSourceRecord>,
    inventory: &mut CreatureSurvivalInventory,
) {
    if !is_creature_path(&context.document_type, &context.record_type) {
        return;
    }
    let Some(loaded) = loaded else {
        return;
    };
    let Some(source) = loaded.facts.npc_source.as_ref() else {
        return;
    };
    let candidates = collect_npc_embedded_candidates(source);
    let conversion =
        convert_npc_embedded_entities(loaded.record.identity.key.clone(), &candidates, |_| None);
    collect_expected_canonical_paths("$", "$", value, loaded, &conversion, inventory);
    collect_sequence_closure(value, loaded, inventory);
    collect_reverse_collection_closure(loaded, &conversion, inventory);
    reconcile_reverse_collection_closure(inventory);
    inventory.reverse_expected.clear();
    inventory.reverse_observed.clear();
    inventory.reverse_observed_owners.clear();
    inventory.actual_observation_ordinals.clear();
}

fn collect_sequence_closure(
    source: &Value,
    loaded: &LoadedSourceRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let source_traits = source
        .get("system")
        .and_then(|value| value.get("traits"))
        .and_then(|value| value.get("value"))
        .and_then(Value::as_array)
        .map(|values| {
            let mut values = values
                .iter()
                .filter_map(Value::as_str)
                .map(crate::source::normalize::normalize_text)
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            values.sort();
            values.dedup();
            values
        });
    let Some(expected) = source_traits else {
        return;
    };
    let observed = &loaded.record.classification.traits;
    if expected == *observed {
        return;
    }
    inventory
        .mismatches
        .entry("$.system.traits.value[]".to_string())
        .or_default()
        .push(SourcePathAuditObservationMismatch {
            normalized_path: "$.system.traits.value[]".to_string(),
            record_key: loaded.record.identity.key.to_string(),
            member_identity: format!("record:{}", loaded.record.identity.key),
            contextual_source_path: "$.system.traits.value[]".to_string(),
            destination: "canonical::AtlasRecord::classification.traits".to_string(),
            expected_state: "value".to_string(),
            expected_type: "array".to_string(),
            expected_value: canonical_json(&serde_json::json!(expected)),
            observed_state: "value".to_string(),
            observed_type: "array".to_string(),
            observed_value: canonical_json(&serde_json::json!(observed)),
            expected_multiplicity: expected.len(),
            observed_multiplicity: observed.len(),
            expected_order: None,
            observed_order: None,
        });
}

fn reverse_closed_destination(destination: &str) -> bool {
    destination != "canonical::missing_destination"
}

fn reverse_observation(
    record_key: &str,
    member_identity: String,
    contextual_source_path: String,
    normalized_path: String,
    destination: &str,
    payload: CanonicalObservationPayload,
    order: usize,
) -> ReverseObservation {
    let order = if normalized_path == "$.items[].system.traits.value[]" {
        0
    } else {
        order
    };
    ReverseObservation {
        record_key: record_key.to_string(),
        member_identity,
        contextual_source_path,
        normalized_path,
        destination: destination.to_string(),
        state: payload.state.to_string(),
        value_type: payload.value_type,
        normalized_value: payload.normalized_value,
        order,
    }
}

fn insert_reverse_observation(
    observations: &mut BTreeMap<ReverseObservation, usize>,
    observation: ReverseObservation,
) {
    *observations.entry(observation).or_default() += 1;
}

fn insert_actual_observation(
    inventory: &mut CreatureSurvivalInventory,
    observation: ReverseObservation,
) {
    inventory.reverse_observed_owners.insert((
        observation.record_key.clone(),
        observation.member_identity.clone(),
        observation.normalized_path.clone(),
    ));
    insert_reverse_observation(&mut inventory.reverse_observed, observation);
}

fn emit_actual_payload(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member_identity: &str,
    normalized_path: &str,
    destination: &'static str,
    payload: CanonicalObservationPayload,
) {
    if payload.state != "value" {
        return;
    }
    if payload.value_type == "string"
        && serde_json::from_str::<String>(&payload.normalized_value)
            .is_ok_and(|value| value.trim().is_empty())
    {
        return;
    }
    let ordinal = inventory
        .actual_observation_ordinals
        .entry((
            record_key.to_string(),
            member_identity.to_string(),
            normalized_path.to_string(),
        ))
        .or_default();
    let order = *ordinal;
    *ordinal += 1;
    insert_actual_observation(
        inventory,
        reverse_observation(
            record_key,
            member_identity.to_string(),
            normalized_path.to_string(),
            normalized_path.to_string(),
            destination,
            payload,
            order,
        ),
    );
}

fn emit_actual_fact<T>(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member_identity: &str,
    normalized_path: &str,
    destination: &'static str,
    fact: &FactValue<T>,
    map: impl FnOnce(&T) -> Value,
) {
    emit_actual_payload(
        inventory,
        record_key,
        member_identity,
        normalized_path,
        destination,
        fact_payload(fact, map),
    );
}

fn collect_actual_record_fields(
    loaded: &LoadedSourceRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    for (path, destination, value) in [
        (
            "$._id",
            "canonical::AtlasRecord::identity.key.id",
            Value::String(loaded.record.identity.key.id().to_string()),
        ),
        (
            "$.name",
            "canonical::AtlasRecord::identity.name",
            Value::String(loaded.record.identity.name.clone()),
        ),
        (
            "$.type",
            "canonical::AtlasRecord::foundry.record_type",
            Value::String(loaded.record.foundry.record_type.as_str().to_string()),
        ),
    ] {
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            path,
            destination,
            CanonicalObservationPayload::value(value),
        );
    }
    if let Some(folder) = &loaded.record.foundry.folder_id {
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.folder",
            "canonical::AtlasRecord::foundry.folder_id",
            CanonicalObservationPayload::value(Value::String(folder.clone())),
        );
    }
    if let Some(level) = loaded.record.classification.level {
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.system.details.level.value",
            "canonical::AtlasRecord::classification.level + mechanics.metrics",
            CanonicalObservationPayload::value(Value::Number(level.into())),
        );
    }
    if let Some(rarity) = loaded.record.classification.rarity {
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.system.traits.rarity",
            "canonical::AtlasRecord::classification.rarity",
            CanonicalObservationPayload::value(Value::String(rarity.as_str().to_string())),
        );
    }
    for value in &loaded.record.classification.traits {
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.system.traits.value[]",
            "canonical::AtlasRecord::classification.traits",
            CanonicalObservationPayload::value(Value::String(value.clone())),
        );
    }
    for metric in &loaded.record.mechanics.metrics {
        let Some(ability) = metric
            .key
            .strip_prefix("ability.")
            .and_then(|key| key.strip_suffix(".mod"))
        else {
            continue;
        };
        let value = match &metric.value {
            atlas_record::MetricValue::Number(value) if value.fract() == 0.0 => {
                Value::Number((*value as i64).into())
            }
            atlas_record::MetricValue::Number(value) => {
                let Some(value) = serde_json::Number::from_f64(*value) else {
                    continue;
                };
                Value::Number(value)
            }
            atlas_record::MetricValue::Text(value) => Value::String(value.clone()),
            atlas_record::MetricValue::Boolean(value) => Value::Bool(*value),
        };
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            &format!("$.system.abilities.{ability}.mod"),
            "canonical::AtlasRecord::mechanics.metrics[ability.*.mod]",
            CanonicalObservationPayload::value(value),
        );
    }
}

fn collect_actual_creature_fields(
    loaded: &LoadedSourceRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(creature) = canonical_creature(loaded) else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    emit_actual_fact(
        inventory,
        &record_key,
        &member,
        "$.system.traits.size.value",
        "canonical::CreatureRecord::size",
        &creature.size.value,
        |value| Value::String(value.as_source().to_string()),
    );
    if let Some(publication) = creature.publication.value.as_value() {
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.details.publication.title",
            "canonical::CreaturePublication::title",
            &publication.title,
            |value| Value::String(value.clone()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.details.publication.remaster",
            "canonical::CreaturePublication::remaster",
            &publication.remaster,
            |value| Value::Bool(*value),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.details.publication.license",
            "canonical::CreaturePublication::license",
            &publication.license,
            |value| Value::String(value.as_str().to_string()),
        );
    }
    emit_actual_payload(
        inventory,
        &record_key,
        &member,
        "$.system.attributes.adjustment",
        "canonical::CreatureRecord::adjustment",
        match &creature.adjustment.value {
            FactValue::Missing => CanonicalObservationPayload::missing(),
            FactValue::Null => CanonicalObservationPayload::null(),
            FactValue::Value(atlas_record::CreatureAdjustment::Elite) => {
                CanonicalObservationPayload::value(Value::String("elite".to_string()))
            }
            FactValue::Value(atlas_record::CreatureAdjustment::Weak) => {
                CanonicalObservationPayload::value(Value::String("weak".to_string()))
            }
            FactValue::Value(atlas_record::CreatureAdjustment::Unsupported(value)) => {
                unsupported_source_payload(value)
            }
        },
    );
    emit_actual_payload(
        inventory,
        &record_key,
        &member,
        "$.system.details.alliance",
        "canonical::CreatureRecord::source_alliance",
        match &creature.source_alliance.value {
            FactValue::Missing => CanonicalObservationPayload::missing(),
            FactValue::Null => CanonicalObservationPayload::null(),
            FactValue::Value(atlas_record::CreatureSourceAlliance::Named(value)) => {
                CanonicalObservationPayload::value(Value::String(value.as_str().to_string()))
            }
            FactValue::Value(atlas_record::CreatureSourceAlliance::Unsupported(value)) => {
                unsupported_source_payload(value)
            }
        },
    );
    collect_actual_perception(loaded, creature, inventory);
    collect_actual_languages(loaded, creature, inventory);
    collect_actual_abilities(loaded, creature, inventory);
    collect_actual_defenses(loaded, creature, inventory);
    collect_actual_movement(loaded, creature, inventory);
    collect_actual_skills(loaded, creature, inventory);
    collect_actual_resources(loaded, creature, inventory);
}

fn collect_actual_perception(
    loaded: &LoadedSourceRecord,
    creature: &atlas_record::CreatureRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    if let Some(perception) = creature.perception.value.as_value() {
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.perception.mod",
            "canonical::CreatureRecord::perception",
            &perception.modifier,
            |value| Value::Number((*value).into()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.perception.details",
            "canonical::CreatureRecord::perception",
            &perception.details,
            |value| Value::String(value.as_str().to_string()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.perception.vision",
            "canonical::CreatureRecord::perception",
            &perception.has_vision,
            |value| Value::Bool(*value),
        );
        if let Some(senses) = perception.senses.as_value() {
            for sense in senses {
                emit_actual_payload(
                    inventory,
                    &record_key,
                    &member,
                    "$.system.perception.senses[].type",
                    "canonical::CreatureRecord::perception",
                    CanonicalObservationPayload::value(Value::String(
                        sense.sense_type.as_str().to_string(),
                    )),
                );
                emit_actual_fact(
                    inventory,
                    &record_key,
                    &member,
                    "$.system.perception.senses[].range",
                    "canonical::CreatureRecord::perception",
                    &sense.range,
                    |value| Value::Number((*value).into()),
                );
                emit_actual_payload(
                    inventory,
                    &record_key,
                    &member,
                    "$.system.perception.senses[].acuity",
                    "canonical::CreatureRecord::perception",
                    match &sense.acuity {
                        FactValue::Missing => CanonicalObservationPayload::missing(),
                        FactValue::Null => CanonicalObservationPayload::null(),
                        FactValue::Value(atlas_record::SenseAcuity::Precise) => {
                            CanonicalObservationPayload::value(Value::String("precise".to_string()))
                        }
                        FactValue::Value(atlas_record::SenseAcuity::Imprecise) => {
                            CanonicalObservationPayload::value(Value::String(
                                "imprecise".to_string(),
                            ))
                        }
                        FactValue::Value(atlas_record::SenseAcuity::Vague) => {
                            CanonicalObservationPayload::value(Value::String("vague".to_string()))
                        }
                        FactValue::Value(atlas_record::SenseAcuity::Unsupported(value)) => {
                            unsupported_source_payload(value)
                        }
                    },
                );
            }
        }
    }
    if let Some(initiative) = creature.initiative.value.as_value() {
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.system.initiative.statistic",
            "canonical::CreatureRecord::initiative",
            match &initiative.statistic {
                FactValue::Missing => CanonicalObservationPayload::missing(),
                FactValue::Null => CanonicalObservationPayload::null(),
                FactValue::Value(atlas_record::CreatureInitiativeStatistic::Named(value)) => {
                    CanonicalObservationPayload::value(Value::String(value.as_str().to_string()))
                }
                FactValue::Value(atlas_record::CreatureInitiativeStatistic::Unsupported(value)) => {
                    unsupported_source_payload(value)
                }
            },
        );
    }
    if let Some(source) = loaded.facts.npc_source.as_ref()
        && let Some(perception) = source.source.core.perception.as_value()
    {
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.system.perception.value",
            "typed_dto::NpcPerceptionSource::legacy_value",
            source_presence_payload(&perception.legacy_value, |value| {
                Value::Number((*value).into())
            }),
        );
    }
}

fn collect_actual_languages(
    loaded: &LoadedSourceRecord,
    creature: &atlas_record::CreatureRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(languages) = creature.languages.value.as_value() else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    if let Some(values) = languages.values.as_value() {
        for value in values {
            emit_actual_payload(
                inventory,
                &record_key,
                &member,
                "$.system.details.languages.value[]",
                "canonical::CreatureRecord::languages",
                CanonicalObservationPayload::value(Value::String(value.as_str().to_string())),
            );
        }
    }
    emit_actual_fact(
        inventory,
        &record_key,
        &member,
        "$.system.details.languages.details",
        "canonical::CreatureRecord::languages",
        &languages.details,
        |value| Value::String(value.as_str().to_string()),
    );
}

fn collect_actual_abilities(
    loaded: &LoadedSourceRecord,
    creature: &atlas_record::CreatureRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(abilities) = creature.legacy_abilities.value.as_value() else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    for (ability, fact) in [
        ("str", &abilities.strength),
        ("dex", &abilities.dexterity),
        ("con", &abilities.constitution),
        ("int", &abilities.intelligence),
        ("wis", &abilities.wisdom),
        ("cha", &abilities.charisma),
    ] {
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            &format!("$.system.abilities.{ability}.value"),
            "canonical::CreatureRecord::legacy_abilities",
            fact,
            |value| Value::Number((*value).into()),
        );
    }
}

fn collect_actual_defenses(
    loaded: &LoadedSourceRecord,
    creature: &atlas_record::CreatureRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(defenses) = creature.defenses.value.as_value() else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    if let Some(armor) = defenses.armor_class.as_value() {
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.attributes.ac.value",
            "canonical::CreatureDefenses::armor_class",
            &armor.value,
            |value| Value::Number((*value).into()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.attributes.ac.details",
            "canonical::CreatureDefenses::armor_class",
            &armor.details,
            |value| Value::String(value.as_str().to_string()),
        );
    }
    if let Some(hp) = defenses.hit_points.as_value() {
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.system.attributes.hp.value",
            "canonical::CreatureDefenses::hit_points",
            match &hp.value {
                FactValue::Missing => CanonicalObservationPayload::missing(),
                FactValue::Null => CanonicalObservationPayload::null(),
                FactValue::Value(atlas_record::CreatureNumber::Integer(value)) => {
                    CanonicalObservationPayload::value(Value::Number((*value).into()))
                }
                FactValue::Value(atlas_record::CreatureNumber::Unsupported(value)) => {
                    unsupported_source_payload(value)
                }
            },
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.attributes.hp.max",
            "canonical::CreatureDefenses::hit_points",
            &hp.maximum,
            |value| Value::Number((*value).into()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.attributes.hp.temp",
            "canonical::CreatureDefenses::hit_points",
            &hp.temporary,
            |value| Value::Number((*value).into()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.attributes.hp.tempmax",
            "canonical::CreatureDefenses::hit_points",
            &hp.temporary_maximum,
            |value| Value::Number((*value).into()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.system.attributes.hp.details",
            "canonical::CreatureDefenses::hit_points",
            &hp.details,
            |value| Value::String(value.as_str().to_string()),
        );
    }
    emit_actual_fact(
        inventory,
        &record_key,
        &member,
        "$.system.attributes.hardness.value",
        "canonical::CreatureDefenses::hardness",
        &defenses.hardness,
        |value| Value::Number((*value).into()),
    );
    emit_actual_fact(
        inventory,
        &record_key,
        &member,
        "$.system.attributes.allSaves.value",
        "canonical::CreatureDefenses::all_saves_note",
        &defenses.all_saves_note,
        |value| Value::String(value.as_str().to_string()),
    );
    if let Some(shield) = defenses.shield.as_value() {
        for (path, fact) in [
            ("$.system.attributes.shield.ac", &shield.armor_class_bonus),
            (
                "$.system.attributes.shield.brokenThreshold",
                &shield.broken_threshold,
            ),
            ("$.system.attributes.shield.hardness", &shield.hardness),
            ("$.system.attributes.shield.max", &shield.maximum_hit_points),
            (
                "$.system.attributes.shield.value",
                &shield.serialized_hit_points,
            ),
        ] {
            emit_actual_fact(
                inventory,
                &record_key,
                &member,
                path,
                "canonical::CreatureDefenses::shield",
                fact,
                |value| Value::Number((*value).into()),
            );
        }
    }
    if let Some(saves) = defenses.saves.as_value() {
        for save in [&saves.fortitude, &saves.reflex, &saves.will]
            .into_iter()
            .filter_map(FactValue::as_value)
        {
            emit_actual_fact(
                inventory,
                &record_key,
                &member,
                "$.system.saves.*.value",
                "canonical::CreatureDefenses::saves",
                &save.value,
                |value| Value::Number((*value).into()),
            );
            emit_actual_fact(
                inventory,
                &record_key,
                &member,
                "$.system.saves.*.saveDetail",
                "canonical::CreatureDefenses::saves",
                &save.details,
                |value| Value::String(value.as_str().to_string()),
            );
        }
    }
    for (prefix, destination, fact) in [
        (
            "$.system.attributes.immunities[]",
            "canonical::CreatureDefenses::immunities",
            &defenses.immunities,
        ),
        (
            "$.system.attributes.resistances[]",
            "canonical::CreatureDefenses::resistances",
            &defenses.resistances,
        ),
        (
            "$.system.attributes.weaknesses[]",
            "canonical::CreatureDefenses::weaknesses",
            &defenses.weaknesses,
        ),
    ] {
        if let Some(entries) = fact.as_value() {
            for entry in entries {
                emit_actual_payload(
                    inventory,
                    &record_key,
                    &member,
                    &format!("{prefix}.type"),
                    destination,
                    CanonicalObservationPayload::value(Value::String(
                        entry.iwr_type.as_str().to_string(),
                    )),
                );
                emit_actual_fact(
                    inventory,
                    &record_key,
                    &member,
                    &format!("{prefix}.value"),
                    destination,
                    &entry.value,
                    |value| Value::Number((*value).into()),
                );
                if let Some(values) = entry.exceptions.as_value() {
                    for value in values {
                        emit_actual_payload(
                            inventory,
                            &record_key,
                            &member,
                            &format!("{prefix}.exceptions[]"),
                            destination,
                            CanonicalObservationPayload::value(Value::String(
                                value.as_str().to_string(),
                            )),
                        );
                    }
                }
                if let Some(values) = entry.double_vs.as_value() {
                    for value in values {
                        emit_actual_payload(
                            inventory,
                            &record_key,
                            &member,
                            &format!("{prefix}.doubleVs[]"),
                            destination,
                            CanonicalObservationPayload::value(Value::String(
                                value.as_str().to_string(),
                            )),
                        );
                    }
                }
                emit_actual_fact(
                    inventory,
                    &record_key,
                    &member,
                    &format!("{prefix}.applyOnce"),
                    destination,
                    &entry.apply_once,
                    |value| Value::Bool(*value),
                );
            }
        }
    }
}

fn collect_actual_movement(
    loaded: &LoadedSourceRecord,
    creature: &atlas_record::CreatureRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(speeds) = creature.movement.value.as_value() else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    for speed in speeds {
        let (prefix, include_mode) = if speed.mode == atlas_record::CreatureMovementMode::Land {
            ("$.system.attributes.speed", false)
        } else {
            ("$.system.attributes.speed.otherSpeeds[]", true)
        };
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            &format!("{prefix}.value"),
            "canonical::CreatureRecord::movement",
            &speed.value,
            |value| Value::Number((*value).into()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            &format!("{prefix}.details"),
            "canonical::CreatureRecord::movement",
            &speed.details,
            |value| Value::String(value.as_str().to_string()),
        );
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            &format!("{prefix}.label"),
            "canonical::CreatureRecord::movement",
            &speed.label,
            |value| Value::String(value.clone()),
        );
        if include_mode {
            emit_actual_payload(
                inventory,
                &record_key,
                &member,
                &format!("{prefix}.type"),
                "canonical::CreatureRecord::movement",
                movement_mode_payload(&speed.mode),
            );
        }
    }
}

fn collect_actual_skills(
    loaded: &LoadedSourceRecord,
    creature: &atlas_record::CreatureRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    let canonical = creature.skills.value.as_value();
    let mut emitted = BTreeSet::new();
    if let Some(source) = loaded.facts.npc_source.as_ref()
        && let Some(skills) = source.source.core.skills.as_value()
    {
        for (slug, skill) in skills {
            if let Some(kind) = atlas_record::CreatureSkillKind::from_source_slug(slug) {
                if let Some(skill) = canonical.and_then(|skills| {
                    skills.iter().find(|skill| {
                        skill.kind == kind && skill.source_item_id.as_value().is_none()
                    })
                }) {
                    emit_actual_skill(inventory, &record_key, &member, skill);
                    emitted.insert(kind);
                }
            } else {
                emit_actual_payload(
                    inventory,
                    &record_key,
                    &member,
                    "$.system.skills.*.base",
                    "typed_dto::NpcSkillSource::invalid_shadow_skill",
                    source_presence_payload(&skill.base, |value| Value::Number((*value).into())),
                );
                emit_actual_payload(
                    inventory,
                    &record_key,
                    &member,
                    "$.system.skills.*.note",
                    "typed_dto::NpcSkillSource::invalid_shadow_skill",
                    source_presence_payload(&skill.note, |value| Value::String(value.clone())),
                );
            }
        }
    }
    if let Some(skills) = canonical {
        for skill in skills {
            if skill.kind != atlas_record::CreatureSkillKind::Lore
                && skill.source_item_id.as_value().is_none()
                && !emitted.contains(&skill.kind)
            {
                emit_actual_skill(inventory, &record_key, &member, skill);
            }
        }
    }
}

fn emit_actual_skill(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member: &str,
    skill: &atlas_record::CreatureSkill,
) {
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.system.skills.*.base",
        "canonical::CreatureRecord::skills",
        &skill.modifier,
        |value| Value::Number((*value).into()),
    );
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.system.skills.*.note",
        "canonical::CreatureRecord::skills",
        &skill.note,
        |value| Value::String(value.as_str().to_string()),
    );
    if let Some(variants) = skill.variants.as_value() {
        for variant in variants {
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.system.skills.*.special[].base",
                "canonical::CreatureRecord::skills",
                &variant.modifier,
                |value| Value::Number((*value).into()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.system.skills.*.special[].label",
                "canonical::CreatureRecord::skills",
                &variant.label,
                |value| Value::String(value.clone()),
            );
            if let Some(predicates) = variant.predicate.as_value() {
                for predicate in predicates {
                    collect_actual_predicate_leaves(
                        inventory,
                        record_key,
                        member,
                        &predicate_source_json(predicate),
                    );
                }
            }
        }
    }
}

fn collect_actual_predicate_leaves(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member: &str,
    value: &Value,
) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if is_meaningful_value(value) {
                    emit_actual_payload(
                        inventory,
                        record_key,
                        member,
                        &format!("$.system.skills.*.special[].predicate[].{key}"),
                        "canonical::CreatureRecord::skills",
                        CanonicalObservationPayload::value(value.clone()),
                    );
                }
                if let Some(values) = value.as_array() {
                    for value in values {
                        emit_actual_payload(
                            inventory,
                            record_key,
                            member,
                            &format!("$.system.skills.*.special[].predicate[].{key}[]"),
                            "canonical::CreatureRecord::skills",
                            CanonicalObservationPayload::value(value.clone()),
                        );
                    }
                }
            }
        }
        Value::String(_) | Value::Bool(_) | Value::Number(_) => emit_actual_payload(
            inventory,
            record_key,
            member,
            "$.system.skills.*.special[].predicate[]",
            "canonical::CreatureRecord::skills",
            CanonicalObservationPayload::value(value.clone()),
        ),
        Value::Array(_) | Value::Null => {}
    }
}

fn collect_actual_resources(
    loaded: &LoadedSourceRecord,
    creature: &atlas_record::CreatureRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(resources) = creature.resources.value.as_value() else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let member = format!("record:{record_key}");
    let typed_resources = loaded
        .facts
        .npc_source
        .as_ref()
        .and_then(|source| source.source.core.resources.as_value());
    for resource in resources {
        match typed_resources
            .and_then(|resources| resources.get(resource.kind.as_str()))
            .and_then(|resource| resource.maximum.as_value())
        {
            Some(crate::source::dto::NpcResourceAmountSource::Nested { maximum, value }) => {
                if maximum.as_value().is_some() {
                    emit_actual_payload(
                        inventory,
                        &record_key,
                        &member,
                        "$.system.resources.*.max.max",
                        "canonical::CreatureRecord::resources",
                        resource_amount_payload(&resource.maximum),
                    );
                }
                if value.as_value().is_some() {
                    emit_actual_payload(
                        inventory,
                        &record_key,
                        &member,
                        "$.system.resources.*.max.value",
                        "canonical::CreatureRecord::resources",
                        resource_amount_payload(&resource.maximum),
                    );
                }
            }
            Some(_) => emit_actual_payload(
                inventory,
                &record_key,
                &member,
                "$.system.resources.*.max",
                "canonical::CreatureRecord::resources",
                resource_amount_payload(&resource.maximum),
            ),
            None => emit_actual_payload(
                inventory,
                &record_key,
                &member,
                "$.system.resources.*.max",
                "canonical::CreatureRecord::resources",
                resource_amount_payload(&resource.maximum),
            ),
        }
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.system.resources.*.value",
            "canonical::CreatureRecord::resources",
            resource_amount_payload(&resource.serialized_value),
        );
        if let Some(drift) = resource.source_drift.as_value() {
            for fact in drift {
                if fact.field == atlas_record::CreatureUnsupportedSourceField::ResourceMaximumDrift
                {
                    emit_actual_payload(
                        inventory,
                        &record_key,
                        &member,
                        "$.system.resources.*.maxx",
                        "canonical::CreatureRecord::resources",
                        unsupported_source_payload(&fact.value),
                    );
                }
            }
        }
    }
}

fn collect_actual_embedded_item_fields(
    loaded: &LoadedSourceRecord,
    inventory: &mut CreatureSurvivalInventory,
) {
    let record_key = loaded.record.identity.key.to_string();
    for fact in &loaded.facts.source_facts.embedded_items {
        let member = format!("item:{}", fact.item_id);
        for (path, destination, value) in [
            (
                "$.items[]._id",
                "canonical::EmbeddedItemFact::item_id",
                Some(Value::String(fact.item_id.clone())),
            ),
            (
                "$.items[].name",
                "canonical::EmbeddedItemFact::name",
                Some(Value::String(fact.name.clone())),
            ),
            (
                "$.items[].type",
                "canonical::EmbeddedItemFact::foundry_item_type",
                Some(Value::String(fact.foundry_item_type.clone())),
            ),
            (
                "$.items[]._stats.compendiumSource",
                "canonical::EmbeddedItemFact::compendium_source",
                fact.compendium_source.clone().map(Value::String),
            ),
            (
                "$.items[].system.category",
                "canonical::EmbeddedItemFact::system_category",
                fact.system_category.clone().map(Value::String),
            ),
            (
                "$.items[].system.publication.remaster",
                "canonical::EmbeddedItemFact::publication_remaster",
                Some(Value::Bool(fact.publication_remaster)),
            ),
            (
                "$.items[].system.slug",
                "canonical::EmbeddedItemFact::slug",
                fact.slug.clone().map(Value::String),
            ),
        ] {
            if let Some(value) = value {
                emit_actual_payload(
                    inventory,
                    &record_key,
                    &member,
                    path,
                    destination,
                    CanonicalObservationPayload::value(value),
                );
            }
        }
        for value in &fact.traits {
            emit_actual_payload(
                inventory,
                &record_key,
                &member,
                "$.items[].system.traits.value[]",
                "canonical::EmbeddedItemFact::traits",
                CanonicalObservationPayload::value(Value::String(value.clone())),
            );
        }
    }
}

fn collect_reverse_collection_closure(
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    inventory: &mut CreatureSurvivalInventory,
) {
    collect_actual_record_fields(loaded, inventory);
    collect_actual_creature_fields(loaded, inventory);
    collect_actual_embedded_item_fields(loaded, inventory);
    collect_actual_occurrence_fields(loaded, conversion, inventory);
    collect_reverse_occurrences(loaded, conversion, inventory);
    collect_reverse_relationships(loaded, conversion, inventory);
    collect_reverse_content(loaded, inventory);
    collect_reverse_unsupported(loaded, conversion, inventory);
}

fn collect_actual_occurrence_fields(
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(embedded) = conversion.embedded.as_value() else {
        return;
    };
    let candidates = loaded
        .facts
        .npc_source
        .as_ref()
        .map(collect_npc_embedded_candidates);
    let record_key = loaded.record.identity.key.to_string();
    if let Some(actor) = embedded.actor_spellcasting.as_value() {
        let member = format!("record:{record_key}");
        emit_actual_payload(
            inventory,
            &record_key,
            &member,
            "$.system.spellcasting.rituals.dc",
            "canonical::CreatureActorSpellcastingContext::rituals_dc",
            source_scalar_i64_payload(&actor.rituals_dc),
        );
    }
    for (occurrence_index, occurrence) in embedded.occurrences.iter().enumerate() {
        let member = occurrence
            .source_identity
            .nested_source_id
            .as_value()
            .map(|id| format!("item:{}", id.as_str()))
            .unwrap_or_else(|| format!("occurrence:{}", occurrence.id.as_str()));
        let candidate = occurrence
            .source_identity
            .nested_source_id
            .as_value()
            .and_then(|id| {
                let duplicate_ordinal = embedded.occurrences[..occurrence_index]
                    .iter()
                    .filter(|candidate| {
                        candidate
                            .source_identity
                            .nested_source_id
                            .as_value()
                            .is_some_and(|candidate_id| candidate_id.as_str() == id.as_str())
                    })
                    .count();
                candidates
                    .as_ref()?
                    .items
                    .as_value()?
                    .iter()
                    .filter(|candidate| candidate.nested_source_id == id.as_str())
                    .nth(duplicate_ordinal)
            });
        emit_actual_fact(
            inventory,
            &record_key,
            &member,
            "$.items[].sort",
            "canonical::CreatureEntityOccurrence::source_sort",
            &occurrence.source_sort,
            |value| Value::Number((*value).into()),
        );
        for locator in &occurrence.source_identity.source_locators {
            if locator.source_path != "$.flags.core.sourceId" {
                continue;
            }
            let path = normalize_observed_source_path(&format!(
                "$.items[]{}",
                locator
                    .source_path
                    .strip_prefix('$')
                    .unwrap_or(&locator.source_path)
            ));
            if inventory.reverse_observed_owners.contains(&(
                record_key.clone(),
                member.clone(),
                path.clone(),
            )) {
                continue;
            }
            emit_actual_payload(
                inventory,
                &record_key,
                &member,
                &path,
                "canonical::CreatureEntitySourceIdentity::source_locators",
                CanonicalObservationPayload::value(Value::String(
                    locator.locator.as_str().to_string(),
                )),
            );
        }
        collect_actual_capability(inventory, &record_key, &member, occurrence, candidate);
    }
}

fn collect_actual_capability(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member: &str,
    occurrence: &atlas_record::CreatureEntityOccurrence,
    candidate: Option<&crate::source::npc_entities::NpcEmbeddedCandidate>,
) {
    match &occurrence.capability {
        CreatureCapability::Action(action) => {
            if !matches!(
                action.action_cost,
                atlas_record::CreatureActionCost::Unsupported(_)
            ) {
                emit_actual_payload(
                    inventory,
                    record_key,
                    member,
                    "$.items[].system.actionType.value",
                    "canonical::CreatureActionCapability::action_cost",
                    action_type_payload(&action.action_cost),
                );
                emit_actual_payload(
                    inventory,
                    record_key,
                    member,
                    "$.items[].system.actions.value",
                    "canonical::CreatureActionCapability::action_cost",
                    action_count_payload(&action.action_cost),
                );
            }
            if let Some(frequency) = action.frequency.as_value() {
                emit_actual_fact(
                    inventory,
                    record_key,
                    member,
                    "$.items[].system.frequency.max",
                    "canonical::CreatureActionCapability::frequency",
                    &frequency.maximum,
                    |value| Value::Number((*value).into()),
                );
                emit_actual_fact(
                    inventory,
                    record_key,
                    member,
                    "$.items[].system.frequency.per",
                    "canonical::CreatureActionCapability::frequency",
                    &frequency.period,
                    |value| Value::String(value.clone()),
                );
                emit_actual_fact(
                    inventory,
                    record_key,
                    member,
                    "$.items[].system.frequency.value",
                    "canonical::CreatureActionCapability::frequency",
                    &frequency.serialized_value,
                    |value| Value::Number((*value).into()),
                );
            }
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.selfEffect.uuid",
                "canonical::CreatureActionCapability::self_effect",
                &action.self_effect,
                |value| Value::String(value.clone()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.selfEffect.name",
                "canonical::CreatureActionCapability::self_effect_label",
                &action.self_effect_label,
                |value| Value::String(value.clone()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.requirements",
                "canonical::CreatureActionCapability::requirements",
                &action.requirements,
                |value| Value::String(value.clone()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.cost.value",
                "canonical::CreatureActionCapability::cost",
                &action.cost,
                |value| Value::String(value.clone()),
            );
            collect_actual_rolls(inventory, record_key, member, &action.rolls, false);
            collect_actual_damage(
                inventory,
                record_key,
                member,
                &action.damage,
                "$.items[].system.damageRolls.*",
                "canonical::CreatureActionCapability::damage",
                true,
            );
        }
        CreatureCapability::Strike(strike) => {
            collect_actual_rolls(inventory, record_key, member, &strike.rolls, true);
            if let Some(values) = strike.attack_effects.as_value() {
                for value in values {
                    emit_actual_payload(
                        inventory,
                        record_key,
                        member,
                        "$.items[].system.attackEffects.value[]",
                        "canonical::CreatureStrikeCapability::attack_effects",
                        CanonicalObservationPayload::value(Value::String(value.clone())),
                    );
                }
            }
            collect_actual_damage(
                inventory,
                record_key,
                member,
                &strike.damage,
                "$.items[].system.damageRolls.*",
                "canonical::CreatureStrikeCapability::damage",
                true,
            );
        }
        CreatureCapability::SpellcastingEntry(entry) => {
            emit_actual_payload(
                inventory,
                record_key,
                member,
                "$.items[].system.prepared.value",
                "canonical::CreatureSpellcastingEntryCapability::preparation",
                spell_preparation_payload(&entry.preparation),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.tradition.value",
                "canonical::CreatureSpellcastingEntryCapability::tradition",
                &entry.tradition,
                |value| Value::String(value.clone()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.spelldc.value",
                "canonical::CreatureSpellcastingEntryCapability::attack",
                &entry.attack,
                |value| Value::Number((*value).into()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.spelldc.dc",
                "canonical::CreatureSpellcastingEntryCapability::dc",
                &entry.dc,
                |value| Value::Number((*value).into()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.autoHeightenLevel.value",
                "canonical::CreatureEntityOccurrence::context.rank",
                &occurrence.context.rank,
                |value| Value::Number((*value).into()),
            );
            let source = candidate.and_then(|candidate| match &candidate.source {
                crate::source::dto::NpcEmbeddedItemSource::SpellcastingEntry(source) => {
                    Some(source)
                }
                _ => None,
            });
            collect_actual_spell_slots(inventory, record_key, member, entry, source);
        }
        CreatureCapability::Spell(spell) => {
            let source = candidate.and_then(|candidate| match &candidate.source {
                crate::source::dto::NpcEmbeddedItemSource::Spell(source) => Some(source.as_ref()),
                _ => None,
            });
            collect_actual_spell(inventory, record_key, member, occurrence, spell, source)
        }
        CreatureCapability::Equipment(equipment) => {
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.level.value",
                "canonical::CreatureEquipmentCapability::level",
                &equipment.level,
                |value| Value::Number((*value).into()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.usage.value",
                "canonical::CreatureEquipmentCapability::usage",
                &equipment.usage,
                |value| Value::String(value.clone()),
            );
            emit_actual_fact(
                inventory,
                record_key,
                member,
                "$.items[].system.quantity",
                "canonical::CreatureEquipmentCapability::quantity",
                &equipment.quantity,
                |value| Value::Number((*value).into()),
            );
            if let Some(uses) = equipment.uses.as_value() {
                emit_actual_fact(
                    inventory,
                    record_key,
                    member,
                    "$.items[].system.uses.max",
                    "canonical::CreatureEquipmentCapability::uses",
                    &uses.maximum,
                    |value| Value::Number((*value).into()),
                );
                emit_actual_fact(
                    inventory,
                    record_key,
                    member,
                    "$.items[].system.uses.value",
                    "canonical::CreatureEquipmentCapability::uses",
                    &uses.serialized_value,
                    |value| Value::Number((*value).into()),
                );
            }
        }
        CreatureCapability::Lore(lore) => emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.mod.value",
            "canonical::CreatureLoreCapability::modifier",
            &lore.modifier,
            |value| Value::Number((*value).into()),
        ),
        CreatureCapability::Unsupported(_) => {}
    }
}

fn collect_actual_rolls(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member: &str,
    rolls: &[atlas_record::CreatureRoll],
    strike: bool,
) {
    for roll in rolls {
        let (path, destination) = match roll.id.as_str() {
            "attack" => (
                "$.items[].system.bonus.value",
                "canonical::CreatureStrikeCapability::rolls[attack]",
            ),
            "check" => (
                "$.items[].system.bonus.value",
                "canonical::CreatureActionCapability::rolls[check]",
            ),
            "dc" => (
                "$.items[].system.dc.value",
                "canonical::CreatureActionCapability::rolls[dc]",
            ),
            _ if strike => (
                "$.items[].system.bonus.value",
                "canonical::CreatureStrikeCapability::rolls[attack]",
            ),
            _ => (
                "$.items[].system.bonus.value",
                "canonical::CreatureActionCapability::rolls[check]",
            ),
        };
        emit_actual_fact(
            inventory,
            record_key,
            member,
            path,
            destination,
            &roll.value,
            |value| Value::Number((*value).into()),
        );
    }
}

fn collect_actual_damage(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member: &str,
    damage: &FactValue<Vec<atlas_record::CreatureDamage>>,
    prefix: &str,
    destination: &'static str,
    legacy: bool,
) {
    let Some(entries) = damage.as_value() else {
        return;
    };
    for entry in entries {
        let formula_field = if legacy { "damage" } else { "formula" };
        let type_field = if legacy { "damageType" } else { "type" };
        emit_actual_fact(
            inventory,
            record_key,
            member,
            &format!("{prefix}.{formula_field}"),
            destination,
            &entry.formula,
            |value| Value::String(value.clone()),
        );
        emit_actual_fact(
            inventory,
            record_key,
            member,
            &format!("{prefix}.{type_field}"),
            destination,
            &entry.damage_type,
            |value| Value::String(value.clone()),
        );
        emit_actual_fact(
            inventory,
            record_key,
            member,
            &format!("{prefix}.category"),
            destination,
            &entry.category,
            |value| Value::String(value.clone()),
        );
        if !legacy {
            if let Some(kinds) = entry.kinds.as_value() {
                for kind in kinds {
                    let payload = match kind {
                        atlas_record::CreatureDamageKind::Damage => {
                            CanonicalObservationPayload::value(Value::String("damage".to_string()))
                        }
                        atlas_record::CreatureDamageKind::Healing => {
                            CanonicalObservationPayload::value(Value::String("healing".to_string()))
                        }
                        atlas_record::CreatureDamageKind::Unsupported(value) => {
                            unsupported_source_payload(value)
                        }
                    };
                    emit_actual_payload(
                        inventory,
                        record_key,
                        member,
                        &format!("{prefix}.kinds[]"),
                        destination,
                        payload,
                    );
                }
            }
            emit_actual_payload(
                inventory,
                record_key,
                member,
                &format!("{prefix}.applyMod"),
                destination,
                source_scalar_bool_payload(&entry.apply_modifier),
            );
        }
    }
}

fn collect_actual_spell_slots(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member: &str,
    entry: &atlas_record::CreatureSpellcastingEntryCapability,
    source: Option<&crate::source::dto::SpellcastingEntrySource>,
) {
    let Some(slots) = entry.slots.as_value() else {
        return;
    };
    let mut slots = slots.iter().collect::<Vec<_>>();
    slots.sort_by_key(|slot| format!("slot{}", slot.rank));
    for slot in slots {
        let source_slot = source.and_then(|source| {
            source
                .slots
                .as_value()?
                .iter()
                .find(|candidate| candidate.rank == slot.rank)
        });
        if source_slot.is_none_or(|slot| slot.maximum.as_value().is_some()) {
            emit_actual_payload(
                inventory,
                record_key,
                member,
                "$.items[].system.slots.*.max",
                "canonical::CreatureSpellcastingEntryCapability::slots",
                source_scalar_i64_payload(&slot.maximum),
            );
        }
        if source_slot.is_none_or(|slot| slot.value.as_value().is_some()) {
            emit_actual_payload(
                inventory,
                record_key,
                member,
                "$.items[].system.slots.*.value",
                "canonical::CreatureSpellcastingEntryCapability::slots",
                source_scalar_i64_payload(&slot.serialized_value),
            );
        }
        if let Some(prepared) = slot.prepared.as_value() {
            for (prepared_index, prepared) in prepared.iter().enumerate() {
                let source_prepared =
                    source_slot.and_then(|slot| slot.prepared.as_value()?.get(prepared_index));
                match prepared {
                    atlas_record::CreaturePreparedSpellSlot::Unsupported(value) => {
                        emit_actual_payload(
                            inventory,
                            record_key,
                            member,
                            "$.items[].system.slots.*.prepared[]",
                            "canonical::CreatureSpellcastingEntryCapability::slots",
                            unsupported_source_payload(value),
                        )
                    }
                    atlas_record::CreaturePreparedSpellSlot::Spell {
                        id,
                        name,
                        expended,
                        prepared,
                        ..
                    } => {
                        if source_prepared.is_none_or(|source| {
                            matches!(
                                source,
                                crate::source::dto::PreparedSlotSource::Spell {
                                    id: SourcePresence::Value(_),
                                    ..
                                }
                            )
                        }) {
                            emit_actual_fact(
                                inventory,
                                record_key,
                                member,
                                "$.items[].system.slots.*.prepared[].id",
                                "canonical::CreatureSpellcastingEntryCapability::slots",
                                id,
                                |value| Value::String(value.as_str().to_string()),
                            );
                        }
                        emit_actual_fact(
                            inventory,
                            record_key,
                            member,
                            "$.items[].system.slots.*.prepared[].name",
                            "canonical::CreatureSpellcastingEntryCapability::slots",
                            name,
                            |value| Value::String(value.clone()),
                        );
                        emit_actual_fact(
                            inventory,
                            record_key,
                            member,
                            "$.items[].system.slots.*.prepared[].expended",
                            "canonical::CreatureSpellcastingEntryCapability::slots",
                            expended,
                            |value| Value::Bool(*value),
                        );
                        emit_actual_fact(
                            inventory,
                            record_key,
                            member,
                            "$.items[].system.slots.*.prepared[].prepared",
                            "canonical::CreatureSpellcastingEntryCapability::slots",
                            prepared,
                            |value| Value::Bool(*value),
                        );
                    }
                }
            }
        }
    }
}

fn collect_actual_spell(
    inventory: &mut CreatureSurvivalInventory,
    record_key: &str,
    member: &str,
    occurrence: &atlas_record::CreatureEntityOccurrence,
    spell: &atlas_record::CreatureSpellCapability,
    source: Option<&crate::source::dto::SpellSource>,
) {
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.level.value",
        "canonical::CreatureSpellCapability::base_rank",
        &spell.base_rank,
        |value| Value::Number((*value).into()),
    );
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.location.value",
        "canonical::CreatureEntityOccurrence::context.location",
        &occurrence.context.location,
        |value| Value::String(value.clone()),
    );
    if source.is_none_or(|source| source.heightened_level.as_value().is_some()) {
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.location.heightenedLevel",
            "canonical::CreatureEntityOccurrence::context.rank",
            &occurrence.context.rank,
            |value| Value::Number((*value).into()),
        );
    }
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.location.signature",
        "canonical::CreatureSpellCapability::signature",
        &spell.signature,
        |value| Value::Bool(*value),
    );
    if let Some(uses) = occurrence.context.uses.as_value() {
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.location.uses.max",
            "canonical::CreatureEntityOccurrence::context.uses",
            &uses.maximum,
            |value| Value::Number((*value).into()),
        );
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.location.uses.value",
            "canonical::CreatureEntityOccurrence::context.uses",
            &uses.serialized_value,
            |value| Value::Number((*value).into()),
        );
    }
    if let Some(values) = spell.traditions.as_value() {
        for value in values {
            emit_actual_payload(
                inventory,
                record_key,
                member,
                "$.items[].system.traits.traditions[]",
                "canonical::CreatureSpellCapability::traditions",
                CanonicalObservationPayload::value(Value::String(value.clone())),
            );
        }
    }
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.requirements",
        "canonical::CreatureSpellCapability::requirements",
        &spell.requirements,
        |value| Value::String(value.clone()),
    );
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.cost.value",
        "canonical::CreatureSpellCapability::cost",
        &spell.cost,
        |value| Value::String(value.clone()),
    );
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.counteraction",
        "canonical::CreatureSpellCapability::counteraction",
        &spell.counteraction,
        |value| Value::Bool(*value),
    );
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.target.value",
        "canonical::CreatureSpellCapability::target",
        &spell.target,
        |value| Value::String(value.clone()),
    );
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.range.value",
        "canonical::CreatureSpellCapability::range",
        &spell.range,
        |value| Value::String(value.clone()),
    );
    emit_actual_fact(
        inventory,
        record_key,
        member,
        "$.items[].system.time.value",
        "canonical::CreatureSpellCapability::time",
        &spell.time,
        |value| Value::String(value.clone()),
    );
    if let Some(ritual) = spell.ritual.as_value() {
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.ritual.primary.check",
            "canonical::CreatureSpellCapability::ritual",
            &ritual.primary_check,
            |value| Value::String(value.clone()),
        );
        emit_actual_payload(
            inventory,
            record_key,
            member,
            "$.items[].system.ritual.secondary.casters",
            "canonical::CreatureSpellCapability::ritual",
            source_scalar_i64_payload(&ritual.secondary_casters),
        );
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.ritual.secondary.checks",
            "canonical::CreatureSpellCapability::ritual",
            &ritual.secondary_checks,
            |value| Value::String(value.clone()),
        );
    }
    if let Some(area) = spell.area.as_value() {
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.area.type",
            "canonical::CreatureSpellCapability::area",
            &area.area_type,
            |value| Value::String(value.clone()),
        );
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.area.value",
            "canonical::CreatureSpellCapability::area",
            &area.value,
            |value| Value::Number((*value).into()),
        );
    }
    if let Some(duration) = spell.duration.as_value() {
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.duration.value",
            "canonical::CreatureSpellCapability::duration",
            &duration.value,
            |value| Value::String(value.clone()),
        );
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.duration.sustained",
            "canonical::CreatureSpellCapability::duration",
            &duration.sustained,
            |value| Value::Bool(*value),
        );
    }
    if let Some(defense) = spell.defense.as_value() {
        emit_actual_payload(
            inventory,
            record_key,
            member,
            "$.items[].system.defense.save.statistic",
            "canonical::CreatureSpellCapability::defense",
            spell_save_payload(&defense.save),
        );
        emit_actual_fact(
            inventory,
            record_key,
            member,
            "$.items[].system.defense.save.basic",
            "canonical::CreatureSpellCapability::defense",
            &defense.basic,
            |value| Value::Bool(*value),
        );
    }
    collect_actual_damage(
        inventory,
        record_key,
        member,
        &spell.damage,
        "$.items[].system.damage.*",
        "canonical::CreatureSpellCapability::damage",
        false,
    );
}

fn collect_reverse_occurrences(
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(source) = loaded.facts.npc_source.as_ref() else {
        return;
    };
    let candidates = collect_npc_embedded_candidates(source);
    let Some(candidates) = candidates.items.as_value() else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let mut ordered = candidates.iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by(|(left_index, left), (right_index, right)| {
        match (left.sort.as_value(), right.sort.as_value()) {
            (Some(left_sort), Some(right_sort)) => left_sort
                .cmp(right_sort)
                .then_with(|| left.nested_source_id.cmp(&right.nested_source_id))
                .then_with(|| left_index.cmp(right_index)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left_index.cmp(right_index),
        }
    });
    for (order, (_, candidate)) in ordered.into_iter().enumerate() {
        let payload = occurrence_payload_from_source(candidate);
        insert_reverse_observation(
            &mut inventory.reverse_expected,
            reverse_observation(
                &record_key,
                format!("item:{}", candidate.nested_source_id),
                "$.items[]._id".to_string(),
                "$.items[]._id".to_string(),
                "canonical::CreatureEntityOccurrence",
                payload,
                order,
            ),
        );
    }
    let Some(embedded) = conversion.embedded.as_value() else {
        return;
    };
    for occurrence in &embedded.occurrences {
        let nested_id = occurrence
            .source_identity
            .nested_source_id
            .as_value()
            .map(|id| id.as_str().to_string())
            .unwrap_or_else(|| occurrence.id.as_str().to_string());
        insert_actual_observation(
            inventory,
            reverse_observation(
                &record_key,
                format!("item:{nested_id}"),
                "$.items[]._id".to_string(),
                "$.items[]._id".to_string(),
                "canonical::CreatureEntityOccurrence",
                occurrence_payload(occurrence),
                occurrence.authored_order as usize,
            ),
        );
    }
}

fn occurrence_payload_from_source(
    candidate: &crate::source::npc_entities::NpcEmbeddedCandidate,
) -> CanonicalObservationPayload {
    let family = crate::source::npc_entities::family(&candidate.source).as_str();
    CanonicalObservationPayload::value(serde_json::json!({
        "nested_source_id": candidate.nested_source_id,
        "family": family,
        "source_sort": source_presence_json(&candidate.sort),
    }))
}

fn occurrence_payload(
    occurrence: &atlas_record::CreatureEntityOccurrence,
) -> CanonicalObservationPayload {
    CanonicalObservationPayload::value(serde_json::json!({
        "nested_source_id": occurrence.source_identity.nested_source_id.as_value().map(|id| id.as_str()),
        "family": occurrence.family.as_str(),
        "source_sort": fact_value_json(&occurrence.source_sort),
    }))
}

fn source_presence_json<T: Serialize>(value: &SourcePresence<T>) -> Value {
    match value {
        SourcePresence::Missing => serde_json::json!({"state": "missing"}),
        SourcePresence::Null => serde_json::json!({"state": "null"}),
        SourcePresence::Value(value) => serde_json::json!({"state": "value", "value": value}),
    }
}

fn fact_value_json<T: Serialize>(value: &FactValue<T>) -> Value {
    match value {
        FactValue::Missing => serde_json::json!({"state": "missing"}),
        FactValue::Null => serde_json::json!({"state": "null"}),
        FactValue::Value(value) => serde_json::json!({"state": "value", "value": value}),
    }
}

fn collect_reverse_relationships(
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(embedded) = conversion.embedded.as_value() else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let typed_owners = inventory.reverse_observed_owners.clone();
    let mut ordinals = BTreeMap::<(String, String), usize>::new();
    for relationship in &embedded.relationships {
        let Some(source) = embedded
            .occurrences
            .iter()
            .find(|occurrence| occurrence.id == relationship.source)
        else {
            continue;
        };
        let Some(source_id) = source.source_identity.nested_source_id.as_value() else {
            continue;
        };
        let member_identity = format!("item:{}", source_id.as_str());
        let full_path = format!(
            "$.items[]{}",
            relationship
                .source_path
                .strip_prefix('$')
                .unwrap_or(&relationship.source_path)
        );
        let normalized_path = normalize_observed_source_path(&full_path);
        if typed_owners.contains(&(
            record_key.clone(),
            member_identity.clone(),
            normalized_path.clone(),
        )) {
            continue;
        }
        let ordinal = ordinals
            .entry((member_identity.clone(), normalized_path.clone()))
            .or_default();
        let order = *ordinal;
        *ordinal += 1;
        let Some(target_id) = relationship_target_source_id(relationship, embedded) else {
            continue;
        };
        insert_actual_observation(
            inventory,
            reverse_observation(
                &record_key,
                member_identity,
                normalize_diagnostic_path(&full_path),
                normalized_path,
                "canonical::CreatureEntityRelationship::target",
                CanonicalObservationPayload::value(Value::String(target_id)),
                order,
            ),
        );
    }
}

fn relationship_target_source_id(
    relationship: &atlas_record::CreatureEntityRelationship,
    embedded: &atlas_record::CreatureEmbeddedEntities,
) -> Option<String> {
    match &relationship.target {
        atlas_record::CreatureRelationshipTarget::UnresolvedNestedSourceId(id) => {
            Some(id.as_str().to_string())
        }
        atlas_record::CreatureRelationshipTarget::Occurrence(id) => embedded
            .occurrences
            .iter()
            .find(|occurrence| occurrence.id == *id)?
            .source_identity
            .nested_source_id
            .as_value()
            .map(|id| id.as_str().to_string()),
    }
}

fn collect_reverse_content(loaded: &LoadedSourceRecord, inventory: &mut CreatureSurvivalInventory) {
    let record_key = loaded.record.identity.key.to_string();
    let mut ordinals = BTreeMap::<(String, String), usize>::new();
    for content in &loaded.facts.source_facts.content_sources {
        let (member_identity, contextual_source_path, normalized_path) =
            if let Some(rest) = content.relative_source_path.strip_prefix("$.items[_id=") {
                let Some((source_id, tail)) = rest.split_once(']') else {
                    continue;
                };
                let normalized_path = normalize_observed_source_path(&format!("$.items[]{tail}"));
                (
                    format!("item:{source_id}"),
                    normalize_diagnostic_path(&format!("$.items[]{tail}")),
                    normalized_path,
                )
            } else {
                (
                    format!("record:{}", loaded.record.identity.key),
                    normalize_diagnostic_path(&content.relative_source_path),
                    normalize_observed_source_path(&content.relative_source_path),
                )
            };
        let ordinal = ordinals
            .entry((member_identity.clone(), normalized_path.clone()))
            .or_default();
        let order = *ordinal;
        *ordinal += 1;
        insert_actual_observation(
            inventory,
            reverse_observation(
                &record_key,
                member_identity,
                contextual_source_path,
                normalized_path,
                "canonical::SourceContentFact::document",
                CanonicalObservationPayload::value(Value::String(format!(
                    "{:?}",
                    content.document
                ))),
                order,
            ),
        );
    }
}

fn collect_reverse_unsupported(
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    inventory: &mut CreatureSurvivalInventory,
) {
    let Some(embedded) = conversion.embedded.as_value() else {
        return;
    };
    let record_key = loaded.record.identity.key.to_string();
    let mut leaves = Vec::new();
    if let Some(actor) = embedded.actor_spellcasting.as_value() {
        for note in &actor.unsupported_notes {
            collect_reverse_note_leaves(
                &note.source_path,
                note,
                format!("record:{}", loaded.record.identity.key),
                &mut leaves,
            );
        }
    }
    for occurrence in &embedded.occurrences {
        let member_identity = occurrence
            .source_identity
            .nested_source_id
            .as_value()
            .map(|id| format!("item:{}", id.as_str()))
            .unwrap_or_else(|| format!("occurrence:{}", occurrence.id.as_str()));
        for note in capability_notes(&occurrence.capability) {
            let full_path = if note.source_path.starts_with("$.items[") {
                note.source_path.clone()
            } else {
                format!(
                    "$.items[]{}",
                    note.source_path
                        .strip_prefix('$')
                        .unwrap_or(&note.source_path)
                )
            };
            collect_reverse_note_leaves(&full_path, note, member_identity.clone(), &mut leaves);
        }
    }
    let typed_owners = inventory.reverse_observed_owners.clone();
    let mut ordinals = BTreeMap::<(String, String), usize>::new();
    for (member_identity, contextual_source_path, normalized_path, payload) in leaves {
        if typed_owners.contains(&(
            record_key.clone(),
            member_identity.clone(),
            normalized_path.clone(),
        )) {
            continue;
        }
        let ordinal = ordinals
            .entry((member_identity.clone(), normalized_path.clone()))
            .or_default();
        let order = *ordinal;
        *ordinal += 1;
        insert_actual_observation(
            inventory,
            reverse_observation(
                &record_key,
                member_identity,
                normalize_diagnostic_path(&contextual_source_path),
                normalized_path,
                "canonical::CreatureCapability::unsupported_notes",
                payload,
                order,
            ),
        );
    }
}

fn collect_reverse_note_leaves(
    source_path: &str,
    note: &atlas_record::UnsupportedMechanicNote,
    member_identity: String,
    leaves: &mut Vec<(String, String, String, CanonicalObservationPayload)>,
) {
    let Some(value) = unsupported_value_json(&note.value) else {
        return;
    };
    collect_reverse_value_leaves(
        source_path,
        &normalize_observed_source_path(source_path),
        &value,
        &member_identity,
        leaves,
    );
}

fn collect_reverse_value_leaves(
    contextual_source_path: &str,
    normalized_path: &str,
    value: &Value,
    member_identity: &str,
    leaves: &mut Vec<(String, String, String, CanonicalObservationPayload)>,
) {
    if is_meaningful_value(value) {
        leaves.push((
            member_identity.to_string(),
            contextual_source_path.to_string(),
            normalized_path.to_string(),
            CanonicalObservationPayload::value(value.clone()),
        ));
    }
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let segment = object_segment(normalized_path, key, map.len());
                collect_reverse_value_leaves(
                    &format!("{contextual_source_path}.{key}"),
                    &format!("{normalized_path}.{segment}"),
                    child,
                    member_identity,
                    leaves,
                );
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                collect_reverse_value_leaves(
                    &format!("{contextual_source_path}[{index}]"),
                    &format!("{normalized_path}[]"),
                    child,
                    member_identity,
                    leaves,
                );
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn reconcile_reverse_collection_closure(inventory: &mut CreatureSurvivalInventory) {
    let keys = inventory
        .reverse_expected
        .keys()
        .chain(inventory.reverse_observed.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for observation in keys {
        let expected_count = inventory
            .reverse_expected
            .get(&observation)
            .copied()
            .unwrap_or_default();
        let observed_count = inventory
            .reverse_observed
            .get(&observation)
            .copied()
            .unwrap_or_default();
        if expected_count == observed_count {
            continue;
        }
        if expected_count == 0 {
            *inventory
                .output_only_counts
                .entry(observation.normalized_path.clone())
                .or_default() += observed_count;
        }
        let mismatches = inventory
            .mismatches
            .entry(observation.normalized_path.clone())
            .or_default();
        if mismatches.len() >= SAMPLE_LIMIT {
            continue;
        }
        let expected_surplus = expected_count > observed_count;
        mismatches.push(SourcePathAuditObservationMismatch {
            normalized_path: observation.normalized_path.clone(),
            record_key: observation.record_key.clone(),
            member_identity: observation.member_identity.clone(),
            contextual_source_path: format!(
                "{}#{}",
                observation.contextual_source_path, observation.order
            ),
            destination: observation.destination.clone(),
            expected_state: if expected_surplus {
                observation.state.clone()
            } else {
                "missing".to_string()
            },
            expected_type: if expected_surplus {
                observation.value_type.clone()
            } else {
                "missing".to_string()
            },
            expected_value: if expected_surplus {
                format!(
                    "{} (multiplicity {expected_count})",
                    observation.normalized_value
                )
            } else {
                String::new()
            },
            observed_state: if expected_surplus {
                "missing".to_string()
            } else {
                observation.state.clone()
            },
            observed_type: if expected_surplus {
                "missing".to_string()
            } else {
                observation.value_type.clone()
            },
            observed_value: if expected_surplus {
                String::new()
            } else {
                format!(
                    "{} (multiplicity {observed_count})",
                    observation.normalized_value
                )
            },
            expected_multiplicity: expected_count,
            observed_multiplicity: observed_count,
            expected_order: (expected_count > 0).then_some(observation.order),
            observed_order: (observed_count > 0).then_some(observation.order),
        });
    }
}

fn record_canonical_observation(
    observed: &mut BTreeMap<String, CanonicalPathStats>,
    path: String,
    destination: &'static str,
) {
    let stats = observed.entry(path).or_default();
    stats.occurrence_count += 1;
    stats.destinations.insert(destination.to_string());
}

fn collect_expected_canonical_paths(
    indexed_path: &str,
    normalized_path: &str,
    value: &Value,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    inventory: &mut CreatureSurvivalInventory,
) {
    if is_meaningful_value(value) {
        observe_canonical_value(
            indexed_path,
            normalized_path,
            value,
            loaded,
            conversion,
            inventory,
        );
    }
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let segment = object_segment(normalized_path, key, map.len());
                collect_expected_canonical_paths(
                    &format!("{indexed_path}.{key}"),
                    &format!("{normalized_path}.{segment}"),
                    child,
                    loaded,
                    conversion,
                    inventory,
                );
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                collect_expected_canonical_paths(
                    &format!("{indexed_path}[{index}]"),
                    &format!("{normalized_path}[]"),
                    child,
                    loaded,
                    conversion,
                    inventory,
                );
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalObservationPayload {
    state: &'static str,
    value_type: String,
    normalized_value: String,
}

impl CanonicalObservationPayload {
    fn value(value: Value) -> Self {
        Self {
            state: "value",
            value_type: value_type(&value).to_string(),
            normalized_value: canonical_json(&value),
        }
    }

    fn missing() -> Self {
        Self {
            state: "missing",
            value_type: "missing".to_string(),
            normalized_value: String::new(),
        }
    }

    fn null() -> Self {
        Self {
            state: "null",
            value_type: "null".to_string(),
            normalized_value: "null".to_string(),
        }
    }
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let sorted = map
                .iter()
                .map(|(key, value)| (key.clone(), sorted_json(value)))
                .collect::<serde_json::Map<_, _>>();
            Value::Object(sorted).to_string()
        }
        _ => value.to_string(),
    }
}

fn sorted_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), sorted_json(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(sorted_json).collect()),
        _ => value.clone(),
    }
}

fn observe_canonical_value(
    indexed_path: &str,
    normalized_path: &str,
    expected_value: &Value,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    inventory: &mut CreatureSurvivalInventory,
) {
    let owner = *inventory
        .declarations
        .entry(normalized_path.to_string())
        .or_insert_with(|| {
            declaration_for("Actor", "npc", normalized_path).and_then(|declaration| {
                (declaration.disposition == SourcePathCoverageDisposition::Consumed)
                    .then_some(declaration.owner)
            })
        });
    let Some(owner) = owner else {
        return;
    };
    let destination =
        canonical_destination(indexed_path, normalized_path, loaded, conversion, inventory)
            .unwrap_or("canonical::missing_destination");
    let member_identity = canonical_member_identity(indexed_path, loaded);
    let ordinal_key = (
        loaded.record.identity.key.to_string(),
        member_identity.clone(),
        normalized_path.to_string(),
    );
    let observation_ordinal = inventory
        .observation_ordinals
        .entry(ordinal_key)
        .or_default();
    let ordinal = *observation_ordinal;
    *observation_ordinal += 1;
    let expected = expected_observation_payload(destination, expected_value);
    if reverse_closed_destination(destination) {
        let contextual_source_path = if matches!(
            destination,
            "canonical::SourceContentFact::document"
                | "canonical::CreatureCapability::unsupported_notes"
                | "canonical::CreatureEntityRelationship::target"
        ) {
            normalize_diagnostic_path(indexed_path)
        } else {
            normalized_path.to_string()
        };
        insert_reverse_observation(
            &mut inventory.reverse_expected,
            reverse_observation(
                &loaded.record.identity.key.to_string(),
                member_identity.clone(),
                contextual_source_path,
                normalized_path.to_string(),
                destination,
                expected.clone(),
                ordinal,
            ),
        );
    }
    let observed = canonical_observed_payload(
        indexed_path,
        normalized_path,
        destination,
        ordinal,
        &expected,
        loaded,
        conversion,
    )
    .unwrap_or_else(CanonicalObservationPayload::missing);
    if expected == observed {
        record_canonical_observation(
            &mut inventory.canonical,
            normalized_path.to_string(),
            destination,
        );
        return;
    }
    let mismatches = inventory
        .mismatches
        .entry(normalized_path.to_string())
        .or_default();
    if mismatches.len() < SAMPLE_LIMIT {
        mismatches.push(SourcePathAuditObservationMismatch {
            normalized_path: normalized_path.to_string(),
            record_key: loaded.record.identity.key.to_string(),
            member_identity,
            contextual_source_path: indexed_path.to_string(),
            destination: destination.to_string(),
            expected_state: expected.state.to_string(),
            expected_type: expected.value_type,
            expected_value: expected.normalized_value,
            observed_state: observed.state.to_string(),
            observed_type: observed.value_type,
            observed_value: observed.normalized_value,
            expected_multiplicity: usize::from(expected.state != "missing"),
            observed_multiplicity: usize::from(observed.state != "missing"),
            expected_order: (expected.state != "missing").then_some(ordinal),
            observed_order: (observed.state != "missing").then_some(ordinal),
        });
    }
    let _ = owner;
}

fn expected_observation_payload(destination: &str, value: &Value) -> CanonicalObservationPayload {
    if destination == "canonical::SourceContentFact::document"
        && let Some(markup) = value.as_str()
    {
        return CanonicalObservationPayload::value(Value::String(format!(
            "{:?}",
            crate::source::normalize::parse_foundry_content(markup).document
        )));
    }
    if destination == "canonical::CreatureSpellcastingEntryCapability::slots"
        && let Some(object) = value.as_object()
    {
        let projected = ["id", "name", "expended", "prepared"]
            .into_iter()
            .filter_map(|key| {
                object
                    .get(key)
                    .map(|value| (key.to_string(), value.clone()))
            })
            .collect();
        return CanonicalObservationPayload::value(Value::Object(projected));
    }
    CanonicalObservationPayload::value(value.clone())
}

fn canonical_member_identity(indexed_path: &str, loaded: &LoadedSourceRecord) -> String {
    source_item_index(indexed_path)
        .and_then(|index| source_item_id(index, loaded))
        .map(|item_id| format!("item:{item_id}"))
        .unwrap_or_else(|| format!("record:{}", loaded.record.identity.key))
}

fn source_item_id(index: usize, loaded: &LoadedSourceRecord) -> Option<&str> {
    loaded
        .facts
        .npc_source
        .as_ref()?
        .source
        .items
        .as_value()?
        .get(index)
        .map(|item| item.source().id.as_str())
}

fn fact_payload<T>(
    fact: &FactValue<T>,
    map: impl FnOnce(&T) -> Value,
) -> CanonicalObservationPayload {
    match fact {
        FactValue::Missing => CanonicalObservationPayload::missing(),
        FactValue::Null => CanonicalObservationPayload::null(),
        FactValue::Value(value) => CanonicalObservationPayload::value(map(value)),
    }
}

fn canonical_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    destination: &str,
    observation_ordinal: usize,
    expected: &CanonicalObservationPayload,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
) -> Option<CanonicalObservationPayload> {
    if destination == "canonical::CreatureCapability::unsupported_notes" {
        return unsupported_note_payload(
            indexed_path,
            loaded,
            conversion,
            Some(observation_ordinal),
        );
    }
    if destination == "typed_dto::NpcPerceptionSource::legacy_value" {
        let value = &loaded
            .facts
            .npc_source
            .as_ref()?
            .source
            .core
            .perception
            .as_value()?
            .legacy_value;
        return Some(source_presence_payload(value, |value| {
            Value::Number((*value).into())
        }));
    }
    if destination == "typed_dto::NpcSkillSource::invalid_shadow_skill" {
        let slug = segment_after(indexed_path, "$.system.skills.")?;
        let skill = loaded
            .facts
            .npc_source
            .as_ref()?
            .source
            .core
            .skills
            .as_value()?
            .get(slug)?;
        if normalized_path.ends_with(".base") {
            return Some(source_presence_payload(&skill.base, |value| {
                Value::Number((*value).into())
            }));
        }
        if normalized_path.ends_with(".note") {
            return Some(source_presence_payload(&skill.note, |value| {
                Value::String(value.clone())
            }));
        }
    }
    match normalized_path {
        "$._id" => Some(CanonicalObservationPayload::value(Value::String(
            loaded.record.identity.key.id().to_string(),
        ))),
        "$.name" => Some(CanonicalObservationPayload::value(Value::String(
            loaded.record.identity.name.clone(),
        ))),
        "$.type" => Some(CanonicalObservationPayload::value(Value::String(
            loaded.record.foundry.record_type.as_str().to_string(),
        ))),
        "$.folder" => loaded
            .record
            .foundry
            .folder_id
            .as_ref()
            .map(|value| CanonicalObservationPayload::value(Value::String(value.clone()))),
        "$.system.details.level.value" => loaded
            .record
            .classification
            .level
            .map(|value| CanonicalObservationPayload::value(Value::Number(value.into()))),
        "$.system.traits.rarity" => loaded.record.classification.rarity.map(|value| {
            CanonicalObservationPayload::value(Value::String(value.as_str().to_string()))
        }),
        "$.system.traits.size.value" => canonical_creature(loaded).map(|creature| {
            fact_payload(&creature.size.value, |value| {
                Value::String(value.as_source().to_string())
            })
        }),
        "$.system.spellcasting.rituals.dc" => {
            capability_observed_payload(indexed_path, normalized_path, loaded, conversion)
        }
        _ if normalized_path.starts_with("$.items[]") => embedded_observed_payload(
            indexed_path,
            normalized_path,
            destination,
            expected,
            loaded,
            conversion,
        ),
        _ => core_observed_payload(indexed_path, normalized_path, loaded),
    }
}

fn source_presence_payload<T>(
    value: &SourcePresence<T>,
    map: impl FnOnce(&T) -> Value,
) -> CanonicalObservationPayload {
    match value {
        SourcePresence::Missing => CanonicalObservationPayload::missing(),
        SourcePresence::Null => CanonicalObservationPayload::null(),
        SourcePresence::Value(value) => CanonicalObservationPayload::value(map(value)),
    }
}

fn unsupported_source_payload(value: &UnsupportedSourceValue) -> CanonicalObservationPayload {
    match value.shape {
        UnsupportedSourceShape::Missing => CanonicalObservationPayload::missing(),
        UnsupportedSourceShape::Null => CanonicalObservationPayload::null(),
        UnsupportedSourceShape::String => CanonicalObservationPayload::value(
            serde_json::from_str(&value.value)
                .ok()
                .filter(Value::is_string)
                .unwrap_or_else(|| Value::String(value.value.clone())),
        ),
        UnsupportedSourceShape::Number
        | UnsupportedSourceShape::Boolean
        | UnsupportedSourceShape::Array
        | UnsupportedSourceShape::Object => CanonicalObservationPayload::value(
            serde_json::from_str(&value.value)
                .unwrap_or_else(|_| Value::String(value.value.clone())),
        ),
    }
}

fn unsupported_note_payload(
    indexed_path: &str,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    observation_ordinal: Option<usize>,
) -> Option<CanonicalObservationPayload> {
    if indexed_path.starts_with("$.items[") {
        let occurrence = source_occurrence(indexed_path, loaded, conversion)?;
        let notes = capability_notes(&occurrence.capability);
        if let Some(payload) = notes
            .iter()
            .filter_map(|note| note_payload_at_path(note, indexed_path))
            .next()
        {
            return Some(payload);
        }
        let normalized = normalize_observed_source_path(indexed_path);
        let matching = notes
            .iter()
            .filter(|note| normalize_observed_source_path(&note.source_path) == normalized)
            .collect::<Vec<_>>();
        let ordinal = observation_ordinal
            .unwrap_or_else(|| trailing_array_index(indexed_path).unwrap_or_default());
        return matching
            .get(ordinal)
            .or_else(|| matching.first())
            .map(|note| unsupported_source_payload(&note.value));
    }
    conversion
        .embedded
        .as_value()?
        .actor_spellcasting
        .as_value()?
        .unsupported_notes
        .iter()
        .find(|note| note.source_path == indexed_path)
        .map(|note| unsupported_source_payload(&note.value))
}

fn note_payload_at_path(
    note: &atlas_record::UnsupportedMechanicNote,
    indexed_path: &str,
) -> Option<CanonicalObservationPayload> {
    let root = unsupported_value_json(&note.value)?;
    if note.source_path == indexed_path {
        return Some(CanonicalObservationPayload::value(root));
    }
    let suffix = indexed_path.strip_prefix(&note.source_path)?;
    if !suffix.starts_with(['.', '[']) {
        return None;
    }
    value_at_suffix(&root, suffix).map(|value| CanonicalObservationPayload::value(value.clone()))
}

fn unsupported_value_json(value: &UnsupportedSourceValue) -> Option<Value> {
    match value.shape {
        UnsupportedSourceShape::Missing => None,
        UnsupportedSourceShape::Null => Some(Value::Null),
        UnsupportedSourceShape::String => Some(
            serde_json::from_str(&value.value)
                .ok()
                .filter(Value::is_string)
                .unwrap_or_else(|| Value::String(value.value.clone())),
        ),
        _ => serde_json::from_str(&value.value)
            .ok()
            .or_else(|| Some(Value::String(value.value.clone()))),
    }
}

fn value_at_suffix<'a>(mut value: &'a Value, mut suffix: &str) -> Option<&'a Value> {
    while !suffix.is_empty() {
        if let Some(rest) = suffix.strip_prefix('.') {
            let end = rest.find(['.', '[']).unwrap_or(rest.len());
            let key = &rest[..end];
            value = value.as_object()?.get(key)?;
            suffix = &rest[end..];
        } else if let Some(rest) = suffix.strip_prefix('[') {
            let (index, tail) = rest.split_once(']')?;
            value = value.as_array()?.get(index.parse::<usize>().ok()?)?;
            suffix = tail;
        } else {
            return None;
        }
    }
    Some(value)
}

fn embedded_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    destination: &str,
    expected: &CanonicalObservationPayload,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
) -> Option<CanonicalObservationPayload> {
    if destination == "canonical::SourceContentFact::document" {
        return content_observed_payload(indexed_path, loaded);
    }
    if destination == "canonical::CreatureCapability::unsupported_notes" {
        return unsupported_note_payload(indexed_path, loaded, conversion, None);
    }
    if destination == "canonical::CreatureEntityRelationship::target"
        || destination == "canonical::CreatureEntitySourceIdentity::source_locators"
    {
        return relationship_observed_payload(indexed_path, loaded, conversion);
    }
    let index = source_item_index(indexed_path)?;
    let source_id = source_item_id(index, loaded)?;
    let fact = loaded
        .facts
        .source_facts
        .embedded_items
        .iter()
        .find(|fact| fact.item_id == source_id)?;
    match normalized_path {
        "$.items[]._id" => Some(CanonicalObservationPayload::value(Value::String(
            fact.item_id.clone(),
        ))),
        "$.items[].name" => Some(CanonicalObservationPayload::value(Value::String(
            fact.name.clone(),
        ))),
        "$.items[].type" => Some(CanonicalObservationPayload::value(Value::String(
            fact.foundry_item_type.clone(),
        ))),
        "$.items[]._stats.compendiumSource" => fact
            .compendium_source
            .as_ref()
            .map(|value| CanonicalObservationPayload::value(Value::String(value.clone()))),
        "$.items[].system.category" if destination.contains("EmbeddedItemFact") => fact
            .system_category
            .as_ref()
            .map(|value| CanonicalObservationPayload::value(Value::String(value.clone()))),
        "$.items[].system.publication.remaster" => Some(CanonicalObservationPayload::value(
            Value::Bool(fact.publication_remaster),
        )),
        "$.items[].system.slug" => fact
            .slug
            .as_ref()
            .map(|value| CanonicalObservationPayload::value(Value::String(value.clone()))),
        "$.items[].system.traits.value[]" if destination.contains("EmbeddedItemFact") => fact
            .traits
            .iter()
            .find(|value| {
                canonical_json(&Value::String((*value).clone())) == expected.normalized_value
            })
            .map(|value| CanonicalObservationPayload::value(Value::String(value.clone()))),
        "$.items[].sort" => {
            let occurrence = source_occurrence(indexed_path, loaded, conversion)?;
            Some(fact_payload(&occurrence.source_sort, |value| {
                Value::Number((*value).into())
            }))
        }
        _ => capability_observed_payload(indexed_path, normalized_path, loaded, conversion),
    }
}

fn trailing_array_index(path: &str) -> Option<usize> {
    path.rsplit_once('[')?.1.strip_suffix(']')?.parse().ok()
}

fn relationship_observed_payload(
    indexed_path: &str,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
) -> Option<CanonicalObservationPayload> {
    let occurrence = source_occurrence(indexed_path, loaded, conversion)?;
    let relative = indexed_path
        .split_once(']')
        .map(|(_, tail)| format!("${tail}"))?;
    if relative == "$.flags.core.sourceId"
        || relative == "$.system.spell.flags.core.sourceId"
        || relative == "$.system.spell._stats.compendiumSource"
    {
        let locator = occurrence
            .source_identity
            .source_locators
            .iter()
            .find(|locator| locator.source_path == relative)?;
        return Some(CanonicalObservationPayload::value(Value::String(
            locator.locator.as_str().to_string(),
        )));
    }
    let embedded = conversion.embedded.as_value()?;
    let relationship = embedded.relationships.iter().find(|relationship| {
        relationship.source == occurrence.id && relationship.source_path == relative
    })?;
    let target_id = match &relationship.target {
        atlas_record::CreatureRelationshipTarget::UnresolvedNestedSourceId(id) => {
            id.as_str().to_string()
        }
        atlas_record::CreatureRelationshipTarget::Occurrence(id) => embedded
            .occurrences
            .iter()
            .find(|candidate| candidate.id == *id)?
            .source_identity
            .nested_source_id
            .as_value()?
            .as_str()
            .to_string(),
    };
    Some(CanonicalObservationPayload::value(Value::String(target_id)))
}

fn array_index_after(path: &str, prefix: &str) -> Option<usize> {
    path.strip_prefix(prefix)?.split_once(']')?.0.parse().ok()
}

fn segment_after<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    path.strip_prefix(prefix)?.split(['.', '[']).next()
}

fn core_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    loaded: &LoadedSourceRecord,
) -> Option<CanonicalObservationPayload> {
    let creature = canonical_creature(loaded)?;
    if normalized_path == "$.system.traits.value[]" {
        let index = trailing_array_index(indexed_path)?;
        return loaded
            .record
            .classification
            .traits
            .get(index)
            .map(|value| CanonicalObservationPayload::value(Value::String(value.clone())));
    }
    if normalized_path.starts_with("$.system.abilities.") {
        let ability = segment_after(indexed_path, "$.system.abilities.")?;
        if normalized_path.ends_with(".mod") {
            let key = format!("ability.{ability}.mod");
            let metric = loaded
                .record
                .mechanics
                .metrics
                .iter()
                .find(|metric| metric.key == key)?;
            return match &metric.value {
                atlas_record::MetricValue::Number(value) if value.fract() == 0.0 => Some(
                    CanonicalObservationPayload::value(Value::Number((*value as i64).into())),
                ),
                atlas_record::MetricValue::Number(value) => serde_json::Number::from_f64(*value)
                    .map(Value::Number)
                    .map(CanonicalObservationPayload::value),
                atlas_record::MetricValue::Text(value) => Some(CanonicalObservationPayload::value(
                    Value::String(value.clone()),
                )),
                atlas_record::MetricValue::Boolean(value) => {
                    Some(CanonicalObservationPayload::value(Value::Bool(*value)))
                }
            };
        }
        let abilities = creature.legacy_abilities.value.as_value()?;
        let fact = match ability {
            "str" => &abilities.strength,
            "dex" => &abilities.dexterity,
            "con" => &abilities.constitution,
            "int" => &abilities.intelligence,
            "wis" => &abilities.wisdom,
            "cha" => &abilities.charisma,
            _ => return None,
        };
        return Some(fact_payload(fact, |value| Value::Number((*value).into())));
    }
    if normalized_path.starts_with("$.system.details.publication.") {
        let publication = creature.publication.value.as_value()?;
        return Some(match normalized_path {
            "$.system.details.publication.title" => {
                fact_payload(&publication.title, |value| Value::String(value.clone()))
            }
            "$.system.details.publication.remaster" => {
                fact_payload(&publication.remaster, |value| Value::Bool(*value))
            }
            "$.system.details.publication.license" => fact_payload(&publication.license, |value| {
                Value::String(value.as_str().to_string())
            }),
            _ => return None,
        });
    }
    if normalized_path.starts_with("$.system.details.languages.") {
        let languages = creature.languages.value.as_value()?;
        return Some(match normalized_path {
            "$.system.details.languages.details" => fact_payload(&languages.details, |value| {
                Value::String(value.as_str().to_string())
            }),
            "$.system.details.languages.value[]" => {
                let index = trailing_array_index(indexed_path)?;
                let values = languages.values.as_value()?;
                CanonicalObservationPayload::value(Value::String(
                    values.get(index)?.as_str().to_string(),
                ))
            }
            _ => return None,
        });
    }
    if normalized_path == "$.system.details.alliance" {
        return Some(match &creature.source_alliance.value {
            FactValue::Missing => CanonicalObservationPayload::missing(),
            FactValue::Null => CanonicalObservationPayload::null(),
            FactValue::Value(atlas_record::CreatureSourceAlliance::Named(value)) => {
                CanonicalObservationPayload::value(Value::String(value.as_str().to_string()))
            }
            FactValue::Value(atlas_record::CreatureSourceAlliance::Unsupported(value)) => {
                unsupported_source_payload(value)
            }
        });
    }
    if normalized_path.starts_with("$.system.perception.") {
        let perception = creature.perception.value.as_value()?;
        return perception_payload(indexed_path, normalized_path, perception);
    }
    if normalized_path == "$.system.initiative.statistic" {
        let initiative = creature.initiative.value.as_value()?;
        return Some(match &initiative.statistic {
            FactValue::Missing => CanonicalObservationPayload::missing(),
            FactValue::Null => CanonicalObservationPayload::null(),
            FactValue::Value(atlas_record::CreatureInitiativeStatistic::Named(value)) => {
                CanonicalObservationPayload::value(Value::String(value.as_str().to_string()))
            }
            FactValue::Value(atlas_record::CreatureInitiativeStatistic::Unsupported(value)) => {
                unsupported_source_payload(value)
            }
        });
    }
    if normalized_path == "$.system.attributes.adjustment" {
        return Some(match &creature.adjustment.value {
            FactValue::Missing => CanonicalObservationPayload::missing(),
            FactValue::Null => CanonicalObservationPayload::null(),
            FactValue::Value(atlas_record::CreatureAdjustment::Elite) => {
                CanonicalObservationPayload::value(Value::String("elite".to_string()))
            }
            FactValue::Value(atlas_record::CreatureAdjustment::Weak) => {
                CanonicalObservationPayload::value(Value::String("weak".to_string()))
            }
            FactValue::Value(atlas_record::CreatureAdjustment::Unsupported(value)) => {
                unsupported_source_payload(value)
            }
        });
    }
    if normalized_path.starts_with("$.system.attributes.")
        || normalized_path.starts_with("$.system.saves.")
    {
        return defenses_observed_payload(indexed_path, normalized_path, creature);
    }
    if normalized_path.starts_with("$.system.skills.") {
        return skills_observed_payload(indexed_path, normalized_path, creature);
    }
    if normalized_path.starts_with("$.system.resources.") {
        return resources_observed_payload(indexed_path, normalized_path, creature);
    }
    if normalized_path.starts_with("$.system.details.")
        || normalized_path.starts_with("$.system.description.")
    {
        return content_observed_payload(indexed_path, loaded);
    }
    None
}

fn perception_payload(
    indexed_path: &str,
    normalized_path: &str,
    perception: &atlas_record::CreaturePerception,
) -> Option<CanonicalObservationPayload> {
    Some(match normalized_path {
        "$.system.perception.mod" | "$.system.perception.value" => {
            fact_payload(&perception.modifier, |value| Value::Number((*value).into()))
        }
        "$.system.perception.details" => fact_payload(&perception.details, |value| {
            Value::String(value.as_str().to_string())
        }),
        "$.system.perception.vision" => {
            fact_payload(&perception.has_vision, |value| Value::Bool(*value))
        }
        path if path.starts_with("$.system.perception.senses[]") => {
            let index = array_index_after(indexed_path, "$.system.perception.senses[")?;
            let sense = perception.senses.as_value()?.get(index)?;
            match path {
                "$.system.perception.senses[].type" => CanonicalObservationPayload::value(
                    Value::String(sense.sense_type.as_str().to_string()),
                ),
                "$.system.perception.senses[].range" => {
                    fact_payload(&sense.range, |value| Value::Number((*value).into()))
                }
                "$.system.perception.senses[].acuity" => match &sense.acuity {
                    FactValue::Missing => CanonicalObservationPayload::missing(),
                    FactValue::Null => CanonicalObservationPayload::null(),
                    FactValue::Value(atlas_record::SenseAcuity::Precise) => {
                        CanonicalObservationPayload::value(Value::String("precise".to_string()))
                    }
                    FactValue::Value(atlas_record::SenseAcuity::Imprecise) => {
                        CanonicalObservationPayload::value(Value::String("imprecise".to_string()))
                    }
                    FactValue::Value(atlas_record::SenseAcuity::Vague) => {
                        CanonicalObservationPayload::value(Value::String("vague".to_string()))
                    }
                    FactValue::Value(atlas_record::SenseAcuity::Unsupported(value)) => {
                        unsupported_source_payload(value)
                    }
                },
                _ => return None,
            }
        }
        _ => return None,
    })
}

fn content_observed_payload(
    indexed_path: &str,
    loaded: &LoadedSourceRecord,
) -> Option<CanonicalObservationPayload> {
    let canonical_path = if let Some(index) = source_item_index(indexed_path) {
        let source_id = source_item_id(index, loaded)?;
        let (_, tail) = indexed_path.split_once(']')?;
        format!("$.items[_id={source_id}]{tail}")
    } else {
        indexed_path.to_string()
    };
    loaded
        .facts
        .source_facts
        .content_sources
        .iter()
        .find(|content| content.relative_source_path == canonical_path)
        .map(|content| {
            CanonicalObservationPayload::value(Value::String(format!("{:?}", content.document)))
        })
}

fn defenses_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    creature: &atlas_record::CreatureRecord,
) -> Option<CanonicalObservationPayload> {
    let defenses = creature.defenses.value.as_value()?;
    if normalized_path.starts_with("$.system.attributes.ac.") {
        let armor = defenses.armor_class.as_value()?;
        return Some(match normalized_path {
            "$.system.attributes.ac.value" => {
                fact_payload(&armor.value, |value| Value::Number((*value).into()))
            }
            "$.system.attributes.ac.details" => fact_payload(&armor.details, |value| {
                Value::String(value.as_str().to_string())
            }),
            _ => return None,
        });
    }
    if normalized_path.starts_with("$.system.attributes.hp.") {
        let hp = defenses.hit_points.as_value()?;
        return Some(match normalized_path {
            "$.system.attributes.hp.value" => match &hp.value {
                FactValue::Missing => CanonicalObservationPayload::missing(),
                FactValue::Null => CanonicalObservationPayload::null(),
                FactValue::Value(atlas_record::CreatureNumber::Integer(value)) => {
                    CanonicalObservationPayload::value(Value::Number((*value).into()))
                }
                FactValue::Value(atlas_record::CreatureNumber::Unsupported(value)) => {
                    unsupported_source_payload(value)
                }
            },
            "$.system.attributes.hp.max" => {
                fact_payload(&hp.maximum, |value| Value::Number((*value).into()))
            }
            "$.system.attributes.hp.temp" => {
                fact_payload(&hp.temporary, |value| Value::Number((*value).into()))
            }
            "$.system.attributes.hp.tempmax" => fact_payload(&hp.temporary_maximum, |value| {
                Value::Number((*value).into())
            }),
            "$.system.attributes.hp.details" => fact_payload(&hp.details, |value| {
                Value::String(value.as_str().to_string())
            }),
            _ => return None,
        });
    }
    if normalized_path == "$.system.attributes.hardness.value" {
        return Some(fact_payload(&defenses.hardness, |value| {
            Value::Number((*value).into())
        }));
    }
    if normalized_path == "$.system.attributes.allSaves.value" {
        return Some(fact_payload(&defenses.all_saves_note, |value| {
            Value::String(value.as_str().to_string())
        }));
    }
    if normalized_path.starts_with("$.system.attributes.shield.") {
        let shield = defenses.shield.as_value()?;
        let fact = match normalized_path {
            "$.system.attributes.shield.ac" => &shield.armor_class_bonus,
            "$.system.attributes.shield.brokenThreshold" => &shield.broken_threshold,
            "$.system.attributes.shield.hardness" => &shield.hardness,
            "$.system.attributes.shield.max" => &shield.maximum_hit_points,
            "$.system.attributes.shield.value" => &shield.serialized_hit_points,
            _ => return None,
        };
        return Some(fact_payload(fact, |value| Value::Number((*value).into())));
    }
    if normalized_path.starts_with("$.system.saves.") {
        let key = segment_after(indexed_path, "$.system.saves.")?;
        let saves = defenses.saves.as_value()?;
        let save = match key {
            "fortitude" => saves.fortitude.as_value()?,
            "reflex" => saves.reflex.as_value()?,
            "will" => saves.will.as_value()?,
            _ => return None,
        };
        return Some(if normalized_path.ends_with(".value") {
            fact_payload(&save.value, |value| Value::Number((*value).into()))
        } else if normalized_path.ends_with(".saveDetail") {
            fact_payload(&save.details, |value| {
                Value::String(value.as_str().to_string())
            })
        } else {
            return None;
        });
    }
    for (prefix, fact) in [
        ("$.system.attributes.immunities", &defenses.immunities),
        ("$.system.attributes.resistances", &defenses.resistances),
        ("$.system.attributes.weaknesses", &defenses.weaknesses),
    ] {
        if normalized_path.starts_with(prefix) {
            let index = array_index_after(indexed_path, &format!("{prefix}["))?;
            let entry = fact.as_value()?.get(index)?;
            return iwr_observed_payload(indexed_path, normalized_path, entry);
        }
    }
    if normalized_path.starts_with("$.system.attributes.speed.") {
        return movement_observed_payload(indexed_path, normalized_path, creature);
    }
    None
}

fn iwr_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    entry: &atlas_record::CreatureIwr,
) -> Option<CanonicalObservationPayload> {
    Some(if normalized_path.ends_with(".type") {
        CanonicalObservationPayload::value(Value::String(entry.iwr_type.as_str().to_string()))
    } else if normalized_path.ends_with(".value") {
        fact_payload(&entry.value, |value| Value::Number((*value).into()))
    } else if normalized_path.ends_with(".exceptions[]") {
        let index = trailing_array_index(indexed_path)?;
        CanonicalObservationPayload::value(Value::String(
            entry
                .exceptions
                .as_value()?
                .get(index)?
                .as_str()
                .to_string(),
        ))
    } else if normalized_path.ends_with(".doubleVs[]") {
        let index = trailing_array_index(indexed_path)?;
        CanonicalObservationPayload::value(Value::String(
            entry.double_vs.as_value()?.get(index)?.as_str().to_string(),
        ))
    } else if normalized_path.ends_with(".applyOnce") {
        fact_payload(&entry.apply_once, |value| Value::Bool(*value))
    } else {
        return None;
    })
}

fn movement_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    creature: &atlas_record::CreatureRecord,
) -> Option<CanonicalObservationPayload> {
    let speeds = creature.movement.value.as_value()?;
    let land = speeds
        .iter()
        .find(|speed| speed.mode == atlas_record::CreatureMovementMode::Land)?;
    Some(match normalized_path {
        "$.system.attributes.speed.value" => {
            fact_payload(&land.value, |value| Value::Number((*value).into()))
        }
        "$.system.attributes.speed.details" => fact_payload(&land.details, |value| {
            Value::String(value.as_str().to_string())
        }),
        path if path.starts_with("$.system.attributes.speed.otherSpeeds[]") => {
            let index = array_index_after(indexed_path, "$.system.attributes.speed.otherSpeeds[")?;
            let speed = speeds
                .iter()
                .filter(|speed| speed.mode != atlas_record::CreatureMovementMode::Land)
                .nth(index)?;
            match path {
                "$.system.attributes.speed.otherSpeeds[].value" => {
                    fact_payload(&speed.value, |value| Value::Number((*value).into()))
                }
                "$.system.attributes.speed.otherSpeeds[].label" => {
                    fact_payload(&speed.label, |value| Value::String(value.clone()))
                }
                "$.system.attributes.speed.otherSpeeds[].type" => {
                    movement_mode_payload(&speed.mode)
                }
                _ => return None,
            }
        }
        _ => return None,
    })
}

fn movement_mode_payload(mode: &atlas_record::CreatureMovementMode) -> CanonicalObservationPayload {
    match mode {
        atlas_record::CreatureMovementMode::Land => {
            CanonicalObservationPayload::value(Value::String("land".to_string()))
        }
        atlas_record::CreatureMovementMode::Burrow => {
            CanonicalObservationPayload::value(Value::String("burrow".to_string()))
        }
        atlas_record::CreatureMovementMode::Climb => {
            CanonicalObservationPayload::value(Value::String("climb".to_string()))
        }
        atlas_record::CreatureMovementMode::Fly => {
            CanonicalObservationPayload::value(Value::String("fly".to_string()))
        }
        atlas_record::CreatureMovementMode::Swim => {
            CanonicalObservationPayload::value(Value::String("swim".to_string()))
        }
        atlas_record::CreatureMovementMode::Unsupported(value) => unsupported_source_payload(value),
    }
}

fn skills_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    creature: &atlas_record::CreatureRecord,
) -> Option<CanonicalObservationPayload> {
    let slug = segment_after(indexed_path, "$.system.skills.")?;
    let skill = creature
        .skills
        .value
        .as_value()?
        .iter()
        .find(|skill| skill.kind.source_slug() == slug)?;
    if normalized_path.contains(".special[]") {
        let variant_index =
            array_index_after(indexed_path, &format!("$.system.skills.{slug}.special["))?;
        let variant = skill.variants.as_value()?.get(variant_index)?;
        if normalized_path.ends_with(".special[].base") {
            return Some(fact_payload(&variant.modifier, |value| {
                Value::Number((*value).into())
            }));
        }
        if normalized_path.ends_with(".special[].label") {
            return Some(fact_payload(&variant.label, |value| {
                Value::String(value.clone())
            }));
        }
        let predicate_index = indexed_path
            .split(".predicate[")
            .nth(1)?
            .split_once(']')?
            .0
            .parse::<usize>()
            .ok()?;
        let predicate = variant.predicate.as_value()?.get(predicate_index)?;
        let predicate_value = predicate_source_json(predicate);
        if normalized_path.ends_with(".predicate[]") {
            return Some(CanonicalObservationPayload::value(predicate_value));
        }
        let leaf = if normalized_path.ends_with(".not") {
            predicate_value.get("not")?.clone()
        } else if normalized_path.ends_with(".or[]") {
            let index = trailing_array_index(indexed_path)?;
            predicate_value.get("or")?.as_array()?.get(index)?.clone()
        } else if normalized_path.ends_with(".gte[]") {
            let index = trailing_array_index(indexed_path)?;
            predicate_value.get("gte")?.as_array()?.get(index)?.clone()
        } else {
            return None;
        };
        return Some(CanonicalObservationPayload::value(leaf));
    }
    if normalized_path.ends_with(".base") {
        return Some(fact_payload(&skill.modifier, |value| {
            Value::Number((*value).into())
        }));
    }
    if normalized_path.ends_with(".note") {
        return Some(fact_payload(&skill.note, |value| {
            Value::String(value.as_str().to_string())
        }));
    }
    None
}

fn predicate_source_json(predicate: &atlas_record::CreaturePredicate) -> Value {
    match predicate {
        atlas_record::CreaturePredicate::Term(term) => Value::String(term.as_str().to_string()),
        atlas_record::CreaturePredicate::Not(term) => {
            serde_json::json!({"not": term.as_str()})
        }
        atlas_record::CreaturePredicate::Any(terms) => serde_json::json!({
            "or": terms.iter().map(|term| term.as_str()).collect::<Vec<_>>()
        }),
        atlas_record::CreaturePredicate::AtLeast { term, minimum } => {
            serde_json::json!({"gte": [term.as_str(), minimum]})
        }
        atlas_record::CreaturePredicate::Unsupported(value) => serde_json::from_str(&value.value)
            .unwrap_or_else(|_| Value::String(value.value.clone())),
    }
}

fn resources_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    creature: &atlas_record::CreatureRecord,
) -> Option<CanonicalObservationPayload> {
    let key = segment_after(indexed_path, "$.system.resources.")?;
    let resource = creature
        .resources
        .value
        .as_value()?
        .iter()
        .find(|resource| resource.kind.as_str() == key)?;
    if normalized_path.ends_with(".maxx") {
        let drift = resource.source_drift.as_value()?.iter().find(|fact| {
            fact.field == atlas_record::CreatureUnsupportedSourceField::ResourceMaximumDrift
        })?;
        return Some(unsupported_source_payload(&drift.value));
    }
    let amount = if normalized_path.contains(".max") {
        &resource.maximum
    } else if normalized_path.ends_with(".value") {
        &resource.serialized_value
    } else {
        return None;
    };
    Some(resource_amount_payload(amount))
}

fn resource_amount_payload(
    amount: &FactValue<atlas_record::CreatureResourceAmount>,
) -> CanonicalObservationPayload {
    match amount {
        FactValue::Missing => CanonicalObservationPayload::missing(),
        FactValue::Null => CanonicalObservationPayload::null(),
        FactValue::Value(atlas_record::CreatureResourceAmount::Integer(value)) => {
            CanonicalObservationPayload::value(Value::Number((*value).into()))
        }
        FactValue::Value(atlas_record::CreatureResourceAmount::Unsupported(value)) => {
            unsupported_source_payload(value)
        }
    }
}

fn capability_observed_payload(
    indexed_path: &str,
    normalized_path: &str,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
) -> Option<CanonicalObservationPayload> {
    if indexed_path == "$.system.spellcasting.rituals.dc" {
        let context = conversion
            .embedded
            .as_value()?
            .actor_spellcasting
            .as_value()?;
        return Some(source_scalar_i64_payload(&context.rituals_dc));
    }
    let occurrence = source_occurrence(indexed_path, loaded, conversion)?;
    let relative = indexed_path.split_once(".system.")?.1;
    match &occurrence.capability {
        CreatureCapability::Action(action) => {
            action_observed_payload(relative, normalized_path, action)
        }
        CreatureCapability::Strike(strike) => {
            strike_observed_payload(relative, indexed_path, normalized_path, strike)
        }
        CreatureCapability::SpellcastingEntry(entry) => {
            entry_observed_payload(relative, indexed_path, normalized_path, entry, occurrence)
        }
        CreatureCapability::Spell(spell) => {
            spell_observed_payload(relative, indexed_path, normalized_path, spell, occurrence)
        }
        CreatureCapability::Equipment(equipment) => {
            equipment_observed_payload(relative, indexed_path, normalized_path, equipment)
        }
        CreatureCapability::Lore(lore) => (relative == "mod.value")
            .then(|| fact_payload(&lore.modifier, |value| Value::Number((*value).into()))),
        CreatureCapability::Unsupported(unsupported) => {
            if relative == "slug" {
                Some(fact_payload(&unsupported.source_slug, |value| {
                    Value::String(value.clone())
                }))
            } else if relative == "traits.value[]" {
                let index = trailing_array_index(indexed_path)?;
                unsupported
                    .traits
                    .as_value()?
                    .get(index)
                    .map(|value| CanonicalObservationPayload::value(Value::String(value.clone())))
            } else {
                None
            }
        }
    }
}

fn action_observed_payload(
    relative: &str,
    normalized_path: &str,
    action: &atlas_record::CreatureActionCapability,
) -> Option<CanonicalObservationPayload> {
    if relative == "actionType.value" {
        return Some(action_type_payload(&action.action_cost));
    }
    if relative == "actions.value" {
        return Some(action_count_payload(&action.action_cost));
    }
    if relative.starts_with("frequency.") {
        let frequency = action.frequency.as_value()?;
        return Some(match relative {
            "frequency.max" => {
                fact_payload(&frequency.maximum, |value| Value::Number((*value).into()))
            }
            "frequency.per" => {
                fact_payload(&frequency.period, |value| Value::String(value.clone()))
            }
            "frequency.value" => fact_payload(&frequency.serialized_value, |value| {
                Value::Number((*value).into())
            }),
            _ => return None,
        });
    }
    if relative == "selfEffect.uuid" {
        return Some(fact_payload(&action.self_effect, |value| {
            Value::String(value.clone())
        }));
    }
    if relative == "selfEffect.name" {
        return Some(fact_payload(&action.self_effect_label, |value| {
            Value::String(value.clone())
        }));
    }
    if relative == "requirements" {
        return Some(fact_payload(&action.requirements, |value| {
            Value::String(value.clone())
        }));
    }
    if relative == "cost.value" {
        return Some(fact_payload(&action.cost, |value| {
            Value::String(value.clone())
        }));
    }
    if relative.starts_with("bonus.") {
        return roll_payload(&action.rolls, "check");
    }
    if relative.starts_with("dc.") {
        return roll_payload(&action.rolls, "dc");
    }
    if relative.starts_with("damage.") || relative.starts_with("damageRolls.") {
        return damage_observed_payload(relative, &action.damage);
    }
    if normalized_path == "$.items[].system.category" {
        return Some(fact_payload(&action.category, |value| {
            Value::String(value.clone())
        }));
    }
    None
}

fn action_type_payload(cost: &atlas_record::CreatureActionCost) -> CanonicalObservationPayload {
    match cost {
        atlas_record::CreatureActionCost::Passive => {
            CanonicalObservationPayload::value(Value::String("passive".to_string()))
        }
        atlas_record::CreatureActionCost::Reaction => {
            CanonicalObservationPayload::value(Value::String("reaction".to_string()))
        }
        atlas_record::CreatureActionCost::FreeAction => {
            CanonicalObservationPayload::value(Value::String("free".to_string()))
        }
        atlas_record::CreatureActionCost::Actions(_) => {
            CanonicalObservationPayload::value(Value::String("action".to_string()))
        }
        atlas_record::CreatureActionCost::Time(value) => {
            CanonicalObservationPayload::value(Value::String(value.clone()))
        }
        atlas_record::CreatureActionCost::Unsupported(value) => unsupported_source_payload(value),
    }
}

fn action_count_payload(cost: &atlas_record::CreatureActionCost) -> CanonicalObservationPayload {
    match cost {
        atlas_record::CreatureActionCost::Actions(value) => {
            CanonicalObservationPayload::value(Value::Number(i64::from(*value).into()))
        }
        atlas_record::CreatureActionCost::Unsupported(value) => unsupported_source_payload(value),
        _ => CanonicalObservationPayload::missing(),
    }
}

fn roll_payload(
    rolls: &[atlas_record::CreatureRoll],
    id: &str,
) -> Option<CanonicalObservationPayload> {
    let roll = rolls.iter().find(|roll| roll.id == id)?;
    Some(fact_payload(&roll.value, |value| {
        Value::Number((*value).into())
    }))
}

fn strike_observed_payload(
    relative: &str,
    indexed_path: &str,
    normalized_path: &str,
    strike: &atlas_record::CreatureStrikeCapability,
) -> Option<CanonicalObservationPayload> {
    if relative.starts_with("bonus.") {
        return roll_payload(&strike.rolls, "attack");
    }
    if normalized_path == "$.items[].system.attackEffects.value[]" {
        let index = trailing_array_index(indexed_path)?;
        return strike
            .attack_effects
            .as_value()?
            .get(index)
            .map(|value| CanonicalObservationPayload::value(Value::String(value.clone())));
    }
    if relative.starts_with("damageRolls.") {
        return damage_observed_payload(relative, &strike.damage);
    }
    None
}

fn damage_observed_payload(
    relative: &str,
    damage: &FactValue<Vec<atlas_record::CreatureDamage>>,
) -> Option<CanonicalObservationPayload> {
    let (rest, prefix) = if let Some(rest) = relative.strip_prefix("damageRolls.") {
        (rest, "damageRolls")
    } else {
        (relative.strip_prefix("damage.")?, "damage")
    };
    let (id, field) = rest.split_once('.')?;
    let entry = damage.as_value()?.iter().find(|entry| entry.id == id)?;
    Some(match (prefix, field) {
        ("damageRolls", "damage") | ("damage", "formula") => {
            fact_payload(&entry.formula, |value| Value::String(value.clone()))
        }
        ("damageRolls", "damageType") | ("damage", "type") => {
            fact_payload(&entry.damage_type, |value| Value::String(value.clone()))
        }
        (_, "category") => fact_payload(&entry.category, |value| Value::String(value.clone())),
        ("damage", field) if field.starts_with("kinds[") => {
            let index = field
                .strip_prefix("kinds[")?
                .strip_suffix(']')?
                .parse::<usize>()
                .ok()?;
            let kind = entry.kinds.as_value()?.get(index)?;
            match kind {
                atlas_record::CreatureDamageKind::Damage => {
                    CanonicalObservationPayload::value(Value::String("damage".to_string()))
                }
                atlas_record::CreatureDamageKind::Healing => {
                    CanonicalObservationPayload::value(Value::String("healing".to_string()))
                }
                atlas_record::CreatureDamageKind::Unsupported(value) => {
                    unsupported_source_payload(value)
                }
            }
        }
        ("damage", "applyMod") => source_scalar_bool_payload(&entry.apply_modifier),
        _ => return None,
    })
}

fn source_scalar_i64_payload(
    source: &FactValue<atlas_record::CreatureSourceScalar<i64>>,
) -> CanonicalObservationPayload {
    match source {
        FactValue::Missing => CanonicalObservationPayload::missing(),
        FactValue::Null => CanonicalObservationPayload::null(),
        FactValue::Value(atlas_record::CreatureSourceScalar::Value(value)) => {
            CanonicalObservationPayload::value(Value::Number((*value).into()))
        }
        FactValue::Value(atlas_record::CreatureSourceScalar::Unsupported(value)) => {
            unsupported_source_payload(value)
        }
    }
}

fn source_scalar_bool_payload(
    source: &FactValue<atlas_record::CreatureSourceScalar<bool>>,
) -> CanonicalObservationPayload {
    match source {
        FactValue::Missing => CanonicalObservationPayload::missing(),
        FactValue::Null => CanonicalObservationPayload::null(),
        FactValue::Value(atlas_record::CreatureSourceScalar::Value(value)) => {
            CanonicalObservationPayload::value(Value::Bool(*value))
        }
        FactValue::Value(atlas_record::CreatureSourceScalar::Unsupported(value)) => {
            unsupported_source_payload(value)
        }
    }
}

fn entry_observed_payload(
    relative: &str,
    indexed_path: &str,
    _normalized_path: &str,
    entry: &atlas_record::CreatureSpellcastingEntryCapability,
    occurrence: &atlas_record::CreatureEntityOccurrence,
) -> Option<CanonicalObservationPayload> {
    match relative {
        "prepared.value" => Some(spell_preparation_payload(&entry.preparation)),
        "tradition.value" => Some(fact_payload(&entry.tradition, |value| {
            Value::String(value.clone())
        })),
        "spelldc.value" => Some(fact_payload(&entry.attack, |value| {
            Value::Number((*value).into())
        })),
        "spelldc.dc" => Some(fact_payload(&entry.dc, |value| {
            Value::Number((*value).into())
        })),
        "autoHeightenLevel.value" => Some(fact_payload(&occurrence.context.rank, |value| {
            Value::Number((*value).into())
        })),
        _ if relative.starts_with("slots.") => {
            spell_slot_observed_payload(relative, indexed_path, entry)
        }
        _ => None,
    }
}

fn spell_preparation_payload(
    preparation: &FactValue<atlas_record::CreatureSpellPreparation>,
) -> CanonicalObservationPayload {
    match preparation {
        FactValue::Missing => CanonicalObservationPayload::missing(),
        FactValue::Null => CanonicalObservationPayload::null(),
        FactValue::Value(atlas_record::CreatureSpellPreparation::Prepared) => {
            CanonicalObservationPayload::value(Value::String("prepared".to_string()))
        }
        FactValue::Value(atlas_record::CreatureSpellPreparation::Spontaneous) => {
            CanonicalObservationPayload::value(Value::String("spontaneous".to_string()))
        }
        FactValue::Value(atlas_record::CreatureSpellPreparation::Focus) => {
            CanonicalObservationPayload::value(Value::String("focus".to_string()))
        }
        FactValue::Value(atlas_record::CreatureSpellPreparation::Innate) => {
            CanonicalObservationPayload::value(Value::String("innate".to_string()))
        }
        FactValue::Value(atlas_record::CreatureSpellPreparation::Ritual) => {
            CanonicalObservationPayload::value(Value::String("ritual".to_string()))
        }
        FactValue::Value(atlas_record::CreatureSpellPreparation::Unsupported(value)) => {
            unsupported_source_payload(value)
        }
    }
}

fn spell_slot_observed_payload(
    relative: &str,
    indexed_path: &str,
    entry: &atlas_record::CreatureSpellcastingEntryCapability,
) -> Option<CanonicalObservationPayload> {
    let slot_key = relative.strip_prefix("slots.")?.split('.').next()?;
    let rank = slot_key.strip_prefix("slot")?.parse::<i64>().ok()?;
    let slot = entry
        .slots
        .as_value()?
        .iter()
        .find(|slot| slot.rank == rank)?;
    let suffix = relative.strip_prefix(&format!("slots.{slot_key}."))?;
    match suffix {
        "max" => Some(source_scalar_i64_payload(&slot.maximum)),
        "value" => Some(source_scalar_i64_payload(&slot.serialized_value)),
        prepared if prepared.starts_with("prepared[") => {
            let index = prepared
                .strip_prefix("prepared[")?
                .split_once(']')?
                .0
                .parse::<usize>()
                .ok()?;
            let prepared = slot.prepared.as_value()?.get(index)?;
            prepared_slot_payload(prepared, suffix, indexed_path)
        }
        _ => None,
    }
}

fn prepared_slot_payload(
    prepared: &atlas_record::CreaturePreparedSpellSlot,
    suffix: &str,
    _indexed_path: &str,
) -> Option<CanonicalObservationPayload> {
    match prepared {
        atlas_record::CreaturePreparedSpellSlot::Unsupported(value) => {
            Some(unsupported_source_payload(value))
        }
        atlas_record::CreaturePreparedSpellSlot::Spell {
            id,
            name,
            expended,
            prepared,
            ..
        } => {
            let (_, tail) = suffix.split_once(']')?;
            if tail.is_empty() {
                let mut object = serde_json::Map::new();
                insert_fact_json(&mut object, "id", id, |value| {
                    Value::String(value.as_str().to_string())
                });
                insert_fact_json(&mut object, "name", name, |value| {
                    Value::String(value.clone())
                });
                insert_fact_json(&mut object, "expended", expended, |value| {
                    Value::Bool(*value)
                });
                insert_fact_json(&mut object, "prepared", prepared, |value| {
                    Value::Bool(*value)
                });
                return Some(CanonicalObservationPayload::value(Value::Object(object)));
            }
            match tail.strip_prefix('.')? {
                "id" => Some(fact_payload(id, |value| {
                    Value::String(value.as_str().to_string())
                })),
                "name" => Some(fact_payload(name, |value| Value::String(value.clone()))),
                "expended" => Some(fact_payload(expended, |value| Value::Bool(*value))),
                "prepared" => Some(fact_payload(prepared, |value| Value::Bool(*value))),
                _ => None,
            }
        }
    }
}

fn insert_fact_json<T>(
    object: &mut serde_json::Map<String, Value>,
    key: &str,
    fact: &FactValue<T>,
    map: impl FnOnce(&T) -> Value,
) {
    match fact {
        FactValue::Missing => {}
        FactValue::Null => {
            object.insert(key.to_string(), Value::Null);
        }
        FactValue::Value(value) => {
            object.insert(key.to_string(), map(value));
        }
    }
}

fn spell_observed_payload(
    relative: &str,
    indexed_path: &str,
    _normalized_path: &str,
    spell: &atlas_record::CreatureSpellCapability,
    occurrence: &atlas_record::CreatureEntityOccurrence,
) -> Option<CanonicalObservationPayload> {
    if matches!(
        relative,
        "spell._stats.compendiumSource" | "spell.flags.core.sourceId"
    ) {
        let source_path = format!("$.system.{relative}");
        let locator = occurrence
            .source_identity
            .source_locators
            .iter()
            .find(|locator| locator.source_path == source_path)?;
        return Some(CanonicalObservationPayload::value(Value::String(
            locator.locator.as_str().to_string(),
        )));
    }
    match relative {
        "level.value" => Some(fact_payload(&spell.base_rank, |value| {
            Value::Number((*value).into())
        })),
        "location.value" => Some(fact_payload(&occurrence.context.location, |value| {
            Value::String(value.clone())
        })),
        "location.heightenedLevel" => Some(fact_payload(&occurrence.context.rank, |value| {
            Value::Number((*value).into())
        })),
        "location.signature" => Some(fact_payload(&spell.signature, |value| Value::Bool(*value))),
        "location.uses.max" => occurrence
            .context
            .uses
            .as_value()
            .map(|uses| fact_payload(&uses.maximum, |value| Value::Number((*value).into()))),
        "location.uses.value" => occurrence.context.uses.as_value().map(|uses| {
            fact_payload(&uses.serialized_value, |value| {
                Value::Number((*value).into())
            })
        }),
        "requirements" => Some(fact_payload(&spell.requirements, |value| {
            Value::String(value.clone())
        })),
        "cost.value" => Some(fact_payload(&spell.cost, |value| {
            Value::String(value.clone())
        })),
        "counteraction" | "spell.system.counteraction" => {
            Some(fact_payload(&spell.counteraction, |value| {
                Value::Bool(*value)
            }))
        }
        "target.value" => Some(fact_payload(&spell.target, |value| {
            Value::String(value.clone())
        })),
        "range.value" => Some(fact_payload(&spell.range, |value| {
            Value::String(value.clone())
        })),
        "time.value" => Some(fact_payload(&spell.time, |value| {
            Value::String(value.clone())
        })),
        "ritual.primary.check" => spell.ritual.as_value().map(|ritual| {
            fact_payload(&ritual.primary_check, |value| Value::String(value.clone()))
        }),
        "ritual.secondary.casters" => spell
            .ritual
            .as_value()
            .map(|ritual| source_scalar_i64_payload(&ritual.secondary_casters)),
        "ritual.secondary.checks" => spell.ritual.as_value().map(|ritual| {
            fact_payload(&ritual.secondary_checks, |value| {
                Value::String(value.clone())
            })
        }),
        "area.type" => spell
            .area
            .as_value()
            .map(|area| fact_payload(&area.area_type, |value| Value::String(value.clone()))),
        "area.value" => spell
            .area
            .as_value()
            .map(|area| fact_payload(&area.value, |value| Value::Number((*value).into()))),
        "duration.value" => spell
            .duration
            .as_value()
            .map(|duration| fact_payload(&duration.value, |value| Value::String(value.clone()))),
        "duration.sustained" => spell
            .duration
            .as_value()
            .map(|duration| fact_payload(&duration.sustained, |value| Value::Bool(*value))),
        "defense.save.basic" => spell
            .defense
            .as_value()
            .map(|defense| fact_payload(&defense.basic, |value| Value::Bool(*value))),
        "defense.save.statistic" => spell
            .defense
            .as_value()
            .map(|defense| spell_save_payload(&defense.save)),
        _ if relative.starts_with("traits.traditions[")
            || relative.starts_with("spell.system.traits.traditions[") =>
        {
            let index = trailing_array_index(indexed_path)?;
            spell
                .traditions
                .as_value()?
                .get(index)
                .map(|value| CanonicalObservationPayload::value(Value::String(value.clone())))
        }
        _ if relative.starts_with("damage.") => damage_observed_payload(relative, &spell.damage),
        _ => None,
    }
}

fn spell_save_payload(
    save: &FactValue<atlas_record::CreatureSpellSave>,
) -> CanonicalObservationPayload {
    match save {
        FactValue::Missing => CanonicalObservationPayload::missing(),
        FactValue::Null => CanonicalObservationPayload::null(),
        FactValue::Value(atlas_record::CreatureSpellSave::Fortitude) => {
            CanonicalObservationPayload::value(Value::String("fortitude".to_string()))
        }
        FactValue::Value(atlas_record::CreatureSpellSave::Reflex) => {
            CanonicalObservationPayload::value(Value::String("reflex".to_string()))
        }
        FactValue::Value(atlas_record::CreatureSpellSave::Will) => {
            CanonicalObservationPayload::value(Value::String("will".to_string()))
        }
        FactValue::Value(atlas_record::CreatureSpellSave::Unsupported(value)) => {
            unsupported_source_payload(value)
        }
    }
}

fn equipment_observed_payload(
    relative: &str,
    _indexed_path: &str,
    _normalized_path: &str,
    equipment: &atlas_record::CreatureEquipmentCapability,
) -> Option<CanonicalObservationPayload> {
    match relative {
        "level.value" => Some(fact_payload(&equipment.level, |value| {
            Value::Number((*value).into())
        })),
        "usage.value" => Some(fact_payload(&equipment.usage, |value| {
            Value::String(value.clone())
        })),
        "quantity" => Some(fact_payload(&equipment.quantity, |value| {
            Value::Number((*value).into())
        })),
        "uses.max" => equipment
            .uses
            .as_value()
            .map(|uses| fact_payload(&uses.maximum, |value| Value::Number((*value).into()))),
        "uses.value" => equipment.uses.as_value().map(|uses| {
            fact_payload(&uses.serialized_value, |value| {
                Value::Number((*value).into())
            })
        }),
        _ => None,
    }
}

fn canonical_destination(
    indexed_path: &str,
    normalized_path: &str,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    inventory: &mut CreatureSurvivalInventory,
) -> Option<&'static str> {
    let owner = *inventory
        .declarations
        .entry(normalized_path.to_string())
        .or_insert_with(|| {
            declaration_for("Actor", "npc", normalized_path).and_then(|declaration| {
                (declaration.disposition == SourcePathCoverageDisposition::Consumed)
                    .then_some(declaration.owner)
            })
        });
    match owner? {
        "source::npc_entities::typed_capability_or_unsupported" => {
            capability_destination(indexed_path, loaded, conversion)
        }
        "source::npc_entities::identity_relationships" => {
            relationship_destination(indexed_path, conversion)
        }
        "source::npc_entities::authored_order" => {
            occurrence_destination(indexed_path, loaded, conversion, |occurrence| {
                !matches!(occurrence.source_sort, FactValue::Missing | FactValue::Null)
            })
            .then_some("canonical::CreatureEntityOccurrence::source_sort")
        }
        "source::npc_core" => core_destination(indexed_path, normalized_path, loaded),
        "source::dto + source::npc_core + atlas-record::creature_projection" => {
            core_destination(indexed_path, normalized_path, loaded)
        }
        "records::metrics::NPC_REMAINDER_DYNAMIC_SPECS" => {
            ability_metric_destination(normalized_path, loaded)
        }
        "source::normalize + records::metrics" => loaded
            .record
            .classification
            .level
            .is_some()
            .then_some("canonical::AtlasRecord::classification.level + mechanics.metrics"),
        "source::normalize::system" => (!loaded.record.classification.traits.is_empty())
            .then_some("canonical::AtlasRecord::classification.traits"),
        "source::normalize::publication" => publication_destination(normalized_path, loaded),
        "source::normalize::content_sources" => content_destination(indexed_path, loaded),
        "source::normalize::embedded_items" | "source::dto + source::normalize::embedded_items" => {
            embedded_item_destination(indexed_path, normalized_path, loaded)
        }
        "source::dto + source::normalize" => match normalized_path {
            "$._id" => Some("canonical::AtlasRecord::identity.key.id"),
            "$.name" => Some("canonical::AtlasRecord::identity.name"),
            "$.folder" if loaded.record.foundry.folder_id.is_some() => {
                Some("canonical::AtlasRecord::foundry.folder_id")
            }
            _ => None,
        },
        "source::dto + source::normalize::kind" => (loaded.record.foundry.record_type.as_str()
            == "npc")
            .then_some("canonical::AtlasRecord::foundry.record_type"),
        "source::normalize" => loaded
            .record
            .classification
            .rarity
            .is_some()
            .then_some("canonical::AtlasRecord::classification.rarity"),
        _ => None,
    }
}

fn canonical_creature(loaded: &LoadedSourceRecord) -> Option<&atlas_record::CreatureRecord> {
    match loaded.facts.canonical_body.as_ref()? {
        RecordBody::Creature(creature) => Some(creature),
    }
}

fn fact_is_value<T>(fact: &FactValue<T>) -> bool {
    matches!(fact, FactValue::Value(_))
}

fn core_destination(
    indexed_path: &str,
    normalized_path: &str,
    loaded: &LoadedSourceRecord,
) -> Option<&'static str> {
    let creature = canonical_creature(loaded)?;
    if normalized_path.starts_with("$.system.abilities.") {
        return fact_is_value(&creature.legacy_abilities.value)
            .then_some("canonical::CreatureRecord::legacy_abilities");
    }
    if normalized_path.starts_with("$.system.attributes.ac.") {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.armor_class).then_some(()))
            .map(|()| "canonical::CreatureDefenses::armor_class");
    }
    if normalized_path.starts_with("$.system.attributes.hp.") {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.hit_points).then_some(()))
            .map(|()| "canonical::CreatureDefenses::hit_points");
    }
    if normalized_path.starts_with("$.system.attributes.immunities") {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.immunities).then_some(()))
            .map(|()| "canonical::CreatureDefenses::immunities");
    }
    if normalized_path.starts_with("$.system.attributes.resistances") {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.resistances).then_some(()))
            .map(|()| "canonical::CreatureDefenses::resistances");
    }
    if normalized_path.starts_with("$.system.attributes.weaknesses") {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.weaknesses).then_some(()))
            .map(|()| "canonical::CreatureDefenses::weaknesses");
    }
    if normalized_path.starts_with("$.system.attributes.shield.") {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.shield).then_some(()))
            .map(|()| "canonical::CreatureDefenses::shield");
    }
    if normalized_path.starts_with("$.system.attributes.speed.") {
        return fact_is_value(&creature.movement.value)
            .then_some("canonical::CreatureRecord::movement");
    }
    if normalized_path == "$.system.attributes.adjustment" {
        return fact_is_value(&creature.adjustment.value)
            .then_some("canonical::CreatureRecord::adjustment");
    }
    if normalized_path == "$.system.attributes.allSaves.value" {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.all_saves_note).then_some(()))
            .map(|()| "canonical::CreatureDefenses::all_saves_note");
    }
    if normalized_path == "$.system.attributes.hardness.value" {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.hardness).then_some(()))
            .map(|()| "canonical::CreatureDefenses::hardness");
    }
    if normalized_path.starts_with("$.system.details.languages.") {
        return fact_is_value(&creature.languages.value)
            .then_some("canonical::CreatureRecord::languages");
    }
    if normalized_path == "$.system.details.alliance" {
        return fact_is_value(&creature.source_alliance.value)
            .then_some("canonical::CreatureRecord::source_alliance");
    }
    if normalized_path.starts_with("$.system.details.publication.") {
        return fact_is_value(&creature.publication.value)
            .then_some("canonical::CreatureRecord::publication");
    }
    if normalized_path.starts_with("$.system.initiative.") {
        return fact_is_value(&creature.initiative.value)
            .then_some("canonical::CreatureRecord::initiative");
    }
    if normalized_path.starts_with("$.system.perception.") {
        if normalized_path == "$.system.perception.value"
            && loaded
                .facts
                .npc_source
                .as_ref()
                .and_then(|source| source.source.core.perception.as_value())
                .is_some_and(|perception| perception.legacy_value.as_value().is_some())
        {
            return Some("typed_dto::NpcPerceptionSource::legacy_value");
        }
        return fact_is_value(&creature.perception.value)
            .then_some("canonical::CreatureRecord::perception");
    }
    if normalized_path.starts_with("$.system.resources.") {
        return fact_is_value(&creature.resources.value)
            .then_some("canonical::CreatureRecord::resources");
    }
    if normalized_path.starts_with("$.system.saves.") {
        return creature
            .defenses
            .value
            .as_value()
            .and_then(|defenses| fact_is_value(&defenses.saves).then_some(()))
            .map(|()| "canonical::CreatureDefenses::saves");
    }
    if normalized_path.starts_with("$.system.skills.") {
        let slug = indexed_path
            .strip_prefix("$.system.skills.")?
            .split('.')
            .next()?;
        if atlas_record::CreatureSkillKind::from_source_slug(slug).is_none()
            && loaded
                .facts
                .npc_source
                .as_ref()
                .and_then(|source| source.source.core.skills.as_value())
                .is_some_and(|skills| skills.contains_key(slug))
        {
            return Some("typed_dto::NpcSkillSource::invalid_shadow_skill");
        }
        return fact_is_value(&creature.skills.value)
            .then_some("canonical::CreatureRecord::skills");
    }
    if normalized_path == "$.system.traits.size.value" {
        return fact_is_value(&creature.size.value).then_some("canonical::CreatureRecord::size");
    }
    None
}

fn ability_metric_destination(
    normalized_path: &str,
    loaded: &LoadedSourceRecord,
) -> Option<&'static str> {
    let ability = normalized_path
        .strip_prefix("$.system.abilities.")?
        .split('.')
        .next()?;
    loaded
        .record
        .mechanics
        .metrics
        .iter()
        .any(|metric| metric.key == format!("ability.{ability}.mod"))
        .then_some("canonical::AtlasRecord::mechanics.metrics[ability.*.mod]")
}

fn publication_destination(
    normalized_path: &str,
    loaded: &LoadedSourceRecord,
) -> Option<&'static str> {
    let creature = canonical_creature(loaded)?;
    let publication = creature.publication.value.as_value()?;
    match normalized_path {
        "$.system.details.publication.title" if fact_is_value(&publication.title) => {
            Some("canonical::CreaturePublication::title")
        }
        "$.system.details.publication.remaster" if fact_is_value(&publication.remaster) => {
            Some("canonical::CreaturePublication::remaster")
        }
        "$.system.details.publication.license" if fact_is_value(&publication.license) => {
            Some("canonical::CreaturePublication::license")
        }
        "$.system.details.publication.authors" if loaded.record.publication.title.is_some() => {
            Some("canonical::AtlasRecord::publication")
        }
        _ => None,
    }
}

fn source_item_index(indexed_path: &str) -> Option<usize> {
    indexed_path
        .strip_prefix("$.items[")?
        .split_once(']')?
        .0
        .parse()
        .ok()
}

fn embedded_item_destination(
    indexed_path: &str,
    normalized_path: &str,
    loaded: &LoadedSourceRecord,
) -> Option<&'static str> {
    let index = source_item_index(indexed_path)?;
    let source_id = source_item_id(index, loaded)?;
    let fact = loaded
        .facts
        .source_facts
        .embedded_items
        .iter()
        .find(|fact| fact.item_id == source_id)?;
    match normalized_path {
        "$.items[]._id" if !fact.item_id.is_empty() => Some("canonical::EmbeddedItemFact::item_id"),
        "$.items[].name" if !fact.name.is_empty() => Some("canonical::EmbeddedItemFact::name"),
        "$.items[].type" if !fact.foundry_item_type.is_empty() => {
            Some("canonical::EmbeddedItemFact::foundry_item_type")
        }
        "$.items[]._stats.compendiumSource" if fact.compendium_source.is_some() => {
            Some("canonical::EmbeddedItemFact::compendium_source")
        }
        "$.items[].system.category" if fact.system_category.is_some() => {
            Some("canonical::EmbeddedItemFact::system_category")
        }
        "$.items[].system.publication.remaster" => {
            Some("canonical::EmbeddedItemFact::publication_remaster")
        }
        "$.items[].system.slug" if fact.slug.is_some() => Some("canonical::EmbeddedItemFact::slug"),
        "$.items[].system.traits.value[]" if !fact.traits.is_empty() => {
            Some("canonical::EmbeddedItemFact::traits")
        }
        _ => None,
    }
}

fn content_destination(indexed_path: &str, loaded: &LoadedSourceRecord) -> Option<&'static str> {
    let nested_id = source_item_index(indexed_path).and_then(|index| source_item_id(index, loaded));
    loaded
        .facts
        .source_facts
        .content_sources
        .iter()
        .any(|fact| nested_id.is_none_or(|id| fact.nested_source_id.as_deref() == Some(id)))
        .then_some("canonical::SourceContentFact::document")
}

fn source_occurrence<'a>(
    indexed_path: &str,
    loaded: &LoadedSourceRecord,
    conversion: &'a crate::source::npc_entities::NpcEmbeddedConversion,
) -> Option<&'a atlas_record::CreatureEntityOccurrence> {
    let index = source_item_index(indexed_path)?;
    let source_id = source_item_id(index, loaded)?;
    let duplicate_ordinal = loaded
        .facts
        .npc_source
        .as_ref()?
        .source
        .items
        .as_value()?
        .iter()
        .take(index + 1)
        .filter(|item| item.source().id == source_id)
        .count()
        .checked_sub(1)?;
    let embedded = conversion.embedded.as_value()?;
    embedded
        .occurrences
        .iter()
        .filter(|occurrence| {
            occurrence
                .source_identity
                .nested_source_id
                .as_value()
                .is_some_and(|id| id.as_str() == source_id)
        })
        .nth(duplicate_ordinal)
}

fn occurrence_destination(
    indexed_path: &str,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    predicate: impl FnOnce(&atlas_record::CreatureEntityOccurrence) -> bool,
) -> bool {
    source_occurrence(indexed_path, loaded, conversion).is_some_and(predicate)
}

fn relationship_destination(
    indexed_path: &str,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
) -> Option<&'static str> {
    let embedded = conversion.embedded.as_value()?;
    let item_relative = indexed_path
        .split_once(']')
        .map(|(_, relative)| format!("${relative}"));
    if item_relative.as_deref() == Some("$.flags.core.sourceId")
        && embedded.occurrences.iter().any(|occurrence| {
            occurrence
                .source_identity
                .source_locators
                .iter()
                .any(|locator| locator.source_path == "$.flags.core.sourceId")
        })
    {
        return Some("canonical::CreatureEntitySourceIdentity::source_locators");
    }
    embedded
        .relationships
        .iter()
        .any(|relationship| {
            item_relative
                .as_deref()
                .is_some_and(|relative| relationship.source_path == relative)
        })
        .then_some("canonical::CreatureEntityRelationship::target")
}

fn capability_destination(
    indexed_path: &str,
    loaded: &LoadedSourceRecord,
    conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
) -> Option<&'static str> {
    if indexed_path == "$.system.spellcasting.rituals.dc" {
        let embedded = conversion.embedded.as_value()?;
        return embedded
            .actor_spellcasting
            .as_value()
            .and_then(|context| fact_is_value(&context.rituals_dc).then_some(()))
            .map(|()| "canonical::CreatureActorSpellcastingContext::rituals_dc");
    }
    let occurrence = source_occurrence(indexed_path, loaded, conversion)?;
    let relative = indexed_path.split_once(".system.")?.1;
    if unsupported_note_payload(indexed_path, loaded, conversion, None).is_some() {
        return Some("canonical::CreatureCapability::unsupported_notes");
    }
    if relative.starts_with("spell.") {
        return Some("canonical::CreatureEntityOccurrence::capability");
    }
    match &occurrence.capability {
        CreatureCapability::Action(action) => {
            if relative.starts_with("actionType.") || relative.starts_with("actions.") {
                return Some("canonical::CreatureActionCapability::action_cost");
            }
            if relative.starts_with("frequency.") && fact_is_value(&action.frequency) {
                return Some("canonical::CreatureActionCapability::frequency");
            }
            if relative == "selfEffect.uuid" && fact_is_value(&action.self_effect) {
                return Some("canonical::CreatureActionCapability::self_effect");
            }
            if relative == "selfEffect.name" && fact_is_value(&action.self_effect_label) {
                return Some("canonical::CreatureActionCapability::self_effect_label");
            }
            if relative == "requirements" && fact_is_value(&action.requirements) {
                return Some("canonical::CreatureActionCapability::requirements");
            }
            if relative == "cost.value" && fact_is_value(&action.cost) {
                return Some("canonical::CreatureActionCapability::cost");
            }
            if relative.starts_with("bonus.") && action.rolls.iter().any(|roll| roll.id == "check")
            {
                return Some("canonical::CreatureActionCapability::rolls[check]");
            }
            if relative.starts_with("dc.") && action.rolls.iter().any(|roll| roll.id == "dc") {
                return Some("canonical::CreatureActionCapability::rolls[dc]");
            }
            if (relative.starts_with("damageRolls.") || relative.starts_with("damage."))
                && fact_is_value(&action.damage)
            {
                return Some("canonical::CreatureActionCapability::damage");
            }
            if relative == "category" && fact_is_value(&action.category) {
                return Some("canonical::CreatureActionCapability::category");
            }
            if relative == "traits.value[]" && fact_is_value(&action.traits) {
                return Some("canonical::CreatureActionCapability::traits");
            }
        }
        CreatureCapability::Strike(strike) => {
            if relative.starts_with("bonus.") && !strike.rolls.is_empty() {
                return Some("canonical::CreatureStrikeCapability::rolls[attack]");
            }
            if relative.starts_with("attackEffects.") && fact_is_value(&strike.attack_effects) {
                return Some("canonical::CreatureStrikeCapability::attack_effects");
            }
            if relative.starts_with("damageRolls.") && fact_is_value(&strike.damage) {
                return Some("canonical::CreatureStrikeCapability::damage");
            }
            if relative == "traits.value[]" && fact_is_value(&strike.traits) {
                return Some("canonical::CreatureStrikeCapability::traits");
            }
        }
        CreatureCapability::SpellcastingEntry(entry) => {
            if relative.starts_with("prepared.") && fact_is_value(&entry.preparation) {
                return Some("canonical::CreatureSpellcastingEntryCapability::preparation");
            }
            if relative.starts_with("tradition.") && fact_is_value(&entry.tradition) {
                return Some("canonical::CreatureSpellcastingEntryCapability::tradition");
            }
            if relative == "spelldc.value" && fact_is_value(&entry.attack) {
                return Some("canonical::CreatureSpellcastingEntryCapability::attack");
            }
            if relative == "spelldc.dc" && fact_is_value(&entry.dc) {
                return Some("canonical::CreatureSpellcastingEntryCapability::dc");
            }
            if (relative.starts_with("slots.") || relative.starts_with("spell."))
                && fact_is_value(&entry.slots)
            {
                return Some("canonical::CreatureSpellcastingEntryCapability::slots");
            }
            if relative.starts_with("autoHeightenLevel.") && fact_is_value(&occurrence.context.rank)
            {
                return Some("canonical::CreatureEntityOccurrence::context.rank");
            }
        }
        CreatureCapability::Spell(spell) => {
            if relative == "level.value" && fact_is_value(&spell.base_rank) {
                return Some("canonical::CreatureSpellCapability::base_rank");
            }
            if relative == "location.value" && fact_is_value(&occurrence.context.location) {
                return Some("canonical::CreatureEntityOccurrence::context.location");
            }
            if relative == "location.heightenedLevel" && fact_is_value(&occurrence.context.rank) {
                return Some("canonical::CreatureEntityOccurrence::context.rank");
            }
            if relative == "location.signature" && fact_is_value(&spell.signature) {
                return Some("canonical::CreatureSpellCapability::signature");
            }
            if relative == "location.uses.max" || relative == "location.uses.value" {
                return fact_is_value(&occurrence.context.uses)
                    .then_some("canonical::CreatureEntityOccurrence::context.uses");
            }
            if relative.starts_with("traits.traditions[") && fact_is_value(&spell.traditions) {
                return Some("canonical::CreatureSpellCapability::traditions");
            }
            if relative == "requirements" && fact_is_value(&spell.requirements) {
                return Some("canonical::CreatureSpellCapability::requirements");
            }
            if relative == "cost.value" && fact_is_value(&spell.cost) {
                return Some("canonical::CreatureSpellCapability::cost");
            }
            if (relative == "counteraction" || relative == "spell.system.counteraction")
                && fact_is_value(&spell.counteraction)
            {
                return Some("canonical::CreatureSpellCapability::counteraction");
            }
            if relative.starts_with("ritual.") && fact_is_value(&spell.ritual) {
                return Some("canonical::CreatureSpellCapability::ritual");
            }
            if relative == "target.value" && fact_is_value(&spell.target) {
                return Some("canonical::CreatureSpellCapability::target");
            }
            if relative.starts_with("area.") && fact_is_value(&spell.area) {
                return Some("canonical::CreatureSpellCapability::area");
            }
            if relative == "range.value" && fact_is_value(&spell.range) {
                return Some("canonical::CreatureSpellCapability::range");
            }
            if relative == "time.value" && fact_is_value(&spell.time) {
                return Some("canonical::CreatureSpellCapability::time");
            }
            if relative.starts_with("duration.") && fact_is_value(&spell.duration) {
                return Some("canonical::CreatureSpellCapability::duration");
            }
            if relative.starts_with("defense.save.") && fact_is_value(&spell.defense) {
                return Some("canonical::CreatureSpellCapability::defense");
            }
            if relative.starts_with("damage.") && fact_is_value(&spell.damage) {
                return Some("canonical::CreatureSpellCapability::damage");
            }
        }
        CreatureCapability::Equipment(equipment) => {
            if relative == "level.value" && fact_is_value(&equipment.level) {
                return Some("canonical::CreatureEquipmentCapability::level");
            }
            if relative == "usage.value" && fact_is_value(&equipment.usage) {
                return Some("canonical::CreatureEquipmentCapability::usage");
            }
            if relative == "quantity" && fact_is_value(&equipment.quantity) {
                return Some("canonical::CreatureEquipmentCapability::quantity");
            }
            if relative.starts_with("uses.") && fact_is_value(&equipment.uses) {
                return Some("canonical::CreatureEquipmentCapability::uses");
            }
            if relative == "traits.value[]" && fact_is_value(&equipment.traits) {
                return Some("canonical::CreatureEquipmentCapability::traits");
            }
        }
        CreatureCapability::Lore(lore) => {
            if relative == "mod.value" && fact_is_value(&lore.modifier) {
                return Some("canonical::CreatureLoreCapability::modifier");
            }
        }
        CreatureCapability::Unsupported(unsupported) => {
            if relative == "traits.value[]" && fact_is_value(&unsupported.traits) {
                return Some("canonical::CreatureUnsupportedCapability::traits");
            }
            if relative == "slug" && fact_is_value(&unsupported.source_slug) {
                return Some("canonical::CreatureUnsupportedCapability::source_slug");
            }
        }
    }
    None
}

fn capability_notes(capability: &CreatureCapability) -> &[atlas_record::UnsupportedMechanicNote] {
    match capability {
        CreatureCapability::Strike(value) => &value.unsupported_notes,
        CreatureCapability::Action(value) => &value.unsupported_notes,
        CreatureCapability::SpellcastingEntry(value) => &value.unsupported_notes,
        CreatureCapability::Spell(value) => &value.unsupported_notes,
        CreatureCapability::Equipment(value) => &value.unsupported_notes,
        CreatureCapability::Lore(value) => &value.unsupported_notes,
        CreatureCapability::Unsupported(value) => &value.unsupported_notes,
    }
}

fn normalize_observed_source_path(path: &str) -> String {
    let indexed = normalize_diagnostic_path(path);
    let mut normalized = "$".to_string();
    for segment in indexed.split('.').skip(1) {
        let normalized_segment = if segment.ends_with("[]") {
            segment.to_string()
        } else {
            object_segment(&normalized, segment, 0)
        };
        normalized.push('.');
        normalized.push_str(&normalized_segment);
    }
    normalized
}

fn reconcile_creature_survival(
    paths: &mut [SourcePathAuditPathReport],
    inventory: &CreatureSurvivalInventory,
) -> Vec<SourcePathAuditClosureFailure> {
    let mut failures = Vec::new();
    let mut reconciled_paths = BTreeSet::new();
    for path in paths.iter_mut().filter(|path| {
        is_creature_path(&path.document_type, &path.record_type)
            && path.disposition == SourcePathCoverageDisposition::Consumed
    }) {
        reconciled_paths.insert(path.path.clone());
        let canonical = inventory.canonical.get(&path.path);
        let preserved = canonical.map_or(0, |stats| stats.occurrence_count);
        let extractor_identity = canonical.map(|stats| {
            stats
                .destinations
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(" + ")
        });
        path.preserved_occurrence_count = Some(preserved);
        path.extractor_identity = extractor_identity;
        let mismatches = inventory
            .mismatches
            .get(&path.path)
            .cloned()
            .unwrap_or_default();
        if preserved != path.occurrence_count || !mismatches.is_empty() {
            failures.push(SourcePathAuditClosureFailure {
                document_type: path.document_type.clone(),
                record_type: path.record_type.clone(),
                path: path.path.clone(),
                source_occurrence_count: path.occurrence_count,
                preserved_occurrence_count: preserved,
                mismatches,
            });
        }
    }
    for (path, mismatches) in &inventory.mismatches {
        if reconciled_paths.contains(path) || mismatches.is_empty() {
            continue;
        }
        failures.push(SourcePathAuditClosureFailure {
            document_type: "Actor".to_string(),
            record_type: "npc".to_string(),
            path: path.clone(),
            source_occurrence_count: 0,
            preserved_occurrence_count: inventory
                .output_only_counts
                .get(path)
                .copied()
                .unwrap_or_default(),
            mismatches: mismatches.clone(),
        });
    }
    failures.sort();
    failures
}

fn validate_typed_source(
    value: &Value,
    context: &RecordContext,
    diagnostics: &mut BTreeMap<DiagnosticKey, MutableDiagnostic>,
) {
    let identity = SourceIdentity::new(&context.record_key, &context.source_path);
    let result = match (context.document_type.as_str(), context.record_type.as_str()) {
        ("Actor", "npc") => {
            parse_npc_source(pinned_source_version_metadata(), identity, value.clone()).map(|_| ())
        }
        ("Item", _) => parse_item_source(
            pinned_source_version_metadata(),
            identity,
            None,
            value.clone(),
        )
        .map(|_| ()),
        _ => return,
    };
    if let Err(diagnostic) = result {
        record_source_diagnostic(diagnostic, context, diagnostics);
    }
}

fn record_source_diagnostic(
    diagnostic: SourceDiagnostic,
    context: &RecordContext,
    diagnostics: &mut BTreeMap<DiagnosticKey, MutableDiagnostic>,
) {
    let key = DiagnosticKey {
        kind: match diagnostic.kind {
            SourceDiagnosticKind::MalformedShape => SourceCoverageDiagnosticKind::MalformedShape,
            SourceDiagnosticKind::UnknownDiscriminator => {
                SourceCoverageDiagnosticKind::UnknownDiscriminator
            }
            SourceDiagnosticKind::InvalidParentContext => {
                SourceCoverageDiagnosticKind::InvalidParentContext
            }
            SourceDiagnosticKind::UnsupportedSourceVersion => {
                SourceCoverageDiagnosticKind::UnsupportedSourceVersion
            }
        },
        document_type: context.document_type.clone(),
        record_type: context.record_type.clone(),
        json_path: normalize_diagnostic_path(diagnostic.json_path()),
        expected_shape: diagnostic.expected_shape().to_string(),
        actual_shape: diagnostic.actual_shape().to_string(),
    };
    let aggregate = diagnostics.entry(key).or_default();
    aggregate.occurrence_count += 1;
    if aggregate.examples.len() < SAMPLE_LIMIT {
        aggregate.examples.insert(SourcePathAuditSample {
            record_key: context.record_key.clone(),
            source_path: context.source_path.clone(),
            value: diagnostic.actual_shape().to_string(),
        });
    }
}

fn normalize_diagnostic_path(path: &str) -> String {
    let mut result = String::with_capacity(path.len());
    let mut chars = path.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '[' && chars.peek().is_some_and(char::is_ascii_digit) {
            while chars.next().is_some_and(|next| next != ']') {}
            result.push_str("[]");
        } else {
            result.push(character);
        }
    }
    result
}

fn unknown_path_diagnostics(paths: &[SourcePathAuditPathReport]) -> Vec<SourceCoverageDiagnostic> {
    paths
        .iter()
        .filter(|path| path.disposition == SourcePathCoverageDisposition::Unknown)
        .map(|path| SourceCoverageDiagnostic {
            kind: SourceCoverageDiagnosticKind::UnknownPath,
            document_type: path.document_type.clone(),
            record_type: path.record_type.clone(),
            json_path: path.path.clone(),
            expected_shape: "real-owner coverage declaration".to_string(),
            actual_shape: path
                .value_types
                .iter()
                .map(|value_type| value_type.kind.as_str())
                .collect::<Vec<_>>()
                .join("|"),
            occurrence_count: path.occurrence_count,
            examples: path.examples.clone(),
        })
        .collect()
}

fn summarize_paths(
    paths: &[SourcePathAuditPathReport],
    diagnostics: &[SourceCoverageDiagnostic],
) -> SourcePathAuditSummary {
    let count = |disposition| {
        paths
            .iter()
            .filter(|path| path.disposition == disposition)
            .count()
    };
    let creature_paths = paths
        .iter()
        .filter(|path| is_creature_path(&path.document_type, &path.record_type))
        .collect::<Vec<_>>();
    let creature_count = |disposition| {
        creature_paths
            .iter()
            .filter(|path| path.disposition == disposition)
            .count()
    };
    SourcePathAuditSummary {
        consumed_paths: count(SourcePathCoverageDisposition::Consumed),
        ignored_with_rationale_paths: count(SourcePathCoverageDisposition::IgnoredWithRationale),
        provenance_only_paths: count(SourcePathCoverageDisposition::ProvenanceOnly),
        deferred_paths: count(SourcePathCoverageDisposition::Deferred),
        unknown_paths: count(SourcePathCoverageDisposition::Unknown),
        generic_deferred_paths: paths
            .iter()
            .filter(|path| {
                path.disposition == SourcePathCoverageDisposition::Deferred
                    && path.recursive_match
                    && !path.complete_family_assignment
            })
            .count(),
        unowned_recursive_matches: paths
            .iter()
            .filter(|path| path.recursive_match && !path.complete_family_assignment)
            .count(),
        consumed_regressions: 0,
        creature_paths: creature_paths.len(),
        creature_consumed_paths: creature_count(SourcePathCoverageDisposition::Consumed),
        creature_provenance_only_paths: creature_count(
            SourcePathCoverageDisposition::ProvenanceOnly,
        ),
        creature_deferred_paths: creature_count(SourcePathCoverageDisposition::Deferred),
        creature_unknown_paths: creature_count(SourcePathCoverageDisposition::Unknown),
        creature_catch_all_paths: creature_paths
            .iter()
            .filter(|path| path.recursive_match || path.complete_family_assignment)
            .count(),
        creature_unowned_paths: creature_paths
            .iter()
            .filter(|path| path.owner.trim().is_empty())
            .count(),
        creature_consumed_regressions: 0,
        type_drift_diagnostics: diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind != SourceCoverageDiagnosticKind::UnknownPath)
            .count(),
        source_diff_changes: 0,
    }
}

fn is_creature_path(document_type: &str, record_type: &str) -> bool {
    document_type == "Actor" && record_type == "npc"
}

fn compare_baseline(
    baseline_path: &Path,
    current_paths: &[SourcePathAuditPathReport],
) -> Result<SourcePathAuditDiff, IngestError> {
    let serialized = fs::read_to_string(baseline_path).map_err(|error| {
        IngestError::RecordParseFailed(format!(
            "coverage baseline {} failed to read: {error}",
            baseline_path.display()
        ))
    })?;
    let root: Value = serde_json::from_str(&serialized).map_err(|error| {
        IngestError::RecordParseFailed(format!(
            "coverage baseline {} failed to parse: {error}",
            baseline_path.display()
        ))
    })?;
    let report = root.get("data").unwrap_or(&root);
    let baseline_policy_version = report
        .get("coverage_policy_version")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            IngestError::RecordParseFailed(format!(
                "coverage baseline {} is missing coverage_policy_version",
                baseline_path.display()
            ))
        })?
        .to_string();
    if baseline_policy_version != COVERAGE_POLICY_VERSION {
        return Err(IngestError::RecordParseFailed(format!(
            "coverage baseline {} uses policy {}, expected {}",
            baseline_path.display(),
            baseline_policy_version,
            COVERAGE_POLICY_VERSION
        )));
    }
    let baseline_paths = report
        .get("paths")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            IngestError::RecordParseFailed(format!(
                "coverage baseline {} is missing paths",
                baseline_path.display()
            ))
        })?;
    let baseline_path_count = report
        .get("path_count")
        .and_then(Value::as_u64)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| {
            IngestError::RecordParseFailed(format!(
                "coverage baseline {} is missing a valid path_count",
                baseline_path.display()
            ))
        })?;
    if baseline_path_count != baseline_paths.len() {
        return Err(IngestError::RecordParseFailed(format!(
            "coverage baseline {} is truncated: path_count is {} but paths contains {}; regenerate it with --limit at least {}",
            baseline_path.display(),
            baseline_path_count,
            baseline_paths.len(),
            baseline_path_count
        )));
    }
    let mut baseline = BTreeMap::new();
    for path in baseline_paths {
        let entry = baseline_entry(path, baseline_path)?;
        baseline.insert(
            (
                entry.document_type.clone(),
                entry.record_type.clone(),
                entry.path.clone(),
            ),
            entry.disposition,
        );
    }
    let current = current_paths
        .iter()
        .map(|path| {
            (
                (
                    path.document_type.clone(),
                    path.record_type.clone(),
                    path.path.clone(),
                ),
                disposition_label(path.disposition).to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let added_paths = current
        .iter()
        .filter(|(key, _)| !baseline.contains_key(*key))
        .map(
            |((document_type, record_type, path), disposition)| SourcePathAuditDiffEntry {
                document_type: document_type.clone(),
                record_type: record_type.clone(),
                path: path.clone(),
                disposition: disposition.clone(),
            },
        )
        .collect();
    let removed_paths: Vec<SourcePathAuditDiffEntry> = baseline
        .iter()
        .filter(|(key, _)| !current.contains_key(*key))
        .map(
            |((document_type, record_type, path), disposition)| SourcePathAuditDiffEntry {
                document_type: document_type.clone(),
                record_type: record_type.clone(),
                path: path.clone(),
                disposition: disposition.clone(),
            },
        )
        .collect();
    let changed_dispositions = current
        .iter()
        .filter_map(
            |((document_type, record_type, path), current_disposition)| {
                let baseline_disposition =
                    baseline.get(&(document_type.clone(), record_type.clone(), path.clone()))?;
                (baseline_disposition != current_disposition).then(|| {
                    SourcePathAuditDispositionChange {
                        document_type: document_type.clone(),
                        record_type: record_type.clone(),
                        path: path.clone(),
                        baseline_disposition: baseline_disposition.clone(),
                        current_disposition: current_disposition.clone(),
                    }
                })
            },
        )
        .collect::<Vec<_>>();
    let mut consumed_regressions = changed_dispositions
        .iter()
        .filter(|change| {
            change.baseline_disposition == "consumed" && change.current_disposition != "consumed"
        })
        .cloned()
        .collect::<Vec<_>>();
    consumed_regressions.extend(
        removed_paths
            .iter()
            .filter(|entry| entry.disposition == "consumed")
            .map(|entry| SourcePathAuditDispositionChange {
                document_type: entry.document_type.clone(),
                record_type: entry.record_type.clone(),
                path: entry.path.clone(),
                baseline_disposition: entry.disposition.clone(),
                current_disposition: "removed".to_string(),
            }),
    );
    Ok(SourcePathAuditDiff {
        baseline_policy_version,
        baseline_source_upstream_commit: report
            .get("source_upstream_commit")
            .and_then(Value::as_str)
            .map(str::to_string),
        added_paths,
        removed_paths,
        changed_dispositions,
        consumed_regressions,
    })
}

fn baseline_entry(
    value: &Value,
    baseline_path: &Path,
) -> Result<SourcePathAuditDiffEntry, IngestError> {
    let field = |name| {
        value
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                IngestError::RecordParseFailed(format!(
                    "coverage baseline {} path entry is missing {name}",
                    baseline_path.display()
                ))
            })
    };
    Ok(SourcePathAuditDiffEntry {
        document_type: field("document_type")?,
        record_type: field("record_type")?,
        path: field("path")?,
        disposition: field("disposition")?,
    })
}

fn coverage_policy_digest() -> Result<String, IngestError> {
    let policy = (
        COVERAGE_POLICY_VERSION,
        SOURCE_COVERAGE_REGISTRY_ASSIGNMENTS,
        coverage_declarations(),
        retrieval_predicate_inventory(),
    );
    let serialized = serde_json::to_vec(&policy).map_err(|error| {
        IngestError::RecordParseFailed(format!("coverage policy failed to serialize: {error}"))
    })?;
    let digest = Sha256::digest(serialized);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn path_report_order(
    left: &SourcePathAuditPathReport,
    right: &SourcePathAuditPathReport,
) -> std::cmp::Ordering {
    disposition_rank(left.disposition)
        .cmp(&disposition_rank(right.disposition))
        .then_with(|| left.document_type.cmp(&right.document_type))
        .then_with(|| left.record_type.cmp(&right.record_type))
        .then_with(|| left.path.cmp(&right.path))
}

fn diagnostic_order(
    left: &SourceCoverageDiagnostic,
    right: &SourceCoverageDiagnostic,
) -> std::cmp::Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.document_type.cmp(&right.document_type))
        .then_with(|| left.record_type.cmp(&right.record_type))
        .then_with(|| left.json_path.cmp(&right.json_path))
        .then_with(|| left.expected_shape.cmp(&right.expected_shape))
        .then_with(|| left.actual_shape.cmp(&right.actual_shape))
}

fn disposition_rank(disposition: SourcePathCoverageDisposition) -> u8 {
    match disposition {
        SourcePathCoverageDisposition::Unknown => 0,
        SourcePathCoverageDisposition::Deferred => 1,
        SourcePathCoverageDisposition::ProvenanceOnly => 2,
        SourcePathCoverageDisposition::IgnoredWithRationale => 3,
        SourcePathCoverageDisposition::Consumed => 4,
    }
}

pub fn disposition_label(disposition: SourcePathCoverageDisposition) -> &'static str {
    match disposition {
        SourcePathCoverageDisposition::Consumed => "consumed",
        SourcePathCoverageDisposition::IgnoredWithRationale => "ignored_with_rationale",
        SourcePathCoverageDisposition::ProvenanceOnly => "provenance_only",
        SourcePathCoverageDisposition::Deferred => "deferred",
        SourcePathCoverageDisposition::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn localization_audit_fixture_root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "atlas-audit-localization-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("packs/fixture-actors")).expect("actor fixture directory");
        fs::create_dir_all(root.join("packs/fixture-spells")).expect("spell fixture directory");
        fs::create_dir_all(root.join("static/lang")).expect("localization fixture directory");
        fs::write(
            root.join("module.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "packs": [
                    {"name": "fixture-actors", "label": "Fixture Actors", "type": "Actor", "path": "packs/fixture-actors"},
                    {"name": "fixture-spells", "label": "Fixture Spells", "type": "Item", "path": "packs/fixture-spells"}
                ]
            }))
            .expect("serialize fixture manifest"),
        )
        .expect("write fixture manifest");
        fs::write(
            root.join("static/lang/en.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "PF2E": {"C2R": {
                    "Record": "<p>Localized record body.</p>",
                    "Embedded": "<p>Localized embedded body.</p>",
                    "Actor": "<p>Localized actor disable.</p>",
                    "Spell": "<p>Localized spell target.</p>"
                }}
            }))
            .expect("serialize fixture localization"),
        )
        .expect("write fixture localization");
        fs::write(
            root.join("packs/fixture-actors/localized-npc.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "_id": "localized-npc",
                "name": "Localized NPC",
                "type": "npc",
                "system": {
                    "description": {"value": "@Localize[PF2E.C2R.Record]"},
                    "details": {"level": {"value": 1}},
                    "traits": {"rarity": "common", "size": {"value": "med"}, "value": ["humanoid"]}
                },
                "items": [{
                    "_id": "localized-action",
                    "name": "Localized Action",
                    "type": "action",
                    "sort": 10,
                    "system": {
                        "description": {"value": "@Localize[PF2E.C2R.Embedded]"},
                        "publication": {"remaster": false},
                        "rules": []
                    }
                }]
            }))
            .expect("serialize localized NPC fixture"),
        )
        .expect("write localized NPC fixture");
        fs::write(
            root.join("packs/fixture-actors/localized-hazard.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "_id": "localized-hazard",
                "name": "Localized Hazard",
                "type": "hazard",
                "system": {
                    "details": {"disable": "@Localize[PF2E.C2R.Actor]", "level": {"value": 1}},
                    "traits": {"rarity": "common", "size": {"value": "med"}, "value": []}
                }
            }))
            .expect("serialize localized actor fixture"),
        )
        .expect("write localized actor fixture");
        fs::write(
            root.join("packs/fixture-spells/localized-spell.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "_id": "localized-spell",
                "name": "Localized Spell",
                "type": "spell",
                "system": {
                    "description": {"value": "<p>Spell body.</p>"},
                    "level": {"value": 1},
                    "target": {"value": "@Localize[PF2E.C2R.Spell]"},
                    "traits": {"rarity": "common", "traditions": ["arcane"], "value": []}
                }
            }))
            .expect("serialize localized spell fixture"),
        )
        .expect("write localized spell fixture");
        root
    }

    #[test]
    fn path_normalization_wildcards_dynamic_keys() {
        assert_eq!(
            object_segment("$.items[].system.damageRolls", "abc123xyz789", 2),
            "*"
        );
        assert_eq!(object_segment("$.system.skills", "acrobatics", 2), "*");
        assert_eq!(
            object_segment("$.system.attributes.ac", "value", 2),
            "value"
        );
    }

    #[test]
    fn meaningful_values_keep_zero_and_false_but_filter_scaffolding() {
        assert!(is_meaningful_value(&Value::Bool(false)));
        assert!(is_meaningful_value(&serde_json::json!(0)));
        assert!(!is_meaningful_value(&Value::Null));
        assert!(!is_meaningful_value(&Value::String("  ".to_string())));
        assert!(!is_meaningful_value(&serde_json::json!([])));
        assert!(!is_meaningful_value(&serde_json::json!({})));
    }

    #[test]
    fn diagnostic_paths_replace_array_indices() {
        assert_eq!(
            normalize_diagnostic_path("$.items[12].system"),
            "$.items[].system"
        );
    }

    #[test]
    fn policy_digest_is_stable_for_same_declarations() -> Result<(), IngestError> {
        assert_eq!(coverage_policy_digest()?, coverage_policy_digest()?);
        Ok(())
    }

    fn collect_inventory_after_actual_output_erasure(
        source_root: &Path,
    ) -> Result<CreatureSurvivalInventory, IngestError> {
        let parsed = parse_manifest(&default_manifest_path(source_root))?;
        let mut inventory = CreatureSurvivalInventory::default();
        for manifest_pack in parsed.manifest.packs {
            if manifest_pack.document_type != "Actor" {
                continue;
            }
            let pack_name = PackName::new(manifest_pack.name.clone()).map_err(|error| {
                IngestError::ManifestParseFailed(format!("invalid pack name: {error}"))
            })?;
            let resolved = resolve_pack_path(source_root, &manifest_pack);
            if !resolved.is_dir() {
                continue;
            }
            for source_file in json_files(&resolved)? {
                let value = read_json_value(&source_file)?;
                if value.get("type").and_then(Value::as_str) != Some("npc") {
                    continue;
                }
                let mut loaded = normalize_record(
                    &manifest_pack,
                    &pack_name,
                    &source_file,
                    source_root,
                    value.clone(),
                    None,
                )?;
                let candidates = collect_npc_embedded_candidates(
                    loaded.facts.npc_source.as_ref().expect("NPC typed DTO"),
                );
                let mut conversion = convert_npc_embedded_entities(
                    loaded.record.identity.key.clone(),
                    &candidates,
                    |_| None,
                );

                // These are the real post-conversion owners. No audit map is altered.
                loaded.record.identity.key = atlas_domain::RecordKey::new(
                    pack_name.clone(),
                    atlas_domain::RecordId::new("canonical-output-erased".to_string())
                        .expect("mutation id"),
                );
                loaded.record.identity.name = "canonical output erased".to_string();
                loaded.record.classification.level = None;
                loaded.record.classification.rarity = None;
                loaded.record.classification.traits.clear();
                loaded.record.foundry.folder_id = None;
                loaded.record.foundry.record_type =
                    atlas_record::FoundryRecordType::Other("erased".to_string());
                loaded.record.mechanics.metrics.clear();
                loaded.facts.source_facts = Default::default();
                loaded.facts.canonical_body = None;
                loaded.facts.npc_source = None;
                conversion.embedded = FactValue::Missing;

                collect_expected_canonical_paths(
                    "$",
                    "$",
                    &value,
                    &loaded,
                    &conversion,
                    &mut inventory,
                );
            }
        }
        Ok(inventory)
    }

    fn mutation_fixture() -> (
        Value,
        LoadedSourceRecord,
        crate::source::npc_entities::NpcEmbeddedConversion,
    ) {
        let value = serde_json::json!({
            "_id": "mutation-fixture",
            "name": "Mutation Fixture",
            "type": "npc",
            "system": {
                "details": {
                    "level": {"value": 7},
                    "alliance": "party",
                    "languages": {"value": ["common"]}
                },
                "perception": {"mod": 12, "senses": [{"type": "darkvision", "acuity": "precise"}]},
                "attributes": {
                    "immunities": [{"type": "fire"}],
                    "speed": {"value": 25, "otherSpeeds": [{"type": "fly", "value": 20}]}
                },
                "resources": {"focus": {"max": 1, "value": 1}},
                "skills": {"acrobatics": {"base": 10, "special": [{"base": 12, "label": "jump", "predicate": ["airborne"]}]}},
                "traits": {
                    "rarity": "rare",
                    "size": {"value": "lg"},
                    "value": ["alpha", "beta"]
                }
            },
            "items": []
        });
        let manifest_pack = crate::source::ManifestPack {
            name: "mutation-pack".to_string(),
            label: "Mutation Pack".to_string(),
            document_type: "Actor".to_string(),
            path: "packs/mutation-pack".to_string(),
        };
        let pack_name = PackName::new(manifest_pack.name.clone()).expect("pack name");
        let loaded = normalize_record(
            &manifest_pack,
            &pack_name,
            Path::new("packs/mutation-pack/mutation-fixture.json"),
            Path::new("."),
            value.clone(),
            None,
        )
        .expect("mutation fixture normalizes");
        let candidates = collect_npc_embedded_candidates(
            loaded.facts.npc_source.as_ref().expect("NPC typed DTO"),
        );
        let conversion =
            convert_npc_embedded_entities(loaded.record.identity.key.clone(), &candidates, |_| {
                None
            });
        (value, loaded, conversion)
    }

    fn collection_mutation_fixture() -> (
        Value,
        LoadedSourceRecord,
        crate::source::npc_entities::NpcEmbeddedConversion,
    ) {
        let value = serde_json::json!({
            "_id": "collection-mutation-fixture",
            "name": "Collection Mutation Fixture",
            "type": "npc",
            "system": {
                "details": {"level": {"value": 7}},
                "traits": {"rarity": "common", "size": {"value": "med"}, "value": ["humanoid"]}
            },
            "items": [
                {
                    "_id": "grantor",
                    "name": "Grantor",
                    "type": "action",
                    "sort": 10,
                    "flags": {"pf2e": {"itemGrants": {"child": {"id": "child"}}}},
                    "system": {
                        "description": {"value": "<p>Grantor body.</p>"},
                        "publication": {"remaster": false},
                        "rules": [{"key": "FutureRule"}]
                    }
                },
                {
                    "_id": "child",
                    "name": "Child",
                    "type": "effect",
                    "sort": 20,
                    "flags": {"pf2e": {"grantedBy": {"id": "grantor"}}},
                    "system": {"publication": {"remaster": false}}
                }
            ]
        });
        let manifest_pack = crate::source::ManifestPack {
            name: "mutation-pack".to_string(),
            label: "Mutation Pack".to_string(),
            document_type: "Actor".to_string(),
            path: "packs/mutation-pack".to_string(),
        };
        let pack_name = PackName::new(manifest_pack.name.clone()).expect("pack name");
        let loaded = normalize_record(
            &manifest_pack,
            &pack_name,
            Path::new("packs/mutation-pack/collection-mutation-fixture.json"),
            Path::new("."),
            value.clone(),
            None,
        )
        .expect("collection mutation fixture normalizes");
        let candidates = collect_npc_embedded_candidates(
            loaded.facts.npc_source.as_ref().expect("NPC typed DTO"),
        );
        let conversion =
            convert_npc_embedded_entities(loaded.record.identity.key.clone(), &candidates, |_| {
                None
            });
        (value, loaded, conversion)
    }

    fn observe_mutation_fixture(
        source: &Value,
        loaded: &LoadedSourceRecord,
        conversion: &crate::source::npc_entities::NpcEmbeddedConversion,
    ) -> CreatureSurvivalInventory {
        let mut inventory = CreatureSurvivalInventory::default();
        collect_expected_canonical_paths("$", "$", source, loaded, conversion, &mut inventory);
        collect_sequence_closure(source, loaded, &mut inventory);
        collect_reverse_collection_closure(loaded, conversion, &mut inventory);
        reconcile_reverse_collection_closure(&mut inventory);
        inventory
    }

    fn assert_public_strict_failure(
        inventory: &CreatureSurvivalInventory,
        path: &str,
        occurrence_count: usize,
    ) {
        let mut stats = MutablePathStats {
            occurrence_count,
            ..MutablePathStats::default()
        };
        stats
            .record_keys
            .insert("mutation-pack:mutation-fixture".to_string());
        let key = PathKey {
            document_type: "Actor".to_string(),
            record_type: "npc".to_string(),
            path: path.to_string(),
        };
        let mut paths = vec![path_report(key, stats, 1)];
        if occurrence_count == 0 {
            assert_eq!(paths[0].disposition, SourcePathCoverageDisposition::Unknown);
        } else {
            assert_eq!(
                paths[0].disposition,
                SourcePathCoverageDisposition::Consumed
            );
        }
        let failures = reconcile_creature_survival(&mut paths, inventory);
        assert!(
            !failures.is_empty(),
            "{path} must reach public closure_failures"
        );
        let failure = failures
            .iter()
            .find(|failure| failure.path == path)
            .unwrap_or_else(|| panic!("{path} must be present on the public failure surface"));
        assert!(!failure.mismatches.is_empty(), "{path} mismatch evidence");
        let enforcement = source_path_enforcement(true, failures.len());
        assert!(!enforcement.passed, "{path} strict result must fail");
        assert!(
            enforcement.violation_count > 0,
            "{path} nonzero strict result"
        );
    }

    #[test]
    fn real_canonical_mutations_detect_value_type_state_membership_and_order() {
        let (source, loaded, conversion) = mutation_fixture();
        assert!(
            observe_mutation_fixture(&source, &loaded, &conversion)
                .mismatches
                .is_empty()
        );

        let mut changed_value = loaded.clone();
        changed_value.record.identity.name = "Wrong Fixture".to_string();
        let mismatch = observe_mutation_fixture(&source, &changed_value, &conversion);
        assert!(mismatch.mismatches.contains_key("$.name"));
        assert_public_strict_failure(&mismatch, "$.name", 1);

        let mut changed_type = loaded.clone();
        let RecordBody::Creature(creature) = changed_type
            .facts
            .canonical_body
            .as_mut()
            .expect("canonical creature");
        creature.source_alliance.value = FactValue::Value(
            atlas_record::CreatureSourceAlliance::Unsupported(UnsupportedSourceValue {
                shape: UnsupportedSourceShape::Boolean,
                value: "false".to_string(),
                reason: atlas_record::UnsupportedSourceReason::OpenVocabulary,
            }),
        );
        let mismatch = observe_mutation_fixture(&source, &changed_type, &conversion);
        assert!(
            mismatch
                .mismatches
                .contains_key("$.system.details.alliance")
        );
        assert_public_strict_failure(&mismatch, "$.system.details.alliance", 1);

        let mut changed_state = loaded.clone();
        changed_state.record.classification.level = None;
        let mismatch = observe_mutation_fixture(&source, &changed_state, &conversion);
        assert!(
            mismatch
                .mismatches
                .contains_key("$.system.details.level.value")
        );
        assert_public_strict_failure(&mismatch, "$.system.details.level.value", 1);

        for traits in [
            vec!["alpha".to_string()],
            vec!["beta".to_string(), "alpha".to_string()],
            vec!["alpha".to_string(), "beta".to_string(), "extra".to_string()],
        ] {
            let mut changed_members = loaded.clone();
            changed_members.record.classification.traits = traits;
            let mismatch = observe_mutation_fixture(&source, &changed_members, &conversion);
            assert!(mismatch.mismatches.contains_key("$.system.traits.value[]"));
            assert_public_strict_failure(&mismatch, "$.system.traits.value[]", 2);
        }
    }

    #[test]
    fn novel_typed_collection_members_fail_reverse_public_closure() {
        let (source, _loaded, conversion) = mutation_fixture();
        let mut surplus_source = source.clone();
        surplus_source["system"]["details"]["languages"]["value"] =
            serde_json::json!(["common", "draconic"]);
        surplus_source["system"]["perception"]["senses"] = serde_json::json!([
            {"type": "darkvision", "acuity": "precise"},
            {"type": "tremorsense", "acuity": "imprecise", "range": 30}
        ]);
        surplus_source["system"]["attributes"]["immunities"] =
            serde_json::json!([{"type": "fire"}, {"type": "cold"}]);
        surplus_source["system"]["attributes"]["speed"]["otherSpeeds"] = serde_json::json!([
            {"type": "fly", "value": 20},
            {"type": "swim", "value": 15, "label": "novel trailing speed"}
        ]);
        surplus_source["system"]["skills"]["arcana"] = serde_json::json!({
            "base": 11,
            "special": [{"base": 13, "label": "novel arcana", "predicate": ["novel:predicate"]}]
        });
        surplus_source["system"]["resources"]["hero"] = serde_json::json!({"max": 3, "value": 2});

        let manifest_pack = crate::source::ManifestPack {
            name: "mutation-pack".to_string(),
            label: "Mutation Pack".to_string(),
            document_type: "Actor".to_string(),
            path: "packs/mutation-pack".to_string(),
        };
        let pack_name = PackName::new(manifest_pack.name.clone()).expect("pack name");
        let surplus_loaded = normalize_record(
            &manifest_pack,
            &pack_name,
            Path::new("packs/mutation-pack/novel-typed-output.json"),
            Path::new("."),
            surplus_source,
            None,
        )
        .expect("novel typed output normalizes");
        let mismatch = observe_mutation_fixture(&source, &surplus_loaded, &conversion);
        for path in [
            "$.system.details.languages.value[]",
            "$.system.perception.senses[].type",
            "$.system.attributes.immunities[].type",
            "$.system.attributes.speed.otherSpeeds[].type",
            "$.system.skills.*.base",
            "$.system.skills.*.special[].predicate[]",
            "$.system.resources.*.max",
        ] {
            assert!(mismatch.mismatches.contains_key(path), "missing {path}");
            assert_public_strict_failure(&mismatch, path, 1);
        }
    }

    #[test]
    fn real_canonical_surplus_occurrence_relationship_content_and_member_fail_public_closure() {
        let (source, loaded, conversion) = collection_mutation_fixture();
        let clean = observe_mutation_fixture(&source, &loaded, &conversion);
        assert!(
            clean.mismatches.is_empty(),
            "clean reverse closure: {:#?}",
            clean.mismatches
        );

        let mut extra_occurrence = conversion.clone();
        let FactValue::Value(embedded) = &mut extra_occurrence.embedded else {
            panic!("embedded")
        };
        let mut novel_occurrence = embedded.occurrences[0].clone();
        novel_occurrence.id = atlas_record::CreatureOccurrenceId::new(
            "occurrence:novel-output-only-identity".to_string(),
        )
        .expect("novel occurrence id");
        novel_occurrence.source_identity.nested_source_id = FactValue::Value(
            atlas_record::CreatureSourceId::new("novel-output-only-item".to_string())
                .expect("novel source id"),
        );
        novel_occurrence.authored_order = 999;
        novel_occurrence.source_sort = FactValue::Value(999_999);
        embedded.occurrences.push(novel_occurrence);
        let mismatch = observe_mutation_fixture(&source, &loaded, &extra_occurrence);
        assert_public_strict_failure(&mismatch, "$.items[]._id", 2);

        let mut extra_relationship = conversion.clone();
        let FactValue::Value(embedded) = &mut extra_relationship.embedded else {
            panic!("embedded")
        };
        let mut novel_relationship = embedded.relationships[0].clone();
        novel_relationship.source_path =
            "$.flags.pf2e.itemGrants.novelClosureTarget.id".to_string();
        embedded.relationships.push(novel_relationship);
        let mismatch = observe_mutation_fixture(&source, &loaded, &extra_relationship);
        assert_public_strict_failure(&mismatch, "$.items[].flags.pf2e.itemGrants.*.id", 1);

        let mut extra_content = loaded.clone();
        let mut novel_content = extra_content.facts.source_facts.content_sources[0].clone();
        novel_content.content_key = "novel-output-only-content".to_string();
        novel_content.relative_source_path =
            "$.items[_id=grantor].system.novelClosureContent.value".to_string();
        extra_content
            .facts
            .source_facts
            .content_sources
            .push(novel_content);
        let mismatch = observe_mutation_fixture(&source, &extra_content, &conversion);
        assert_public_strict_failure(&mismatch, "$.items[].system.novelClosureContent.value", 0);

        let mut extra_member = conversion.clone();
        let FactValue::Value(embedded) = &mut extra_member.embedded else {
            panic!("embedded")
        };
        let notes = match &mut embedded.occurrences[0].capability {
            CreatureCapability::Action(value) => &mut value.unsupported_notes,
            capability => panic!("expected action, got {capability:?}"),
        };
        let mut novel_note = notes[0].clone();
        novel_note.source_path = "$.system.rules[99].novelClosureMember".to_string();
        novel_note.value = UnsupportedSourceValue {
            shape: UnsupportedSourceShape::String,
            value: "\"novel-output-only-value\"".to_string(),
            reason: atlas_record::UnsupportedSourceReason::OpenVocabulary,
        };
        notes.push(novel_note);
        let mismatch = observe_mutation_fixture(&source, &loaded, &extra_member);
        assert_public_strict_failure(&mismatch, "$.items[].system.rules[].novelClosureMember", 0);
    }

    #[test]
    fn reused_audit_matches_legacy_pre_localization_observations_exactly() {
        let source_root = localization_audit_fixture_root();
        let source = crate::source_pipeline::load_foundry_source(&source_root, None)
            .expect("localized fixture source load");

        let npc = source
            .records
            .iter()
            .find(|loaded| loaded.record.identity.key.id().as_str() == "localized-npc")
            .expect("localized NPC");
        let localized_content = npc
            .facts
            .source_facts
            .content_sources
            .iter()
            .map(|content| format!("{:?}", content.document))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(localized_content.contains("Localized record body"));
        assert!(localized_content.contains("Localized embedded body"));

        let actor = source
            .records
            .iter()
            .find(|loaded| loaded.record.identity.key.id().as_str() == "localized-hazard")
            .and_then(|loaded| loaded.record.mechanics.actor())
            .expect("localized actor mechanics");
        assert_eq!(
            actor.disable_text.as_deref(),
            Some("Localized actor disable.")
        );
        let spell = source
            .records
            .iter()
            .find(|loaded| loaded.record.identity.key.id().as_str() == "localized-spell")
            .and_then(|loaded| loaded.record.mechanics.spell())
            .expect("localized spell mechanics");
        assert_eq!(
            spell.target.as_ref().map(|target| target.text.as_str()),
            Some("Localized spell target.")
        );

        let options = SourcePathAuditOptions {
            source_root: source_root.clone(),
            min_records: 1,
            limit: Some(usize::MAX),
            strict: true,
            ..SourcePathAuditOptions::default()
        };
        let legacy = audit_source_paths(options.clone()).expect("legacy fixture audit");
        let reused = audit_loaded_source(options, &source).expect("reused fixture audit");
        assert_eq!(reused, legacy);

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn pinned_creature_audit_proves_all_consumed_occurrences_and_destinations_survive() {
        let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let report = audit_source_paths(SourcePathAuditOptions {
            source_root: PathBuf::from(source_root.clone()),
            document_type: Some("Actor".to_string()),
            record_type: Some("npc".to_string()),
            strict: true,
            limit: Some(usize::MAX),
            ..SourcePathAuditOptions::default()
        })
        .expect("pinned creature audit");

        assert_eq!(report.summary.creature_paths, 614);
        assert_eq!(report.summary.creature_consumed_paths, 607);
        assert_eq!(report.summary.creature_provenance_only_paths, 7);
        assert_eq!(report.summary.creature_deferred_paths, 0);
        assert_eq!(report.summary.creature_unknown_paths, 0);
        assert_eq!(report.summary.creature_catch_all_paths, 0);
        assert_eq!(report.summary.creature_unowned_paths, 0);
        assert_eq!(
            report.summary.creature_consumed_regressions, 0,
            "closure failures: {:#?}",
            report.closure_failures
        );
        assert!(report.closure_failures.is_empty());
        assert!(report.enforcement.passed);

        let consumed = report
            .paths
            .iter()
            .filter(|path| path.disposition == SourcePathCoverageDisposition::Consumed)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(consumed.len(), 607);
        assert_eq!(
            consumed
                .iter()
                .map(|path| path.occurrence_count)
                .sum::<usize>(),
            1_746_725
        );
        assert_eq!(
            consumed
                .iter()
                .map(|path| path.preserved_occurrence_count.unwrap_or_default())
                .sum::<usize>(),
            1_746_725
        );
        assert!(consumed.iter().all(|path| {
            path.extractor_identity.as_deref().is_some_and(|identity| {
                (identity.contains("canonical::") || identity.starts_with("typed_dto::"))
                    && !identity.contains("hydrated::")
                    && !identity.contains("serde_json")
            })
        }));

        let erased = collect_inventory_after_actual_output_erasure(&PathBuf::from(source_root))
            .expect("actual canonical output erasure audit");
        let mut mutated = consumed.clone();
        let failures = reconcile_creature_survival(&mut mutated, &erased);
        assert_eq!(
            failures.len(),
            607,
            "erasing real typed DTO and canonical owners must fail every consumed path"
        );
        assert!(
            failures
                .iter()
                .all(|failure| !failure.mismatches.is_empty())
        );

        let retained = report
            .paths
            .iter()
            .filter(|path| RETAINED_CAPABILITY_PATHS.contains(&path.path.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(retained.len(), RETAINED_CAPABILITY_PATHS.len());
        assert_eq!(
            retained
                .iter()
                .map(|path| path.occurrence_count)
                .sum::<usize>(),
            3_841
        );
        assert_eq!(
            retained
                .iter()
                .map(|path| path.preserved_occurrence_count.unwrap_or_default())
                .sum::<usize>(),
            3_841
        );
    }
}
