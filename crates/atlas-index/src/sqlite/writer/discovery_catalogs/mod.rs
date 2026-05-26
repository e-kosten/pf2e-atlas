mod field_seeds;
mod fields;
mod stats;
mod values;

use rusqlite::Connection;
use tracing::info;

use crate::IndexWriteError;

pub(super) fn write_discovery_catalogs(connection: &Connection) -> Result<(), IndexWriteError> {
    fields::write_field_catalogs(connection)?;
    values::write_value_catalogs(connection)
}

pub(super) fn progress(phase: &'static str, current: u64, total: u64, message: String) {
    info!(target: "atlas_progress", phase, current, total, "{message}");
}
