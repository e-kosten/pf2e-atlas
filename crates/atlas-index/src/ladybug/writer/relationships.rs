use std::collections::BTreeSet;
use std::path::Path;

use crate::IndexBuildInput;
use arrow_schema::DataType;
use atlas_record::{
    ContentBlock, ContentDocument, ContentReference, ContentReferenceLocator,
    iter_content_references, render_plain_text,
};

use crate::IndexWriteError;
use crate::ladybug::writer::embeddings::LadybugEmbedding;
use crate::ladybug::writer::evidence::evidence_units;
use crate::ladybug::writer::facts::{
    alias_key, filter_value_key, metric_key_id, publication_key, record_filter_values,
};
use crate::ladybug::writer::parquet::{
    arrow_field, arrow_optional_bools, arrow_optional_floats, arrow_optional_strings,
    arrow_strings, write_parquet,
};
use crate::writer_visibility::RetrievalVisibility;
use atlas_record::MetricValue;

pub(crate) fn write_graph_relationship_parquet(
    staging_path: &Path,
    input: &IndexBuildInput<'_>,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IndexWriteError> {
    let retrieval_visibility = RetrievalVisibility::from_remaster_links(input.remaster_links);
    let records = input.records.clone();

    write_parquet(
        &staging_path.join("from_pack.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(records.iter().map(|record| record.key.to_string())),
            arrow_strings(records.iter().map(|record| record.pack_name.to_string())),
        ],
    )?;

    let publication_relationships = records
        .iter()
        .filter_map(|record| {
            record
                .publication_title
                .as_ref()
                .map(|title| (record.key.to_string(), publication_key(title)))
        })
        .collect::<Vec<_>>();
    write_parquet(
        &staging_path.join("published_in.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                publication_relationships
                    .iter()
                    .map(|(record_key, _)| record_key.clone()),
            ),
            arrow_strings(
                publication_relationships
                    .iter()
                    .map(|(_, publication_key)| publication_key.clone()),
            ),
        ],
    )?;

    let visible_records = records
        .iter()
        .filter(|record| retrieval_visibility.is_default_visible(record))
        .copied()
        .collect::<Vec<_>>();
    write_parquet(
        &staging_path.join("has_search_document.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(visible_records.iter().map(|record| record.key.to_string())),
            arrow_strings(
                visible_records
                    .iter()
                    .map(|record| format!("{}#fts", record.key)),
            ),
        ],
    )?;

    let mut content_from = Vec::new();
    let mut content_to = Vec::new();
    for record in &records {
        for ordinal in 0..record.supplemental_content.len() {
            content_from.push(record.key.to_string());
            content_to.push(format!("{}#content#{ordinal}", record.key));
        }
    }
    write_parquet(
        &staging_path.join("has_content_unit.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![arrow_strings(content_from), arrow_strings(content_to)],
    )?;

    let evidence_units = evidence_units(&records);
    write_parquet(
        &staging_path.join("has_evidence_unit.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(evidence_units.iter().map(|unit| unit.record_key.clone())),
            arrow_strings(
                evidence_units
                    .iter()
                    .map(|unit| unit.evidence_unit_key.clone()),
            ),
        ],
    )?;

    let mut content_reference_from = Vec::new();
    let mut content_reference_to = Vec::new();
    let mut content_reference_display_texts = Vec::new();
    let mut content_reference_texts = Vec::new();
    let mut content_reference_source_kinds = Vec::new();
    let mut content_reference_visibilities = Vec::new();
    let mut content_reference_seen = BTreeSet::new();
    for record in &records {
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            if !supplemental.contributes_to_references {
                continue;
            }
            let content_unit_key = format!("{}#content#{ordinal}", record.key);
            for reference in iter_content_references(&supplemental.document) {
                let Some(to_record_key) = &reference.resolved_key else {
                    continue;
                };
                let reference_text = content_reference_text(reference);
                let dedupe_key = (
                    content_unit_key.clone(),
                    to_record_key.to_string(),
                    reference_text.clone(),
                    supplemental.source_kind.as_str().to_string(),
                );
                if content_reference_seen.insert(dedupe_key) {
                    content_reference_from.push(content_unit_key.clone());
                    content_reference_to.push(to_record_key.to_string());
                    content_reference_display_texts.push(content_reference_display_text(reference));
                    content_reference_texts.push(reference_text);
                    content_reference_source_kinds
                        .push(supplemental.source_kind.as_str().to_string());
                    content_reference_visibilities
                        .push(supplemental.visibility.as_str().to_string());
                }
            }
        }
    }
    write_parquet(
        &staging_path.join("content_references.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("display_text", DataType::Utf8, true),
            arrow_field("reference_text", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("visibility", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(content_reference_from),
            arrow_strings(content_reference_to),
            arrow_optional_strings(content_reference_display_texts),
            arrow_strings(content_reference_texts),
            arrow_strings(content_reference_source_kinds),
            arrow_strings(content_reference_visibilities),
        ],
    )?;

    let mut evidence_reference_from = Vec::new();
    let mut evidence_reference_to = Vec::new();
    let mut evidence_reference_display_texts = Vec::new();
    let mut evidence_reference_texts = Vec::new();
    let mut evidence_reference_source_kinds = Vec::new();
    let mut evidence_reference_visibilities = Vec::new();
    let mut evidence_reference_seen = BTreeSet::new();
    for unit in &evidence_units {
        for reference in iter_content_references(&unit.document) {
            let Some(to_record_key) = &reference.resolved_key else {
                continue;
            };
            let reference_text = content_reference_text(reference);
            let dedupe_key = (
                unit.evidence_unit_key.clone(),
                to_record_key.to_string(),
                reference_text.clone(),
                unit.source_kind.as_str().to_string(),
            );
            if evidence_reference_seen.insert(dedupe_key) {
                evidence_reference_from.push(unit.evidence_unit_key.clone());
                evidence_reference_to.push(to_record_key.to_string());
                evidence_reference_display_texts.push(content_reference_display_text(reference));
                evidence_reference_texts.push(reference_text);
                evidence_reference_source_kinds.push(unit.source_kind.as_str().to_string());
                evidence_reference_visibilities.push(unit.visibility.as_str().to_string());
            }
        }
    }
    write_parquet(
        &staging_path.join("evidence_references.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("display_text", DataType::Utf8, true),
            arrow_field("reference_text", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("visibility", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(evidence_reference_from),
            arrow_strings(evidence_reference_to),
            arrow_optional_strings(evidence_reference_display_texts),
            arrow_strings(evidence_reference_texts),
            arrow_strings(evidence_reference_source_kinds),
            arrow_strings(evidence_reference_visibilities),
        ],
    )?;

    let embedding_unit_keys = embeddings
        .iter()
        .map(|embedding| embedding.embedding_unit_key.as_str())
        .collect::<BTreeSet<_>>();
    let evidence_embedding_units = evidence_units
        .iter()
        .filter(|unit| embedding_unit_keys.contains(unit.evidence_unit_key.as_str()))
        .collect::<Vec<_>>();
    write_parquet(
        &staging_path.join("has_evidence_embedding.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                evidence_embedding_units
                    .iter()
                    .map(|unit| unit.evidence_unit_key.clone()),
            ),
            arrow_strings(
                evidence_embedding_units
                    .iter()
                    .map(|unit| unit.evidence_unit_key.clone()),
            ),
        ],
    )?;

    let mut trait_from = Vec::new();
    let mut trait_to = Vec::new();
    for record in &records {
        for trait_name in &record.traits {
            trait_from.push(record.key.to_string());
            trait_to.push(trait_name.clone());
        }
    }
    write_parquet(
        &staging_path.join("has_trait.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![arrow_strings(trait_from), arrow_strings(trait_to)],
    )?;

    let mut filter_value_from = Vec::new();
    let mut filter_value_to = Vec::new();
    for record in &records {
        for (field, value) in record_filter_values(record) {
            filter_value_from.push(record.key.to_string());
            filter_value_to.push(filter_value_key(field, &value));
        }
    }
    write_parquet(
        &staging_path.join("has_filter_value.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(filter_value_from),
            arrow_strings(filter_value_to),
        ],
    )?;

    write_parquet(
        &staging_path.join("has_alias.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                input
                    .aliases
                    .iter()
                    .map(|alias| alias.canonical_record_key.to_string()),
            ),
            arrow_strings(input.aliases.iter().map(alias_key)),
        ],
    )?;

    let mut metric_from = Vec::new();
    let mut metric_to = Vec::new();
    let mut metric_number_values = Vec::new();
    let mut metric_text_values = Vec::new();
    let mut metric_bool_values = Vec::new();
    for record in &records {
        for metric in &record.metrics {
            metric_from.push(record.key.to_string());
            metric_to.push(metric_key_id(metric.domain.as_str(), &metric.key));
            match &metric.value {
                MetricValue::Number(value) => {
                    metric_number_values.push(Some(*value));
                    metric_text_values.push(None);
                    metric_bool_values.push(None);
                }
                MetricValue::Text(value) => {
                    metric_number_values.push(None);
                    metric_text_values.push(Some(value.clone()));
                    metric_bool_values.push(None);
                }
                MetricValue::Boolean(value) => {
                    metric_number_values.push(None);
                    metric_text_values.push(None);
                    metric_bool_values.push(Some(*value));
                }
            }
        }
    }
    write_parquet(
        &staging_path.join("has_metric.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("number_value", DataType::Float64, true),
            arrow_field("text_value", DataType::Utf8, true),
            arrow_field("bool_value", DataType::Boolean, true),
        ],
        vec![
            arrow_strings(metric_from),
            arrow_strings(metric_to),
            arrow_optional_floats(metric_number_values),
            arrow_optional_strings(metric_text_values),
            arrow_optional_bools(metric_bool_values),
        ],
    )?;

    write_parquet(
        &staging_path.join("has_embedding_unit.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                embeddings
                    .iter()
                    .map(|embedding| embedding.record_key.clone()),
            ),
            arrow_strings(
                embeddings
                    .iter()
                    .map(|embedding| embedding.embedding_unit_key.clone()),
            ),
        ],
    )?;

    write_parquet(
        &staging_path.join("references.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("display_text", DataType::Utf8, true),
            arrow_field("reference_text", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("visibility", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                input
                    .references
                    .iter()
                    .map(|reference| reference.from_record_key.to_string()),
            ),
            arrow_strings(
                input
                    .references
                    .iter()
                    .map(|reference| reference.to_record_key.to_string()),
            ),
            arrow_optional_strings(
                input
                    .references
                    .iter()
                    .map(|reference| reference.display_text.clone()),
            ),
            arrow_strings(
                input
                    .references
                    .iter()
                    .map(|reference| reference.reference_text.clone()),
            ),
            arrow_strings(
                input
                    .references
                    .iter()
                    .map(|reference| reference.source_kind.as_str().to_string()),
            ),
            arrow_strings(
                input
                    .references
                    .iter()
                    .map(|reference| reference.visibility.as_str().to_string()),
            ),
        ],
    )?;

    write_parquet(
        &staging_path.join("remastered_by.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("source_ref", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                input
                    .remaster_links
                    .iter()
                    .map(|link| link.legacy_record_key.to_string()),
            ),
            arrow_strings(
                input
                    .remaster_links
                    .iter()
                    .map(|link| link.remaster_record_key.to_string()),
            ),
            arrow_strings(
                input
                    .remaster_links
                    .iter()
                    .map(|link| link.source.as_str().to_string()),
            ),
            arrow_strings(
                input
                    .remaster_links
                    .iter()
                    .map(|link| link.source_ref.clone()),
            ),
        ],
    )?;

    let mut variant_from = Vec::new();
    let mut variant_to = Vec::new();
    let mut variant_labels = Vec::new();
    let mut variant_axes_jsons = Vec::new();
    for record in &records {
        if let Some(key) = &record.variant_group_key {
            variant_from.push(record.key.to_string());
            variant_to.push(key.clone());
            variant_labels.push(record.variant_label.clone());
            variant_axes_jsons.push(
                serde_json::to_string(&record.variant_axes)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
            );
        }
    }
    write_parquet(
        &staging_path.join("in_variant_group.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("variant_label", DataType::Utf8, true),
            arrow_field("variant_axes_json", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(variant_from),
            arrow_strings(variant_to),
            arrow_optional_strings(variant_labels),
            arrow_strings(variant_axes_jsons),
        ],
    )?;

    Ok(())
}

fn content_reference_text(reference: &ContentReference) -> String {
    match &reference.locator {
        ContentReferenceLocator::FoundryUuid { raw_target }
        | ContentReferenceLocator::Compendium { raw_target } => raw_target.clone(),
        ContentReferenceLocator::PackAndLocator { pack_name, locator } => {
            format!("{pack_name}:{locator}")
        }
        ContentReferenceLocator::Unknown { raw } => raw.clone(),
    }
}

fn content_reference_display_text(reference: &ContentReference) -> Option<String> {
    reference
        .label
        .as_ref()
        .map(|label| {
            render_plain_text(&ContentDocument::new(vec![ContentBlock::Paragraph {
                content: label.clone(),
            }]))
        })
        .filter(|label| !label.trim().is_empty())
}
