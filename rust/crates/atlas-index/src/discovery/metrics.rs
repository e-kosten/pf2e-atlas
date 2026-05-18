use atlas_domain::{
    BooleanFieldCounts, FilterDiscoveryExecution, FilterValueCount, FilterValueDiscovery,
    FilterValuePayload, MetricDomain, MetricKeyDiscovery, MetricValuePayload, NumericFieldStats,
    RecordFamily, SearchFilterNode,
};
use atlas_record::{MetricRow, MetricValue, label_for_row};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

use crate::filters::compile_eligible_records_query;

use super::error::{DiscoveryError, query_error};
use super::request::FilterValueRequest;
use super::stats;

pub(super) fn values(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    request: FilterValueRequest,
) -> Result<FilterValueDiscovery, DiscoveryError> {
    if request.sort.is_some() {
        return Err(DiscoveryError::InvalidOption(
            "--sort applies only to enumerable metadata value fields".to_string(),
        ));
    }
    if request.sample_limit.is_some() {
        return Err(DiscoveryError::InvalidOption(
            "--sample-limit applies only to sampled text fields".to_string(),
        ));
    }
    let matching_record_count = super::dynamic::count_matching_records(connection, filter)?;
    let catalog_scope = metric_catalog_scope(filter);
    let execution = if catalog_scope.is_some() {
        FilterDiscoveryExecution::Catalog
    } else {
        FilterDiscoveryExecution::Dynamic
    };
    let payload = if let Some(metric) = request.metric.as_deref() {
        if request.metric_prefix.is_some() {
            return Err(DiscoveryError::InvalidOption(
                "--metric-prefix cannot be combined with --metric".to_string(),
            ));
        }
        if request.metric_label.is_some() {
            return Err(DiscoveryError::InvalidOption(
                "--metric-label cannot be combined with --metric".to_string(),
            ));
        }
        let metric = if let Some(scope) = catalog_scope {
            let metrics = catalog_metric_keys(
                connection,
                scope,
                None,
                None,
                request.metric_domain.as_deref(),
            )?;
            resolve_metric_from_candidates(metrics, metric)?
        } else {
            resolve_metric(connection, filter, metric, request.metric_domain.as_deref())?
        };
        let values = metric_values(connection, filter, catalog_scope, &metric)?;
        FilterValuePayload::MetricValues {
            metric: Box::new(metric),
            values,
        }
    } else {
        let metrics = if let Some(scope) = catalog_scope {
            catalog_metric_keys(
                connection,
                scope,
                request.metric_prefix.as_deref(),
                request.metric_label.as_deref(),
                request.metric_domain.as_deref(),
            )?
        } else {
            metric_keys(
                connection,
                filter,
                request.metric_prefix.as_deref(),
                request.metric_label.as_deref(),
                request.metric_domain.as_deref(),
            )?
        };
        FilterValuePayload::MetricKeys { metrics }
    };
    Ok(FilterValueDiscovery {
        field: "metric".to_string(),
        filter: request.filter_json,
        execution,
        matching_record_count,
        payload,
    })
}

pub(super) fn metric_key_count(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    prefix: Option<&str>,
    label_query: Option<&str>,
    domain: Option<&str>,
) -> Result<u64, DiscoveryError> {
    Ok(metric_keys(connection, filter, prefix, label_query, domain)?.len() as u64)
}

