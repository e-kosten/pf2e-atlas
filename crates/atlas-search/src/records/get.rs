use atlas_index::FilteredRecordKeyPage;
use atlas_record::PersistedRecord;

use crate::{AtlasRetrievalService, SearchError};

use super::{
    BrowseRecordsRequest, BrowseRecordsResult, GetRecordRequest, GetRecordsRequest,
    RecordRetrieval, ResolveRecordRequest, resolution,
};

impl RecordRetrieval for AtlasRetrievalService {
    fn get_records(
        &self,
        request: GetRecordsRequest<'_>,
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        let mut records = self
            .index
            .load_records_by_key(request.record_keys)
            .map_err(SearchError::from_record_load)?;
        self.enrich_reference_labels(&mut records)?;
        Ok(records)
    }

    fn get_record(
        &self,
        request: GetRecordRequest<'_>,
    ) -> Result<Option<PersistedRecord>, SearchError> {
        Ok(self
            .get_records(GetRecordsRequest {
                record_keys: std::slice::from_ref(request.record_key),
            })?
            .into_iter()
            .next())
    }

    fn browse_records(
        &self,
        request: BrowseRecordsRequest<'_>,
    ) -> Result<BrowseRecordsResult, SearchError> {
        let resolved_filter = self
            .index
            .resolve_metric_filters(request.filter)
            .map_err(SearchError::from_filter)?;
        let filter = resolved_filter.as_ref().or(request.filter);
        let FilteredRecordKeyPage { record_keys, total } = self
            .index
            .list_filtered_record_keys(filter, request.sort.into(), request.limit, request.offset)
            .map_err(SearchError::from_filter)?;
        let mut records = self
            .index
            .load_records_by_key(&record_keys)
            .map_err(SearchError::from_record_load)?;
        self.enrich_reference_labels(&mut records)?;
        Ok(BrowseRecordsResult {
            record_keys,
            records,
            total,
        })
    }

    fn resolve_record(
        &self,
        request: ResolveRecordRequest<'_>,
    ) -> Result<Vec<super::RecordResolutionResult>, SearchError> {
        resolution::resolve_record(self, request.query, request.filter)
    }
}
