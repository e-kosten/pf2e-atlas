use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_embedding::EmbeddingUnitKind;
use atlas_index::{RecordEmbeddingUnit, RecordLoadOptions, ReferenceEdgeDirection};
use atlas_record::PersistedRecord;

use crate::{
    AtlasRetrievalService, SearchError, SemanticSearchHit, SemanticSearchMode, collapse_vector_hits,
};

const DEFAULT_SEMANTIC_WEIGHT: f64 = 0.80;
const DEFAULT_MECHANIC_WEIGHT: f64 = 0.15;
const DEFAULT_TRAIT_WEIGHT: f64 = 0.05;

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarRecordRequest<'a> {
    pub seed: &'a RecordKey,
    pub filter: Option<&'a SearchFilterNode>,
    pub limit: u32,
    pub candidate_limit: u32,
    pub load_options: RecordLoadOptions,
    pub weights: SimilarScoreWeights,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarScoreWeights {
    pub semantic: f64,
    pub mechanic: f64,
    pub traits: f64,
}

impl Default for SimilarScoreWeights {
    fn default() -> Self {
        Self {
            semantic: DEFAULT_SEMANTIC_WEIGHT,
            mechanic: DEFAULT_MECHANIC_WEIGHT,
            traits: DEFAULT_TRAIT_WEIGHT,
        }
    }
}

