//! Owned publication preserves admission identities and rolls back malformed consumed frames.
use crate::{test_support::*, *};
use atlas_record::source_record::select_record_search;
use rusqlite::{Connection, types::Value};
use std::path::Path;

fn fixture() -> IndexBuildInput {
    let mut input = crate::foundation_tests::fixture(false);
    input.records.reverse();
    input.records.insert(
        1,
        crate::foundation_tests::record(
            "items",
            "Item",
            serde_json::json!({"_id":"zzzzzzzzzzzzzzzz","type":"effect"}),
        ),
    );
    input.records.push(crate::foundation_tests::record(
        "items",
        "Item",
        serde_json::json!({"_id":"eeeeeeeeeeeeeeee","type":"effect","name":""}),
    ));
    input.context.source_file_count = input.records.len();
    input.lexical_units.clear();
    for source in &mut input.records {
        source.preparation_context_hash = input.context.preparation_sha256().unwrap();
        let label = &input
            .packs
            .iter()
            .find(|p| p.pack_id == source.record.key().pack().as_str())
            .unwrap()
            .label;
        let selection = select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            label,
            &input.context.used_trait_labels,
        )
        .unwrap();
        input
            .lexical_units
            .extend(project_lexical_units(&selection, &[]));
    }
    input
}
fn spool(
    scratch: &Path,
    input: IndexBuildInput,
    expected: ArtifactInstance,
    damage: Option<&str>,
) -> PublicationSpool {
    let mut builder = PublicationSpool::new_in(
        scratch,
        input.context,
        input.packs,
        input.inventory,
        expected,
    )
    .unwrap();
    for source in input.records {
        let label = &builder
            .packs
            .iter()
            .find(|pack| pack.pack_id == source.record.key().pack().as_str())
            .unwrap()
            .label;
        let selection = select_record_search(
            &source.record,
            &source.content,
            builder.context.audience,
            label,
            &builder.context.used_trait_labels,
        )
        .unwrap();
        let lexical = input
            .lexical_units
            .iter()
            .filter(|row| row.record == *source.record.key())
            .cloned()
            .collect();
        builder
            .push_record(PreparedRecordInput {
                source,
                selection,
                lexical,
            })
            .unwrap();
    }
    if let Some(damage) = damage {
        let mut altered = BuildFrameStream::new_in(scratch).unwrap();
        let mut frames = builder.records.cursor().unwrap();
        let count = builder.record_count();
        for ordinal in 0..count {
            let mut frame = frames.next_frame().unwrap().unwrap();
            if ordinal + 1 == count {
                frame.snapshot = match damage {
                    "contract" => frame.snapshot.replace(
                        atlas_foundry_model::SOURCE_CONTRACT_ID,
                        "retired-fixture-contract",
                    ),
                    "root" => frame
                        .snapshot
                        .replace("eeeeeeeeeeeeeeee", "qqqqqqqqqqqqqqqq"),
                    other => panic!("unknown corruption {other}"),
                };
            }
            altered.push(&frame).unwrap();
        }
        builder.records = altered;
    }
    for row in input.lexical_units {
        builder.lexical.push(&row).unwrap();
    }
    builder
}
fn origin(workspace: &atlas_storage::Workspace) -> ArtifactInstance {
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    ArtifactInstance::new(
        read.workspace_key().unwrap(),
        read.reference_revision().unwrap(),
    )
}

