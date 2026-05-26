use atlas_domain::{
    FilterFieldStats, FilterValueCount, MetricKeyDiscovery, NumericFieldStats, PackName,
    PublicationFamily, RecordFamily, RecordId, RecordKey, RemasterLinkSource, TimeKind, TimeUnit,
};
use atlas_record::{
    ActorSideData, AliasSource, ContentDocument, ContentSourceKind, ContentVisibility,
    ItemSideData, NormalizedTime, PersistedRecord, RecordAlias, ReferenceEdge, RemasterLink,
    SpellSideData,
};
use lbug::{Connection, Value};
use std::time::Instant;

use crate::{DiscoveryError, GraphReferenceEdge, RecordEmbeddingUnit, VectorSearchHit};

use super::{LadybugIndexReaderError, invalid, trace_ladybug_phase};

pub(crate) fn record_from_row(row: &[Value]) -> Result<PersistedRecord, LadybugIndexReaderError> {
    let key = record_key_at(row, 0)?;
    let id = RecordId::new(string_at(row, 1)?).map_err(invalid)?;
    let record_family = string_at(row, 4)?
        .parse::<RecordFamily>()
        .map_err(invalid)?;
    let pack_name = PackName::new(string_at(row, 5)?).map_err(invalid)?;
    let publication_family = PublicationFamily::from_canonical(&string_at(row, 32)?)
        .unwrap_or(PublicationFamily::Unknown);
    let actor_data = optional_actor_data(row)?;
    let item_data = optional_item_data(row)?;
    let spell_data = optional_spell_data(row)?;
    Ok(PersistedRecord {
        key,
        id,
        name: string_at(row, 2)?,
        normalized_name: string_at(row, 3)?,
        record_family,
        pack_name,
        pack_label: string_at(row, 6)?,
        foundry_document_type: string_at(row, 7)?,
        foundry_record_type: string_at(row, 8)?,
        level: optional_i64_at(row, 9)?,
        rarity: optional_string_at(row, 10)?,
        traits: json_string_array_at(row, 11)?,
        prerequisites: json_string_array_at(row, 12)?,
        system_category: optional_string_at(row, 13)?,
        system_group: optional_string_at(row, 14)?,
        system_base_item: optional_string_at(row, 15)?,
        system_usage: optional_string_at(row, 16)?,
        system_price_json: optional_string_at(row, 17)?,
        system_actions_value: optional_i64_at(row, 18)?,
        system_time_value: optional_string_at(row, 19)?,
        system_duration_value: optional_string_at(row, 20)?,
        price_cp: optional_i64_at(row, 21)?,
        activation_time: normalized_time_at(row, 22, 23, 24, 25, 26)?,
        duration: normalized_time_at(row, 27, usize::MAX, 28, 29, 30)?,
        metrics: Vec::new(),
        actor_data,
        item_data,
        spell_data,
        publication_title: optional_string_at(row, 31)?,
        publication_remaster: bool_at(row, 33)?,
        description: optional_content_document_at(row, 34)?,
        blurb: optional_content_document_at(row, 35)?,
        supplemental_content: Vec::new(),
        publication_family,
        folder_id: optional_string_at(row, 36)?,
        taxonomy_families: json_string_array_at(row, 37)?,
        variant_group_key: optional_string_at(row, 38)?,
        variant_base_name: optional_string_at(row, 39)?,
        variant_label: optional_string_at(row, 40)?,
        variant_axes: json_string_array_at(row, 41)?,
        variant_confidence: optional_float_at(row, 42)?,
        variant_source: string_at(row, 43)?,
        is_default_visible: bool_at(row, 44)?,
        source_path: string_at(row, 45)?,
        raw_json: string_at(row, 46)?,
    })
}

pub(crate) fn alias_from_row(row: &[Value]) -> Result<RecordAlias, LadybugIndexReaderError> {
    Ok(RecordAlias {
        canonical_record_key: record_key_at(row, 0)?,
        alias_text: string_at(row, 1)?,
        normalized_alias: string_at(row, 2)?,
        source: AliasSource::from_canonical(&string_at(row, 3)?)
            .unwrap_or(AliasSource::CompendiumSource),
        source_ref: string_at(row, 4)?,
    })
}

