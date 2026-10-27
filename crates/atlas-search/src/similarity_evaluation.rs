//! Maintained exact scoring over an explicitly declared candidate universe.
//!
//! Unlike RecordScope::Keys on a product search, this path performs no KNN and
//! visits every eligible key. Candidate vectors are streamed in bounded batches;
//! passage maxima are reused across decay variants, never across role masks.
use crate::{AtlasRetrievalService, SearchError, SimilarityPartScore, SimilarityRoleScore};
use atlas_domain::{
    QueryPredicate, RecordKey, SearchProfileId, SimilarityCandidateParts, SimilarityOptions,
    SimilaritySeed,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimilarityEvaluationVariant {
    pub id: String,
    pub options: SimilarityOptions,
}
pub struct SimilarityEvaluationRequest<'a> {
    pub seed: &'a SimilaritySeed,
    pub profile: &'a SearchProfileId,
    pub variants: &'a [SimilarityEvaluationVariant],
    pub candidate_keys: &'a [RecordKey],
    pub filter: Option<&'a QueryPredicate>,
    pub cancelled: Option<&'a dyn Fn() -> bool>,
}
#[derive(Debug, Clone, Serialize)]
pub struct SimilarityEvaluationEvidence {
    pub seed: Option<crate::SourceUnitLocation>,
    pub candidate: crate::SourceUnitLocation,
    pub cosine: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct SimilarityEvaluationRow {
    pub record_key: RecordKey,
    /// None denotes an explicitly reported unavailable comparison, not a zero.
    pub score: Option<f64>,
    pub disposition: String,
    pub comparable_parts: usize,
    pub undefined_candidate_centroids: usize,
    pub parts: Vec<SimilarityPartScore>,
    pub roles: Vec<SimilarityRoleScore>,
    pub evidence: Vec<SimilarityEvaluationEvidence>,
}
#[derive(Debug, Clone, Serialize)]
pub struct SimilarityEvaluationRun {
    pub id: String,
    pub options: SimilarityOptions,
    pub recipe: String,
    pub rows: Vec<SimilarityEvaluationRow>,
    /// Includes only aggregation for reused passage maxima; preparation costs
    /// are reported separately and must not be called product request latency.
    pub scoring_ms: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct SimilarityEvaluationResult {
    pub artifact_generation: String,
    pub search_profile: SearchProfileId,
    pub requested_candidates: usize,
    pub eligible_candidates: usize,
    pub candidate_vectors: usize,
    /// Identity participation in the content-method candidate reads. Baseline
    /// reads remain separately reported and always retain their native identity.
    pub content_candidate_identity: bool,
    pub core_recipe: Option<String>,
    pub baseline_vectors: usize,
    pub pairwise_comparisons: usize,
    pub seed_preparation_ms: f64,
    pub vector_read_ms: f64,
    /// SQLite computes the legacy cosine/max while reading; this is deliberately
    /// separate from content-vector I/O and from Rust aggregation timings.
    pub baseline_read_and_scoring_ms: f64,
    pub shared_maxima_ms: f64,
    pub total_ms: f64,
    pub runs: Vec<SimilarityEvaluationRun>,
}
impl SimilarityEvaluationRequest<'_> {
    fn check(&self) -> Result<(), SearchError> {
        if self.cancelled.is_some_and(|cancelled| cancelled()) {
            return Err(SearchError::operation_cancelled());
        }
        Ok(())
    }
}
fn unavailable(key: &RecordKey, reason: &str) -> SimilarityEvaluationRow {
    SimilarityEvaluationRow {
        record_key: key.clone(),
        score: None,
        disposition: reason.into(),
        comparable_parts: 0,
        undefined_candidate_centroids: 0,
        parts: vec![],
        roles: vec![],
        evidence: vec![],
    }
}
impl AtlasRetrievalService {
    pub fn evaluate_similarity_candidates(
        &self,
        request: SimilarityEvaluationRequest<'_>,
    ) -> Result<SimilarityEvaluationResult, SearchError> {
        let started = Instant::now();
        request.check()?;
        self.resolve_search_profile(Some(request.profile), true)?;
        if request.variants.is_empty()
            || request.variants.len() > 64
            || request.candidate_keys.len() > 32768
            || request
                .variants
                .len()
                .saturating_mul(request.candidate_keys.len())
                > 100000
        {
            return Err(SearchError::invalid_search_options(
                "evaluation supports 1..64 variants, at most 32768 explicit candidates, and at most 100000 matrix rows",
            ));
        }
        let unique = request.candidate_keys.iter().collect::<BTreeSet<_>>();
        if unique.len() != request.candidate_keys.len() {
            return Err(SearchError::invalid_search_options(
                "evaluation candidate keys must be unique",
            ));
        }
        let mut ids = BTreeSet::new();
        for variant in request.variants {
            if variant.id.trim().is_empty() || !ids.insert(&variant.id) {
                return Err(SearchError::invalid_search_options(
                    "evaluation variant identifiers must be nonempty and unique",
                ));
            }
            variant
                .options
                .validate(request.seed)
                .map_err(SearchError::invalid_search_options)?;
        }
        let content = request
            .variants
            .iter()
            .find(|v| !matches!(v.options, SimilarityOptions::Baseline {}));
        if request
            .variants
            .iter()
            .filter(|v| !matches!(v.options, SimilarityOptions::Baseline {}))
            .any(|v| v.options.parts() != content.and_then(|v| v.options.parts()))
        {
            return Err(SearchError::invalid_search_options(
                "one evaluation matrix must use one exact seed selection; use separate scenarios for different selections",
            ));
        }
        let seed_units = if let Some(variant) = content {
            let inventory = self.similarity_parts(request.seed, Some(request.profile))?;
            let selected = super::configurable_similarity::selected_components(
                &inventory,
                &variant.options,
                super::configurable_similarity::resolve_source_sections(&variant.options, None)?,
            )?;
            let selectors = selected.iter().cloned().collect::<Vec<_>>();
            let mut units = match request.seed {
                SimilaritySeed::Record { record_key } => self.index.selected_similarity_units(
                    request.profile,
                    std::slice::from_ref(record_key),
                    Some(&selectors),
                    atlas_domain::SearchContentScope::All,
                    inventory.parts.iter().any(|component| {
                        component.is_identity() && selected.contains(component.selector())
                    }),
                    128,
                )?,
                SimilaritySeed::Trait { trait_key } => {
                    self.index
                        .trait_similarity_units(request.profile, trait_key, 128)?
                }
            };
            units.retain(|unit| selected.contains(unit.component.selector()));
            if units.is_empty()
                || selected.iter().any(|selector| {
                    !units
                        .iter()
                        .any(|unit| unit.component.selector() == selector)
                })
            {
                return Err(SearchError::vector_readiness_required(
                    "selected evaluation content has no stored vectors",
                ));
            }
            units
        } else {
            vec![]
        };
        let candidate_identity = seed_units.iter().any(|unit| unit.component.is_identity());
        let seed_parts = super::similarity_scoring::parts(&seed_units)?;
        let mut average_seed_defined = BTreeMap::new();
        for variant in request.variants {
            if let SimilarityOptions::ContentAverage {
                candidate_parts: mode,
                ..
            } = variant.options
            {
                let mask = mode == SimilarityCandidateParts::Corresponding;
                if let std::collections::btree_map::Entry::Vacant(entry) =
                    average_seed_defined.entry(mask)
                {
                    entry.insert(
                        super::similarity_scoring::content_average(
                            &seed_parts,
                            &seed_parts,
                            mode,
                            request.cancelled,
                        )?
                        .is_some(),
                    );
                }
            }
        }
        let baseline_vector = if request
            .variants
            .iter()
            .any(|v| matches!(v.options, SimilarityOptions::Baseline {}))
        {
            Some(
                match request.seed {
                    SimilaritySeed::Record { record_key } => self
                        .index
                        .seed_identity_vector(request.profile, record_key)?
                        .map(|value| value.vector),
                    SimilaritySeed::Trait { trait_key } => {
                        self.index.trait_vector(trait_key, request.profile)?
                    }
                }
                .ok_or_else(|| {
                    SearchError::vector_readiness_required(
                        "baseline evaluation seed has no stored legacy vector",
                    )
                })?,
            )
        } else {
            None
        };
        let predicate = self.predicate(request.filter)?;
        let mut eligible = BTreeSet::new();
        for keys in request.candidate_keys.chunks(256) {
            request.check()?;
            let page = self.index.eligible_keys(
                &predicate,
                Some(keys),
                false,
                atlas_domain::ResultSort::attribute(
                    atlas_domain::SortField::Name,
                    atlas_domain::SortDirection::Asc,
                ),
                0,
                1024,
            )?;
            eligible.extend(page.keys.into_iter().filter(|key| !matches!(request.seed, SimilaritySeed::Record { record_key } if record_key == key)));
        }
        let mut result = SimilarityEvaluationResult {
            artifact_generation: self.artifact_generation()?,
            search_profile: request.profile.clone(),
            requested_candidates: request.candidate_keys.len(),
            eligible_candidates: eligible.len(),
            candidate_vectors: 0,
            content_candidate_identity: candidate_identity,
            core_recipe: candidate_identity.then(|| atlas_domain::CORE_SECTIONS_RECIPE.into()),
            baseline_vectors: 0,
            pairwise_comparisons: 0,
            seed_preparation_ms: started.elapsed().as_secs_f64() * 1000.0,
            vector_read_ms: 0.0,
            baseline_read_and_scoring_ms: 0.0,
            shared_maxima_ms: 0.0,
            total_ms: 0.0,
            runs: request
                .variants
                .iter()
                .map(|v| SimilarityEvaluationRun {
                    id: v.id.clone(),
                    options: v.options.clone(),
                    recipe: if matches!(v.options, SimilarityOptions::Baseline {}) {
                        "current-baseline/v1".into()
                    } else if candidate_identity {
                        atlas_domain::SIMILARITY_COMPONENT_RECIPE.into()
                    } else {
                        atlas_domain::SIMILARITY_RECIPE.into()
                    },
                    rows: vec![],
                    scoring_ms: 0.0,
                })
                .collect(),
        };
        for keys in request.candidate_keys.chunks(128) {
            request.check()?;
            let eligible_keys = keys
                .iter()
                .filter(|key| eligible.contains(*key))
                .cloned()
                .collect::<Vec<_>>();
            let read_started = Instant::now();
            let baseline = if let Some(vector) = &baseline_vector {
                self.index
                    .baseline_similarity_scores(
                        request.profile,
                        &eligible_keys,
                        vector,
                        atlas_domain::SearchContentScope::All,
                    )?
                    .into_iter()
                    .map(|(key, score, count)| {
                        result.baseline_vectors += count;
                        (key, score)
                    })
                    .collect::<BTreeMap<_, _>>()
            } else {
                BTreeMap::new()
            };
            if baseline_vector.is_some() {
                result.baseline_read_and_scoring_ms +=
                    read_started.elapsed().as_secs_f64() * 1000.0;
            }
            let read_started = Instant::now();
            let units = if content.is_some() {
                self.index.similarity_units(
                    request.profile,
                    &eligible_keys,
                    atlas_domain::SearchContentScope::All,
                    candidate_identity,
                    100000,
                )?
            } else {
                vec![]
            };
            result.candidate_vectors += units.len();
            result.vector_read_ms += read_started.elapsed().as_secs_f64() * 1000.0;
            if result.candidate_vectors > 1_000_000 || result.baseline_vectors > 1_000_000 {
                return Err(SearchError::invalid_search_options(
                    "evaluation exceeds one million stored vectors; reduce the declared corpus",
                ));
            }
            let mut grouped = BTreeMap::<RecordKey, Vec<_>>::new();
            for unit in units {
                if let Some(location) = &unit.location {
                    grouped
                        .entry(location.record.clone())
                        .or_default()
                        .push(unit);
                }
            }
            for key in keys {
                request.check()?;
                let candidate_units = grouped.remove(key).unwrap_or_default();
                let candidate_parts = super::similarity_scoring::parts(&candidate_units)?;
                let mut maxima = BTreeMap::new();
                for (variant, run) in request.variants.iter().zip(&mut result.runs) {
                    if !eligible.contains(key) {
                        run.rows.push(unavailable(key, "ineligible_or_seed"));
                        continue;
                    }
                    let score_started = Instant::now();
                    let maxima_before = result.shared_maxima_ms;
                    let row = match &variant.options {
                        SimilarityOptions::Baseline {} => match baseline.get(key) {
                            Some(score) => SimilarityEvaluationRow {
                                record_key: key.clone(),
                                score: Some(*score),
                                disposition: "scored".into(),
                                comparable_parts: 1,
                                undefined_candidate_centroids: 0,
                                parts: vec![],
                                roles: vec![],
                                evidence: vec![],
                            },
                            None => unavailable(key, "no_stored_vectors"),
                        },
                        _ if candidate_units.is_empty() => {
                            unavailable(key, "no_selected_content_vectors")
                        }
                        SimilarityOptions::Passages {
                            decay,
                            candidate_parts: mode,
                            ..
                        } => {
                            let mask = *mode == SimilarityCandidateParts::Corresponding;
                            if let std::collections::btree_map::Entry::Vacant(entry) =
                                maxima.entry(mask)
                            {
                                let maxima_started = Instant::now();
                                let score = super::similarity_scoring::passage_maxima(
                                    &seed_parts,
                                    &candidate_parts,
                                    *mode,
                                    request.cancelled,
                                )?;
                                result.pairwise_comparisons += score.comparisons;
                                result.shared_maxima_ms +=
                                    maxima_started.elapsed().as_secs_f64() * 1000.0;
                                entry.insert(score);
                            }
                            let measured = &maxima[&mask];
                            let weights = super::similarity_scoring::decay_weights(
                                measured.parts.len(),
                                *decay,
                            );
                            let parts = measured
                                .parts
                                .iter()
                                .zip(weights)
                                .map(|(part, weight)| SimilarityPartScore {
                                    part: part.part.clone(),
                                    score: part.score,
                                    weight,
                                    comparable: part.comparable,
                                })
                                .collect::<Vec<_>>();
                            let score = parts.iter().map(|part| part.score * part.weight).sum();
                            let selected_pairs = super::similarity_scoring::select_passage_pairs(
                                &measured.pairs,
                                |pair| {
                                    parts
                                        .iter()
                                        .find(|part| {
                                            part.part.selector() == pair.seed.component.selector()
                                        })
                                        .map_or(0.0, |part| part.score * part.weight)
                                },
                                3,
                            );
                            SimilarityEvaluationRow {
                                record_key: key.clone(),
                                score: Some(score),
                                disposition: if measured.comparable_parts == 0 {
                                    "no_corresponding_content"
                                } else if score <= 0.0 {
                                    "no_positive_evidence"
                                } else {
                                    "scored"
                                }
                                .into(),
                                comparable_parts: measured.comparable_parts,
                                undefined_candidate_centroids: measured.undefined_centroids,
                                evidence: selected_pairs
                                    .iter()
                                    .filter_map(|pair| {
                                        Some(SimilarityEvaluationEvidence {
                                            seed: pair.seed.location.clone(),
                                            candidate: pair.candidate.location.clone()?,
                                            cosine: pair.cosine,
                                        })
                                    })
                                    .collect(),
                                parts,
                                roles: vec![],
                            }
                        }
                        SimilarityOptions::ContentAverage {
                            candidate_parts: mode,
                            ..
                        } if !average_seed_defined
                            [&(*mode == SimilarityCandidateParts::Corresponding)] =>
                        {
                            unavailable(key, "undefined_seed_centroid")
                        }
                        SimilarityOptions::ContentAverage {
                            candidate_parts: mode,
                            ..
                        } => {
                            match super::similarity_scoring::content_average(
                                &seed_parts,
                                &candidate_parts,
                                *mode,
                                request.cancelled,
                            )? {
                                None => unavailable(key, "undefined_candidate_centroid"),
                                Some(measured) => {
                                    result.pairwise_comparisons += measured.comparisons;
                                    SimilarityEvaluationRow {
                                        record_key: key.clone(),
                                        score: Some(measured.score),
                                        disposition: if measured.comparable_parts == 0 {
                                            "no_corresponding_content"
                                        } else if measured.score <= 0.0 {
                                            "no_positive_evidence"
                                        } else {
                                            "scored"
                                        }
                                        .into(),
                                        comparable_parts: measured.comparable_parts,
                                        undefined_candidate_centroids: measured.undefined_centroids,
                                        evidence: measured
                                            .pairs
                                            .iter()
                                            .filter_map(|pair| {
                                                Some(SimilarityEvaluationEvidence {
                                                    seed: pair.seed.location.clone(),
                                                    candidate: pair.candidate.location.clone()?,
                                                    cosine: pair.cosine,
                                                })
                                            })
                                            .collect(),
                                        parts: vec![],
                                        roles: measured
                                            .roles
                                            .into_iter()
                                            .map(|role| SimilarityRoleScore {
                                                key: role.key,
                                                score: role.score,
                                                weight: role.weight,
                                                comparable: role.comparable,
                                            })
                                            .collect(),
                                    }
                                }
                            }
                        }
                    };
                    run.scoring_ms += (score_started.elapsed().as_secs_f64() * 1000.0
                        - (result.shared_maxima_ms - maxima_before))
                        .max(0.0);
                    run.rows.push(row);
                }
                if result.pairwise_comparisons > 64_000_000 {
                    return Err(SearchError::invalid_search_options(
                        "evaluation exceeds 64 million vector comparisons; reduce the declared corpus",
                    ));
                }
            }
            for run in &mut result.runs {
                retain_top_evidence(run);
            }
        }
        for run in &mut result.runs {
            retain_top_evidence(run);
            run.rows.sort_by(|a, b| {
                b.score
                    .unwrap_or(f64::NEG_INFINITY)
                    .total_cmp(&a.score.unwrap_or(f64::NEG_INFINITY))
                    .then(a.record_key.cmp(&b.record_key))
            });
        }
        result.total_ms = started.elapsed().as_secs_f64() * 1000.0;
        Ok(result)
    }
}

// Non-top rows remain in the exact score census, but bounded source evidence
// is kept only for the ten highest rows to bound evaluation report memory.
fn retain_top_evidence(run: &mut SimilarityEvaluationRun) {
    let mut indices = run
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            !row.parts.is_empty() || !row.roles.is_empty() || !row.evidence.is_empty()
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if indices.len() <= 10 {
        return;
    }
    indices.sort_by(|a, b| {
        run.rows[*b]
            .score
            .unwrap_or(f64::NEG_INFINITY)
            .total_cmp(&run.rows[*a].score.unwrap_or(f64::NEG_INFINITY))
            .then(run.rows[*a].record_key.cmp(&run.rows[*b].record_key))
    });
    for index in indices.into_iter().skip(10) {
        run.rows[index].parts.clear();
        run.rows[index].roles.clear();
        run.rows[index].evidence.clear();
    }
}
impl AtlasRetrievalService {
    /// Apply the same matching-set remaster policy to an exact score matrix.
    pub fn evaluation_ranking(
        &self,
        run: &SimilarityEvaluationRun,
        prefer_remaster: bool,
    ) -> Result<Vec<RecordKey>, SearchError> {
        let roots = run
            .rows
            .iter()
            .filter(|row| row.disposition == "scored")
            .filter_map(|row| {
                Some(crate::text::RankedRoot {
                    key: row.record_key.clone(),
                    lexical_rank: None,
                    similarity: row.score,
                    score: row.score?,
                    tier: 3,
                    witnesses: vec![],
                })
            })
            .collect();
        Ok(self
            .suppress_pairs(roots, prefer_remaster)?
            .into_iter()
            .map(|row| row.key)
            .collect())
    }
}

#[cfg(test)]
mod tests;