fn metric_keys(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    prefix: Option<&str>,
    label_query: Option<&str>,
    domain: Option<&str>,
) -> Result<Vec<MetricKeyDiscovery>, DiscoveryError> {
    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    let mut predicates = Vec::new();
    if let Some(prefix) = prefix {
        parameters.push(Value::Text(format!("{prefix}%")));
        predicates.push(format!("rm.metric_key LIKE ?{}", parameters.len()));
    }
    if let Some(domain) = domain {
        parameters.push(Value::Text(domain.to_string()));
        predicates.push(format!("rm.metric_domain = ?{}", parameters.len()));
    }
    let where_extra = if predicates.is_empty() {
        String::new()
    } else {
        format!(" AND {}", predicates.join(" AND "))
    };
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible})
         SELECT rm.metric_domain, r.record_family, rm.metric_key, rm.value_type, COUNT(*) AS catalog_count
         FROM record_metrics rm
         JOIN eligible e ON e.record_key = rm.record_key
         JOIN records r ON r.record_key = rm.record_key
         WHERE 1 = 1 {where_extra}
         GROUP BY rm.metric_domain, r.record_family, rm.metric_key, rm.value_type
         ORDER BY rm.metric_key ASC, r.record_family ASC",
        eligible = eligible.sql,
    );
    let mut statement = connection.prepare(&sql).map_err(query_error)?;
    let rows = statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            metric_key_from_parts(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                None,
            )
        })
        .map_err(query_error)?;
    let label_query = label_query.map(normalize_metric_label);
    let mut metrics = Vec::new();
    for row in rows {
        let metric = row.map_err(query_error)?;
        if let Some(query) = &label_query {
            let label = metric.label.as_deref().unwrap_or("");
            let haystack = format!(
                "{} {}",
                normalize_metric_label(&metric.metric_key),
                normalize_metric_label(label)
            );
            if !haystack.contains(query) {
                continue;
            }
        }
        metrics.push(metric);
    }
    Ok(metrics)
}

fn resolve_metric(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    value: &str,
    domain: Option<&str>,
) -> Result<MetricKeyDiscovery, DiscoveryError> {
    let metrics = metric_keys(connection, filter, None, None, domain)?;
    resolve_metric_from_candidates(metrics, value)
}

fn resolve_metric_from_candidates(
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
                metric_candidates(&key_matches)
            )));
        }
    }
    let normalized = normalize_metric_label(value);
    let matches = metrics
        .into_iter()
        .filter(|metric| {
            metric
                .label
                .as_deref()
                .map(normalize_metric_label)
                .is_some_and(|label| label == normalized)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [metric] => Ok(metric.clone()),
        [] => Err(DiscoveryError::InvalidOption(format!(
            "metric `{value}` did not match a metric key or exact known label"
        ))),
        _ => Err(DiscoveryError::AmbiguousMetric(format!(
            "metric label `{value}` is ambiguous; candidates: {}",
            metric_candidates(&matches)
        ))),
    }
}

