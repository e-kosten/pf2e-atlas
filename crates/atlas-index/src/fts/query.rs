use std::collections::{BTreeMap, BTreeSet};

use crate::schema_inventory::TABLE_RECORDS_FTS;
use crate::sqlite::raw_sql::SqlBindValue;
use atlas_domain::{RecordKey, SearchFilterNode};
use diesel::sql_types::{Double, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::filters::{FilterCompileError, SqliteEligibleRecordKeyset};
use crate::fts::ranking::{
    FtsDocument, FtsDocumentHit, FtsMatchTier, adjusted_rank, compare_fts_document_hits,
    normalize_text, tokenize_query,
};
use crate::sqlite::raw_sql::{RecordKeyRow, bind_sql_query};
use crate::sqlite::{FtsColumnWeights, FtsQuery, FtsSearchHit, FtsSearchLane};

pub(crate) fn query_weighted_fts_index(
    connection: &mut SqliteConnection,
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
            record_key: hit.record_key,
            rank: hit.rank,
            lane: FtsSearchLane::Mixed,
            lane_rank: (index + 1) as u32,
            title_alias_texts: title_alias_texts(&hit.document),
        })
        .collect())
}

pub(crate) fn query_precision_fts_index(
    connection: &mut SqliteConnection,
    fts_query: &FtsQuery,
    filter: Option<&SearchFilterNode>,
    limit: u32,
) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
    let candidate_limit = rerank_candidate_limit(limit);
    let mut hits = Vec::new();
    hits.extend(query_precision_fts_lane(
        connection,
        fts_query,
        filter,
        candidate_limit,
        FtsSearchLane::TitleAlias,
        &["title", "aliases"],
        precision_title_alias_weights(),
    )?);
    hits.extend(query_precision_fts_lane(
        connection,
        fts_query,
        filter,
        candidate_limit,
        FtsSearchLane::Facet,
        &["traits", "taxonomy_terms"],
        precision_facet_weights(),
    )?);
    hits = best_precision_hit_per_record(hits);
    hits.sort_by(compare_precision_hits);
    hits.truncate(limit as usize);
    Ok(hits)
}

fn best_precision_hit_per_record(hits: Vec<FtsSearchHit>) -> Vec<FtsSearchHit> {
    let mut best_by_record = BTreeMap::<RecordKey, FtsSearchHit>::new();
    for hit in hits {
        match best_by_record.get(&hit.record_key) {
            Some(existing) if compare_precision_hits(existing, &hit).is_le() => {}
            _ => {
                best_by_record.insert(hit.record_key.clone(), hit);
            }
        }
    }
    best_by_record.into_values().collect()
}

fn compare_precision_hits(left: &FtsSearchHit, right: &FtsSearchHit) -> std::cmp::Ordering {
    left.rank
        .total_cmp(&right.rank)
        .then_with(|| left.lane_rank.cmp(&right.lane_rank))
        .then_with(|| left.lane.as_str().cmp(right.lane.as_str()))
        .then_with(|| left.record_key.cmp(&right.record_key))
}

fn query_precision_fts_lane(
    connection: &mut SqliteConnection,
    fts_query: &FtsQuery,
    filter: Option<&SearchFilterNode>,
    limit: u32,
    lane: FtsSearchLane,
    columns: &[&str],
    weights: FtsColumnWeights,
) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
    let mut hits = query_fts_documents(
        connection,
        &scoped_match_query(columns, &fts_query.as_disjunction_match_query()),
        filter,
        limit,
        weights,
        FtsMatchTier::Fallback,
    )?;

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
            record_key: hit.record_key,
            rank: hit.rank,
            lane,
            lane_rank: (index + 1) as u32,
            title_alias_texts: title_alias_texts(&hit.document),
        })
        .collect())
}

fn scoped_match_query(columns: &[&str], match_query: &str) -> String {
    format!("{{{}}} : ({match_query})", columns.join(" "))
}

fn precision_title_alias_weights() -> FtsColumnWeights {
    FtsColumnWeights {
        title: 8.0,
        aliases: 8.0,
        traits: 0.0,
        taxonomy_terms: 0.0,
        constraint_terms: 0.0,
        mechanic_terms: 0.0,
        source_terms: 0.0,
        metric_terms: 0.0,
        headings: 0.0,
        body: 0.0,
        facts: 0.0,
        reference_terms: 0.0,
        embedded_content: 0.0,
    }
}

