//! Stored-vector similarity workflows shared by terminal and browser consumers.
use crate::{RecordScope, SearchError, SearchPage, SearchWitness, TextSearchResult};
use atlas_domain::{
    CORE_SECTIONS_RECIPE, CoreSectionSelection, QueryPredicate, RecordKey, ResultSort,
    SIMILARITY_COMPONENT_RECIPE, SIMILARITY_RECIPE, SIMILARITY_ROLE_RECIPE, SearchContentScope,
    SearchProfileId, SimilarityCandidateParts, SimilarityComparisonKey, SimilarityComponent,
    SimilarityOptions, SimilarityPartSelector, SimilarityParts, SimilaritySeed,
    SimilaritySourceSections,
};
use atlas_index::{SourceSimilarityUnit, SourceUnitLocation};
use std::collections::{BTreeMap, BTreeSet, HashMap};

const MAX_SEED_UNITS: usize = 128;
const MAX_CANDIDATE_UNITS: usize = 65_536;
const MAX_COMPARISONS: usize = 8_000_000;
const CANDIDATE_WINDOW: usize = 256;
const MAX_EXPLANATION_PARTS: usize = 32;
const LIMIT_RECOVERY: &str =
    "Deselect some sections under Sections to compare, or add filters to narrow the results.";

#[derive(Default)]
struct CandidateWorkBudget {
    vectors: usize,
    comparisons: usize,
}

impl CandidateWorkBudget {
    fn remaining_vectors(&self) -> Result<usize, SearchError> {
        let remaining = MAX_CANDIDATE_UNITS.saturating_sub(self.vectors);
        if remaining == 0 {
            return Err(SearchError::invalid_search_options(format!(
                "This comparison is too broad to complete. {LIMIT_RECOVERY} (Vector limit: {MAX_CANDIDATE_UNITS}.)"
            )));
        }
        Ok(remaining)
    }

    fn include(
        &mut self,
        vectors: usize,
        seed_units: usize,
        seed_parts: usize,
        roots: usize,
        passages: bool,
    ) -> Result<(), SearchError> {
        let total_vectors = self.vectors.checked_add(vectors).ok_or_else(|| {
            SearchError::invalid_search_options(format!(
                "This comparison is too broad to complete. {LIMIT_RECOVERY} (Workload exceeds the supported range.)"
            ))
        })?;
        if total_vectors > MAX_CANDIDATE_UNITS {
            return Err(SearchError::invalid_search_options(format!(
                "This comparison is too broad to complete. {LIMIT_RECOVERY} (Vector limit: {MAX_CANDIDATE_UNITS}.)"
            )));
        }
        let workload = if passages {
            vectors.checked_mul(seed_units)
        } else {
            seed_units
                .checked_add(seed_parts)
                .and_then(|count| count.checked_add(1))
                .and_then(|count| roots.checked_mul(count))
                .and_then(|count| count.checked_add(vectors))
        };
        let comparisons = workload
            .and_then(|count| self.comparisons.checked_add(count))
            .ok_or_else(|| {
                SearchError::invalid_search_options(format!(
                    "This comparison is too broad to complete. {LIMIT_RECOVERY} (Workload exceeds the supported range.)"
                ))
            })?;
        if comparisons > MAX_COMPARISONS {
            return Err(SearchError::invalid_search_options(format!(
                "This comparison is too broad to complete. {LIMIT_RECOVERY} (Vector-pair limit: {MAX_COMPARISONS}.)"
            )));
        }
        self.vectors = total_vectors;
        self.comparisons = comparisons;
        Ok(())
    }
}

