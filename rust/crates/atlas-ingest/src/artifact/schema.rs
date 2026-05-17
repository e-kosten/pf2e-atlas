use atlas_artifact::schema::create_artifact_schema_sql;
use rusqlite::Connection;

use crate::error::IngestError;

pub(crate) fn create_artifact_schema(connection: &Connection) -> Result<(), IngestError> {
    connection
        .execute_batch(&create_artifact_schema_sql())
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}
