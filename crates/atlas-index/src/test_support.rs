//! Disposable test workspaces use the same complete schema as runtime composition.
use crate::{
    ArtifactValidationReport, IndexArtifactWriter, IndexBuildInput, IndexError, SqliteIndexReader,
    SqliteIndexWriter,
};
use atlas_storage::{Migration, Workspace};
use std::path::Path;
pub fn initialize_workspace(path: impl AsRef<Path>) -> Result<Workspace, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    let workspace = Workspace::at(path)?;
    workspace.migrate(
        &[
            Migration {
                version: 1,
                name: "reference-baseline",
                sql: crate::REFERENCE_SCHEMA_SQL,
            },
            Migration {
                version: 2,
                name: "saved-state-baseline",
                sql: atlas_local_state::LOCAL_STATE_SCHEMA_SQL,
            },
        ],
        |_| Ok(()),
    )?;
    Ok(workspace)
}
pub fn open_reader(path: impl AsRef<Path>) -> Result<SqliteIndexReader, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    SqliteIndexReader::from_session(Workspace::at(path)?.read(2)?)
}
pub fn publish_input(
    path: impl AsRef<Path>,
    input: &IndexBuildInput,
) -> Result<ArtifactValidationReport, IndexError> {
    let workspace = initialize_workspace(path)?;
    let owner = workspace.update_owner()?;
    let maintenance = owner.maintenance(2, "test publication")?;
    SqliteIndexWriter::new(&maintenance).write(input)
}

/// Model an older preparation contract coherently, including the allocation
/// digests used to decide whether publication can retain a record's rows.
pub fn set_content_interpretation_version(
    path: impl AsRef<Path>,
    version: &str,
) -> Result<(), IndexError> {
    set_preparation_policy(path, |context| {
        context.content_interpretation_version = version.into()
    })
}

pub fn set_content_selection_version(
    path: impl AsRef<Path>,
    version: &str,
) -> Result<(), IndexError> {
    set_preparation_policy(path, |context| {
        context.content_selection_version = version.into()
    })
}

fn set_preparation_policy(
    path: impl AsRef<Path>,
    update: impl FnOnce(&mut crate::SourceArtifactBuildContext),
) -> Result<(), IndexError> {
    let workspace = Workspace::at(path)?;
    let write = workspace.write(2)?;
    let c = write.connection();
    let mut context = crate::read_stored_reference_context(c)?
        .ok_or_else(|| IndexError::Invalid("fixture has no reference context".into()))?;
    update(&mut context);
    let json = serde_json::to_string(&context)?;
    c.execute(
        "UPDATE artifact_context SET context_hash=?1,context_json=?2",
        [crate::codec::sha256(json.as_bytes()), json],
    )?;
    c.execute(
        "UPDATE prepared_content SET preparation_context_hash=?",
        [context.preparation_sha256()?],
    )?;
    for source in crate::read_persisted_inputs(c, &context)? {
        let (id, label): (i64, String) = c.query_row(
            "SELECT r.record_id,p.label FROM records r JOIN packs p USING(pack_id) WHERE r.key=?",
            [source.record.key().to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        c.execute(
            "UPDATE projection_allocations SET input_hash=? WHERE record_id=?",
            rusqlite::params![crate::publication_delta::root_digest(&source, &label)?, id],
        )?;
    }
    write.commit()?;
    Ok(())
}
