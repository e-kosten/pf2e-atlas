use super::{
    SCHEMA_VERSION, SourcePathAuditDiff, SourcePathAuditPathReport, SourcePathAuditReport,
    SourceSchemaTypeChange,
};
use crate::error::IngestError;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(super) fn compare(
    current: &SourcePathAuditReport,
    path: &Path,
) -> Result<SourcePathAuditDiff, IngestError> {
    let error = |message: String| {
        IngestError::ManifestParseFailed(format!(
            "invalid schema baseline {}: {message}",
            path.display()
        ))
    };
    let bytes = fs::read(path).map_err(|e| error(e.to_string()))?;
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| error(e.to_string()))?;
    let baseline: SourcePathAuditReport =
        serde_json::from_value(json.get("data").unwrap_or(&json).clone())
            .map_err(|e| error(e.to_string()))?;
    if baseline.schema_version != SCHEMA_VERSION
        || !baseline.complete
        || baseline.paths.len() != baseline.path_count
    {
        return Err(error(
            "expected a complete pf2e-source-schema/v1 snapshot".into(),
        ));
    }
    if baseline.filters != current.filters {
        return Err(error(
            "source selections differ; use identical pack/document/record filters".into(),
        ));
    }
    let previous = baseline
        .paths
        .iter()
        .map(|entry| (&entry.key, entry))
        .collect::<BTreeMap<_, _>>();
    if previous.len() != baseline.paths.len() {
        return Err(error("duplicate schema path identities".into()));
    }
    let observed = current
        .paths
        .iter()
        .map(|entry| (&entry.key, entry))
        .collect::<BTreeMap<_, _>>();
    let mut diff = SourcePathAuditDiff {
        added_paths: vec![],
        removed_paths: vec![],
        changed_types: vec![],
        changed_duplicate_members: vec![],
    };
    for (key, after) in &observed {
        if let Some(before) = previous.get(key) {
            let old_types = types(before);
            let new_types = types(after);
            if old_types != new_types {
                diff.changed_types.push(SourceSchemaTypeChange {
                    key: (*key).clone(),
                    before: old_types,
                    after: new_types,
                });
            }
            if (before.duplicate_member_count > 0) != (after.duplicate_member_count > 0) {
                diff.changed_duplicate_members.push((*key).clone());
            }
        } else {
            diff.added_paths.push((*key).clone());
        }
    }
    for key in previous.keys() {
        if !observed.contains_key(key) {
            diff.removed_paths.push((*key).clone());
        }
    }
    Ok(diff)
}

fn types(entry: &SourcePathAuditPathReport) -> Vec<String> {
    entry
        .value_types
        .iter()
        .map(|kind| kind.kind.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
