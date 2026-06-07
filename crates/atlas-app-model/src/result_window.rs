use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{BasicSearchFilter, RecordSummaryView};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SearchPageRequest {
    pub number: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SearchPageView {
    pub number: u32,
    pub size: u32,
    pub count: usize,
    pub total: u64,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub next_page: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct OpenResultWindowRequest {
    pub mode: ResultWindowMode,
    pub page: SearchPageRequest,
    pub include_diagnostics: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ReadResultWindowPageRequest {
    pub page: SearchPageRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum ResultWindowMode {
    ListRecords {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        filter: Option<BasicSearchFilter>,
        sort: RecordListSortView,
    },
    TextSearch {
        query: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        exclude: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        filter: Option<BasicSearchFilter>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum ResultWindowModeSummary {
    ListRecords,
    TextSearch { query: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RecordListSortView {
    Alphabetical,
    LevelAsc,
    LevelDesc,
    PriceAsc,
    PriceDesc,
    RecordKey,
    Random { seed: u64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ResultWindowPage {
    pub window_id: u64,
    pub mode: ResultWindowModeSummary,
    pub page: SearchPageView,
    pub rows: Vec<ResultWindowRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ResultWindowRow {
    pub record: RecordSummaryView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub match_summary: Option<ResultMatchSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ResultMatchSummary {
    pub label: String,
}
