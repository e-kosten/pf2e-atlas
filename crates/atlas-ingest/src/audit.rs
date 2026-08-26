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
use crate::source::dto::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, SourceDiagnostic,
    SourceDiagnosticKind, SourceIdentity, parse_item_source, parse_npc_source,
    pinned_source_version_metadata,
};
use crate::source::loader::{
    default_manifest_path, json_files, parse_manifest, relative_source_path, resolve_pack_path,
};
use crate::source::normalize::normalize_record;
#[cfg(test)]
use crate::source::npc_entities::RETAINED_CAPABILITY_PATHS;
use crate::source::npc_entities::{
    capability_note_survival, collect_npc_embedded_candidates, convert_npc_embedded_entities,
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
    canonical: BTreeMap<String, CanonicalPathStats>,
    declarations: BTreeMap<String, Option<&'static str>>,
}

#[derive(Debug, Clone, Default)]
struct CanonicalPathStats {
    occurrence_count: usize,
    destinations: BTreeSet<String>,
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
    let mut canonical_notes = BTreeMap::new();
    for (source_path, unsupported) in capability_note_survival(&conversion) {
        collect_unsupported_note_paths(source_path, unsupported, &mut canonical_notes);
    }
    let mut remaining_note_counts = canonical_notes
        .iter()
        .map(|(path, stats)| (path.clone(), stats.occurrence_count))
        .collect::<BTreeMap<_, _>>();
    for (path, notes) in canonical_notes {
        let observed = inventory.canonical.entry(path).or_default();
        observed.occurrence_count += notes.occurrence_count;
        observed.destinations.extend(notes.destinations);
    }
    collect_expected_canonical_paths(
        "$",
        "$",
        value,
        loaded,
        &conversion,
        &mut remaining_note_counts,
        inventory,
    );
}

fn collect_unsupported_note_paths(
    source_path: &str,
    unsupported: &UnsupportedSourceValue,
    observed: &mut BTreeMap<String, CanonicalPathStats>,
) {
    let value = match unsupported.shape {
        UnsupportedSourceShape::String
        | UnsupportedSourceShape::Number
        | UnsupportedSourceShape::Boolean
        | UnsupportedSourceShape::Array
        | UnsupportedSourceShape::Object
        | UnsupportedSourceShape::Null => serde_json::from_str(&unsupported.value)
            .unwrap_or_else(|_| Value::String(unsupported.value.clone())),
        UnsupportedSourceShape::Missing => return,
    };
    collect_observed_value_paths(
        source_path,
        &value,
        "canonical::CreatureCapability::unsupported_notes",
        observed,
    );
}