pub(crate) fn graph_edge_from_row(
    row: &[Value],
    from_index: usize,
    to_index: usize,
) -> Result<GraphReferenceEdge, LadybugIndexReaderError> {
    let source_kind = ContentSourceKind::from_canonical(&string_at(row, 4)?)
        .unwrap_or(ContentSourceKind::Description);
    let visibility =
        ContentVisibility::from_canonical(&string_at(row, 5)?).unwrap_or(ContentVisibility::Public);
    Ok(GraphReferenceEdge {
        from_record_key: record_key_at(row, from_index)?,
        to_record_key: record_key_at(row, to_index)?,
        display_text: optional_string_at(row, 2)?,
        reference_text: string_at(row, 3)?,
        source_kind,
        visibility,
    })
}

pub(crate) fn reference_edge_from_row(
    row: &[Value],
) -> Result<ReferenceEdge, LadybugIndexReaderError> {
    Ok(ReferenceEdge {
        from_record_key: record_key_at(row, 0)?,
        to_record_key: record_key_at(row, 1)?,
        display_text: optional_string_at(row, 2)?,
        reference_text: string_at(row, 3)?,
        source_kind: ContentSourceKind::from_canonical(&string_at(row, 4)?)
            .unwrap_or(ContentSourceKind::Description),
        visibility: ContentVisibility::from_canonical(&string_at(row, 5)?)
            .unwrap_or(ContentVisibility::Public),
    })
}

pub(crate) fn remaster_link_from_row(
    row: &[Value],
) -> Result<RemasterLink, LadybugIndexReaderError> {
    Ok(RemasterLink {
        remaster_record_key: record_key_at(row, 0)?,
        legacy_record_key: record_key_at(row, 1)?,
        source: RemasterLinkSource::from_canonical(&string_at(row, 2)?)
            .unwrap_or(RemasterLinkSource::Migration),
        source_ref: string_at(row, 3)?,
    })
}

pub(crate) fn vector_hit_from_row(
    row: &[Value],
) -> Result<VectorSearchHit, LadybugIndexReaderError> {
    Ok(VectorSearchHit {
        record_key: string_at(row, 0)?,
        embedding_unit_key: string_at(row, 1)?,
        unit_kind: string_at(row, 2)?,
        label: optional_string_at(row, 3)?,
        distance: float_at(row, 4)?,
    })
}

pub(crate) fn record_embedding_unit_from_row(
    row: &[Value],
) -> Result<RecordEmbeddingUnit, LadybugIndexReaderError> {
    let embedding_unit_key = string_at(row, 0)?;
    Ok(RecordEmbeddingUnit {
        embedding_unit_key: embedding_unit_key.clone(),
        record_key: record_key_at(row, 1)?,
        unit_kind: string_at(row, 2)?,
        label: optional_string_at(row, 3)?,
        ordinal: int_at(row, 4)?,
        vector: vector_at(row, 5, &embedding_unit_key)?,
    })
}

pub(crate) fn vector_at(
    row: &[Value],
    index: usize,
    context: &str,
) -> Result<Vec<f32>, LadybugIndexReaderError> {
    let values = match row.get(index) {
        Some(Value::Array(_, values) | Value::List(_, values)) => values,
        Some(value) => {
            return Err(LadybugIndexReaderError::InvalidData(format!(
                "`{context}` vector column {index} had unexpected value {value:?}"
            )));
        }
        None => {
            return Err(LadybugIndexReaderError::InvalidData(format!(
                "`{context}` vector column {index} is missing"
            )));
        }
    };
    values
        .iter()
        .map(|value| match value {
            Value::Float(value) => Ok(*value),
            Value::Double(value) => Ok(*value as f32),
            other => Err(LadybugIndexReaderError::InvalidData(format!(
                "`{context}` vector column {index} contains non-float value {other:?}"
            ))),
        })
        .collect()
}

pub(crate) fn query_rows(
    connection: &Connection<'_>,
    sql: &str,
) -> Result<Vec<Vec<Value>>, LadybugIndexReaderError> {
    query_rows_impl(connection, sql, None)
}

pub(crate) fn query_rows_traced(
    connection: &Connection<'_>,
    sql: &str,
    phase: &str,
) -> Result<Vec<Vec<Value>>, LadybugIndexReaderError> {
    query_rows_impl(connection, sql, Some(phase))
}

