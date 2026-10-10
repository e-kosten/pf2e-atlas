use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::error::IngestError;
use crate::source::discovery::{
    default_manifest_path, json_files, parse_manifest, relative_source_path, resolve_pack_path,
};

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditReport {
    pub source_root: String,
    pub manifest_path: String,
    pub pack_count: usize,
    pub record_count: usize,
    pub path_count: usize,
    pub filters: SourcePathAuditFilters,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditPathReport {
    pub path: String,
    pub record_count: usize,
    pub occurrence_count: usize,
    pub value_types: Vec<SourcePathAuditValueType>,
    pub examples: Vec<SourcePathAuditSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditValueType {
    pub kind: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePathAuditSample {
    pub record_key: String,
    pub source_path: String,
    pub value: String,
}

#[derive(Debug, Default)]
struct MutablePathStats {
    occurrence_count: usize,
    value_types: BTreeMap<String, usize>,
    record_keys: BTreeSet<String>,
    examples: Vec<SourcePathAuditSample>,
}

#[derive(Debug)]
struct RecordContext {
    record_key: String,
    source_path: String,
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
    let mut stats = BTreeMap::<String, MutablePathStats>::new();
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
        for path in json_files(&resolved_path)? {
            let value = read_json_value(&path)?;
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
                .unwrap_or_else(|| relative_source_path(&source_root, &path));
            let context = RecordContext {
                record_key,
                source_path: relative_source_path(&source_root, &path),
            };
            record_count += 1;
            let mut record_paths = BTreeSet::new();
            collect_value_paths("$", &value, &context, &mut stats, &mut record_paths);
            for path in record_paths {
                if let Some(path_stats) = stats.get_mut(&path) {
                    path_stats.record_keys.insert(context.record_key.clone());
                }
            }
        }
    }

    let limit = options.limit.unwrap_or(DEFAULT_PATH_LIMIT);
    let mut paths = stats
        .into_iter()
        .filter_map(|(path, stats)| {
            let record_count = stats.record_keys.len();
            (record_count >= options.min_records).then(|| SourcePathAuditPathReport {
                path,
                record_count,
                occurrence_count: stats.occurrence_count,
                value_types: stats
                    .value_types
                    .into_iter()
                    .map(|(kind, count)| SourcePathAuditValueType { kind, count })
                    .collect(),
                examples: stats.examples,
            })
        })
        .collect::<Vec<_>>();
    paths.sort_by(|left, right| {
        right
            .record_count
            .cmp(&left.record_count)
            .then_with(|| right.occurrence_count.cmp(&left.occurrence_count))
            .then_with(|| left.path.cmp(&right.path))
    });
    let path_count = paths.len();
    paths.truncate(limit);

    Ok(SourcePathAuditReport {
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
        },
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
    stats: &mut BTreeMap<String, MutablePathStats>,
    record_paths: &mut BTreeSet<String>,
) {
    record_path(path, value, context, stats, record_paths);
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let segment = object_segment(path, key, map.len());
                let child_path = format!("{path}.{segment}");
                collect_value_paths(&child_path, child, context, stats, record_paths);
            }
        }
        Value::Array(values) => {
            let child_path = format!("{path}[]");
            for child in values {
                collect_value_paths(&child_path, child, context, stats, record_paths);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn record_path(
    path: &str,
    value: &Value,
    context: &RecordContext,
    stats: &mut BTreeMap<String, MutablePathStats>,
    record_paths: &mut BTreeSet<String>,
) {
    if matches!(value, Value::Array(_) | Value::Object(_)) {
        return;
    }
    let path_stats = stats.entry(path.to_string()).or_default();
    path_stats.occurrence_count += 1;
    *path_stats
        .value_types
        .entry(value_type(value).to_string())
        .or_insert(0) += 1;
    if let Some(sample) = scalar_sample(value)
        && path_stats.examples.len() < SAMPLE_LIMIT
        && !sample.is_empty()
    {
        path_stats.examples.push(SourcePathAuditSample {
            record_key: context.record_key.clone(),
            source_path: context.source_path.clone(),
            value: sample,
        });
    }
    record_paths.insert(path.to_string());
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
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => return None,
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
    path.ends_with(".damageRolls")
        || path.ends_with(".damage")
        || path.ends_with(".overlays")
        || path.ends_with(".resources")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_normalization_wildcards_dynamic_damage_keys() {
        assert_eq!(
            object_segment("$.items[].system.damageRolls", "abc123xyz789", 2),
            "*"
        );
        assert_eq!(
            object_segment("$.system.attributes.ac", "value", 2),
            "value"
        );
    }
}