fn precision_facet_weights() -> FtsColumnWeights {
    FtsColumnWeights {
        title: 0.0,
        aliases: 0.0,
        traits: 4.0,
        taxonomy_terms: 2.5,
        constraint_terms: 0.0,
        mechanic_terms: 0.0,
        source_terms: 0.0,
        metric_terms: 0.0,
        headings: 0.0,
        body: 0.0,
        facts: 0.0,
        reference_terms: 0.0,
        embedded_content: 0.0,
    }
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
    connection: &mut SqliteConnection,
    fts_query: &FtsQuery,
    filter: Option<&SearchFilterNode>,
    limit: u32,
) -> Result<Vec<RecordKey>, FilterCompileError> {
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|builder| {
            let query_placeholder = builder.push_text(fts_query.as_disjunction_match_query());
            let limit_placeholder = builder.push_integer(i64::from(limit));
            format!(
                "SELECT f.record_key
         FROM {fts_table} f
         WHERE {fts_table} MATCH {query_placeholder}
           AND f.record_key IN (SELECT record_key FROM eligible)
         ORDER BY f.record_key ASC
         LIMIT {limit_placeholder}",
                fts_table = TABLE_RECORDS_FTS,
            )
        });

    read_record_key_query(connection, &query.sql, &query.parameters)
}

pub(crate) fn query_fts_candidate_record_keys(
    connection: &mut SqliteConnection,
    fts_query: &FtsQuery,
    candidate_keys: &[RecordKey],
) -> Result<Vec<RecordKey>, FilterCompileError> {
    if candidate_keys.is_empty() {
        return Ok(Vec::new());
    }
    let mut parameters = vec![SqlBindValue::Text(fts_query.as_disjunction_match_query())];
    let query_placeholder = "?1";
    let candidate_placeholders = candidate_keys
        .iter()
        .map(|key| {
            parameters.push(SqlBindValue::Text(key.to_string()));
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
    connection: &mut SqliteConnection,
    match_query: &str,
    filter: Option<&SearchFilterNode>,
    limit: u32,
    weights: FtsColumnWeights,
    tier: FtsMatchTier,
) -> Result<Vec<FtsDocumentHit>, FilterCompileError> {
    let query = SqliteEligibleRecordKeyset::new(filter).compile()?.with_eligible_cte(
        |builder| {
            let query_placeholder = builder.push_text(match_query.to_string());
            let limit_placeholder = builder.push_integer(i64::from(limit));
            format!(
                "SELECT f.record_key,
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
         WHERE {fts_table} MATCH {query_placeholder}
           AND f.record_key IN (SELECT record_key FROM eligible)
         ORDER BY rank ASC, f.record_key ASC
         LIMIT {limit_placeholder}",
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
            )
        },
    );

    bind_sql_query(query.sql, &query.parameters)
        .load::<FtsDocumentRow>(connection)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .into_iter()
        .map(|row| {
            Ok(FtsDocumentHit {
                record_key: RecordKey::parse(&row.record_key)
                    .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))?,
                base_rank: row.rank,
                rank: row.rank,
                tier,
                document: FtsDocument {
                    title: row.title,
                    aliases: row.aliases,
                    traits: row.traits,
                    taxonomy_terms: row.taxonomy_terms,
                    constraint_terms: row.constraint_terms,
                    mechanic_terms: row.mechanic_terms,
                    source_terms: row.source_terms,
                    metric_terms: row.metric_terms,
                    headings: row.headings,
                    body: row.body,
                    facts: row.facts,
                    reference_terms: row.reference_terms,
                    embedded_content: row.embedded_content,
                    record_family: row.record_family,
                    foundry_record_type: row.foundry_record_type,
                },
            })
        })
        .collect()
}

fn read_record_key_query(
    connection: &mut SqliteConnection,
    sql: &str,
    parameters: &[SqlBindValue],
) -> Result<Vec<RecordKey>, FilterCompileError> {
    bind_sql_query(sql.to_string(), parameters)
        .load::<RecordKeyRow>(connection)
        .map_err(|error| FilterCompileError::QueryFailed(error.to_string()))?
        .into_iter()
        .map(|row| {
            RecordKey::parse(&row.record_key)
                .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
        })
        .collect()
}

#[derive(QueryableByName)]
struct FtsDocumentRow {
    #[diesel(sql_type = Text)]
    record_key: String,
    #[diesel(sql_type = Double)]
    rank: f64,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Text)]
    aliases: String,
    #[diesel(sql_type = Text)]
    traits: String,
    #[diesel(sql_type = Text)]
    taxonomy_terms: String,
    #[diesel(sql_type = Text)]
    constraint_terms: String,
    #[diesel(sql_type = Text)]
    mechanic_terms: String,
    #[diesel(sql_type = Text)]
    source_terms: String,
    #[diesel(sql_type = Text)]
    metric_terms: String,
    #[diesel(sql_type = Text)]
    headings: String,
    #[diesel(sql_type = Text)]
    body: String,
    #[diesel(sql_type = Text)]
    facts: String,
    #[diesel(sql_type = Text)]
    reference_terms: String,
    #[diesel(sql_type = Text)]
    embedded_content: String,
    #[diesel(sql_type = Text)]
    record_family: String,
    #[diesel(sql_type = Text)]
    foundry_record_type: String,
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
