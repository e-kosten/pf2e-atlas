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
        let prepared = if let SearchCriteria::TextSearch {
            query,
            filter,
            mode,
        } = &request.criteria
        {
            let expected = request.expected_artifact_generation.clone();
            let filter = filter.clone();
            let relationships = request.relationships.clone().unwrap_or_default();
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
                crate::relationship_search::prepare_constraints(r, &relationships)?;
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
            let (relationships, relationship_summaries) =
                crate::relationship_search::prepare_constraints(
                    retrieval,
                    request.relationships.as_deref().unwrap_or_default(),
                )?;
            let scope = RecordScope::Constrained {
                keys: None,
                relationships: &relationships,
            };
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
                        ListRecordsRequest::new(filter.as_ref(), page).with_scope(scope),
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
            for row in &mut rows {
                projection_execution.check()?;
                let key = atlas_domain::RecordKey::parse(&row.record.record_key).map_err(|_| {
                    AppServiceError::invalid_request("The result record is unavailable")
                })?;
                for (constraint_index, constraint) in relationships.iter().enumerate() {
                    let (occurrences, page) = crate::relationship_search::evidence_page(
                        retrieval,
                        constraint,
                        &key,
                        SearchPage::new(1, 3)?,
                    )?;
                    row.relationship_evidence
                        .push(atlas_app_model::RelationshipEvidenceView {
                            constraint_index,
                            occurrences,
                            page,
                        });
                }
            }
            Ok(SearchResultsPage {
                relationships: relationship_summaries,
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
