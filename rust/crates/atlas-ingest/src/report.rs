use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use atlas_domain::{MetricDomain, PublicationFamily};
use serde_json::{Value, json};

use crate::{IngestDiagnostics, LoadedRecord, MetricValue, SkippedRecord, SourceLoad};

pub fn analyze_source_load(source_root: PathBuf, source: SourceLoad) -> Value {
    let hidden_record_keys = source
        .remaster_links
        .iter()
        .map(|link| link.legacy_record_key.to_string())
        .collect::<BTreeSet<_>>();
    let default_visible_record_count = source
        .records
        .iter()
        .filter(|record| {
            record.is_default_visible && !hidden_record_keys.contains(&record.key.to_string())
        })
        .count();
    let generated_record_count = source
        .records
        .iter()
        .filter(|record| is_generated_record(record))
        .count();

    json!({
        "status": "ok",
        "source": {
            "root": source_root.display().to_string(),
            "manifest": source.manifest_path.display().to_string(),
            "source_signature": source.source_signature,
        },
        "pack_count": source.packs.len(),
        "loaded_source_pack_count": source.packs.iter().filter(|pack| !pack.declared_path.starts_with("derived://")).count(),
        "record_count": source.records.len(),
        "loaded_source_record_count": source.records.len() - generated_record_count,
        "generated_record_count": generated_record_count,
        "default_visible_record_count": default_visible_record_count,
        "hidden_record_count": source.records.len() - default_visible_record_count,
        "by_record_family": count_by_record_family(&source.records),
        "by_foundry_taxonomy": count_by_foundry_taxonomy(&source.records),
        "by_publication_family": count_by_publication_family(&source.records),
        "text": {
            "records_with_description": source.records.iter().filter(|record| record.description_text.is_some()).count(),
            "records_with_blurb": source.records.iter().filter(|record| record.blurb_text.is_some()).count(),
        },
        "side_data": {
            "actor_records": source.records.iter().filter(|record| record.actor_data.is_some()).count(),
            "item_records": source.records.iter().filter(|record| record.item_data.is_some()).count(),
            "spell_records": source.records.iter().filter(|record| record.spell_data.is_some()).count(),
        },
        "metrics": metrics_json(&source.records, &hidden_record_keys),
        "relationships": {
            "reference_edges": source.references.len(),
            "record_aliases": source.aliases.len(),
            "remaster_links": source.remaster_links.len(),
        },
        "diagnostics": diagnostics_json(&source.diagnostics),
        "skipped_record_count": source.skipped_records.len(),
        "skipped_records": skipped_records_json(&source.skipped_records),
        "warnings": source.warnings,
    })
}

pub fn diagnostics_json(diagnostics: &IngestDiagnostics) -> Value {
    json!({
        "taxonomy": {
            "folder_records": diagnostics.taxonomy_folder_records,
            "glossary_records": diagnostics.taxonomy_glossary_records,
        },
        "variants": {
            "parenthetical_records": diagnostics.variant_parenthetical_records,
            "suffix_records": diagnostics.variant_suffix_records,
            "creature_blurb_records": diagnostics.variant_creature_blurb_records,
            "creature_suffix_records": diagnostics.variant_creature_suffix_records,
            "exact_base_records": diagnostics.variant_exact_base_records,
        },
        "generated_afflictions": {
            "canonical_records": diagnostics.generated_affliction_canonical_records,
            "instance_records": diagnostics.generated_affliction_instance_records,
            "reference_edges": diagnostics.generated_affliction_reference_edges,
        }
    })
}

pub fn skipped_records_json(skipped_records: &[SkippedRecord]) -> Vec<Value> {
    skipped_records
        .iter()
        .map(|record| {
            json!({
                "path": record.path.display().to_string(),
                "reason": record.reason,
            })
        })
        .collect()
}

fn count_by_record_family(records: &[LoadedRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in records {
        *counts
            .entry(record.record_family.as_str().to_string())
            .or_insert(0) += 1;
    }
    counts
}

fn count_by_foundry_taxonomy(records: &[LoadedRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in records {
        *counts
            .entry(format!(
                "{}|{}",
                record.foundry_document_type, record.foundry_record_type
            ))
            .or_insert(0) += 1;
    }
    counts
}

fn count_by_publication_family(records: &[LoadedRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in records {
        *counts
            .entry(publication_family_label(record.publication_family).to_string())
            .or_insert(0) += 1;
    }
    counts
}

fn metrics_json(records: &[LoadedRecord], hidden_record_keys: &BTreeSet<String>) -> Value {
    let mut rows_by_domain = BTreeMap::<String, usize>::new();
    let mut keys_by_domain = BTreeMap::<String, BTreeSet<String>>::new();
    let mut text_boolean_values = BTreeSet::<(String, String, String, String)>::new();
    for record in records {
        let is_default_visible =
            record.is_default_visible && !hidden_record_keys.contains(&record.key.to_string());
        for metric in &record.metrics {
            let domain = metric_domain_label(metric.domain).to_string();
            *rows_by_domain.entry(domain.clone()).or_insert(0) += 1;
            keys_by_domain
                .entry(domain.clone())
                .or_default()
                .insert(metric.key.clone());
            match &metric.value {
                MetricValue::Text(value) if is_default_visible => {
                    text_boolean_values.insert((
                        domain,
                        record.record_family.as_str().to_string(),
                        metric.key.clone(),
                        value.clone(),
                    ));
                }
                MetricValue::Boolean(value) => {
                    if is_default_visible {
                        text_boolean_values.insert((
                            domain,
                            record.record_family.as_str().to_string(),
                            metric.key.clone(),
                            i64::from(*value).to_string(),
                        ));
                    }
                }
                MetricValue::Number(_) | MetricValue::Text(_) => {}
            }
        }
    }

    let keys_by_domain = keys_by_domain
        .into_iter()
        .map(|(domain, keys)| (domain, keys.len()))
        .collect::<BTreeMap<_, _>>();

    json!({
        "metric_rows_by_domain": rows_by_domain,
        "metric_keys_by_domain": keys_by_domain,
        "metric_value_catalog_rows": text_boolean_values.len(),
    })
}

fn is_generated_record(record: &LoadedRecord) -> bool {
    matches!(
        record.pack_name.as_str(),
        "derived-afflictions" | "derived-affliction-instances"
    )
}

fn publication_family_label(family: PublicationFamily) -> &'static str {
    match family {
        PublicationFamily::Core => "core",
        PublicationFamily::Rules => "rules",
        PublicationFamily::Adventure => "adventure",
        PublicationFamily::Unknown => "unknown",
    }
}

fn metric_domain_label(domain: MetricDomain) -> &'static str {
    match domain {
        MetricDomain::Actor => "actor",
        MetricDomain::Item => "item",
    }
}
