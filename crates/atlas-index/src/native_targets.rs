//! Explicit update-owned complete registration, promotion and bounded retirement.
use crate::{
    ArtifactInstance, IndexError, NATIVE_VALUE_BATCH_LIMIT, PublicationSource,
    SourceSearchProfileIdentity,
};
use atlas_embedding::NativeInputDescriptor;
use atlas_storage::{UpdateOwner, Workspace};
use rusqlite::{Connection, params};

pub(crate) fn check_owner(owner: &UpdateOwner, workspace: &Workspace) -> Result<(), IndexError> {
    if owner.path() != workspace.path() {
        return Err(IndexError::Invalid(
            "native update owner belongs to another workspace".into(),
        ));
    }
    Ok(())
}
fn check_instance(c: &Connection, expected: &ArtifactInstance) -> Result<(), IndexError> {
    let (key, revision): (String, u64) = c.query_row(
        "SELECT workspace_key,reference_revision FROM workspace WHERE id=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if key != expected.workspace_key() || revision != expected.reference_revision() {
        return Err(IndexError::Invalid(
            "workspace generation changed during native registration".into(),
        ));
    }
    Ok(())
}

/// Retire only known obsolete metadata. Historical jobs with unknown membership
/// remain intact until a complete new target has actually been promoted.
fn retire_metadata(
    workspace: &Workspace,
    schema: u32,
    expected: &ArtifactInstance,
    unknown: bool,
) -> Result<(), IndexError> {
    loop {
        let write = workspace.write(schema)?;
        let c = write.connection();
        check_instance(c, expected)?;
        let mut changed=c.execute("DELETE FROM native_target_inputs WHERE (job_key,input_id) IN (SELECT i.job_key,i.input_id FROM native_target_inputs i JOIN native_update_targets t USING(job_key) WHERE t.phase='retired' LIMIT 128)",[])?;
        // The parent FK is deliberately explicit: migration batches may suspend
        // FK actions, and lifecycle retirement never relies on a cascade.
        let empty=c.prepare("SELECT job_key FROM native_update_targets t WHERE phase='retired' AND NOT EXISTS(SELECT 1 FROM native_target_inputs i WHERE i.job_key=t.job_key) LIMIT 128")?.query_map([],|r|r.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
        for job in empty {
            changed += c.execute("DELETE FROM native_update_targets WHERE job_key=?", [&job])?;
            changed += c.execute("DELETE FROM reference_update_jobs WHERE job_key=?", [&job])?;
        }
        if unknown {
            changed+=c.execute("DELETE FROM reference_update_jobs WHERE job_key IN (SELECT j.job_key FROM reference_update_jobs j WHERE NOT EXISTS(SELECT 1 FROM native_update_targets t WHERE t.job_key=j.job_key) LIMIT 128)",[])?;
        }
        write.commit()?;
        if changed == 0 {
            break;
        }
    }
    Ok(())
}

/// Collect only inert metadata. This is safe before complete promotion because
/// every pending blob, including historical unknown-membership work, is retained.
fn prune_inert_metadata(
    workspace: &Workspace,
    schema: u32,
    expected: &ArtifactInstance,
) -> Result<(), IndexError> {
    loop {
        let write = workspace.write(schema)?;
        let c = write.connection();
        check_instance(c, expected)?;
        let mut changed=c.execute("DELETE FROM embedding_inputs WHERE input_id IN (SELECT i.input_id FROM embedding_inputs i WHERE NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=i.input_id) AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=i.input_id) AND NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=i.input_id) AND NOT EXISTS(SELECT 1 FROM native_target_inputs n WHERE n.input_id=i.input_id) LIMIT 128)",[])?;
        changed+=c.execute("DELETE FROM embedding_models WHERE model_id IN (SELECT m.model_id FROM embedding_models m WHERE NOT EXISTS(SELECT 1 FROM embedding_inputs i WHERE i.model_id=m.model_id) LIMIT 128)",[])?;
        write.commit()?;
        if changed == 0 {
            break;
        }
    }
    Ok(())
}

fn prune_pending(
    workspace: &Workspace,
    schema: u32,
    expected: &ArtifactInstance,
    job: &str,
) -> Result<(), IndexError> {
    loop {
        let write = workspace.write(schema)?;
        let c = write.connection();
        check_instance(c, expected)?;
        crate::native_selection::require_active(c, job)?;
        // Pending blobs are independent of published storage: obsolete pending
        // bytes are removable even when the same input has published consumers.
        let mut changed=c.execute("DELETE FROM pending_embedding_values WHERE input_id IN (SELECT p.input_id FROM pending_embedding_values p WHERE NOT EXISTS(SELECT 1 FROM native_target_inputs i WHERE i.job_key=? AND i.input_id=p.input_id) LIMIT 128)",[job])?;
        changed+=c.execute("DELETE FROM embedding_inputs WHERE input_id IN (SELECT i.input_id FROM embedding_inputs i WHERE NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=i.input_id) AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=i.input_id) AND NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=i.input_id) AND NOT EXISTS(SELECT 1 FROM native_target_inputs n WHERE n.input_id=i.input_id) LIMIT 128)",[])?;
        changed+=c.execute("DELETE FROM embedding_models WHERE model_id IN (SELECT m.model_id FROM embedding_models m WHERE NOT EXISTS(SELECT 1 FROM embedding_inputs i WHERE i.model_id=m.model_id) LIMIT 128)",[])?;
        write.commit()?;
        if changed == 0 {
            break;
        }
    }
    let read = workspace.read(schema)?;
    let (count, bytes): (u64, u64) = read.connection().query_row(
        "SELECT count(*),coalesce(sum(length(vector)),0) FROM pending_embedding_values",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    tracing::info!(target:"atlas_build_diagnostic",pending_native_values=count,logical_pending_bytes=bytes,"Retained active native checkpoints");
    Ok(())
}

/// Register every requested profile's exact membership before inference or GC.
/// A failed/incomplete candidate cannot displace or prune the previous active
/// target. Each metadata/checkpoint transaction is bounded; no SQL session spans
/// source visitation or native inference.
pub fn register_native_target(
    owner: &UpdateOwner,
    workspace: &Workspace,
    schema: u32,
    source: &dyn PublicationSource,
) -> Result<String, IndexError> {
    check_owner(owner, workspace)?;
    source.check_seal()?;
    crate::writer::validate_context(source.context())?;
    let expected = {
        let read = workspace.read(schema)?;
        let actual = ArtifactInstance::new(read.workspace_key()?, read.reference_revision()?);
        if let Some(expected) = source.expected_instance() {
            check_instance(read.connection(), expected)?;
        }
        actual
    };
    let target = source.context().sha256()?;
    {
        let write = workspace.write(schema)?;
        check_instance(write.connection(), &expected)?;
        write.connection().execute(
            "UPDATE native_update_targets SET phase='retired' WHERE phase='candidate'",
            [],
        )?;
        write.commit()?;
    }
    retire_metadata(workspace, schema, &expected, false)?;
    prune_inert_metadata(workspace, schema, &expected)?;
    let job = {
        let write = workspace.write(schema)?;
        let c = write.connection();
        check_instance(c, &expected)?;
        let job: String = c.query_row("SELECT lower(hex(randomblob(32)))", [], |r| r.get(0))?;
        c.execute("INSERT INTO reference_update_jobs(job_key,expected_revision,target_hash,state) VALUES(?,?,?,'preparing')",params![job,expected.reference_revision(),target])?;
        c.execute(
            "INSERT INTO native_update_targets(job_key,phase) VALUES(?,'candidate')",
            [&job],
        )?;
        write.commit()?;
        job
    };
    let mut rows: Vec<(SourceSearchProfileIdentity, NativeInputDescriptor)> =
        Vec::with_capacity(NATIVE_VALUE_BATCH_LIMIT);
    let mut members = 0usize;
    let mut registered = 0usize;
    let mut flush=|rows:&mut Vec<(SourceSearchProfileIdentity,NativeInputDescriptor)>|->Result<(),IndexError>{
        if rows.is_empty(){return Ok(());}
        let write=workspace.write(schema)?;let c=write.connection();check_instance(c,&expected)?;
        for (model,input) in rows.iter(){let id=crate::embedding_values::input_id(c,model,&input.input_sha256,input.token_count)?;members+=c.execute("INSERT INTO native_target_inputs(job_key,input_id) VALUES(?,?) ON CONFLICT DO NOTHING",params![job,id])?;}
        registered+=rows.len();c.execute("UPDATE native_update_targets SET registered_count=?,member_count=? WHERE job_key=? AND phase='candidate'",params![registered,members,job])?;write.commit()?;rows.clear();Ok(())
    };
    let manifest = crate::native_manifest::visit_native_manifest(source, &mut |model, input| {
        rows.push((model.clone(), input.clone()));
        if rows.len() == NATIVE_VALUE_BATCH_LIMIT {
            flush(&mut rows)?;
        }
        Ok(())
    })?;
    flush(&mut rows)?;
    // Independent repeat traversal authenticates complete source membership,
    // rather than declaring whatever candidate rows happened to arrive complete.
    let mut checked = Vec::with_capacity(NATIVE_VALUE_BATCH_LIMIT);
    let verify=|rows:&mut Vec<(SourceSearchProfileIdentity,NativeInputDescriptor)>|->Result<(),IndexError>{let read=workspace.read(schema)?;check_instance(read.connection(),&expected)?;for(model,input)in rows.iter(){crate::native_selection::member(read.connection(),&job,model,input)?;}rows.clear();Ok(())};
    let proof = crate::native_manifest::visit_native_manifest(source, &mut |model, input| {
        checked.push((model.clone(), input.clone()));
        if checked.len() == NATIVE_VALUE_BATCH_LIMIT {
            verify(&mut checked)?;
        }
        Ok(())
    })?;
    if !checked.is_empty() {
        verify(&mut checked)?;
    }
    if proof != manifest || registered != manifest.occurrences {
        return Err(IndexError::Invalid(
            "native registration is incomplete or changed".into(),
        ));
    }
    {
        let write = workspace.write(schema)?;
        let c = write.connection();
        check_instance(c, &expected)?;
        let actual: usize = c.query_row(
            "SELECT count(*) FROM native_target_inputs WHERE job_key=?",
            [&job],
            |r| r.get(0),
        )?;
        if actual != members {
            return Err(IndexError::Invalid(
                "native registration member count differs".into(),
            ));
        }
        c.execute(
            "UPDATE native_update_targets SET phase='retired' WHERE phase='active'",
            [],
        )?;
        let changed=c.execute("UPDATE native_update_targets SET phase='active',membership_digest=? WHERE job_key=? AND phase='candidate' AND registered_count=? AND member_count=?",params![manifest.digest,job,registered,members])?;
        if changed != 1 {
            return Err(IndexError::Invalid(
                "native candidate disappeared before promotion".into(),
            ));
        }
        write.commit()?;
    }
    retire_metadata(workspace, schema, &expected, true)?;
    prune_pending(workspace, schema, &expected, &job)?;
    Ok(job)
}