fn metric_candidates(metrics: &[MetricKeyDiscovery]) -> String {
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

fn metric_values(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<MetricValuePayload, DiscoveryError> {
    match metric.value_type.as_str() {
        "number" => metric_numeric_stats(connection, filter, catalog_scope, metric)
            .map(|stats| MetricValuePayload::NumericStats { stats }),
        "boolean" => metric_boolean_counts(connection, filter, catalog_scope, metric)
            .map(|counts| MetricValuePayload::BooleanCounts { counts }),
        _ => metric_text_values(connection, filter, catalog_scope, metric)
            .map(|values| MetricValuePayload::TextValues { values }),
    }
}

fn metric_text_values(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<Vec<FilterValueCount>, DiscoveryError> {
    if let Some(scope) = catalog_scope {
        return catalog_metric_text_values(connection, scope, metric);
    }
    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    parameters.push(Value::Text(metric.metric_domain.clone()));
    parameters.push(Value::Text(metric.metric_key.clone()));
    parameters.push(Value::Text(metric.value_type.clone()));
    let domain_index = parameters.len() - 2;
    let key_index = parameters.len() - 1;
    let value_type_index = parameters.len();
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible})
         SELECT rm.text_value, COUNT(*)
         FROM record_metrics rm
         JOIN eligible e ON e.record_key = rm.record_key
         WHERE rm.metric_domain = ?{domain_index}
           AND rm.metric_key = ?{key_index}
           AND rm.value_type = ?{value_type_index}
           AND rm.text_value IS NOT NULL
         GROUP BY rm.text_value
         ORDER BY COUNT(*) DESC, rm.text_value ASC",
        eligible = eligible.sql,
    );
    let mut statement = connection.prepare(&sql).map_err(query_error)?;
    statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            Ok(FilterValueCount {
                value: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)
}

fn metric_boolean_counts(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<BooleanFieldCounts, DiscoveryError> {
    if let Some(scope) = catalog_scope {
        return catalog_metric_boolean_counts(connection, scope, metric);
    }
    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    parameters.push(Value::Text(metric.metric_domain.clone()));
    parameters.push(Value::Text(metric.metric_key.clone()));
    parameters.push(Value::Text(metric.value_type.clone()));
    let domain_index = parameters.len() - 2;
    let key_index = parameters.len() - 1;
    let value_type_index = parameters.len();
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible})
         SELECT
           SUM(CASE WHEN rm.bool_value = 1 THEN 1 ELSE 0 END),
           SUM(CASE WHEN rm.bool_value = 0 THEN 1 ELSE 0 END),
           (SELECT COUNT(*) FROM eligible) - COUNT(rm.bool_value)
         FROM record_metrics rm
         JOIN eligible e ON e.record_key = rm.record_key
         WHERE rm.metric_domain = ?{domain_index}
           AND rm.metric_key = ?{key_index}
           AND rm.value_type = ?{value_type_index}",
        eligible = eligible.sql,
    );
    connection
        .query_row(&sql, params_from_iter(parameters.iter()), |row| {
            Ok(BooleanFieldCounts {
                r#true: row.get::<_, Option<u64>>(0)?.unwrap_or(0),
                r#false: row.get::<_, Option<u64>>(1)?.unwrap_or(0),
                null: row.get::<_, Option<u64>>(2)?.unwrap_or(0),
            })
        })
        .map_err(query_error)
}

fn metric_numeric_stats(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<atlas_domain::NumericFieldStats, DiscoveryError> {
    if let Some(scope) = catalog_scope
        && let Some(stats) = catalog_metric_numeric_stats(connection, scope, metric)?
    {
        return Ok(stats);
    }
    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    parameters.push(Value::Text(metric.metric_domain.clone()));
    parameters.push(Value::Text(metric.metric_key.clone()));
    parameters.push(Value::Text(metric.value_type.clone()));
    let domain_index = parameters.len() - 2;
    let key_index = parameters.len() - 1;
    let value_type_index = parameters.len();
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible})
         SELECT rm.number_value
         FROM record_metrics rm
         JOIN eligible e ON e.record_key = rm.record_key
         WHERE rm.metric_domain = ?{domain_index}
           AND rm.metric_key = ?{key_index}
           AND rm.value_type = ?{value_type_index}
           AND rm.number_value IS NOT NULL
         ORDER BY rm.number_value ASC",
        eligible = eligible.sql,
    );
    let mut statement = connection.prepare(&sql).map_err(query_error)?;
    let values = statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            row.get::<_, f64>(0)
        })
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)?;
    let matching_record_count = super::dynamic::count_matching_records(connection, filter)?;
    Ok(stats::numeric_stats_from_values(
        &values,
        matching_record_count,
    ))
}

