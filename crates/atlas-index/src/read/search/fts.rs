mod query;
mod ranking;

use atlas_domain::{RecordKey, SearchFilterNode};

use crate::read::search::filters::FilterCompileError;
use crate::sqlite::SqliteIndexReader;

pub(crate) use query::{
    query_fts_candidate_record_keys, query_fts_record_keys, query_precision_fts_index,
    query_weighted_fts_index,
};
pub(crate) use ranking::is_primary_type_intent_token;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FtsColumnWeights {
    pub title: f64,
    pub aliases: f64,
    pub traits: f64,
    pub taxonomy_terms: f64,
    pub constraint_terms: f64,
    pub mechanic_terms: f64,
    pub source_terms: f64,
    pub metric_terms: f64,
    pub headings: f64,
    pub body: f64,
    pub facts: f64,
    pub reference_terms: f64,
    pub embedded_content: f64,
}

impl Default for FtsColumnWeights {
    fn default() -> Self {
        Self {
            title: 8.0,
            aliases: 8.0,
            traits: 4.0,
            taxonomy_terms: 2.5,
            constraint_terms: 2.0,
            mechanic_terms: 1.5,
            source_terms: 0.5,
            metric_terms: 1.0,
            headings: 4.0,
            facts: 2.0,
            body: 1.0,
            reference_terms: 0.5,
            embedded_content: 0.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FtsSearchHit {
    pub record_key: RecordKey,
    pub rank: f64,
    pub lane: FtsSearchLane,
    pub lane_rank: u32,
    pub title_alias_texts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FtsSearchLane {
    Mixed,
    TitleAlias,
    Facet,
}

impl FtsSearchLane {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::TitleAlias => "title-alias",
            Self::Facet => "facet",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtsQuery {
    pub(crate) tokens: Vec<String>,
}

impl FtsQuery {
    pub fn from_tokens(tokens: Vec<String>) -> Option<Self> {
        let tokens = tokens
            .into_iter()
            .filter(|token| is_safe_fts_token(token))
            .map(|token| token.to_lowercase())
            .collect::<Vec<_>>();
        (!tokens.is_empty()).then_some(Self { tokens })
    }

    pub fn as_match_query(&self) -> String {
        self.as_disjunction_match_query()
    }

    pub(crate) fn as_conjunction_match_query(&self) -> String {
        self.tokens
            .iter()
            .filter(|token| !is_primary_type_intent_token(token))
            .map(|token| format!("\"{token}\""))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(crate) fn as_disjunction_match_query(&self) -> String {
        self.tokens
            .iter()
            .map(|token| format!("\"{token}\""))
            .collect::<Vec<_>>()
            .join(" OR ")
    }
}

impl SqliteIndexReader {
    pub fn query_weighted_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        weights: FtsColumnWeights,
    ) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        self.with_diesel_connection(|connection| {
            query_weighted_fts_index(connection, fts_query, filter, limit, weights)
        })
    }

    pub fn query_precision_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        self.with_diesel_connection(|connection| {
            query_precision_fts_index(connection, fts_query, filter, limit)
        })
    }

    pub fn query_fts_record_keys(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<RecordKey>, FilterCompileError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        self.with_diesel_connection(|connection| {
            query_fts_record_keys(connection, fts_query, filter, limit)
        })
    }

    pub fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, FilterCompileError> {
        if candidate_keys.is_empty() {
            return Ok(Vec::new());
        }
        self.with_diesel_connection(|connection| {
            query_fts_candidate_record_keys(connection, fts_query, candidate_keys)
        })
    }
}

fn is_safe_fts_token(token: &str) -> bool {
    !token.is_empty() && token.chars().all(char::is_alphanumeric)
}
