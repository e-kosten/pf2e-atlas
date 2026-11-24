//! Generation-checked inventory for stored-vector content comparison.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    AppRecoverableAction, SearchCriteria, SearchResultsRequest, SimilarityPartsRequest,
    SimilarityPartsView,
};
use atlas_domain::{RecordKey, SimilarityOptions, SimilaritySeed};

impl AtlasAppService {
    pub fn similarity_parts(
        &self,
        request: SimilarityPartsRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<SimilarityPartsView> {
        execution.check()?;
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |retrieval| {
            let generation = retrieval.artifact_generation()?;
            check_generation(request.expected_artifact_generation.as_deref(), &generation)?;
            let profile =
                retrieval.resolve_search_profile(request.search_profile.as_ref(), false)?;
            let inventory =
                retrieval.similarity_parts(&request.seed, request.search_profile.as_ref())?;
            projection_execution.check()?;
            let SimilaritySeed::Record { record_key } = &inventory.seed;
            let addresses = retrieval.similarity_part_addresses(record_key)?;
            projection_execution.check()?;
            let parts = inventory
                .parts
                .into_iter()
                .map(|part| {
                    component_view(part, record_key, &addresses, retrieval.source_fingerprint())
                })
                .collect();
            Ok(SimilarityPartsView {
                supports_corresponding_parts: matches!(
                    inventory.seed,
                    SimilaritySeed::Record { .. }
                ),
                artifact_generation: generation,
                search_profile: profile,
                seed: inventory.seed,
                parts,
                core: inventory.core,
                role_recipe: inventory.role_recipe,
            })
        })
    }
}

pub(crate) fn validate_options(
    options: &SimilarityOptions,
    expected_generation: Option<&str>,
) -> AppServiceResult<()> {
    options
        .validate()
        .map_err(AppServiceError::invalid_request)?;
    require_parts_generation(options, expected_generation)
}

pub(crate) fn require_parts_generation(
    options: &SimilarityOptions,
    expected_generation: Option<&str>,
) -> AppServiceResult<()> {
    if options.parts().is_some() && expected_generation.is_none() {
        return Err(AppServiceError::invalid_request(
            "explicit comparison parts require the artifact generation from the part inventory; refresh Compare parts or rerun `atlas similar <SEED> --list-parts`",
        ));
    }
    Ok(())
}

