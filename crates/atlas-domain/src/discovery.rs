use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterFieldType {
    Set,
    EnumString,
    Text,
    Number,
    Boolean,
    Metric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterFieldGroup {
    Record,
    Spell,
    Actor,
    Item,
    Variant,
    Metric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterValuePolicy {
    Enumerable,
    Sample,
    NumericStats,
    BooleanCounts,
    MetricKeys,
    MetricValues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Includes,
    Eq,
    NotEq,
    Contains,
    NotContains,
    Gt,
    Gte,
    Lt,
    Lte,
    Between,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterValueSort {
    Count,
    Alpha,
    Canonical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterDiscoveryExecution {
    Catalog,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterFieldInfo {
    pub field: String,
    pub field_type: FilterFieldType,
    pub group: FilterFieldGroup,
    pub value_policy: FilterValuePolicy,
    pub operators: Vec<FilterOperator>,
    pub applicable_families: Vec<String>,
    pub cli_flags: Vec<String>,
    pub catalog_available: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterFieldDiscovery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<serde_json::Value>,
    pub execution: FilterDiscoveryExecution,
    pub matching_record_count: u64,
    pub fields: Vec<FilterFieldInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterValueDiscovery {
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<serde_json::Value>,
    pub execution: FilterDiscoveryExecution,
    pub matching_record_count: u64,
    #[serde(flatten)]
    pub payload: FilterValuePayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "value_policy", rename_all = "snake_case")]
pub enum FilterValuePayload {
    Enumerable {
        values: Vec<FilterValueCount>,
        null_count: u64,
        sort: FilterValueSort,
    },
    Sample {
        sample: FilterSample,
        field_stats: FilterFieldStats,
        null_count: u64,
    },
    NumericStats {
        stats: NumericFieldStats,
    },
    BooleanCounts {
        counts: BooleanFieldCounts,
    },
    MetricKeys {
        metrics: Vec<MetricKeyDiscovery>,
    },
    MetricValues {
        metric: Box<MetricKeyDiscovery>,
        values: MetricValuePayload,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterValueCount {
    pub value: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterSample {
    pub selection: String,
    pub sample_limit: usize,
    pub distinct_count: u64,
    pub omitted_distinct_count: u64,
    pub examples: Vec<FilterSampleExample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterSampleExample {
    pub text: String,
    pub count: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterFieldStats {
    pub value_count: u64,
    pub null_count: u64,
    pub distinct_count: u64,
    pub singleton_count: u64,
    pub singleton_ratio: f64,
    pub observation_singleton_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericFieldStats {
    pub count: u64,
    pub null_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p05: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p25: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p75: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BooleanFieldCounts {
    pub r#true: u64,
    pub r#false: u64,
    pub null: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricKeyDiscovery {
    pub metric_domain: String,
    #[serde(alias = "record_family")]
    pub kind: String,
    pub namespace_prefix: String,
    pub metric_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub known: bool,
    pub value_type: String,
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric_stats: Option<NumericFieldStats>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetricValuePayload {
    NumericStats { stats: NumericFieldStats },
    TextValues { values: Vec<FilterValueCount> },
    BooleanCounts { counts: BooleanFieldCounts },
}
