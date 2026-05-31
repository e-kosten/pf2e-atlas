use crate::sqlite::raw_sql::SqlBindValue;
use atlas_domain::{
    FilterFieldGroup, FilterFieldInfo, FilterFieldStats, FilterFieldType, FilterOperator,
    FilterSample, FilterValueCount, FilterValuePayload, FilterValuePolicy, RecordFamily,
};
use diesel::OptionalExtension;
use diesel::sql_types::{BigInt, Double, Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use super::definitions::FieldDefinition;
use super::error::{DiscoveryError, query_error};
use super::request::{DiscoveryValueSort, FilterValueRequest};
use super::stats;
use crate::sqlite::raw_sql::{CountRow, bind_sql_query};

pub(super) fn fields(
    connection: &mut SqliteConnection,
    scope: Option<RecordFamily>,
) -> Result<Vec<FilterFieldInfo>, DiscoveryError> {
    let scope = scope.map(record_family_string);
    bind_sql_query(
        "SELECT field, field_type, field_group, value_policy, operators_json, cli_flags_json,
                applicable_families_json
         FROM filter_field_catalog
         WHERE (record_family IS NULL AND ?1 IS NULL) OR record_family = ?1
         ORDER BY field"
            .to_string(),
        &[optional_text_value(scope)],
    )
    .load::<FilterFieldInfoRow>(connection)
    .map_err(query_error)?
    .into_iter()
    .map(filter_field_info_from_row)
    .collect()
}

pub(super) fn values(
    connection: &mut SqliteConnection,
    definition: FieldDefinition,
    scope: Option<RecordFamily>,
    request: &FilterValueRequest,
) -> Result<FilterValuePayload, DiscoveryError> {
    match definition.value_policy {
        FilterValuePolicy::Enumerable => {
            let sort = request.sort.unwrap_or(definition.default_sort);
            Ok(FilterValuePayload::Enumerable {
                values: enumerable_values(connection, definition.field, scope, sort)?,
                null_count: field_null_count(connection, definition.field, scope)?,
                sort,
            })
        }
        FilterValuePolicy::Sample => sample_values(
            connection,
            definition.field,
            scope,
            request.sample_limit.unwrap_or(20),
        ),
        FilterValuePolicy::NumericStats => Ok(FilterValuePayload::NumericStats {
            stats: numeric_stats(connection, definition.field, scope)?,
        }),
        _ => Err(DiscoveryError::InvalidOption(format!(
            "field `{}` is not catalog-backed for values",
            definition.field
        ))),
    }
}

fn enumerable_values(
    connection: &mut SqliteConnection,
    field: &str,
    scope: Option<RecordFamily>,
    sort: DiscoveryValueSort,
) -> Result<Vec<FilterValueCount>, DiscoveryError> {
    let scope = scope.map(record_family_string);
    let order = match sort {
        DiscoveryValueSort::Alpha | DiscoveryValueSort::Canonical => "value ASC",
        DiscoveryValueSort::Count => "catalog_count DESC, value ASC",
    };
    let sql = format!(
        "SELECT value, catalog_count FROM filter_value_catalog
         WHERE field = ?1 AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)
         ORDER BY {order}"
    );
    bind_sql_query(
        sql,
        &[
            SqlBindValue::Text(field.to_string()),
            optional_text_value(scope),
        ],
    )
    .load::<FilterValueCountRow>(connection)
    .map_err(query_error)
    .map(|rows| {
        rows.into_iter()
            .map(|row| FilterValueCount {
                value: row.value,
                count: row.catalog_count as u64,
            })
            .collect()
    })
}

