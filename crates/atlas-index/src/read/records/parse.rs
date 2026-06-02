use atlas_domain::{
    MetricDomain, MetricValueType, PublicationCategory, Rarity, RecordKey, RecordKind,
    RemasterLinkSource, TimeKind, TimeUnit,
};
use atlas_record::{
    AliasSource, ContentDocument, ContentSourceKind, ContentVisibility, NormalizedTime,
    VariantSource,
};

use super::RecordLoadError;

pub(super) fn normalized_time(
    column_family: &'static str,
    kind: Option<String>,
    actions: Option<i64>,
    duration_value: Option<i64>,
    duration_unit: Option<String>,
    text: Option<String>,
) -> Result<Option<NormalizedTime>, RecordLoadError> {
    let Some(kind) = kind else {
        return Ok(None);
    };
    Ok(Some(NormalizedTime {
        kind: parse_time_kind(&kind)?,
        actions,
        duration_value,
        duration_unit: duration_unit.as_deref().map(parse_time_unit).transpose()?,
        text: text.ok_or_else(|| invalid_missing(format!("{column_family}_text")))?,
    }))
}

pub(super) fn json_string_array(
    name: &'static str,
    value: &str,
) -> Result<Vec<String>, RecordLoadError> {
    serde_json::from_str(value).map_err(|error| {
        RecordLoadError::InvalidData(format!("{name} must be a JSON string array: {error}"))
    })
}

pub(super) fn content_document(
    name: &'static str,
    value: &str,
) -> Result<ContentDocument, RecordLoadError> {
    serde_json::from_str(value).map_err(|error| {
        RecordLoadError::InvalidData(format!("{name} must be a content document JSON: {error}"))
    })
}

pub(super) fn parse_record_key(value: &str) -> Result<RecordKey, RecordLoadError> {
    RecordKey::parse(value).map_err(invalid_parse("record_key"))
}

pub(super) fn parse_record_kind(value: &str) -> Result<RecordKind, RecordLoadError> {
    RecordKind::from_canonical(value)
        .ok_or_else(|| invalid_value("records.record_kind", value.to_string()))
}

pub(super) fn parse_rarity(value: &str) -> Result<Rarity, RecordLoadError> {
    Rarity::from_canonical(value).ok_or_else(|| invalid_value("records.rarity", value.to_string()))
}

pub(super) fn parse_publication_family(
    value: &str,
) -> Result<PublicationCategory, RecordLoadError> {
    PublicationCategory::from_canonical(value)
        .ok_or_else(|| invalid_value("records.publication_family", value.to_string()))
}

pub(super) fn parse_metric_domain(value: &str) -> Result<MetricDomain, RecordLoadError> {
    MetricDomain::from_canonical(value)
        .ok_or_else(|| invalid_value("record_metrics.metric_domain", value.to_string()))
}

pub(super) fn parse_metric_value_type(value: &str) -> Result<MetricValueType, RecordLoadError> {
    MetricValueType::from_canonical(value)
        .ok_or_else(|| invalid_value("record_metrics.value_type", value.to_string()))
}

pub(super) fn parse_alias_source(value: &str) -> Result<AliasSource, RecordLoadError> {
    AliasSource::from_canonical(value)
        .ok_or_else(|| invalid_value("record_aliases.source_kind", value.to_string()))
}

pub(super) fn parse_content_source_kind(value: &str) -> Result<ContentSourceKind, RecordLoadError> {
    ContentSourceKind::from_canonical(value)
        .ok_or_else(|| invalid_value("content.source_kind", value.to_string()))
}

pub(super) fn parse_content_visibility(value: &str) -> Result<ContentVisibility, RecordLoadError> {
    ContentVisibility::from_canonical(value)
        .ok_or_else(|| invalid_value("content.visibility", value.to_string()))
}

pub(super) fn parse_remaster_link_source(
    value: &str,
) -> Result<RemasterLinkSource, RecordLoadError> {
    RemasterLinkSource::from_canonical(value)
        .ok_or_else(|| invalid_value("remaster_links.source_kind", value.to_string()))
}

pub(super) fn parse_variant_source(value: &str) -> Result<VariantSource, RecordLoadError> {
    VariantSource::from_canonical(value)
        .ok_or_else(|| invalid_value("records.variant_source", value.to_string()))
}

fn parse_time_kind(value: &str) -> Result<TimeKind, RecordLoadError> {
    TimeKind::from_canonical(value).ok_or_else(|| invalid_value("time.kind", value.to_string()))
}

fn parse_time_unit(value: &str) -> Result<TimeUnit, RecordLoadError> {
    TimeUnit::from_canonical(value).ok_or_else(|| invalid_value("time.unit", value.to_string()))
}

pub(super) fn invalid_parse<E: std::fmt::Display + 'static>(
    field: &'static str,
) -> impl FnOnce(E) -> RecordLoadError {
    move |error| RecordLoadError::InvalidData(format!("{field} is invalid: {error}"))
}

fn invalid_value(field: &'static str, value: String) -> RecordLoadError {
    RecordLoadError::InvalidData(format!("{field} has unsupported value `{value}`"))
}

fn invalid_missing(field: impl Into<String>) -> RecordLoadError {
    RecordLoadError::InvalidData(format!("required field `{}` is missing", field.into()))
}
