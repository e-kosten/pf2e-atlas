use diesel::SqliteConnection;
use diesel::prelude::*;

use super::models::PackRow;
use super::output::sqlite_payload_path;
use crate::IndexBuildPack;
use crate::IndexWriteError;

pub(super) fn write_packs(
    connection: &mut SqliteConnection,
    packs: &[IndexBuildPack],
) -> Result<(), IndexWriteError> {
    let rows = packs
        .iter()
        .map(|pack| {
            Ok::<PackRow, IndexWriteError>(PackRow {
                name: pack.name.to_string(),
                label: pack.label.to_string(),
                document_type: pack.document_type.to_string(),
                declared_path: pack.declared_path.to_string(),
                resolved_path: sqlite_payload_path(&pack.resolved_path, "pack resolved")?,
                record_count: i64::try_from(pack.record_count).map_err(|_| {
                    IndexWriteError::WriteFailed(format!(
                        "pack `{}` record count does not fit in SQLite INTEGER",
                        pack.name
                    ))
                })?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for rows in rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::packs::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}
