use rusqlite::Connection;

pub(super) fn probe_sqlite_vec(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "CREATE VIRTUAL TABLE temp.atlas_vec_capability_probe
             USING vec0(embedding FLOAT[1]);
             DROP TABLE temp.atlas_vec_capability_probe;",
        )
        .map_err(|error| error.to_string())
}

pub(crate) fn register_sqlite_vec_extension() -> Result<(), String> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().map_err(|error| error.to_string())
}