fn collect_observed_value_paths(
    source_path: &str,
    value: &Value,
    destination: &'static str,
    observed: &mut BTreeMap<String, CanonicalPathStats>,
) {
    if is_meaningful_value(value) {
        record_canonical_observation(
            observed,
            normalize_observed_source_path(source_path),
            destination,
        );
    }
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                collect_observed_value_paths(
                    &format!("{source_path}.{key}"),
                    child,
                    destination,
                    observed,
                );
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_observed_value_paths(
                    &format!("{source_path}[]"),
                    child,
                    destination,
                    observed,
                );
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
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
    remaining_note_counts: &mut BTreeMap<String, usize>,
    inventory: &mut CreatureSurvivalInventory,
) {
    let preserved_by_note = is_meaningful_value(value)
        && remaining_note_counts
            .get_mut(normalized_path)
            .is_some_and(|remaining| {
                if *remaining == 0 {
                    false
                } else {
                    *remaining -= 1;
                    true
                }
            });
    if is_meaningful_value(value)
        && !preserved_by_note
        && let Some(destination) =
            canonical_destination(indexed_path, normalized_path, loaded, conversion, inventory)
    {
        record_canonical_observation(
            &mut inventory.canonical,
            normalized_path.to_string(),
            destination,
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
                    remaining_note_counts,
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
                    remaining_note_counts,
                    inventory,
                );
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
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
        "source::npc_core" => core_destination(normalized_path, loaded),
        "source::dto + source::npc_core + atlas-record::creature_projection" => {
            core_destination(normalized_path, loaded)
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

fn core_destination(normalized_path: &str, loaded: &LoadedSourceRecord) -> Option<&'static str> {
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
    let fact = loaded.facts.source_facts.embedded_items.get(index)?;
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
    let nested_id = source_item_index(indexed_path).and_then(|index| {
        loaded
            .facts
            .source_facts
            .embedded_items
            .get(index)
            .map(|fact| fact.item_id.as_str())
    });
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
    let candidate = loaded
        .facts
        .npc_embedded_candidates
        .as_ref()?
        .items
        .as_value()?
        .get(index)?;
    let embedded = conversion.embedded.as_value()?;
    embedded.occurrences.iter().find(|occurrence| {
        occurrence
            .source_identity
            .nested_source_id
            .as_value()
            .is_some_and(|id| id.as_str() == candidate.nested_source_id)
    })
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
    let normalized_expected = normalize_observed_source_path(indexed_path);
    if capability_notes(&occurrence.capability)
        .iter()
        .any(|note| normalize_observed_source_path(&note.source_path) == normalized_expected)
    {
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
    for path in paths.iter_mut().filter(|path| {
        is_creature_path(&path.document_type, &path.record_type)
            && path.disposition == SourcePathCoverageDisposition::Consumed
    }) {
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
                .map(|path| {
                    (
                        path.to_string(),
                        CanonicalPathStats {
                            occurrence_count: 1,
                            destinations: BTreeSet::from([String::from(
                                "canonical::CreatureCapability::unsupported_notes",
                            )]),
                        },
                    )
                })
                .collect(),
            ..CreatureSurvivalInventory::default()
        };

        let mut complete = reports.clone();
        assert!(reconcile_creature_survival(&mut complete, &inventory).is_empty());
        assert!(complete.iter().all(|path| {
            path.preserved_occurrence_count == Some(1)
                && path.extractor_identity.as_deref()
                    == Some("canonical::CreatureCapability::unsupported_notes")
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
    fn pinned_creature_audit_proves_all_consumed_occurrences_and_destinations_survive() {
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

        let consumed = report
            .paths
            .iter()
            .filter(|path| path.disposition == SourcePathCoverageDisposition::Consumed)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(consumed.len(), 608);
        assert_eq!(
            consumed
                .iter()
                .map(|path| path.occurrence_count)
                .sum::<usize>(),
            1_746_734
        );
        assert_eq!(
            consumed
                .iter()
                .map(|path| path.preserved_occurrence_count.unwrap_or_default())
                .sum::<usize>(),
            1_746_734
        );
        assert!(consumed.iter().all(|path| {
            path.extractor_identity.as_deref().is_some_and(|identity| {
                identity.contains("canonical::")
                    && !identity.contains("hydrated::")
                    && !identity.contains("serde_json")
            })
        }));

        let inventory = CreatureSurvivalInventory {
            canonical: consumed
                .iter()
                .map(|path| {
                    (
                        path.path.clone(),
                        CanonicalPathStats {
                            occurrence_count: path
                                .preserved_occurrence_count
                                .expect("consumed path has canonical closure count"),
                            destinations: path
                                .extractor_identity
                                .as_deref()
                                .expect("consumed path has canonical destination")
                                .split(" + ")
                                .map(str::to_string)
                                .collect(),
                        },
                    )
                })
                .collect(),
            ..CreatureSurvivalInventory::default()
        };
        for dropped in &consumed {
            let mut mutated = consumed.clone();
            let mut mutated_inventory = CreatureSurvivalInventory {
                canonical: inventory.canonical.clone(),
                ..CreatureSurvivalInventory::default()
            };
            mutated_inventory.canonical.remove(&dropped.path);
            let failures = reconcile_creature_survival(&mut mutated, &mutated_inventory);
            assert_eq!(
                failures.len(),
                1,
                "dropping consumed canonical destination {} must fail closure",
                dropped.path
            );
            assert_eq!(failures[0].path, dropped.path);
        }

        let destination_families = inventory
            .canonical
            .values()
            .flat_map(|stats| stats.destinations.iter().cloned())
            .collect::<BTreeSet<_>>();
        assert!(destination_families.len() > 10);
        for destination in destination_families {
            let mut mutated = consumed.clone();
            let mut mutated_inventory = CreatureSurvivalInventory {
                canonical: inventory.canonical.clone(),
                ..CreatureSurvivalInventory::default()
            };
            mutated_inventory
                .canonical
                .retain(|_, stats| !stats.destinations.contains(&destination));
            let failures = reconcile_creature_survival(&mut mutated, &mutated_inventory);
            assert!(
                !failures.is_empty(),
                "dropping canonical destination family {destination} must fail closure"
            );
        }

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
