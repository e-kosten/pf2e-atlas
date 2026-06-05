use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_index::{
    GraphReferenceEdge, RecordLoadError, ReferenceEdgeDirection, ReferenceReadIndex,
};
use atlas_record::AtlasRecord;

use crate::{AtlasRetrievalService, SearchError};
use crate::{GetRecordRequest, GetRecordsRequest, RecordRetrieval};

pub const DEFAULT_GRAPH_OUTGOING_LIMIT: usize = 8;
pub const DEFAULT_GRAPH_BACKLINK_LIMIT: usize = 0;
pub const DEFAULT_GRAPH_USES_LIMIT: usize = 25;
pub const MAX_GRAPH_CONTEXT_LIMIT: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphContextRequest {
    pub seed: RecordKey,
    pub outgoing_limit: usize,
    pub backlink_limit: usize,
}

impl GraphContextRequest {
    pub fn new(seed: RecordKey) -> Self {
        Self {
            seed,
            outgoing_limit: DEFAULT_GRAPH_OUTGOING_LIMIT,
            backlink_limit: DEFAULT_GRAPH_BACKLINK_LIMIT,
        }
    }

    pub fn uses(seed: RecordKey) -> Self {
        Self {
            seed,
            outgoing_limit: 0,
            backlink_limit: DEFAULT_GRAPH_USES_LIMIT,
        }
    }

    pub fn with_outgoing_limit(mut self, limit: usize) -> Self {
        self.outgoing_limit = limit;
        self
    }

