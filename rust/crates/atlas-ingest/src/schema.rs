use atlas_artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;
use rusqlite::Connection;

use crate::IngestError;

pub(crate) fn create_artifact_schema(connection: &Connection) -> Result<(), IngestError> {
    connection
        .execute_batch(CREATE_ARTIFACT_SCHEMA_SQL)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}