fn sample_values(
    connection: &mut SqliteConnection,
    field: &str,
    scope: Option<RecordFamily>,
    sample_limit: usize,
) -> Result<FilterValuePayload, DiscoveryError> {
    let scope = scope.map(record_family_string);
    let stats = field_stats(connection, field, scope.as_deref())?;
    let examples = bind_sql_query(
        "SELECT value, catalog_count
         FROM filter_sample_catalog
         WHERE field = ?1 AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)
         ORDER BY sample_rank ASC
         LIMIT ?3"
            .to_string(),
        &[
            SqlBindValue::Text(field.to_string()),
            optional_text_value(scope),
            SqlBindValue::Integer(sample_limit as i64),
        ],
    )
    .load::<FilterValueCountRow>(connection)
    .map_err(query_error)?
    .into_iter()
    .map(|row| {
        stats::sample_example(&FilterValueCount {
            value: row.value,
            count: row.catalog_count as u64,
        })
    })
    .collect::<Vec<_>>();
    let null_count = stats.null_count;
    Ok(FilterValuePayload::Sample {
        sample: FilterSample {
            selection: "top_repeated_then_deterministic".to_string(),
            sample_limit,
            distinct_count: stats.distinct_count,
            omitted_distinct_count: stats.distinct_count.saturating_sub(examples.len() as u64),
            examples,
        },
        field_stats: stats,
        null_count,
    })
}

fn numeric_stats(
    connection: &mut SqliteConnection,
    field: &str,
    scope: Option<RecordFamily>,
) -> Result<atlas_domain::NumericFieldStats, DiscoveryError> {
    let scope = scope.map(record_family_string);
    bind_sql_query(
        "SELECT catalog_count, null_count, min, p05, p25, p50, mean, p75, p95, max
         FROM filter_numeric_catalog
         WHERE field = ?1
           AND metric_domain IS NULL
           AND metric_key IS NULL
           AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)"
            .to_string(),
        &[
            SqlBindValue::Text(field.to_string()),
            optional_text_value(scope),
        ],
    )
    .get_result::<NumericStatsRow>(connection)
    .optional()
    .map_err(query_error)?
    .map(numeric_stats_from_row)
    .ok_or_else(|| {
        DiscoveryError::FieldNotApplicable(format!("field `{field}` has no numeric catalog rows"))
    })
}

fn field_stats(
    connection: &mut SqliteConnection,
    field: &str,
    scope: Option<&str>,
) -> Result<FilterFieldStats, DiscoveryError> {
    bind_sql_query(
        "SELECT value_count, distinct_count, singleton_count, singleton_ratio,
                observation_singleton_ratio, null_count
         FROM filter_field_catalog
         WHERE field = ?1 AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)"
            .to_string(),
        &[
            SqlBindValue::Text(field.to_string()),
            optional_text_ref_value(scope),
        ],
    )
    .get_result::<FilterFieldStatsRow>(connection)
    .map(filter_field_stats_from_row)
    .map_err(query_error)
}

fn field_null_count(
    connection: &mut SqliteConnection,
    field: &str,
    scope: Option<RecordFamily>,
) -> Result<u64, DiscoveryError> {
    let scope = scope.map(record_family_string);
    bind_sql_query(
        "SELECT null_count AS count
         FROM filter_field_catalog
         WHERE field = ?1 AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)"
            .to_string(),
        &[
            SqlBindValue::Text(field.to_string()),
            optional_text_value(scope),
        ],
    )
    .get_result::<CountRow>(connection)
    .map(|row| row.count as u64)
    .map_err(query_error)
}

fn filter_field_info_from_row(row: FilterFieldInfoRow) -> Result<FilterFieldInfo, DiscoveryError> {
    Ok(FilterFieldInfo {
        field: row.field,
        field_type: parse_field_type(&row.field_type),
        group: parse_field_group(&row.field_group),
        value_policy: parse_value_policy(&row.value_policy),
        operators: parse_json_operators(&row.operators_json),
        cli_flags: serde_json::from_str(&row.cli_flags_json).unwrap_or_default(),
        applicable_families: serde_json::from_str(&row.applicable_families_json)
            .unwrap_or_default(),
        catalog_available: true,
    })
}

