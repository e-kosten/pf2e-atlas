//! Shared offline source selection and traversal; runtime ingestion does not use this scanner.
use super::model::{SourcePathAuditFilters, SourcePathAuditSample};
use crate::error::IngestError;
use crate::source::dto::{
    SerializedSourceObject, SerializedSourceValue, parse_serialized_source_object,
};
use crate::source::loader::{
    default_manifest_path, json_files, parse_manifest, relative_source_path, resolve_pack_path,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(super) struct SourceScanSummary {
    pub signature: String,
    pub pack_count: usize,
    pub record_count: usize,
}

pub(super) fn scan_source(
    source_root: &Path,
    manifest_path: Option<&Path>,
    filters: &SourcePathAuditFilters,
    mut on_record: impl FnMut(&str, &str, &SerializedSourceValue, &SourcePathAuditSample),
) -> Result<SourceScanSummary, IngestError> {
    let default_manifest = default_manifest_path(source_root);
    let manifest_path = manifest_path.unwrap_or(&default_manifest);
    let mut packs = parse_manifest(manifest_path)?.manifest.packs;
    packs.sort_by(|a, b| a.name.cmp(&b.name));
    let mut fingerprint = Sha256::new();
    hash_part(
        &mut fingerprint,
        &fs::read(manifest_path).map_err(|e| IngestError::ManifestParseFailed(e.to_string()))?,
    );
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
        let files = json_files(&resolve_pack_path(source_root, &pack))?;
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
            let source_path = relative_source_path(source_root, &file);
            hash_part(&mut fingerprint, source_path.as_bytes());
            hash_part(&mut fingerprint, &bytes);
            let id = identity_string(&object, "_id", &file)?;
            let context = SourcePathAuditSample {
                source_path,
                record_key: format!("{}:{id}", pack.name),
                value: String::new(),
            };
            on_record(
                &pack.document_type,
                &record_type,
                &SerializedSourceValue::Object(object),
                &context,
            );
            record_count += 1;
        }
    }
    Ok(SourceScanSummary {
        signature: format!("{:x}", fingerprint.finalize()),
        pack_count,
        record_count,
    })
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

/// Emit the inventory path plus an RFC 6901 pointer into the concrete source document.
pub(super) fn visit(
    path: &str,
    pointer: &str,
    value: &SerializedSourceValue,
    duplicate: bool,
    on_value: &mut impl FnMut(&str, &str, &SerializedSourceValue, bool),
) {
    on_value(path, pointer, value, duplicate);
    match value {
        SerializedSourceValue::Object(object) => {
            let mut counts = BTreeMap::<&str, usize>::new();
            for (name, _) in object.fields() {
                *counts.entry(name).or_default() += 1;
            }
            for (name, child) in object.fields() {
                let escaped = name.replace('~', "~0").replace('/', "~1");
                visit(
                    &object_path(path, name),
                    &format!("{pointer}/{escaped}"),
                    child,
                    counts[name.as_str()] > 1,
                    on_value,
                );
            }
        }
        SerializedSourceValue::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                visit(
                    &format!("{path}[]"),
                    &format!("{pointer}/{index}"),
                    child,
                    false,
                    on_value,
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

pub(super) fn value_kind(value: &SerializedSourceValue) -> &'static str {
    match value {
        SerializedSourceValue::Null => "null",
        SerializedSourceValue::Boolean(_) => "boolean",
        SerializedSourceValue::Number(_) => "number",
        SerializedSourceValue::String(_) => "string",
        SerializedSourceValue::Array(_) => "array",
        SerializedSourceValue::Object(_) => "object",
    }
}
