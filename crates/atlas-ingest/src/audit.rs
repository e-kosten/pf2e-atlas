//! Offline discovery of serialized source shapes, independent of product models.

use crate::error::IngestError;
use crate::source::dto::{
    SerializedSourceObject, SerializedSourceValue, parse_serialized_source_object,
};
use crate::source::loader::{
    default_manifest_path, json_files, parse_manifest, relative_source_path, resolve_pack_path,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

mod diff;
mod model;
pub use model::{
    SourcePathAuditDiff, SourcePathAuditFilters, SourcePathAuditOptions, SourcePathAuditPathReport,
    SourcePathAuditReport, SourcePathAuditSample, SourcePathAuditValueType, SourceSchemaKey,
    SourceSchemaTypeChange,
};

const SCHEMA_VERSION: &str = "pf2e-source-schema/v1";
const SAMPLE_LIMIT: usize = 3;

#[derive(Default)]
struct PathStats {
    record_count: usize,
    occurrence_count: usize,
    duplicate_member_count: usize,
    value_types: BTreeMap<String, usize>,
    examples: Vec<SourcePathAuditSample>,
}

/// Missing packs and malformed documents are errors; partial checkouts cannot
/// silently produce complete baselines. Display limits apply after comparison.
pub fn audit_source_paths(
    options: SourcePathAuditOptions,
) -> Result<SourcePathAuditReport, IngestError> {
    if options.strict && options.baseline_report.is_none() {
        return Err(IngestError::ManifestParseFailed(
            "--strict requires --baseline; it checks schema changes, not model coverage".into(),
        ));
    }
    let manifest_path = options
        .manifest_path
        .clone()
        .unwrap_or_else(|| default_manifest_path(&options.source_root));
    let mut packs = parse_manifest(&manifest_path)?.manifest.packs;
    packs.sort_by(|a, b| a.name.cmp(&b.name));
    let filters = SourcePathAuditFilters {
        pack_name: options.pack_name,
        document_type: options.document_type,
        record_type: options.record_type,
    };
    let mut fingerprint = Sha256::new();
    hash_part(
        &mut fingerprint,
        &fs::read(&manifest_path).map_err(|e| IngestError::ManifestParseFailed(e.to_string()))?,
    );
    let mut stats = BTreeMap::<SourceSchemaKey, PathStats>::new();
    let mut pack_count = 0;
    let mut record_count = 0;
    for pack in packs {
        if filters
            .pack_name
            .as_ref()
            .is_some_and(|name| name != &pack.name)
            || filters
                .document_type
                .as_ref()
                .is_some_and(|kind| kind != &pack.document_type)
        {
            continue;
        }
        let files = json_files(&resolve_pack_path(&options.source_root, &pack))?;
        pack_count += 1;
        for file in files {
            let bytes = fs::read(&file)
                .map_err(|e| IngestError::SourceUnavailable(format!("{}: {e}", file.display())))?;
            let object = parse_serialized_source_object(&bytes)
                .map_err(|e| IngestError::RecordParseFailed(format!("{}: {e}", file.display())))?;
            let record_type = identity_string(&object, "type", &file)?;
            if filters
                .record_type
                .as_ref()
                .is_some_and(|kind| kind != &record_type)
            {
                continue;
            }
            let source_path = relative_source_path(&options.source_root, &file);
            hash_part(&mut fingerprint, source_path.as_bytes());
            hash_part(&mut fingerprint, &bytes);
            let id = identity_string(&object, "_id", &file)?;
            let context = SourcePathAuditSample {
                source_path,
                record_key: format!("{}:{id}", pack.name),
                value: String::new(),
            };
            let mut seen = BTreeSet::new();
            visit(
                "$",
                &SerializedSourceValue::Object(object),
                &pack.document_type,
                &record_type,
                &context,
                false,
                &mut seen,
                &mut stats,
            );
            record_count += 1;
        }
    }
    let paths = stats
        .into_iter()
        .map(|(key, stats)| SourcePathAuditPathReport {
            key,
            record_count: stats.record_count,
            occurrence_count: stats.occurrence_count,
            duplicate_member_count: stats.duplicate_member_count,
            value_types: stats
                .value_types
                .into_iter()
                .map(|(kind, count)| SourcePathAuditValueType { kind, count })
                .collect(),
            examples: stats.examples,
        })
        .collect::<Vec<_>>();
    let mut report = SourcePathAuditReport {
        schema_version: SCHEMA_VERSION.into(),
        source_signature: format!("{:x}", fingerprint.finalize()),
        filters,
        pack_count,
        record_count,
        path_count: paths.len(),
        complete: true,
        paths,
        source_diff: None,
    };
    if let Some(baseline) = options.baseline_report {
        report.source_diff = Some(diff::compare(&report, &baseline)?);
    }
    report
        .paths
        .retain(|path| path.record_count >= options.min_records);
    if let Some(limit) = options.limit {
        report.paths.truncate(limit);
    }
    report.complete = report.paths.len() == report.path_count;
    Ok(report)
}

fn hash_part(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

fn identity_string(
    object: &SerializedSourceObject,
    field: &str,
    file: &Path,
) -> Result<String, IngestError> {
    let members = object.members(field);
    match members.as_slice() {
        [] => Ok(String::new()),
        [SerializedSourceValue::String(value)] => Ok(value.clone()),
        _ => Err(IngestError::RecordParseFailed(format!(
            "{}: {field} must be absent or one string",
            file.display()
        ))),
    }
}

#[allow(clippy::too_many_arguments)]
fn visit(
    path: &str,
    value: &SerializedSourceValue,
    document_type: &str,
    record_type: &str,
    context: &SourcePathAuditSample,
    duplicate: bool,
    seen: &mut BTreeSet<SourceSchemaKey>,
    inventory: &mut BTreeMap<SourceSchemaKey, PathStats>,
) {
    let key = SourceSchemaKey {
        document_type: document_type.into(),
        record_type: record_type.into(),
        path: path.into(),
    };
    let stats = inventory.entry(key.clone()).or_default();
    stats.record_count += usize::from(seen.insert(key));
    stats.occurrence_count += 1;
    stats.duplicate_member_count += usize::from(duplicate);
    *stats
        .value_types
        .entry(value_kind(value).into())
        .or_default() += 1;
    if stats.examples.len() < SAMPLE_LIMIT {
        let mut sample = context.clone();
        sample.value = match value {
            SerializedSourceValue::Object(_) => "{…}".into(),
            SerializedSourceValue::Array(_) => "[…]".into(),
            _ => value.compact_json().chars().take(160).collect(),
        };
        if !stats.examples.contains(&sample) {
            stats.examples.push(sample);
        }
    }
    match value {
        SerializedSourceValue::Object(object) => {
            let mut counts = BTreeMap::<&str, usize>::new();
            for (name, _) in object.fields() {
                *counts.entry(name).or_default() += 1;
            }
            for (name, child) in object.fields() {
                visit(
                    &object_path(path, name),
                    child,
                    document_type,
                    record_type,
                    context,
                    counts[name.as_str()] > 1,
                    seen,
                    inventory,
                );
            }
        }
        SerializedSourceValue::Array(values) => {
            for child in values {
                visit(
                    &format!("{path}[]"),
                    child,
                    document_type,
                    record_type,
                    context,
                    false,
                    seen,
                    inventory,
                );
            }
        }
        _ => {}
    }
}

fn object_path(parent: &str, key: &str) -> String {
    // Named skills/saves/resources stay visible; only known keyed maps collapse.
    if [
        ".damageRolls",
        ".system.damage",
        ".heightening.damage",
        ".overlays",
        ".itemGrants",
        ".system.items",
    ]
    .iter()
    .any(|suffix| parent.ends_with(suffix))
        || (parent.contains(".heightening.levels[") && parent.ends_with("].damage"))
    {
        format!("{parent}.*")
    } else if !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && key
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    {
        format!("{parent}.{key}")
    } else {
        format!("{parent}[{}]", serde_json::Value::String(key.into()))
    }
}

fn value_kind(value: &SerializedSourceValue) -> &'static str {
    match value {
        SerializedSourceValue::Null => "null",
        SerializedSourceValue::Boolean(_) => "boolean",
        SerializedSourceValue::Number(_) => "number",
        SerializedSourceValue::String(_) => "string",
        SerializedSourceValue::Array(_) => "array",
        SerializedSourceValue::Object(_) => "object",
    }
}
