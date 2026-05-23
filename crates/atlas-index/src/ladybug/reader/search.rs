use crate::{FtsQuery, FtsSearchHit, VectorSearchHit};
use atlas_domain::{RecordKey, SearchFilterNode};

use super::filter::compile_scope;
use super::row::{float_at, query_rows, record_key_at, vector_hit_from_row};
use super::{
    LadybugIndexReader, LadybugIndexReaderError, list_literal, stable_hash, string_literal,
    vector_literal,
};

impl LadybugIndexReader {
    pub(crate) fn query_fts_index_impl(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<FtsSearchHit>, LadybugIndexReaderError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let scope = compile_scope(filter)?;
        let sql = format!(
            "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', {}, top := {})
             WITH node AS doc, score
             MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
             {}
             {}
             RETURN record.record_key, score
             ORDER BY score DESC
             LIMIT {};",
            string_literal(&fts_query.as_match_query()),
            limit,
            scope.optional_match_suffix("record"),
            scope.where_clause("record"),
            limit
        );
        query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| {
                Ok(FtsSearchHit {
                    record_key: record_key_at(row, 0)?,
                    rank: float_at(row, 1)?,
                })
            })
            .collect()
    }

    pub(crate) fn query_fts_candidate_record_keys_impl(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, LadybugIndexReaderError> {
        if candidate_keys.is_empty() {
            return Ok(Vec::new());
        }
        let sql = format!(
            "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', {}, top := {})
             WITH node AS doc, score
             MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
             WHERE record.record_key IN {}
             RETURN DISTINCT record.record_key;",
            string_literal(&fts_query.as_match_query()),
            candidate_keys.len(),
            list_literal(candidate_keys.iter().map(ToString::to_string))
        );
        query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| record_key_at(row, 0))
            .collect()
    }

    pub(crate) fn query_vector_index_impl(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, LadybugIndexReaderError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let scope = compile_scope(filter)?;
        let graph_name = if filter.is_some() {
            let projection = format!(
                "{} MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit)
                 {} RETURN embedding",
                scope.optional_match_prefix(),
                scope.where_clause("record")
            );
            let name = format!("eligible_embeddings_{}", stable_hash(&projection));
            let _ = self.connection.query(&format!(
                "CALL PROJECT_GRAPH_CYPHER({}, {});",
                string_literal(&name),
                string_literal(&projection)
            ));
            name
        } else {
            "EmbeddingUnit".to_string()
        };
        let unit_filter = if include_child_units {
            String::new()
        } else {
            "AND embedding.unit_kind = 'parent'".to_string()
        };
        let sql = format!(
            "CALL QUERY_VECTOR_INDEX({}, 'embedding_hnsw', CAST({}, 'FLOAT[{}]'), {}, efs := 50)
             WITH node AS embedding, distance
             MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding)
             WHERE record.is_default_visible {unit_filter}
             RETURN record.record_key, embedding.embedding_unit_key, embedding.unit_kind,
                    embedding.label, distance
             ORDER BY distance
             LIMIT {};",
            string_literal(&graph_name),
            vector_literal(query_vector),
            query_vector.len(),
            limit,
            limit
        );
        query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| vector_hit_from_row(row))
            .collect()
    }
}
