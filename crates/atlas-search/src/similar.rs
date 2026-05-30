use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_embedding::EmbeddingUnitKind;
use atlas_index::{RecordEmbeddingVector, ReferenceEdgeDirection};
use atlas_record::PersistedRecord;

use crate::semantic::collapse_vector_hits;
use crate::{AtlasRetrievalService, SearchError, SemanticSearchHit, SemanticSearchMode};

const DEFAULT_SEMANTIC_WEIGHT: f64 = 0.80;
const DEFAULT_REFERENCE_WEIGHT: f64 = 0.15;
const DEFAULT_TRAIT_WEIGHT: f64 = 0.05;
const MAX_SIMILAR_CANDIDATES: u32 = 1_000;

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarRecordRequest<'a> {
    pub seed: &'a RecordKey,
    pub filter: Option<&'a SearchFilterNode>,
    pub limit: u32,
    pub candidate_limit: u32,
    pub weights: SimilarScoreWeights,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarScoreWeights {
    pub semantic: f64,
    pub reference: f64,
    pub traits: f64,
}

impl Default for SimilarScoreWeights {
    fn default() -> Self {
        Self {
            semantic: DEFAULT_SEMANTIC_WEIGHT,
            reference: DEFAULT_REFERENCE_WEIGHT,
            traits: DEFAULT_TRAIT_WEIGHT,
        }
    }
}

