use crate::{RecordNavigationView, RecordSummaryView};
use atlas_domain::QueryPredicate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalModeView {
    Lexical,
    Semantic,
    Hybrid,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SearchPageRequest {
    pub number: u32,
    pub size: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SearchPageView {
    pub number: u32,
    pub size: u32,
    pub count: usize,
    pub total: u64,
    pub has_more: bool,
    pub next_page: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OpenResultWindowRequest {
    pub mode: ResultWindowMode,
    pub page: SearchPageRequest,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReadResultWindowPageRequest {
    pub page: SearchPageRequest,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResultWindowMode {
    ListRecords {
        filter: Option<QueryPredicate>,
    },
    TextSearch {
        query: String,
        filter: Option<QueryPredicate>,
        mode: RetrievalModeView,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResultWindowModeSummary {
    ListRecords,
    TextSearch {
        query: String,
        mode: RetrievalModeView,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ResultWindowPage {
    pub window_id: u64,
    pub mode: ResultWindowModeSummary,
    pub page: SearchPageView,
    pub rows: Vec<ResultWindowRow>,
    pub coverage: Option<SearchCoverageView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ResultWindowRow {
    pub record: RecordSummaryView,
    pub matches: Vec<SearchWitnessView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SearchWitnessView {
    pub navigation: RecordNavigationView,
    pub lane: SearchLaneView,
    pub label: Option<String>,
    pub snippet: Option<String>,
    pub lexical_rank: Option<f64>,
    pub semantic_similarity: Option<f64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SearchLaneView {
    Lexical,
    Semantic,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SearchCoverageView {
    pub exhaustive: bool,
    pub semantic_unit_window: Option<usize>,
    pub candidate_roots: usize,
    pub count_basis: String,
}
