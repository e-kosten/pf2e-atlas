use atlas_artifact::schema::pack_insert_sql;
use rusqlite::Connection;

use crate::error::IngestError;
use crate::source::LoadedPack;

pub(super) fn write_packs(
    connection: &Connection,
    packs: &[LoadedPack],
) -> Result<(), IngestError> {
    let mut statement = connection
        .prepare(&pack_insert_sql())
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    for pack in packs {
        statement
            .execute((
                pack.name.as_str(),
                pack.label.as_str(),
                pack.document_type.as_str(),
                pack.declared_path.as_str(),
                pack.resolved_path.display().to_string(),
                pack.record_count,
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}
