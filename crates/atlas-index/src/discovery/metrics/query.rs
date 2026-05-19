use atlas_domain::MetricKeyDiscovery;

pub(super) fn normalize_metric_label(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}

pub(super) fn metric_label_matches(value: Option<&str>, normalized: &str) -> bool {
    value
        .map(normalize_metric_label)
        .is_some_and(|label| label == normalized)
}

pub(super) fn metric_query_tokens(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(normalize_metric_label)
        .filter(|token| !token.is_empty())
        .collect()
}

pub(super) fn metric_matches_query(metric: &MetricKeyDiscovery, tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return true;
    }
    let haystack = [
        Some(metric.metric_key.as_str()),
        Some(metric.namespace_prefix.as_str()),
        metric.label.as_deref(),
        metric.short_label.as_deref(),
        metric.group.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(normalize_metric_label)
    .collect::<Vec<_>>()
    .join(" ");
    tokens.iter().all(|token| haystack.contains(token))
}
