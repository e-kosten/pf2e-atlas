//! Complete source-discovery evidence and compact publication outcomes.
use crate::{IndexBuildInput, IndexError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAdmissionSummary {
    pub discovered_files: usize,
    pub admitted_records: usize,
    pub partial_records: usize,
    pub quarantined_files: usize,
    pub excluded_documents: usize,
    pub exclusions: Vec<SourceExcludedInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceExcludedInput {
    pub pack: String,
    pub path: String,
    pub reason: String,
}

/// Evidence supplied by the filesystem owner after discovering every declared pack.
/// The writer binds it to the exact source fingerprint and publication pack set.
#[derive(Debug)]
pub struct CompleteSourceInventory {
    fingerprint: String,
    packs: BTreeSet<String>,
}

impl CompleteSourceInventory {
    pub fn checked(
        fingerprint: &str,
        declared: impl IntoIterator<Item = String>,
        discovered: impl IntoIterator<Item = String>,
    ) -> Result<Self, IndexError> {
        let declared = declared.into_iter().collect::<Vec<_>>();
        let packs = declared.iter().cloned().collect::<BTreeSet<_>>();
        let discovered = discovered.into_iter().collect::<Vec<_>>();
        let observed = discovered.iter().cloned().collect::<BTreeSet<_>>();
        if !crate::codec::checked_hash(fingerprint)
            || packs.len() != declared.len()
            || observed.len() != discovered.len()
            || packs != observed
        {
            return Err(IndexError::Invalid(
                "source inventory is incomplete or ambiguous".into(),
            ));
        }
        Ok(Self {
            fingerprint: fingerprint.into(),
            packs,
        })
    }

    pub(crate) fn validate(&self, input: &IndexBuildInput) -> Result<(), IndexError> {
        let packs = input
            .packs
            .iter()
            .map(|p| p.pack_id.clone())
            .collect::<BTreeSet<_>>();
        if self.fingerprint != input.context.source_fingerprint || self.packs != packs {
            return Err(IndexError::Invalid(
                "complete source inventory differs from publication input".into(),
            ));
        }
        Ok(())
    }
}

impl SourceAdmissionSummary {
    pub(crate) fn validate(&self) -> Result<(), IndexError> {
        let excluded = self.quarantined_files.checked_add(self.excluded_documents);
        let total = excluded.and_then(|n| n.checked_add(self.admitted_records));
        if (self.admitted_records == 0 && self.discovered_files > 0)
            || total != Some(self.discovered_files)
            || excluded != Some(self.exclusions.len())
            || self.partial_records > self.admitted_records
            || self.exclusions.iter().any(|e| {
                e.pack.is_empty()
                    || e.path.is_empty()
                    || e.reason.is_empty()
                    || std::path::Path::new(&e.path)
                        .components()
                        .any(|p| !matches!(p, std::path::Component::Normal(_)))
            })
        {
            return Err(IndexError::Invalid(
                "source admission summary is inconsistent".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_discovery_requires_every_declaration_and_exact_publication_identity() {
        let hash = "a".repeat(64);
        assert!(
            CompleteSourceInventory::checked(&hash, ["a".into(), "b".into()], ["a".into()])
                .is_err()
        );
        assert!(
            CompleteSourceInventory::checked(&hash, ["a".into(), "a".into()], ["a".into()])
                .is_err()
        );
        let mut input = crate::foundation_tests::fixture(false);
        input.inventory =
            CompleteSourceInventory::checked(&hash, ["items".into()], ["items".into()]).unwrap();
        let (_directory, path) =
            crate::foundation_tests::build(&crate::foundation_tests::fixture(false));
        assert!(crate::test_support::publish_input(&path, &input).is_err());
        input.inventory = CompleteSourceInventory::checked(
            &"b".repeat(64),
            input.packs.iter().map(|p| p.pack_id.clone()),
            input.packs.iter().map(|p| p.pack_id.clone()),
        )
        .unwrap();
        assert!(crate::test_support::publish_input(&path, &input).is_err());
    }
    #[test]
    fn historical_absent_admission_does_not_change_context_hash() {
        let input = crate::foundation_tests::fixture(false);
        let json = serde_json::to_value(&input.context).unwrap();
        assert!(json.get("admission").is_none());
        let restored: crate::SourceArtifactBuildContext = serde_json::from_value(json).unwrap();
        assert!(restored.admission.is_none());
        assert_eq!(input.context.sha256().unwrap(), restored.sha256().unwrap());
    }
}

#[cfg(test)]
mod upgrade_tests {
    #[test]
    fn metadata_upgrade_preserves_historical_unknown_admission_without_source_or_models() {
        let input = crate::foundation_tests::fixture(false);
        let (_directory, path) = crate::foundation_tests::build(&input);
        let connection = rusqlite::Connection::open(&path).unwrap();
        let revision: u64 = connection
            .query_row("SELECT reference_revision FROM workspace", [], |r| r.get(0))
            .unwrap();
        let mut historical = input.context.clone();
        historical.core_sections_recipe.clear();
        assert!(historical.admission.is_none());
        let json = serde_json::to_string(&historical).unwrap();
        assert!(
            serde_json::from_str::<serde_json::Value>(&json)
                .unwrap()
                .get("admission")
                .is_none()
        );
        connection
            .execute(
                "UPDATE artifact_context SET context_json=?,context_hash=?",
                rusqlite::params![json, historical.sha256().unwrap()],
            )
            .unwrap();
        assert_eq!(
            crate::classify_reference_upgrade(&connection).unwrap(),
            crate::ReferenceUpgrade::CoreProjections
        );
        drop(connection);
        let workspace = crate::test_support::initialize_workspace(&path).unwrap();
        assert!(
            crate::ensure_reference_projections(
                &workspace,
                crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION
            )
            .unwrap()
        );
        let read = workspace
            .read(crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        let upgraded = crate::read_stored_reference_context(read.connection())
            .unwrap()
            .unwrap();
        assert!(upgraded.admission.is_none());
        let stored: String = read
            .connection()
            .query_row("SELECT context_json FROM artifact_context", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(
            serde_json::from_str::<serde_json::Value>(&stored)
                .unwrap()
                .get("admission")
                .is_none()
        );
        assert_eq!(upgraded.source_fingerprint, historical.source_fingerprint);
        assert_eq!(
            upgraded.preparation_sha256().unwrap(),
            historical.preparation_sha256().unwrap()
        );
        assert_eq!(read.reference_revision().unwrap(), revision + 1);
        assert_eq!(
            crate::validate_reference(read.connection()).unwrap().roots,
            4
        );
    }
}