pub struct ConfigurableSimilarRequest<'a> {
    pub seed: &'a SimilaritySeed,
    pub options: &'a SimilarityOptions,
    pub source_sections: Option<SimilaritySourceSections>,
    pub content_scope: SearchContentScope,
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
    pub source_sections: Option<SimilaritySourceSections>,
    pub content_scope: SearchContentScope,
    pub candidate_identity: bool,
    pub core_recipe: Option<String>,
    pub resolved_selectors: Vec<SimilarityPartSelector>,
    pub core: Option<CoreSectionSelection>,
    pub recipe: String,
    pub role_recipe: String,
    pub diagnostics: SimilarityDiagnostics,
    /// Complete pre-reranking root union for content methods; legacy baseline
    /// retains its separate candidate pipeline and does not expose this union.
    pub candidate_keys: Option<Vec<RecordKey>>,
}
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SimilarityDiagnostics {
    pub seed_units: usize,
    pub knn_queries: usize,
    pub candidate_vectors: usize,
    pub pairwise_comparisons: usize,
    pub undefined_candidate_centroids: usize,
    pub timings: SimilarityTimings,
    #[cfg(feature = "read-metrics")]
    pub eligibility: Option<atlas_index::EligibilityStats>,
}
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SimilarityTimings {
    pub seed_preparation_ms: Option<f64>,
    pub candidate_generation_ms: Option<f64>,
    pub vector_read_ms: Option<f64>,
    pub scoring_ms: Option<f64>,
    pub evidence_hydration_ms: Option<f64>,
    pub total_ms: Option<f64>,
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
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimilarityPartScore {
    pub part: SimilarityComponent,
    pub score: f64,
    pub weight: f64,
    pub comparable: bool,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimilarityRoleScore {
    pub key: SimilarityComparisonKey,
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
    seed_unit: usize,
    candidate: SourceUnitLocation,
    cosine: f64,
    candidate_label: String,
}
struct PendingPartScore {
    source_part: usize,
    score: f64,
    weight: f64,
    comparable: bool,
}
struct PendingEvidence {
    score: f64,
    aggregation: &'static str,
    selected_parts: usize,
    comparable_parts: usize,
    parts: Vec<PendingPartScore>,
    roles: Vec<SimilarityRoleScore>,
    pairs: Vec<PendingPair>,
    representative: bool,
    truncated: bool,
}

#[derive(PartialEq, Eq, Hash)]
enum SeedUnitKey<'a> {
    Native(i64),
    Trait(
        &'a SimilarityPartSelector,
        &'a atlas_domain::SourcePassageAddress,
    ),
}

fn seed_unit_key(unit: &SourceSimilarityUnit) -> SeedUnitKey<'_> {
    match &unit.location {
        Some(location) => SeedUnitKey::Native(location.unit_id),
        // Trait chunks have no record witness. Equal source addresses recover
        // the same trait text and label, including any repeated chunk coverage.
        None => SeedUnitKey::Trait(unit.component.selector(), &unit.address),
    }
}

