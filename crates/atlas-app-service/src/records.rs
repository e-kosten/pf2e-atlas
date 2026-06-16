use atlas_app_model::{AppErrorCode, RecordDetailView};
use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, RecordKey, SearchFilterNode};
use atlas_search::{
    DiscoverFilterFieldsRequest, DiscoverFilterValuesRequest, FilterDiscoveryRetrieval,
    GetRecordRequest, GetRecordsRequest, GraphContextRequest, GraphContextResult, GraphRetrieval,
    ListRecordsRequest, ListRecordsResult, RecordListSort, RecordRefResolutionResult,
    RecordResolutionResult, RecordRetrieval, RemasterLinksRequest, RemasterLinksResult,
    RemasterRetrieval, ResolveRecordRefRequest, ResolveRecordRequest,
    ResolveVariantGroupRefRequest, SearchPage, SimilarRecordRefRequest, SimilarRecordRefResult,
    SimilarRetrieval, SimilarScoreWeights, TextRetrieval, TextSearchRequest, TextSearchResult,
    TextSearchTuning, VariantGroupRefResolutionResult, VariantRetrieval,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::projection::record_detail;
use crate::service::AtlasAppService;

#[derive(Debug, Clone)]
pub struct RawFilterValuesRequest {
    pub field: String,
    pub filter: Option<SearchFilterNode>,
    pub filter_json: Option<serde_json::Value>,
    pub sort: Option<atlas_domain::FilterValueSort>,
    pub sample_limit: Option<usize>,
    pub metric_selector: Option<atlas_search::MetricDiscoverySelector>,
    pub metric_domain: Option<String>,
}

impl AtlasAppService {
    pub fn get_records(
        &self,
        record_keys: Vec<RecordKey>,
    ) -> AppServiceResult<Vec<atlas_record::AtlasRecord>> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?)
        })
    }

    pub fn resolve_record(
        &self,
        query: String,
        filter: Option<SearchFilterNode>,
    ) -> AppServiceResult<Vec<RecordResolutionResult>> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.resolve_record(ResolveRecordRequest {
                query: &query,
                filter: filter.as_ref(),
            })?)
        })
    }

    pub fn resolve_record_ref(
        &self,
        record_ref: String,
        filter: Option<SearchFilterNode>,
    ) -> AppServiceResult<RecordRefResolutionResult> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.resolve_record_ref(ResolveRecordRefRequest {
                record_ref: &record_ref,
                filter: filter.as_ref(),
            })?)
        })
    }

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

    pub fn discover_raw_filter_fields(
        &self,
        filter: Option<SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
    ) -> AppServiceResult<FilterFieldDiscovery> {
        self.submit_retrieval(move |retrieval| {
            Ok(
                retrieval.discover_filter_fields(DiscoverFilterFieldsRequest {
                    filter: filter.as_ref(),
                    filter_json,
                })?,
            )
        })
    }

    pub fn discover_raw_filter_values(
        &self,
        request: RawFilterValuesRequest,
    ) -> AppServiceResult<FilterValueDiscovery> {
        self.submit_retrieval(move |retrieval| {
            Ok(
                retrieval.discover_filter_values(DiscoverFilterValuesRequest {
                    field: request.field,
                    filter: request.filter.as_ref(),
                    filter_json: request.filter_json,
                    sort: request.sort,
                    sample_limit: request.sample_limit,
                    metric_selector: request.metric_selector,
                    metric_domain: request.metric_domain,
                })?,
            )
        })
    }

    pub fn record_detail(&self, record_key: &str) -> AppServiceResult<RecordDetailView> {
        let record_key = RecordKey::parse(record_key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        self.submit_retrieval(move |retrieval| {
            let record = retrieval
                .get_record(GetRecordRequest {
                    record_key: &record_key,
                })?
                .ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::RecordNotFound,
                        format!("record `{record_key}` was not found"),
                    )
                })?;
            record_detail(&record)
        })
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::AppErrorCode;

    use crate::test_support::fixture_worker;

    #[test]
    fn worker_record_detail_reports_valid_invalid_and_missing_keys() {
        let fixture = fixture_worker();
        let worker = &fixture.worker;

        let detail = worker
            .record_detail("actions:testAction1")
            .expect("fixture record should load");
        assert_eq!(detail.record_key, "actions:testAction1");
        assert_eq!(detail.title, "Test Action 1");
        assert_eq!(detail.kind, "rule");
        assert_eq!(detail.presentation.title, "Test Action 1");

        let invalid = worker
            .record_detail("not a key")
            .expect_err("invalid keys should be rejected")
            .into_app_error();
        assert_eq!(invalid.code, AppErrorCode::InvalidRecordKey);

        let missing = worker
            .record_detail("actions:missing")
            .expect_err("missing keys should return not found")
            .into_app_error();
        assert_eq!(missing.code, AppErrorCode::RecordNotFound);
    }
}
