//! Reference update job lifecycle and validated maintenance recovery.
use crate::{IndexError, codec::sha256};
use atlas_storage::WriteSession;
use rusqlite::params;

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
    connection.execute("DELETE FROM reference_update_jobs WHERE job_key<>?", [job])?;
    Ok(())
}
