//! A physical schema-29 producer, without new metadata, ledger edits or inference.
use crate::{test_support::*, *};
use atlas_domain::RecordKey;
use atlas_storage::Workspace;
use rusqlite::{Connection, params};
use std::collections::BTreeMap;

fn seed_schema29(workspace: &Workspace, input: &IndexBuildInput) {
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let read = workspace.read(29).unwrap();
    let expected = ArtifactInstance::new(
        read.workspace_key().unwrap(),
        read.reference_revision().unwrap(),
    );
    drop(read);
    let spool = publication_spool_in(scratch.path(), input, expected).unwrap();
    let result: Result<(), IndexError> = owner.publish(29, |c| {
        let mut context = input.context.clone();
        context.remaster_inputs_identity.clear();
        c.execute(
            "INSERT INTO artifact_context VALUES(1,?,?,?)",
            params![
                context.format_version,
                context.sha256()?,
                serde_json::to_string(&context)?
            ],
        )?;
        for pack in &input.packs {
            c.execute(
                "INSERT INTO packs VALUES(?,?)",
                params![pack.pack_id, pack.label],
            )?;
        }
        let ids = input
            .records
            .iter()
            .map(|r| {
                crate::identity::retain_source(
                    c,
                    r.record.key(),
                    atlas_record::source_record::SourceNodeView::from(r.record.source())
                        .name()
                        .value()
                        .map(String::as_str),
                )
                .map(|id| (r.record.key().clone(), id))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        spool.records(&mut |frame| {
            let id = ids[frame.source.record.key()];
            let label = &input
                .packs
                .iter()
                .find(|p| p.pack_id == frame.source.record.key().pack().as_str())
                .unwrap()
                .label;
            let derived = crate::component_digests::DerivedComponents::project(
                PreparedRecordRef {
                    source: frame.source,
                    snapshot: frame.snapshot,
                    selection: frame.selection,
                    lexical: frame.lexical,
                },
                label,
                input.context.audience,
            )?;
            let mut allocated =
                crate::publication_delta::allocate(c, id, derived.allocation_footprint)?;
            for row in crate::projections::root_rows(frame.source, label, id, &mut allocated)? {
                row.insert(c)?;
            }
            c.execute(
                "INSERT INTO record_bodies VALUES(?,?,?,?)",
                params![
                    id,
                    atlas_foundry_model::SNAPSHOT_VERSION,
                    "gzip6",
                    crate::codec::gzip(frame.snapshot)?
                ],
            )?;
            crate::writer::write_content(c, frame.source, id, &ids, context.audience, true, true)?;
            assert!(
                frame.source.relationships.is_empty(),
                "this fixture has no structured relationships"
            );
            for section in &derived.sections {
                crate::selected_sections::upsert(c, id, section)?;
            }
            derived.digests.write(c, id)?;
            Ok(())
        })?;
        crate::publication_delta::lexical_stream(c, &ids, &mut |visitor| {
            for unit in &input.lexical_units {
                visitor(unit)?;
            }
            Ok(())
        })?;
        for model in &input.context.search_profiles {
            c.execute(
                "INSERT INTO search_profiles VALUES(?,?,?)",
                params![
                    model.profile_id.as_str(),
                    serde_json::to_string(model)?,
                    model.dimensions
                ],
            )?;
        }
        for (ordinal, unit) in input.semantic_units.iter().enumerate() {
            let model = input.context.search_profile(&unit.profile_id)?;
            let descriptor = atlas_embedding::NativeInputDescriptor {
                input_sha256: unit.input_hash.clone(),
                token_count: unit.input_token_count,
            };
            let bytes = crate::read_units::vector_bytes(&unit.vector);
            let native = crate::native_values::install(
                c,
                model,
                &descriptor,
                &bytes,
                &crate::embedding_values::hash_bytes(&bytes),
            )?;
            let location = crate::semantic_locations::location_id(
                c,
                ids[&unit.record],
                &SemanticOccurrence {
                    profile_id: unit.profile_id.clone(),
                    record: unit.record.clone(),
                    owners: unit.owners.clone(),
                    field: unit.field.clone(),
                    address: unit.address.clone(),
                    chunk_ordinal: unit.chunk_ordinal,
                    input: descriptor,
                },
            )?;
            c.execute(
                "INSERT INTO semantic_units VALUES(?,?,?,?,?,?)",
                params![
                    ordinal as i64 + 1,
                    ids[&unit.record],
                    unit.profile_id.as_str(),
                    location,
                    native,
                    if unit.field.is_none() {
                        "identity"
                    } else {
                        "passage"
                    }
                ],
            )?;
        }
        assert!(input.traits.is_empty() && input.trait_vectors.is_empty());
        for source in &input.records {
            let mut evidence = crate::traits::TraitEvidence::default();
            evidence.add_record(&source.record, input.context.audience)?;
            crate::traits::write_record_trait_relations(c, source.record.key(), evidence)?;
        }
        for record in &input.records {
            crate::core_projection::write_record_core(c, &context, record)?;
        }
        assert!(!c.prepare("PRAGMA foreign_key_check")?.exists([])?);
        Ok(())
    });
    result.unwrap();
}
fn native_snapshot(c: &Connection) -> Vec<(String, Vec<u8>, usize, Vec<u8>, Vec<u8>)> {
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
fn authentic_v7_schema29_upgrade_reuses_sources_layouts_and_native_values() {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v29.sqlite");
    let workspace = Workspace::at(&path).unwrap();
    workspace
        .migrate(&TEST_MIGRATIONS[..29], |_| Ok(()))
        .unwrap();
    let mut input = crate::foundation_tests::fixture(true);
    assert_eq!(
        input.context.relationship_policy,
        "typed-authored-occurrences/v7"
    );
    seed_schema29(&workspace, &input);
    let (saved, native, key, old_revision);
    {
        let write = workspace.write(29).unwrap();
        write.connection().execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,description,created_at,updated_at) VALUES(1,'kept','kept','Kept','Exact editions','before','before'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'items:bbbbbbbbbbbbbbbb',0,'Keep exact old key','Dragon Staff','before','before');").unwrap();
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
    initialize_workspace(&path).unwrap();
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
    let expected = input
        .semantic_units
        .iter()
        .filter(|u| u.field.is_some())
        .count();
    assert!(expected > 0);
    assert_eq!(layouts.iter().map(Vec::len).sum::<usize>(), expected);
    // Current captured declarations are newly projected; the old physical fixture
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
        true,
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
            .remaster_record_information(&"items:bbbbbbbbbbbbbbbb".parse().unwrap(), &[], 20)
            .unwrap()
            .declarations
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
            .query_row("SELECT count(*) FROM remaster_bindings", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        2
    );
}
