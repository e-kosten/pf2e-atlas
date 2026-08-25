use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::RecordKey;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::IngestError;
use crate::source::dto::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, SourceDiagnostic,
    SourceDiagnosticKind, SourceIdentity, parse_item_source, parse_npc_source,
    pinned_source_version_metadata,
};
use crate::source::loader::{
    default_manifest_path, json_files, parse_manifest, relative_source_path, resolve_pack_path,
};
use crate::source::npc_entities::{
    RETAINED_CAPABILITY_PATHS, collect_npc_embedded_candidates, convert_npc_embedded_entities,
    retained_capability_survival,
};

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
    hydrated: BTreeMap<PathKey, MutablePathStats>,
    canonical: BTreeMap<String, usize>,
}

pub fn audit_source_paths(
    options: SourcePathAuditOptions,
) -> Result<SourcePathAuditReport, IngestError> {
    let source_root = options.source_root;
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
            collect_creature_survival(&value, &context, &mut creature_survival);
            validate_typed_source(&value, &context, &mut typed_diagnostics);
        }
    }

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
    let enforcement = SourcePathAuditEnforcement {
        mode: if options.strict {
            SourcePathAuditMode::Strict
        } else {
            SourcePathAuditMode::Relaxed
        },
        passed: !options.strict || violation_count == 0,
        violation_count: if options.strict { violation_count } else { 0 },
        aggregate_warning_count: violation_count,
    };
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
        source_diff,
        closure_failures,
        diagnostics,
        retrieval_predicate_inventory: predicate_inventory,
        paths,
    })
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
    inventory: &mut CreatureSurvivalInventory,
) {
    if !is_creature_path(&context.document_type, &context.record_type) {
        return;
    }
    let identity = SourceIdentity::new(&context.record_key, &context.source_path);
    let Ok(source) = parse_npc_source(pinned_source_version_metadata(), identity, value.clone())
    else {
        return;
    };
    collect_value_paths("$", value, context, &mut inventory.hydrated);

    let Ok(owner) = RecordKey::parse(&context.record_key) else {
        return;
    };
    let candidates = collect_npc_embedded_candidates(&source);
    let conversion = convert_npc_embedded_entities(owner, &candidates, |_| None);
    for (path, count) in retained_capability_survival(&conversion) {
        *inventory.canonical.entry(path.to_string()).or_insert(0) += count;
    }
}

fn reconcile_creature_survival(
    paths: &mut [SourcePathAuditPathReport],
    inventory: &CreatureSurvivalInventory,
) -> Vec<SourcePathAuditClosureFailure> {
    let canonical_paths = RETAINED_CAPABILITY_PATHS
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut failures = Vec::new();
    for path in paths.iter_mut().filter(|path| {
        is_creature_path(&path.document_type, &path.record_type)
            && path.disposition == SourcePathCoverageDisposition::Consumed
    }) {
        let (preserved, extractor_identity) = if canonical_paths.contains(path.path.as_str()) {
            (
                inventory.canonical.get(&path.path).copied().unwrap_or(0),
                "canonical::CreatureEmbeddedEntities::occurrence.unsupported_notes",
            )
        } else {
            let key = PathKey {
                document_type: path.document_type.clone(),
                record_type: path.record_type.clone(),
                path: path.path.clone(),
            };
            (
                inventory
                    .hydrated
                    .get(&key)
                    .map_or(0, |stats| stats.occurrence_count),
                "hydrated::VersionedNpcSource",
            )
        };
        path.preserved_occurrence_count = Some(preserved);
        path.extractor_identity = Some(extractor_identity.to_string());
        if preserved != path.occurrence_count {
            failures.push(SourcePathAuditClosureFailure {
                document_type: path.document_type.clone(),
                record_type: path.record_type.clone(),
                path: path.path.clone(),
                source_occurrence_count: path.occurrence_count,
                preserved_occurrence_count: preserved,
            });
        }
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

    fn consumed_creature_path(path: &str, occurrences: usize) -> SourcePathAuditPathReport {
        let mut stats = MutablePathStats {
            occurrence_count: occurrences,
            ..MutablePathStats::default()
        };
        stats.record_keys.insert("test-pack:test-id".to_string());
        path_report(
            PathKey {
                document_type: "Actor".to_string(),
                record_type: "npc".to_string(),
                path: path.to_string(),
            },
            stats,
            1,
        )
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

    #[test]
    fn canonical_capability_closure_fails_when_any_retained_family_is_dropped() {
        let reports = RETAINED_CAPABILITY_PATHS
            .into_iter()
            .map(|path| consumed_creature_path(path, 1))
            .collect::<Vec<_>>();
        let inventory = CreatureSurvivalInventory {
            canonical: RETAINED_CAPABILITY_PATHS
                .into_iter()
                .map(|path| (path.to_string(), 1))
                .collect(),
            ..CreatureSurvivalInventory::default()
        };

        let mut complete = reports.clone();
        assert!(reconcile_creature_survival(&mut complete, &inventory).is_empty());
        assert!(complete.iter().all(|path| {
            path.preserved_occurrence_count == Some(1)
                && path.extractor_identity.as_deref()
                    == Some("canonical::CreatureEmbeddedEntities::occurrence.unsupported_notes")
        }));

        for dropped in RETAINED_CAPABILITY_PATHS {
            let mut mutated = reports.clone();
            let mut mutated_inventory = CreatureSurvivalInventory {
                canonical: inventory.canonical.clone(),
                ..CreatureSurvivalInventory::default()
            };
            mutated_inventory.canonical.remove(dropped);
            let failures = reconcile_creature_survival(&mut mutated, &mutated_inventory);
            assert_eq!(failures.len(), 1, "dropping {dropped} must fail closure");
            assert_eq!(failures[0].path, dropped);
            assert_eq!(failures[0].source_occurrence_count, 1);
            assert_eq!(failures[0].preserved_occurrence_count, 0);
        }
    }

    #[test]
    fn pinned_creature_audit_proves_all_retained_capability_occurrences_survive() {
        let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let report = audit_source_paths(SourcePathAuditOptions {
            source_root: PathBuf::from(source_root),
            document_type: Some("Actor".to_string()),
            record_type: Some("npc".to_string()),
            strict: true,
            limit: Some(usize::MAX),
            ..SourcePathAuditOptions::default()
        })
        .expect("pinned creature audit");

        assert_eq!(report.summary.creature_paths, 614);
        assert_eq!(report.summary.creature_consumed_paths, 608);
        assert_eq!(report.summary.creature_provenance_only_paths, 6);
        assert_eq!(report.summary.creature_deferred_paths, 0);
        assert_eq!(report.summary.creature_unknown_paths, 0);
        assert_eq!(report.summary.creature_catch_all_paths, 0);
        assert_eq!(report.summary.creature_unowned_paths, 0);
        assert_eq!(report.summary.creature_consumed_regressions, 0);
        assert!(report.closure_failures.is_empty());
        assert!(report.enforcement.passed);

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
