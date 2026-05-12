use serde::{Deserialize, Serialize};

use crate::metadata::{MetadataPredicate, MetricOperator, NumericMetricOperator};
use crate::{Category, RecordKey};

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
        profile: Option<SearchProfile>,
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
pub enum SearchProfile {
    Lexical,
    Balanced,
    Concept,
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
    Pack {
        value: String,
    },
    Scope {
        category: Category,
    },
    Level {
        #[serde(rename = "match")]
        r#match: NumericMatch,
    },
    Price {
        #[serde(rename = "match")]
        r#match: NumericMatch,
    },
    Rarity {
        #[serde(rename = "match")]
        r#match: NullableStringMatch,
    },
    ActionCost {
        #[serde(rename = "match")]
        r#match: NullableNumericMatch,
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
        op: MetricOperator,
        value: ScalarValue,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NullableStringMatch {
    Eq { value: String },
    In { values: Vec<String> },
    NotIn { values: Vec<String> },
    IsNull,
    IsNotNull,
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
        CollectionOperator, MetadataNumberField, MetadataPredicate, MetadataSetField,
        NumberOperator,
    };

    #[test]
    fn search_request_round_trips_with_rust_canonical_names() {
        let request = SearchRequest::Search {
            query: "cold primal focus".to_string(),
            exclude: Some("ritual".to_string()),
            profile: Some(SearchProfile::Balanced),
            filter: Some(SearchFilterNode::AllOf {
                children: vec![
                    SearchFilterNode::Scope {
                        category: Category::Spell,
                    },
                    SearchFilterNode::MetadataPredicate {
                        predicate: MetadataPredicate::Set {
                            field: MetadataSetField::Traditions,
                            op: CollectionOperator::Includes,
                            value: Some("primal".to_string()),
                        },
                    },
                    SearchFilterNode::LinksTo {
                        target: RecordKey::parse("rules:abc123").expect("record key parses"),
                    },
                ],
            }),
            offset: Some(0),
            limit: Some(10),
            explain: Some(true),
        };

        let json = serde_json::to_string(&request).expect("request serializes");
        assert!(json.contains("\"mode\":\"search\""));
        assert!(json.contains("\"kind\":\"all_of\""));
        assert!(json.contains("\"kind\":\"links_to\""));
        assert!(json.contains("\"field_type\":\"set\""));

        let decoded: SearchRequest = serde_json::from_str(&json).expect("request deserializes");
        assert_eq!(decoded, request);
    }

    #[test]
    fn browse_request_round_trips_with_sort_and_numeric_filter() {
        let request = SearchRequest::Browse {
            filter: Some(SearchFilterNode::Level {
                r#match: NumericMatch::Between { min: 1.0, max: 5.0 },
            }),
            offset: None,
            limit: Some(20),
            sort: Some(BrowseSortSpec::Random { seed: Some(123) }),
        };

        let json = serde_json::to_string(&request).expect("request serializes");
        assert!(json.contains("\"mode\":\"browse\""));
        assert!(json.contains("\"kind\":\"level\""));
        assert!(json.contains("\"match\":{\"kind\":\"between\""));

        let decoded: SearchRequest = serde_json::from_str(&json).expect("request deserializes");
        assert_eq!(decoded, request);
    }

    #[test]
    fn lookup_request_round_trips_with_metric_compare_and_not() {
        let request = SearchRequest::Lookup {
            query: "Treat Wounds".to_string(),
            filter: Some(SearchFilterNode::Not {
                child: Box::new(SearchFilterNode::MetricCompare {
                    left_metric: "skill.medicine".to_string(),
                    op: MetricOperator::Gte,
                    right_metric: "dc.standard".to_string(),
                }),
            }),
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
    fn metadata_number_predicate_shape_round_trips() {
        let predicate = MetadataPredicate::Number {
            field: MetadataNumberField::ActionCost,
            op: NumberOperator::Between,
            value: None,
            min: Some(1.0),
            max: Some(2.0),
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
