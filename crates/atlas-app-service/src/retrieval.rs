use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_search::{
    GraphContextRequest, GraphContextResult, GraphRetrieval, ListRecordsRequest, ListRecordsResult,
    RecordListSort, RecordRetrieval, RemasterLinksRequest, RemasterLinksResult, RemasterRetrieval,
    ResolveVariantGroupRefRequest, SearchPage, SimilarRecordRefRequest, SimilarRecordRefResult,
    SimilarRetrieval, SimilarScoreWeights, TextRetrieval, TextSearchRequest, TextSearchResult,
    TextSearchTuning, VariantGroupRefResolutionResult, VariantRetrieval,
};

use crate::error::AppServiceResult;
use crate::service::AtlasAppService;

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
