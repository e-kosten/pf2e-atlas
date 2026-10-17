//! Shared typed filter requests. SQL and CEL ASTs belong to atlas-index.
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum QueryFieldState {
    Value,
    Missing,
    Null,
    Invalid,
    NotApplicable,
}
impl QueryFieldState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Missing => "missing",
            Self::Null => "null",
            Self::Invalid => "invalid",
            Self::NotApplicable => "not_applicable",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum QueryCompare {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum QuerySetMatch {
    Includes,
    IncludesAny,
    IncludesAll,
    ExcludesAny,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(untagged)]
pub enum QueryLiteral {
    String(String),
    Boolean(bool),
    Number(#[ts(type = "number")] Number),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryPredicate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub clause_id: Option<String>,
    #[serde(flatten)]
    pub expression: QueryExpression,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryExpression {
    BooleanConstant {
        value: bool,
    },
    Compare {
        field: String,
        op: QueryCompare,
        value: QueryLiteral,
    },
    In {
        field: String,
        values: Vec<QueryLiteral>,
    },
    SetMatch {
        field: String,
        op: QuerySetMatch,
        values: Vec<String>,
    },
    StateMatch {
        field: String,
        state: QueryFieldState,
    },
    Exists {
        collection: String,
        scope_id: String,
        predicate: Box<QueryPredicate>,
    },
    AllOf {
        children: Vec<QueryPredicate>,
    },
    AnyOf {
        children: Vec<QueryPredicate>,
    },
    Not {
        predicate: Box<QueryPredicate>,
    },
}
impl QueryPredicate {
    pub fn new(expression: QueryExpression) -> Self {
        Self {
            clause_id: None,
            expression,
        }
    }
    pub fn boolean(value: bool) -> Self {
        Self::new(QueryExpression::BooleanConstant { value })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum QueryFieldType {
    String,
    Number,
    Boolean,
    Set,
    Collection,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum QueryValueDiscovery {
    OpenValues,
    ClosedChoices,
    BooleanCounts,
    NumericStatistics,
    CollectionStates,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryFieldDefinition {
    pub id: String,
    pub path: String,
    pub scope: Option<String>,
    pub field_type: QueryFieldType,
    pub label: String,
    pub family_types: Vec<String>,
    pub units: Option<String>,
    pub basis: String,
    pub choices: Vec<String>,
    pub operators: Vec<String>,
    pub examples: Vec<String>,
    pub value_discovery: QueryValueDiscovery,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryCapability {
    pub version: u32,
    pub fields: Vec<QueryFieldDefinition>,
    pub limits: QueryLimits,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryLimits {
    pub source_bytes: usize,
    pub nodes: usize,
    pub depth: usize,
    pub literal_list: usize,
}
impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            source_bytes: 16_384,
            nodes: 256,
            depth: 32,
            literal_list: 128,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct QueryError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub clause_id: Option<String>,
    pub source_span: Option<(usize, usize)>,
}
impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for QueryError {}

/// Error/unknown equivalence at the final match boundary: only True accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryTruth {
    True,
    False,
    Unknown,
}
impl QueryTruth {
    pub fn and(self, other: Self) -> Self {
        use QueryTruth::*;
        match (self, other) {
            (False, _) | (_, False) => False,
            (True, True) => True,
            _ => Unknown,
        }
    }
    pub fn or(self, other: Self) -> Self {
        use QueryTruth::*;
        match (self, other) {
            (True, _) | (_, True) => True,
            (False, False) => False,
            _ => Unknown,
        }
    }
    pub fn negate(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ts_rs::TS;
    #[test]
    fn query_ts_matches_flattened_serde_shape() {
        let declaration = QueryPredicate::decl();
        assert!(declaration.contains("\"kind\":"), "{declaration}");
        assert!(declaration.contains("\"exists\""), "{declaration}");
        assert!(declaration.contains("clause_id?:"), "{declaration}");
        assert!(QueryExpression::decl().contains("scope_id"));
        assert!(QueryLiteral::decl().contains("number"));
    }
}
