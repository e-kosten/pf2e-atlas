use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::metadata::{
    MetadataEnumStringField, MetadataNumberField, MetadataNumberMatch, MetadataPredicate,
    MetadataStringMatch, NumericMetricOperator,
};
use crate::{RecordKey, RecordKind};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SearchRequest {
    Browse {
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<SearchFilterNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sort: Option<BrowseSortSpec>,
    },
    Search {
        query: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclude: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        retrieval: Option<SearchRetrievalMode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fusion: Option<SearchFusionMethod>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<SearchFilterNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        explain: Option<bool>,
    },
    Lookup {
        query: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<SearchFilterNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sort: Option<LookupSortSpec>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchRetrievalMode {
    Fts,
    Vector,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SearchFusionMethod {
    Rrf,
    WeightedRrf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BrowseSortSpec {
    Alphabetical,
    LevelAsc,
    LevelDesc,
    Random {
        #[serde(skip_serializing_if = "Option::is_none")]
        seed: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookupSortSpec {
    pub kind: LookupSortKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<LookupSortPolicy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LookupSortKind {
    Alphabetical,
    LevelAsc,
    LevelDesc,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LookupSortPolicy {
    Tiered,
    Global,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SearchFilterNode {
    #[serde(alias = "record_family")]
    RecordKind {
        value: RecordKind,
    },
    LinksTo {
        target: RecordKey,
    },
    LinkedFrom {
        source: RecordKey,
    },
    MetadataPredicate {
        predicate: MetadataPredicate,
    },
    Metric {
        metric: String,
        #[serde(rename = "match")]
        r#match: MetricMatch,
    },
    MetricCompare {
        left_metric: String,
        op: NumericMetricOperator,
        right_metric: String,
    },
    AnyOf {
        children: Vec<SearchFilterNode>,
    },
    AllOf {
        children: Vec<SearchFilterNode>,
    },
    Not {
        child: Box<SearchFilterNode>,
    },
}

impl SearchFilterNode {
    pub fn pack(value: impl Into<String>) -> Self {
        Self::metadata(MetadataPredicate::EnumString {
            field: MetadataEnumStringField::PackName,
            r#match: MetadataStringMatch::Eq {
                value: value.into(),
            },
        })
    }

    pub fn record_kind(value: RecordKind) -> Self {
        Self::RecordKind { value }
    }

    pub fn level(r#match: NumericMatch) -> Self {
        Self::metadata(MetadataPredicate::Number {
            field: MetadataNumberField::Level,
            r#match: r#match.into(),
        })
    }

    pub fn price(r#match: NumericMatch) -> Self {
        Self::metadata(MetadataPredicate::Number {
            field: MetadataNumberField::PriceCp,
            r#match: r#match.into(),
        })
    }

    pub fn rarity(r#match: NullableStringMatch) -> Self {
        Self::metadata(MetadataPredicate::EnumString {
            field: MetadataEnumStringField::Rarity,
            r#match: r#match.into(),
        })
    }

    pub fn action_cost(r#match: NullableNumericMatch) -> Self {
        Self::metadata(MetadataPredicate::Number {
            field: MetadataNumberField::ActionCost,
            r#match: r#match.into(),
        })
    }

    pub fn links_to(target: RecordKey) -> Self {
        Self::LinksTo { target }
    }

    pub fn linked_from(source: RecordKey) -> Self {
        Self::LinkedFrom { source }
    }

    pub fn metadata(predicate: MetadataPredicate) -> Self {
        Self::MetadataPredicate { predicate }
    }

    pub fn metric(metric: impl Into<String>, r#match: MetricMatch) -> Self {
        Self::Metric {
            metric: metric.into(),
            r#match,
        }
    }

    pub fn metric_compare(
        left_metric: impl Into<String>,
        op: NumericMetricOperator,
        right_metric: impl Into<String>,
    ) -> Self {
        Self::MetricCompare {
            left_metric: left_metric.into(),
            op,
            right_metric: right_metric.into(),
        }
    }

    pub fn any_of(children: impl Into<Vec<Self>>) -> Self {
        Self::AnyOf {
            children: children.into(),
        }
    }

    pub fn all_of(children: impl Into<Vec<Self>>) -> Self {
        Self::AllOf {
            children: children.into(),
        }
    }

    pub fn not_of(child: Self) -> Self {
        Self::Not {
            child: Box::new(child),
        }
    }

    pub fn validate(&self) -> Result<(), SearchFilterValidationError> {
        self.validate_at("filter")
    }

    fn validate_at(&self, path: &'static str) -> Result<(), SearchFilterValidationError> {
        match self {
            Self::AnyOf { children } => validate_children("any_of", children, path),
            Self::AllOf { children } => validate_children("all_of", children, path),
            Self::Not { child } => child.validate_at("not"),
            Self::RecordKind { .. }
            | Self::LinksTo { .. }
            | Self::LinkedFrom { .. }
            | Self::MetadataPredicate { .. }
            | Self::Metric { .. }
            | Self::MetricCompare { .. } => Ok(()),
        }
    }
}

fn validate_children(
    kind: &'static str,
    children: &[SearchFilterNode],
    path: &'static str,
) -> Result<(), SearchFilterValidationError> {
    if children.is_empty() {
        return Err(SearchFilterValidationError::EmptyBooleanGroup { kind, path });
    }
    for child in children {
        child.validate_at(kind)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchFilterValidationError {
    EmptyBooleanGroup {
        kind: &'static str,
        path: &'static str,
    },
}

impl fmt::Display for SearchFilterValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBooleanGroup { kind, path } => {
                write!(formatter, "{path} contains empty {kind} group")
            }
        }
    }
}

impl Error for SearchFilterValidationError {}

impl SearchRequest {
    pub fn validate(&self) -> Result<(), SearchFilterValidationError> {
        match self {
            Self::Browse { filter, .. }
            | Self::Search { filter, .. }
            | Self::Lookup { filter, .. } => {
                if let Some(filter) = filter {
                    filter.validate()?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NumericMatch {
    Eq { value: f64 },
    Gt { value: f64 },
    Gte { value: f64 },
    Lt { value: f64 },
    Lte { value: f64 },
    Between { min: f64, max: f64 },
}

impl From<NumericMatch> for MetadataNumberMatch {
    fn from(value: NumericMatch) -> Self {
        match value {
            NumericMatch::Eq { value } => Self::Eq { value },
            NumericMatch::Gt { value } => Self::Gt { value },
            NumericMatch::Gte { value } => Self::Gte { value },
            NumericMatch::Lt { value } => Self::Lt { value },
            NumericMatch::Lte { value } => Self::Lte { value },
            NumericMatch::Between { min, max } => Self::Between { min, max },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NullableNumericMatch {
    Eq { value: f64 },
    Gt { value: f64 },
    Gte { value: f64 },
    Lt { value: f64 },
    Lte { value: f64 },
    Between { min: f64, max: f64 },
    IsNull,
    IsNotNull,
}

impl From<NullableNumericMatch> for MetadataNumberMatch {
    fn from(value: NullableNumericMatch) -> Self {
        match value {
            NullableNumericMatch::Eq { value } => Self::Eq { value },
            NullableNumericMatch::Gt { value } => Self::Gt { value },
            NullableNumericMatch::Gte { value } => Self::Gte { value },
            NullableNumericMatch::Lt { value } => Self::Lt { value },
            NullableNumericMatch::Lte { value } => Self::Lte { value },
            NullableNumericMatch::Between { min, max } => Self::Between { min, max },
            NullableNumericMatch::IsNull => Self::IsNull,
            NullableNumericMatch::IsNotNull => Self::IsNotNull,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NullableStringMatch {
    Eq { value: String },
    IsNull,
    IsNotNull,
}

impl From<NullableStringMatch> for MetadataStringMatch {
    fn from(value: NullableStringMatch) -> Self {
        match value {
            NullableStringMatch::Eq { value } => Self::Eq { value },
            NullableStringMatch::IsNull => Self::IsNull,
            NullableStringMatch::IsNotNull => Self::IsNotNull,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetricMatch {
    Eq { value: ScalarValue },
    NotEq { value: ScalarValue },
    Gt { value: f64 },
    Gte { value: f64 },
    Lt { value: f64 },
    Lte { value: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScalarValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{
        MetadataEnumStringField, MetadataNumberField, MetadataNumberMatch, MetadataPredicate,
        MetadataSetField, MetadataSetMatch, MetadataStringMatch,
    };

    #[test]
    fn search_request_round_trips_with_rust_canonical_names() {
        let request = SearchRequest::Search {
            query: "cold primal focus".to_string(),
            exclude: Some("ritual".to_string()),
            retrieval: Some(SearchRetrievalMode::Hybrid),
            fusion: Some(SearchFusionMethod::WeightedRrf),
            filter: Some(SearchFilterNode::all_of(vec![
                SearchFilterNode::record_kind(RecordKind::Spell),
                SearchFilterNode::metadata(MetadataPredicate::Set {
                    field: MetadataSetField::Traditions,
                    r#match: MetadataSetMatch::Includes {
                        value: "primal".to_string(),
                    },
                }),
                SearchFilterNode::links_to(
                    RecordKey::parse("rules:abc123").expect("record key parses"),
                ),
            ])),
            offset: Some(0),
            limit: Some(10),
            explain: Some(true),
        };

        let json = serde_json::to_string(&request).expect("request serializes");
        assert!(json.contains("\"mode\":\"search\""));
        assert!(json.contains("\"retrieval\":\"hybrid\""));
        assert!(json.contains("\"fusion\":\"weighted-rrf\""));
        assert!(json.contains("\"kind\":\"all_of\""));
        assert!(json.contains("\"kind\":\"record_kind\""));
        assert!(json.contains("\"kind\":\"links_to\""));
        assert!(json.contains("\"field_type\":\"set\""));

        let decoded: SearchRequest = serde_json::from_str(&json).expect("request deserializes");
        assert_eq!(decoded, request);
    }

    #[test]
    fn browse_request_round_trips_with_sort_and_numeric_filter() {
        let request = SearchRequest::Browse {
            filter: Some(SearchFilterNode::level(NumericMatch::Between {
                min: 1.0,
                max: 5.0,
            })),
            offset: None,
            limit: Some(20),
            sort: Some(BrowseSortSpec::Random { seed: Some(123) }),
        };

        let json = serde_json::to_string(&request).expect("request serializes");
        assert!(json.contains("\"mode\":\"browse\""));
        assert!(json.contains("\"kind\":\"metadata_predicate\""));
        assert!(json.contains("\"field\":\"level\""));
        assert!(json.contains("\"match\":{\"kind\":\"between\""));

        let decoded: SearchRequest = serde_json::from_str(&json).expect("request deserializes");
        assert_eq!(decoded, request);
    }

    #[test]
    fn friendly_field_constructors_lower_to_metadata_predicates() {
        assert_eq!(
            SearchFilterNode::pack("actions"),
            SearchFilterNode::metadata(MetadataPredicate::EnumString {
                field: MetadataEnumStringField::PackName,
                r#match: MetadataStringMatch::Eq {
                    value: "actions".to_string(),
                },
            })
        );
        assert_eq!(
            SearchFilterNode::level(NumericMatch::Gte { value: 2.0 }),
            SearchFilterNode::metadata(MetadataPredicate::Number {
                field: MetadataNumberField::Level,
                r#match: MetadataNumberMatch::Gte { value: 2.0 },
            })
        );
    }

    #[test]
    fn lookup_request_round_trips_with_metric_compare_and_not() {
        let request = SearchRequest::Lookup {
            query: "Treat Wounds".to_string(),
            filter: Some(SearchFilterNode::not_of(SearchFilterNode::metric_compare(
                "skill.medicine",
                NumericMetricOperator::Gte,
                "dc.standard",
            ))),
            offset: None,
            limit: None,
            sort: Some(LookupSortSpec {
                kind: LookupSortKind::Alphabetical,
                policy: Some(LookupSortPolicy::Tiered),
            }),
        };

        let json = serde_json::to_string(&request).expect("request serializes");
        assert!(json.contains("\"kind\":\"metric_compare\""));
        assert!(json.contains("\"left_metric\""));

        let decoded: SearchRequest = serde_json::from_str(&json).expect("request deserializes");
        assert_eq!(decoded, request);
    }

    #[test]
    fn metric_ordering_requires_numeric_value() {
        let json =
            r#"{"kind":"metric","metric":"defense.ac","match":{"kind":"gte","value":"high"}}"#;
        assert!(serde_json::from_str::<SearchFilterNode>(json).is_err());
    }

    #[test]
    fn rejects_removed_duplicate_field_leaf_json() {
        let json = r#"{"kind":"level","match":{"kind":"gte","value":2}}"#;
        assert!(serde_json::from_str::<SearchFilterNode>(json).is_err());
    }

    #[test]
    fn rejects_empty_boolean_groups_during_validation() {
        assert_eq!(
            SearchFilterNode::any_of(Vec::new()).validate().unwrap_err(),
            SearchFilterValidationError::EmptyBooleanGroup {
                kind: "any_of",
                path: "filter",
            }
        );
        assert_eq!(
            SearchRequest::Browse {
                filter: Some(SearchFilterNode::all_of(Vec::new())),
                offset: None,
                limit: None,
                sort: None,
            }
            .validate()
            .unwrap_err(),
            SearchFilterValidationError::EmptyBooleanGroup {
                kind: "all_of",
                path: "filter",
            }
        );
    }

    #[test]
    fn metadata_number_predicate_shape_round_trips() {
        let predicate = MetadataPredicate::Number {
            field: MetadataNumberField::ActionCost,
            r#match: MetadataNumberMatch::Between { min: 1.0, max: 2.0 },
        };

        let json = serde_json::to_string(&predicate).expect("predicate serializes");
        assert!(json.contains("\"field_type\":\"number\""));
        assert!(json.contains("\"field\":\"action_cost\""));

        let decoded: MetadataPredicate =
            serde_json::from_str(&json).expect("predicate deserializes");
        assert_eq!(decoded, predicate);
    }

    #[test]
    fn rejects_ts_camel_case_filter_kind_in_domain_json() {
        let json = r#"{"kind":"linksTo","target":"rules:abc"}"#;
        assert!(serde_json::from_str::<SearchFilterNode>(json).is_err());
    }
}
