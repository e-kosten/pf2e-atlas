//! Validated recovery of interrupted atomic reference maintenance.
use crate::IndexError;

pub fn recover_reference_publication(
    owner: &atlas_storage::UpdateOwner,
    schema: u32,
) -> Result<bool, IndexError> {
    recover_reference_publication_inner(owner, schema, None)
}

/// Recover only an interrupted maintenance operation whose exact prior
/// publication matches the expectation. A workspace without maintenance returns
/// an error: no guarded/authenticated recovery was performed in that case.
pub fn recover_reference_publication_checked(
    owner: &atlas_storage::UpdateOwner,
    schema: u32,
    expected: &crate::ReferencePublicationExpectation,
) -> Result<(), IndexError> {
    expected.validate()?;
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    if !recover_reference_publication_inner(owner, schema, Some(expected))? {
        return Err(IndexError::Invalid(
            "guarded recovery requires interrupted maintenance; expectation was not authenticated"
                .into(),
        ));
    }
    Ok(())
}
fn recover_reference_publication_inner(
    owner: &atlas_storage::UpdateOwner,
    schema: u32,
    expected: Option<&crate::ReferencePublicationExpectation>,
) -> Result<bool, IndexError> {
    owner.recover_maintenance(schema, |connection| {
        if let Some(expected) = expected {
            expected.check(connection)?;
        }

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
                crate::component_upgrade::ReferenceUpgrade::SourceReplacement
                | crate::component_upgrade::ReferenceUpgrade::SourcePreparation => {
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
                | crate::component_upgrade::ReferenceUpgrade::SectionProjections
                | crate::component_upgrade::ReferenceUpgrade::CoreProjections => {
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
                    crate::visit_persisted_inputs(connection, &context, &mut |_| Ok(()))?;
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
