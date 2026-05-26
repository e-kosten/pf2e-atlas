use atlas_artifact::schema::create_artifact_schema_sql;
use rusqlite::Connection;

use crate::IndexWriteError;

pub(crate) fn create_artifact_schema(connection: &Connection) -> Result<(), IndexWriteError> {
    connection
        .execute_batch(&create_artifact_schema_sql())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}
