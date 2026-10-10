use super::query::{QueryFieldState, QueryLiteral};
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryStateCount {
    pub state: QueryFieldState,
    pub occurrences: u64,
    pub distinct_roots: u64,
}
/// `counting_scope` partitions roots for root values and counts occurrences
/// for collection-member states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryFieldCounts {
    pub field: String,
    pub counting_scope: String,
    pub states: Vec<QueryStateCount>,
    pub exhaustive: bool,
    pub count_basis: String,
    #[ts(type = "number | null")]
    pub minimum: Option<Number>,
    #[ts(type = "number | null")]
    pub maximum: Option<Number>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryValueOption {
    pub value: QueryLiteral,
    pub distinct_roots: u64,
    pub selected: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryValueOptions {
    pub field: String,
    pub options: Vec<QueryValueOption>,
    pub total_values: u64,
    pub exhaustive: bool,
    pub count_basis: String,
}
