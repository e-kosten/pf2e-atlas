use atlas_artifact::schema::{
    metric_key_catalog_insert_select_sql, metric_value_catalog_insert_select_sql,
};
use rusqlite::Connection;

use crate::error::IngestError;

pub(super) fn write_metric_catalogs(connection: &Connection) -> Result<(), IngestError> {
    connection
        .execute_batch(&format!(
            "{};
             {};",
            metric_key_catalog_insert_select_sql(),
            metric_value_catalog_insert_select_sql()
        ))
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}
