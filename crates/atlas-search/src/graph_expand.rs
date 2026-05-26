use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_index::{RecordLoadOptions, ReferenceEdgeDirection};
use atlas_record::PersistedRecord;

use crate::{AtlasRetrievalService, SearchError, SemanticSearchHit, SemanticSearchMode};

#[derive(Debug, Clone, PartialEq)]
pub struct GraphExpandRequest<'a> {
    pub query: &'a str,
    pub filter: Option<&'a SearchFilterNode>,
    pub semantic_limit: u32,
    pub mechanic_limit: usize,
    pub min_support: usize,
    pub expansion_limit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphExpandResult {
    pub query: String,
    pub semantic_seeds: Vec<GraphExpandSemanticSeed>,
    pub mechanics: Vec<GraphExpandMechanic>,
    pub expanded_records: Vec<GraphExpandRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphExpandSemanticSeed {
    pub record: PersistedRecord,
    pub hit: SemanticSearchHit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphExpandMechanic {
    pub record: PersistedRecord,
    pub support_count: usize,
    pub supported_by: Vec<RecordKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphExpandRecord {
    pub record: PersistedRecord,
    pub is_semantic_seed: bool,
    pub evidence: Vec<GraphExpandEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphExpandEvidence {
    pub mechanic_key: RecordKey,
    pub mechanic_name: String,
    pub support_count: usize,
    pub edge_count: usize,
    pub display_text: Option<String>,
    pub reference_text: String,
}

impl AtlasRetrievalService {
    pub fn graph_expand(
        &mut self,
        request: GraphExpandRequest<'_>,
    ) -> Result<GraphExpandResult, SearchError> {
        let resolved_filter = self.index.resolve_metric_filters(request.filter)?;
        let filter = resolved_filter.as_ref().or(request.filter);
        let semantic_hits = self.semantic(
            request.query,
            filter,
            request.semantic_limit,
            SemanticSearchMode::WeightedChunks,
        )?;
        let semantic_keys = semantic_hits
            .iter()
            .filter_map(|hit| RecordKey::parse(&hit.record_key).ok())
            .collect::<Vec<_>>();
        let seed_records = self
            .get_records_with_options(&semantic_keys, RecordLoadOptions::omit_raw_json())?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let semantic_seeds = semantic_hits
            .into_iter()
            .filter_map(|hit| {
                let key = RecordKey::parse(&hit.record_key).ok()?;
                let record = seed_records.get(&key)?.clone();
                Some(GraphExpandSemanticSeed { record, hit })
            })
            .collect::<Vec<_>>();
        let seed_key_set = semantic_seeds
            .iter()
            .map(|seed| seed.record.key.clone())
            .collect::<BTreeSet<_>>();

        let mechanic_candidates = mechanic_candidates_for_seeds(self, &semantic_seeds)?;
        let retained_mechanic_candidates = rank_mechanic_candidates(mechanic_candidates)
            .into_iter()
            .filter(|candidate| candidate.support_count() >= request.min_support)
            .take(request.mechanic_limit)
            .collect::<Vec<_>>();
        let retained_mechanic_keys = retained_mechanic_candidates
            .iter()
            .map(|candidate| candidate.key.clone())
            .collect::<Vec<_>>();
        let mechanic_records = self
            .get_records_with_options(&retained_mechanic_keys, RecordLoadOptions::omit_raw_json())?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let mechanics = retained_mechanic_keys
            .iter()
            .zip(retained_mechanic_candidates)
            .filter_map(|(key, candidate)| {
                let record = mechanic_records.get(key)?.clone();
                Some(GraphExpandMechanic {
                    record,
                    support_count: candidate.support_count(),
                    supported_by: candidate.supported_by.into_iter().collect(),
                })
            })
            .collect::<Vec<_>>();
        let expanded_records = expanded_records_for_mechanics(
            self,
            &mechanics,
            &seed_key_set,
            filter,
            request.expansion_limit,
        )?;

        Ok(GraphExpandResult {
            query: request.query.to_string(),
            semantic_seeds,
            mechanics,
            expanded_records,
        })
    }
}

fn mechanic_candidates_for_seeds(
    service: &AtlasRetrievalService,
    semantic_seeds: &[GraphExpandSemanticSeed],
) -> Result<BTreeMap<RecordKey, MechanicCandidate>, SearchError> {
    let mut candidates = BTreeMap::<RecordKey, MechanicCandidate>::new();
    for (rank, seed) in semantic_seeds.iter().enumerate() {
        let edges = service
            .index
            .reference_edges_for_seed(&seed.record.key, ReferenceEdgeDirection::Outgoing)?;
        for edge in edges {
            let candidate = candidates
                .entry(edge.to_record_key.clone())
                .or_insert_with(|| MechanicCandidate {
                    key: edge.to_record_key.clone(),
                    supported_by: BTreeSet::new(),
                    best_semantic_rank: rank,
                    edge_count: 0,
                });
            candidate.supported_by.insert(seed.record.key.clone());
            candidate.best_semantic_rank = candidate.best_semantic_rank.min(rank);
            candidate.edge_count += 1;
        }
    }
    Ok(candidates)
}

fn expanded_records_for_mechanics(
    service: &AtlasRetrievalService,
    mechanics: &[GraphExpandMechanic],
    semantic_key_set: &BTreeSet<RecordKey>,
    filter: Option<&SearchFilterNode>,
    limit: usize,
) -> Result<Vec<GraphExpandRecord>, SearchError> {
    let mechanic_names = mechanics
        .iter()
        .map(|mechanic| (mechanic.record.key.clone(), mechanic.record.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let mechanic_support = mechanics
        .iter()
        .map(|mechanic| (mechanic.record.key.clone(), mechanic.support_count))
        .collect::<BTreeMap<_, _>>();
    let mut evidence_by_record =
        BTreeMap::<RecordKey, BTreeMap<RecordKey, EvidenceAccumulator>>::new();
    for mechanic in mechanics {
        let edges = service
            .index
            .reference_edges_for_seed(&mechanic.record.key, ReferenceEdgeDirection::Backlink)?;
        let mut edge_counts = BTreeMap::<RecordKey, usize>::new();
        for edge in &edges {
            *edge_counts.entry(edge.from_record_key.clone()).or_default() += 1;
        }
        for edge in edges {
            let mechanic_name = mechanic_names
                .get(&mechanic.record.key)
                .cloned()
                .unwrap_or_else(|| mechanic.record.key.to_string());
            evidence_by_record
                .entry(edge.from_record_key.clone())
                .or_default()
                .entry(mechanic.record.key.clone())
                .or_insert_with(|| EvidenceAccumulator {
                    mechanic_name,
                    support_count: *mechanic_support.get(&mechanic.record.key).unwrap_or(&0),
                    edge_count: *edge_counts.get(&edge.from_record_key).unwrap_or(&1),
                    display_text: edge.display_text,
                    reference_text: edge.reference_text,
                });
        }
    }
    let mut evidence_by_record = evidence_by_record
        .into_iter()
        .map(|(record_key, by_mechanic)| {
            let evidence = by_mechanic
                .into_iter()
                .map(|(mechanic_key, evidence)| GraphExpandEvidence {
                    mechanic_key,
                    mechanic_name: evidence.mechanic_name,
                    support_count: evidence.support_count,
                    edge_count: evidence.edge_count,
                    display_text: evidence.display_text,
                    reference_text: evidence.reference_text,
                })
                .collect::<Vec<_>>();
            (record_key, evidence)
        })
        .collect::<BTreeMap<_, _>>();
    for evidence in evidence_by_record.values_mut() {
        evidence.sort_by(|left, right| {
            right
                .support_count
                .cmp(&left.support_count)
                .then_with(|| left.mechanic_name.cmp(&right.mechanic_name))
                .then_with(|| left.reference_text.cmp(&right.reference_text))
        });
    }
    if filter.is_some() {
        let candidate_keys = evidence_by_record.keys().cloned().collect::<Vec<_>>();
        let matching_keys = service
            .index
            .filter_record_keys(&candidate_keys, filter)?
            .into_iter()
            .collect::<BTreeSet<_>>();
        evidence_by_record.retain(|key, _| matching_keys.contains(key));
    }
    let retained_keys = rank_expanded_record_keys(&evidence_by_record, semantic_key_set)
        .into_iter()
        .take(limit)
        .collect::<Vec<_>>();
    let records_by_key = service
        .get_records_with_options(&retained_keys, RecordLoadOptions::omit_raw_json())?
        .into_iter()
        .map(|record| (record.key.clone(), record))
        .collect::<BTreeMap<_, _>>();
    Ok(retained_keys
        .into_iter()
        .filter_map(|key| {
            let record = records_by_key.get(&key)?.clone();
            let evidence = evidence_by_record.get(&key)?.clone();
            Some(GraphExpandRecord {
                record,
                is_semantic_seed: semantic_key_set.contains(&key),
                evidence,
            })
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MechanicCandidate {
    key: RecordKey,
    supported_by: BTreeSet<RecordKey>,
    best_semantic_rank: usize,
    edge_count: usize,
}

impl MechanicCandidate {
    fn support_count(&self) -> usize {
        self.supported_by.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceAccumulator {
    mechanic_name: String,
    support_count: usize,
    edge_count: usize,
    display_text: Option<String>,
    reference_text: String,
}

fn rank_mechanic_candidates(
    candidates: BTreeMap<RecordKey, MechanicCandidate>,
) -> Vec<MechanicCandidate> {
    let mut candidates = candidates.into_values().collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .support_count()
            .cmp(&left.support_count())
            .then_with(|| right.edge_count.cmp(&left.edge_count))
            .then_with(|| left.best_semantic_rank.cmp(&right.best_semantic_rank))
            .then_with(|| left.key.cmp(&right.key))
    });
    candidates
}

fn rank_expanded_record_keys(
    evidence_by_record: &BTreeMap<RecordKey, Vec<GraphExpandEvidence>>,
    semantic_key_set: &BTreeSet<RecordKey>,
) -> Vec<RecordKey> {
    let mut keys = evidence_by_record.keys().cloned().collect::<Vec<_>>();
    keys.sort_by(|left, right| {
        let left_evidence = &evidence_by_record[left];
        let right_evidence = &evidence_by_record[right];
        support_weight(right_evidence)
            .cmp(&support_weight(left_evidence))
            .then_with(|| right_evidence.len().cmp(&left_evidence.len()))
            .then_with(|| {
                semantic_key_set
                    .contains(right)
                    .cmp(&semantic_key_set.contains(left))
            })
            .then_with(|| left.cmp(right))
    });
    keys
}

fn support_weight(evidence: &[GraphExpandEvidence]) -> usize {
    evidence
        .iter()
        .map(|evidence| evidence.support_count * evidence.edge_count)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(value: &str) -> RecordKey {
        RecordKey::parse(value).expect("record key should parse")
    }

    #[test]
    fn ranks_mechanic_candidates_by_seed_support_then_edge_count() {
        let mut candidates = BTreeMap::new();
        candidates.insert(
            key("conditions:frightened"),
            MechanicCandidate {
                key: key("conditions:frightened"),
                supported_by: BTreeSet::from([key("records:a"), key("records:b")]),
                best_semantic_rank: 3,
                edge_count: 2,
            },
        );
        candidates.insert(
            key("traits:fear"),
            MechanicCandidate {
                key: key("traits:fear"),
                supported_by: BTreeSet::from([key("records:a")]),
                best_semantic_rank: 0,
                edge_count: 5,
            },
        );

        let ranked = rank_mechanic_candidates(candidates);

        assert_eq!(ranked[0].key, key("conditions:frightened"));
        assert_eq!(ranked[1].key, key("traits:fear"));
    }

    #[test]
    fn ranks_expanded_records_by_support_weight_then_evidence_count() {
        let frightened = key("conditions:frightened");
        let fear = key("traits:fear");
        let mut evidence = BTreeMap::new();
        evidence.insert(
            key("records:a"),
            vec![
                GraphExpandEvidence {
                    mechanic_key: frightened.clone(),
                    mechanic_name: "Frightened".to_string(),
                    support_count: 2,
                    edge_count: 1,
                    display_text: None,
                    reference_text: "a".to_string(),
                },
                GraphExpandEvidence {
                    mechanic_key: fear,
                    mechanic_name: "Fear".to_string(),
                    support_count: 1,
                    edge_count: 1,
                    display_text: None,
                    reference_text: "b".to_string(),
                },
            ],
        );
        evidence.insert(
            key("records:b"),
            vec![GraphExpandEvidence {
                mechanic_key: frightened,
                mechanic_name: "Frightened".to_string(),
                support_count: 4,
                edge_count: 1,
                display_text: None,
                reference_text: "c".to_string(),
            }],
        );

        let ranked = rank_expanded_record_keys(&evidence, &BTreeSet::new());

        assert_eq!(ranked[0], key("records:b"));
        assert_eq!(ranked[1], key("records:a"));
    }

    #[test]
    fn ranks_semantic_seed_after_support_and_evidence_ties() {
        let frightened = key("conditions:frightened");
        let mut evidence = BTreeMap::new();
        evidence.insert(
            key("records:a"),
            vec![GraphExpandEvidence {
                mechanic_key: frightened.clone(),
                mechanic_name: "Frightened".to_string(),
                support_count: 2,
                edge_count: 1,
                display_text: None,
                reference_text: "a".to_string(),
            }],
        );
        evidence.insert(
            key("records:b"),
            vec![GraphExpandEvidence {
                mechanic_key: frightened,
                mechanic_name: "Frightened".to_string(),
                support_count: 2,
                edge_count: 1,
                display_text: None,
                reference_text: "b".to_string(),
            }],
        );

        let ranked = rank_expanded_record_keys(&evidence, &BTreeSet::from([key("records:b")]));

        assert_eq!(ranked[0], key("records:b"));
        assert_eq!(ranked[1], key("records:a"));
    }
}