fn check_generation(expected: Option<&str>, actual: &str) -> AppServiceResult<()> {
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

pub(crate) fn configured_results(
    retrieval: &atlas_search::AtlasRetrievalService,
    request: atlas_search::ConfigurableSimilarRequest<'_>,
    execution: &ReadExecutionContext,
) -> AppServiceResult<(
    atlas_app_model::SearchModeSummary,
    atlas_app_model::SearchResultsView,
    atlas_app_model::SimilarityConfigurationView,
)> {
    let result = retrieval.configurable_similar(request)?.ok_or_else(|| {
        AppServiceError::new(
            atlas_app_model::AppErrorCode::RecordNotFound,
            "the selected similarity seed is unavailable",
        )
    })?;
    execution.check()?;
    let mode = atlas_app_model::SearchModeSummary::SimilarRecords {
        seed: Box::new(crate::projection::localized_summary_view(
            &result.seed,
            retrieval,
        )),
    };
    execution.check()?;
    let addresses = if result.evidence.is_empty() {
        Default::default()
    } else {
        retrieval.similarity_part_addresses(&result.seed.key)?
    };
    execution.check()?;
    let mut evidence = result
        .evidence
        .into_iter()
        .map(|(key, evidence)| {
            (
                key.to_string(),
                evidence_view(
                    evidence,
                    &result.seed.key,
                    &addresses,
                    retrieval.source_fingerprint(),
                ),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut results = crate::projection::search_results_view(result.results, retrieval);
    for row in &mut results.rows {
        row.similarity = evidence.remove(&row.record.record_key);
    }
    Ok((
        mode,
        results,
        atlas_app_model::SimilarityConfigurationView {
            source_sections: result.source_sections,
            search_content: result.content_scope,
            candidate_identity: result.candidate_identity,
            core_recipe: result.core_recipe,
            resolved_selectors: result.resolved_selectors,
            core: result.core,
            options: result.effective,
            recipe: result.recipe,
            role_recipe: result.role_recipe,
            diagnostics: atlas_app_model::SimilarityDiagnosticsView {
                seed_units: result.diagnostics.seed_units,
                knn_queries: result.diagnostics.knn_queries,
                candidate_vectors: result.diagnostics.candidate_vectors,
                pairwise_comparisons: result.diagnostics.pairwise_comparisons,
                undefined_candidate_centroids: result.diagnostics.undefined_candidate_centroids,
            },
        },
    ))
}

fn evidence_view(
    evidence: atlas_search::SimilarityEvidence,
    seed: &RecordKey,
    addresses: &PartAddresses,
    fingerprint: &str,
) -> atlas_app_model::SimilarityEvidenceView {
    atlas_app_model::SimilarityEvidenceView {
        truncated: evidence.truncated,
        score: evidence.score,
        aggregation: evidence.aggregation,
        selected_parts: evidence.selected_parts,
        comparable_parts: evidence.comparable_parts,
        representative: evidence.representative,
        parts: evidence
            .parts
            .into_iter()
            .map(|part| atlas_app_model::SimilarityPartScoreView {
                part: component_view(part.part, seed, addresses, fingerprint),
                score: part.score,
                weight: part.weight,
                comparable: part.comparable,
            })
            .collect(),
        roles: evidence
            .roles
            .into_iter()
            .map(|role| atlas_app_model::SimilarityRoleScoreView {
                key: role.key,
                role_label: role.key.label().into(),
                score: role.score,
                weight: role.weight,
                comparable: role.comparable,
            })
            .collect(),
        pairs: evidence
            .pairs
            .into_iter()
            .map(|pair| atlas_app_model::SimilarityPairView {
                seed_part: pair.seed_part,
                seed: pair
                    .seed
                    .map(|witness| crate::projection::search_witness_view(witness, fingerprint)),
                candidate: crate::projection::search_witness_view(pair.candidate, fingerprint),
                cosine: pair.cosine,
            })
            .collect(),
    }
}

type PartAddresses = std::collections::BTreeMap<
    atlas_domain::SimilarityPartSelector,
    atlas_domain::SourcePassageAddress,
>;

fn component_view(
    component: atlas_domain::SimilarityComponent,
    seed: &RecordKey,
    addresses: &PartAddresses,
    fingerprint: &str,
) -> atlas_app_model::SimilarityComponentView {
    match component {
        atlas_domain::SimilarityComponent::Identity {
            selector,
            label,
            owner_label,
            has_vectors,
        } => {
            let navigation = Some(atlas_app_model::RecordNavigationView {
                spell_rank: None,
                record_key: seed.to_string(),
                owners: vec![],
                field: None,
                passage: Some(atlas_domain::SourcePassageAddress::Identity {}),
                source_fingerprint: None,
            });
            atlas_app_model::SimilarityComponentView::Identity {
                navigation,
                selector,
                label,
                owner_label,
                has_vectors,
            }
        }
        atlas_domain::SimilarityComponent::Section { part } => {
            atlas_app_model::SimilarityComponentView::Section {
                part: part_view(part, seed, addresses, fingerprint),
            }
        }
    }
}

fn part_view(
    part: atlas_domain::SimilarityPart,
    seed: &RecordKey,
    addresses: &PartAddresses,
    fingerprint: &str,
) -> atlas_app_model::SimilarityPartView {
    let address = addresses.get(&part.selector);
    let label = address
        .and_then(|address| match address {
            atlas_domain::SourcePassageAddress::HtmlSection { label, .. }
            | atlas_domain::SourcePassageAddress::PlainSection { label, .. } => label.clone(),
            atlas_domain::SourcePassageAddress::Identity {} => None,
        })
        .unwrap_or_else(|| match part.field.as_str() {
            "/system/details/publicNotes" => "Description".into(),
            "/system/details/privateNotes" => "Private notes".into(),
            "/system/details/blurb" => "Blurb".into(),
            _ => part.role.label().into(),
        });
    let navigation = Some(atlas_app_model::RecordNavigationView {
        spell_rank: None,
        record_key: seed.to_string(),
        owners: part.owners.clone(),
        field: Some(part.field.clone()),
        passage: address.cloned(),
        source_fingerprint: crate::projection::navigation_fingerprint(&part.owners, fingerprint),
    });
    atlas_app_model::SimilarityPartView {
        navigation,
        selector: part.selector,
        label,
        owner_label: part.owner_label,
        owners: part.owners,
        field: part.field,
        section_ordinal: part.section_ordinal,
        role: part.role,
        role_label: part.role.label().into(),
        has_vectors: part.has_vectors,
        label_only: part.label_only,
    }
}

pub(crate) fn validate_caller_similarity(request: &SearchResultsRequest) -> AppServiceResult<()> {
    match &request.criteria {
        SearchCriteria::SimilarRecords {
            similarity: Some(options),
            ..
        } => require_parts_generation(options, request.expected_artifact_generation.as_deref()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::{SimilarityCandidateParts, SimilarityPartSelector};

    #[test]
    fn baseline_and_content_search_project_the_seed_from_one_summary_read() {
        let (retrieval, _artifact) = atlas_search::test_support::open_source_fixture(
            vec![atlas_search::test_support::record(
                "items",
                "Item",
                serde_json::json!({
                    "_id":"0000000000000000","name":"Ember","type":"effect",
                    "system":{"description":{"value":"<p>A glowing ember.</p>"}}
                }),
            )],
            true,
        )
        .unwrap();
        let key = "items:0000000000000000";
        let seed = SimilaritySeed::Record {
            record_key: key.parse().unwrap(),
        };
        for options in [SimilarityOptions::Baseline {}, SimilarityOptions::default()] {
            retrieval.reset_read_metrics();
            let (mode, results, _) = configured_results(
                &retrieval,
                atlas_search::ConfigurableSimilarRequest {
                    include_evidence: true,
                    seed: &seed,
                    options: &options,
                    source_sections: None,
                    content_scope: atlas_domain::SearchContentScope::All,
                    search_profile: None,
                    filter: None,
                    scope: atlas_search::RecordScope::All,
                    page: Default::default(),
                    sort: atlas_domain::ResultSort::Relevance {},
                    cancelled: None,
                },
                &ReadExecutionContext::uncancellable(),
            )
            .unwrap();
            let atlas_app_model::SearchModeSummary::SimilarRecords { seed } = mode else {
                panic!("expected Similar results");
            };
            assert_eq!(seed.record_key, key);
            assert_eq!(seed.title, "Ember");
            assert!(results.rows.is_empty());
            assert_eq!(retrieval.read_metrics().summary_batches, 1, "{options:?}");
        }
    }

    #[test]
    fn configured_similarity_observes_abandonment_before_presentation_reads() {
        let (retrieval, _artifact) = atlas_search::test_support::open_source_fixture(
            ["0000000000000000", "1111111111111111"]
                .into_iter()
                .map(|id| atlas_search::test_support::record("items", "Item", serde_json::json!({
                    "_id":id,"name":id,"type":"effect","system":{"description":{"value":"<p>Shared prose</p>"}}
                })))
                .collect(),
            true,
        ).unwrap();
        let execution = ReadExecutionContext::tracked();
        // Let retrieval complete while the app caller abandons its response.
        let cancelled = || {
            execution.abandon();
            false
        };
        let seed = SimilaritySeed::Record {
            record_key: "items:0000000000000000".parse().unwrap(),
        };
        let options = SimilarityOptions::default();
        let error = configured_results(
            &retrieval,
            atlas_search::ConfigurableSimilarRequest {
                include_evidence: true,
                seed: &seed,
                options: &options,
                source_sections: None,
                content_scope: atlas_domain::SearchContentScope::All,
                search_profile: None,
                filter: None,
                scope: atlas_search::RecordScope::All,
                page: Default::default(),
                sort: atlas_domain::ResultSort::Relevance {},

                cancelled: Some(&cancelled),
            },
            &execution,
        )
        .unwrap_err()
        .into_app_error();
        assert_eq!(
            error.code,
            atlas_app_model::AppErrorCode::OperationCancelled
        );
    }

    #[test]
    fn explicit_part_selection_requires_a_generation_guard() {
        let options = SimilarityOptions::Passages {
            decay: 0.5,
            parts: Some(vec![SimilarityPartSelector("part-selector".into())]),
            candidate_parts: SimilarityCandidateParts::Any,
        };
        assert_eq!(
            validate_options(&options, None)
                .unwrap_err()
                .into_app_error()
                .code,
            atlas_app_model::AppErrorCode::InvalidRequest
        );
        assert!(validate_options(&options, Some("generation")).is_ok());
    }
}
