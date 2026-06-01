use atlas_domain::{
    BooleanFieldCounts, FilterValueCount, MetricKeyDiscovery, NumericFieldStats, SearchFilterNode,
};
use diesel::sql_types::{BigInt, Double, Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::read::discovery::dynamic as field_dynamic;
use crate::read::discovery::error::{DiscoveryError, query_error};
use crate::read::discovery::stats;
use crate::read::search::filters::SqliteEligibleRecordKeyset;
use crate::read::sql::bind_sql_query;

use super::keys::metric_key_from_parts;
use super::query::{metric_matches_query, metric_query_tokens, normalize_metric_label};

pub(super) fn metric_keys(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    prefix: Option<&str>,
    label_query: Option<&str>,
    metric_query: Option<&str>,
    domain: Option<&str>,
) -> Result<Vec<MetricKeyDiscovery>, DiscoveryError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|builder| {
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
        });
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

pub(super) fn metric_text_values(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    metric: &MetricKeyDiscovery,
) -> Result<Vec<FilterValueCount>, DiscoveryError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
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

pub(super) fn metric_boolean_counts(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    metric: &MetricKeyDiscovery,
) -> Result<BooleanFieldCounts, DiscoveryError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
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

pub(super) fn metric_numeric_stats(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    metric: &MetricKeyDiscovery,
) -> Result<NumericFieldStats, DiscoveryError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
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
    let matching_record_count = field_dynamic::count_matching_records(connection, filter)?;
    Ok(stats::numeric_stats_from_values(
        &values,
        matching_record_count,
    ))
}

fn filter_value_counts_from_rows(rows: Vec<MetricValueCountRow>) -> Vec<FilterValueCount> {
    rows.into_iter()
        .map(|row| FilterValueCount {
            value: row.value,
            count: row.catalog_count as u64,
        })
        .collect()
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