fn parse_json_operators(value: &str) -> Vec<FilterOperator> {
    serde_json::from_str::<Vec<String>>(value)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| match value.as_str() {
            "includes" => Some(FilterOperator::Includes),
            "eq" => Some(FilterOperator::Eq),
            "not_eq" => Some(FilterOperator::NotEq),
            "contains" => Some(FilterOperator::Contains),
            "not_contains" => Some(FilterOperator::NotContains),
            "gt" => Some(FilterOperator::Gt),
            "gte" => Some(FilterOperator::Gte),
            "lt" => Some(FilterOperator::Lt),
            "lte" => Some(FilterOperator::Lte),
            "between" => Some(FilterOperator::Between),
            "is_null" => Some(FilterOperator::IsNull),
            "is_not_null" => Some(FilterOperator::IsNotNull),
            _ => None,
        })
        .collect()
}

fn parse_field_type(value: &str) -> FilterFieldType {
    match value {
        "set" => FilterFieldType::Set,
        "text" => FilterFieldType::Text,
        "number" => FilterFieldType::Number,
        "boolean" => FilterFieldType::Boolean,
        "metric" => FilterFieldType::Metric,
        _ => FilterFieldType::EnumString,
    }
}

fn parse_field_group(value: &str) -> FilterFieldGroup {
    match value {
        "spell" => FilterFieldGroup::Spell,
        "actor" => FilterFieldGroup::Actor,
        "item" => FilterFieldGroup::Item,
        "variant" => FilterFieldGroup::Variant,
        "metric" => FilterFieldGroup::Metric,
        _ => FilterFieldGroup::Record,
    }
}

fn parse_value_policy(value: &str) -> FilterValuePolicy {
    match value {
        "sample" => FilterValuePolicy::Sample,
        "numeric_stats" => FilterValuePolicy::NumericStats,
        "boolean_counts" => FilterValuePolicy::BooleanCounts,
        "metric_keys" => FilterValuePolicy::MetricKeys,
        "metric_values" => FilterValuePolicy::MetricValues,
        _ => FilterValuePolicy::Enumerable,
    }
}

fn record_family_string(value: RecordFamily) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| format!("{value:?}").to_lowercase())
}

fn optional_text_value(value: Option<String>) -> SqlBindValue {
    value.map(SqlBindValue::Text).unwrap_or(SqlBindValue::Null)
}

fn optional_text_ref_value(value: Option<&str>) -> SqlBindValue {
    value
        .map(|value| SqlBindValue::Text(value.to_string()))
        .unwrap_or(SqlBindValue::Null)
}

fn numeric_stats_from_row(row: NumericStatsRow) -> atlas_domain::NumericFieldStats {
    atlas_domain::NumericFieldStats {
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

fn filter_field_stats_from_row(row: FilterFieldStatsRow) -> FilterFieldStats {
    FilterFieldStats {
        value_count: row.value_count as u64,
        distinct_count: row.distinct_count as u64,
        singleton_count: row.singleton_count as u64,
        singleton_ratio: row.singleton_ratio,
        observation_singleton_ratio: row.observation_singleton_ratio,
        null_count: row.null_count as u64,
    }
}

#[derive(QueryableByName)]
struct FilterFieldInfoRow {
    #[diesel(sql_type = Text)]
    field: String,
    #[diesel(sql_type = Text)]
    field_type: String,
    #[diesel(sql_type = Text)]
    field_group: String,
    #[diesel(sql_type = Text)]
    value_policy: String,
    #[diesel(sql_type = Text)]
    operators_json: String,
    #[diesel(sql_type = Text)]
    cli_flags_json: String,
    #[diesel(sql_type = Text)]
    applicable_families_json: String,
}

#[derive(QueryableByName)]
struct FilterValueCountRow {
    #[diesel(sql_type = Text)]
    value: String,
    #[diesel(sql_type = BigInt)]
    catalog_count: i64,
}

#[derive(QueryableByName)]
struct NumericStatsRow {
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

#[derive(QueryableByName)]
struct FilterFieldStatsRow {
    #[diesel(sql_type = BigInt)]
    value_count: i64,
    #[diesel(sql_type = BigInt)]
    distinct_count: i64,
    #[diesel(sql_type = BigInt)]
    singleton_count: i64,
    #[diesel(sql_type = Double)]
    singleton_ratio: f64,
    #[diesel(sql_type = Double)]
    observation_singleton_ratio: f64,
    #[diesel(sql_type = BigInt)]
    null_count: i64,
}
