use atlas_artifact::schema::pack_insert_sql;
use rusqlite::Connection;

use crate::IndexBuildPack;
use crate::IndexWriteError;

pub(super) fn write_packs(
    connection: &Connection,
    packs: &[IndexBuildPack<'_>],
) -> Result<(), IndexWriteError> {
    let mut statement = connection
        .prepare(&pack_insert_sql())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    for pack in packs {
        statement
            .execute((
                pack.name.to_string(),
                pack.label,
                pack.document_type,
                pack.declared_path,
                pack.resolved_path.display().to_string(),
                pack.record_count,
            ))
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}
