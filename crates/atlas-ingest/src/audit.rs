//! Offline discovery of serialized source shapes, independent of product models.

use crate::error::IngestError;
use crate::source::dto::SerializedSourceValue;
use std::collections::{BTreeMap, BTreeSet};

mod diff;
mod model;
mod scan;
mod values;
pub use model::{
    SourceFieldValue, SourceFieldValueReport, SourcePathAuditDiff, SourcePathAuditFilters,
    SourcePathAuditOptions, SourcePathAuditPathReport, SourcePathAuditReport,
    SourcePathAuditSample, SourcePathAuditValueType, SourceSchemaKey, SourceSchemaTypeChange,
    SourceValueDiscoveryOptions, SourceValueDiscoveryReport, SourceValueReference,
};
pub use values::discover_source_values;

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
    let filters = SourcePathAuditFilters {
        pack_name: options.pack_name,
        document_type: options.document_type,
        record_type: options.record_type,
    };
    let mut stats = BTreeMap::<SourceSchemaKey, PathStats>::new();
    let source = scan::scan_source(
        &options.source_root,
        options.manifest_path.as_deref(),
        &filters,
        |document_type, record_type, value, context| {
            let mut seen = BTreeSet::new();
            scan::visit("$", "", value, false, &mut |path, _, value, duplicate| {
                let key = SourceSchemaKey {
                    document_type: document_type.into(),
                    record_type: record_type.into(),
                    path: path.into(),
                };
                let stats = stats.entry(key.clone()).or_default();
                stats.record_count += usize::from(seen.insert(key));
                stats.occurrence_count += 1;
                stats.duplicate_member_count += usize::from(duplicate);
                *stats
                    .value_types
                    .entry(scan::value_kind(value).into())
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
            });
        },
    )?;
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
        source_signature: source.signature,
        filters,
        pack_count: source.pack_count,
        record_count: source.record_count,
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
