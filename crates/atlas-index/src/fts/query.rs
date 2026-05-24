use std::collections::BTreeSet;

use atlas_artifact::schema::TABLE_RECORDS_FTS;
use atlas_domain::{RecordKey, SearchFilterNode};
use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

use crate::filters::{FilterCompileError, compile_eligible_records_query};
use crate::fts::ranking::{
    FtsDocument, FtsDocumentHit, FtsMatchTier, adjusted_rank, compare_fts_document_hits,
    normalize_text, tokenize_query,
};
use crate::sqlite::{FtsColumnWeights, FtsQuery, FtsSearchHit, FtsSearchLane};

pub(crate) fn query_fts_index(
    connection: &Connection,
    fts_query: &FtsQuery,
    filter: Option<&SearchFilterNode>,
    limit: u32,
    weights: FtsColumnWeights,
) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
    let candidate_limit = rerank_candidate_limit(limit);
    let mut hits = Vec::new();
    let strict_match_query = fts_query.as_conjunction_match_query();
    if !strict_match_query.is_empty() {
        hits.extend(query_fts_documents(
            connection,
            &strict_match_query,
            filter,
            candidate_limit,
            weights,
            FtsMatchTier::Strict,
        )?);
    }

    let seen = hits
        .iter()
        .map(|hit| hit.record_key.clone())
        .collect::<BTreeSet<_>>();
    hits.extend(
        query_fts_documents(
            connection,
            &fts_query.as_disjunction_match_query(),
            filter,
            candidate_limit,
            weights,
            FtsMatchTier::Fallback,
        )?
        .into_iter()
        .filter(|hit| !seen.contains(&hit.record_key)),
    );

    let tokens = tokenize_query(&fts_query.tokens.join(" "));
    let query_phrase = normalize_text(&fts_query.tokens.join(" "));
    for hit in &mut hits {
        hit.rank = adjusted_rank(&tokens, &query_phrase, hit, weights);
    }
    hits.sort_by(compare_fts_document_hits);
    hits.truncate(limit as usize);
    Ok(hits
        .into_iter()
        .enumerate()
        .map(|(index, hit)| FtsSearchHit {
            title_alias_texts: title_alias_texts(&hit.document),
            record_key: hit.record_key,
            rank: hit.rank,
            lane: FtsSearchLane::Mixed,
            lane_rank: (index + 1) as u32,
        })
        .collect())
}

fn title_alias_texts(document: &FtsDocument) -> Vec<String> {
    std::iter::once(document.title.as_str())
        .chain(document.aliases.lines())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn rerank_candidate_limit(limit: u32) -> u32 {
    limit.max(limit.saturating_mul(5).clamp(50, 1_000))
}

pub(crate) fn query_fts_record_keys(
    connection: &Connection,
    fts_query: &FtsQuery,
    filter: Option<&SearchFilterNode>,
    limit: u32,
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    parameters.push(Value::Text(fts_query.as_disjunction_match_query()));
    parameters.push(Value::Integer(i64::from(limit)));
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible_sql})
         SELECT f.record_key
         FROM {fts_table} f
         WHERE {fts_table} MATCH ?{query_index}
           AND f.record_key IN (SELECT record_key FROM eligible)
         ORDER BY f.record_key ASC
         LIMIT ?{limit_index}",
        eligible_sql = eligible.sql,
        fts_table = TABLE_RECORDS_FTS,
        query_index = parameters.len() - 1,
        limit_index = parameters.len(),
    );

    read_record_key_query(connection, &sql, &parameters)
}

pub(crate) fn query_fts_candidate_record_keys(
    connection: &Connection,
    fts_query: &FtsQuery,
    candidate_keys: &[RecordKey],
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let mut parameters = vec![Value::Text(fts_query.as_disjunction_match_query())];
    let query_placeholder = "?1";
    let candidate_placeholders = candidate_keys
        .iter()
        .map(|key| {
            parameters.push(Value::Text(key.to_string()));
            format!("?{}", parameters.len())
        })
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT f.record_key
         FROM {fts_table} f
         WHERE {fts_table} MATCH {query_placeholder}
           AND f.record_key IN ({candidate_placeholders})
         ORDER BY f.record_key ASC",
        fts_table = TABLE_RECORDS_FTS,
    );

    read_record_key_query(connection, &sql, &parameters)
}

