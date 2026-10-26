//! Stored-vector similarity workflows shared by terminal and browser consumers.
use crate::{RecordScope, SearchError, SearchPage, SearchWitness, TextSearchResult};
use atlas_domain::{
    QueryPredicate, RecordKey, ResultSort, SIMILARITY_RECIPE, SearchProfileId,
    SimilarityCandidateParts, SimilarityOptions, SimilarityPart, SimilarityPartSelector,
    SimilarityParts, SimilarityRole, SimilaritySeed,
};
use atlas_index::{SourceSimilarityUnit, SourceUnitLocation};
use std::collections::{BTreeMap, BTreeSet};

const MAX_SEED_UNITS: usize = 128;
const MAX_CANDIDATE_ROOTS: usize = 4096;
const MAX_CANDIDATE_UNITS: usize = 65_536;
const MAX_COMPARISONS: usize = 8_000_000;
const CANDIDATE_WINDOW: usize = 256;
const MAX_EXPLANATION_PARTS: usize = 32;

pub struct ConfigurableSimilarRequest<'a> {
    pub seed: &'a SimilaritySeed,
    pub options: &'a SimilarityOptions,
    pub search_profile: Option<&'a SearchProfileId>,
    pub filter: Option<&'a QueryPredicate>,
    pub scope: RecordScope<'a>,
    pub page: SearchPage,
    pub sort: ResultSort,
    pub prefer_remaster: bool,
    pub cancelled: Option<&'a dyn Fn() -> bool>,
}
#[derive(Debug)]
pub struct ConfigurableSimilarResult {
    pub seed: SimilaritySeed,
    pub results: TextSearchResult,
    pub evidence: Vec<(RecordKey, SimilarityEvidence)>,
    pub effective: SimilarityOptions,
    pub recipe: String,
    pub diagnostics: SimilarityDiagnostics,
}
#[derive(Debug, Clone, Default)]
pub struct SimilarityDiagnostics {
    pub seed_units: usize,
    pub knn_queries: usize,
    pub candidate_vectors: usize,
    pub pairwise_comparisons: usize,
    pub undefined_candidate_centroids: usize,
}
#[derive(Debug, Clone)]
pub struct SimilarityEvidence {
    pub score: f64,
    pub aggregation: String,
    pub selected_parts: usize,
    pub comparable_parts: usize,
    pub parts: Vec<SimilarityPartScore>,
    pub roles: Vec<SimilarityRoleScore>,
    pub pairs: Vec<SimilarityPair>,
    pub representative: bool,
    pub truncated: bool,
}
#[derive(Debug, Clone)]
pub struct SimilarityPartScore {
    pub part: SimilarityPart,
    pub score: f64,
    pub weight: f64,
    pub comparable: bool,
}
#[derive(Debug, Clone)]
pub struct SimilarityRoleScore {
    pub role: SimilarityRole,
    pub score: f64,
    pub weight: f64,
    pub comparable: bool,
}
#[derive(Debug, Clone)]
pub struct SimilarityPair {
    pub seed_part: SimilarityPartSelector,
    pub seed: Option<SearchWitness>,
    pub seed_text: String,
    pub candidate: SearchWitness,
    pub cosine: f64,
}
struct PendingPair {
    seed_part: SimilarityPartSelector,
    seed: Option<SourceUnitLocation>,
    seed_text: Option<String>,
    candidate: SourceUnitLocation,
    cosine: f64,
    seed_label: String,
    candidate_label: String,
}
struct PendingEvidence {
    score: f64,
    aggregation: String,
    selected_parts: usize,
    comparable_parts: usize,
    parts: Vec<SimilarityPartScore>,
    roles: Vec<SimilarityRoleScore>,
    pairs: Vec<PendingPair>,
    representative: bool,
}
impl ConfigurableSimilarRequest<'_> {
    fn check(&self) -> Result<(), SearchError> {
        if self.cancelled.is_some_and(|cancelled| cancelled()) {
            return Err(SearchError::operation_cancelled());
        }
        Ok(())
    }
}

