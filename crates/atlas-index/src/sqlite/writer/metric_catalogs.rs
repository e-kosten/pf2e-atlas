use atlas_artifact::schema::{
    metric_key_catalog_insert_select_sql, metric_value_catalog_insert_select_sql,
};
use rusqlite::Connection;

use crate::IndexWriteError;

pub(super) fn write_metric_catalogs(connection: &Connection) -> Result<(), IndexWriteError> {
    connection
        .execute_batch(&format!(
            "{};
             {};",
            metric_key_catalog_insert_select_sql(),
            metric_value_catalog_insert_select_sql()
        ))
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}
