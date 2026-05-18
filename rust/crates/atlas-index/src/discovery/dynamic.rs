use atlas_domain::{
    BooleanFieldCounts, FilterFieldInfo, FilterSample, FilterValueCount, FilterValuePayload,
    FilterValuePolicy, SearchFilterNode,
};
use rusqlite::{Connection, params_from_iter};

use crate::filters::compile_eligible_records_query;

use super::definitions::{FieldDefinition, all_definitions};
use super::error::{DiscoveryError, query_error};
use super::request::{DiscoveryValueSort, FilterValueRequest};
use super::stats;

pub(super) fn fields(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
) -> Result<Vec<FilterFieldInfo>, DiscoveryError> {
    let mut fields = Vec::new();
    for definition in all_definitions() {
        if field_applies(connection, *definition, filter)? {
            fields.push(definition.info(false));
        }
    }
    Ok(fields)
}

pub(super) fn values(
    connection: &Connection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
    request: &FilterValueRequest,
) -> Result<FilterValuePayload, DiscoveryError> {
    match definition.value_policy {
        FilterValuePolicy::Enumerable => {
            let sort = request.sort.unwrap_or(definition.default_sort);
            let (values, null_count) = enumerable_values(connection, definition, filter, sort)?;
            Ok(FilterValuePayload::Enumerable {
                values,
                null_count,
                sort,
            })
        }
        FilterValuePolicy::Sample => sample_values(connection, definition, filter, request),
        FilterValuePolicy::NumericStats => Ok(FilterValuePayload::NumericStats {
            stats: numeric_stats(connection, definition, filter)?,
        }),
        FilterValuePolicy::BooleanCounts => Ok(FilterValuePayload::BooleanCounts {
            counts: boolean_counts(connection, definition, filter)?,
        }),
        _ => Err(DiscoveryError::InvalidOption(format!(
            "field `{}` is not a metadata value field",
            definition.field
        ))),
    }
}

pub(super) fn count_matching_records(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
) -> Result<u64, DiscoveryError> {
    let eligible = compile_eligible_records_query(filter)?;
    let sql = format!(
        "WITH eligible(record_key) AS ({}) SELECT COUNT(*) FROM eligible",
        eligible.sql
    );
    connection
        .query_row(&sql, params_from_iter(eligible.parameters.iter()), |row| {
            row.get(0)
        })
        .map_err(query_error)
}

pub(super) fn field_applies(
    connection: &Connection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
) -> Result<bool, DiscoveryError> {
    let eligible = compile_eligible_records_query(filter)?;
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible}), field_values(record_key, value) AS ({values})
         SELECT EXISTS (
           SELECT 1
           FROM field_values fv
           JOIN eligible e ON e.record_key = fv.record_key
           WHERE fv.value IS NOT NULL AND CAST(fv.value AS TEXT) <> ''
           LIMIT 1
         )",
        eligible = eligible.sql,
        values = definition.value_sql,
    );
    connection
        .query_row(&sql, params_from_iter(eligible.parameters.iter()), |row| {
            row.get::<_, bool>(0)
        })
        .map_err(query_error)
}

