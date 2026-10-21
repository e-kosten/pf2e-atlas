//! Attribute order consumes an accepted matching set without choosing its members.
use crate::{AtlasRetrievalService, SearchError, SourceUnitPrice};
use atlas_domain::{RecordKey, ResultSort};
use std::collections::{BTreeMap, BTreeSet};

impl AtlasRetrievalService {
    pub(crate) fn validate_search_sort(sort: &ResultSort) -> Result<(), SearchError> {
        sort.validate()
            .map_err(SearchError::invalid_search_options)?;
        if matches!(sort, ResultSort::SavedOrder {}) {
            return Err(SearchError::invalid_search_options(
                "Saved order is available only for saved lists",
            ));
        }
        Ok(())
    }
    pub fn order_record_keys(
        &self,
        keys: &[RecordKey],
        sort: ResultSort,
    ) -> Result<Vec<RecordKey>, SearchError> {
        let ordered = self.index.order_record_keys(keys, sort)?;
        if ordered.len() != keys.len()
            || ordered.iter().collect::<BTreeSet<_>>() != keys.iter().collect::<BTreeSet<_>>()
        {
            return Err(SearchError::artifact_contract_violation(
                "ordered candidates have missing or unexpected record keys",
            ));
        }
        Ok(ordered)
    }
    pub fn read_unit_prices(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SourceUnitPrice>, SearchError> {
        let facts = self.index.read_unit_prices(keys)?;
        if facts.len() != keys.len() {
            return Err(SearchError::artifact_contract_violation(
                "unit-price candidates have missing facts",
            ));
        }
        Ok(facts)
    }
    pub(crate) fn order_ranked_roots(
        &self,
        roots: Vec<crate::text::RankedRoot>,
        sort: ResultSort,
    ) -> Result<Vec<crate::text::RankedRoot>, SearchError> {
        Self::validate_search_sort(&sort)?;
        if matches!(sort, ResultSort::Relevance {}) {
            return Ok(roots);
        }
        let keys = roots.iter().map(|r| r.key.clone()).collect::<Vec<_>>();
        let order = self.order_record_keys(&keys, sort)?;
        let mut by_key = roots
            .into_iter()
            .map(|r| (r.key.clone(), r))
            .collect::<BTreeMap<_, _>>();
        order
            .into_iter()
            .map(|key| {
                by_key.remove(&key).ok_or_else(|| {
                    SearchError::artifact_contract_violation("ordered candidate is absent")
                })
            })
            .collect()
    }
}
