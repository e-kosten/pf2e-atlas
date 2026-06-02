use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use atlas_domain::{MetricDomain, PublicationCategory};
use atlas_record::{
    MetricValue, ReferenceEdgeFacts, ReferenceGraphMode, reference_edge_matches_mode,
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::diagnostics::IngestDiagnostics;
use crate::error::IngestError;
use crate::records::LoadedSourceRecord;
use crate::source::model::{SkippedRecord, SourceLoad};
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
    pub by_kind: BTreeMap<String, usize>,
    pub by_foundry_taxonomy: BTreeMap<String, usize>,
    pub by_publication_category: BTreeMap<String, usize>,
    pub text: SourceAnalysisTextReport,
    pub embeddings: SourceAnalysisEmbeddingReport,
    pub mechanics: SourceAnalysisMechanicsReport,
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
    pub parent_units: usize,
    pub child_units: usize,
    pub records_with_child_units: usize,
    pub records_over_20_child_units: usize,
    pub records_over_50_child_units: usize,
    pub records_over_100_child_units: usize,
    pub max_child_units_per_record: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAnalysisMechanicsReport {
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
        by_kind: count_by_kind(&source.records),
        by_foundry_taxonomy: count_by_foundry_taxonomy(&source.records),
        by_publication_category: count_by_publication_category(&source.records),
        text: SourceAnalysisTextReport {
            records_with_description: source
                .records
                .iter()
                .filter(|loaded| loaded.record.content.description().is_some())
                .count(),
            records_with_blurb: source
                .records
                .iter()
                .filter(|loaded| loaded.record.content.blurb().is_some())
                .count(),
        },
        embeddings: embedding_report(&source.pending_document_embeddings),
        mechanics: SourceAnalysisMechanicsReport {
            actor_records: source
                .records
                .iter()
                .filter(|loaded| loaded.record.mechanics.actor().is_some())
                .count(),
            item_records: source
                .records
                .iter()
                .filter(|loaded| loaded.record.mechanics.item().is_some())
                .count(),
            spell_records: source
                .records
                .iter()
                .filter(|loaded| loaded.record.mechanics.spell().is_some())
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

fn skipped_record_reports(skipped_records: &[SkippedRecord]) -> Vec<SkippedRecordReport> {
    skipped_records
        .iter()
        .map(|record| SkippedRecordReport {
            path: record.path.display().to_string(),
            reason: record.reason.clone(),
        })
        .collect()
}

fn embedding_report(
    pending: &[atlas_embedding::PendingDocumentEmbedding],
) -> SourceAnalysisEmbeddingReport {
    let summary = crate::embeddings::summarize_pending_document_embeddings(pending);
    SourceAnalysisEmbeddingReport {
        pending_document_embeddings: summary.total_units,
        parent_units: summary.parent_units,
        child_units: summary.child_units,
        records_with_child_units: summary.records_with_child_units,
        records_over_20_child_units: summary.records_over_20_child_units,
        records_over_50_child_units: summary.records_over_50_child_units,
        records_over_100_child_units: summary.records_over_100_child_units,
        max_child_units_per_record: summary.max_child_units_per_record,
    }
}

fn count_by_kind(records: &[LoadedSourceRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for loaded in records {
        let record = &loaded.record;
        *counts
            .entry(record.classification.kind.as_str().to_string())
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
                record.foundry.document_type.as_str(),
                record.foundry.record_type.as_str()
            ))
            .or_insert(0) += 1;
    }
    counts
}

fn count_by_publication_category(records: &[LoadedSourceRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for loaded in records {
        let record = &loaded.record;
        *counts
            .entry(publication_category_label(record.publication.category).to_string())
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
        for metric in &record.mechanics.metrics {
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
                        record.classification.kind.as_str().to_string(),
                        metric.key.clone(),
                        value.clone(),
                    ));
                }
                MetricValue::Boolean(value) => {
                    if is_default_visible {
                        text_boolean_values.insert((
                            domain,
                            record.classification.kind.as_str().to_string(),
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
        record.identity.pack().as_str(),
        "derived-afflictions" | "derived-affliction-instances"
    )
}

fn publication_category_label(category: PublicationCategory) -> &'static str {
    category.as_str()
}

fn metric_domain_label(domain: MetricDomain) -> &'static str {
    domain.as_str()
}
