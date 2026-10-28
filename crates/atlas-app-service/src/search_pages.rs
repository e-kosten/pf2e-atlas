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
        let query_request = match &request.criteria {
            SearchCriteria::TextSearch {
                query,
                filter,
                mode,
                ..
            } => Some((query, filter, *mode)),
            SearchCriteria::TraitReferences {
                query: Some(query),
                filter,
                ..
            } => Some((query, filter, atlas_app_model::RetrievalModeView::Hybrid)),
            _ => None,
        };
        let prepared = if let Some((query, filter, mode)) = query_request {
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
            self.prepare_query(request.search_profile.as_ref(), query, retrieval_mode(mode))?
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
                | SearchCriteria::SimilarRecords { filter, .. }
                | SearchCriteria::SimilarTraits { filter, .. }
                | SearchCriteria::TraitReferences { filter, .. } => filter.as_ref(),
            };
            let relationship_summaries =
                crate::relationship_search::prepare_filter(retrieval, filter)?;
            let evidence_filter = filter.cloned();
            let scope = RecordScope::All;
            let cancelled = || projection_execution.check().is_err();
            let search_content = match &request.criteria {
                SearchCriteria::TextSearch { search_content, .. }
                | SearchCriteria::SimilarRecords { search_content, .. }
                | SearchCriteria::SimilarTraits { search_content, .. } => {
                    search_content.unwrap_or_default()
                }
                _ => atlas_domain::SearchContentScope::All,
            };
            let mut similarity = None;
            let (mode, mut rows, page, coverage) = match request.criteria {
                SearchCriteria::SimilarRecords {
                    seed_record_key,
                    filter,
                    similarity: options,
                    source_sections,
                    search_content: _,
                } => {
                    let key = atlas_domain::RecordKey::parse(&seed_record_key).map_err(|e| {
                        AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string())
                    })?;
                    let seed = atlas_domain::SimilaritySeed::Record { record_key: key };
                    let options = options.unwrap_or_else(|| {
                        if source_sections.is_some() {
                            atlas_domain::SimilarityOptions::default()
                        } else {
                            atlas_domain::SimilarityOptions::default_for_seed(&seed)
                        }
                    });
                    crate::similarity::validate_options(
                        &seed,
                        &options,
                        request.expected_artifact_generation.as_deref(),
                    )?;
                    let (mode, results, effective) = crate::similarity::configured_results(
                        retrieval,
                        atlas_search::ConfigurableSimilarRequest {
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
                            prefer_remaster: true,
                        },
                        &projection_execution,
                    )?;
                    projection_execution.check()?;
                    similarity = Some(effective);
                    (mode, results.rows, results.page, Some(results.coverage))
                }
                SearchCriteria::SimilarTraits {
                    seed_trait_key,
                    filter,
                    similarity: options,
                    source_sections,
                    search_content: _,
                } => {
                    let key = seed_trait_key
                        .parse::<atlas_domain::TraitKey>()
                        .map_err(|error| AppServiceError::invalid_request(error.to_string()))?;
                    let seed = atlas_domain::SimilaritySeed::Trait { trait_key: key };
                    let options = options.unwrap_or_else(|| {
                        if source_sections.is_some() {
                            atlas_domain::SimilarityOptions::default()
                        } else {
                            atlas_domain::SimilarityOptions::default_for_seed(&seed)
                        }
                    });
                    crate::similarity::validate_options(
                        &seed,
                        &options,
                        request.expected_artifact_generation.as_deref(),
                    )?;
                    let (mode, results, effective) = crate::similarity::configured_results(
                        retrieval,
                        atlas_search::ConfigurableSimilarRequest {
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
                            prefer_remaster: true,
                        },
                        &projection_execution,
                    )?;
                    projection_execution.check()?;
                    similarity = Some(effective);
                    (mode, results.rows, results.page, Some(results.coverage))
                }
                SearchCriteria::TraitReferences {
                    trait_key,
                    query,
                    filter,
                } => {
                    let key = trait_key
                        .parse::<atlas_domain::TraitKey>()
                        .map_err(|error| AppServiceError::invalid_request(error.to_string()))?;
                    let definition = retrieval.trait_definition(&key)?.ok_or_else(|| {
                        AppServiceError::invalid_request("Trait definition not found")
                    })?;
                    let keys = retrieval.trait_reference_record_keys(&key)?;
                    let scope = RecordScope::Keys(&keys);
                    let mode = SearchModeSummary::TraitReferences {
                        trait_key,
                        title: definition.label,
                    };
                    if let Some(query) = query {
                        let result = retrieval.search_text(TextSearchRequest {
                            content_scope: atlas_domain::SearchContentScope::All,
                            sort: sort.clone(),
                            ranking: request.ranking.unwrap_or_default(),
                            search_profile: request.search_profile.as_ref(),
                            query: &query,
                            mode: atlas_search::RetrievalMode::Hybrid,
                            filter: filter.as_ref(),
                            scope,
                            page,
                            prefer_remaster: true,
                        })?;
                        projection_execution.check()?;
                        let result = search_results_view(result, retrieval);
                        (mode, result.rows, result.page, Some(result.coverage))
                    } else {
                        let result = retrieval.list_records(
                            ListRecordsRequest::new(filter.as_ref(), page)
                                .with_scope(scope)
                                .with_sort(sort.clone()),
                        )?;
                        projection_execution.check()?;
                        let rows = result
                            .records
                            .iter()
                            .map(|record| SearchResultRow {
                                similarity: None,
                                relationship_evidence: vec![],
                                record: localized_summary_view(record, retrieval),
                                matches: vec![],
                                matching_sections: None,
                            })
                            .collect();
                        (mode, rows, search_page_view(result.page), None)
                    }
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
                    search_content: _,
                } => {
                    // Stateless paging deliberately reruns query inference and ranking.
                    let result = retrieval.search_text(TextSearchRequest {
                        content_scope: search_content,
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
