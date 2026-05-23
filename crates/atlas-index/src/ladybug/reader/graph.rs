use atlas_domain::RecordKey;
use atlas_record::{ReferenceEdge, RemasterLink};

use crate::{GraphReferenceEdge, ReferenceEdgeDirection};

use super::row::{
    graph_edge_from_row, query_rows, reference_edge_from_row, remaster_link_from_row,
};
use super::{LadybugIndexReader, LadybugIndexReaderError, string_literal};

impl LadybugIndexReader {
    pub(crate) fn reference_edges_for_seed_impl(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, LadybugIndexReaderError> {
        let (pattern, from_index, to_index) = match direction {
            ReferenceEdgeDirection::Outgoing => (
                format!(
                    "MATCH (from:Record {{record_key: {}}})-[edge:REFERENCES]->(to:Record)",
                    string_literal(&seed.to_string())
                ),
                0,
                1,
            ),
            ReferenceEdgeDirection::Backlink => (
                format!(
                    "MATCH (from:Record)-[edge:REFERENCES]->(to:Record {{record_key: {}}})",
                    string_literal(&seed.to_string())
                ),
                0,
                1,
            ),
        };
        let sql = format!(
            "{pattern}
             WHERE edge.visibility = 'public'
             RETURN from.record_key, to.record_key, edge.display_text, edge.reference_text,
                    edge.source_kind, edge.visibility;"
        );
        query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| graph_edge_from_row(row, from_index, to_index))
            .collect()
    }

    pub(crate) fn reference_edges_impl(
        &self,
    ) -> Result<Vec<ReferenceEdge>, LadybugIndexReaderError> {
        query_rows(
            &self.connection,
            "MATCH (from:Record)-[edge:REFERENCES]->(to:Record)
             RETURN from.record_key, to.record_key, edge.display_text, edge.reference_text,
                    edge.source_kind, edge.visibility
             ORDER BY from.record_key, to.record_key, edge.reference_text;",
        )?
        .iter()
        .map(|row| reference_edge_from_row(row))
        .collect()
    }

    pub(crate) fn remaster_links_impl(&self) -> Result<Vec<RemasterLink>, LadybugIndexReaderError> {
        query_rows(
            &self.connection,
            "MATCH (legacy:Record)-[edge:REMASTERED_BY]->(remaster:Record)
             RETURN remaster.record_key, legacy.record_key, edge.source_kind, edge.source_ref
             ORDER BY legacy.record_key, remaster.record_key;",
        )?
        .iter()
        .map(|row| remaster_link_from_row(row))
        .collect()
    }
}
