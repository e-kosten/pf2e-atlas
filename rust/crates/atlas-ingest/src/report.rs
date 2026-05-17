use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use atlas_domain::{MetricDomain, PublicationFamily};
use atlas_record::{ReferenceEdgeFacts, ReferenceGraphMode, reference_edge_matches_mode};
use serde::Serialize;
use serde_json::{Value, json};

use crate::diagnostics::IngestDiagnostics;
use crate::error::IngestError;
use crate::records::{LoadedSourceRecord, MetricValue};
use crate::source::model::{
    BuildArtifactReport, DocumentEmbeddingTokenizationReport,
    DocumentEmbeddingTruncationExampleReport, SkippedRecord, SourceLoad,
};
use crate::source_pipeline;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SourceAnalysisReport {
    pub status: &'static str,
    pub source: SourceAnalysisSourceReport,
    pub pack_count: usize,
    pub loaded_source_pack_count: usize,
    pub record_count: usize,
    pub loaded_source_record_count: usize,
    pub generated_record_count: usize,
    pub default_visible_record_count: usize,
    pub hidden_record_count: usize,
    pub by_record_family: BTreeMap<String, usize>,
    pub by_foundry_taxonomy: BTreeMap<String, usize>,
    pub by_publication_family: BTreeMap<String, usize>,
    pub text: SourceAnalysisTextReport,
    pub embeddings: SourceAnalysisEmbeddingReport,
    pub side_data: SourceAnalysisSideDataReport,
    pub metrics: SourceAnalysisMetricReport,
    pub relationships: SourceAnalysisRelationshipReport,
    pub diagnostics: Value,
    pub skipped_record_count: usize,
    pub skipped_records: Vec<SkippedRecordReport>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisSourceReport {
    pub root: String,
    pub manifest: String,
    pub source_signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisTextReport {
    pub records_with_description: usize,
    pub records_with_blurb: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisEmbeddingReport {
    pub pending_document_embeddings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisSideDataReport {
    pub actor_records: usize,
    pub item_records: usize,
    pub spell_records: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisMetricReport {
    pub metric_rows_by_domain: BTreeMap<String, usize>,
    pub metric_keys_by_domain: BTreeMap<String, usize>,
    pub metric_value_catalog_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisRelationshipReport {
    pub reference_edges: usize,
    pub default_reference_edges: usize,
    pub expanded_reference_edges: usize,
    pub record_aliases: usize,
    pub remaster_links: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkippedRecordReport {
    pub path: String,
    pub reason: String,
}

pub fn analyze_foundry_source(
    source_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<SourceAnalysisReport, IngestError> {
    let source_root = source_root.as_ref();
    let source = source_pipeline::load_foundry_source(source_root, manifest_path)?;
    Ok(analyze_source_load(source_root.to_path_buf(), source))
}

pub(crate) fn analyze_source_load(
    source_root: PathBuf,
    source: SourceLoad,
) -> SourceAnalysisReport {
    let retrieval_visibility = crate::records::visibility::RetrievalVisibility::from_remaster_links(
        &source.remaster_links,
    );
    let default_visible_record_count = source
        .records
        .iter()
        .filter(|loaded| retrieval_visibility.is_default_visible(&loaded.record))
        .count();
    let generated_record_count = source
        .records
        .iter()
        .filter(|record| is_generated_record(record))
        .count();

    SourceAnalysisReport {
        status: "ok",
        source: SourceAnalysisSourceReport {
            root: source_root.display().to_string(),
            manifest: source.manifest_path.display().to_string(),
            source_signature: source.source_signature,
        },
        pack_count: source.packs.len(),
        loaded_source_pack_count: source
            .packs
            .iter()
            .filter(|pack| !pack.declared_path.starts_with("derived://"))
            .count(),
        record_count: source.records.len(),
        loaded_source_record_count: source.records.len() - generated_record_count,
        generated_record_count,
        default_visible_record_count,
        hidden_record_count: source.records.len() - default_visible_record_count,
        by_record_family: count_by_record_family(&source.records),
        by_foundry_taxonomy: count_by_foundry_taxonomy(&source.records),
        by_publication_family: count_by_publication_family(&source.records),
        text: SourceAnalysisTextReport {
            records_with_description: source
                .records
                .iter()
                .filter(|loaded| loaded.record.description.is_some())
                .count(),
            records_with_blurb: source
                .records
                .iter()
                .filter(|loaded| loaded.record.blurb.is_some())
                .count(),
        },
        embeddings: SourceAnalysisEmbeddingReport {
            pending_document_embeddings: source.pending_document_embeddings.len(),
        },
        side_data: SourceAnalysisSideDataReport {
            actor_records: source
                .records
                .iter()
                .filter(|loaded| loaded.record.actor_data.is_some())
                .count(),
            item_records: source
                .records
                .iter()
                .filter(|loaded| loaded.record.item_data.is_some())
                .count(),
            spell_records: source
                .records
                .iter()
                .filter(|loaded| loaded.record.spell_data.is_some())
                .count(),
        },
        metrics: metrics_report(&source.records, &retrieval_visibility),
        relationships: SourceAnalysisRelationshipReport {
            reference_edges: source.references.len(),
            default_reference_edges: reference_edge_count(
                &source.references,
                ReferenceGraphMode::Default,
            ),
            expanded_reference_edges: reference_edge_count(
                &source.references,
                ReferenceGraphMode::AllVisible,
            ),
            record_aliases: source.aliases.len(),
            remaster_links: source.remaster_links.len(),
        },
        diagnostics: diagnostics_json(&source.diagnostics),
        skipped_record_count: source.skipped_records.len(),
        skipped_records: skipped_record_reports(&source.skipped_records),
        warnings: source.warnings,
    }
}

fn reference_edge_count(
    references: &[atlas_record::ReferenceEdge],
    mode: ReferenceGraphMode,
) -> usize {
    references
        .iter()
        .filter(|reference| {
            reference_edge_matches_mode(
                ReferenceEdgeFacts {
                    source_kind: reference.source_kind,
                    visibility: reference.visibility,
                },
                mode,
            )
        })
        .count()
}

pub(crate) fn diagnostics_json(diagnostics: &IngestDiagnostics) -> Value {
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
        },
        "dropped_inline_macros": diagnostics.dropped_inline_macros.iter().map(|(name, diagnostic)| {
            json!({
                "name": name,
                "count": diagnostic.count,
                "examples": diagnostic.examples,
            })
        }).collect::<Vec<_>>(),
    })
}

pub(crate) fn skipped_records_json(skipped_records: &[SkippedRecord]) -> Vec<Value> {
    skipped_record_reports(skipped_records)
        .into_iter()
        .map(|record| json!(record))
        .collect()
}

pub fn build_artifact_json(report: &BuildArtifactReport) -> Value {
    json!({
        "status": "ok",
        "output": report.output_path.display().to_string(),
        "pack_count": report.pack_count,
        "record_count": report.record_count,
        "source_record_count": report.source_record_count,
        "artifact_record_count": report.artifact_record_count,
        "generated_record_count": report.generated_record_count,
        "pending_document_embedding_count": report.pending_document_embedding_count,
        "document_embedding_count": report.document_embedding_count,
        "reused_document_embedding_count": report.reused_document_embedding_count,
        "generated_document_embedding_count": report.generated_document_embedding_count,
        "document_embedding_tokenization": document_embedding_tokenization_json(
            &report.document_embedding_tokenization,
        ),
        "embedding_timing": {
            "tokenization_duration_ms": report.embedding_timing.tokenization_duration_ms,
            "model_load_duration_ms": report.embedding_timing.model_load_duration_ms,
            "generation_duration_ms": report.embedding_timing.generation_duration_ms,
            "batch_count": report.embedding_timing.batch_count,
            "batch_duration_min_ms": report.embedding_timing.batch_duration_min_ms,
            "batch_duration_p50_ms": report.embedding_timing.batch_duration_p50_ms,
            "batch_duration_p95_ms": report.embedding_timing.batch_duration_p95_ms,
            "batch_duration_max_ms": report.embedding_timing.batch_duration_max_ms,
        },
        "build_duration_ms": report.build_duration_ms,
        "source_signature": report.source_signature,
        "diagnostics": diagnostics_json(&report.diagnostics),
        "skipped_record_count": report.skipped_records.len(),
        "skipped_records": skipped_records_json(&report.skipped_records),
        "warnings": report.warnings,
    })
}

fn document_embedding_tokenization_json(telemetry: &DocumentEmbeddingTokenizationReport) -> Value {
    json!({
        "document_count": telemetry.document_count,
        "truncated_document_count": telemetry.truncated_document_count,
        "max_token_count": telemetry.max_token_count,
        "max_observed_token_count": telemetry.max_observed_token_count,
        "total_observed_token_count": telemetry.total_observed_token_count,
        "total_tokens_over_limit": telemetry.total_tokens_over_limit,
        "unit_kind_truncations": telemetry.unit_kind_truncations
            .iter()
            .map(|truncation| {
                json!({
                    "unit_kind": truncation.unit_kind,
                    "unit_count": truncation.unit_count,
                    "record_count": truncation.record_count,
                    "total_tokens_over_limit": truncation.total_tokens_over_limit,
                    "max_observed_token_count": truncation.max_observed_token_count,
                    "examples": truncation.examples
                        .iter()
                        .map(truncation_example_json)
                        .collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
        "record_truncation_coverage": {
            "record_count": telemetry.record_truncation_coverage.record_count,
            "records_with_child_units": telemetry.record_truncation_coverage.records_with_child_units,
            "records_with_any_truncated_unit": telemetry.record_truncation_coverage.records_with_any_truncated_unit,
            "records_with_truncated_parent_unit": telemetry.record_truncation_coverage.records_with_truncated_parent_unit,
            "records_with_truncated_child_unit": telemetry.record_truncation_coverage.records_with_truncated_child_unit,
            "records_with_truncated_parent_and_child_units": telemetry.record_truncation_coverage.records_with_truncated_parent_and_child_units,
            "records_with_truncated_parent_and_all_child_units_fit": telemetry.record_truncation_coverage.records_with_truncated_parent_and_all_child_units_fit,
            "records_with_truncated_parent_without_child_units": telemetry.record_truncation_coverage.records_with_truncated_parent_without_child_units,
        },
        "section_truncations": telemetry.section_truncations
            .iter()
            .map(|section| {
                json!({
                    "section": section.section,
                    "document_count": section.document_count,
                    "dropped_chunk_count": section.dropped_chunk_count,
                })
            })
            .collect::<Vec<_>>(),
        "truncated_examples": telemetry.truncated_examples
            .iter()
            .map(truncation_example_json)
            .collect::<Vec<_>>(),
    })
}

fn truncation_example_json(example: &DocumentEmbeddingTruncationExampleReport) -> Value {
    json!({
        "embedding_unit_key": example.embedding_unit_key,
        "record_key": example.record_key,
        "unit_kind": example.unit_kind,
        "label": example.label,
        "token_count": example.token_count,
        "max_token_count": example.max_token_count,
        "truncated_sections": example.truncated_sections,
    })
}

fn skipped_record_reports(skipped_records: &[SkippedRecord]) -> Vec<SkippedRecordReport> {
    skipped_records
        .iter()
        .map(|record| SkippedRecordReport {
            path: record.path.display().to_string(),
            reason: record.reason.clone(),
        })
        .collect()
}

fn count_by_record_family(records: &[LoadedSourceRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for loaded in records {
        let record = &loaded.record;
        *counts
            .entry(record.record_family.as_str().to_string())
            .or_insert(0) += 1;
    }
    counts
}

fn count_by_foundry_taxonomy(records: &[LoadedSourceRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for loaded in records {
        let record = &loaded.record;
        *counts
            .entry(format!(
                "{}|{}",
                record.foundry_document_type, record.foundry_record_type
            ))
            .or_insert(0) += 1;
    }
    counts
}

fn count_by_publication_family(records: &[LoadedSourceRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for loaded in records {
        let record = &loaded.record;
        *counts
            .entry(publication_family_label(record.publication_family).to_string())
            .or_insert(0) += 1;
    }
    counts
}

fn metrics_report(
    records: &[LoadedSourceRecord],
    retrieval_visibility: &crate::records::visibility::RetrievalVisibility,
) -> SourceAnalysisMetricReport {
    let mut rows_by_domain = BTreeMap::<String, usize>::new();
    let mut keys_by_domain = BTreeMap::<String, BTreeSet<String>>::new();
    let mut text_boolean_values = BTreeSet::<(String, String, String, String)>::new();
    for loaded in records {
        let record = &loaded.record;
        let is_default_visible = retrieval_visibility.is_default_visible(record);
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

    SourceAnalysisMetricReport {
        metric_rows_by_domain: rows_by_domain,
        metric_keys_by_domain: keys_by_domain,
        metric_value_catalog_rows: text_boolean_values.len(),
    }
}

fn is_generated_record(loaded: &LoadedSourceRecord) -> bool {
    let record = &loaded.record;
    matches!(
        record.pack_name.as_str(),
        "derived-afflictions" | "derived-affliction-instances"
    )
}

fn publication_family_label(family: PublicationFamily) -> &'static str {
    family.as_str()
}

fn metric_domain_label(domain: MetricDomain) -> &'static str {
    domain.as_str()
}
