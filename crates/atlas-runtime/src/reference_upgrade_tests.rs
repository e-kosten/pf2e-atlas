//! The glossary owner upgrade preserves retained records, traits and saved state.
use super::*;
use crate::test_support::{occurrence_vector_bits, sql_rows};

#[test]
fn glossary_upgrade_is_atomic_and_keeps_source_free_trait_and_record_reads() {
    let (_directory, workspace, input) = super::native_storage_tests::main24(1);
    workspace.migrate(&MIGRATIONS[..29], |_| Ok(())).unwrap();
    let read = workspace.read(29).unwrap();
    let before_traits = sql_rows(
        read.connection(),
        "SELECT key,definition_json,definition_hash FROM trait_definitions ORDER BY key",
    );
    let before_vectors = sql_rows(
        read.connection(),
        "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id",
    );
    let before_chunks = sql_rows(
        read.connection(),
        "SELECT * FROM trait_seed_chunks ORDER BY trait_key,profile_id,ordinal",
    );
    let before_native = occurrence_vector_bits(read.connection());
    let before_saved = sql_rows(read.connection(), "SELECT * FROM saved_lists ORDER BY id");
    let before_ledger = sql_rows(
        read.connection(),
        "SELECT * FROM schema_migrations ORDER BY version",
    );
    let before_generation = read.reference_revision().unwrap();
    assert!(!before_traits.is_empty());
    assert!(!before_vectors.is_empty());
    drop(read);

    // A failure after the reference transformation must restore the released owner.
    let failure = workspace.migrate(MIGRATIONS, |_| {
        Err(atlas_storage::StorageError::Migration(
            "injected glossary validation failure".into(),
        ))
    });
    assert!(failure.is_err());
    let read = workspace.read(29).unwrap();
    assert_eq!(
        sql_rows(
            read.connection(),
            "SELECT * FROM schema_migrations ORDER BY version"
        ),
        before_ledger
    );
    assert_eq!(
        sql_rows(
            read.connection(),
            "SELECT key,definition_json,definition_hash FROM trait_definitions ORDER BY key"
        ),
        before_traits
    );
    assert_eq!(
        sql_rows(
            read.connection(),
            "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id"
        ),
        before_vectors
    );
    assert_eq!(read.reference_revision().unwrap(), before_generation);
    drop(read);

    let parent = workspace.path().parent().unwrap();
    let runtime = crate::AtlasRuntime::resolve(crate::AtlasRuntimeOptions {
        path_mode: crate::AtlasPathMode::Global,
        overrides: crate::AtlasPathOverrides {
            index_path: Some(workspace.path().to_owned()),
            source_root: Some(parent.join("source")),
            embedding_cache_root: Some(parent.join("models")),
        },
        ..Default::default()
    })
    .unwrap();
    runtime.upgrade_workspace_if_present().unwrap();
    let upgraded = Workspace::at(workspace.path()).unwrap();
    let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        sql_rows(
            read.connection(),
            "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id"
        ),
        before_vectors
    );
    assert_eq!(
        sql_rows(
            read.connection(),
            "SELECT * FROM trait_seed_chunks ORDER BY trait_key,profile_id,ordinal"
        ),
        before_chunks
    );
    assert_eq!(occurrence_vector_bits(read.connection()), before_native);
    assert_eq!(
        sql_rows(read.connection(), "SELECT * FROM saved_lists ORDER BY id"),
        before_saved
    );
    assert_eq!(read.reference_revision().unwrap(), before_generation + 1);
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='trait_definitions'",
                [],
                |row| row.get::<_, usize>(0)
            )
            .unwrap(),
        0
    );
    drop(read);

    let reader = open_workspace_index(workspace.path()).unwrap();
    assert_eq!(
        reader.reference_coverage(),
        atlas_index::ReferenceCoverage::TraitsOnly
    );
    let key = atlas_domain::TraitKey::new("fire").unwrap();
    let definition = reader.trait_definition(&key).unwrap().unwrap();
    assert_eq!(definition, input.traits[0]);
    assert!(
        reader
            .read_summary(input.records[0].record.key())
            .unwrap()
            .is_some()
    );
    assert!(
        reader
            .read_source_record(input.records[0].record.key())
            .unwrap()
            .is_some()
    );
    reader.require_vectors().unwrap();
    assert!(runtime.inference.loaded_profiles().is_empty());
    // The fixture has no external source tree or model package at any point.
    assert!(!workspace.path().parent().unwrap().join("source").exists());
    assert!(!workspace.path().parent().unwrap().join("models").exists());
}
