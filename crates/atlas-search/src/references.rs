//! Definition reads over the same bounded reference snapshot as ordinary records.
use crate::{AtlasRetrievalService, SearchError};
use atlas_domain::{DefinitionKey, OwnedContentLocator, RecordKey};
pub use atlas_index::{
    OwnedAbilityTarget, ReferenceAvailability, ReferenceBacking, ReferenceCoverage, ReferenceEntry,
    ReferenceGrantScope, ReferenceGrantUse, ReferenceNotice, ReferencePermission,
    ReferencePermissionDisposition, ReferenceProvenance, ReferencePublication,
    ReferenceRecordMapping, ReferenceSummary, ReferenceSummaryBacking,
};

impl AtlasRetrievalService {
    pub fn reference_coverage(&self) -> ReferenceCoverage {
        self.index.reference_coverage()
    }

    pub fn reference_definition(
        &self,
        key: &DefinitionKey,
    ) -> Result<Option<ReferenceEntry>, SearchError> {
        Ok(self.index.reference_entry(key)?)
    }

    pub fn reference_summary(
        &self,
        key: &DefinitionKey,
    ) -> Result<Option<ReferenceSummary>, SearchError> {
        Ok(self.index.reference_summary(key)?)
    }

    /// At most 100 exact keys; the canonical envelope is checked without decoding
    /// prepared bodies. No source files or query models are needed.
    pub fn reference_summaries(
        &self,
        keys: &[DefinitionKey],
    ) -> Result<Vec<ReferenceSummary>, SearchError> {
        Ok(self.index.reference_summaries(keys)?)
    }

    pub fn reference_notices(&self, ids: &[String]) -> Result<Vec<ReferenceNotice>, SearchError> {
        Ok(self.index.reference_notices(ids)?)
    }

    pub fn owned_ability_help(
        &self,
        record: &RecordKey,
        owners: &[OwnedContentLocator],
    ) -> Result<Option<OwnedAbilityTarget>, SearchError> {
        Ok(self.index.owned_ability_help(record, owners)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{input, open_input_fixture, record};
    use serde_json::json;

    #[test]
    fn retained_coverage_reports_setup_separately_from_ordinary_source_free_reads() {
        let source = record(
            "items",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"action","name":"Retained action"}),
        );
        let (retrieval, _artifact) = open_input_fixture(input(vec![source], false)).unwrap();
        assert_eq!(
            retrieval.reference_coverage(),
            ReferenceCoverage::TraitsOnly
        );
        let unknown: DefinitionKey = "trait:unpublished-custom".parse().unwrap();
        assert!(retrieval.reference_summary(&unknown).unwrap().is_none());
        let new_key = "language:common".parse().unwrap();
        assert_eq!(
            retrieval
                .reference_summaries(&[new_key])
                .unwrap_err()
                .kind(),
            crate::SearchErrorKind::IndexUnavailable
        );
        assert_eq!(
            retrieval
                .get_records(crate::GetRecordsRequest {
                    record_keys: &["items:aaaaaaaaaaaaaaaa".parse().unwrap()]
                })
                .unwrap()
                .records
                .len(),
            1
        );
        assert!(
            retrieval
                .reference_notices(&["unknown".into()])
                .unwrap()
                .is_empty()
        );
    }
}
