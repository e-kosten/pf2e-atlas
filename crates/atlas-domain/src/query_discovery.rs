use super::query::{QueryFieldState, QueryLiteral};
use serde::{Deserialize, Serialize};
use serde_json::Number;

/// Optional contextual discovery can be unavailable for a valid query. Invalid
/// predicates and requests remain errors rather than discovery outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum QueryFacetOutcome<T> {
    Available { value: T },
    Unavailable { reason: FacetUnavailableReason },
}
impl<T> QueryFacetOutcome<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> QueryFacetOutcome<U> {
        match self {
            Self::Available { value } => QueryFacetOutcome::Available { value: f(value) },
            Self::Unavailable { reason } => QueryFacetOutcome::Unavailable { reason },
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum FacetUnavailableReason {
    NonConjunctiveContext,
    NegativeCondition,
    TextContext,
    RelationshipContext,
}

/// Suggestions are independent of search conditions and have no result counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryVocabularyOption {
    pub value: QueryLiteral,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum QueryVocabularyBasis {
    ProductArtifactBeforeEditionPreference,
    ProductKeyScopeBeforeEditionPreference,
}
/// `selected_values` contains valid selected literals, including values absent
/// from the suggestion page. `total_values` counts available values matching
/// vocabulary text, excluding absent selections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryVocabulary {
    pub field: String,
    pub options: Vec<QueryVocabularyOption>,
    pub selected_values: Vec<QueryLiteral>,
    pub total_values: u64,
    pub basis: QueryVocabularyBasis,
}

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
