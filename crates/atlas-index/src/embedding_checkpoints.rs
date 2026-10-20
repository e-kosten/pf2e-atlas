//! Durable exact-input vectors, independent of active content/projection versions.
use crate::{
    IndexError, SourceSearchProfileIdentity,
    codec::sha256,
    read_units::{decode_vector, vector_bytes},
    validation::check_vector,
};
use atlas_embedding::PreparedEmbeddingInput;
use atlas_storage::{ReadSession, WriteSession};
use rusqlite::{OptionalExtension, params};
use std::collections::{BTreeMap, BTreeSet};

pub fn recover_reference_publication(
    owner: &atlas_storage::UpdateOwner,
    schema: u32,
) -> Result<bool, IndexError> {
    owner.recover_maintenance(schema, |connection| {
        let revision: u64 = connection.query_row(
            "SELECT reference_revision FROM workspace WHERE id=1",
            [],
            |row| row.get(0),
        )?;
        if revision > 0 {
            match crate::component_upgrade::classify_reference_upgrade(connection)? {
                crate::component_upgrade::ReferenceUpgrade::Current => {
                    crate::validate_reference(connection)?;
                }
                crate::component_upgrade::ReferenceUpgrade::SourceReplacement => {
                    // Recovery opens the workspace for an explicit source rebuild,
                    // not for product reads of the retired snapshot contract.
                    let integrity: String =
                        connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
                    if integrity != "ok" {
                        return Err(IndexError::Invalid(format!(
                            "SQLite integrity: {integrity}"
                        )));
                    }
                    let mut foreign_keys = connection.prepare("PRAGMA foreign_key_check")?;
                    if foreign_keys.query([])?.next()?.is_some() {
                        return Err(IndexError::Invalid(
                            "workspace foreign key violation".into(),
                        ));
                    }
                }
                crate::component_upgrade::ReferenceUpgrade::QueryProjections
                | crate::component_upgrade::ReferenceUpgrade::LexicalProjections
                | crate::component_upgrade::ReferenceUpgrade::SourcePreparation => {
                    // Old derived outputs cannot be validated against new policy.
                    // Authenticate retained inputs and storage integrity; product
                    // readers remain blocked by their component-version checks.
                    let integrity: String =
                        connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
                    if integrity != "ok" {
                        return Err(IndexError::Invalid(format!(
                            "SQLite integrity: {integrity}"
                        )));
                    }
                    let context =
                        crate::read_stored_reference_context(connection)?.ok_or_else(|| {
                            IndexError::Invalid("published reference has no context".into())
                        })?;
                    crate::read_persisted_inputs(connection, &context)?;
                }
            }
        } else {
            let partial: bool = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM artifact_context) OR EXISTS(SELECT 1 FROM records)",
                [],
                |row| row.get(0),
            )?;
            if partial {
                return Err(IndexError::Invalid(
                    "unpublished workspace contains partial reference data".into(),
                ));
            }
        }
        Ok(())
    })
}

pub fn start_reference_job(
    session: &WriteSession,
    target_hash: &str,
) -> Result<String, IndexError> {
    let revision = session.reference_revision()?;
    let job = sha256(format!("{revision}:{target_hash}").as_bytes());
    session.connection().execute(
        "INSERT INTO reference_update_jobs(job_key,expected_revision,target_hash,state) VALUES(?,?,?,'preparing') ON CONFLICT(job_key) DO NOTHING",
        params![job, revision, target_hash],
    )?;
    Ok(job)
}

/// Called only inside the final reference publication transaction, after its
/// semantic rows and profile identities have been reconciled.
pub(crate) fn finish_reference_job(
    connection: &rusqlite::Connection,
    job: &str,
    target_hash: &str,
) -> Result<(), IndexError> {
    let updated = connection.execute(
        "UPDATE reference_update_jobs SET state='published',updated_at=CURRENT_TIMESTAMP WHERE job_key=? AND target_hash=? AND state='preparing' AND expected_revision=(SELECT reference_revision FROM workspace WHERE id=1)",
        params![job, target_hash],
    )?;
    if updated != 1 {
        return Err(IndexError::Invalid(
            "reference update job no longer matches the workspace revision or target".into(),
        ));
    }
    // Retain every exact active input, even if a caller did not touch its cache
    // row during this run. Abandoned inputs are pruned only after publication.
    // Decode each trait's native chunk evidence once. The checkpoint corpus is
    // much larger than the vocabulary: a correlated json_each would repeatedly
    // parse every chunk vector for every checkpoint before testing its identity.
    connection.execute_batch(ACTIVE_TRAIT_INPUTS_SQL)?;
    connection.execute(PRUNE_CHECKPOINTS_SQL, [])?;
    connection.execute_batch("DROP TABLE temp.active_trait_checkpoint_inputs")?;
    connection.execute("DELETE FROM reference_update_jobs WHERE job_key<>?", [job])?;
    Ok(())
}