fn query_fts_documents(
    connection: &Connection,
    match_query: &str,
    filter: Option<&SearchFilterNode>,
    limit: u32,
    weights: FtsColumnWeights,
    tier: FtsMatchTier,
) -> Result<Vec<FtsDocumentHit>, FilterCompileError> {
    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    parameters.push(Value::Text(match_query.to_string()));
    parameters.push(Value::Integer(i64::from(limit)));
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible_sql})
         SELECT f.record_key,
                bm25({fts_table}, 0.0, {title}, {aliases}, {traits}, {taxonomy_terms}, {constraint_terms}, {mechanic_terms}, {source_terms}, {metric_terms}, {headings}, {body}, {facts}, {reference_terms}, {embedded_content}) AS rank,
                f.title,
                f.aliases,
                f.traits,
                f.taxonomy_terms,
                f.constraint_terms,
                f.mechanic_terms,
                f.source_terms,
                f.metric_terms,
                f.headings,
                f.body,
                f.facts,
                f.reference_terms,
                f.embedded_content,
                r.record_family,
                r.foundry_record_type
         FROM {fts_table} f
         JOIN records r ON r.record_key = f.record_key
         WHERE {fts_table} MATCH ?{query_index}
           AND f.record_key IN (SELECT record_key FROM eligible)
         ORDER BY rank ASC, f.record_key ASC
         LIMIT ?{limit_index}",
        eligible_sql = eligible.sql,
        fts_table = TABLE_RECORDS_FTS,
        title = weights.title,
        aliases = weights.aliases,
        traits = weights.traits,
        taxonomy_terms = weights.taxonomy_terms,
        constraint_terms = weights.constraint_terms,
        mechanic_terms = weights.mechanic_terms,
        source_terms = weights.source_terms,
        metric_terms = weights.metric_terms,
        headings = weights.headings,
        body = weights.body,
        facts = weights.facts,
        reference_terms = weights.reference_terms,
        embedded_content = weights.embedded_content,
        query_index = parameters.len() - 1,
        limit_index = parameters.len(),
    );

    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?;
    statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                FtsDocument {
                    title: row.get(2)?,
                    aliases: row.get(3)?,
                    traits: row.get(4)?,
                    taxonomy_terms: row.get(5)?,
                    constraint_terms: row.get(6)?,
                    mechanic_terms: row.get(7)?,
                    source_terms: row.get(8)?,
                    metric_terms: row.get(9)?,
                    headings: row.get(10)?,
                    body: row.get(11)?,
                    facts: row.get(12)?,
                    reference_terms: row.get(13)?,
                    embedded_content: row.get(14)?,
                    record_family: row.get(15)?,
                    foundry_record_type: row.get(16)?,
                },
            ))
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
                .and_then(|(record_key, base_rank, document)| {
                    Ok(FtsDocumentHit {
                        record_key: RecordKey::parse(&record_key)
                            .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))?,
                        base_rank,
                        rank: base_rank,
                        tier,
                        document,
                    })
                })
        })
        .collect()
}

fn read_record_key_query(
    connection: &Connection,
    sql: &str,
    parameters: &[Value],
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?;
    statement
        .query_map(params_from_iter(parameters.iter()), |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .map(|row| {
            row.map_err(|error| FilterCompileError::QueryFailed(error.to_string()))
                .and_then(|record_key| {
                    RecordKey::parse(&record_key)
                        .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::rerank_candidate_limit;

    #[test]
    fn rerank_candidate_limit_expands_small_windows_without_capping_public_limit() {
        assert_eq!(rerank_candidate_limit(1), 50);
        assert_eq!(rerank_candidate_limit(250), 1_000);
        assert_eq!(rerank_candidate_limit(1_200), 1_200);
    }
}
