mod field_seeds;
mod fields;
mod stats;
mod values;

use rusqlite::Connection;

use crate::error::IngestError;

pub(super) fn write_discovery_catalogs(connection: &Connection) -> Result<(), IngestError> {
    fields::write_field_catalogs(connection)?;
    values::write_value_catalogs(connection)
}