const ACTIVE_TRAIT_INPUTS_SQL:&str="
CREATE TEMP TABLE active_trait_checkpoint_inputs(
 input_hash TEXT NOT NULL,
 token_count INTEGER NOT NULL,
 dimensions INTEGER NOT NULL,
 model_identity TEXT NOT NULL,
 PRIMARY KEY(input_hash,token_count,dimensions,model_identity)
) WITHOUT ROWID;
INSERT OR IGNORE INTO temp.active_trait_checkpoint_inputs
SELECT json_extract(h.value,'$.input_hash'),json_extract(h.value,'$.token_count'),p.dimensions,p.identity_json
FROM trait_vectors tv JOIN search_profiles p ON p.profile_id=tv.profile_id,json_each(tv.chunks_json) h;
";
const PRUNE_CHECKPOINTS_SQL: &str = "
DELETE FROM embedding_checkpoints AS c
WHERE NOT EXISTS (
 SELECT 1 FROM temp.active_trait_checkpoint_inputs a
 WHERE a.input_hash=c.input_hash AND a.token_count=c.token_count
 AND a.dimensions=c.dimensions AND a.model_identity=c.model_identity
) AND NOT EXISTS (
 SELECT 1 FROM semantic_units u JOIN search_profiles p ON p.profile_id=u.profile_id
 WHERE u.input_hash=c.input_hash AND u.input_token_count=c.token_count
 AND p.identity_json=c.model_identity AND p.dimensions=c.dimensions
)";

