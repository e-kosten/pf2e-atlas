use atlas_domain::{RecordFamily, SearchFilterNode};
use diesel::RunQueryDsl;
use diesel::SqliteConnection;

use crate::read::discovery::error::{DiscoveryError, query_error};
use crate::read::sql::{CountRow, SqlBindValue, bind_sql_query};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MetricCatalogScope {
    Global,
    Family(RecordFamily),
}

pub(super) fn metric_catalog_scope(
    filter: Option<&SearchFilterNode>,
) -> Option<MetricCatalogScope> {
    match filter {
        None => Some(MetricCatalogScope::Global),
        Some(SearchFilterNode::RecordFamily { value }) => Some(MetricCatalogScope::Family(*value)),
        _ => None,
    }
}

pub(super) fn push_catalog_scope_predicate(
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

pub(super) fn matching_count_for_catalog_scope(
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