impl crate::AtlasRetrievalService {
    pub fn similarity_parts(
        &self,
        seed: &SimilaritySeed,
        profile: Option<&SearchProfileId>,
    ) -> Result<SimilarityParts, SearchError> {
        if let SimilaritySeed::Record { record_key } = seed {
            if self.index.read_summary(record_key)?.is_none() {
                return Err(SearchError::invalid_search_options(format!(
                    "comparison seed {record_key} was not found"
                )));
            }
        }
        let profile = self.resolve_search_profile(profile, false)?;
        Ok(self.index.similarity_parts(seed, Some(&profile))?)
    }

    pub fn configurable_similar(
        &self,
        request: ConfigurableSimilarRequest<'_>,
    ) -> Result<Option<ConfigurableSimilarResult>, SearchError> {
        request.check()?;
        request
            .options
            .validate(request.seed)
            .map_err(SearchError::invalid_search_options)?;
        Self::validate_search_sort(&request.sort)?;
        let found = match request.seed {
            SimilaritySeed::Record { record_key } => self.index.read_summary(record_key)?.is_some(),
            SimilaritySeed::Trait { trait_key } => {
                self.index.trait_definition(trait_key)?.is_some()
            }
        };
        if !found {
            return Ok(None);
        }
        if matches!(request.options, SimilarityOptions::Baseline {}) {
            let results = match request.seed {
                SimilaritySeed::Record { record_key } => self
                    .similar_records(crate::SimilarRecordRequest {
                        sort: request.sort,
                        search_profile: request.search_profile,
                        record_key,
                        filter: request.filter,
                        scope: request.scope,
                        page: request.page,
                        prefer_remaster: request.prefer_remaster,
                    })?
                    .map(|result| result.results),
                SimilaritySeed::Trait { trait_key } => self
                    .similar_trait(crate::SimilarTraitRequest {
                        sort: request.sort,
                        search_profile: request.search_profile,
                        trait_key,
                        filter: request.filter,
                        scope: request.scope,
                        page: request.page,
                        prefer_remaster: request.prefer_remaster,
                        ranking: Default::default(),
                    })?
                    .map(|result| result.results),
            };
            return Ok(results.map(|results| ConfigurableSimilarResult {
                seed: request.seed.clone(),
                results,
                evidence: vec![],
                effective: request.options.clone(),
                recipe: "current-baseline/v1".into(),
                diagnostics: SimilarityDiagnostics::default(),
            }));
        }
        let profile = self.resolve_search_profile(request.search_profile, true)?;
        let inventory = self.similarity_parts(request.seed, Some(&profile))?;
        let selected = selected_parts(&inventory, request.options)?;
        let mut seed_units = match request.seed {
            SimilaritySeed::Record { record_key } => self.index.selected_similarity_units(
                &profile,
                std::slice::from_ref(record_key),
                Some(&selected.iter().cloned().collect::<Vec<_>>()),
                MAX_SEED_UNITS,
            )?,
            SimilaritySeed::Trait { trait_key } => {
                self.index
                    .trait_similarity_units(&profile, trait_key, MAX_SEED_UNITS)?
            }
        };
        seed_units.retain(|unit| selected.contains(&unit.part.selector));
        if selected.iter().any(|selector| {
            !seed_units
                .iter()
                .any(|unit| &unit.part.selector == selector)
        }) {
            return Err(SearchError::vector_readiness_required(
                "a selected comparison part has no stored vectors; refresh the parts inventory or update the reference workspace",
            ));
        }
        if seed_units.is_empty() {
            return Err(SearchError::invalid_search_options(
                "this seed has no indexed prose to compare; choose Name and identity instead",
            ));
        }
        let seed_parts = crate::similarity_scoring::parts(&seed_units);
        let mode = request.options.candidate_parts();
        let predicate = self.predicate(request.filter)?;
        let exclude = match request.seed {
            SimilaritySeed::Record { record_key } => Some(record_key),
            SimilaritySeed::Trait { .. } => None,
        };
        let mut diagnostics = SimilarityDiagnostics {
            seed_units: seed_units.len(),
            ..Default::default()
        };
        let mut queries = seed_units
            .iter()
            .map(|unit| {
                (
                    unit.vector.clone(),
                    if mode == SimilarityCandidateParts::Corresponding {
                        Some(unit.part.role)
                    } else {
                        None
                    },
                )
            })
            .collect::<Vec<_>>();
        if mode == SimilarityCandidateParts::Any {
            let average = crate::similarity_scoring::parts_centroid(&seed_parts);
            if matches!(request.options, SimilarityOptions::ContentAverage { .. })
                && average.is_none()
            {
                return Err(SearchError::invalid_search_options(
                    "selected content has no defined average vector; use Passage matches or change the parts",
                ));
            }
            if let Some(average) = average {
                queries.push((average, None));
            }
        } else {
            let mut roles = BTreeMap::<SimilarityRole, Vec<Vec<f32>>>::new();
            for part in &seed_parts {
                let vector = crate::similarity_scoring::part_centroid(part);
                if matches!(request.options, SimilarityOptions::ContentAverage { .. })
                    && vector.is_none()
                {
                    return Err(SearchError::invalid_search_options(
                        "a selected part has no defined average vector; use Passage matches or change the parts",
                    ));
                }
                if let Some(vector) = vector {
                    roles.entry(part.metadata.role).or_default().push(vector);
                }
            }
            for (role, vectors) in roles {
                let average = crate::similarity_scoring::centroid(
                    vectors.iter().map(|vector| (vector.as_slice(), 1.0)),
                );
                if matches!(request.options, SimilarityOptions::ContentAverage { .. })
                    && average.is_none()
                {
                    return Err(SearchError::invalid_search_options(
                        "a selected content role has no defined average vector; use Passage matches or change the parts",
                    ));
                }
                if let Some(average) = average {
                    queries.push((average, Some(role)));
                }
            }
        }
        let mut keys = BTreeSet::new();
        for (vector, role) in queries {
            request.check()?;
            for hit in self.index.similarity_candidates(
                &profile,
                &vector,
                &predicate,
                request.scope.keys(),
                exclude,
                role,
                CANDIDATE_WINDOW,
            )? {
                keys.insert(hit.location.record);
            }
            diagnostics.knn_queries += 1;
            if keys.len() > MAX_CANDIDATE_ROOTS {
                return Err(SearchError::invalid_search_options(format!(
                    "comparison found more than {MAX_CANDIDATE_ROOTS} candidate records; narrow the selected parts or candidate filters"
                )));
            }
        }
        let keys = keys.into_iter().collect::<Vec<_>>();
        let mut roots = Vec::new();
        let mut pending = BTreeMap::new();
        let mut budgeted_comparisons = 0usize;
        for batch in keys.chunks(128) {
            request.check()?;
            let remaining = MAX_CANDIDATE_UNITS.saturating_sub(diagnostics.candidate_vectors);
            if remaining == 0 {
                return Err(SearchError::invalid_search_options(
                    "comparison exceeded its candidate vector budget; narrow the selected parts or candidate filters",
                ));
            }
            let units = self.index.similarity_units(&profile, batch, remaining)?;
            diagnostics.candidate_vectors += units.len();
            let workload = if matches!(request.options, SimilarityOptions::Passages { .. }) {
                units.len().checked_mul(seed_units.len())
            } else {
                batch
                    .len()
                    .checked_mul(seed_units.len() + seed_parts.len() + 1)
                    .and_then(|count| count.checked_add(units.len()))
            };
            let comparisons = workload
                .and_then(|count| count.checked_add(budgeted_comparisons))
                .ok_or_else(|| {
                    SearchError::invalid_search_options(
                        "comparison workload exceeds its supported range",
                    )
                })?;
            if comparisons > MAX_COMPARISONS {
                return Err(SearchError::invalid_search_options(format!(
                    "comparison needs more than {MAX_COMPARISONS} vector pairs; narrow the selected parts or candidate filters"
                )));
            }
            budgeted_comparisons = comparisons;
            let mut grouped = BTreeMap::<RecordKey, Vec<SourceSimilarityUnit>>::new();
            for unit in units {
                if let Some(location) = &unit.location {
                    grouped
                        .entry(location.record.clone())
                        .or_default()
                        .push(unit);
                }
            }
            for (key, units) in grouped {
                request.check()?;
                let candidate_parts = crate::similarity_scoring::parts(&units);
                let score = match request.options {
                    SimilarityOptions::Passages { decay, .. } => {
                        Some(crate::similarity_scoring::passages(
                            &seed_parts,
                            &candidate_parts,
                            mode,
                            *decay,
                            request.cancelled,
                        )?)
                    }
                    SimilarityOptions::ContentAverage { .. } => {
                        crate::similarity_scoring::content_average(
                            &seed_parts,
                            &candidate_parts,
                            mode,
                            request.cancelled,
                        )?
                    }
                    SimilarityOptions::Baseline {} => None,
                };
                let Some(score) = score else {
                    diagnostics.undefined_candidate_centroids += 1;
                    continue;
                };
                diagnostics.pairwise_comparisons += score.comparisons;
                diagnostics.undefined_candidate_centroids += score.undefined_centroids;
                if score.score <= 0.0 || score.comparable_parts == 0 {
                    continue;
                }
                let pairs = score
                    .pairs
                    .into_iter()
                    .filter_map(|pair| {
                        Some(PendingPair {
                            seed_part: pair.seed.part.selector.clone(),
                            seed: pair.seed.location.clone(),
                            seed_text: pair.seed.trait_text.clone(),
                            candidate: pair.candidate.location.clone()?,
                            cosine: pair.cosine,
                            seed_label: pair.seed.part.label.clone(),
                            candidate_label: pair.candidate.part.label.clone(),
                        })
                    })
                    .collect::<Vec<_>>();

                roots.push(crate::text::RankedRoot {
                    key: key.clone(),
                    lexical_rank: None,
                    similarity: Some(score.score),
                    score: score.score,
                    tier: 3,
                    // Page selection needs ranks/summaries only. Recover seed and
                    // candidate witnesses together once the page is known.
                    witnesses: vec![],
                });
                pending.insert(
                    key,
                    PendingEvidence {
                        score: score.score,
                        aggregation: score.aggregation.into(),
                        selected_parts: seed_parts.len(),
                        comparable_parts: score.comparable_parts,
                        parts: score
                            .parts
                            .into_iter()
                            .map(|part| SimilarityPartScore {
                                part: part.part,
                                score: part.score,
                                weight: part.weight,
                                comparable: part.comparable,
                            })
                            .collect(),
                        roles: score
                            .roles
                            .into_iter()
                            .map(|role| SimilarityRoleScore {
                                role: role.role,
                                score: role.score,
                                weight: role.weight,
                                comparable: role.comparable,
                            })
                            .collect(),
                        pairs,
                        representative: score.representative,
                    },
                );
            }
        }
        roots.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.key.cmp(&b.key)));
        let roots = self.suppress_pairs(roots, request.prefer_remaster)?;
        request.check()?;
        let mut results = self.finish_results(
            &profile,
            roots,
            request.page,
            true,
            Some(CANDIDATE_WINDOW),
            request.sort.clone(),
        )?;
        let mut evidence = Vec::new();
        for record in &results.records {
            request.check()?;
            if let Some(pending) = pending.remove(&record.record.key) {
                evidence.push((
                    record.record.key.clone(),
                    prepare_similarity_evidence(pending),
                ));
            }
        }
        let mut witnesses = evidence
            .iter_mut()
            .flat_map(|(_, evidence)| {
                evidence.pairs.iter_mut().flat_map(|pair| {
                    pair.seed
                        .iter_mut()
                        .chain(std::iter::once(&mut pair.candidate))
                })
            })
            .collect::<Vec<_>>();
        self.hydrate_similarity_witnesses(&mut witnesses, &request)?;
        for (key, evidence) in &mut evidence {
            for pair in &mut evidence.pairs {
                if let Some(text) = pair.seed.as_ref().and_then(|seed| seed.snippet.as_ref()) {
                    pair.seed_text = text.clone();
                }
            }
            if let Some(record) = results
                .records
                .iter_mut()
                .find(|record| &record.record.key == key)
            {
                record.matches = evidence
                    .pairs
                    .iter()
                    .map(|pair| pair.candidate.clone())
                    .collect();
            }
        }
        Ok(Some(ConfigurableSimilarResult {
            seed: request.seed.clone(),
            results,
            evidence,
            effective: request.options.clone(),
            recipe: SIMILARITY_RECIPE.into(),
            diagnostics,
        }))
    }

    fn hydrate_similarity_witnesses(
        &self,
        witnesses: &mut [&mut SearchWitness],
        request: &ConfigurableSimilarRequest<'_>,
    ) -> Result<(), SearchError> {
        let mut locations = Vec::new();
        for witness in witnesses.iter() {
            if !locations.contains(&witness.location) {
                locations.push(witness.location.clone());
            }
        }
        request.check()?;
        let recovered = self.index.recover_passages(&locations)?;
        let mut owned_labels = Vec::new();
        for batch in locations.chunks(256) {
            request.check()?;
            let owned = batch
                .iter()
                .filter(|location| !location.owners.is_empty())
                .cloned()
                .collect::<Vec<_>>();
            if !owned.is_empty() {
                owned_labels.extend(self.index.owned_search_labels(&owned)?);
            }
        }
        for witness in witnesses {
            request.check()?;
            if let Some((_, text)) = locations
                .iter()
                .zip(&recovered)
                .find(|(location, _)| *location == &witness.location)
            {
                witness.snippet = text.as_deref().map(crate::text::reading_excerpt);
            }
            if let Some((_, _, owner_label)) = owned_labels.iter().find(|(key, owners, _)| {
                key == &witness.location.record && owners == &witness.location.owners
            }) && !owner_label.trim().is_empty()
            {
                witness.label = Some(match witness.label.as_deref() {
                    Some(section) if !section.trim().is_empty() && section != owner_label => {
                        format!("{owner_label} — {section}")
                    }
                    _ => owner_label.clone(),
                });
            }
        }
        Ok(())
    }
}
fn prepare_similarity_evidence(pending: PendingEvidence) -> SimilarityEvidence {
    let pairs = pending
        .pairs
        .into_iter()
        .map(|pair| SimilarityPair {
            seed_part: pair.seed_part,
            seed: pair
                .seed
                .map(|location| raw_witness(location, pair.cosine, &pair.seed_label)),
            seed_text: crate::text::reading_excerpt(pair.seed_text.as_deref().unwrap_or_default()),
            candidate: raw_witness(pair.candidate, pair.cosine, &pair.candidate_label),
            cosine: pair.cosine,
        })
        .collect();
    let truncated = pending.parts.len() > MAX_EXPLANATION_PARTS;
    SimilarityEvidence {
        score: pending.score,
        aggregation: pending.aggregation,
        selected_parts: pending.selected_parts,
        comparable_parts: pending.comparable_parts,
        parts: pending
            .parts
            .into_iter()
            .take(MAX_EXPLANATION_PARTS)
            .collect(),
        roles: pending.roles,
        pairs,
        representative: pending.representative,
        truncated,
    }
}

fn raw_witness(location: SourceUnitLocation, cosine: f64, label: &str) -> SearchWitness {
    SearchWitness {
        location,
        lane: crate::SearchLane::Semantic,
        label: Some(label.into()),
        snippet: None,
        lexical_rank: None,
        semantic_similarity: Some(cosine),
    }
}
fn selected_parts(
    inventory: &SimilarityParts,
    options: &SimilarityOptions,
) -> Result<BTreeSet<SimilarityPartSelector>, SearchError> {
    let available = inventory
        .parts
        .iter()
        .map(|part| part.selector.clone())
        .collect::<BTreeSet<_>>();
    let selected = options
        .parts()
        .map(|parts| parts.iter().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_else(|| available.clone());
    if selected.is_empty() {
        return Err(SearchError::invalid_search_options(
            "this seed has no indexed prose; choose Name and identity instead",
        ));
    }
    if !selected.is_subset(&available) {
        return Err(SearchError::invalid_search_options(
            "a selected comparison part is stale or unavailable; refresh the parts inventory",
        ));
    }
    Ok(selected)
}

#[cfg(test)]
mod tests;