impl SimilarScoreWeights {
    pub fn validate(self) -> Result<Self, String> {
        for (label, weight) in [
            ("semantic", self.semantic),
            ("reference", self.reference),
            ("traits", self.traits),
        ] {
            if !weight.is_finite() || weight < 0.0 {
                return Err(format!(
                    "similar {label} weight must be finite and non-negative"
                ));
            }
        }
        if self.semantic <= 0.0 {
            return Err("similar semantic weight must be greater than zero".to_string());
        }
        if self.reference + self.traits >= self.semantic {
            return Err(
                "similar semantic weight must be greater than the combined reference and trait weights"
                    .to_string(),
            );
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarRecordResult {
    pub seed: PersistedRecord,
    pub records: Vec<SimilarRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarRecord {
    pub record: PersistedRecord,
    pub score: f64,
    pub semantic: SimilarRecordSemanticEvidence,
    pub graph: SimilarRecordGraphEvidence,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarRecordSemanticEvidence {
    pub unit_kind: String,
    pub label: Option<String>,
    pub distance: f64,
    pub rank_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarRecordGraphEvidence {
    pub shared_references: Vec<SimilarSharedReference>,
    pub shared_traits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarSharedReference {
    pub key: RecordKey,
    pub name: String,
}

impl AtlasRetrievalService {
    pub fn similar_records(
        &self,
        request: SimilarRecordRequest<'_>,
    ) -> Result<Option<SimilarRecordResult>, SearchError> {
        let Some(seed) = self.get_record(request.seed)? else {
            return Ok(None);
        };

        let resolved_filter = self.index.resolve_metric_filters(request.filter)?;
        let filter = resolved_filter.as_ref().or(request.filter);
        if let Some(filter) = filter {
            filter
                .validate()
                .map_err(|error| SearchError::InvalidSearchOptions(error.to_string()))?;
        }
        let weights = request
            .weights
            .validate()
            .map_err(SearchError::InvalidSearchOptions)?;

        let seed_unit = select_seed_embedding_unit(
            self.index
                .load_record_embedding_vectors(request.seed)?
                .as_slice(),
        )?
        .clone();
        let candidate_limit = request
            .candidate_limit
            .max(request.limit)
            .min(MAX_SIMILAR_CANDIDATES);
        let vector_limit = candidate_limit
            .saturating_add(1)
            .min(MAX_SIMILAR_CANDIDATES);
        let vector_hits =
            self.index
                .query_vector_index(&seed_unit.vector, filter, vector_limit, false)?;
        let mut semantic_hits = collapse_vector_hits(
            vector_hits,
            vector_limit as usize,
            SemanticSearchMode::ParentOnly,
        );
        semantic_hits.retain(|hit| hit.record_key != request.seed.to_string());
        semantic_hits.truncate(candidate_limit as usize);

        let candidate_keys = semantic_hits
            .iter()
            .filter_map(|hit| RecordKey::parse(&hit.record_key).ok())
            .collect::<Vec<_>>();
        let records_by_key = self
            .get_records(&candidate_keys)?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();

        let seed_references = outgoing_reference_keys(self, request.seed)?;
        let reference_names = self.reference_names(&seed_references)?;
        let seed_traits = string_set(&seed.traits);
        let mut ranked = semantic_hits
            .into_iter()
            .filter_map(|hit| {
                let key = RecordKey::parse(&hit.record_key).ok()?;
                let record = records_by_key.get(&key)?.clone();
                Some((hit, record))
            })
            .map(|(hit, record)| {
                self.similar_record_for_hit(
                    hit,
                    record,
                    &seed_references,
                    &reference_names,
                    &seed_traits,
                    weights,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        ranked.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| {
                    left.semantic
                        .rank_distance
                        .total_cmp(&right.semantic.rank_distance)
                })
                .then_with(|| left.record.key.cmp(&right.record.key))
        });
        ranked.truncate(request.limit as usize);

        Ok(Some(SimilarRecordResult {
            seed,
            records: ranked,
        }))
    }

    fn reference_names(
        &self,
        reference_keys: &BTreeSet<RecordKey>,
    ) -> Result<BTreeMap<RecordKey, String>, SearchError> {
        Ok(self
            .get_records(&reference_keys.iter().cloned().collect::<Vec<_>>())?
            .into_iter()
            .map(|record| (record.key, record.name))
            .collect())
    }

    fn similar_record_for_hit(
        &self,
        hit: SemanticSearchHit,
        record: PersistedRecord,
        seed_references: &BTreeSet<RecordKey>,
        reference_names: &BTreeMap<RecordKey, String>,
        seed_traits: &BTreeSet<String>,
        weights: SimilarScoreWeights,
    ) -> Result<SimilarRecord, SearchError> {
        let candidate_references = outgoing_reference_keys(self, &record.key)?;
        let shared_references = seed_references
            .intersection(&candidate_references)
            .map(|key| SimilarSharedReference {
                key: key.clone(),
                name: reference_names
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| key.to_string()),
            })
            .collect::<Vec<_>>();
        let candidate_traits = string_set(&record.traits);
        let shared_traits = seed_traits
            .intersection(&candidate_traits)
            .cloned()
            .collect::<Vec<_>>();
        let score = similar_score(
            hit.rank_distance,
            shared_references.len(),
            shared_traits.len(),
            weights,
        );
        Ok(SimilarRecord {
            record,
            score,
            semantic: SimilarRecordSemanticEvidence {
                unit_kind: hit.unit_kind,
                label: hit.label,
                distance: hit.distance,
                rank_distance: hit.rank_distance,
            },
            graph: SimilarRecordGraphEvidence {
                shared_references,
                shared_traits,
            },
        })
    }
}

fn select_seed_embedding_unit(
    units: &[RecordEmbeddingVector],
) -> Result<&RecordEmbeddingVector, SearchError> {
    units
        .iter()
        .find(|unit| unit.unit_kind == EmbeddingUnitKind::Parent.as_str())
        .ok_or(SearchError::UnsupportedRetrievalPattern(
            "similar records require a stored parent seed embedding",
        ))
}

fn outgoing_reference_keys(
    service: &AtlasRetrievalService,
    seed: &RecordKey,
) -> Result<BTreeSet<RecordKey>, SearchError> {
    Ok(service
        .index
        .reference_edges_for_seed(seed, ReferenceEdgeDirection::Outgoing)?
        .into_iter()
        .map(|edge| edge.to_record_key)
        .collect())
}

fn string_set(values: &[String]) -> BTreeSet<String> {
    values
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect()
}

fn similar_score(
    rank_distance: f64,
    shared_references: usize,
    shared_traits: usize,
    weights: SimilarScoreWeights,
) -> f64 {
    let semantic = 1.0 / (1.0 + rank_distance.max(0.0));
    let reference = capped_ratio(shared_references, 5);
    let traits = capped_ratio(shared_traits, 5);
    weights.semantic.mul_add(
        semantic,
        weights
            .reference
            .mul_add(reference, weights.traits * traits),
    )
}

fn capped_ratio(count: usize, cap: usize) -> f64 {
    (count.min(cap) as f64) / (cap as f64)
}

#[cfg(test)]
mod tests;
