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
        let sort = crate::result_order::search_sort(request.sort, &request.criteria)?;
        let projection_execution = execution.clone();
        let prepared = if let SearchCriteria::TextSearch {
            query,
            filter,
            mode,
        } = &request.criteria
        {
            let expected = request.expected_artifact_generation.clone();
            let filter = filter.clone();
            self.with_retrieval(execution, move |r| {
                let actual = r.artifact_generation()?;
                if let Some(expected) = expected
                    && expected != actual
                {
                    return Err(AppServiceError::artifact_generation_changed(
                        &expected,
                        &actual,
                        AppRecoverableAction::RefreshResults,
                    ));
                }
                if let Some(filter) = filter {
                    r.validate_filter(&filter)?;
                }
                Ok(())
            })?;
            self.prepare_query(
                request.search_profile.as_ref(),
                query,
                retrieval_mode(*mode),
            )?
        } else {
            None
        };
        execution.check()?;
        self.with_prepared_workspace(execution, prepared, move |workspace| {
            let retrieval = workspace.retrieval_mut();
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
            let search_profile =
                retrieval.resolve_search_profile(request.search_profile.as_ref(), false)?;
            let filter = match &request.criteria {
                SearchCriteria::ListRecords { filter }
                | SearchCriteria::TextSearch { filter, .. }
                | SearchCriteria::SimilarRecords { filter, .. } => filter.as_ref(),
            };
            let relationship_summaries =
                crate::relationship_search::prepare_filter(retrieval, filter)?;
            let evidence_filter = filter.cloned();
            let scope = RecordScope::All;
            let (mode, mut rows, page, coverage) = match request.criteria {
                SearchCriteria::SimilarRecords {
                    seed_record_key,
                    filter,
                } => {
                    let key = atlas_domain::RecordKey::parse(&seed_record_key).map_err(|e| {
                        AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string())
                    })?;
                    let result = retrieval
                        .similar_records(atlas_search::SimilarRecordRequest {
                            sort: sort.clone(),
                            search_profile: request.search_profile.as_ref(),
                            record_key: &key,
                            filter: filter.as_ref(),
                            scope,
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
                    let result = retrieval.list_records(
                        ListRecordsRequest::new(filter.as_ref(), page)
                            .with_scope(scope)
                            .with_sort(sort.clone()),
                    )?;
                    projection_execution.check()?;
                    (
                        SearchModeSummary::ListRecords,
                        result
                            .records
                            .iter()
                            .map(|record| SearchResultRow {
                                relationship_evidence: vec![],
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
                        sort: sort.clone(),
                        ranking: request.ranking.unwrap_or_default(),
                        search_profile: request.search_profile.as_ref(),
                        query: &query,
                        mode: retrieval_mode(mode),
                        filter: filter.as_ref(),
                        scope,
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
            if sort.is_unit_price() {
                let mut records = rows.iter().map(|r| r.record.clone()).collect::<Vec<_>>();
                crate::result_order::price_labels(retrieval, &mut records)?;
                for (row, record) in rows.iter_mut().zip(records) {
                    row.record = record;
                }
            }
            let keys = rows
                .iter()
                .map(|row| {
                    atlas_domain::RecordKey::parse(&row.record.record_key).map_err(|_| {
                        AppServiceError::invalid_request("The result record is unavailable")
                    })
                })
                .collect::<AppServiceResult<Vec<_>>>()?;
            let proofs = if let Some(filter) = &evidence_filter {
                retrieval.relationship_proofs(filter, &keys)?
            } else {
                vec![]
            };
            for (row, key) in rows.iter_mut().zip(keys) {
                projection_execution.check()?;
                for summary in &relationship_summaries {
                    if !proofs
                        .iter()
                        .any(|(record, id)| record == &key && id == &summary.clause_id)
                    {
                        continue;
                    }
                    let (occurrences, page) = crate::relationship_search::evidence_page(
                        retrieval,
                        &summary.relationship,
                        &key,
                        SearchPage::new(1, 3)?,
                    )?;
                    row.relationship_evidence
                        .push(atlas_app_model::RelationshipEvidenceView {
                            clause_id: summary.clause_id.clone(),
                            occurrences,
                            page,
                        });
                }
            }
            Ok(SearchResultsPage {
                sort,
                relationship_clauses: relationship_summaries,
                ranking: request.ranking.unwrap_or_default(),
                search_profile,
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
