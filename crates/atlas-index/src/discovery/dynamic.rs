use atlas_domain::{
    BooleanFieldCounts, FilterFieldInfo, FilterSample, FilterValueCount, FilterValuePayload,
    FilterValuePolicy, SearchFilterNode,
};
use diesel::sql_types::{BigInt, Bool, Double, Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::filters::SqliteEligibleRecordKeyset;
use crate::sqlite::raw_sql::{CountRow, bind_sql_query};

use super::definitions::{FieldDefinition, all_definitions};
use super::error::{DiscoveryError, query_error};
use super::request::{DiscoveryValueSort, FilterValueRequest};
use super::stats;

pub(super) fn fields(
    connection: &mut SqliteConnection,
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
    connection: &mut SqliteConnection,
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
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
) -> Result<u64, DiscoveryError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .count_query();
    bind_sql_query(query.sql, &query.parameters)
        .get_result::<CountRow>(connection)
        .map(|row| row.count as u64)
        .map_err(query_error)
}

pub(super) fn field_applies(
    connection: &mut SqliteConnection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
) -> Result<bool, DiscoveryError> {
    let value_sql = definition.value_sql();
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|_| {
            format!(
                ", field_values(record_key, value) AS ({values})
         SELECT EXISTS (
           SELECT 1
           FROM field_values fv
           JOIN eligible e ON e.record_key = fv.record_key
           WHERE fv.value IS NOT NULL AND CAST(fv.value AS TEXT) <> ''
           LIMIT 1
         ) AS value",
                values = value_sql,
            )
        });
    bind_sql_query(query.sql, &query.parameters)
        .get_result::<BoolRow>(connection)
        .map(|row| row.value)
        .map_err(query_error)
}

fn enumerable_values(
    connection: &mut SqliteConnection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
    sort: DiscoveryValueSort,
) -> Result<(Vec<FilterValueCount>, u64), DiscoveryError> {
    let value_sql = definition.value_sql();
    let order = match sort {
        DiscoveryValueSort::Alpha | DiscoveryValueSort::Canonical => "value ASC",
        DiscoveryValueSort::Count => "catalog_count DESC, value ASC",
    };
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|_| {
            format!(
                ", field_values(record_key, value) AS ({values})
         SELECT value, COUNT(*) AS catalog_count
         FROM field_values fv
         JOIN eligible e ON e.record_key = fv.record_key
         WHERE value IS NOT NULL AND CAST(value AS TEXT) <> ''
         GROUP BY value
         ORDER BY {order}",
                values = value_sql,
            )
        });
    let values = bind_sql_query(query.sql, &query.parameters)
        .load::<FilterValueCountRow>(connection)
        .map_err(query_error)?
        .into_iter()
        .map(|row| FilterValueCount {
            value: row.value,
            count: row.catalog_count as u64,
        })
        .collect::<Vec<_>>();
    Ok((values, null_count(connection, definition, filter)?))
}

fn sample_values(
    connection: &mut SqliteConnection,
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
    connection: &mut SqliteConnection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
) -> Result<atlas_domain::NumericFieldStats, DiscoveryError> {
    let value_sql = definition.value_sql();
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|_| {
            format!(
                ", field_values(record_key, value) AS ({values})
         SELECT value
         FROM field_values fv
         JOIN eligible e ON e.record_key = fv.record_key
         WHERE value IS NOT NULL
         ORDER BY value ASC",
                values = value_sql,
            )
        });
    let values = bind_sql_query(query.sql, &query.parameters)
        .load::<NumericValueRow>(connection)
        .map_err(query_error)?
        .into_iter()
        .map(|row| row.value)
        .collect::<Vec<_>>();
    Ok(stats::numeric_stats_from_values(
        &values,
        count_matching_records(connection, filter)?,
    ))
}

fn boolean_counts(
    connection: &mut SqliteConnection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
) -> Result<BooleanFieldCounts, DiscoveryError> {
    let value_sql = definition.value_sql();
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|_| {
            format!(
                ", field_values(record_key, value) AS ({values})
         SELECT
           SUM(CASE WHEN value = 1 THEN 1 ELSE 0 END) AS true_count,
           SUM(CASE WHEN value = 0 THEN 1 ELSE 0 END) AS false_count,
           (SELECT COUNT(*) FROM eligible) - COUNT(value) AS null_count
         FROM field_values fv
         JOIN eligible e ON e.record_key = fv.record_key",
                values = value_sql,
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

fn null_count(
    connection: &mut SqliteConnection,
    definition: FieldDefinition,
    filter: Option<&SearchFilterNode>,
) -> Result<u64, DiscoveryError> {
    let value_sql = definition.value_sql();
    let query = SqliteEligibleRecordKeyset::new(filter).compile()?.with_eligible_cte(
        |_| {
            format!(
                ", field_values(record_key, value) AS ({values})
         SELECT COUNT(*) AS count
         FROM eligible e
         WHERE NOT EXISTS (
           SELECT 1 FROM field_values fv
           WHERE fv.record_key = e.record_key AND fv.value IS NOT NULL AND CAST(fv.value AS TEXT) <> ''
         )",
                values = value_sql,
            )
        },
    );
    bind_sql_query(query.sql, &query.parameters)
        .get_result::<CountRow>(connection)
        .map(|row| row.count as u64)
        .map_err(query_error)
}

#[derive(QueryableByName)]
struct BoolRow {
    #[diesel(sql_type = Bool)]
    value: bool,
}

#[derive(QueryableByName)]
struct FilterValueCountRow {
    #[diesel(sql_type = Text)]
    value: String,
    #[diesel(sql_type = BigInt)]
    catalog_count: i64,
}

#[derive(QueryableByName)]
struct NumericValueRow {
    #[diesel(sql_type = Double)]
    value: f64,
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