pub fn read_embedding_checkpoints(
    session: &ReadSession,
    model: &SourceSearchProfileIdentity,
    inputs: &[PreparedEmbeddingInput],
) -> Result<BTreeMap<String, Vec<f32>>, IndexError> {
    let identity = serde_json::to_string(model)?;
    let mut statement = session.connection().prepare_cached(
        "SELECT input_hash,token_count,dimensions,vector,vector_hash FROM embedding_checkpoints WHERE reuse_key=? AND model_identity=?",
    )?;
    let mut reusable = BTreeMap::new();
    let mut visited = BTreeSet::new();
    for input in inputs {
        if !visited.insert(&input.reuse_key) {
            continue;
        }
        let row = statement
            .query_row(params![input.reuse_key, identity], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, usize>(1)?,
                    r.get::<_, usize>(2)?,
                    r.get::<_, Vec<u8>>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .optional()?;
        let Some((hash, tokens, dimensions, bytes, vector_hash)) = row else {
            continue;
        };
        if hash != input.input_sha256
            || tokens != input.token_count
            || dimensions != model.dimensions
            || sha256(&bytes) != vector_hash
        {
            return Err(IndexError::Invalid(
                "embedding checkpoint identity or checksum mismatch".into(),
            ));
        }
        let vector = decode_vector(&bytes)?;
        check_vector(&vector, model.dimensions)?;
        reusable.insert(input.reuse_key.clone(), vector);
    }
    Ok(reusable)
}

/// The caller commits this bounded batch after inference has released its work.
pub fn save_embedding_checkpoints(
    session: &WriteSession,
    job: &str,
    model: &SourceSearchProfileIdentity,
    batch: &[(&PreparedEmbeddingInput, &[f32])],
) -> Result<(), IndexError> {
    let identity = serde_json::to_string(model)?;
    let mut statement = session.connection().prepare_cached(
        "INSERT INTO embedding_checkpoints(reuse_key,model_identity,input_hash,token_count,dimensions,vector,vector_hash,touched_job) VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(reuse_key) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,touched_job=excluded.touched_job WHERE model_identity=excluded.model_identity AND input_hash=excluded.input_hash AND token_count=excluded.token_count AND dimensions=excluded.dimensions",
    )?;
    for (input, vector) in batch {
        check_vector(vector, model.dimensions)?;
        let bytes = vector_bytes(vector);
        let affected = statement.execute(params![
            input.reuse_key,
            identity,
            input.input_sha256,
            input.token_count,
            model.dimensions,
            bytes,
            sha256(&bytes),
            job
        ])?;
        if affected != 1 {
            return Err(IndexError::Invalid(
                "embedding checkpoint reuse key has conflicting input identity".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_checkpoint_pruning_uses_materialized_identity_primary_key() {
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            crate::test_support::initialize_workspace(directory.path().join("workspace.sqlite"))
                .unwrap();
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        let c = write.connection();
        c.execute_batch(ACTIVE_TRAIT_INPUTS_SQL).unwrap();
        let plan = c
            .prepare(&format!("EXPLAIN QUERY PLAN {PRUNE_CHECKPOINTS_SQL}"))
            .unwrap()
            .query_map([], |r| r.get::<_, String>(3))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(plan.iter().any(|step|step.contains("SEARCH a USING PRIMARY KEY (input_hash=? AND token_count=? AND dimensions=? AND model_identity=?)")),"{plan:?}");
        assert!(
            !plan.iter().any(|step| step.contains("VIRTUAL TABLE")),
            "checkpoint deletion must not scan json_each for each checkpoint: {plan:?}"
        );
        assert!(
            plan.iter()
                .any(|step| step.contains("semantic_units_input_hash")),
            "normal-record checkpoint lookup stays indexed: {plan:?}"
        );
    }

    #[test]
    fn publication_commits_job_and_prunes_only_inactive_inputs() {
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            crate::test_support::initialize_workspace(directory.path().join("workspace.sqlite"))
                .unwrap();
        let fixture = crate::foundation_tests::fixture(true);
        let model = &fixture.context.search_profiles[0];
        let unit = &fixture.semantic_units[0];
        let active = PreparedEmbeddingInput {
            address: unit.address.clone(),
            input: String::new(),
            input_sha256: unit.input_hash.clone(),
            reuse_key: "active-input".into(),
            token_count: unit.input_token_count,
            body_token_count: unit.input_token_count,
            overlap_token_count: 0,
            context_shortened: false,
        };
        let mut abandoned = active.clone();
        abandoned.input_sha256 = sha256(b"unused input");
        abandoned.reuse_key = "unused-input".into();
        let target = fixture.context.sha256().unwrap();
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        let job = start_reference_job(&write, &target).unwrap();
        save_embedding_checkpoints(
            &write,
            &job,
            model,
            &[(&active, &unit.vector), (&abandoned, &unit.vector)],
        )
        .unwrap();
        write.commit().unwrap();
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(
                crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
                "publication",
            )
            .unwrap();
        // Rejected job identity rolls the complete reference publication back.
        assert!(
            crate::SqliteIndexWriter::new(&maintenance)
                .write_job(&fixture, "missing-job", &target)
                .is_err()
        );
        let count: usize = maintenance
            .connection()
            .query_row("SELECT count(*) FROM records", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let cached: usize = maintenance
            .connection()
            .query_row("SELECT count(*) FROM embedding_checkpoints", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(cached, 2);
        crate::SqliteIndexWriter::new(&maintenance)
            .write_job(&fixture, &job, &target)
            .unwrap();
        drop(maintenance);
        drop(owner);
        let read = workspace
            .read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        assert_eq!(read.reference_revision().unwrap(), 1);
        let state: String = read
            .connection()
            .query_row(
                "SELECT state FROM reference_update_jobs WHERE job_key=?",
                [&job],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "published");
        let restored =
            read_embedding_checkpoints(&read, model, &[active.clone(), abandoned]).unwrap();
        assert_eq!(restored.len(), 1);
        assert!(restored.contains_key(&active.reuse_key));
    }

    #[test]
    fn committed_batches_survive_reopen_and_reject_corruption() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace.sqlite");
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        let fixture = crate::foundation_tests::fixture(true);
        let model = &fixture.context.search_profiles[0];
        let unit = &fixture.semantic_units[0];
        let input = PreparedEmbeddingInput {
            address: unit.address.clone(),
            input: "fixture text".into(),
            input_sha256: sha256(b"fixture text"),
            reuse_key: sha256(b"fixture model and input"),
            token_count: 3,
            body_token_count: 3,
            overlap_token_count: 0,
            context_shortened: false,
        };
        let mut vector = vec![0.0; model.dimensions];
        vector[0] = 1.0;
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        let job = start_reference_job(&write, "target").unwrap();
        save_embedding_checkpoints(&write, &job, model, &[(&input, &vector)]).unwrap();
        write.commit().unwrap();
        drop(workspace);
        let workspace = atlas_storage::Workspace::at(&path).unwrap();
        let read = workspace
            .read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        let restored =
            read_embedding_checkpoints(&read, model, std::slice::from_ref(&input)).unwrap();
        assert_eq!(restored[&input.reuse_key], vector);
        drop(read);

        // A reused key cannot silently replace a different input identity.
        let mut conflicting = input.clone();
        conflicting.input_sha256 = sha256(b"different text");
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        assert!(
            save_embedding_checkpoints(&write, &job, model, &[(&conflicting, &vector)]).is_err()
        );
        drop(write);
        let write = workspace
            .write(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        write
            .connection()
            .execute("UPDATE embedding_checkpoints SET vector_hash='corrupt'", [])
            .unwrap();
        write.commit().unwrap();
        let read = workspace
            .read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        assert!(read_embedding_checkpoints(&read, model, &[input]).is_err());
    }
}
