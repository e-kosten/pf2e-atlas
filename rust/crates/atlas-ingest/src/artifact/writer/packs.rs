use rusqlite::Connection;

use crate::error::IngestError;
use crate::source::LoadedPack;

pub(super) fn write_packs(
    connection: &Connection,
    packs: &[LoadedPack],
) -> Result<(), IngestError> {
    let mut statement = connection
        .prepare(
            "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
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