    pub fn with_backlink_limit(mut self, limit: usize) -> Self {
        self.backlink_limit = limit;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphContextResult {
    pub seed: AtlasRecord,
    pub outgoing: GraphContextSection,
    pub backlinks: GraphContextSection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphContextSection {
    pub records: Vec<AtlasRecord>,
    pub edges: Vec<GraphContextEdge>,
    pub total_records: usize,
    pub total_edges: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphContextEdge {
    pub from: RecordKey,
    pub to: RecordKey,
    pub display_text: Option<String>,
    pub reference_text: String,
    pub source: GraphContextEdgeSource,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphContextEdgeSource {
    pub kind: String,
    pub visibility: String,
    pub relation_kind: String,
}

pub trait GraphRetrieval {
    fn graph_context(
        &self,
        request: GraphContextRequest,
    ) -> Result<Option<GraphContextResult>, SearchError>;
}

impl GraphRetrieval for AtlasRetrievalService {
    fn graph_context(
        &self,
        request: GraphContextRequest,
    ) -> Result<Option<GraphContextResult>, SearchError> {
        validate_graph_context_request(&request)?;
        let Some(seed) = self.get_record(GetRecordRequest {
            record_key: &request.seed,
        })?
        else {
            return Ok(None);
        };
        let outgoing = self.graph_context_section(
            &request.seed,
            ReferenceEdgeDirection::Outgoing,
            request.outgoing_limit,
        )?;
        let backlinks = self.graph_context_section(
            &request.seed,
            ReferenceEdgeDirection::Backlink,
            request.backlink_limit,
        )?;
        Ok(Some(GraphContextResult {
            seed,
            outgoing,
            backlinks,
        }))
    }
}

fn validate_graph_context_request(request: &GraphContextRequest) -> Result<(), SearchError> {
    if request.outgoing_limit > MAX_GRAPH_CONTEXT_LIMIT
        || request.backlink_limit > MAX_GRAPH_CONTEXT_LIMIT
    {
        return Err(SearchError::invalid_search_options(format!(
            "graph context limits must be at most {MAX_GRAPH_CONTEXT_LIMIT}; got outgoing {}, backlinks {}",
            request.outgoing_limit, request.backlink_limit
        )));
    }
    Ok(())
}

impl AtlasRetrievalService {
    fn graph_context_section(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
        record_limit: usize,
    ) -> Result<GraphContextSection, SearchError> {
        if record_limit == 0 {
            return Ok(GraphContextSection {
                records: Vec::new(),
                edges: Vec::new(),
                total_records: 0,
                total_edges: 0,
                truncated: false,
            });
        }

        let raw_edges = reference_edges_for_seed(self.index.as_ref(), seed, direction)?;
        let edges = sorted_unique_graph_edges(raw_edges, direction);
        let total_edges = edges.len();
        let mut neighbor_order = Vec::new();
        let mut seen_neighbors = BTreeSet::new();
        for edge in &edges {
            let neighbor = graph_neighbor_key(edge, direction);
            if seen_neighbors.insert(neighbor.clone()) {
                neighbor_order.push(neighbor.clone());
            }
        }
        let total_records = neighbor_order.len();
        let retained_keys = neighbor_order
            .into_iter()
            .take(record_limit)
            .collect::<Vec<_>>();
        let mut records_by_key = self
            .get_records(GetRecordsRequest {
                record_keys: &retained_keys,
            })?
            .into_iter()
            .map(|record| (record.identity.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let records = retained_records(&retained_keys, &mut records_by_key)?;
        let retained_key_set = records
            .iter()
            .map(|record| record.identity.key.clone())
            .collect::<BTreeSet<_>>();
        let retained_edges = edges
            .into_iter()
            .filter(|edge| retained_key_set.contains(graph_neighbor_key(edge, direction)))
            .collect::<Vec<_>>();

        Ok(GraphContextSection {
            records,
            edges: retained_edges,
            total_records,
            total_edges,
            truncated: total_records > record_limit,
        })
    }
}

fn reference_edges_for_seed<I>(
    index: &I,
    seed: &RecordKey,
    direction: ReferenceEdgeDirection,
) -> Result<Vec<GraphReferenceEdge>, SearchError>
where
    I: ReferenceReadIndex + ?Sized,
{
    index
        .reference_edges_for_seed(seed, direction)
        .map_err(SearchError::from_record_load)
}

fn retained_records(
    retained_keys: &[RecordKey],
    records_by_key: &mut BTreeMap<RecordKey, AtlasRecord>,
) -> Result<Vec<AtlasRecord>, SearchError> {
    let mut records = Vec::with_capacity(retained_keys.len());
    for key in retained_keys {
        let Some(record) = records_by_key.remove(key) else {
            return Err(SearchError::from_record_load(RecordLoadError::InvalidData(
                format!(
                    "graph neighbor record `{key}` was referenced by an edge but was not found"
                ),
            )));
        };
        records.push(record);
    }
    Ok(records)
}

fn sorted_unique_graph_edges(
    raw_edges: Vec<GraphReferenceEdge>,
    direction: ReferenceEdgeDirection,
) -> Vec<GraphContextEdge> {
    let mut edges = raw_edges
        .into_iter()
        .map(|edge| GraphContextEdge {
            from: edge.from_record_key,
            to: edge.to_record_key,
            display_text: edge.display_text,
            reference_text: edge.reference_text,
            source: GraphContextEdgeSource {
                kind: edge.source_kind.as_str().to_string(),
                visibility: edge.visibility.as_str().to_string(),
                relation_kind: edge.relation_kind.as_str().to_string(),
            },
        })
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        edge_sort_label(left)
            .cmp(edge_sort_label(right))
            .then_with(|| {
                graph_neighbor_key(left, direction).cmp(graph_neighbor_key(right, direction))
            })
            .then_with(|| left.reference_text.cmp(&right.reference_text))
            .then_with(|| left.from.cmp(&right.from))
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.source.cmp(&right.source))
    });
    edges.dedup();
    edges
}

fn edge_sort_label(edge: &GraphContextEdge) -> &str {
    edge.display_text.as_deref().unwrap_or("")
}

fn graph_neighbor_key(edge: &GraphContextEdge, direction: ReferenceEdgeDirection) -> &RecordKey {
    match direction {
        ReferenceEdgeDirection::Outgoing => &edge.to,
        ReferenceEdgeDirection::Backlink => &edge.from,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_record::{ContentSourceKind, ContentVisibility, ReferenceRelationKind};

    fn key(value: &str) -> RecordKey {
        RecordKey::parse(value).expect("record key should parse")
    }

    fn edge(
        from: &str,
        to: &str,
        display_text: Option<&str>,
        reference_text: &str,
    ) -> GraphReferenceEdge {
        GraphReferenceEdge {
            from_record_key: key(from),
            to_record_key: key(to),
            display_text: display_text.map(str::to_string),
            reference_text: reference_text.to_string(),
            relation_kind: ReferenceRelationKind::Reference,
            source_kind: ContentSourceKind::Description,
            visibility: ContentVisibility::Public,
        }
    }

    #[test]
    fn sorted_unique_graph_edges_collapses_exact_duplicates() {
        let edges = sorted_unique_graph_edges(
            vec![
                edge("actions:seed", "actions:target", Some("Grab"), "same"),
                edge("actions:seed", "actions:target", Some("Grab"), "same"),
                edge("actions:seed", "actions:target", Some("Grab"), "different"),
            ],
            ReferenceEdgeDirection::Outgoing,
        );

        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].reference_text, "different");
        assert_eq!(edges[1].reference_text, "same");
    }

    #[test]
    fn sorted_unique_graph_edges_orders_by_label_neighbor_and_reference_text() {
        let edges = sorted_unique_graph_edges(
            vec![
                edge("actions:seed", "actions:targetB", Some("B"), "c"),
                edge("actions:seed", "actions:targetA", None, "z"),
                edge("actions:seed", "actions:targetA", Some("A"), "b"),
                edge("actions:seed", "actions:targetA", Some("A"), "a"),
            ],
            ReferenceEdgeDirection::Outgoing,
        );

        assert_eq!(
            edges
                .iter()
                .map(|edge| edge.reference_text.as_str())
                .collect::<Vec<_>>(),
            vec!["z", "a", "b", "c"]
        );
    }

    #[test]
    fn retained_records_errors_when_neighbor_hydration_is_missing() {
        let retained = vec![key("actions:missing")];
        let mut records = BTreeMap::new();

        let error = retained_records(&retained, &mut records).expect_err("missing record fails");

        assert!(error.to_string().contains("graph neighbor record"));
    }

    #[test]
    fn graph_context_request_rejects_limits_above_product_max() {
        let seed = key("actions:seed");

        let error = validate_graph_context_request(
            &GraphContextRequest::new(seed.clone())
                .with_outgoing_limit(MAX_GRAPH_CONTEXT_LIMIT + 1),
        )
        .expect_err("oversized outgoing limit should be rejected");

        assert_eq!(error.kind(), crate::SearchErrorKind::InvalidOptions);
        assert!(
            error
                .to_string()
                .contains("graph context limits must be at most")
        );

        validate_graph_context_request(
            &GraphContextRequest::uses(seed).with_backlink_limit(MAX_GRAPH_CONTEXT_LIMIT),
        )
        .expect("max backlink limit should be accepted");
    }
}
