use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct BasicSearchFilter {
    pub clauses: Vec<FilterClause>,
}

impl BasicSearchFilter {
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterClause {
    pub id: String,
    pub field: String,
    pub operator: FilterClauseOperator,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub range: Option<FilterRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub metric: Option<MetricComparison>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum FilterClauseOperator {
    IncludeAny,
    IncludeAll,
    ExcludeAny,
    Range,
    MetricCompare,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct MetricComparison {
    pub key: String,
    pub op: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum FilterDiscoveryContext {
    Filtered { filter: BasicSearchFilter },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct DiscoverFilterEditorRequest {
    pub context: FilterDiscoveryContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct DiscoverFilterValuesRequest {
    pub context: FilterDiscoveryContext,
    pub field_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub metric_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub metric_domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterEditorView {
    pub matching_record_count: u64,
    pub groups: Vec<FilterEditorGroupView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterEditorGroupView {
    pub id: String,
    pub label: String,
    pub fields: Vec<FilterEditorFieldView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterEditorFieldView {
    pub id: String,
    pub label: String,
    pub control: FilterControlView,
    pub placement: FilterFieldPlacement,
    pub allowed_operators: Vec<FilterClauseOperator>,
    pub default_operator: FilterClauseOperator,
    pub supports_counts: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum FilterControlView {
    MultiSelect,
    Range {
        min_label: String,
        max_label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        max: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        step: Option<f64>,
    },
    Boolean {
        true_label: String,
        false_label: String,
    },
    MetricComparison {
        key_label: String,
        operator_label: String,
        value_label: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum FilterFieldPlacement {
    AlwaysVisible,
    InitiallyVisible,
    Addable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterValueListView {
    pub field_id: String,
    pub matching_record_count: u64,
    pub options: Vec<FilterValueOption>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterValueOption {
    pub value: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub count: Option<u64>,
    pub selected: bool,
    pub disabled: bool,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterValidationResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub compiled_filter: Option<serde_json::Value>,
    pub messages: Vec<FilterValidationMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub normalized_filter: Option<BasicSearchFilter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterValidationMessage {
    pub severity: String,
    pub code: FilterValidationCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub clause_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub field_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub recoverable_action: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum FilterValidationCode {
    FieldUnknown,
    FieldNotApplicable,
    OperatorNotSupported,
    ValueUnknown,
    ValueNotApplicable,
    ValueZeroCount,
    RangeInvalid,
    RangeEmpty,
    MetricUnknown,
    MetricAmbiguous,
    MetricNotNumeric,
    ClauseEmpty,
    ClauseConflict,
    FilterTooComplex,
}
