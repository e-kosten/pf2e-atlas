//! Exact publication expectations for explicit guarded maintenance recovery.
use crate::{ArtifactInstance, IndexError, SqliteIndexReader};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// A comparison value, not a publication capability. Guarded recovery checks it
/// against the exact committed workspace/context inside the recovery transaction.
/// Revision0 has no published context; later revisions require its stored hash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePublicationExpectation {
    instance: ArtifactInstance,
    context_sha256: Option<String>,
}
impl ReferencePublicationExpectation {
    pub fn from_reader(reader: &SqliteIndexReader) -> Self {
        Self {
            instance: reader.artifact_instance().clone(),
            context_sha256: Some(reader.context_sha256().into()),
        }
    }
    pub fn instance(&self) -> &ArtifactInstance {
        &self.instance
    }
    pub fn context_sha256(&self) -> Option<&str> {
        self.context_sha256.as_deref()
    }
    pub(crate) fn read(connection: &Connection) -> Result<Self, IndexError> {
        let (key,revision,context):(String,u64,Option<String>)=connection.query_row("SELECT workspace_key,reference_revision,(SELECT context_hash FROM artifact_context WHERE singleton=1) FROM workspace WHERE id=1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let value = Self {
            instance: ArtifactInstance::new(key, revision),
            context_sha256: context,
        };
        value.validate()?;
        Ok(value)
    }
    pub(crate) fn validate(&self) -> Result<(), IndexError> {
        let valid_context = match (self.instance.reference_revision(), self.context_sha256()) {
            (0, None) => true,
            (0, Some(_)) | (_, None) => false,
            (_, Some(hash)) => crate::codec::checked_hash(hash),
        };
        if self.instance.workspace_key().is_empty() || !valid_context {
            return Err(IndexError::Invalid(
                "reference recovery expectation has invalid workspace/revision/context identity"
                    .into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn check(&self, connection: &Connection) -> Result<(), IndexError> {
        self.validate()?;
        if self != &Self::read(connection)? {
            return Err(IndexError::Invalid(
                "reference publication differs from the guarded recovery expectation".into(),
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn maintenance(path: &std::path::Path) -> Option<String> {
        rusqlite::Connection::open(path)
            .unwrap()
            .query_row("SELECT maintenance FROM workspace WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap()
    }
    #[test]
    fn stale_workspace_revision_or_context_cannot_clear_maintenance_or_change_any_database_bytes() {
        let input = crate::foundation_tests::fixture(false);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let workspace = atlas_storage::Workspace::at(&path).unwrap();
        let expected = {
            let reader = crate::test_support::open_reader(&path).unwrap();
            ReferencePublicationExpectation::from_reader(&reader)
        };
        let schema = crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
        let owner = workspace.update_owner().unwrap();
        drop(owner.maintenance(schema, "guarded recovery test").unwrap());
        let before = std::fs::read(&path).unwrap();
        for changed in ["workspace", "revision", "context", "missing-context"] {
            let mut wrong = expected.clone();
            match changed {
                "workspace" => {
                    wrong.instance = ArtifactInstance::new(
                        "another workspace".into(),
                        expected.instance.reference_revision(),
                    )
                }
                "revision" => {
                    wrong.instance = ArtifactInstance::new(
                        expected.instance.workspace_key().into(),
                        expected.instance.reference_revision() + 1,
                    )
                }
                "context" => wrong.context_sha256 = Some("0".repeat(64)),
                _ => wrong.context_sha256 = None,
            }
            assert!(
                crate::recover_reference_publication_checked(&owner, schema, &wrong).is_err(),
                "{changed}"
            );
            assert_eq!(maintenance(&path).as_deref(), Some("guarded recovery test"));
            assert_eq!(
                std::fs::read(&path).unwrap(),
                before,
                "{changed} changed database bytes"
            );
        }
        crate::recover_reference_publication_checked(&owner, schema, &expected).unwrap();
        assert_eq!(maintenance(&path), None);
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert_eq!(
            ReferencePublicationExpectation::from_reader(&reader),
            expected
        );
        drop(reader);
        let recovered = std::fs::read(&path).unwrap();
        assert!(
            crate::recover_reference_publication_checked(&owner, schema, &expected).is_err(),
            "no maintenance is not authenticated recovery"
        );
        assert_eq!(std::fs::read(&path).unwrap(), recovered);
    }
    #[test]
    fn guarded_recovery_uses_exact_stored_context_bytes_instead_of_reserialization() {
        let input = crate::foundation_tests::fixture(false);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let c = rusqlite::Connection::open(&path).unwrap();
        let json: String = c
            .query_row(
                "SELECT context_json FROM artifact_context WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let spaced = format!("\n{json}\n");
        let hash = crate::codec::sha256(spaced.as_bytes());
        c.execute(
            "UPDATE artifact_context SET context_json=?,context_hash=? WHERE singleton=1",
            rusqlite::params![spaced, hash],
        )
        .unwrap();
        drop(c);
        let expected = {
            let reader = crate::test_support::open_reader(&path).unwrap();
            assert_eq!(reader.context_sha256(), hash);
            assert_ne!(reader.context().sha256().unwrap(), hash);
            ReferencePublicationExpectation::from_reader(&reader)
        };
        let workspace = atlas_storage::Workspace::at(&path).unwrap();
        let schema = crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
        let owner = workspace.update_owner().unwrap();
        drop(owner.maintenance(schema, "exact stored hash").unwrap());
        crate::recover_reference_publication_checked(&owner, schema, &expected).unwrap();
        assert_eq!(maintenance(&path), None);
    }
    #[test]
    fn unpublished_revision_requires_absent_context_and_recovers_without_a_fabricated_hash() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace.sqlite");
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        let schema = crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
        let expected = {
            let read = workspace.read(schema).unwrap();
            ReferencePublicationExpectation::read(read.connection()).unwrap()
        };
        assert_eq!(expected.instance.reference_revision(), 0);
        assert_eq!(expected.context_sha256(), None);
        let owner = workspace.update_owner().unwrap();
        drop(owner.maintenance(schema, "unpublished guard").unwrap());
        let mut wrong = expected.clone();
        wrong.context_sha256 = Some("a".repeat(64));
        let before = std::fs::read(&path).unwrap();
        assert!(crate::recover_reference_publication_checked(&owner, schema, &wrong).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        crate::recover_reference_publication_checked(&owner, schema, &expected).unwrap();
        assert_eq!(maintenance(&path), None);
    }
}
