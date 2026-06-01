use atlas_domain::{
    FilterFieldGroup, FilterFieldInfo, FilterFieldStats, FilterFieldType, FilterOperator,
    FilterSample, FilterValueCount, FilterValuePayload, FilterValuePolicy, RecordFamily,
};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{OptionalExtension, Queryable, Selectable, SelectableHelper, SqliteConnection};

use super::definitions::FieldDefinition;
use super::error::{DiscoveryError, query_error};
use super::request::{DiscoveryValueSort, FilterValueRequest};
use super::stats;
use crate::schema::{
    filter_field_catalog, filter_numeric_catalog, filter_sample_catalog, filter_value_catalog,
};

pub(super) fn fields(
    connection: &mut SqliteConnection,
    scope: Option<RecordFamily>,
) -> Result<Vec<FilterFieldInfo>, DiscoveryError> {
    let scope = scope.map(record_family_string);
    let mut query = filter_field_catalog::table.into_boxed();
    query = match scope.as_deref() {
        Some(scope) => query.filter(filter_field_catalog::record_family.eq(scope)),
        None => query.filter(filter_field_catalog::record_family.is_null()),
    };
    query
        .select(FilterFieldInfoRow::as_select())
        .order(filter_field_catalog::field.asc())
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
    let mut query = filter_value_catalog::table
        .filter(filter_value_catalog::field.eq(field))
        .into_boxed();
    query = match scope.as_deref() {
        Some(scope) => query.filter(filter_value_catalog::record_family.eq(scope)),
        None => query.filter(filter_value_catalog::record_family.is_null()),
    };
    let rows = match sort {
        DiscoveryValueSort::Alpha | DiscoveryValueSort::Canonical => query
            .select((
                filter_value_catalog::value,
                filter_value_catalog::catalog_count,
            ))
            .order(filter_value_catalog::value.asc())
            .load::<FilterValueCountRow>(connection),
        DiscoveryValueSort::Count => query
            .select((
                filter_value_catalog::value,
                filter_value_catalog::catalog_count,
            ))
            .order((
                filter_value_catalog::catalog_count.desc(),
                filter_value_catalog::value.asc(),
            ))
            .load::<FilterValueCountRow>(connection),
    };
    rows.map_err(query_error).map(|rows| {
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
    let mut query = filter_sample_catalog::table
        .filter(filter_sample_catalog::field.eq(field))
        .into_boxed();
    query = match scope.as_deref() {
        Some(scope) => query.filter(filter_sample_catalog::record_family.eq(scope)),
        None => query.filter(filter_sample_catalog::record_family.is_null()),
    };
    let examples = query
        .select((
            filter_sample_catalog::value,
            filter_sample_catalog::catalog_count,
        ))
        .order(filter_sample_catalog::sample_rank.asc())
        .limit(sample_limit as i64)
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
    let mut query = filter_numeric_catalog::table
        .filter(filter_numeric_catalog::field.eq(field))
        .filter(filter_numeric_catalog::metric_domain.is_null())
        .filter(filter_numeric_catalog::metric_key.is_null())
        .into_boxed();
    query = match scope.as_deref() {
        Some(scope) => query.filter(filter_numeric_catalog::record_family.eq(scope)),
        None => query.filter(filter_numeric_catalog::record_family.is_null()),
    };
    query
        .select(NumericStatsRow::as_select())
        .get_result::<NumericStatsRow>(connection)
        .optional()
        .map_err(query_error)?
        .map(numeric_stats_from_row)
        .ok_or_else(|| {
            DiscoveryError::FieldNotApplicable(format!(
                "field `{field}` has no numeric catalog rows"
            ))
        })
}

fn field_stats(
    connection: &mut SqliteConnection,
    field: &str,
    scope: Option<&str>,
) -> Result<FilterFieldStats, DiscoveryError> {
    let mut query = filter_field_catalog::table
        .filter(filter_field_catalog::field.eq(field))
        .into_boxed();
    query = match scope {
        Some(scope) => query.filter(filter_field_catalog::record_family.eq(scope)),
        None => query.filter(filter_field_catalog::record_family.is_null()),
    };
    query
        .select(FilterFieldStatsRow::as_select())
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
    let mut query = filter_field_catalog::table
        .filter(filter_field_catalog::field.eq(field))
        .into_boxed();
    query = match scope.as_deref() {
        Some(scope) => query.filter(filter_field_catalog::record_family.eq(scope)),
        None => query.filter(filter_field_catalog::record_family.is_null()),
    };
    query
        .select(filter_field_catalog::null_count)
        .get_result::<i64>(connection)
        .map(|count| count as u64)
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
        singleton_ratio: row.singleton_ratio.unwrap_or(0.0),
        observation_singleton_ratio: row.observation_singleton_ratio.unwrap_or(0.0),
        null_count: row.null_count as u64,
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = filter_field_catalog)]
#[diesel(check_for_backend(Sqlite))]
struct FilterFieldInfoRow {
    field: String,
    field_type: String,
    field_group: String,
    value_policy: String,
    operators_json: String,
    cli_flags_json: String,
    applicable_families_json: String,
}

#[derive(Queryable)]
struct FilterValueCountRow {
    value: String,
    catalog_count: i64,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = filter_numeric_catalog)]
#[diesel(check_for_backend(Sqlite))]
struct NumericStatsRow {
    catalog_count: i64,
    null_count: i64,
    min: Option<f64>,
    p05: Option<f64>,
    p25: Option<f64>,
    p50: Option<f64>,
    mean: Option<f64>,
    p75: Option<f64>,
    p95: Option<f64>,
    max: Option<f64>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = filter_field_catalog)]
#[diesel(check_for_backend(Sqlite))]
struct FilterFieldStatsRow {
    value_count: i64,
    distinct_count: i64,
    singleton_count: i64,
    singleton_ratio: Option<f64>,
    observation_singleton_ratio: Option<f64>,
    null_count: i64,
}
