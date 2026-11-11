//! Reuse from retained reference data before remaster source preparation.
use crate::{test_support::*, *};
use atlas_domain::RecordKey;
use rusqlite::Connection;

type NativeInputSnapshot = (String, Vec<u8>, usize, Vec<u8>, Vec<u8>);

fn native_snapshot(c: &Connection) -> Vec<NativeInputSnapshot> {
    let models = c
        .prepare("SELECT model_id,identity_json FROM embedding_models ORDER BY model_id")
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut values = Vec::new();
    for (id, identity) in models {
        let model: SourceSearchProfileIdentity = serde_json::from_str(&identity).unwrap();
        let table = crate::schema::native_vector_table(&model).unwrap();
        let mut statement=c.prepare(&format!("SELECT i.input_hash,i.token_count,i.published_hash,v.embedding FROM embedding_inputs i JOIN {table} v USING(input_id) WHERE i.model_id=? ORDER BY i.input_id")).unwrap();
        values.extend(
            statement
                .query_map([id], |r| {
                    Ok((identity.clone(), r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        );
    }
    values
}

#[test]
fn retained_v7_reference_data_reuses_sources_layouts_and_native_values() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("retained.sqlite");
    let workspace = initialize_workspace(&path).unwrap();
    let mut input = crate::core_tests::fixture(
        vec![
            (
                "items",
                "Item",
                serde_json::json!({"_id":"bbbbbbbbbbbbbbbb","name":"Dragon Staff","type":"weapon","system":{"description":{"value":"<h2>Flame</h2><p>Dragon breath.</p>"}}}),
            ),
            (
                "items",
                "Item",
                serde_json::json!({"_id":"cccccccccccccccc","name":"Remastered Dragon Staff","type":"weapon","system":{"description":{"value":"<h2>Flame</h2><p>Dragon breath.</p>"}}}),
            ),
        ],
        true,
    );
    assert_eq!(
        input.context.relationship_policy,
        "typed-authored-occurrences/v7"
    );
    publish_input_in(&workspace, TEST_WORKSPACE_SCHEMA_VERSION, &input).unwrap();
    let (saved, native, key, old_revision);
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let mut context = input.context.clone();
        context.remaster_inputs_identity.clear();
        write
            .connection()
            .execute(
                "UPDATE artifact_context SET context_json=?1,context_hash=?2",
                [
                    serde_json::to_string(&context).unwrap(),
                    context.sha256().unwrap(),
                ],
            )
            .unwrap();
        write.connection().execute_batch("INSERT INTO saved_lists(id,owner_id,list_key,slug,name,description,created_at,updated_at) VALUES(1,1,'kept','kept','Kept','Exact editions','before','before'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'items:bbbbbbbbbbbbbbbb',0,'Keep exact old key','Dragon Staff','before','before');").unwrap();
        saved = write
            .connection()
            .query_row(
                "SELECT record_key,note,record_title_snapshot FROM saved_list_items",
                [],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                },
            )
            .unwrap();
        native = native_snapshot(write.connection());
        key = write
            .connection()
            .query_row("SELECT workspace_key FROM workspace", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap();
        old_revision = write.reference_revision().unwrap();
        write.commit().unwrap();
    }
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let stored = crate::read_stored_reference_context(read.connection())
        .unwrap()
        .unwrap();
    assert!(stored.remaster_inputs_identity.is_empty());
    assert_eq!(
        stored.preparation_sha256().unwrap(),
        input.context.preparation_sha256().unwrap()
    );
    assert_eq!(
        crate::classify_reference_upgrade(read.connection()).unwrap(),
        ReferenceUpgrade::SourcePreparation
    );
    assert_eq!(native_snapshot(read.connection()), native);
    assert_eq!(read.reference_revision().unwrap(), old_revision);
    drop(read);
    assert!(open_reader(&path).is_err());
    let reuse = ReferenceReuseReader::new(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let keys = input
        .records
        .iter()
        .map(|r| r.record.key())
        .collect::<Vec<_>>();
    assert_eq!(reuse.identities().unwrap().len(), input.records.len());
    assert!(
        reuse
            .read_admissions(&keys)
            .unwrap()
            .iter()
            .all(Option::is_some)
    );
    assert!(
        reuse
            .read_records(&keys)
            .unwrap()
            .iter()
            .all(Option::is_some)
    );
    assert!(
        reuse
            .preparation_matches(&input.context.preparation_sha256().unwrap())
            .unwrap()
    );
    let layouts = reuse.read_layouts(&keys).unwrap();
    // Both profiles publish occurrences, but segmentation is shared. Compare
    // the exact distinct source locations, not the number of model occurrences.
    assert_eq!(input.context.search_profiles.len(), 2);
    let expected = input
        .semantic_units
        .iter()
        .filter_map(|unit| {
            unit.field.as_ref().map(|field| {
                serde_json::to_string(&(
                    &unit.record,
                    &unit.owners,
                    field,
                    unit.chunk_ordinal,
                    &unit.address,
                    &unit.input_hash,
                ))
                .unwrap()
            })
        })
        .collect::<std::collections::BTreeSet<_>>();
    let actual = keys
        .iter()
        .zip(&layouts)
        .flat_map(|(key, layouts)| {
            layouts.iter().map(move |layout| {
                serde_json::to_string(&(
                    key,
                    &layout.owners,
                    &layout.field,
                    layout.chunk_ordinal,
                    &layout.passage.address,
                    &layout.passage.input_sha256,
                ))
                .unwrap()
            })
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert!(!expected.is_empty());
    assert_eq!(actual, expected);
    assert_eq!(layouts.iter().map(Vec::len).sum::<usize>(), actual.len());
    // Current captured declarations are newly projected; the retained reference data
    // contains neither a remaster identity nor any new metadata table/authority.
    add_remaster_relationship(
        &mut input,
        &"items:bbbbbbbbbbbbbbbb".parse::<RecordKey>().unwrap(),
        &"items:cccccccccccccccc".parse().unwrap(),
    )
    .unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let publication = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    assert_eq!(
        publication.logical_native_bytes().unwrap(),
        0,
        "all unchanged values selected from published storage, no staging/inference"
    );
    publication.publish().unwrap();
    drop(owner);
    let reader = open_reader(&path).unwrap();
    assert_eq!(reader.artifact_instance().workspace_key(), key);
    assert!(reader.artifact_instance().reference_revision() > old_revision);
    assert_eq!(
        reader
            .read_remaster_links(&"items:bbbbbbbbbbbbbbbb".parse().unwrap())
            .unwrap()
            .len(),
        1
    );
    drop(reader);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(native_snapshot(read.connection()), native);
    let after = read
        .connection()
        .query_row(
            "SELECT record_key,note,record_title_snapshot FROM saved_list_items",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(after, saved);
    crate::semantic_locations::validate(read.connection()).unwrap();
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM remaster_pairs", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        1
    );
}
