//! Fixture publication through the runtime-owned workspace composition.
use atlas_index::{
    ArtifactValidationReport, IndexArtifactWriter, IndexBuildInput, SqliteIndexWriter,
};

pub fn publish_input(
    path: impl AsRef<std::path::Path>,
    input: &IndexBuildInput,
) -> Result<ArtifactValidationReport, Box<dyn std::error::Error>> {
    crate::initialize_workspace(path.as_ref())?;
    let workspace = atlas_storage::Workspace::at(path)?;
    let owner = workspace.update_owner()?;
    let maintenance = owner.maintenance(
        crate::WORKSPACE_SCHEMA_VERSION,
        "runtime fixture publication",
    )?;
    Ok(SqliteIndexWriter::new(&maintenance).write(input)?)
}

/// Read a composed runtime fixture using the current admission contract.
pub fn open_reader(
    path: impl AsRef<std::path::Path>,
) -> Result<atlas_index::SqliteIndexReader, Box<dyn std::error::Error>> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()?;
    let session = atlas_storage::Workspace::at(path)?.read(crate::WORKSPACE_SCHEMA_VERSION)?;
    Ok(atlas_index::SqliteIndexReader::from_session(session)?)
}
