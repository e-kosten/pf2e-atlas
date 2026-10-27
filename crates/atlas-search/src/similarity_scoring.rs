//! Pure hierarchical aggregation over already selected, normalized vectors.
use crate::SearchError;
use atlas_domain::{
    SimilarityCandidateParts, SimilarityComparisonKey, SimilarityComponent, SourceByteRange,
    SourcePassageAddress,
};
use atlas_index::SourceSimilarityUnit;
use std::collections::BTreeMap;

pub(crate) struct Part<'a> {
    pub metadata: SimilarityComponent,
    pub units: Vec<&'a SourceSimilarityUnit>,
    pub weights: Vec<f64>,
}
pub(crate) struct PartScore {
    pub part: SimilarityComponent,
    pub score: f64,
    pub weight: f64,
    pub comparable: bool,
}
pub(crate) struct RoleScore {
    pub key: SimilarityComparisonKey,
    pub score: f64,
    pub weight: f64,
    pub comparable: bool,
}
#[derive(Clone, Copy)]
pub(crate) struct Pair<'a> {
    pub seed: &'a SourceSimilarityUnit,
    pub candidate: &'a SourceSimilarityUnit,
    pub cosine: f64,
}
pub(crate) struct Score<'a> {
    pub score: f64,
    pub aggregation: &'static str,
    pub parts: Vec<PartScore>,
    pub roles: Vec<RoleScore>,
    pub pairs: Vec<Pair<'a>>,
    pub comparable_parts: usize,
    pub representative: bool,
    pub comparisons: usize,
    pub undefined_centroids: usize,
}

/// At most 64 vector operations run between cooperative cancellation checks.
struct ScoringControl<'a> {
    cancelled: Option<&'a dyn Fn() -> bool>,
    operations: usize,
}
impl<'a> ScoringControl<'a> {
    fn new(cancelled: Option<&'a dyn Fn() -> bool>) -> Result<Self, SearchError> {
        let mut control = Self {
            cancelled,
            operations: 0,
        };
        control.check()?;
        Ok(control)
    }
    fn check(&mut self) -> Result<(), SearchError> {
        if self.cancelled.is_some_and(|cancelled| cancelled()) {
            return Err(SearchError::operation_cancelled());
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), SearchError> {
        self.operations += 1;
        if self.operations.is_multiple_of(64) {
            self.check()?;
        }
        Ok(())
    }
}

fn byte_range(address: &SourcePassageAddress) -> Option<SourceByteRange> {
    match address {
        SourcePassageAddress::HtmlSection { chunk_bytes, .. }
        | SourcePassageAddress::PlainSection { chunk_bytes, .. } => Some(*chunk_bytes),
        SourcePassageAddress::Identity {} => None,
    }
}

pub(crate) fn parts(units: &[SourceSimilarityUnit]) -> Result<Vec<Part<'_>>, SearchError> {
    if units
        .iter()
        .filter(|unit| unit.component.is_identity())
        .count()
        > 1
    {
        return Err(SearchError::artifact_contract_violation(
            "root identity must have exactly one stored vector",
        ));
    }
    if units.iter().any(|unit| {
        unit.component.is_identity() != matches!(unit.address, SourcePassageAddress::Identity {})
    }) {
        return Err(SearchError::artifact_contract_violation(
            "comparison component does not match its native source address",
        ));
    }
    let mut grouped = BTreeMap::new();
    for unit in units {
        grouped
            .entry(unit.component.selector().clone())
            .or_insert_with(Vec::new)
            .push(unit);
    }
    Ok(grouped
        .into_values()
        .map(|mut units| {
            units.sort_by_key(|unit| {
                byte_range(&unit.address).map(|range| (range.start, range.end))
            });
            if units[0].component.is_identity() {
                return Part {
                    metadata: units[0].component.clone(),
                    units,
                    weights: vec![1.0],
                };
            }
            let mut covered = 0;
            let mut weights = Vec::with_capacity(units.len());
            for unit in &units {
                let weight = byte_range(&unit.address).map_or(0, |range| {
                    range.end.saturating_sub(covered.max(range.start))
                });
                covered = covered.max(byte_range(&unit.address).map_or(0, |range| range.end));
                weights.push(weight as f64);
            }
            let total = weights.iter().sum::<f64>();
            if total > 0.0 {
                for weight in &mut weights {
                    *weight /= total;
                }
            }
            Part {
                metadata: units[0].component.clone(),
                units,
                weights,
            }
        })
        .collect())
}