fn compact_part_scores(
    parts: Vec<crate::similarity_scoring::PartScore>,
    source_indices: &BTreeMap<&SimilarityPartSelector, usize>,
) -> Result<(Vec<PendingPartScore>, bool), SearchError> {
    let truncated = parts.len() > MAX_EXPLANATION_PARTS;
    // Allocate only the capped compact rows, rather than reusing the scorer's
    // larger allocation (which contains full metadata for every source part).
    let mut compact = Vec::with_capacity(parts.len().min(MAX_EXPLANATION_PARTS));
    for part in parts.into_iter().take(MAX_EXPLANATION_PARTS) {
        compact.push(PendingPartScore {
            source_part: source_indices
                .get(part.part.selector())
                .copied()
                .ok_or_else(|| {
                    SearchError::artifact_contract_violation(
                        "comparison source section disappeared",
                    )
                })?,
            score: part.score,
            weight: part.weight,
            comparable: part.comparable,
        });
    }
    Ok((compact, truncated))
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
    pub fn similarity_part_addresses(
        &self,
        record: &RecordKey,
    ) -> Result<BTreeMap<SimilarityPartSelector, atlas_domain::SourcePassageAddress>, SearchError>
    {
        Ok(self.index.similarity_part_addresses(record)?)
    }

    pub fn similarity_parts(
        &self,
        seed: &SimilaritySeed,
        profile: Option<&SearchProfileId>,
    ) -> Result<SimilarityParts, SearchError> {
        if let SimilaritySeed::Record { record_key } = seed
            && self.index.read_summary(record_key)?.is_none()
        {
            return Err(SearchError::invalid_search_options(format!(
                "comparison seed {record_key} was not found"
            )));
        }
        let profile = self.resolve_search_profile(profile, false)?;
        Ok(self.index.similarity_parts(seed, Some(&profile))?)
    }

    pub fn configurable_similar(
        &self,
        request: ConfigurableSimilarRequest<'_>,
    ) -> Result<Option<ConfigurableSimilarResult>, SearchError> {
        self.configurable_similar_impl::<true>(request)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn configurable_similar_exact(
        &self,
        request: ConfigurableSimilarRequest<'_>,
    ) -> Result<Option<ConfigurableSimilarResult>, SearchError> {
        self.configurable_similar_impl::<false>(request)
    }

    fn configurable_similar_impl<const REUSE: bool>(
        &self,
        request: ConfigurableSimilarRequest<'_>,
    ) -> Result<Option<ConfigurableSimilarResult>, SearchError> {
        let started = std::time::Instant::now();
        request.check()?;
        request
            .options
            .validate(request.seed)
            .map_err(SearchError::invalid_search_options)?;
        Self::validate_search_sort(&request.sort)?;
        let source_sections = if matches!(request.options, SimilarityOptions::Baseline {}) {
            if request.source_sections.is_some() {
                return Err(SearchError::invalid_search_options(
                    "Name and identity / Trait definition does not support a source section preset",
                ));
            }
            None
        } else {
            Some(resolve_source_sections(
                request.options,
                request.source_sections,
            )?)
        };
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
                        content_scope: request.content_scope,
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
                        content_scope: request.content_scope,
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
                source_sections: None,
                content_scope: request.content_scope,
                candidate_identity: true,
                core_recipe: (request.content_scope == SearchContentScope::Core)
                    .then(|| CORE_SECTIONS_RECIPE.into()),
                resolved_selectors: vec![],
                core: None,
                recipe: "current-baseline/v1".into(),
                role_recipe: SIMILARITY_ROLE_RECIPE.into(),
                diagnostics: SimilarityDiagnostics {
                    timings: SimilarityTimings {
                        total_ms: Some(started.elapsed().as_secs_f64() * 1000.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                candidate_keys: None,
            }));
        }
        let profile = self.resolve_search_profile(request.search_profile, true)?;
        let inventory = self.similarity_parts(request.seed, Some(&profile))?;
        let Some(source_sections) = source_sections else {
            return Err(SearchError::artifact_contract_violation(
                "content comparison source mode was not resolved",
            ));
        };
        let selected = selected_components(&inventory, request.options, source_sections)?;
        if inventory
            .parts
            .iter()
            .any(|component| selected.contains(component.selector()) && !component.has_vectors())
        {
            return Err(SearchError::vector_readiness_required(
                "a selected comparison component has no stored vectors; refresh the sections inventory or update the reference workspace",
            ));
        }
        let source_identity = inventory
            .parts
            .iter()
            .any(|component| component.is_identity() && selected.contains(component.selector()));
        let candidate_identity =
            source_identity || request.content_scope == SearchContentScope::Core;
        let mut seed_units = match request.seed {
            SimilaritySeed::Record { record_key } => self.index.selected_similarity_units(
                &profile,
                std::slice::from_ref(record_key),
                Some(&selected.iter().cloned().collect::<Vec<_>>()),
                SearchContentScope::All,
                source_identity,
                MAX_SEED_UNITS,
            )?,
            SimilaritySeed::Trait { trait_key } => {
                self.index
                    .trait_similarity_units(&profile, trait_key, MAX_SEED_UNITS)?
            }
        };
        seed_units.retain(|unit| selected.contains(unit.component.selector()));
        if selected.iter().any(|selector| {
            !seed_units
                .iter()
                .any(|unit| unit.component.selector() == selector)
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
        let seed_parts = crate::similarity_scoring::parts(&seed_units)?;
        let seed_part_indices = seed_parts
            .iter()
            .enumerate()
            .map(|(index, part)| (part.metadata.selector(), index))
            .collect::<BTreeMap<_, _>>();
        let seed_unit_indices = seed_units
            .iter()
            .enumerate()
            .map(|(index, unit)| (seed_unit_key(unit), index))
            .collect::<HashMap<_, _>>();
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
                        Some(unit.component.comparison_key())
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
            let mut roles = BTreeMap::<SimilarityComparisonKey, Vec<Vec<f32>>>::new();
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
                    roles
                        .entry(part.metadata.comparison_key())
                        .or_default()
                        .push(vector);
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
        diagnostics.timings.seed_preparation_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
        let generation_started = std::time::Instant::now();
        // Every query comes from a bounded seed unit or one nonempty role/group
        // of those units. Thus queries <= 2*128 and their complete 256-hit union
        // contains at most 65,536 roots, before the actual-work budgets below.
        debug_assert!(queries.len() <= seed_units.len() * 2);
        let mut keys = BTreeSet::new();
        let mut candidate_operation = REUSE.then(|| self.index.configured_similarity_candidates());
        for (vector, role) in queries {
            request.check()?;
            let input = atlas_index::SourceSimilarityCandidateRequest {
                profile: &profile,
                vector: &vector,
                query: &predicate,
                keys: request.scope.keys(),
                exclude,
                comparison_key: role,
                content_scope: request.content_scope,
                include_identity: candidate_identity,
                limit: CANDIDATE_WINDOW,
            };
            let hits = match candidate_operation.as_mut() {
                Some(operation) => {
                    operation.candidates::<SearchError>(input, || request.check())?
                }
                None => self.index.similarity_candidates(input)?,
            };
            for hit in hits {
                keys.insert(hit.location.record);
            }
            diagnostics.knn_queries += 1;
        }
        #[cfg(feature = "read-metrics")]
        {
            diagnostics.eligibility = candidate_operation
                .as_ref()
                .map(|operation| operation.stats());
        }
        drop(candidate_operation);
        diagnostics.timings.candidate_generation_ms =
            Some(generation_started.elapsed().as_secs_f64() * 1000.0);
        diagnostics.timings.vector_read_ms = Some(0.0);
        diagnostics.timings.scoring_ms = Some(0.0);
        let keys = keys.into_iter().collect::<Vec<_>>();
        let mut roots = Vec::new();
        let mut pending = BTreeMap::new();
        let mut budget = CandidateWorkBudget::default();
        for batch in keys.chunks(128) {
            request.check()?;
            let remaining = budget.remaining_vectors()?;
            let reading_started = std::time::Instant::now();
            let units = self.index.similarity_units(
                &profile,
                batch,
                request.content_scope,
                candidate_identity,
                remaining,
            )?;
            *diagnostics.timings.vector_read_ms.get_or_insert(0.0) +=
                reading_started.elapsed().as_secs_f64() * 1000.0;
            let scoring_started = std::time::Instant::now();
            budget.include(
                units.len(),
                seed_units.len(),
                seed_parts.len(),
                batch.len(),
                matches!(request.options, SimilarityOptions::Passages { .. }),
            )?;
            diagnostics.candidate_vectors = budget.vectors;
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
                let candidate_parts = crate::similarity_scoring::parts(&units)?;
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
                    .filter(|pair| pair.candidate.location.is_some())
                    .map(|pair| {
                        Ok(PendingPair {
                            seed_unit: seed_unit_indices
                                .get(&seed_unit_key(pair.seed))
                                .copied()
                                .ok_or_else(|| {
                                    SearchError::artifact_contract_violation(
                                        "comparison seed unit disappeared",
                                    )
                                })?,
                            candidate: pair.candidate.location.clone().ok_or_else(|| {
                                SearchError::artifact_contract_violation(
                                    "comparison candidate location disappeared",
                                )
                            })?,
                            cosine: pair.cosine,
                            candidate_label: pair.candidate.component.label().to_owned(),
                        })
                    })
                    .collect::<Result<Vec<_>, SearchError>>()?;
                let (parts, truncated) = compact_part_scores(score.parts, &seed_part_indices)?;

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
                        aggregation: score.aggregation,
                        selected_parts: seed_parts.len(),
                        comparable_parts: score.comparable_parts,
                        parts,
                        roles: score
                            .roles
                            .into_iter()
                            .map(|role| SimilarityRoleScore {
                                key: role.key,
                                score: role.score,
                                weight: role.weight,
                                comparable: role.comparable,
                            })
                            .collect(),
                        pairs,
                        representative: score.representative,
                        truncated,
                    },
                );
            }
            *diagnostics.timings.scoring_ms.get_or_insert(0.0) +=
                scoring_started.elapsed().as_secs_f64() * 1000.0;
        }
        roots.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.key.cmp(&b.key)));
        let roots = self.suppress_pairs(roots, request.prefer_remaster)?;
        request.check()?;
        let hydration_started = std::time::Instant::now();
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
                    prepare_similarity_evidence(pending, &seed_parts, &seed_units)?,
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
        diagnostics.timings.evidence_hydration_ms =
            Some(hydration_started.elapsed().as_secs_f64() * 1000.0);
        diagnostics.timings.total_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
        Ok(Some(ConfigurableSimilarResult {
            seed: request.seed.clone(),
            results,
            evidence,
            effective: request.options.clone(),
            source_sections: Some(source_sections),
            content_scope: request.content_scope,
            candidate_identity,
            core_recipe: (source_sections == SimilaritySourceSections::Core
                || candidate_identity
                || request.content_scope == SearchContentScope::Core)
                .then(|| CORE_SECTIONS_RECIPE.into()),
            resolved_selectors: selected.into_iter().collect(),
            core: (source_sections == SimilaritySourceSections::Core).then_some(inventory.core),
            recipe: if source_sections == SimilaritySourceSections::Core
                || candidate_identity
                || request.content_scope == SearchContentScope::Core
            {
                SIMILARITY_COMPONENT_RECIPE
            } else {
                SIMILARITY_RECIPE
            }
            .into(),
            role_recipe: inventory.role_recipe,
            diagnostics,
            candidate_keys: Some(keys),
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
fn prepare_similarity_evidence(
    pending: PendingEvidence,
    seed_parts: &[crate::similarity_scoring::Part<'_>],
    seed_units: &[SourceSimilarityUnit],
) -> Result<SimilarityEvidence, SearchError> {
    let pairs = pending
        .pairs
        .into_iter()
        .map(|pair| {
            let seed = seed_units.get(pair.seed_unit).ok_or_else(|| {
                SearchError::artifact_contract_violation("comparison seed unit disappeared")
            })?;
            Ok(SimilarityPair {
                seed_part: seed.component.selector().clone(),
                seed: seed
                    .location
                    .clone()
                    .map(|location| raw_witness(location, pair.cosine, seed.component.label())),
                seed_text: crate::text::reading_excerpt(
                    seed.trait_text.as_deref().unwrap_or_default(),
                ),
                candidate: raw_witness(pair.candidate, pair.cosine, &pair.candidate_label),
                cosine: pair.cosine,
            })
        })
        .collect::<Result<Vec<_>, SearchError>>()?;
    let parts = pending
        .parts
        .into_iter()
        .map(|part| {
            let source = seed_parts.get(part.source_part).ok_or_else(|| {
                SearchError::artifact_contract_violation("comparison source section disappeared")
            })?;
            Ok(SimilarityPartScore {
                part: source.metadata.clone(),
                score: part.score,
                weight: part.weight,
                comparable: part.comparable,
            })
        })
        .collect::<Result<Vec<_>, SearchError>>()?;
    Ok(SimilarityEvidence {
        score: pending.score,
        aggregation: pending.aggregation.into(),
        selected_parts: pending.selected_parts,
        comparable_parts: pending.comparable_parts,
        parts,
        roles: pending.roles,
        pairs,
        representative: pending.representative,
        truncated: pending.truncated,
    })
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
pub(crate) fn resolve_source_sections(
    options: &SimilarityOptions,
    source_sections: Option<SimilaritySourceSections>,
) -> Result<SimilaritySourceSections, SearchError> {
    let mode = source_sections.unwrap_or_else(|| {
        if options.parts().is_some() {
            SimilaritySourceSections::Custom
        } else {
            SimilaritySourceSections::All
        }
    });
    match (mode, options.parts()) {
        (SimilaritySourceSections::All | SimilaritySourceSections::Core, Some(_)) => {
            Err(SearchError::invalid_search_options(
                "a source section preset cannot be combined with exact section selectors",
            ))
        }
        (SimilaritySourceSections::Custom, None) => Err(SearchError::invalid_search_options(
            "Custom sections requires exact section selectors from the current inventory",
        )),
        _ => Ok(mode),
    }
}

pub(crate) fn selected_components(
    inventory: &SimilarityParts,
    options: &SimilarityOptions,
    source_sections: SimilaritySourceSections,
) -> Result<BTreeSet<SimilarityPartSelector>, SearchError> {
    let available = inventory
        .parts
        .iter()
        .map(|component| component.selector().clone())
        .collect::<BTreeSet<_>>();
    let selected: BTreeSet<SimilarityPartSelector> = match source_sections {
        SimilaritySourceSections::All => inventory
            .parts
            .iter()
            .map(|component| component.selector().clone())
            .collect(),
        SimilaritySourceSections::Core => inventory.core.selectors.iter().cloned().collect(),
        SimilaritySourceSections::Custom => options
            .parts()
            .ok_or_else(|| {
                SearchError::invalid_search_options(
                    "Custom sections requires exact section selectors",
                )
            })?
            .iter()
            .cloned()
            .collect(),
    };
    if selected.is_empty() {
        return Err(SearchError::invalid_search_options(
            "this seed has no indexed prose; choose Core sections or Name and identity instead",
        ));
    }
    if !selected.is_subset(&available) {
        return Err(SearchError::invalid_search_options(
            "a selected comparison section is stale or unavailable; refresh the sections inventory",
        ));
    }
    Ok(selected)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod eligibility_tests;
