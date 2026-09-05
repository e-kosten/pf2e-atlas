//! Offline raw-source inventory.
//!
//! This surface is deliberately diagnostic-only. It discovers normalized raw
//! paths and examples, but cannot classify product ownership or establish
//! source-leaf completeness. Exact coverage belongs to `source_coverage`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::IngestError;
use crate::source::dto::{PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT};
use crate::source::loader::{
    default_manifest_path, json_files, parse_manifest, relative_source_path, resolve_pack_path,
};

mod predicate_inventory;

pub use predicate_inventory::RetrievalPredicateInventoryEntry;
use predicate_inventory::retrieval_predicate_inventory;

const SOURCE_PATH_INVENTORY_VERSION: &str = "pf2e-source-path-inventory/v1";
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
    pub authoritative_completeness: bool,
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
    Unconsumed,
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
    let mut stats = BTreeMap::new();
    let mut pack_count = 0;
    let mut record_count = 0;

    for pack in parsed_manifest.manifest.packs {
        if !selected(&options, &pack.name, &pack.document_type, None) {
            continue;
        }
        let resolved_path = resolve_pack_path(&source_root, &pack);
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
            if !selected(
                &options,
                &pack.name,
                &pack.document_type,
                Some(&record_type),
            ) {
                continue;
            }
            let context = RecordContext {
                document_type: pack.document_type.clone(),
                record_type,
                record_key: value
                    .get("_id")
                    .and_then(Value::as_str)
                    .map(|id| format!("{}:{id}", pack.name))
                    .unwrap_or_else(|| relative_source_path(&source_root, &source_file)),
                source_path: relative_source_path(&source_root, &source_file),
            };
            record_count += 1;
            collect_inventory_paths("$", &value, &context, &mut stats);
        }
    }
    finish_inventory(
        options,
        source_root,
        manifest_path,
        pack_count,
        record_count,
        stats,
    )
}

fn selected(
    options: &SourcePathAuditOptions,
    pack_name: &str,
    document_type: &str,
    record_type: Option<&str>,
) -> bool {
    options
        .pack_name
        .as_deref()
        .is_none_or(|expected| expected == pack_name)
        && options
            .document_type
            .as_deref()
            .is_none_or(|expected| expected == document_type)
        && record_type.is_none_or(|record_type| {
            options
                .record_type
                .as_deref()
                .is_none_or(|expected| expected == record_type)
        })
}

#[allow(clippy::too_many_arguments)]
fn finish_inventory(
    options: SourcePathAuditOptions,
    source_root: PathBuf,
    manifest_path: PathBuf,
    pack_count: usize,
    record_count: usize,
    stats: BTreeMap<PathKey, MutablePathStats>,
) -> Result<SourcePathAuditReport, IngestError> {
    let mut paths = stats
        .into_iter()
        .filter_map(|(key, stats)| {
            let record_count = stats.record_keys.len();
            (record_count >= options.min_records)
                .then(|| diagnostic_path_report(key, stats, record_count))
        })
        .collect::<Vec<_>>();
    paths.sort_by(path_report_order);
    let path_count = paths.len();
    let source_diff = options
        .baseline_report
        .as_deref()
        .map(|baseline| compare_baseline(baseline, &paths))
        .transpose()?;
    let source_diff_changes = source_diff
        .as_ref()
        .map_or(0, SourcePathAuditDiff::change_count);
    let creature_paths = paths
        .iter()
        .filter(|path| is_creature_path(&path.document_type, &path.record_type))
        .count();
    let summary = SourcePathAuditSummary {
        consumed_paths: 0,
        ignored_with_rationale_paths: 0,
        provenance_only_paths: 0,
        deferred_paths: 0,
        unknown_paths: path_count,
        generic_deferred_paths: 0,
        unowned_recursive_matches: 0,
        consumed_regressions: 0,
        creature_paths,
        creature_consumed_paths: 0,
        creature_provenance_only_paths: 0,
        creature_deferred_paths: 0,
        creature_unknown_paths: creature_paths,
        creature_catch_all_paths: 0,
        creature_unowned_paths: creature_paths,
        creature_consumed_regressions: 0,
        type_drift_diagnostics: 0,
        source_diff_changes,
    };
    let warning_count = path_count + source_diff_changes + 1;
    let enforcement = SourcePathAuditEnforcement {
        mode: if options.strict {
            SourcePathAuditMode::Strict
        } else {
            SourcePathAuditMode::Relaxed
        },
        passed: !options.strict,
        violation_count: if options.strict { warning_count } else { 0 },
        aggregate_warning_count: warning_count,
    };
    let diagnostics = paths
        .iter()
        .map(|path| SourceCoverageDiagnostic {
            kind: SourceCoverageDiagnosticKind::UnknownPath,
            document_type: path.document_type.clone(),
            record_type: path.record_type.clone(),
            json_path: path.path.clone(),
            expected_shape: "exact source-leaf declaration plus actual-read receipt".to_string(),
            actual_shape: "diagnostic inventory observation only".to_string(),
            occurrence_count: path.occurrence_count,
            examples: path.examples.clone(),
        })
        .collect();
    paths.truncate(options.limit.unwrap_or(DEFAULT_PATH_LIMIT));

    Ok(SourcePathAuditReport {
        coverage_policy_version: SOURCE_PATH_INVENTORY_VERSION,
        coverage_policy_digest: inventory_policy_digest(),
        authoritative_completeness: false,
        source_contract_version: PF2E_SOURCE_CONTRACT_VERSION,
        source_upstream_commit: PF2E_SOURCE_PINNED_COMMIT,
        registry_assignment_count: 0,
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
        closure_totals: SourcePathAuditClosureTotals {
            expected_observation_count: 0,
            observed_observation_count: 0,
            failure_count: 0,
            mismatch_count: 0,
        },
        source_diff,
        closure_failures: Vec::new(),
        diagnostics,
        retrieval_predicate_inventory: retrieval_predicate_inventory(),
        paths,
    })
}

