use crate::projection::{localized_summary_view, search_page_view, search_results_view};
use crate::retrieval::retrieval_mode;
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppErrorCode, SearchCriteria, SearchModeSummary, SearchResultRow, SearchResultsPage,
    SearchResultsRequest,
};
use atlas_search::{ListRecordsRequest, RecordScope, SearchPage, TextSearchRequest};

impl AtlasAppService {
    pub fn search_page(
        &self,
        request: SearchResultsRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<SearchResultsPage> {
        execution.check()?;
        crate::similarity::validate_caller_similarity(&request)?;
        let page = SearchPage::new(request.page.number, request.page.size)?;
        let sort = crate::result_order::search_sort(request.sort, &request.criteria)?;
        let projection_execution = execution.clone();
        let result_detail = request.result_detail;
        let query_request = match &request.criteria {
            SearchCriteria::TextSearch {
                query,
                filter,
                mode,
                lexical,
                ..
            } => {
                if lexical.is_some() && *mode != atlas_app_model::RetrievalModeView::Lexical {
                    return Err(AppServiceError::invalid_request(
                        "Lexical options require lexical retrieval",
                    ));
                }
                (*mode != atlas_app_model::RetrievalModeView::Lexical)
                    .then_some((query, filter, *mode))
            }
            _ => None,
        };
        let prepared = if let Some((query, filter, mode)) = query_request {
            let expected = request.expected_artifact_generation.clone();
            let filter = filter.clone();
            let connection = request.connection_scope.clone();
            self.with_retrieval(execution, move |r| {
                crate::connection_scope::check_generation(
                    expected.as_deref(),
                    &r.artifact_generation()?,
                )?;
                if let Some(filter) = filter {
                    r.validate_filter(&filter)?;
                }
                if let Some(connection) = connection {
                    crate::connection_scope::validate_connection_scope(r, &connection)?;
                }
                Ok(())
            })?;
            self.prepare_query(request.search_profile.as_ref(), query, retrieval_mode(mode))?
        } else {
            None
        };
        execution.check()?;
        self.with_prepared_workspace(execution, prepared, move |workspace| {
            let artifact_generation = workspace.retrieval().artifact_generation()?;
            let lexical = matches!(
                request.criteria,
                SearchCriteria::TextSearch {
                    mode: atlas_app_model::RetrievalModeView::Lexical,
                    ..
                }
            );
            let check_generation = || {
                crate::connection_scope::check_generation(
                    request.expected_artifact_generation.as_deref(),
                    &artifact_generation,
                )
            };
            if lexical {
                check_generation()?;
                if let SearchCriteria::TextSearch {
                    filter: Some(filter),
                    ..
                } = &request.criteria
                {
                    workspace.retrieval_mut().prepare_page_filter(filter)?;
                }
            }
            let connected = request
                .connection_scope
                .as_ref()
                .map(|scope| crate::connection_scope::resolve_connection_scope(workspace, scope))
                .transpose()?;
            if !lexical {
                check_generation()?;
            }
            let retrieval = workspace.retrieval_mut();
            let search_profile =
                retrieval.resolve_search_profile(request.search_profile.as_ref(), false)?;
            let filter = match &request.criteria {
                SearchCriteria::ListRecords { filter }
                | SearchCriteria::TextSearch { filter, .. }
                | SearchCriteria::SimilarRecords { filter, .. } => filter.as_ref(),
            };
            if !lexical && let Some(filter) = filter {
                retrieval.prepare_page_filter(filter)?;
            }
            let relationship_summaries =
                crate::relationship_search::prepare_filter(retrieval, filter)?;

            let evidence_filter = filter
                .filter(|_| result_detail.includes_evidence() && !relationship_summaries.is_empty())
                .cloned();
            let scope = connected
                .as_ref()
                .and_then(|scope| scope.keys.as_deref())
                .map(RecordScope::Keys)
                .unwrap_or(RecordScope::All);
            let cancelled = || projection_execution.check().is_err();
            let search_content = match &request.criteria {
                SearchCriteria::TextSearch { search_content, .. }
                | SearchCriteria::SimilarRecords { search_content, .. } => {
                    search_content.unwrap_or_default()
                }
                _ => atlas_domain::SearchContentScope::All,
            };
            let mut similarity = None;
            let (mode, mut rows, page, coverage) = match request.criteria {
                SearchCriteria::SimilarRecords {
                    seed_record_key,
                    embedded_path,
                    source_fingerprint,
                    filter,
                    similarity: options,
                    source_sections,
                    search_content: _,
                } => {
                    let key = atlas_domain::RecordKey::parse(&seed_record_key).map_err(|e| {
                        AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string())
                    })?;
                    let seed = atlas_domain::SimilaritySeed::Record {
                        record_key: key,
                        embedded_path,
                        source_fingerprint,
                    };
                    let options = options.unwrap_or_default();
                    crate::similarity::validate_options(
                        &options,
                        request.expected_artifact_generation.as_deref(),
                    )?;
                    let (mode, results, effective) = crate::similarity::configured_results(
                        retrieval,
                        atlas_search::ConfigurableSimilarRequest {
                            result_detail,
                            cancelled: Some(&cancelled),
                            seed: &seed,
                            options: &options,
                            source_sections,
                            content_scope: search_content,
                            sort: sort.clone(),
                            search_profile: request.search_profile.as_ref(),
                            filter: filter.as_ref(),
                            scope,
                            page,
                        },
                        &projection_execution,
                    )?;
                    projection_execution.check()?;
                    similarity = Some(effective);
                    (mode, results.rows, results.page, Some(results.coverage))
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
                                exact_journal_pages: None,
                                similarity: None,
                                relationship_evidence: vec![],
                                record: localized_summary_view(record, retrieval),
                                matches: vec![],
                                matching_sections: None,
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
                    lexical,
                    search_content: _,
                } => {
                    let result = retrieval.search_text(TextSearchRequest {
                        lexical: lexical.unwrap_or_default(),
                        result_detail,
                        content_scope: search_content,
                        sort: sort.clone(),
                        ranking: request.ranking.unwrap_or_default(),
                        search_profile: request.search_profile.as_ref(),
                        query: &query,
                        mode: retrieval_mode(mode),
                        filter: filter.as_ref(),
                        scope,
                        page,
                    })?;
                    projection_execution.check()?;
                    let results = search_results_view(result, result_detail, retrieval);
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
            if let Some(filter) = &evidence_filter {
                let keys = rows
                    .iter()
                    .map(|row| {
                        atlas_domain::RecordKey::parse(&row.record.record_key).map_err(|_| {
                            AppServiceError::invalid_request("The result record is unavailable")
                        })
                    })
                    .collect::<AppServiceResult<Vec<_>>>()?;
                let proofs = retrieval.relationship_proofs(filter, &keys)?;
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
            }
            if request.include_summary_definitions {
                crate::presentation::reference_help::annotate_summaries(
                    rows.iter_mut().map(|row| &mut row.record),
                    retrieval,
                )?;
            }
            Ok(SearchResultsPage {
                result_detail,
                connection_scope: connected.map(|scope| scope.view),
                core_recipe: if matches!(mode, SearchModeSummary::TextSearch { .. })
                    && search_content == atlas_domain::SearchContentScope::Core
                {
                    Some(atlas_domain::CORE_SECTIONS_RECIPE.into())
                } else {
                    similarity
                        .as_ref()
                        .and_then(|configuration| configuration.core_recipe.clone())
                },
                search_content,
                similarity,
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