pub(crate) fn centroid<'a>(
    vectors: impl IntoIterator<Item = (&'a [f32], f64)>,
) -> Option<Vec<f32>> {
    let mut sum = Vec::<f64>::new();
    for (vector, weight) in vectors {
        if sum.is_empty() {
            sum.resize(vector.len(), 0.0);
        }
        if vector.len() != sum.len() || !weight.is_finite() || weight < 0.0 {
            return None;
        }
        for (sum, value) in sum.iter_mut().zip(vector) {
            *sum += f64::from(*value) * weight;
        }
    }
    let norm = sum.iter().map(|value| value * value).sum::<f64>().sqrt();
    if !norm.is_finite() || norm < 1e-12 {
        return None;
    }
    Some(sum.into_iter().map(|value| (value / norm) as f32).collect())
}
pub(crate) fn part_centroid(part: &Part<'_>) -> Option<Vec<f32>> {
    centroid(
        part.units
            .iter()
            .zip(&part.weights)
            .map(|(unit, weight)| (unit.vector.as_slice(), *weight)),
    )
}
pub(crate) fn parts_centroid(parts: &[Part<'_>]) -> Option<Vec<f32>> {
    let vectors = parts
        .iter()
        .map(part_centroid)
        .collect::<Option<Vec<_>>>()?;
    centroid(vectors.iter().map(|vector| (vector.as_slice(), 1.0)))
}
fn cosine(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(a, b)| f64::from(*a) * f64::from(*b))
        .sum::<f64>()
        .clamp(-1.0, 1.0)
}
fn allowed(
    seed: SimilarityComparisonKey,
    candidate: SimilarityComparisonKey,
    mode: SimilarityCandidateParts,
) -> bool {
    mode == SimilarityCandidateParts::Any || seed.corresponds_to(candidate)
}

/// Shared normalized geometric weights, also used to evaluate decay variants
/// over already measured part maxima without repeating vector comparisons.
pub(crate) fn decay_weights(count: usize, decay: f64) -> Vec<f64> {
    let mut value = 1.0;
    let mut weights = Vec::with_capacity(count);
    for _ in 0..count {
        weights.push(value);
        value *= decay;
    }
    let total: f64 = weights.iter().sum();
    for weight in &mut weights {
        *weight /= total;
    }
    weights
}

pub(crate) fn passages<'a>(
    seed: &[Part<'a>],
    candidate: &[Part<'a>],
    mode: SimilarityCandidateParts,
    decay: f64,
    cancelled: Option<&dyn Fn() -> bool>,
) -> Result<Score<'a>, SearchError> {
    passages_with_limit(seed, candidate, mode, decay, 3, cancelled)
}

/// Evaluation reuses every part's best pair before choosing the displayed
/// contributors for each decay. A capped explanation is not a reusable maximum.
pub(crate) fn passage_maxima<'a>(
    seed: &[Part<'a>],
    candidate: &[Part<'a>],
    mode: SimilarityCandidateParts,
    cancelled: Option<&dyn Fn() -> bool>,
) -> Result<Score<'a>, SearchError> {
    passages_with_limit(seed, candidate, mode, 1.0, usize::MAX, cancelled)
}

pub(crate) fn select_passage_pairs<'a>(
    pairs: &[Pair<'a>],
    contribution: impl Fn(&Pair<'a>) -> f64,
    limit: usize,
) -> Vec<Pair<'a>> {
    let mut pairs = pairs
        .iter()
        .copied()
        .filter(|pair| contribution(pair) > 0.0)
        .collect::<Vec<_>>();
    pairs.sort_by(|a, b| {
        contribution(b)
            .total_cmp(&contribution(a))
            .then(b.cosine.total_cmp(&a.cosine))
    });
    pairs.truncate(limit);
    pairs
}

