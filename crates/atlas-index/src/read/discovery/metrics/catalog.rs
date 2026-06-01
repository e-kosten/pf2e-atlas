use atlas_domain::{BooleanFieldCounts, FilterValueCount, MetricKeyDiscovery, NumericFieldStats};
use diesel::OptionalExtension;
use diesel::sql_types::{BigInt, Double, Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::read::sql::{SqlBindValue, bind_sql_query};

use super::keys::metric_key_from_parts;
use super::query::{metric_matches_query, metric_query_tokens, normalize_metric_label};
use super::scope::{
    MetricCatalogScope, matching_count_for_catalog_scope, push_catalog_scope_predicate,
};
use crate::read::discovery::error::{DiscoveryError, query_error};

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

pub(super) fn catalog_metric_numeric_stats(
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

pub(super) fn catalog_metric_text_values(
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

pub(super) fn catalog_metric_boolean_counts(
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
