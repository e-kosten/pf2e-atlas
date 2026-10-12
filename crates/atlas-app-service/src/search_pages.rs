use crate::projection::{localized_summary_view, search_page_view, search_results_view};
use crate::retrieval::retrieval_mode;
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppErrorCode, AppRecoverableAction, SearchCriteria, SearchModeSummary, SearchResultRow,
    SearchResultsPage, SearchResultsRequest,
};
use atlas_search::{ListRecordsRequest, RecordScope, SearchPage, TextSearchRequest};

impl AtlasAppService {
    pub fn search_page(
        &self,
        request: SearchResultsRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<SearchResultsPage> {
        execution.check()?;
        let page = SearchPage::new(request.page.number, request.page.size)?;
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |retrieval| {
            let artifact_generation = retrieval.artifact_generation()?;
            if let Some(expected) = &request.expected_artifact_generation
                && expected != &artifact_generation
            {
                return Err(AppServiceError::artifact_generation_changed(
                    expected,
                    &artifact_generation,
                    AppRecoverableAction::RefreshResults,
                ));
            }
            let (mode, rows, page, coverage) = match request.criteria {
                SearchCriteria::SimilarRecords {
                    seed_record_key,
                    filter,
                } => {
                    let key = atlas_domain::RecordKey::parse(&seed_record_key).map_err(|e| {
                        AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string())
                    })?;
                    let result = retrieval
                        .similar_records(atlas_search::SimilarRecordRequest {
                            record_key: &key,
                            filter: filter.as_ref(),
                            scope: RecordScope::All,
                            page,
                            prefer_remaster: true,
                        })?
                        .ok_or_else(|| {
                            AppServiceError::new(
                                AppErrorCode::RecordNotFound,
                                "similar seed not found",
                            )
                        })?;
                    projection_execution.check()?;
                    let seed = localized_summary_view(&result.seed, retrieval);
                    let results = search_results_view(result.results, retrieval);
                    (
                        SearchModeSummary::SimilarRecords {
                            seed: Box::new(seed),
                        },
                        results.rows,
                        results.page,
                        Some(results.coverage),
                    )
                }
                SearchCriteria::ListRecords { filter } => {
                    let result =
                        retrieval.list_records(ListRecordsRequest::new(filter.as_ref(), page))?;
                    projection_execution.check()?;
                    (
                        SearchModeSummary::ListRecords,
                        result
                            .records
                            .iter()
                            .map(|record| SearchResultRow {
                                record: localized_summary_view(record, retrieval),
                                matches: vec![],
                            })
                            .collect(),
                        search_page_view(result.page),
                        None,
                    )
                }
                SearchCriteria::TextSearch {
                    query,
                    filter,
                    mode,
                } => {
                    // Stateless paging deliberately reruns query inference and ranking.
                    let result = retrieval.search_text(TextSearchRequest {
                        query: &query,
                        mode: retrieval_mode(mode),
                        filter: filter.as_ref(),
                        scope: RecordScope::All,
                        page,
                        prefer_remaster: true,
                    })?;
                    projection_execution.check()?;
                    let results = search_results_view(result, retrieval);
                    (
                        SearchModeSummary::TextSearch { query, mode },
                        results.rows,
                        results.page,
                        Some(results.coverage),
                    )
                }
            };
            Ok(SearchResultsPage {
                artifact_generation,
                mode,
                page,
                rows,
                coverage,
            })
        })
    }
}

#[cfg(test)]
mod tests;