fn query_rows_impl(
    connection: &Connection<'_>,
    sql: &str,
    phase: Option<&str>,
) -> Result<Vec<Vec<Value>>, LadybugIndexReaderError> {
    let started_at = Instant::now();
    let mut result = connection
        .query(sql)
        .map_err(|error| LadybugIndexReaderError::Query(format!("{error}; query: {sql}")))?;
    if let Some(phase) = phase {
        trace_ladybug_phase(&format!("{phase}_execute"), started_at);
    }
    let started_at = Instant::now();
    let mut rows = Vec::new();
    for row in &mut result {
        rows.push(row.to_vec());
    }
    if let Some(phase) = phase {
        trace_ladybug_phase(&format!("{phase}_materialize"), started_at);
    }
    Ok(rows)
}

pub(crate) fn record_key_at(
    row: &[Value],
    index: usize,
) -> Result<RecordKey, LadybugIndexReaderError> {
    RecordKey::parse(&string_at(row, index)?).map_err(invalid)
}

pub(crate) fn string_at(row: &[Value], index: usize) -> Result<String, LadybugIndexReaderError> {
    match row.get(index) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(value) => Err(LadybugIndexReaderError::InvalidData(format!(
            "expected string at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn optional_string_at(
    row: &[Value],
    index: usize,
) -> Result<Option<String>, LadybugIndexReaderError> {
    match row.get(index) {
        Some(Value::Null(_)) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(value) => Err(LadybugIndexReaderError::InvalidData(format!(
            "expected optional string at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn int_at(row: &[Value], index: usize) -> Result<i64, LadybugIndexReaderError> {
    match row.get(index) {
        Some(Value::Int64(value)) => Ok(*value),
        Some(Value::Int32(value)) => Ok(i64::from(*value)),
        Some(value) => Err(LadybugIndexReaderError::InvalidData(format!(
            "expected integer at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn u64_at(row: &[Value], index: usize) -> Result<u64, LadybugIndexReaderError> {
    let value = int_at(row, index)?;
    u64::try_from(value).map_err(|_| {
        LadybugIndexReaderError::InvalidData(format!(
            "expected non-negative integer at column {index}, got {value}"
        ))
    })
}

pub(crate) fn optional_i64_at(
    row: &[Value],
    index: usize,
) -> Result<Option<i64>, LadybugIndexReaderError> {
    if index == usize::MAX {
        return Ok(None);
    }
    match row.get(index) {
        Some(Value::Null(_)) => Ok(None),
        Some(_) => int_at(row, index).map(Some),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn optional_bool_at(
    row: &[Value],
    index: usize,
) -> Result<Option<bool>, LadybugIndexReaderError> {
    match row.get(index) {
        Some(Value::Null(_)) => Ok(None),
        Some(_) => bool_at(row, index).map(Some),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn bool_at(row: &[Value], index: usize) -> Result<bool, LadybugIndexReaderError> {
    match row.get(index) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(value) => Err(LadybugIndexReaderError::InvalidData(format!(
            "expected bool at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn float_at(row: &[Value], index: usize) -> Result<f64, LadybugIndexReaderError> {
    match row.get(index) {
        Some(Value::Double(value)) => Ok(*value),
        Some(Value::Float(value)) => Ok(f64::from(*value)),
        Some(Value::Int64(value)) => Ok(*value as f64),
        Some(Value::Int32(value)) => Ok(f64::from(*value)),
        Some(value) => Err(LadybugIndexReaderError::InvalidData(format!(
            "expected float at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn value_to_discovery_string(
    row: &[Value],
    index: usize,
) -> Result<String, LadybugIndexReaderError> {
    match row.get(index) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(Value::Bool(value)) => Ok(value.to_string()),
        Some(Value::Int64(value)) => Ok(value.to_string()),
        Some(Value::Int32(value)) => Ok(value.to_string()),
        Some(Value::Double(value)) => Ok(value.to_string()),
        Some(Value::Float(value)) => Ok(value.to_string()),
        Some(value) => Err(LadybugIndexReaderError::InvalidData(format!(
            "expected scalar discovery value at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn discovery_error(error: LadybugIndexReaderError) -> DiscoveryError {
    DiscoveryError::QueryFailed(error.to_string())
}

pub(crate) fn unknown_discovery_field(field: &str) -> DiscoveryError {
    let suggestion = match field {
        "packs" | "pack" => " Did you mean `pack_name` or `pack_label`?",
        "sources" | "source" => " Did you mean `publication_title`?",
        "actorMetrics" | "itemMetrics" | "actor_metrics" | "item_metrics" => {
            " Did you mean `metric`?"
        }
        _ => "",
    };
    DiscoveryError::InvalidField(format!(
        "unknown filter field `{field}`.{suggestion} Run `atlas filters fields` to discover supported fields."
    ))
}

pub(crate) fn filter_field_stats(values: &[FilterValueCount]) -> FilterFieldStats {
    let value_count = values.iter().map(|value| value.count).sum::<u64>();
    let distinct_count = values.len() as u64;
    let singleton_count = values.iter().filter(|value| value.count == 1).count() as u64;
    FilterFieldStats {
        value_count,
        null_count: 0,
        distinct_count,
        singleton_count,
        singleton_ratio: if distinct_count == 0 {
            0.0
        } else {
            singleton_count as f64 / distinct_count as f64
        },
        observation_singleton_ratio: if value_count == 0 {
            0.0
        } else {
            singleton_count as f64 / value_count as f64
        },
    }
}

pub(crate) fn numeric_stats_from_values(
    values: &[f64],
    matching_record_count: u64,
) -> NumericFieldStats {
    let count = values.len() as u64;
    NumericFieldStats {
        count,
        null_count: matching_record_count.saturating_sub(count),
        min: values.first().copied(),
        p05: percentile(values, 0.05),
        p25: percentile(values, 0.25),
        p50: percentile(values, 0.50),
        mean: (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64),
        p75: percentile(values, 0.75),
        p95: percentile(values, 0.95),
        max: values.last().copied(),
    }
}

fn percentile(values: &[f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let rank = ((values.len() as f64) * percentile).ceil() as usize;
    let index = rank.saturating_sub(1).min(values.len() - 1);
    values.get(index).copied()
}

pub(crate) fn resolve_ladybug_metric_from_candidates(
    metrics: Vec<MetricKeyDiscovery>,
    value: &str,
) -> Result<MetricKeyDiscovery, DiscoveryError> {
    let key_matches = metrics
        .iter()
        .filter(|metric| metric.metric_key == value)
        .cloned()
        .collect::<Vec<_>>();
    match key_matches.as_slice() {
        [metric] => return Ok(metric.clone()),
        [] => {}
        _ => {
            return Err(DiscoveryError::AmbiguousMetric(format!(
                "metric key `{value}` is ambiguous; candidates: {}",
                ladybug_metric_candidates(&key_matches)
            )));
        }
    }

    let normalized = normalize_ladybug_metric_label(value);
    let label_matches = metrics
        .into_iter()
        .filter(|metric| {
            metric.known
                && (ladybug_metric_label_matches(metric.label.as_deref(), &normalized)
                    || ladybug_metric_label_matches(metric.short_label.as_deref(), &normalized))
        })
        .collect::<Vec<_>>();
    match label_matches.as_slice() {
        [metric] => Ok(metric.clone()),
        [] => Err(DiscoveryError::InvalidOption(format!(
            "metric `{value}` did not match a metric key, exact known label, or exact known short label"
        ))),
        _ => Err(DiscoveryError::AmbiguousMetric(format!(
            "metric label `{value}` is ambiguous; candidates: {}",
            ladybug_metric_candidates(&label_matches)
        ))),
    }
}

fn ladybug_metric_candidates(metrics: &[MetricKeyDiscovery]) -> String {
    metrics
        .iter()
        .map(|metric| {
            format!(
                "{} ({}, {}, {})",
                metric.metric_key, metric.metric_domain, metric.record_family, metric.value_type
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn ladybug_metric_label_matches(value: Option<&str>, normalized: &str) -> bool {
    value
        .map(normalize_ladybug_metric_label)
        .is_some_and(|label| label == normalized)
}

fn normalize_ladybug_metric_label(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}

pub(crate) fn optional_float_at(
    row: &[Value],
    index: usize,
) -> Result<Option<f64>, LadybugIndexReaderError> {
    match row.get(index) {
        Some(Value::Null(_)) => Ok(None),
        Some(_) => float_at(row, index).map(Some),
        None => Err(LadybugIndexReaderError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

pub(crate) fn json_string_array_at(
    row: &[Value],
    index: usize,
) -> Result<Vec<String>, LadybugIndexReaderError> {
    let json = string_at(row, index)?;
    serde_json::from_str::<Vec<String>>(&json).map_err(invalid)
}

pub(crate) fn optional_content_document_at(
    row: &[Value],
    index: usize,
) -> Result<Option<ContentDocument>, LadybugIndexReaderError> {
    optional_string_at(row, index)?
        .map(|json| serde_json::from_str::<ContentDocument>(&json).map_err(invalid))
        .transpose()
}

pub(crate) fn normalized_time_at(
    row: &[Value],
    kind_index: usize,
    actions_index: usize,
    duration_value_index: usize,
    duration_unit_index: usize,
    text_index: usize,
) -> Result<Option<NormalizedTime>, LadybugIndexReaderError> {
    let Some(kind) = optional_string_at(row, kind_index)? else {
        return Ok(None);
    };
    let Some(text) = optional_string_at(row, text_index)? else {
        return Ok(None);
    };
    Ok(Some(NormalizedTime {
        kind: TimeKind::from_canonical(&kind).unwrap_or(TimeKind::Other),
        actions: optional_i64_at(row, actions_index)?,
        duration_value: optional_i64_at(row, duration_value_index)?,
        duration_unit: optional_string_at(row, duration_unit_index)?
            .and_then(|value| TimeUnit::from_canonical(&value)),
        text,
    }))
}

pub(crate) fn optional_actor_data(
    row: &[Value],
) -> Result<Option<ActorSideData>, LadybugIndexReaderError> {
    if optional_string_at(row, 47)?.is_none()
        && optional_string_at(row, 48)?.is_none()
        && optional_string_at(row, 54)?.is_none()
        && optional_bool_at(row, 56)?.is_none()
    {
        return Ok(None);
    }
    Ok(Some(ActorSideData {
        size: optional_string_at(row, 47)?,
        languages: optional_json_string_array_at(row, 48)?,
        speed_types: optional_json_string_array_at(row, 49)?,
        senses: optional_json_string_array_at(row, 50)?,
        immunities: optional_json_string_array_at(row, 51)?,
        resistances: optional_json_string_array_at(row, 52)?,
        weaknesses: optional_json_string_array_at(row, 53)?,
        disable_text: optional_string_at(row, 54)?,
        disable_skills: optional_json_string_array_at(row, 55)?,
        is_complex: optional_bool_at(row, 56)?.unwrap_or(false),
    }))
}

pub(crate) fn optional_item_data(
    row: &[Value],
) -> Result<Option<ItemSideData>, LadybugIndexReaderError> {
    if optional_float_at(row, 57)?.is_none()
        && optional_string_at(row, 58)?.is_none()
        && optional_string_at(row, 59)?.is_none()
    {
        return Ok(None);
    }
    Ok(Some(ItemSideData {
        system_category: optional_string_at(row, 13)?,
        system_base_item: optional_string_at(row, 15)?,
        system_group: optional_string_at(row, 14)?,
        system_usage: optional_string_at(row, 16)?,
        price_cp: optional_i64_at(row, 21)?,
        bulk_value: optional_float_at(row, 57)?,
        hands_requirement: optional_string_at(row, 58)?,
        damage_types: optional_json_string_array_at(row, 59)?,
    }))
}

pub(crate) fn optional_spell_data(
    row: &[Value],
) -> Result<Option<SpellSideData>, LadybugIndexReaderError> {
    if optional_string_at(row, 60)?.is_none()
        && optional_string_at(row, 61)?.is_none()
        && optional_string_at(row, 62)?.is_none()
    {
        return Ok(None);
    }
    Ok(Some(SpellSideData {
        traditions: optional_json_string_array_at(row, 60)?,
        spell_kinds: optional_json_string_array_at(row, 61)?,
        range_text: optional_string_at(row, 62)?,
        range_value: optional_float_at(row, 63)?,
        target_text: optional_string_at(row, 64)?,
        area_type: optional_string_at(row, 65)?,
        area_value: optional_float_at(row, 66)?,
        save_type: optional_string_at(row, 67)?,
        sustained: optional_bool_at(row, 68)?.unwrap_or(false),
        basic_save: optional_bool_at(row, 69)?.unwrap_or(false),
        damage_types: optional_json_string_array_at(row, 70)?,
    }))
}

pub(crate) fn optional_json_string_array_at(
    row: &[Value],
    index: usize,
) -> Result<Vec<String>, LadybugIndexReaderError> {
    optional_string_at(row, index)?
        .map(|json| serde_json::from_str::<Vec<String>>(&json).map_err(invalid))
        .transpose()
        .map(|value| value.unwrap_or_default())
}
