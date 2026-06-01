use crate::sqlite::raw_sql::SqlBindValue;
use atlas_domain::{
    BooleanFieldCounts, FilterDiscoveryExecution, FilterValueCount, FilterValueDiscovery,
    FilterValuePayload, MetricDomain, MetricKeyDiscovery, MetricValuePayload, NumericFieldStats,
    RecordFamily, SearchFilterNode,
};
use atlas_record::{MetricRow, MetricValue, definition_for, label_for_row};
use diesel::OptionalExtension;
use diesel::sql_types::{BigInt, Double, Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::filters::EligibleRecordKeyset;
use crate::sqlite::raw_sql::{CountRow, bind_sql_query};

use super::error::{DiscoveryError, query_error};
use super::request::FilterValueRequest;
use super::stats;

mod query;
mod resolution;

use query::{
    metric_label_matches, metric_matches_query, metric_query_tokens, normalize_metric_label,
};
pub(super) use resolution::resolve_filter_metrics;

pub(super) fn values(
    connection: &mut SqliteConnection,
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
        if request.metric_query.is_some() {
            return Err(DiscoveryError::InvalidOption(
                "--metric-query cannot be combined with --metric".to_string(),
            ));
        }
        let metric = if let Some(scope) = catalog_scope {
            let metrics = catalog_metric_keys(
                connection,
                scope,
                None,
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
                request.metric_query.as_deref(),
                request.metric_domain.as_deref(),
            )?
        } else {
            metric_keys(
                connection,
                filter,
                request.metric_prefix.as_deref(),
                request.metric_label.as_deref(),
                request.metric_query.as_deref(),
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
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    prefix: Option<&str>,
    label_query: Option<&str>,
    metric_query: Option<&str>,
    domain: Option<&str>,
) -> Result<u64, DiscoveryError> {
    Ok(metric_keys(
        connection,
        filter,
        prefix,
        label_query,
        metric_query,
        domain,
    )?
    .len() as u64)
}

fn metric_keys(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    prefix: Option<&str>,
    label_query: Option<&str>,
    metric_query: Option<&str>,
    domain: Option<&str>,
) -> Result<Vec<MetricKeyDiscovery>, DiscoveryError> {
    let query = EligibleRecordKeyset::new(filter).compile()?.with_eligible_cte(
        |builder| {
            let mut predicates = Vec::new();
            if let Some(prefix) = prefix {
                let placeholder = builder.push_text(format!("{prefix}%"));
                predicates.push(format!("rm.metric_key LIKE {placeholder}"));
            }
            if let Some(domain) = domain {
                let placeholder = builder.push_text(domain);
                predicates.push(format!("rm.metric_domain = {placeholder}"));
            }
            let where_extra = if predicates.is_empty() {
                String::new()
            } else {
                format!(" AND {}", predicates.join(" AND "))
            };
            format!(
                "SELECT rm.metric_domain, r.record_family, rm.metric_key, rm.value_type, COUNT(*) AS catalog_count
         FROM record_metrics rm
         JOIN eligible e ON e.record_key = rm.record_key
         JOIN records r ON r.record_key = rm.record_key
         WHERE 1 = 1 {where_extra}
         GROUP BY rm.metric_domain, r.record_family, rm.metric_key, rm.value_type
         ORDER BY rm.metric_key ASC, r.record_family ASC",
            )
        },
    );
    let rows = bind_sql_query(query.sql, &query.parameters)
        .load::<MetricKeyRow>(connection)
        .map_err(query_error)?;
    let label_query = label_query.map(normalize_metric_label);
    let metric_query = metric_query.map(metric_query_tokens);
    let mut metrics = Vec::new();
    for row in rows {
        let metric = metric_key_from_parts(
            row.metric_domain,
            row.record_family,
            row.metric_key,
            row.value_type,
            row.catalog_count as u64,
            None,
        )?;
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
        if let Some(query) = &metric_query
            && !metric_matches_query(&metric, query)
        {
            continue;
        }
        metrics.push(metric);
    }
    Ok(metrics)
}

fn resolve_metric(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    value: &str,
    domain: Option<&str>,
) -> Result<MetricKeyDiscovery, DiscoveryError> {
    let metrics = metric_keys(connection, filter, None, None, None, domain)?;
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
            metric.known
                && (metric_label_matches(metric.label.as_deref(), &normalized)
                    || metric_label_matches(metric.short_label.as_deref(), &normalized))
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [metric] => Ok(metric.clone()),
        [] => Err(DiscoveryError::InvalidOption(format!(
            "metric `{value}` did not match a metric key, exact known label, or exact known short label"
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
    connection: &mut SqliteConnection,
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
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<Vec<FilterValueCount>, DiscoveryError> {
    if let Some(scope) = catalog_scope {
        return catalog_metric_text_values(connection, scope, metric);
    }
    let query = EligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|builder| {
            let domain_placeholder = builder.push_text(metric.metric_domain.clone());
            let key_placeholder = builder.push_text(metric.metric_key.clone());
            let value_type_placeholder = builder.push_text(metric.value_type.clone());
            format!(
                "SELECT rm.text_value AS value, COUNT(*) AS catalog_count
         FROM record_metrics rm
         JOIN eligible e ON e.record_key = rm.record_key
         WHERE rm.metric_domain = {domain_placeholder}
           AND rm.metric_key = {key_placeholder}
           AND rm.value_type = {value_type_placeholder}
           AND rm.text_value IS NOT NULL
         GROUP BY rm.text_value
         ORDER BY COUNT(*) DESC, rm.text_value ASC",
            )
        });
    bind_sql_query(query.sql, &query.parameters)
        .load::<MetricValueCountRow>(connection)
        .map_err(query_error)
        .map(filter_value_counts_from_rows)
}

fn metric_boolean_counts(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<BooleanFieldCounts, DiscoveryError> {
    if let Some(scope) = catalog_scope {
        return catalog_metric_boolean_counts(connection, scope, metric);
    }
    let query = EligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|builder| {
            let domain_placeholder = builder.push_text(metric.metric_domain.clone());
            let key_placeholder = builder.push_text(metric.metric_key.clone());
            let value_type_placeholder = builder.push_text(metric.value_type.clone());
            format!(
                "SELECT
           SUM(CASE WHEN rm.bool_value = 1 THEN 1 ELSE 0 END) AS true_count,
           SUM(CASE WHEN rm.bool_value = 0 THEN 1 ELSE 0 END) AS false_count,
           (SELECT COUNT(*) FROM eligible) - COUNT(rm.bool_value) AS null_count
         FROM record_metrics rm
         JOIN eligible e ON e.record_key = rm.record_key
         WHERE rm.metric_domain = {domain_placeholder}
           AND rm.metric_key = {key_placeholder}
           AND rm.value_type = {value_type_placeholder}",
            )
        });
    bind_sql_query(query.sql, &query.parameters)
        .get_result::<BooleanCountsRow>(connection)
        .map(|row| BooleanFieldCounts {
            r#true: row.true_count.unwrap_or(0) as u64,
            r#false: row.false_count.unwrap_or(0) as u64,
            null: row.null_count.unwrap_or(0) as u64,
        })
        .map_err(query_error)
}

fn metric_numeric_stats(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<atlas_domain::NumericFieldStats, DiscoveryError> {
    if let Some(scope) = catalog_scope
        && let Some(stats) = catalog_metric_numeric_stats(connection, scope, metric)?
    {
        return Ok(stats);
    }
    let query = EligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|builder| {
            let domain_placeholder = builder.push_text(metric.metric_domain.clone());
            let key_placeholder = builder.push_text(metric.metric_key.clone());
            let value_type_placeholder = builder.push_text(metric.value_type.clone());
            format!(
                "SELECT rm.number_value AS number_value
         FROM record_metrics rm
         JOIN eligible e ON e.record_key = rm.record_key
         WHERE rm.metric_domain = {domain_placeholder}
           AND rm.metric_key = {key_placeholder}
           AND rm.value_type = {value_type_placeholder}
           AND rm.number_value IS NOT NULL
         ORDER BY rm.number_value ASC",
            )
        });
    let values = bind_sql_query(query.sql, &query.parameters)
        .load::<MetricNumericValueRow>(connection)
        .map_err(query_error)?
        .into_iter()
        .map(|row| row.number_value)
        .collect::<Vec<_>>();
    let matching_record_count = super::dynamic::count_matching_records(connection, filter)?;
    Ok(stats::numeric_stats_from_values(
        &values,
        matching_record_count,
    ))
}

pub(super) fn catalog_metric_keys(
    connection: &mut SqliteConnection,
    scope: MetricCatalogScope,
    prefix: Option<&str>,
    label_query: Option<&str>,
    metric_query: Option<&str>,
    domain: Option<&str>,
) -> Result<Vec<MetricKeyDiscovery>, DiscoveryError> {
    let mut sql = String::from(
        "SELECT mk.metric_domain, COALESCE(mk.record_family, 'all') AS record_family,
                mk.metric_key, mk.value_type, mk.catalog_count,
                ns.catalog_count AS ns_catalog_count, ns.null_count AS ns_null_count,
                ns.min, ns.p05, ns.p25, ns.p50, ns.mean, ns.p75, ns.p95, ns.max
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
        parameters.push(SqlBindValue::Text(format!("{prefix}%")));
        sql.push_str(&format!(" AND mk.metric_key LIKE ?{}", parameters.len()));
    }
    if let Some(domain) = domain {
        parameters.push(SqlBindValue::Text(domain.to_string()));
        sql.push_str(&format!(" AND mk.metric_domain = ?{}", parameters.len()));
    }
    sql.push_str(" ORDER BY mk.metric_key ASC, mk.record_family ASC");
    let rows = bind_sql_query(sql, &parameters)
        .load::<CatalogMetricKeyRow>(connection)
        .map_err(query_error)?;
    let label_query = label_query.map(normalize_metric_label);
    let metric_query = metric_query.map(metric_query_tokens);
    let mut metrics = Vec::new();
    for row in rows {
        let numeric_stats = row.ns_catalog_count.map(|count| NumericFieldStats {
            count: count as u64,
            null_count: row.ns_null_count.unwrap_or(0) as u64,
            min: row.min,
            p05: row.p05,
            p25: row.p25,
            p50: row.p50,
            mean: row.mean,
            p75: row.p75,
            p95: row.p95,
            max: row.max,
        });
        let metric = metric_key_from_parts(
            row.metric_domain,
            row.record_family,
            row.metric_key,
            row.value_type,
            row.catalog_count as u64,
            numeric_stats,
        )?;
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
        if let Some(query) = &metric_query
            && !metric_matches_query(&metric, query)
        {
            continue;
        }
        metrics.push(metric);
    }
    Ok(metrics)
}

fn catalog_metric_numeric_stats(
    connection: &mut SqliteConnection,
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
    parameters.push(SqlBindValue::Text(metric.metric_domain.clone()));
    sql.push_str(&format!(" AND metric_domain = ?{}", parameters.len()));
    parameters.push(SqlBindValue::Text(metric.metric_key.clone()));
    sql.push_str(&format!(" AND metric_key = ?{}", parameters.len()));
    bind_sql_query(sql, &parameters)
        .get_result::<MetricNumericStatsRow>(connection)
        .optional()
        .map_err(query_error)
        .map(|row| row.map(metric_numeric_stats_from_row))
}

fn catalog_metric_text_values(
    connection: &mut SqliteConnection,
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
        SqlBindValue::Text(metric.metric_domain.clone()),
        SqlBindValue::Text(metric.metric_key.clone()),
    ];
    push_catalog_scope_predicate(&mut sql, &mut parameters, "record_family", scope);
    sql.push_str(" ORDER BY catalog_count DESC, value ASC");
    bind_sql_query(sql, &parameters)
        .load::<MetricValueCountRow>(connection)
        .map_err(query_error)
        .map(filter_value_counts_from_rows)
}

fn catalog_metric_boolean_counts(
    connection: &mut SqliteConnection,
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
) -> Result<MetricKeyDiscovery, DiscoveryError> {
    let metric_value = match value_type.as_str() {
        "number" => MetricValue::Number(0.0),
        "boolean" => MetricValue::Boolean(false),
        _ => MetricValue::Text(String::new()),
    };
    let domain = parse_metric_domain(&domain)?;
    let row = MetricRow {
        domain,
        key: metric_key.clone(),
        value: metric_value,
    };
    let label = label_for_row(&row);
    let group = definition_for(domain, &metric_key)
        .map(|matched| matched.definition.group().as_str().to_string());
    Ok(MetricKeyDiscovery {
        metric_domain: metric_domain_string(domain),
        record_family,
        namespace_prefix: metric_key
            .split_once('.')
            .map(|(prefix, _)| format!("{prefix}."))
            .unwrap_or_default(),
        metric_key,
        label: Some(label.label),
        short_label: label.short_label,
        group,
        known: label.known,
        value_type,
        count,
        numeric_stats,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MetricCatalogScope {
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
    parameters: &mut Vec<SqlBindValue>,
    column: &str,
    scope: MetricCatalogScope,
) {
    match scope {
        MetricCatalogScope::Global => sql.push_str(&format!(" AND {column} IS NULL")),
        MetricCatalogScope::Family(family) => {
            parameters.push(SqlBindValue::Text(record_family_string(family)));
            sql.push_str(&format!(" AND {column} = ?{}", parameters.len()));
        }
    }
}

fn matching_count_for_catalog_scope(
    connection: &mut SqliteConnection,
    scope: MetricCatalogScope,
) -> Result<u64, DiscoveryError> {
    match scope {
        MetricCatalogScope::Global => bind_sql_query(
            "SELECT COUNT(*) AS count FROM records WHERE is_default_visible = 1".to_string(),
            &[],
        )
        .get_result::<CountRow>(connection)
        .map(|row| row.count as u64)
        .map_err(query_error),
        MetricCatalogScope::Family(family) => bind_sql_query(
            "SELECT COUNT(*) AS count FROM records WHERE is_default_visible = 1 AND record_family = ?1"
                .to_string(),
            &[SqlBindValue::Text(record_family_string(family))],
        )
        .get_result::<CountRow>(connection)
        .map(|row| row.count as u64)
        .map_err(query_error),
    }
}

fn record_family_string(value: RecordFamily) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| format!("{value:?}").to_lowercase())
}

fn parse_metric_domain(value: &str) -> Result<MetricDomain, DiscoveryError> {
    MetricDomain::from_canonical(value)
        .ok_or_else(|| DiscoveryError::QueryFailed(format!("unknown metric domain `{value}`")))
}

fn metric_domain_string(value: MetricDomain) -> String {
    value.as_str().to_string()
}

fn filter_value_counts_from_rows(rows: Vec<MetricValueCountRow>) -> Vec<FilterValueCount> {
    rows.into_iter()
        .map(|row| FilterValueCount {
            value: row.value,
            count: row.catalog_count as u64,
        })
        .collect()
}

fn metric_numeric_stats_from_row(row: MetricNumericStatsRow) -> NumericFieldStats {
    NumericFieldStats {
        count: row.catalog_count as u64,
        null_count: row.null_count as u64,
        min: row.min,
        p05: row.p05,
        p25: row.p25,
        p50: row.p50,
        mean: row.mean,
        p75: row.p75,
        p95: row.p95,
        max: row.max,
    }
}

#[derive(QueryableByName)]
struct MetricKeyRow {
    #[diesel(sql_type = Text)]
    metric_domain: String,
    #[diesel(sql_type = Text)]
    record_family: String,
    #[diesel(sql_type = Text)]
    metric_key: String,
    #[diesel(sql_type = Text)]
    value_type: String,
    #[diesel(sql_type = BigInt)]
    catalog_count: i64,
}

#[derive(QueryableByName)]
struct CatalogMetricKeyRow {
    #[diesel(sql_type = Text)]
    metric_domain: String,
    #[diesel(sql_type = Text)]
    record_family: String,
    #[diesel(sql_type = Text)]
    metric_key: String,
    #[diesel(sql_type = Text)]
    value_type: String,
    #[diesel(sql_type = BigInt)]
    catalog_count: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    ns_catalog_count: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    ns_null_count: Option<i64>,
    #[diesel(sql_type = Nullable<Double>)]
    min: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p05: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p25: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p50: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    mean: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p75: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p95: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    max: Option<f64>,
}

#[derive(QueryableByName)]
struct MetricValueCountRow {
    #[diesel(sql_type = Text)]
    value: String,
    #[diesel(sql_type = BigInt)]
    catalog_count: i64,
}

#[derive(QueryableByName)]
struct BooleanCountsRow {
    #[diesel(sql_type = Nullable<BigInt>)]
    true_count: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    false_count: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    null_count: Option<i64>,
}

#[derive(QueryableByName)]
struct MetricNumericValueRow {
    #[diesel(sql_type = Double)]
    number_value: f64,
}

#[derive(QueryableByName)]
struct MetricNumericStatsRow {
    #[diesel(sql_type = BigInt)]
    catalog_count: i64,
    #[diesel(sql_type = BigInt)]
    null_count: i64,
    #[diesel(sql_type = Nullable<Double>)]
    min: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p05: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p25: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p50: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    mean: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p75: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    p95: Option<f64>,
    #[diesel(sql_type = Nullable<Double>)]
    max: Option<f64>,
}
