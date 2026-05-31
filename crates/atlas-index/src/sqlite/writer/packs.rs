use diesel::SqliteConnection;
use diesel::prelude::*;

use super::models::PackRow;
use crate::IndexBuildPack;
use crate::IndexWriteError;

pub(super) fn write_packs(
    connection: &mut SqliteConnection,
    packs: &[IndexBuildPack<'_>],
) -> Result<(), IndexWriteError> {
    let rows = packs
        .iter()
        .map(|pack| {
            Ok::<PackRow, IndexWriteError>(PackRow {
                name: pack.name.to_string(),
                label: pack.label.to_string(),
                document_type: pack.document_type.to_string(),
                declared_path: pack.declared_path.to_string(),
                resolved_path: pack.resolved_path.display().to_string(),
                record_count: i64::try_from(pack.record_count).map_err(|_| {
                    IndexWriteError::WriteFailed(format!(
                        "pack `{}` record count does not fit in SQLite INTEGER",
                        pack.name
                    ))
                })?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    diesel::insert_into(crate::schema::packs::table)
        .values(&rows)
        .execute(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    Ok(())
}
