use atlas_domain::{MetricDomain, MetricValueType, PublicationFamily, TimeKind, TimeUnit};

use atlas_record::MetricValue;

pub(super) fn metric_value_parts(
    value: &MetricValue,
) -> (&'static str, Option<f64>, Option<&str>, Option<i64>) {
    match value {
        MetricValue::Number(number) => (
            metric_value_type_label(MetricValueType::Number),
            Some(*number),
            None,
            None,
        ),
        MetricValue::Text(text) => (
            metric_value_type_label(MetricValueType::Text),
            None,
            Some(text.as_str()),
            None,
        ),
        MetricValue::Boolean(boolean) => (
            metric_value_type_label(MetricValueType::Boolean),
            None,
            None,
            Some(i64::from(*boolean)),
        ),
    }
}

pub(super) fn metric_domain_label(domain: MetricDomain) -> &'static str {
    domain.as_str()
}

fn metric_value_type_label(value_type: MetricValueType) -> &'static str {
    value_type.as_str()
}

pub(super) fn time_kind_label(kind: TimeKind) -> &'static str {
    kind.as_str()
}

pub(super) fn time_unit_label(unit: TimeUnit) -> &'static str {
    unit.as_str()
}

pub(super) fn publication_family_label(publication_family: PublicationFamily) -> &'static str {
    publication_family.as_str()
}
