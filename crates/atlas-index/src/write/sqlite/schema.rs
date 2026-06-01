use diesel::SqliteConnection;
use diesel::connection::SimpleConnection;

use crate::IndexWriteError;
use crate::artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;

pub(crate) fn create_artifact_schema(
    connection: &mut SqliteConnection,
) -> Result<(), IndexWriteError> {
    connection
        .batch_execute(CREATE_ARTIFACT_SCHEMA_SQL)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}