fn collect_inventory_paths(
    path: &str,
    value: &Value,
    context: &RecordContext,
    stats: &mut BTreeMap<PathKey, MutablePathStats>,
) {
    match value {
        Value::Object(map) if map.is_empty() => record_path(path, value, context, stats),
        Value::Object(map) => {
            for (key, child) in map {
                collect_inventory_paths(
                    &format!("{path}.{}", object_segment(path, key, map.len())),
                    child,
                    context,
                    stats,
                );
            }
        }
        Value::Array(values) if values.is_empty() => record_path(path, value, context, stats),
        Value::Array(values) => {
            for child in values {
                collect_inventory_paths(&format!("{path}[]"), child, context, stats);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            record_path(path, value, context, stats);
        }
    }
}

fn record_path(
    path: &str,
    value: &Value,
    context: &RecordContext,
    stats: &mut BTreeMap<PathKey, MutablePathStats>,
) {
    let stats = stats
        .entry(PathKey {
            document_type: context.document_type.clone(),
            record_type: context.record_type.clone(),
            path: path.to_string(),
        })
        .or_default();
    stats.occurrence_count += 1;
    *stats
        .value_types
        .entry(value_type(value).to_string())
        .or_insert(0) += 1;
    stats.record_keys.insert(context.record_key.clone());
    if stats.examples.len() < SAMPLE_LIMIT {
        stats.examples.push(SourcePathAuditSample {
            record_key: context.record_key.clone(),
            source_path: context.source_path.clone(),
            value: sample(value),
        });
    }
}

fn diagnostic_path_report(
    key: PathKey,
    stats: MutablePathStats,
    record_count: usize,
) -> SourcePathAuditPathReport {
    SourcePathAuditPathReport {
        document_type: key.document_type,
        record_type: key.record_type,
        path: key.path.clone(),
        path_family: key.path,
        matched_rule_id: "diagnostic_inventory_only".to_string(),
        owner_family: "none".to_string(),
        extractor_identity: None,
        preserved_occurrence_count: None,
        fixture_key: "not_applicable".to_string(),
        validation: "non-authoritative inventory; exact coverage contract required".to_string(),
        checkpoint: "none".to_string(),
        recursive_match: false,
        complete_family_assignment: false,
        record_count,
        occurrence_count: stats.occurrence_count,
        value_types: stats
            .value_types
            .into_iter()
            .map(|(kind, count)| SourcePathAuditValueType { kind, count })
            .collect(),
        disposition: SourcePathCoverageDisposition::Unconsumed,
        owner: "unassigned".to_string(),
        product_rationale:
            "Diagnostic discovery is not product ownership or completeness evidence.".to_string(),
        future_owner: None,
        future_plan: None,
        examples: stats.examples,
    }
}

fn read_json_value(path: &Path) -> Result<Value, IngestError> {
    let serialized = fs::read_to_string(path)
        .map_err(|error| IngestError::RecordParseFailed(error.to_string()))?;
    serde_json::from_str(&serialized)
        .map_err(|error| IngestError::RecordParseFailed(format!("{}: {error}", path.display())))
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

fn sample(value: &Value) -> String {
    const MAX_CHARS: usize = 160;
    let serialized = match value {
        Value::String(value) => value.clone(),
        value => value.to_string(),
    };
    let mut chars = serialized.chars();
    let truncated = chars.by_ref().take(MAX_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn compare_baseline(
    path: &Path,
    current_paths: &[SourcePathAuditPathReport],
) -> Result<SourcePathAuditDiff, IngestError> {
    let baseline = fs::read(path)
        .map_err(|error| baseline_error(path, &format!("failed to read: {error}")))?;
    let baseline: Value = serde_json::from_slice(&baseline)
        .map_err(|error| baseline_error(path, &format!("failed to parse: {error}")))?;
    let version = baseline
        .get("coverage_policy_version")
        .and_then(Value::as_str)
        .ok_or_else(|| baseline_error(path, "is missing coverage_policy_version"))?;
    if version != SOURCE_PATH_INVENTORY_VERSION {
        return Err(baseline_error(
            path,
            &format!("uses policy {version}, expected {SOURCE_PATH_INVENTORY_VERSION}"),
        ));
    }
    let count = baseline
        .get("path_count")
        .and_then(Value::as_u64)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| baseline_error(path, "is missing a valid path_count"))?;
    let entries = baseline
        .get("paths")
        .and_then(Value::as_array)
        .ok_or_else(|| baseline_error(path, "is missing paths"))?;
    if entries.len() != count {
        return Err(baseline_error(
            path,
            &format!(
                "is truncated: path_count is {count} but paths contains {}",
                entries.len()
            ),
        ));
    }
    let baseline_keys = entries
        .iter()
        .map(|entry| baseline_key(path, entry))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let current_keys = current_paths
        .iter()
        .map(|entry| {
            (
                entry.document_type.clone(),
                entry.record_type.clone(),
                entry.path.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    Ok(SourcePathAuditDiff {
        baseline_policy_version: version.to_string(),
        baseline_source_upstream_commit: baseline
            .get("source_upstream_commit")
            .and_then(Value::as_str)
            .map(str::to_string),
        added_paths: current_keys
            .difference(&baseline_keys)
            .map(diff_entry)
            .collect(),
        removed_paths: baseline_keys
            .difference(&current_keys)
            .map(diff_entry)
            .collect(),
        changed_dispositions: Vec::new(),
        consumed_regressions: Vec::new(),
    })
}

fn baseline_key(path: &Path, entry: &Value) -> Result<(String, String, String), IngestError> {
    let field = |name: &str| {
        entry
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| baseline_error(path, &format!("path entry is missing {name}")))
    };
    Ok((
        field("document_type")?,
        field("record_type")?,
        field("path")?,
    ))
}

fn diff_entry(key: &(String, String, String)) -> SourcePathAuditDiffEntry {
    SourcePathAuditDiffEntry {
        document_type: key.0.clone(),
        record_type: key.1.clone(),
        path: key.2.clone(),
        disposition: "unconsumed".to_string(),
    }
}

fn baseline_error(path: &Path, message: &str) -> IngestError {
    IngestError::RecordParseFailed(format!(
        "source inventory baseline {} {message}",
        path.display()
    ))
}

fn inventory_policy_digest() -> String {
    let mut hasher = Sha256::new();
    hasher.update(SOURCE_PATH_INVENTORY_VERSION.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn path_report_order(
    left: &SourcePathAuditPathReport,
    right: &SourcePathAuditPathReport,
) -> std::cmp::Ordering {
    left.document_type
        .cmp(&right.document_type)
        .then_with(|| left.record_type.cmp(&right.record_type))
        .then_with(|| left.path.cmp(&right.path))
}

fn is_creature_path(document_type: &str, record_type: &str) -> bool {
    document_type == "Actor" && record_type == "npc"
}

pub fn disposition_label(disposition: SourcePathCoverageDisposition) -> &'static str {
    match disposition {
        SourcePathCoverageDisposition::Unconsumed => "unconsumed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_inventory_never_claims_coverage() {
        let mut stats = MutablePathStats {
            occurrence_count: 1,
            ..MutablePathStats::default()
        };
        stats.record_keys.insert("pack:id".to_string());
        let report = diagnostic_path_report(
            PathKey {
                document_type: "Actor".to_string(),
                record_type: "npc".to_string(),
                path: "$.system.abilities.*.mod".to_string(),
            },
            stats,
            1,
        );
        assert_eq!(
            report.disposition,
            SourcePathCoverageDisposition::Unconsumed
        );
        assert_eq!(report.matched_rule_id, "diagnostic_inventory_only");
        assert!(report.extractor_identity.is_none());
        assert!(!report.complete_family_assignment);
    }
}
