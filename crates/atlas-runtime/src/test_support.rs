//! Fixture publication through the runtime-owned workspace composition.
use atlas_index::IndexBuildInput;

pub fn publish_input(
    path: impl AsRef<std::path::Path>,
    input: &IndexBuildInput,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::initialize_workspace(path.as_ref())?;
    let workspace = atlas_storage::Workspace::at(path)?;
    Ok(atlas_index::test_support::publish_input_in(
        &workspace,
        crate::WORKSPACE_SCHEMA_VERSION,
        input,
    )?)
}

/// Read a composed runtime fixture using the current admission contract.
pub fn open_reader(
    path: impl AsRef<std::path::Path>,
) -> Result<atlas_index::SqliteIndexReader, Box<dyn std::error::Error>> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()?;
    let session = atlas_storage::Workspace::at(path)?.read(crate::WORKSPACE_SCHEMA_VERSION)?;
    Ok(atlas_index::SqliteIndexReader::from_session(session)?)
}

/// Literal model identity and ordered occurrence bytes for fixture equality.
#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OccurrenceVectorSnapshot {
    identity_json: String,
    rows: Vec<(i64, Vec<u8>)>,
}

/// Compare attributed numerical values in admitted workspaces.
/// This fixture oracle expands input-owned rows back to the retained occurrence IDs.
#[cfg(test)]
pub(crate) fn occurrence_vector_bits(
    connection: &rusqlite::Connection,
) -> Vec<OccurrenceVectorSnapshot> {
    let query = "SELECT p.profile_id,m.identity_json FROM search_profiles p JOIN embedding_models m USING(model_id) ORDER BY p.profile_id";
    let profiles = connection
        .prepare(query)
        .unwrap()
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    profiles.into_iter().map(|(profile,json)| {
        let identity: atlas_index::SourceSearchProfileIdentity = serde_json::from_str(&json).unwrap();
        let digest = atlas_search::test_support::hash(&serde_json::to_vec(&identity).unwrap());
        let native = format!("native_vectors_{digest}");
        let sql = format!("SELECT u.unit_id,v.embedding FROM semantic_units u LEFT JOIN {native} v USING(input_id) WHERE u.profile_id=? ORDER BY u.unit_id");
        let rows = connection.prepare(&sql).unwrap()
            .query_map([profile], |row| Ok((row.get(0)?,row.get(1)?))).unwrap()
            .collect::<Result<Vec<_>,_>>().unwrap();
        OccurrenceVectorSnapshot { identity_json: json, rows }
    }).collect()
}

#[cfg(test)]
pub(crate) fn sql_rows(
    connection: &rusqlite::Connection,
    sql: &str,
) -> Vec<Vec<rusqlite::types::Value>> {
    let mut statement = connection.prepare(sql).unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            (0..columns).map(|column| row.get(column)).collect()
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}
