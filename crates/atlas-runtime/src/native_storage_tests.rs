//! Direct upgrades from published layouts preserve numerical and saved identities.
use super::*;
use crate::test_support::sql_rows as rows;
use atlas_index::{IndexBuildInput, SourceSearchProfileIdentity};
use rusqlite::{Connection, params, types::Value};
use std::collections::BTreeMap;

type Rows = Vec<Vec<Value>>;
fn bytes(vector: &[f32]) -> Vec<u8> {
    vector
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}
fn checksum(bytes: &[u8]) -> Vec<u8> {
    let hash = atlas_search::test_support::hash(bytes);
    (0..hash.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hash[offset..offset + 2], 16).unwrap())
        .collect()
}
fn table(model: &SourceSearchProfileIdentity, prefix: &str) -> String {
    format!(
        "{prefix}_{}",
        atlas_search::test_support::hash(&serde_json::to_vec(model).unwrap())
    )
}
fn input(profile_count: usize) -> IndexBuildInput {
    let records = ["aaaaaaaaaaaaaaaa","bbbbbbbbbbbbbbbb"].into_iter().map(|id| {
        atlas_search::test_support::record("native-probe","Item",serde_json::json!({
            "_id":id,"type":"action","name":"Native probe","system":{"description":{"value":"<h2>Kept</h2><p>Same selected passage.</p>"}}
        }))
    }).collect();
    let mut input = atlas_search::test_support::input_with_both_profiles(records);
    input.context.search_profiles.truncate(profile_count);
    input.semantic_units.retain(|unit| {
        input
            .context
            .search_profiles
            .iter()
            .any(|model| model.profile_id == unit.profile_id)
    });
    for model in &input.context.search_profiles {
        let selected = input
            .semantic_units
            .iter()
            .find(|unit| unit.profile_id == model.profile_id && unit.field.is_some())
            .unwrap()
            .clone();
        // The embedding engine is synthetic here: identical prepared passage inputs
        // deliberately share one exact value across two real occurrence addresses.
        for unit in input
            .semantic_units
            .iter_mut()
            .filter(|unit| unit.profile_id == model.profile_id && unit.field.is_some())
        {
            unit.input_hash = selected.input_hash.clone();
            unit.input_token_count = selected.input_token_count;
            unit.vector.clone_from(&selected.vector);
        }
    }
    if profile_count != 0 {
        let key = atlas_domain::TraitKey::new("fire").unwrap();
        let label = "Fire\nCold\nFire";
        let text = format!("{label}\n");
        let text_hash = atlas_search::test_support::hash(text.as_bytes());
        input.traits.push(atlas_index::TraitDefinition {
            key: key.clone(),
            label: label.into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec![],
            source_locations: vec![],
            source_revision: None,
            indexing_locale: "en".into(),
            content_locale: None,
        });
        for model in &input.context.search_profiles {
            let mut x = vec![0.; model.dimensions];
            x[42] = 1.;
            let mut t = vec![0.; model.dimensions];
            t[43] = 1.;
            let x_hash = atlas_search::test_support::hash(b"Fire\n");
            // The synthetic embedding contract includes a document/trait-shared
            // input X, and a trait-only input T. Ordered [X,T,X] keeps both X weights.
            for unit in input
                .semantic_units
                .iter_mut()
                .filter(|unit| unit.profile_id == model.profile_id && unit.field.is_some())
            {
                unit.input_hash = x_hash.clone();
                unit.input_token_count = 2;
                unit.vector.clone_from(&x);
            }
            let mut chunks = Vec::new();
            for (ordinal, (body, vector)) in [("Fire\n", x.clone()), ("Cold\n", t), ("Fire\n", x)]
                .into_iter()
                .enumerate()
            {
                chunks.push(atlas_index::TraitSeedChunk {
                    input: body.into(),
                    input_hash: atlas_search::test_support::hash(body.as_bytes()),
                    token_count: 2,
                    vector,
                    address: atlas_domain::SourcePassageAddress::PlainSection {
                        source_text_sha256: text_hash.clone(),
                        selection_version: "trait-definition-centroid/v1".into(),
                        section_ordinal: 0,
                        label: Some(label.into()),
                        chunk_bytes: atlas_domain::SourceByteRange {
                            start: ordinal * 5,
                            end: (ordinal + 1) * 5,
                        },
                    },
                });
            }
            let mut centroid = vec![0.; model.dimensions];
            centroid[42] = (2.0f64 / 5.0f64.sqrt()) as f32;
            centroid[43] = (1.0f64 / 5.0f64.sqrt()) as f32;
            let hashes = chunks
                .iter()
                .map(|chunk| chunk.input_hash.as_str())
                .collect::<Vec<_>>()
                .join(":");
            input.trait_vectors.push(atlas_index::TraitVectorInput {
                key: key.clone(),
                profile_id: model.profile_id.clone(),
                input_hash: atlas_search::test_support::hash(
                    format!("trait-definition-centroid/v1:{hashes}").as_bytes(),
                ),
                vector: centroid,
                chunks,
            });
        }
    }
    input
}
pub(super) fn main24(profile_count: usize) -> (tempfile::TempDir, Workspace, IndexBuildInput) {
    main24_input(input(profile_count))
}
pub(super) fn main24_input(
    input: IndexBuildInput,
) -> (tempfile::TempDir, Workspace, IndexBuildInput) {
    register_extensions().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(dir.path().join("main24.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(24, "authentic main24 storage fixture")
            .unwrap();
        atlas_index::test_support::publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    let write = workspace.write(24).unwrap();
    write.connection().execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(77,'kept','kept','Preserved list','before','after');") .unwrap();
    write.commit().unwrap();
    (dir, workspace, input)
}
fn legacy_cache(workspace: &Workspace, input: &IndexBuildInput) {
    let write = workspace.write(24).unwrap();
    write.connection().execute("INSERT INTO reference_update_jobs(job_key,expected_revision,target_hash,state,updated_at) VALUES('failed-build',?,'kept-target','preparing','2001-01-01 00:00:00')",[write.reference_revision().unwrap()]).unwrap();
    for model in &input.context.search_profiles {
        let mut vector = vec![0.; model.dimensions];
        vector[71] = 1.;
        let bytes = bytes(&vector);
        let id:i64=write.connection().query_row("SELECT i.input_id FROM embedding_inputs i JOIN embedding_models m USING(model_id) WHERE m.identity_json=? AND i.published_hash IS NOT NULL ORDER BY i.input_id LIMIT 1",[serde_json::to_string(model).unwrap()],|row|row.get(0)).unwrap();
        write
            .connection()
            .execute(
                "INSERT INTO pending_embedding_values VALUES(?,?,?,'failed-build')",
                params![id, bytes, checksum(&bytes)],
            )
            .unwrap();
    }
    if let Some(model) = input.context.search_profiles.first() {
        let mut model = model.clone();
        model.execution_policy = "unpublished-only/v1".into();
        write
            .connection()
            .execute(
                "INSERT INTO embedding_models(identity_json) VALUES(?)",
                [serde_json::to_string(&model).unwrap()],
            )
            .unwrap();
        let model_id = write.connection().last_insert_rowid();
        write.connection().execute("INSERT INTO embedding_inputs(model_id,input_hash,token_count) VALUES(?,zeroblob(32),2)",[model_id]).unwrap();
        let id = write.connection().last_insert_rowid();
        let mut vector = vec![0.; model.dimensions];
        vector[33] = 1.;
        let bytes = bytes(&vector);
        write
            .connection()
            .execute(
                "INSERT INTO pending_embedding_values VALUES(?,?,?,'failed-build')",
                params![id, bytes, checksum(&bytes)],
            )
            .unwrap();
    }
    write.commit().unwrap();
}
fn snapshot(c: &Connection) -> BTreeMap<String, Rows> {
    let mut snapshot = BTreeMap::new();
    for table in [
        "workspace",
        "schema_migrations",
        "projection_allocations",
        "artifact_context",
        "record_identity",
        "records",
        "saved_lists",
        "embedding_models",
        "embedding_inputs",
        "semantic_units",
        "trait_vectors",
        "trait_seed_chunks",
        "trait_native_values",
        "pending_embedding_values",
        "reference_update_jobs",
    ] {
        snapshot.insert(
            table.into(),
            rows(c, &format!("SELECT * FROM {table} ORDER BY 1,2")),
        );
    }
    let models = c
        .prepare("SELECT identity_json FROM search_profiles ORDER BY profile_id")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for json in models {
        let model: SourceSearchProfileIdentity = serde_json::from_str(&json).unwrap();
        let table = table(&model, "semantic_vectors");
        snapshot.insert(
            table.clone(),
            rows(
                c,
                &format!("SELECT unit_id,embedding FROM {table} ORDER BY unit_id"),
            ),
        );
    }
    snapshot
}
fn published_values(
    c: &Connection,
    input: &IndexBuildInput,
    current: bool,
) -> BTreeMap<(String, i64), Vec<u8>> {
    let mut out = BTreeMap::new();
    for model in &input.context.search_profiles {
        let table = table(
            model,
            if current {
                "native_vectors"
            } else {
                "semantic_vectors"
            },
        );
        let sql = if current {
            format!("SELECT input_id,embedding FROM {table} ORDER BY input_id")
        } else {
            format!(
                "SELECT u.input_id,v.embedding FROM semantic_units u JOIN {table} v USING(unit_id) WHERE u.profile_id=?1 UNION ALL SELECT t.input_id,t.vector FROM trait_native_values t JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) WHERE m.identity_json=?2 ORDER BY 1"
            )
        };
        let mut statement = c.prepare(&sql).unwrap();
        let mut raw = if current {
            statement.query([]).unwrap()
        } else {
            statement
                .query((
                    model.profile_id.as_str(),
                    serde_json::to_string(model).unwrap(),
                ))
                .unwrap()
        };
        while let Some(row) = raw.next().unwrap() {
            let id: i64 = row.get(0).unwrap();
            let bytes: Vec<u8> = row.get(1).unwrap();
            if let Some(old) =
                out.insert((serde_json::to_string(model).unwrap(), id), bytes.clone())
            {
                assert_eq!(old, bytes);
            }
        }
    }
    out
}
#[test]
fn historical_seeding_keeps_reserved_identities_and_saved_state_and_rejects_republication() {
    register_extensions().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(directory.path().join("reserved-main24.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
    let input = input(2);
    let write = workspace.write(24).unwrap();
    write.connection().execute("INSERT INTO record_identity(id,key,last_known_name,availability) VALUES(77,?,'Reserved source','unavailable'),(88,'native-probe:cccccccccccccccc','Retained unavailable key','unavailable')",[input.records[0].record.key().to_string()]).unwrap();
    write.connection().execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(5,'saved','saved','Saved before source','before','after');").unwrap();
    let saved = rows(write.connection(), "SELECT * FROM saved_lists ORDER BY id");
    write.commit().unwrap();
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(24, "initial historical reference")
            .unwrap();
        atlas_index::test_support::publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    let read = workspace.read(24).unwrap();
    assert_eq!(
        rows(read.connection(), "SELECT * FROM saved_lists ORDER BY id"),
        saved
    );
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT record_id FROM records WHERE key=?",
                [input.records[0].record.key().to_string()],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        77
    );
    assert_eq!(
        rows(
            read.connection(),
            "SELECT id,key,last_known_name,availability FROM record_identity WHERE id=88"
        ),
        vec![vec![
            Value::Integer(88),
            Value::Text("native-probe:cccccccccccccccc".into()),
            Value::Text("Retained unavailable key".into()),
            Value::Text("unavailable".into())
        ]]
    );
    drop(read);
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner
        .maintenance(24, "reject historical replacement")
        .unwrap();
    let before = snapshot(maintenance.connection());
    let error =
        atlas_index::test_support::publish_pre_section_fixture(&maintenance, &input).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("requires unpublished reference tables"),
        "{error}"
    );
    assert_eq!(snapshot(maintenance.connection()), before);
}

#[test]
fn main24_native_upgrade_preserves_shared_inputs_traits_pending_models_and_saved_rows() {
    for selected in [
        vec![],
        vec!["bge-base"],
        vec!["embeddinggemma"],
        vec!["bge-base", "embeddinggemma"],
    ] {
        let profiles = selected.len();
        let mut input = input(if profiles == 0 { 0 } else { 2 });
        input
            .context
            .search_profiles
            .retain(|model| selected.contains(&model.profile_id.as_str()));
        input
            .semantic_units
            .retain(|unit| selected.contains(&unit.profile_id.as_str()));
        input
            .trait_vectors
            .retain(|unit| selected.contains(&unit.profile_id.as_str()));
        let (_dir, workspace, input) = main24_input(input);
        legacy_cache(&workspace, &input);
        let read = workspace.read(24).unwrap();
        let key = read.workspace_key().unwrap();
        let revision = read.reference_revision().unwrap();
        let values = published_values(read.connection(), &input, false);
        let allocations = rows(
            read.connection(),
            "SELECT record_id,seed,end_id FROM projection_allocations ORDER BY record_id",
        );
        let old_hashes = read.connection().prepare("SELECT r.key,a.input_hash FROM projection_allocations a JOIN records r USING(record_id) ORDER BY r.key").unwrap()
            .query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?))).unwrap().collect::<Result<BTreeMap<_,_>,_>>().unwrap();
        let expected_hashes = input
            .records
            .iter()
            .map(|source| {
                let label = &input
                    .packs
                    .iter()
                    .find(|pack| pack.pack_id == source.record.key().pack().as_str())
                    .unwrap()
                    .label;
                let hash = atlas_search::test_support::hash(
                    &serde_json::to_vec(&(
                        atlas_foundry_model::encode_snapshot(source.record.source()).unwrap(),
                        &source.source_path,
                        &source.content_hash,
                        &source.preparation_context_hash,
                        &source.content,
                        &source.relationships,
                        &source.diagnostics,
                        label,
                        "authored-query/v5",
                    ))
                    .unwrap(),
                );
                (source.record.key().to_string(), hash)
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(old_hashes, expected_hashes);
        let associations = rows(
            read.connection(),
            "SELECT * FROM semantic_units ORDER BY unit_id",
        );
        let traits = rows(
            read.connection(),
            "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id",
        );
        let chunks = rows(
            read.connection(),
            "SELECT * FROM trait_seed_chunks ORDER BY trait_key,profile_id,ordinal",
        );
        let pending = rows(
            read.connection(),
            "SELECT input_id,vector,vector_hash FROM pending_embedding_values ORDER BY input_id",
        );
        let ledger = rows(
            read.connection(),
            "SELECT version,name,checksum FROM schema_migrations ORDER BY version",
        );
        let saved = rows(read.connection(), "SELECT * FROM saved_lists ORDER BY id");
        drop(read);
        initialize_workspace(workspace.path()).unwrap();
        let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(read.workspace_key().unwrap(), key);
        assert_eq!(read.reference_revision().unwrap(), revision);
        assert_eq!(published_values(read.connection(), &input, true), values);
        assert_eq!(
            rows(
                read.connection(),
                "SELECT record_id,seed,end_id FROM projection_allocations ORDER BY record_id"
            ),
            allocations
        );
        let allocation_columns = read
            .connection()
            .prepare("PRAGMA table_info(projection_allocations)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(allocation_columns, ["record_id", "seed", "end_id"]);
        assert_eq!(
            rows(
                read.connection(),
                "SELECT * FROM semantic_units ORDER BY unit_id"
            ),
            associations
        );
        assert_eq!(
            rows(
                read.connection(),
                "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id"
            ),
            traits
        );
        assert_eq!(
            rows(
                read.connection(),
                "SELECT * FROM trait_seed_chunks ORDER BY trait_key,profile_id,ordinal"
            ),
            chunks
        );
        assert_eq!(
            rows(
                read.connection(),
                "SELECT input_id,vector,vector_hash FROM pending_embedding_values ORDER BY input_id"
            ),
            pending
        );
        assert_eq!(
            rows(
                read.connection(),
                "SELECT version,name,checksum FROM schema_migrations WHERE version<=24 ORDER BY version"
            ),
            ledger
        );
        assert_eq!(
            rows(read.connection(), "SELECT * FROM saved_lists ORDER BY id"),
            saved
        );
        assert!(!read.connection().query_row("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name IN ('native_update_targets','native_target_inputs','reference_update_jobs','trait_native_values','legacy_pending_embedding_values'))",[],|row|row.get::<_,bool>(0)).unwrap());
        if profiles != 0 {
            assert_eq!(
                read.connection()
                    .query_row(
                        "SELECT count(*) FROM pending_embedding_values WHERE cached_at=978307200",
                        [],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                profiles + 1
            );
            assert!(values.len() < input.semantic_units.len() + input.trait_vectors.len());
        }
        assert!(
            !read
                .connection()
                .prepare("PRAGMA foreign_key_check")
                .unwrap()
                .exists([])
                .unwrap()
        );
        drop(read);
        initialize_workspace(workspace.path()).unwrap();
    }
}
#[test]
fn invalid_old_native_values_reject_the_entire_main24_upgrade() {
    for corrupt in [
        "late_duplicate",
        "missing",
        "orphan",
        "wrong_model",
        "oversized_pending",
        "wrong_pending_checksum",
        "invalid_tokens",
        "trait_disagrees",
        "oversized_trait",
        "invalid_pending_norm",
        "wrong_centroid",
        "trait_chunk_gap",
        "wrong_model_geometry",
        "wrong_profile_geometry",
        "wrong_old_ddl",
    ] {
        let (_dir, workspace, input) = main24(2);
        legacy_cache(&workspace, &input);
        let model = &input.context.search_profiles[0];
        let table = table(model, "semantic_vectors");
        let write = workspace.write(24).unwrap();
        let unit:i64=write.connection().query_row("SELECT max(unit_id) FROM semantic_units WHERE profile_id=? AND unit_kind='passage'",[model.profile_id.as_str()],|row|row.get(0)).unwrap();
        match corrupt {
            "late_duplicate" => {
                let mut vector = vec![0.; model.dimensions];
                vector[99] = 1.;
                write
                    .connection()
                    .execute(
                        &format!("UPDATE {table} SET embedding=? WHERE unit_id=?"),
                        params![bytes(&vector), unit],
                    )
                    .unwrap();
            }
            "missing" => {
                write
                    .connection()
                    .execute(&format!("DELETE FROM {table} WHERE unit_id=?"), [unit])
                    .unwrap();
            }
            "orphan" => {
                let mut vector = vec![0.; model.dimensions];
                vector[0] = 1.;
                write
                    .connection()
                    .execute(
                        &format!("INSERT INTO {table}(unit_id,embedding) VALUES(999999,?)"),
                        [bytes(&vector)],
                    )
                    .unwrap();
            }
            "wrong_model" => {
                write.connection().execute("UPDATE semantic_units SET input_id=(SELECT input_id FROM semantic_units WHERE profile_id<>?1 LIMIT 1) WHERE unit_id=?2",params![model.profile_id.as_str(),unit]).unwrap();
            }
            "oversized_pending" => {
                write.connection().execute("UPDATE pending_embedding_values SET vector=zeroblob(?) WHERE input_id=(SELECT min(input_id) FROM pending_embedding_values)",[model.dimensions*4+4]).unwrap();
            }
            "wrong_pending_checksum" => {
                write
                    .connection()
                    .execute(
                        "UPDATE pending_embedding_values SET vector_hash=zeroblob(32)",
                        [],
                    )
                    .unwrap();
            }
            "trait_disagrees" => {
                let mut vector = vec![0.; model.dimensions];
                vector[88] = 1.;
                write
                    .connection()
                    .execute("UPDATE trait_native_values SET vector=?", [bytes(&vector)])
                    .unwrap();
            }
            "oversized_trait" => {
                write
                    .connection()
                    .execute(
                        "UPDATE trait_native_values SET vector=zeroblob(?)",
                        [model.dimensions * 4 + 4],
                    )
                    .unwrap();
            }
            "invalid_pending_norm" => {
                let vector = vec![0.; model.dimensions];
                let bytes = bytes(&vector);
                write
                    .connection()
                    .execute(
                        "UPDATE pending_embedding_values SET vector=?,vector_hash=?",
                        params![bytes, checksum(&bytes)],
                    )
                    .unwrap();
            }
            "wrong_centroid" => {
                let mut vector = vec![0.; model.dimensions];
                vector[44] = 1.;
                let value = bytes(&vector);
                write
                    .connection()
                    .execute(
                        "UPDATE trait_vectors SET vector=?,vector_hash=?",
                        params![value, checksum(&value)],
                    )
                    .unwrap();
            }
            "trait_chunk_gap" => {
                write
                    .connection()
                    .execute("UPDATE trait_seed_chunks SET ordinal=3 WHERE ordinal=2", [])
                    .unwrap();
            }
            "wrong_model_geometry" => {
                let mut wrong = model.clone();
                wrong.dimensions += 1;
                write
                    .connection()
                    .execute(
                        "UPDATE embedding_models SET identity_json=? WHERE identity_json=?",
                        (
                            serde_json::to_string(&wrong).unwrap(),
                            serde_json::to_string(model).unwrap(),
                        ),
                    )
                    .unwrap();
            }
            "wrong_profile_geometry" => {
                write
                    .connection()
                    .execute(
                        "UPDATE search_profiles SET dimensions=dimensions+1 WHERE profile_id=?",
                        [model.profile_id.as_str()],
                    )
                    .unwrap();
            }
            "wrong_old_ddl" => {
                write.connection().execute_batch(&format!("DROP TABLE {table}; CREATE VIRTUAL TABLE {table} USING vec0(unit_id INTEGER PRIMARY KEY,embedding FLOAT[769] distance_metric=cosine);")).unwrap();
            }
            "invalid_tokens" => {
                write.connection().execute("UPDATE embedding_inputs SET token_count=32769 WHERE input_id=(SELECT min(input_id) FROM embedding_inputs)",[]).unwrap();
            }
            _ => unreachable!(),
        }
        let before = snapshot(write.connection());
        write.commit().unwrap();
        let error = initialize_workspace(workspace.path())
            .unwrap_err()
            .to_string();
        let expected = match corrupt {
            "late_duplicate" | "wrong_pending_checksum" | "trait_disagrees" => {
                "native bytes disagree"
            }
            "missing" => "occurrence vector missing",
            "orphan" => "unmapped occurrence vector",
            "wrong_model" | "invalid_tokens" => "model/token/hash identity mismatch",
            "oversized_pending" => "recovery vector/checksum has invalid length",
            "oversized_trait" => "trait native value has invalid length",
            "invalid_pending_norm" => "normal",
            "wrong_centroid" => "centroid differs from its ordered",
            "trait_chunk_gap" => "chunk order has gaps",
            "wrong_model_geometry" => "invalid full model execution identity",
            "wrong_profile_geometry" => "profile identity/geometry mismatch",
            "wrong_old_ddl" => "occurrence vector DDL differs",
            _ => unreachable!(),
        };
        assert!(
            error.contains("native migration") && error.contains(expected),
            "{corrupt}: {error}"
        );
        let read = workspace.read(24).unwrap();
        assert_eq!(snapshot(read.connection()), before, "{corrupt}");
        assert_eq!(
            read.connection()
                .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert!(
            !read
                .connection()
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name GLOB 'native_vectors_*')",
                    [],
                    |row| row.get::<_, bool>(0)
                )
                .unwrap()
        );
    }
}
#[test]
fn failure_after_native_drop_section_reconstruction_or_final_schema_restores_main24() {
    for boundary in [27, 28, 29] {
        let (_dir, workspace, input) = main24(2);
        legacy_cache(&workspace, &input);
        let read = workspace.read(24).unwrap();
        let before = snapshot(read.connection());
        drop(read);
        let error = workspace
            .migrate(&MIGRATIONS[..boundary], |_| {
                Err(atlas_storage::StorageError::Migration(
                    "injected final validator failure".into(),
                ))
            })
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("injected final validator failure"),
            "boundary {boundary}: {error}"
        );
        let read = workspace.read(24).unwrap();
        assert_eq!(snapshot(read.connection()), before);
        assert_eq!(
            read.connection()
                .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        drop(read);
        initialize_workspace(workspace.path()).unwrap();
    }
}

#[test]
fn published_schema_nine_replay_preserves_vectors_and_forced_pending_without_assets() {
    register_extensions().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(dir.path().join("published-nine.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..2], |_| Ok(())).unwrap();
    let mut input = input(2);
    input.traits.clear();
    input.trait_vectors.clear();
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(2, "authentic pre-trait reference fixture")
            .unwrap();
        atlas_index::test_support::publish_historical_record_fixture(&maintenance, &input).unwrap();
    }
    workspace.migrate(&MIGRATIONS[..9], |_| Ok(())).unwrap();
    let write = workspace.write(9).unwrap();
    write.connection().execute(
        "INSERT INTO reference_update_jobs(job_key,expected_revision,target_hash,state) VALUES('forced-retry',?,'fixture','preparing')",
        [write.reference_revision().unwrap()],
    ).unwrap();
    let model = &input.context.search_profiles[0];
    let unit = input
        .semantic_units
        .iter()
        .find(|unit| unit.profile_id == model.profile_id)
        .unwrap();
    let descriptor = atlas_embedding::PreparedNativeInput {
        input: String::new(),
        input_sha256: unit.input_hash.clone(),
        token_count: unit.input_token_count,
    };
    let mut forced = vec![0.; model.dimensions];
    forced[71] = 1.;
    atlas_index::test_support::stage_historical_embedding_fixture(
        &write,
        "forced-retry",
        "forced-retry",
        model,
        &descriptor,
        &forced,
    )
    .unwrap();
    write.connection().execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(91,'legacy','legacy','Published nine','before','after');").unwrap();
    let vectors = crate::test_support::occurrence_vector_bits(write.connection());
    let units = rows(
        write.connection(),
        "SELECT unit_id,record_id,profile_id FROM semantic_units ORDER BY unit_id",
    );
    let ledger = rows(
        write.connection(),
        "SELECT version,name,checksum FROM schema_migrations ORDER BY version",
    );
    write.commit().unwrap();
    initialize_workspace(workspace.path()).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        crate::test_support::occurrence_vector_bits(read.connection()),
        vectors
    );
    assert_eq!(
        rows(
            read.connection(),
            "SELECT unit_id,record_id,profile_id FROM semantic_units ORDER BY unit_id"
        ),
        units
    );
    assert_eq!(
        rows(
            read.connection(),
            "SELECT version,name,checksum FROM schema_migrations WHERE version<=9 ORDER BY version"
        ),
        ledger
    );
    let cached:Vec<u8>=read.connection().query_row(
        "SELECT p.vector FROM pending_embedding_values p JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) WHERE m.identity_json=? AND lower(hex(i.input_hash))=?",
        (serde_json::to_string(model).unwrap(),&unit.input_hash),|row|row.get(0),
    ).unwrap();
    assert_eq!(cached, bytes(&forced));
    assert_eq!(
        read.connection()
            .query_row("SELECT name FROM saved_lists WHERE id=91", [], |row| row
                .get::<_, String>(
                0
            ))
            .unwrap(),
        "Published nine"
    );
    assert!(
        !read
            .connection()
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
}

#[test]
fn published_empty_profiles_upgrade_without_inventing_numerical_values() {
    register_extensions().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(dir.path().join("empty-main24.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
    let input = atlas_search::test_support::input_with_both_profiles(vec![]);
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(24, "empty published profile fixture")
            .unwrap();
        atlas_index::test_support::publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    let read = workspace.read(24).unwrap();
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM embedding_models", [], |row| row
                .get::<_, usize>(0))
            .unwrap(),
        0
    );
    let revision = read.reference_revision().unwrap();
    drop(read);
    initialize_workspace(workspace.path()).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), revision);
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM embedding_inputs", [], |row| row
                .get::<_, usize>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM pending_embedding_values", [], |row| {
                row.get::<_, usize>(0)
            })
            .unwrap(),
        0
    );
    assert_eq!(
        crate::test_support::occurrence_vector_bits(read.connection()).len(),
        2
    );
    for model in &input.context.search_profiles {
        let table = table(model, "native_vectors");
        assert_eq!(
            read.connection()
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row
                    .get::<_, usize>(0))
                .unwrap(),
            0
        );
    }
}