fn passages_with_limit<'a>(
    seed: &[Part<'a>],
    candidate: &[Part<'a>],
    mode: SimilarityCandidateParts,
    decay: f64,
    evidence_limit: usize,
    cancelled: Option<&dyn Fn() -> bool>,
) -> Result<Score<'a>, SearchError> {
    let mut control = ScoringControl::new(cancelled)?;
    let mut part_scores = Vec::with_capacity(seed.len());
    let mut pairs = Vec::new();
    let mut comparisons = 0;
    for part in seed {
        let eligible = candidate
            .iter()
            .filter(|candidate| {
                allowed(
                    part.metadata.comparison_key(),
                    candidate.metadata.comparison_key(),
                    mode,
                )
            })
            .flat_map(|part| part.units.iter().copied())
            .collect::<Vec<_>>();
        let mut score = 0.0;
        let mut best_pair = None;
        for (unit, weight) in part.units.iter().zip(&part.weights) {
            if *weight == 0.0 {
                continue;
            }
            let mut best: Option<(&SourceSimilarityUnit, f64)> = None;
            for candidate in &eligible {
                control.step()?;
                comparisons += 1;
                let value = cosine(&unit.vector, &candidate.vector);
                if best.as_ref().is_none_or(|(_, previous)| value >= *previous) {
                    best = Some((*candidate, value));
                }
            }
            if let Some((candidate, value)) = best {
                score += value.max(0.0) * weight;
                if best_pair
                    .as_ref()
                    .is_none_or(|pair: &Pair<'_>| value > pair.cosine)
                {
                    best_pair = Some(Pair {
                        seed: unit,
                        candidate,
                        cosine: value,
                    });
                }
            }
        }
        if let Some(pair) = best_pair.filter(|pair| pair.cosine > 0.0) {
            pairs.push(pair);
        }
        part_scores.push(PartScore {
            part: part.metadata.clone(),
            score,
            weight: 0.0,
            comparable: !eligible.is_empty(),
        });
    }
    part_scores.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then(a.part.selector().cmp(b.part.selector()))
    });
    for (part, weight) in part_scores.iter_mut().zip(decay_weights(seed.len(), decay)) {
        part.weight = weight;
    }
    let score = part_scores
        .iter()
        .map(|part| part.score * part.weight)
        .sum();
    let comparable_parts = part_scores.iter().filter(|part| part.comparable).count();
    let contribution = |pair: &Pair<'_>| {
        part_scores
            .iter()
            .find(|part| part.part.selector() == pair.seed.component.selector())
            .map_or(0.0, |part| part.score * part.weight)
    };
    let pairs = select_passage_pairs(&pairs, contribution, evidence_limit);
    Ok(Score {
        score,
        aggregation: "part_maxima_rank_decay",
        parts: part_scores,
        roles: vec![],
        pairs,
        comparable_parts,
        representative: false,
        comparisons,
        undefined_centroids: 0,
    })
}

fn representative_pair<'a>(
    seed: Vec<&'a SourceSimilarityUnit>,
    candidate: Vec<&'a SourceSimilarityUnit>,
    seed_vector: &[f32],
    candidate_vector: &[f32],
    control: &mut ScoringControl<'_>,
) -> Result<(Option<Pair<'a>>, usize), SearchError> {
    fn best<'a>(
        units: Vec<&'a SourceSimilarityUnit>,
        vector: &[f32],
        control: &mut ScoringControl<'_>,
    ) -> Result<Option<&'a SourceSimilarityUnit>, SearchError> {
        let mut best = None;
        let mut value = f64::NEG_INFINITY;
        for unit in units {
            control.step()?;
            let cosine = cosine(&unit.vector, vector);
            if cosine >= value {
                value = cosine;
                best = Some(unit);
            }
        }
        Ok(best)
    }
    let comparisons = seed.len() + candidate.len();
    let seed = best(seed, candidate_vector, control)?;
    let candidate = best(candidate, seed_vector, control)?;
    control.step()?;
    let pair = seed.zip(candidate).map(|(seed, candidate)| Pair {
        seed,
        candidate,
        cosine: cosine(&seed.vector, &candidate.vector),
    });
    Ok((pair, comparisons + 1))
}

