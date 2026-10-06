use std::collections::BTreeMap;

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_record::RetrievedRecord;
use atlas_search::{
    AtlasRetrievalService, GraphContextRequest, GraphContextResult, GraphRetrieval,
    ListRecordsRequest, ListRecordsResult, RecordListSort, RecordRetrieval, RemasterLinksRequest,
    RemasterLinksResult, RemasterRetrieval, ResolveVariantGroupRefRequest, SearchError, SearchPage,
    SimilarRecordRefRequest, SimilarRecordRefResult, SimilarRetrieval, SimilarScoreWeights,
    TextRetrieval, TextSearchRequest, TextSearchResult, TextSearchTuning,
    VariantGroupRefResolutionResult, VariantRetrieval,
};

use crate::error::AppServiceResult;
use crate::service::AtlasAppService;

#[derive(Debug, Clone)]
pub(crate) struct VerifiedRemasterLookup {
    links: RemasterLinksResult,
}

impl VerifiedRemasterLookup {
    pub(crate) fn links(&self) -> &RemasterLinksResult {
        &self.links
    }

    #[cfg(test)]
    pub(crate) fn from_test_result(links: RemasterLinksResult) -> Self {
        Self { links }
    }
}

impl AtlasAppService {
    pub fn list_records(
        &self,
        filter: Option<SearchFilterNode>,
        sort: RecordListSort,
        page: SearchPage,
    ) -> AppServiceResult<ListRecordsResult> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval
                .list_records(ListRecordsRequest::new(filter.as_ref(), page).with_sort(sort))?)
        })
    }

    pub fn search_text(
        &self,
        query: String,
        exclude: Option<String>,
        filter: Option<SearchFilterNode>,
        page: SearchPage,
        tuning: Option<TextSearchTuning>,
        explain: bool,
    ) -> AppServiceResult<TextSearchResult> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.search_text(TextSearchRequest {
                query: &query,
                exclude: exclude.as_deref(),
                filter: filter.as_ref(),
                scope: atlas_search::RecordScope::All,
                page,
                tuning,
                explain,
            })?)
        })
    }

    pub fn similar_records_for_ref(
        &self,
        record_ref: String,
        filter: Option<SearchFilterNode>,
        limit: u32,
        candidate_limit: u32,
        weights: SimilarScoreWeights,
    ) -> AppServiceResult<SimilarRecordRefResult> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.similar_records_for_ref(SimilarRecordRefRequest {
                record_ref: &record_ref,
                filter: filter.as_ref(),
                limit,
                candidate_limit,
                weights,
            })?)
        })
    }

    pub fn graph_context(
        &self,
        request: GraphContextRequest,
    ) -> AppServiceResult<Option<GraphContextResult>> {
        self.submit_retrieval(move |retrieval| Ok(retrieval.graph_context(request)?))
    }

    pub fn resolve_variant_group_ref(
        &self,
        variant_group_ref: String,
    ) -> AppServiceResult<VariantGroupRefResolutionResult> {
        self.submit_retrieval(move |retrieval| {
            Ok(
                retrieval.resolve_variant_group_ref(ResolveVariantGroupRefRequest {
                    variant_group_ref: &variant_group_ref,
                })?,
            )
        })
    }

    pub fn remaster_links(
        &self,
        record_key: RecordKey,
    ) -> AppServiceResult<Option<RemasterLinksResult>> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.remaster_links(RemasterLinksRequest {
                record_key: &record_key,
            })?)
        })
    }
}

pub(crate) fn verified_remaster_lookup(
    retrieval: &AtlasRetrievalService,
    record: &RetrievedRecord,
) -> Result<VerifiedRemasterLookup, SearchError> {
    let links = retrieval
        .remaster_links(RemasterLinksRequest {
            record_key: &record.record.identity.key,
        })?
        .ok_or_else(|| {
            SearchError::query_failed(format!(
                "canonical record `{}` disappeared during remaster lookup",
                record.record.identity.key
            ))
        })?;
    Ok(VerifiedRemasterLookup { links })
}

pub(crate) fn verified_remaster_lookups_for_records<'a>(
    retrieval: &AtlasRetrievalService,
    records: impl IntoIterator<Item = &'a RetrievedRecord>,
) -> Result<BTreeMap<String, VerifiedRemasterLookup>, SearchError> {
    let mut lookups_by_key = BTreeMap::new();
    for record in records {
        let key = record.record.identity.key.to_string();
        if lookups_by_key.contains_key(&key) {
            continue;
        }
        lookups_by_key.insert(key, verified_remaster_lookup(retrieval, record)?);
    }
    Ok(lookups_by_key)
}
