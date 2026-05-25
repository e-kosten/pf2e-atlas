use std::time::Instant;

use crate::{FtsQuery, FtsSearchHit, FtsSearchLane, VectorSearchHit};
use atlas_domain::{RecordKey, SearchFilterNode};

use super::filter::compile_scope;
use super::row::{
    float_at, query_rows, query_rows_traced, record_key_at, string_at, vector_hit_from_row,
};
use super::{
    LadybugIndexReader, LadybugIndexReaderError, list_literal, stable_hash, string_literal,
    trace_ladybug_phase, vector_literal,
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
        let mut hits = Vec::new();
        hits.extend(self.query_search_document_fts_lane(
            "search_document_title_alias_fts",
            FtsSearchLane::TitleAlias,
            fts_query,
            filter,
            limit,
        )?);
        hits.extend(self.query_search_document_fts_lane(
            "search_document_facet_fts",
            FtsSearchLane::Facet,
            fts_query,
            filter,
            limit,
        )?);
        Ok(hits)
    }

    fn query_search_document_fts_lane(
        &self,
        index_name: &str,
        lane: FtsSearchLane,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<FtsSearchHit>, LadybugIndexReaderError> {
        let started_at = Instant::now();
        let scope = compile_scope(filter)?;
        trace_ladybug_phase("ladybug_fts_compile_filter", started_at);
        let sql = format!(
            "CALL QUERY_FTS_INDEX('SearchDocument', {}, {}, top := {})
             WITH node AS doc, score
             MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
             {}
             {}
             RETURN record.record_key, score, doc.title, doc.aliases
             ORDER BY score DESC
             LIMIT {};",
            string_literal(index_name),
            string_literal(&fts_query.as_match_query()),
            limit,
            scope.optional_match_suffix("record"),
            scope.where_clause("record"),
            limit
        );
        let rows = query_rows_traced(
            &self.connection,
            &sql,
            &format!("ladybug_fts_{index_name}_query"),
        )?;
        let started_at = Instant::now();
        let hits = rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                Ok(FtsSearchHit {
                    record_key: record_key_at(row, 0)?,
                    rank: float_at(row, 1)?,
                    lane,
                    lane_rank: (index + 1) as u32,
                    title_alias_texts: title_alias_texts(string_at(row, 2)?, string_at(row, 3)?),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        trace_ladybug_phase(&format!("ladybug_fts_{index_name}_decode"), started_at);
        Ok(hits)
    }

    pub(crate) fn query_fts_candidate_record_keys_impl(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, LadybugIndexReaderError> {
        if candidate_keys.is_empty() {
            return Ok(Vec::new());
        }
        let mut keys = Vec::new();
        for index_name in [
            "search_document_title_alias_fts",
            "search_document_facet_fts",
        ] {
            let sql = format!(
                "CALL QUERY_FTS_INDEX('SearchDocument', {}, {}, top := {})
                 WITH node AS doc, score
                 MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
                 WHERE record.record_key IN {}
                 RETURN DISTINCT record.record_key;",
                string_literal(index_name),
                string_literal(&fts_query.as_match_query()),
                candidate_keys.len(),
                list_literal(candidate_keys.iter().map(ToString::to_string))
            );
            keys.extend(
                query_rows(&self.connection, &sql)?
                    .iter()
                    .map(|row| record_key_at(row, 0))
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        keys.sort();
        keys.dedup();
        Ok(keys)
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
        let started_at = Instant::now();
        let scope = compile_scope(filter)?;
        trace_ladybug_phase("ladybug_vector_compile_filter", started_at);
        let graph_name = if filter.is_some() {
            let projection = scope.embedding_projection_query("record", "embedding");
            let name = format!("eligible_embeddings_{}", stable_hash(&projection));
            let started_at = Instant::now();
            let _ = self.connection.query(&format!(
                "CALL DROP_PROJECTED_GRAPH({});",
                string_literal(&name)
            ));
            trace_ladybug_phase("ladybug_vector_drop_projection", started_at);
            let started_at = Instant::now();
            self.connection
                .query(&format!(
                    "CALL PROJECT_GRAPH_CYPHER({}, {});",
                    string_literal(&name),
                    string_literal(&projection)
                ))
                .map_err(|error| {
                    LadybugIndexReaderError::Query(format!(
                        "{error}; query: CALL PROJECT_GRAPH_CYPHER({}, {});",
                        string_literal(&name),
                        string_literal(&projection)
                    ))
                })?;
            trace_ladybug_phase("ladybug_vector_create_projection", started_at);
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
        let rows = query_rows_traced(&self.connection, &sql, "ladybug_vector_query_index")?;
        let started_at = Instant::now();
        let hits = rows
            .iter()
            .map(|row| vector_hit_from_row(row))
            .collect::<Result<Vec<_>, _>>()?;
        trace_ladybug_phase("ladybug_vector_decode_hits", started_at);
        Ok(hits)
    }
}

fn title_alias_texts(title: String, aliases: String) -> Vec<String> {
    std::iter::once(title.as_str())
        .chain(aliases.lines())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}
