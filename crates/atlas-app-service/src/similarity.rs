//! Generation-checked inventory for stored-vector content comparison.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{AppRecoverableAction, SimilarityPartsRequest, SimilarityPartsView};
use atlas_domain::{SimilarityOptions, SimilaritySeed};

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
            Ok(SimilarityPartsView {
                supports_corresponding_parts: matches!(
                    inventory.seed,
                    SimilaritySeed::Record { .. }
                ),
                artifact_generation: generation,
                search_profile: profile,
                seed: inventory.seed,
                parts: inventory.parts.into_iter().map(part_view).collect(),
                role_recipe: inventory.role_recipe,
            })
        })
    }
}

pub(crate) fn validate_options(
    seed: &SimilaritySeed,
    options: &SimilarityOptions,
    expected_generation: Option<&str>,
) -> AppServiceResult<()> {
    options
        .validate(seed)
        .map_err(AppServiceError::invalid_request)?;
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
) -> AppServiceResult<(
    atlas_app_model::SearchModeSummary,
    atlas_app_model::SearchResultsView,
    atlas_app_model::SimilarityConfigurationView,
)> {
    let missing_code = if matches!(request.seed, SimilaritySeed::Record { .. }) {
        atlas_app_model::AppErrorCode::RecordNotFound
    } else {
        atlas_app_model::AppErrorCode::InvalidRequest
    };
    let result = retrieval.configurable_similar(request)?.ok_or_else(|| {
        AppServiceError::new(missing_code, "the selected similarity seed is unavailable")
    })?;
    let mode = match &result.seed {
        SimilaritySeed::Record { record_key } => {
            let summaries = retrieval.get_records(atlas_search::GetRecordsRequest {
                record_keys: std::slice::from_ref(record_key),
            })?;
            let summary = summaries.records.first().ok_or_else(|| {
                AppServiceError::invalid_request("the selected similarity record is unavailable")
            })?;
            atlas_app_model::SearchModeSummary::SimilarRecords {
                seed: Box::new(crate::projection::localized_summary_view(
                    summary, retrieval,
                )),
            }
        }
        SimilaritySeed::Trait { trait_key } => {
            let definition = retrieval.trait_definition(trait_key)?.ok_or_else(|| {
                AppServiceError::invalid_request("the selected similarity trait is unavailable")
            })?;
            atlas_app_model::SearchModeSummary::SimilarTraits {
                trait_key: trait_key.to_string(),
                title: definition.label,
            }
        }
    };
    let mut evidence = result
        .evidence
        .into_iter()
        .map(|(key, evidence)| {
            (
                key.to_string(),
                evidence_view(evidence, retrieval.source_fingerprint()),
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
            options: result.effective,
            recipe: result.recipe,
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
                part: part_view(part.part),
                score: part.score,
                weight: part.weight,
                comparable: part.comparable,
            })
            .collect(),
        roles: evidence
            .roles
            .into_iter()
            .map(|role| atlas_app_model::SimilarityRoleScoreView {
                role: role.role,
                role_label: role.role.label().into(),
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
                seed_text: pair.seed_text,
                candidate: crate::projection::search_witness_view(pair.candidate, fingerprint),
                cosine: pair.cosine,
            })
            .collect(),
    }
}

fn part_view(part: atlas_domain::SimilarityPart) -> atlas_app_model::SimilarityPartView {
    atlas_app_model::SimilarityPartView {
        selector: part.selector,
        label: part.label,
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

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::{SimilarityCandidateParts, SimilarityPartSelector};

    #[test]
    fn explicit_part_selection_requires_a_generation_guard() {
        let seed = SimilaritySeed::Record {
            record_key: "items:0000000000000000".parse().unwrap(),
        };
        let options = SimilarityOptions::Passages {
            decay: 0.5,
            parts: Some(vec![SimilarityPartSelector("part-selector".into())]),
            candidate_parts: SimilarityCandidateParts::Any,
        };
        assert_eq!(
            validate_options(&seed, &options, None)
                .unwrap_err()
                .into_app_error()
                .code,
            atlas_app_model::AppErrorCode::InvalidRequest
        );
        assert!(validate_options(&seed, &options, Some("generation")).is_ok());
    }

    #[test]
    fn empty_parts_and_trait_correspondence_remain_invalid() {
        let seed = SimilaritySeed::Trait {
            trait_key: "trait:fire".parse().unwrap(),
        };
        for options in [
            SimilarityOptions::ContentAverage {
                parts: Some(vec![]),
                candidate_parts: SimilarityCandidateParts::Any,
            },
            SimilarityOptions::Passages {
                decay: 0.5,
                parts: None,
                candidate_parts: SimilarityCandidateParts::Corresponding,
            },
        ] {
            assert!(validate_options(&seed, &options, Some("generation")).is_err());
        }
    }
}