fn catalog_metric_keys(
    connection: &Connection,
    scope: MetricCatalogScope,
    prefix: Option<&str>,
    label_query: Option<&str>,
    domain: Option<&str>,
) -> Result<Vec<MetricKeyDiscovery>, DiscoveryError> {
    let mut sql = String::from(
        "SELECT mk.metric_domain, COALESCE(mk.record_family, 'all'), mk.metric_key, mk.value_type,
                mk.catalog_count, ns.catalog_count, ns.null_count, ns.min, ns.p05,
                ns.p25, ns.p50, ns.mean, ns.p75, ns.p95, ns.max
         FROM metric_key_catalog mk
         LEFT JOIN filter_numeric_catalog ns
           ON ns.field = 'metric'
          AND ((ns.record_family IS NULL AND mk.record_family IS NULL)
               OR ns.record_family = mk.record_family)
          AND ns.metric_domain = mk.metric_domain
          AND ns.metric_key = mk.metric_key
         WHERE 1 = 1",
    );
    let mut parameters = Vec::new();
    push_catalog_scope_predicate(&mut sql, &mut parameters, "mk.record_family", scope);
    if let Some(prefix) = prefix {
        parameters.push(Value::Text(format!("{prefix}%")));
        sql.push_str(&format!(" AND mk.metric_key LIKE ?{}", parameters.len()));
    }
    if let Some(domain) = domain {
        parameters.push(Value::Text(domain.to_string()));
        sql.push_str(&format!(" AND mk.metric_domain = ?{}", parameters.len()));
    }
    sql.push_str(" ORDER BY mk.metric_key ASC, mk.record_family ASC");
    let mut statement = connection.prepare(&sql).map_err(query_error)?;
    let rows = statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            let numeric_stats = if let Some(count) = row.get::<_, Option<u64>>(5)? {
                Some(NumericFieldStats {
                    count,
                    null_count: row.get(6)?,
                    min: row.get(7)?,
                    p05: row.get(8)?,
                    p25: row.get(9)?,
                    p50: row.get(10)?,
                    mean: row.get(11)?,
                    p75: row.get(12)?,
                    p95: row.get(13)?,
                    max: row.get(14)?,
                })
            } else {
                None
            };
            metric_key_from_parts(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                numeric_stats,
            )
        })
        .map_err(query_error)?;
    let label_query = label_query.map(normalize_metric_label);
    let mut metrics = Vec::new();
    for row in rows {
        let metric = row.map_err(query_error)?;
        if let Some(query) = &label_query {
            let label = metric.label.as_deref().unwrap_or("");
            let haystack = format!(
                "{} {}",
                normalize_metric_label(&metric.metric_key),
                normalize_metric_label(label)
            );
            if !haystack.contains(query) {
                continue;
            }
        }
        metrics.push(metric);
    }
    Ok(metrics)
}

fn catalog_metric_numeric_stats(
    connection: &Connection,
    scope: MetricCatalogScope,
    metric: &MetricKeyDiscovery,
) -> Result<Option<NumericFieldStats>, DiscoveryError> {
    let mut sql = String::from(
        "SELECT catalog_count, null_count, min, p05, p25, p50, mean, p75, p95, max
         FROM filter_numeric_catalog
         WHERE field = 'metric'",
    );
    let mut parameters = Vec::new();
    push_catalog_scope_predicate(&mut sql, &mut parameters, "record_family", scope);
    parameters.push(Value::Text(metric.metric_domain.clone()));
    sql.push_str(&format!(" AND metric_domain = ?{}", parameters.len()));
    parameters.push(Value::Text(metric.metric_key.clone()));
    sql.push_str(&format!(" AND metric_key = ?{}", parameters.len()));
    connection
        .query_row(
            &sql,
            params_from_iter(parameters.iter()),
            stats::numeric_stats_from_row,
        )
        .optional()
        .map_err(query_error)
}

