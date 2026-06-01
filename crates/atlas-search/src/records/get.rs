use atlas_index::{FilterReadIndex, FilteredRecordKeyPage, RecordReadIndex};
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
        let mut records = get_records(self.index.as_ref(), request)?;
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
        let mut result = browse_records(self.index.as_ref(), request)?;
        self.enrich_reference_labels(&mut result.records)?;
        Ok(result)
    }

    fn resolve_record(
        &self,
        request: ResolveRecordRequest<'_>,
    ) -> Result<Vec<super::RecordResolutionResult>, SearchError> {
        let mut matches =
            resolution::resolve_record(self.index.as_ref(), request.query, request.filter)?;
        self.enrich_resolution_reference_labels(&mut matches)?;
        Ok(matches)
    }
}

fn get_records<I>(
    index: &I,
    request: GetRecordsRequest<'_>,
) -> Result<Vec<PersistedRecord>, SearchError>
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
