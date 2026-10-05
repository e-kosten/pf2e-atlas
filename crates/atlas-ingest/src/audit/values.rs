//! Value frequencies for one observed source path, independent of product modeling.
use super::model::{
    SourceFieldValue, SourceFieldValueReport, SourcePathAuditValueType, SourceSchemaKey,
    SourceValueDiscoveryOptions, SourceValueDiscoveryReport, SourceValueReference,
};
use super::scan::{scan_source, value_kind, visit};
use crate::error::IngestError;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct FieldStats {
    total_records: usize,
    record_count: usize,
    occurrence_count: usize,
    duplicate_member_count: usize,
    types: BTreeMap<String, usize>,
    values: BTreeMap<String, SourceFieldValue>,
}

pub fn discover_source_values(
    options: SourceValueDiscoveryOptions,
) -> Result<SourceValueDiscoveryReport, IngestError> {
    if options.path != "$" && !options.path.starts_with("$.") && !options.path.starts_with("$[") {
        return Err(IngestError::InvalidSourceDiscoveryPath(options.path));
    }
    let mut fields = BTreeMap::<SourceSchemaKey, FieldStats>::new();
    let source = scan_source(
        &options.source_root,
        options.manifest_path.as_deref(),
        &options.filters,
        |document_type, record_type, value, context| {
            let key = SourceSchemaKey {
                document_type: document_type.into(),
                record_type: record_type.into(),
                path: options.path.clone(),
            };
            let stats = fields.entry(key).or_default();
            stats.total_records += 1;
            let mut seen = BTreeSet::new();
            visit(
                "$",
                "",
                value,
                false,
                &mut |path, pointer, value, duplicate| {
                    if path != options.path {
                        return;
                    }
                    stats.occurrence_count += 1;
                    stats.duplicate_member_count += usize::from(duplicate);
                    let kind = value_kind(value);
                    *stats.types.entry(kind.into()).or_default() += 1;
                    let value_json = value.compact_json();
                    let first_in_record = seen.insert(value_json.clone());
                    let entry = stats.values.entry(value_json.clone()).or_insert_with(|| {
                        SourceFieldValue {
                            value_json,
                            value_type: kind.into(),
                            record_count: 0,
                            occurrence_count: 0,
                            examples: Vec::new(),
                        }
                    });
                    entry.record_count += usize::from(first_in_record);
                    entry.occurrence_count += 1;
                    if first_in_record && entry.examples.len() < options.sample_limit {
                        let example = SourceValueReference {
                            source_path: context.source_path.clone(),
                            record_key: context.record_key.clone(),
                            source_pointer: pointer.into(),
                        };
                        if !entry.examples.contains(&example) {
                            entry.examples.push(example);
                        }
                    }
                },
            );
            stats.record_count += usize::from(!seen.is_empty());
        },
    )?;
    let fields = fields
        .into_iter()
        .map(|(key, stats)| {
            let distinct_value_count = stats.values.len();
            let mut values = stats.values.into_values().collect::<Vec<_>>();
            values.sort_by(|a, b| {
                b.occurrence_count
                    .cmp(&a.occurrence_count)
                    .then_with(|| a.value_json.cmp(&b.value_json))
            });
            if let Some(limit) = options.limit {
                values.truncate(limit);
            }
            SourceFieldValueReport {
                key,
                record_count: stats.record_count,
                missing_record_count: stats.total_records - stats.record_count,
                occurrence_count: stats.occurrence_count,
                duplicate_member_count: stats.duplicate_member_count,
                value_types: stats
                    .types
                    .into_iter()
                    .map(|(kind, count)| SourcePathAuditValueType { kind, count })
                    .collect(),
                distinct_value_count,
                complete: values.len() == distinct_value_count,
                values,
            }
        })
        .collect::<Vec<_>>();
    Ok(SourceValueDiscoveryReport {
        schema_version: "pf2e-source-values/v1".into(),
        source_signature: source.signature,
        filters: options.filters,
        path: options.path,
        pack_count: source.pack_count,
        record_count: source.record_count,
        sample_limit: options.sample_limit,
        complete: fields.iter().all(|field| field.complete),
        fields,
    })
}
