use atlas_domain::{
    FilterFieldGroup, FilterFieldInfo, FilterFieldStats, FilterFieldType, FilterOperator,
    FilterSample, FilterValueCount, FilterValuePayload, FilterValuePolicy, RecordFamily,
};
use rusqlite::{Connection, OptionalExtension, params};

use super::definitions::FieldDefinition;
use super::error::{DiscoveryError, query_error};
use super::request::{DiscoveryValueSort, FilterValueRequest};
use super::stats;

pub(super) fn fields(
    connection: &Connection,
    scope: Option<RecordFamily>,
) -> Result<Vec<FilterFieldInfo>, DiscoveryError> {
    let mut statement = connection
        .prepare(
            "SELECT field, field_type, field_group, value_policy, operators_json, cli_flags_json,
                    applicable_families_json
             FROM filter_field_catalog
             WHERE (record_family IS NULL AND ?1 IS NULL) OR record_family = ?1
             ORDER BY field",
        )
        .map_err(query_error)?;
    let scope = scope.map(record_family_string);
    statement
        .query_map(params![scope], field_info_from_row)
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)
}

pub(super) fn values(
    connection: &Connection,
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
    connection: &Connection,
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
    let mut statement = connection.prepare(&sql).map_err(query_error)?;
    statement
        .query_map(params![field, scope], |row| {
            Ok(FilterValueCount {
                value: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)
}

fn sample_values(
    connection: &Connection,
    field: &str,
    scope: Option<RecordFamily>,
    sample_limit: usize,
) -> Result<FilterValuePayload, DiscoveryError> {
    let scope = scope.map(record_family_string);
    let stats = field_stats(connection, field, scope.as_deref())?;
    let mut statement = connection
        .prepare(
            "SELECT value, catalog_count
             FROM filter_sample_catalog
             WHERE field = ?1 AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)
             ORDER BY sample_rank ASC
             LIMIT ?3",
        )
        .map_err(query_error)?;
    let examples = statement
        .query_map(params![field, scope, sample_limit as u64], |row| {
            Ok(stats::sample_example(&FilterValueCount {
                value: row.get(0)?,
                count: row.get(1)?,
            }))
        })
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)?;
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
    connection: &Connection,
    field: &str,
    scope: Option<RecordFamily>,
) -> Result<atlas_domain::NumericFieldStats, DiscoveryError> {
    let scope = scope.map(record_family_string);
    connection
        .query_row(
            "SELECT catalog_count, null_count, min, p05, p25, p50, mean, p75, p95, max
             FROM filter_numeric_catalog
             WHERE field = ?1
               AND metric_domain IS NULL
               AND metric_key IS NULL
               AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)",
            params![field, scope],
            stats::numeric_stats_from_row,
        )
        .optional()
        .map_err(query_error)?
        .ok_or_else(|| {
            DiscoveryError::FieldNotApplicable(format!(
                "field `{field}` has no numeric catalog rows"
            ))
        })
}

fn field_stats(
    connection: &Connection,
    field: &str,
    scope: Option<&str>,
) -> Result<FilterFieldStats, DiscoveryError> {
    connection
        .query_row(
            "SELECT value_count, distinct_count, singleton_count, singleton_ratio,
                    observation_singleton_ratio, null_count
             FROM filter_field_catalog
             WHERE field = ?1 AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)",
            params![field, scope],
            |row| {
                Ok(FilterFieldStats {
                    value_count: row.get(0)?,
                    distinct_count: row.get(1)?,
                    singleton_count: row.get(2)?,
                    singleton_ratio: row.get(3)?,
                    observation_singleton_ratio: row.get(4)?,
                    null_count: row.get(5)?,
                })
            },
        )
        .map_err(query_error)
}

fn field_null_count(
    connection: &Connection,
    field: &str,
    scope: Option<RecordFamily>,
) -> Result<u64, DiscoveryError> {
    let scope = scope.map(record_family_string);
    connection
        .query_row(
            "SELECT null_count
             FROM filter_field_catalog
             WHERE field = ?1 AND ((record_family IS NULL AND ?2 IS NULL) OR record_family = ?2)",
            params![field, scope],
            |row| row.get(0),
        )
        .map_err(query_error)
}

fn field_info_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FilterFieldInfo> {
    Ok(FilterFieldInfo {
        field: row.get(0)?,
        field_type: parse_field_type(row.get::<_, String>(1)?.as_str()),
        group: parse_field_group(row.get::<_, String>(2)?.as_str()),
        value_policy: parse_value_policy(row.get::<_, String>(3)?.as_str()),
        operators: parse_json_operators(&row.get::<_, String>(4)?),
        cli_flags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
        applicable_families: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
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