impl SimilarScoreWeights {
    pub fn validate(self) -> Result<Self, String> {
        for (label, weight) in [
            ("semantic", self.semantic),
            ("mechanic", self.mechanic),
            ("traits", self.traits),
        ] {
            if !weight.is_finite() || weight < 0.0 {
                return Err(format!(
                    "similar {label} weight must be finite and non-negative"
                ));
            }
        }
        if self.semantic == 0.0 && self.mechanic == 0.0 && self.traits == 0.0 {
            return Err("at least one similar score weight must be greater than zero".to_string());
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarRecordResult {
    pub seed: PersistedRecord,
    pub seed_embedding_unit_key: String,
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
    pub embedding_unit_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub distance: f64,
    pub rank_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarRecordGraphEvidence {
    pub shared_mechanics: Vec<SimilarSharedMechanic>,
    pub shared_traits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarSharedMechanic {
    pub key: RecordKey,
    pub name: String,
}

impl AtlasRetrievalService {
    pub fn similar_records(
        &self,
        request: SimilarRecordRequest<'_>,
    ) -> Result<Option<SimilarRecordResult>, SearchError> {
        let Some(seed) = self
            .get_records_with_options(std::slice::from_ref(request.seed), request.load_options)?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };

        let resolved_filter = self.index.resolve_metric_filters(request.filter)?;
        let filter = resolved_filter.as_ref().or(request.filter);
        let weights = request
            .weights
            .validate()
            .map_err(SearchError::InvalidSearchOptions)?;
        if let Some(filter) = filter {
            filter
                .validate()
                .map_err(|error| SearchError::InvalidSearchOptions(error.to_string()))?;
        }

        let seed_unit = select_seed_embedding_unit(
            self.index
                .load_record_embedding_units(request.seed)?
                .as_slice(),
        )?
        .clone();
        let vector_limit = request
            .candidate_limit
            .max(request.limit)
            .saturating_add(1)
            .min(1000);
        let vector_hits =
            self.index
                .query_vector_index(&seed_unit.vector, filter, vector_limit, false)?;
        let mut semantic_hits = collapse_vector_hits(
            vector_hits,
            vector_limit as usize,
            SemanticSearchMode::ParentOnly,
        );
        semantic_hits.retain(|hit| hit.record_key != request.seed.to_string());
        semantic_hits.truncate(request.candidate_limit as usize);

        let candidate_keys = semantic_hits
            .iter()
            .filter_map(|hit| RecordKey::parse(&hit.record_key).ok())
            .collect::<Vec<_>>();
        let records_by_key = self
            .get_records_with_options(&candidate_keys, request.load_options)?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();

        let seed_mechanics = outgoing_mechanic_keys(self, request.seed)?;
        let mechanic_names = self.mechanic_names(&seed_mechanics)?;
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
                    &seed_mechanics,
                    &mechanic_names,
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
            seed_embedding_unit_key: seed_unit.embedding_unit_key,
            records: ranked,
        }))
    }

    fn mechanic_names(
        &self,
        mechanic_keys: &BTreeSet<RecordKey>,
    ) -> Result<BTreeMap<RecordKey, String>, SearchError> {
        Ok(self
            .get_records_with_options(
                &mechanic_keys.iter().cloned().collect::<Vec<_>>(),
                RecordLoadOptions::omit_raw_json(),
            )?
            .into_iter()
            .map(|record| (record.key, record.name))
            .collect())
    }

    fn similar_record_for_hit(
        &self,
        hit: SemanticSearchHit,
        record: PersistedRecord,
        seed_mechanics: &BTreeSet<RecordKey>,
        mechanic_names: &BTreeMap<RecordKey, String>,
        seed_traits: &BTreeSet<String>,
        weights: SimilarScoreWeights,
    ) -> Result<SimilarRecord, SearchError> {
        let candidate_mechanics = outgoing_mechanic_keys(self, &record.key)?;
        let shared_mechanics = seed_mechanics
            .intersection(&candidate_mechanics)
            .map(|key| SimilarSharedMechanic {
                key: key.clone(),
                name: mechanic_names
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
            shared_mechanics.len(),
            shared_traits.len(),
            weights,
        );
        Ok(SimilarRecord {
            record,
            score,
            semantic: SimilarRecordSemanticEvidence {
                embedding_unit_key: hit.embedding_unit_key,
                unit_kind: hit.unit_kind,
                label: hit.label,
                distance: hit.distance,
                rank_distance: hit.rank_distance,
            },
            graph: SimilarRecordGraphEvidence {
                shared_mechanics,
                shared_traits,
            },
        })
    }
}

fn select_seed_embedding_unit(
    units: &[RecordEmbeddingUnit],
) -> Result<&RecordEmbeddingUnit, SearchError> {
    units
        .iter()
        .find(|unit| unit.unit_kind == EmbeddingUnitKind::Parent.as_str())
        .or_else(|| units.first())
        .ok_or(SearchError::UnsupportedRetrievalPattern(
            "similar records require stored seed embeddings",
        ))
}

fn outgoing_mechanic_keys(
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
    shared_mechanics: usize,
    shared_traits: usize,
    weights: SimilarScoreWeights,
) -> f64 {
    let semantic = 1.0 / (1.0 + rank_distance.max(0.0));
    let mechanic = capped_ratio(shared_mechanics, 5);
    let traits = capped_ratio(shared_traits, 5);
    weights.semantic.mul_add(
        semantic,
        weights.mechanic.mul_add(mechanic, weights.traits * traits),
    )
}

fn capped_ratio(count: usize, cap: usize) -> f64 {
    (count.min(cap) as f64) / (cap as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similar_score_rewards_graph_overlap_when_semantic_distance_is_close() {
        let weights = SimilarScoreWeights::default();
        let plain = similar_score(0.20, 0, 0, weights);
        let connected = similar_score(0.21, 3, 2, weights);

        assert!(connected > plain);
    }

    #[test]
    fn similar_score_can_disable_graph_and_trait_weights() {
        let weights = SimilarScoreWeights {
            semantic: 1.0,
            mechanic: 0.0,
            traits: 0.0,
        };

        let closer = similar_score(0.20, 0, 0, weights);
        let connected = similar_score(0.21, 5, 5, weights);

        assert!(closer > connected);
    }

    #[test]
    fn select_seed_embedding_prefers_parent_unit() {
        let units = vec![
            unit("records:test#heading", "heading_section"),
            unit("records:test#parent", "parent"),
        ];

        let selected = select_seed_embedding_unit(&units).expect("unit should be selected");
        assert_eq!(selected.embedding_unit_key, "records:test#parent");
    }

    fn unit(key: &str, unit_kind: &str) -> RecordEmbeddingUnit {
        RecordEmbeddingUnit {
            embedding_unit_key: key.to_string(),
            record_key: RecordKey::parse("records:test").expect("record key"),
            unit_kind: unit_kind.to_string(),
            label: None,
            ordinal: 0,
            vector: vec![0.1, 0.2],
        }
    }
}
