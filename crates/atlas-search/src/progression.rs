use crate::{AtlasRetrievalService, SearchError};
use atlas_domain::RecordKey;
use atlas_index::{PrerequisiteDependent, ProgressionPage};
use atlas_record::progression::ProgressionRecord;
use std::collections::BTreeSet;

impl AtlasRetrievalService {
    pub fn read_progression(
        &self,
        key: &RecordKey,
    ) -> Result<Option<ProgressionRecord>, SearchError> {
        Ok(self.index.read_progression(key)?)
    }

    pub fn has_progression(&self, key: &RecordKey) -> Result<bool, SearchError> {
        Ok(self.index.has_progression(key)?)
    }

    pub fn related_progression_keys(
        &self,
        keys: &[RecordKey],
    ) -> Result<BTreeSet<RecordKey>, SearchError> {
        Ok(self.index.related_progression_keys(keys)?)
    }

    pub fn read_prerequisite_dependents(
        &self,
        key: &RecordKey,
        offset: usize,
        limit: usize,
    ) -> Result<ProgressionPage<PrerequisiteDependent>, SearchError> {
        Ok(self
            .index
            .read_prerequisite_dependents(key, offset, limit)?)
    }

    pub fn read_ancestry_feat_pool(
        &self,
        trait_key: &str,
        offset: usize,
        limit: usize,
    ) -> Result<ProgressionPage<RecordKey>, SearchError> {
        Ok(self
            .index
            .read_ancestry_feat_pool(trait_key, offset, limit)?)
    }

    pub fn read_ancestry_feat_access(
        &self,
        key: &RecordKey,
        offset: usize,
        limit: usize,
    ) -> Result<ProgressionPage<RecordKey>, SearchError> {
        Ok(self.index.read_ancestry_feat_access(key, offset, limit)?)
    }
}
