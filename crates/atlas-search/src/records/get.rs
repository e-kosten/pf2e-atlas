use atlas_index::{FilterReadIndex, FilteredRecordKeyPage, RecordReadIndex};
use atlas_record::AtlasRecord;

use crate::{AtlasRetrievalService, SearchError};

use super::{
    BrowseRecordsRequest, BrowseRecordsResult, GetRecordRequest, GetRecordsRequest,
    RecordRetrieval, ResolveRecordRequest, resolution,
};

impl RecordRetrieval for AtlasRetrievalService {
    fn get_records(&self, request: GetRecordsRequest<'_>) -> Result<Vec<AtlasRecord>, SearchError> {
        get_records(self.index.as_ref(), request)
    }

    fn get_record(
        &self,
        request: GetRecordRequest<'_>,
    ) -> Result<Option<AtlasRecord>, SearchError> {
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
        browse_records(self.index.as_ref(), request)
    }

    fn resolve_record(
        &self,
        request: ResolveRecordRequest<'_>,
    ) -> Result<Vec<super::RecordResolutionResult>, SearchError> {
        resolution::resolve_record(self.index.as_ref(), request.query, request.filter)
    }
}

fn get_records<I>(
    index: &I,
    request: GetRecordsRequest<'_>,
) -> Result<Vec<AtlasRecord>, SearchError>
where
    I: RecordReadIndex + ?Sized,
{
    index
        .load_records_by_key(request.record_keys)
        .map_err(SearchError::from_record_load)
}

fn browse_records<I>(
    index: &I,
    request: BrowseRecordsRequest<'_>,
) -> Result<BrowseRecordsResult, SearchError>
where
    I: FilterReadIndex + RecordReadIndex + ?Sized,
{
    let resolved_filter = index
        .resolve_metric_filters(request.filter)
        .map_err(SearchError::from_filter)?;
    let filter = resolved_filter.as_ref().or(request.filter);
    let FilteredRecordKeyPage { record_keys, total } = index
        .list_filtered_record_keys(filter, request.sort.into(), request.limit, request.offset)
        .map_err(SearchError::from_filter)?;
    let records = index
        .load_records_by_key(&record_keys)
        .map_err(SearchError::from_record_load)?;
    Ok(BrowseRecordsResult {
        record_keys,
        records,
        total,
    })
}