fn stable_rows(c: &Connection) -> std::collections::BTreeMap<String, Vec<Vec<Value>>> {
    reference_preservation_rows(c).unwrap()
}
#[test]
fn owned_publication_preserves_stream_order_optional_names_macro_exclusion_and_saved_rows() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("identities.sqlite")).unwrap();
    let mut previous = None;
    for iteration in 0..2 {
        let owner = workspace.update_owner().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let source = spool(scratch.path(), fixture(), origin(&workspace), None);
        let publication = PreparedPublication::new_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            source,
        )
        .unwrap();
        publication.publish().unwrap();
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let c = read.connection();
        let roots: usize = c
            .query_row("SELECT count(*) FROM records", [], |row| row.get(0))
            .unwrap();
        assert_eq!(roots, 6);
        let identities = c
            .prepare("SELECT key,last_known_name FROM record_identity ORDER BY id")
            .unwrap()
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            identities,
            vec![
                (
                    "macros:dddddddddddddddd".into(),
                    Some("Hidden Dragon".into())
                ),
                ("items:zzzzzzzzzzzzzzzz".into(), None),
                ("items:cccccccccccccccc".into(), Some("Dragon Spell".into())),
                ("items:bbbbbbbbbbbbbbbb".into(), Some("Dragon Staff".into())),
                (
                    "actors:aaaaaaaaaaaaaaaa".into(),
                    Some("Dragon Guard".into())
                ),
                ("items:eeeeeeeeeeeeeeee".into(), Some("".into())),
            ]
        );
        assert_eq!(c.query_row("SELECT count(*) FROM core_roots WHERE record_id=(SELECT id FROM record_identity WHERE key='macros:dddddddddddddddd')",[],|r|r.get::<_,usize>(0)).unwrap(),0);
        if let Some(previous) = &previous {
            assert_eq!(&stable_rows(c), previous);
        }
        drop(read);
        if iteration == 0 {
            let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            write.connection().execute_batch("INSERT INTO saved_lists(id,owner_id,list_key,slug,name,created_at,updated_at) VALUES(1,1,'kept','kept','Keep note','now','now'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'actors:aaaaaaaaaaaaaaaa',0,'Retain note','Dragon Guard','now','now')").unwrap();
            write.commit().unwrap();
            previous = Some(stable_rows(
                workspace
                    .read(TEST_WORKSPACE_SCHEMA_VERSION)
                    .unwrap()
                    .connection(),
            ));
        }
    }
}
#[test]
fn late_snapshot_contract_or_root_error_rolls_back_reference_changes() {
    for (damage, expected_error) in [
        ("contract", "Unsupported source contract"),
        ("root", "selection belongs to another root"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("late-corruption.sqlite");
        let workspace = initialize_workspace(&path).unwrap();
        publish_input(&path, &fixture()).unwrap();
        let owner = workspace.update_owner().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let source = spool(scratch.path(), fixture(), origin(&workspace), Some(damage));
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let before = reference_preservation_rows(write.connection()).unwrap();
        write.commit().unwrap();
        let error = PreparedPublication::new_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            source,
        )
        .unwrap()
        .publish()
        .unwrap_err();
        assert!(
            error.to_string().contains(expected_error),
            "{damage}: {error}"
        );
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(
            reference_preservation_rows(read.connection()).unwrap(),
            before
        );
        assert_eq!(read.reference_revision().unwrap(), 1);
    }
}

#[test]
fn mismatched_preparation_stamp_is_rejected_before_published_state_changes() {
    let input = crate::foundation_tests::fixture(false);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let workspace = initialize_workspace(&path).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let before = stable_rows(read.connection());
    let revision = read.reference_revision().unwrap();
    drop(read);
    let mut bad = input;
    bad.records[1].preparation_context_hash = "0".repeat(64);
    assert!(publish_input(&path, &bad).is_err());
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), revision);
    assert_eq!(stable_rows(read.connection()), before);
    crate::validate_reference(read.connection()).unwrap();
}

#[test]
fn unsafe_prepared_html_rolls_back_reference_changes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("unsafe-prepared-html.sqlite");
    let workspace = initialize_workspace(&path).unwrap();
    let mut input = fixture();
    publish_input(&path, &input).unwrap();
    let before = stable_rows(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .connection(),
    );

    let prepared = input
        .records
        .iter_mut()
        .flat_map(|record| &mut record.content)
        .find_map(|field| match &mut field.status {
            atlas_record::source_record::SourceContentStatus::Prepared(prepared) => Some(prepared),
            _ => None,
        })
        .unwrap();
    prepared.html.push_str("<script>alert(1)</script>");

    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let source = spool(scratch.path(), input, origin(&workspace), None);
    let error = PreparedPublication::new_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        source,
    )
    .unwrap()
    .publish()
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("prepared HTML violates the sanitizer policy"),
        "{error}"
    );

    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 1);
    assert_eq!(stable_rows(read.connection()), before);
}