#[test]
fn valid_signed_zero_centroids_keep_their_original_bytes_during_upgrade() {
    let (_dir, workspace, input) = main24(2);
    let write = workspace.write(24).unwrap();
    for model in &input.context.search_profiles {
        let mut value: Vec<u8> = write
            .connection()
            .query_row(
                "SELECT vector FROM trait_vectors WHERE profile_id=?",
                [model.profile_id.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        value[..4].copy_from_slice(&(-0.0f32).to_le_bytes());
        write
            .connection()
            .execute(
                "UPDATE trait_vectors SET vector=?,vector_hash=? WHERE profile_id=?",
                params![value, checksum(&value), model.profile_id.as_str()],
            )
            .unwrap();
    }
    let before = rows(
        write.connection(),
        "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id",
    );
    write.commit().unwrap();
    initialize_workspace(workspace.path()).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        rows(
            read.connection(),
            "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id"
        ),
        before
    );
}

#[test]
fn trait_similarity_retirement_preserves_document_values_on_historical_upgrade() {
    for intermediate_upgrade in [false, true] {
        let (_dir, workspace, input) = main24(2);
        legacy_cache(&workspace, &input);
        let read = workspace.read(24).unwrap();
        let key = read.workspace_key().unwrap();
        let revision = read.reference_revision().unwrap();
        let saved = rows(read.connection(), "SELECT * FROM saved_lists ORDER BY id");
        let document_ids = read
            .connection()
            .prepare("SELECT DISTINCT input_id FROM semantic_units")
            .unwrap()
            .query_map([], |row| row.get::<_, i64>(0))
            .unwrap()
            .collect::<Result<std::collections::BTreeSet<_>, _>>()
            .unwrap();
        let mut document_values = published_values(read.connection(), &input, false);
        document_values.retain(|(_, id), _| document_ids.contains(id));
        let pending = rows(
            read.connection(),
            "SELECT input_id,vector,vector_hash FROM pending_embedding_values ORDER BY input_id",
        );
        drop(read);

        let historical = &MIGRATIONS[..33];
        if intermediate_upgrade {
            workspace.migrate(historical, |_| Ok(())).unwrap();
        }
        let migrations = historical
            .iter()
            .copied()
            .chain([Migration {
                version: 34,
                name: "retire-trait-similarity",
                sql: "",
                backfill: Some(atlas_storage::DataBackfill {
                    reconstruct_tables: false,
                    id: "retire-trait-similarity/v1",
                    apply: atlas_index::retire_trait_similarity,
                }),
            }])
            .collect::<Vec<_>>();
        workspace.migrate(&migrations, |_| Ok(())).unwrap();
        let read = workspace.read(34).unwrap();
        assert_eq!(read.workspace_key().unwrap(), key);
        assert_eq!(read.reference_revision().unwrap(), revision + 1);
        assert_eq!(
            published_values(read.connection(), &input, true),
            document_values
        );
        assert_eq!(
            rows(read.connection(), "SELECT * FROM saved_lists ORDER BY id"),
            saved
        );
        assert_eq!(
            rows(
                read.connection(),
                "SELECT input_id,vector,vector_hash FROM pending_embedding_values ORDER BY input_id"
            ),
            pending
        );
        assert_eq!(
            rows(
                read.connection(),
                "SELECT name FROM sqlite_schema WHERE name IN ('trait_seed_chunks','trait_vectors')"
            ),
            Vec::<Vec<Value>>::new()
        );
        assert!(rows(read.connection(), "PRAGMA foreign_key_check").is_empty());
    }
}
