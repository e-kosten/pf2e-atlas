use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_index::{FilteredRecordKeyPage, FilteredRecordSort};
use atlas_record::PersistedRecord;

use crate::{AtlasRetrievalService, SearchError};

#[derive(Debug, Clone, PartialEq)]
pub struct FilterOnlyRecordPage {
    pub record_keys: Vec<RecordKey>,
    pub records: Vec<PersistedRecord>,
    pub total: u64,
}

impl AtlasRetrievalService {
    pub fn get_records(
        &self,
        record_keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        let mut records = self.index.load_records_by_key(record_keys)?;
        self.enrich_reference_labels(&mut records)?;
        Ok(records)
    }

    pub fn get_record(
        &self,
        record_key: &RecordKey,
    ) -> Result<Option<PersistedRecord>, SearchError> {
        Ok(self
            .get_records(std::slice::from_ref(record_key))?
            .into_iter()
            .next())
    }

    pub fn filter_only_records(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilterOnlyRecordPage, SearchError> {
        let resolved_filter = self.index.resolve_metric_filters(filter)?;
        let filter = resolved_filter.as_ref().or(filter);
        let FilteredRecordKeyPage { record_keys, total } = self
            .index
            .list_filtered_record_keys(filter, sort, limit, offset)?;
        let mut records = self.index.load_records_by_key(&record_keys)?;
        self.enrich_reference_labels(&mut records)?;
        Ok(FilterOnlyRecordPage {
            record_keys,
            records,
            total,
        })
    }
}
