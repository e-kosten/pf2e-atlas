//! Stateless link-picker membership, composed from the parent's normal retrieval policy.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppRecoverableAction, SearchConnectionScope, SearchCriteria, SearchResultsRequest,
};
use atlas_domain::{RecordKey, SimilaritySeed};
use atlas_search::{RecordScope, SearchPage, TextSearchRequest};

pub(crate) fn check_generation(expected: Option<&str>, actual: &str) -> AppServiceResult<()> {
    if let Some(expected) = expected
        && expected != actual
    {
        return Err(AppServiceError::artifact_generation_changed(
            expected,
            actual,
            AppRecoverableAction::RefreshResults,
        ));
    }
    Ok(())
}

pub(crate) fn validate_caller_similarity(request: &SearchResultsRequest) -> AppServiceResult<()> {
    match &request.criteria {
        SearchCriteria::SimilarRecords {
            similarity: Some(options),
            ..
        }
        | SearchCriteria::SimilarTraits {
            similarity: Some(options),
            ..
        } => crate::similarity::require_parts_generation(
            options,
            request.expected_artifact_generation.as_deref(),
        ),
        _ => Ok(()),
    }
}

impl AtlasAppService {
    pub(crate) fn connection_keys(
        &self,
        context: &SearchConnectionScope,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<(String, Vec<RecordKey>)> {
        let mut depth = 1;
        let mut parent = &*context.parent;
        while let Some(scope) = &parent.connection_scope {
            depth += 1;
            if depth > 4 {
                return Err(AppServiceError::invalid_request(
                    "Link pickers may be nested at most four levels",
                ));
            }
            parent = &scope.parent;
        }
        let (generation, parents) = self.result_membership(
            (*context.parent).clone(),
            context.list_ref.clone(),
            execution,
        )?;
        let direction = context.direction;
        let include_contents = context.include_contents;
        let operation = execution.clone();
        self.with_retrieval(execution, move |retrieval| {
            check_generation(Some(&generation), &retrieval.artifact_generation()?)?;
            operation.check()?;
            Ok((
                generation,
                retrieval.connected_record_keys(&parents, direction, include_contents)?,
            ))
        })
    }

    fn result_membership(
        &self,
        mut request: SearchResultsRequest,
        list_ref: Option<String>,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<(String, Vec<RecordKey>)> {
        validate_caller_similarity(&request)?;
        let inherited = request
            .connection_scope
            .as_ref()
            .map(|scope| self.connection_keys(scope, execution))
            .transpose()?;
        if let Some((generation, _)) = &inherited {
            check_generation(request.expected_artifact_generation.as_deref(), generation)?;
            request.expected_artifact_generation = Some(generation.clone());
        }
        let text = match &request.criteria {
            SearchCriteria::TextSearch { query, mode, .. } => {
                Some((query.as_str(), crate::retrieval::retrieval_mode(*mode)))
            }
            SearchCriteria::TraitReferences {
                query: Some(query), ..
            } => Some((query.as_str(), atlas_search::RetrievalMode::Hybrid)),
            _ => None,
        };
        // Each text query is prepared outside its consuming snapshot. Parent and picker
        // can use different semantic queries, so they deliberately use separate reads.
        let expected = request.expected_artifact_generation.clone();
        let filter = match &request.criteria {
            SearchCriteria::ListRecords { filter }
            | SearchCriteria::TextSearch { filter, .. }
            | SearchCriteria::SimilarRecords { filter, .. }
            | SearchCriteria::SimilarTraits { filter, .. }
            | SearchCriteria::TraitReferences { filter, .. } => filter.clone(),
        };
        let generation = self.with_retrieval(execution, move |retrieval| {
            let generation = retrieval.artifact_generation()?;
            check_generation(expected.as_deref(), &generation)?;
            if let Some(filter) = filter {
                retrieval.validate_filter(&filter)?;
            }
            Ok(generation)
        })?;
        request.expected_artifact_generation = Some(generation);
        let prepared = if list_ref.is_none() {
            if let Some((query, mode)) = text {
                self.prepare_query(request.search_profile.as_ref(), query, mode)?
            } else {
                None
            }
        } else {
            None
        };
        let operation = execution.clone();
        self.with_prepared_workspace(execution, prepared, move |workspace| {
            let list_keys = if let Some(list_ref) = &list_ref {
                Some(
                    workspace
                        .local_state()
                        .saved_lists()
                        .get_with_items(list_ref)?
                        .ok_or_else(|| AppServiceError::invalid_request("Saved list not found"))?
                        .items
                        .into_iter()
                        .filter_map(|item| RecordKey::parse(&item.record_key).ok())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            };
            let mut keys = inherited.map(|(_, keys)| keys);
            if let Some(list_keys) = list_keys {
                keys = Some(match keys {
                    Some(keys) => {
                        let allowed: std::collections::BTreeSet<_> =
                            list_keys.into_iter().collect();
                        keys.into_iter()
                            .filter(|key| allowed.contains(key))
                            .collect()
                    }
                    None => list_keys,
                });
            }
            let retrieval = workspace.retrieval_mut();
            let generation = retrieval.artifact_generation()?;
            check_generation(request.expected_artifact_generation.as_deref(), &generation)?;
            let scope = keys
                .as_deref()
                .map(RecordScope::Keys)
                .unwrap_or(RecordScope::All);
            let page = SearchPage::first(1)?;
            let matched = match request.criteria {
                SearchCriteria::ListRecords { filter } => retrieval.all_record_keys(
                    filter.as_ref(),
                    scope,
                    list_ref.is_none() || filter.is_some(),
                )?,
                SearchCriteria::TextSearch {
                    query,
                    filter,
                    mode,
                    search_content,
                } => retrieval.search_text_membership(TextSearchRequest {
                    query: &query,
                    filter: filter.as_ref(),
                    scope,
                    page,
                    prefer_remaster: true,
                    content_scope: search_content.unwrap_or_default(),
                    sort: atlas_domain::ResultSort::Relevance {},
                    ranking: request.ranking.unwrap_or_default(),
                    search_profile: request.search_profile.as_ref(),
                    mode: if list_ref.is_some() {
                        atlas_search::RetrievalMode::Fts
                    } else {
                        crate::retrieval::retrieval_mode(mode)
                    },
                })?,
                SearchCriteria::TraitReferences {
                    trait_key,
                    query,
                    filter,
                } => {
                    let trait_key =
                        trait_key
                            .parse()
                            .map_err(|error: atlas_domain::TraitKeyParseError| {
                                AppServiceError::invalid_request(error.to_string())
                            })?;
                    let mut trait_keys = retrieval.trait_reference_record_keys(&trait_key)?;
                    if let Some(keys) = keys {
                        let keys: std::collections::BTreeSet<_> = keys.into_iter().collect();
                        trait_keys.retain(|key| keys.contains(key));
                    }
                    let scope = RecordScope::Keys(&trait_keys);
                    if let Some(query) = query {
                        retrieval.search_text_membership(TextSearchRequest {
                            query: &query,
                            filter: filter.as_ref(),
                            scope,
                            page,
                            prefer_remaster: true,
                            content_scope: Default::default(),
                            sort: atlas_domain::ResultSort::Relevance {},
                            ranking: request.ranking.unwrap_or_default(),
                            search_profile: request.search_profile.as_ref(),
                            mode: atlas_search::RetrievalMode::Hybrid,
                        })?
                    } else {
                        retrieval.all_record_keys(filter.as_ref(), scope, true)?
                    }
                }
                criteria @ (SearchCriteria::SimilarRecords { .. }
                | SearchCriteria::SimilarTraits { .. }) => {
                    let (seed, filter, options, sections, content) = match criteria {
                        SearchCriteria::SimilarRecords {
                            seed_record_key,
                            filter,
                            similarity,
                            source_sections,
                            search_content,
                        } => (
                            SimilaritySeed::Record {
                                record_key: RecordKey::parse(&seed_record_key).map_err(
                                    |error| AppServiceError::invalid_request(error.to_string()),
                                )?,
                            },
                            filter,
                            similarity,
                            source_sections,
                            search_content,
                        ),
                        SearchCriteria::SimilarTraits {
                            seed_trait_key,
                            filter,
                            similarity,
                            source_sections,
                            search_content,
                        } => (
                            SimilaritySeed::Trait {
                                trait_key: seed_trait_key.parse().map_err(
                                    |error: atlas_domain::TraitKeyParseError| {
                                        AppServiceError::invalid_request(error.to_string())
                                    },
                                )?,
                            },
                            filter,
                            similarity,
                            source_sections,
                            search_content,
                        ),
                        _ => unreachable!(),
                    };
                    let options = options.unwrap_or_else(|| {
                        if sections.is_some() {
                            Default::default()
                        } else {
                            atlas_domain::SimilarityOptions::default_for_seed(&seed)
                        }
                    });
                    crate::similarity::validate_options(
                        &seed,
                        &options,
                        request.expected_artifact_generation.as_deref(),
                    )?;
                    retrieval
                        .configurable_similar_membership(
                            atlas_search::ConfigurableSimilarRequest {
                                seed: &seed,
                                options: &options,
                                source_sections: sections,
                                content_scope: content.unwrap_or_default(),
                                search_profile: request.search_profile.as_ref(),
                                filter: filter.as_ref(),
                                scope,
                                page,
                                prefer_remaster: true,
                                sort: atlas_domain::ResultSort::Relevance {},
                                cancelled: Some(&|| operation.check().is_err()),
                            },
                        )?
                        .ok_or_else(|| {
                            AppServiceError::invalid_request("The similarity seed is unavailable")
                        })?
                }
            };
            operation.check()?;
            Ok((generation, matched))
        })
    }
}