fn enumerable_values(
    connection: &Connection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
    sort: DiscoveryValueSort,
) -> Result<(Vec<FilterValueCount>, u64), DiscoveryError> {
    let eligible = compile_eligible_records_query(filter)?;
    let order = match sort {
        DiscoveryValueSort::Alpha | DiscoveryValueSort::Canonical => "value ASC",
        DiscoveryValueSort::Count => "catalog_count DESC, value ASC",
    };
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible}), field_values(record_key, value) AS ({values})
         SELECT value, COUNT(*) AS catalog_count
         FROM field_values fv
         JOIN eligible e ON e.record_key = fv.record_key
         WHERE value IS NOT NULL AND CAST(value AS TEXT) <> ''
         GROUP BY value
         ORDER BY {order}",
        eligible = eligible.sql,
        values = definition.value_sql,
    );
    let mut statement = connection.prepare(&sql).map_err(query_error)?;
    let values = statement
        .query_map(params_from_iter(eligible.parameters.iter()), |row| {
            Ok(FilterValueCount {
                value: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)?;
    Ok((values, null_count(connection, definition, filter)?))
}

fn sample_values(
    connection: &Connection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
    request: &FilterValueRequest,
) -> Result<FilterValuePayload, DiscoveryError> {
    let sample_limit = request.sample_limit.unwrap_or(20);
    let (values, null_count) =
        enumerable_values(connection, definition, filter, DiscoveryValueSort::Count)?;
    let mut field_stats = stats::stats_from_counts(&values);
    field_stats.null_count = null_count;
    let examples = values
        .iter()
        .take(sample_limit)
        .map(stats::sample_example)
        .collect::<Vec<_>>();
    Ok(FilterValuePayload::Sample {
        sample: FilterSample {
            selection: "top_repeated_then_deterministic".to_string(),
            sample_limit,
            distinct_count: field_stats.distinct_count,
            omitted_distinct_count: field_stats
                .distinct_count
                .saturating_sub(examples.len() as u64),
            examples,
        },
        field_stats,
        null_count,
    })
}

fn numeric_stats(
    connection: &Connection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
) -> Result<atlas_domain::NumericFieldStats, DiscoveryError> {
    let eligible = compile_eligible_records_query(filter)?;
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible}), field_values(record_key, value) AS ({values})
         SELECT value
         FROM field_values fv
         JOIN eligible e ON e.record_key = fv.record_key
         WHERE value IS NOT NULL
         ORDER BY value ASC",
        eligible = eligible.sql,
        values = definition.value_sql,
    );
    let mut statement = connection.prepare(&sql).map_err(query_error)?;
    let values = statement
        .query_map(params_from_iter(eligible.parameters.iter()), |row| {
            row.get::<_, f64>(0)
        })
        .map_err(query_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(query_error)?;
    Ok(stats::numeric_stats_from_values(
        &values,
        count_matching_records(connection, filter)?,
    ))
}

fn boolean_counts(
    connection: &Connection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
) -> Result<BooleanFieldCounts, DiscoveryError> {
    let eligible = compile_eligible_records_query(filter)?;
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible}), field_values(record_key, value) AS ({values})
         SELECT
           SUM(CASE WHEN value = 1 THEN 1 ELSE 0 END),
           SUM(CASE WHEN value = 0 THEN 1 ELSE 0 END),
           (SELECT COUNT(*) FROM eligible) - COUNT(value)
         FROM field_values fv
         JOIN eligible e ON e.record_key = fv.record_key",
        eligible = eligible.sql,
        values = definition.value_sql,
    );
    connection
        .query_row(&sql, params_from_iter(eligible.parameters.iter()), |row| {
            Ok(BooleanFieldCounts {
                r#true: row.get::<_, Option<u64>>(0)?.unwrap_or(0),
                r#false: row.get::<_, Option<u64>>(1)?.unwrap_or(0),
                null: row.get::<_, Option<u64>>(2)?.unwrap_or(0),
            })
        })
        .map_err(query_error)
}

fn null_count(
    connection: &Connection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
) -> Result<u64, DiscoveryError> {
    let eligible = compile_eligible_records_query(filter)?;
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible}), field_values(record_key, value) AS ({values})
         SELECT COUNT(*)
         FROM eligible e
         WHERE NOT EXISTS (
           SELECT 1 FROM field_values fv
           WHERE fv.record_key = e.record_key AND fv.value IS NOT NULL AND CAST(fv.value AS TEXT) <> ''
         )",
        eligible = eligible.sql,
        values = definition.value_sql,
    );
    connection
        .query_row(&sql, params_from_iter(eligible.parameters.iter()), |row| {
            row.get(0)
        })
        .map_err(query_error)
}