fn catalog_metric_text_values(
    connection: &Connection,
    scope: MetricCatalogScope,
    metric: &MetricKeyDiscovery,
) -> Result<Vec<FilterValueCount>, DiscoveryError> {
    let mut sql = String::from(
        "SELECT value, catalog_count
         FROM metric_value_catalog
         WHERE metric_domain = ?1
           AND metric_key = ?2",
    );
    let mut parameters = vec![
        Value::Text(metric.metric_domain.clone()),
        Value::Text(metric.metric_key.clone()),
    ];
    push_catalog_scope_predicate(&mut sql, &mut parameters, "record_family", scope);
    sql.push_str(" ORDER BY catalog_count DESC, value ASC");
    let mut statement = connection.prepare(&sql).map_err(query_error)?;
    statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            Ok(FilterValueCount {
                value: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)
}

fn catalog_metric_boolean_counts(
    connection: &Connection,
    scope: MetricCatalogScope,
    metric: &MetricKeyDiscovery,
) -> Result<BooleanFieldCounts, DiscoveryError> {
    let values = catalog_metric_text_values(connection, scope, metric)?;
    let matching_count = matching_count_for_catalog_scope(connection, scope)?;
    let true_count = values
        .iter()
        .find(|value| value.value == "1")
        .map(|value| value.count)
        .unwrap_or(0);
    let false_count = values
        .iter()
        .find(|value| value.value == "0")
        .map(|value| value.count)
        .unwrap_or(0);
    Ok(BooleanFieldCounts {
        r#true: true_count,
        r#false: false_count,
        null: matching_count.saturating_sub(true_count + false_count),
    })
}

fn metric_key_from_parts(
    domain: String,
    record_family: String,
    metric_key: String,
    value_type: String,
    count: u64,
    numeric_stats: Option<NumericFieldStats>,
) -> rusqlite::Result<MetricKeyDiscovery> {
    let metric_value = match value_type.as_str() {
        "number" => MetricValue::Number(0.0),
        "boolean" => MetricValue::Boolean(false),
        _ => MetricValue::Text(String::new()),
    };
    let row = MetricRow {
        domain: parse_metric_domain(&domain),
        key: metric_key.clone(),
        value: metric_value,
    };
    let label = label_for_row(&row);
    Ok(MetricKeyDiscovery {
        metric_domain: domain,
        record_family,
        namespace_prefix: metric_key
            .split_once('.')
            .map(|(prefix, _)| format!("{prefix}."))
            .unwrap_or_default(),
        metric_key,
        label: Some(label.label),
        known: label.known,
        value_type,
        count,
        numeric_stats,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricCatalogScope {
    Global,
    Family(RecordFamily),
}

fn metric_catalog_scope(filter: Option<&SearchFilterNode>) -> Option<MetricCatalogScope> {
    match filter {
        None => Some(MetricCatalogScope::Global),
        Some(SearchFilterNode::RecordFamily { value }) => Some(MetricCatalogScope::Family(*value)),
        _ => None,
    }
}

fn push_catalog_scope_predicate(
    sql: &mut String,
    parameters: &mut Vec<Value>,
    column: &str,
    scope: MetricCatalogScope,
) {
    match scope {
        MetricCatalogScope::Global => sql.push_str(&format!(" AND {column} IS NULL")),
        MetricCatalogScope::Family(family) => {
            parameters.push(Value::Text(record_family_string(family)));
            sql.push_str(&format!(" AND {column} = ?{}", parameters.len()));
        }
    }
}

fn matching_count_for_catalog_scope(
    connection: &Connection,
    scope: MetricCatalogScope,
) -> Result<u64, DiscoveryError> {
    match scope {
        MetricCatalogScope::Global => connection
            .query_row(
                "SELECT COUNT(*) FROM records WHERE is_default_visible = 1",
                [],
                |row| row.get(0),
            )
            .map_err(query_error),
        MetricCatalogScope::Family(family) => connection
            .query_row(
                "SELECT COUNT(*) FROM records WHERE is_default_visible = 1 AND record_family = ?1",
                params![record_family_string(family)],
                |row| row.get(0),
            )
            .map_err(query_error),
    }
}

fn record_family_string(value: RecordFamily) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| format!("{value:?}").to_lowercase())
}

fn parse_metric_domain(value: &str) -> MetricDomain {
    serde_json::from_value(serde_json::Value::String(value.to_string()))
        .expect("metric domain from catalog should be valid")
}

fn normalize_metric_label(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}