fn checked_centroid<'a>(
    vectors: impl IntoIterator<Item = (&'a [f32], f64)>,
    control: &mut ScoringControl<'_>,
) -> Result<Option<Vec<f32>>, SearchError> {
    // Stop the iterator immediately on cancellation, preserving normal summation
    // order and weights and avoiding a second copy of the centroid arithmetic.
    let mut cancelled = false;
    let vector = centroid(vectors.into_iter().map_while(|entry| {
        if control.step().is_err() {
            cancelled = true;
            None
        } else {
            Some(entry)
        }
    }));
    if cancelled {
        return Err(SearchError::operation_cancelled());
    }
    Ok(vector)
}
fn checked_part_centroid(
    part: &Part<'_>,
    control: &mut ScoringControl<'_>,
) -> Result<Option<Vec<f32>>, SearchError> {
    checked_centroid(
        part.units
            .iter()
            .zip(&part.weights)
            .map(|(unit, weight)| (unit.vector.as_slice(), *weight)),
        control,
    )
}
fn checked_centroids(
    parts: &[Part<'_>],
    control: &mut ScoringControl<'_>,
) -> Result<Option<Vec<f32>>, SearchError> {
    let mut vectors = Vec::with_capacity(parts.len());
    for part in parts {
        let Some(vector) = checked_part_centroid(part, control)? else {
            return Ok(None);
        };
        vectors.push(vector);
    }
    checked_centroid(
        vectors.iter().map(|vector| (vector.as_slice(), 1.0)),
        control,
    )
}

pub(crate) fn content_average<'a>(
    seed: &[Part<'a>],
    candidate: &[Part<'a>],
    mode: SimilarityCandidateParts,
    cancelled: Option<&dyn Fn() -> bool>,
) -> Result<Option<Score<'a>>, SearchError> {
    let mut control = ScoringControl::new(cancelled)?;
    let mut evidence = Score {
        score: 0.0,
        aggregation: "global_centroids",
        parts: vec![],
        roles: vec![],
        pairs: vec![],
        comparable_parts: 0,
        representative: true,
        comparisons: 0,
        undefined_centroids: 0,
    };
    if mode == SimilarityCandidateParts::Any {
        let Some(seed_vector) = checked_centroids(seed, &mut control)? else {
            return Ok(None);
        };
        let Some(candidate_vector) = checked_centroids(candidate, &mut control)? else {
            return Ok(None);
        };
        evidence.score = cosine(&seed_vector, &candidate_vector).max(0.0);
        evidence.comparisons = 1;
        evidence.comparable_parts = seed.len();
        if evidence.score > 0.0 {
            let (pair, comparisons) = representative_pair(
                seed.iter()
                    .flat_map(|part| part.units.iter().copied())
                    .collect(),
                candidate
                    .iter()
                    .flat_map(|part| part.units.iter().copied())
                    .collect(),
                &seed_vector,
                &candidate_vector,
                &mut control,
            )?;
            evidence.comparisons += comparisons;
            evidence.pairs.extend(pair);
        }
    } else {
        let mut roles = BTreeMap::<SimilarityComparisonKey, Vec<&Part<'_>>>::new();
        for part in seed {
            roles
                .entry(part.metadata.comparison_key())
                .or_default()
                .push(part);
        }
        let mut representative_pairs = Vec::new();
        for (key, parts) in roles {
            let mut seed_vectors = Vec::with_capacity(parts.len());
            for part in &parts {
                let Some(vector) = checked_part_centroid(part, &mut control)? else {
                    return Ok(None);
                };
                seed_vectors.push(vector);
            }
            let Some(seed_vector) = checked_centroid(
                seed_vectors.iter().map(|vector| (vector.as_slice(), 1.0)),
                &mut control,
            )?
            else {
                return Ok(None);
            };
            let eligible = candidate
                .iter()
                .filter(|candidate| key.corresponds_to(candidate.metadata.comparison_key()))
                .collect::<Vec<_>>();
            let candidate_vectors = eligible
                .iter()
                .map(|part| checked_part_centroid(part, &mut control))
                .collect::<Result<Option<Vec<_>>, _>>()?;
            let candidate_vector = match candidate_vectors.as_ref() {
                Some(vectors) => checked_centroid(
                    vectors.iter().map(|vector| (vector.as_slice(), 1.0)),
                    &mut control,
                )?,
                None => None,
            };
            if !eligible.is_empty() {
                evidence.comparable_parts += parts.len();
                if candidate_vector.is_none() {
                    evidence.undefined_centroids += 1;
                }
            }
            let score = if let Some(vector) = &candidate_vector {
                evidence.comparisons += 1;
                cosine(&seed_vector, vector).max(0.0)
            } else {
                0.0
            };
            let weight = parts.len() as f64 / seed.len() as f64;
            evidence.score += score * weight;
            evidence.roles.push(RoleScore {
                key,
                score,
                weight,
                comparable: !eligible.is_empty(),
            });
            if score > 0.0
                && let Some(candidate_vector) = candidate_vector
            {
                let (pair, comparisons) = representative_pair(
                    parts
                        .iter()
                        .flat_map(|part| part.units.iter().copied())
                        .collect(),
                    eligible
                        .iter()
                        .flat_map(|part| part.units.iter().copied())
                        .collect(),
                    &seed_vector,
                    &candidate_vector,
                    &mut control,
                )?;
                evidence.comparisons += comparisons;
                if let Some(pair) = pair {
                    representative_pairs.push((score * weight, pair));
                }
            }
        }
        representative_pairs.sort_by(|a, b| b.0.total_cmp(&a.0));
        evidence.pairs = representative_pairs
            .into_iter()
            .take(3)
            .map(|(_, pair)| pair)
            .collect();
        evidence.aggregation = "role_centroids";
    }
    Ok(Some(evidence))
}

#[cfg(test)]
mod tests;
