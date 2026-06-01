use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_index::{
    FilterCompileError, FilterReadIndex, FtsQuery, FtsReadIndex, FtsSearchHit, RecordReadIndex,
    SearchCandidateRecord,
};
use atlas_record::PersistedRecord;

use crate::SearchError;
use crate::records::{RecordResolutionResult, RecordRetrieval, ResolveRecordRequest};
use crate::semantic::{SemanticRetrieval, SemanticSearchHit, SemanticSearchRequest};

pub(super) fn resolve_identity_tier<S>(
    search: &S,
    query: &str,
    filter: Option<&SearchFilterNode>,
) -> Result<Vec<RecordResolutionResult>, SearchError>
where
    S: RecordRetrieval + ?Sized,
{
    search.resolve_record(ResolveRecordRequest { query, filter })
}

pub(super) fn search_vector_lane<S>(
    search: &mut S,
    request: SemanticSearchRequest<'_>,
) -> Result<Vec<SemanticSearchHit>, SearchError>
where
    S: SemanticRetrieval + ?Sized,
{
    Ok(search.search_semantic(request)?.hits)
}

pub(super) fn resolve_text_filter<I>(
    index: &I,
    filter: Option<&SearchFilterNode>,
) -> Result<Option<SearchFilterNode>, SearchError>
where
    I: FilterReadIndex + ?Sized,
{
    index
        .resolve_metric_filters(filter)
        .map_err(SearchError::from_filter)
}

pub(super) fn query_precision_fts_index<I>(
    index: &I,
    fts_query: &FtsQuery,
    filter: Option<&SearchFilterNode>,
    limit: u32,
) -> Result<Vec<FtsSearchHit>, SearchError>
where
    I: FtsReadIndex + ?Sized,
{
    index
        .query_precision_fts_index(fts_query, filter, limit)
        .map_err(SearchError::from_filter)
}

pub(super) fn query_fts_candidate_record_keys<I>(
    index: &I,
    fts_query: &FtsQuery,
    candidate_keys: &[RecordKey],
) -> Result<Vec<RecordKey>, FilterCompileError>
where
    I: FtsReadIndex + ?Sized,
{
    index.query_fts_candidate_record_keys(fts_query, candidate_keys)
}

pub(super) fn load_search_candidate_records<I>(
    index: &I,
    keys: &[RecordKey],
) -> Result<Vec<SearchCandidateRecord>, SearchError>
where
    I: RecordReadIndex + ?Sized,
{
    index
        .load_search_candidate_records(keys)
        .map_err(SearchError::from_record_load)
}

pub(super) fn load_records_by_key<I>(
    index: &I,
    keys: &[RecordKey],
) -> Result<Vec<PersistedRecord>, SearchError>
where
    I: RecordReadIndex + ?Sized,
{
    index
        .load_records_by_key(keys)
        .map_err(SearchError::from_record_load)
}
