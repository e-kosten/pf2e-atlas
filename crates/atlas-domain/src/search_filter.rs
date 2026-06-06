use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::metadata::{
    MetadataEnumStringField, MetadataNumberField, MetadataNumberMatch, MetadataPredicate,
    MetadataSetField, MetadataSetMatch, MetadataStringMatch, MetadataTextMatch,
    MetadataTextStringField, NumericMetricOperator,
};
use crate::{RecordKey, RecordKind};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SearchFilterNode {
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimpleSearchFilter {
    pub kinds: Vec<RecordKind>,
    pub pack_names: Vec<String>,
    pub pack_labels: Vec<String>,
    pub rarities: Vec<String>,
    pub publication_titles: Vec<String>,
    pub level: Option<NumericMatch>,
    pub price_cp: Option<NumericMatch>,
    pub traits_all: Vec<String>,
    pub traits_any: Vec<String>,
    pub links_to: Vec<RecordKey>,
    pub linked_from: Vec<RecordKey>,
    pub metrics: Vec<MetricFilter>,
}

impl SimpleSearchFilter {
    pub fn into_filter_node(self) -> Result<Option<SearchFilterNode>, SimpleSearchFilterError> {
        let mut children = Vec::new();

        push_optional_group(
            &mut children,
            self.kinds
                .into_iter()
                .map(SearchFilterNode::record_kind)
                .collect(),
        );
        push_enum_string_filter(
            &mut children,
            MetadataEnumStringField::PackName,
            self.pack_names,
        );
        push_enum_string_filter(
            &mut children,
            MetadataEnumStringField::PackLabel,
            self.pack_labels,
        );
        push_enum_string_filter(
            &mut children,
            MetadataEnumStringField::Rarity,
            self.rarities,
        );
        push_text_string_filter(
            &mut children,
            MetadataTextStringField::PublicationTitle,
            self.publication_titles,
        );
        if let Some(level) = self.level {
            validate_numeric_match(level, "level")?;
            children.push(SearchFilterNode::level(level));
        }
        if let Some(price_cp) = self.price_cp {
            validate_numeric_match(price_cp, "price_cp")?;
            children.push(SearchFilterNode::price(price_cp));
        }
        for value in self.traits_all {
            children.push(trait_filter(value));
        }
        if !self.traits_any.is_empty() {
            children.push(any_or_single(
                self.traits_any.into_iter().map(trait_filter).collect(),
            ));
        }
        children.extend(self.links_to.into_iter().map(SearchFilterNode::links_to));
        children.extend(
            self.linked_from
                .into_iter()
                .map(SearchFilterNode::linked_from),
        );
        for metric in self.metrics {
            metric.validate()?;
            children.push(SearchFilterNode::metric(metric.key, metric.r#match));
        }

        match children.len() {
            0 => Ok(None),
            1 => Ok(children.into_iter().next()),
            _ => Ok(Some(SearchFilterNode::all_of(children))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricFilter {
    pub key: String,
    #[serde(rename = "match")]
    pub r#match: MetricMatch,
}

impl MetricFilter {
    fn validate(&self) -> Result<(), SimpleSearchFilterError> {
        if self.key.is_empty() {
            return Err(SimpleSearchFilterError::EmptyMetricKey);
        }
        validate_metric_match(&self.r#match, &self.key)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSearchFilterError {
    EmptyMetricKey,
    NonFiniteNumber { field: String, value: f64 },
    InvalidRange { field: String, min: f64, max: f64 },
}

impl fmt::Display for SimpleSearchFilterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMetricKey => formatter.write_str("metric filter key cannot be empty"),
            Self::NonFiniteNumber { field, value } => {
                write!(
                    formatter,
                    "{field} filter value must be finite, got {value}"
                )
            }
            Self::InvalidRange { field, min, max } => {
                write!(
                    formatter,
                    "{field} filter range minimum must be <= maximum, got {min}..{max}"
                )
            }
        }
    }
}

impl Error for SimpleSearchFilterError {}

fn push_enum_string_filter(
    children: &mut Vec<SearchFilterNode>,
    field: MetadataEnumStringField,
    values: Vec<String>,
) {
    push_optional_group(
        children,
        values
            .into_iter()
            .map(|value| {
                SearchFilterNode::metadata(MetadataPredicate::EnumString {
                    field,
                    r#match: MetadataStringMatch::Eq { value },
                })
            })
            .collect(),
    );
}

fn push_text_string_filter(
    children: &mut Vec<SearchFilterNode>,
    field: MetadataTextStringField,
    values: Vec<String>,
) {
    push_optional_group(
        children,
        values
            .into_iter()
            .map(|value| {
                SearchFilterNode::metadata(MetadataPredicate::Text {
                    field,
                    r#match: MetadataTextMatch::Eq { value },
                })
            })
            .collect(),
    );
}

fn trait_filter(value: String) -> SearchFilterNode {
    SearchFilterNode::metadata(MetadataPredicate::Set {
        field: MetadataSetField::Traits,
        r#match: MetadataSetMatch::Includes { value },
    })
}

fn push_optional_group(children: &mut Vec<SearchFilterNode>, nodes: Vec<SearchFilterNode>) {
    if !nodes.is_empty() {
        children.push(any_or_single(nodes));
    }
}

fn any_or_single(mut nodes: Vec<SearchFilterNode>) -> SearchFilterNode {
    match nodes.len() {
        1 => nodes.remove(0),
        _ => SearchFilterNode::any_of(nodes),
    }
}

fn validate_numeric_match(
    r#match: NumericMatch,
    field: &'static str,
) -> Result<(), SimpleSearchFilterError> {
    match r#match {
        NumericMatch::Eq { value }
        | NumericMatch::Gt { value }
        | NumericMatch::Gte { value }
        | NumericMatch::Lt { value }
        | NumericMatch::Lte { value } => validate_finite(value, field),
        NumericMatch::Between { min, max } => {
            validate_finite(min, field)?;
            validate_finite(max, field)?;
            if min > max {
                return Err(SimpleSearchFilterError::InvalidRange {
                    field: field.to_string(),
                    min,
                    max,
                });
            }
            Ok(())
        }
    }
}

fn validate_metric_match(
    r#match: &MetricMatch,
    field: &str,
) -> Result<(), SimpleSearchFilterError> {
    match r#match {
        MetricMatch::Eq { value } | MetricMatch::NotEq { value } => {
            validate_scalar_value(value, field)
        }
        MetricMatch::Gt { value }
        | MetricMatch::Gte { value }
        | MetricMatch::Lt { value }
        | MetricMatch::Lte { value } => validate_finite(*value, field),
    }
}

fn validate_scalar_value(value: &ScalarValue, field: &str) -> Result<(), SimpleSearchFilterError> {
    if let ScalarValue::Number(value) = value {
        validate_finite(*value, field)?;
    }
    Ok(())
}

fn validate_finite(value: f64, field: &str) -> Result<(), SimpleSearchFilterError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SimpleSearchFilterError::NonFiniteNumber {
            field: field.to_string(),
            value,
        })
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
            SearchFilterNode::all_of(Vec::new()).validate().unwrap_err(),
            SearchFilterValidationError::EmptyBooleanGroup {
                kind: "all_of",
                path: "filter",
            }
        );
    }

    #[test]
    fn simple_filter_lowers_repeated_fields_to_any_and_cross_fields_to_all() {
        let filter = SimpleSearchFilter {
            kinds: vec![RecordKind::Spell, RecordKind::Feat],
            rarities: vec!["rare".to_string(), "unique".to_string()],
            ..SimpleSearchFilter::default()
        }
        .into_filter_node()
        .expect("simple filter should lower");

        assert_eq!(
            filter,
            Some(SearchFilterNode::all_of(vec![
                SearchFilterNode::any_of(vec![
                    SearchFilterNode::record_kind(RecordKind::Spell),
                    SearchFilterNode::record_kind(RecordKind::Feat),
                ]),
                SearchFilterNode::any_of(vec![
                    SearchFilterNode::rarity(NullableStringMatch::Eq {
                        value: "rare".to_string(),
                    }),
                    SearchFilterNode::rarity(NullableStringMatch::Eq {
                        value: "unique".to_string(),
                    }),
                ]),
            ]))
        );
    }

    #[test]
    fn simple_filter_supports_all_and_any_trait_groups() {
        let filter = SimpleSearchFilter {
            traits_all: vec!["healing".to_string(), "vitality".to_string()],
            traits_any: vec!["fire".to_string(), "cold".to_string()],
            ..SimpleSearchFilter::default()
        }
        .into_filter_node()
        .expect("simple filter should lower");

        let Some(SearchFilterNode::AllOf { children }) = filter else {
            panic!("expected all_of");
        };
        assert_eq!(children.len(), 3);
        assert_eq!(
            children[0],
            SearchFilterNode::metadata(MetadataPredicate::Set {
                field: MetadataSetField::Traits,
                r#match: MetadataSetMatch::Includes {
                    value: "healing".to_string(),
                },
            })
        );
        assert!(matches!(
            children.last(),
            Some(SearchFilterNode::AnyOf { children }) if children.len() == 2
        ));
    }

    #[test]
    fn simple_filter_lowers_numeric_relationship_and_metric_filters() {
        let filter = SimpleSearchFilter {
            price_cp: Some(NumericMatch::Between {
                min: 100.0,
                max: 500.0,
            }),
            links_to: vec![RecordKey::parse("spells:fireball").expect("key should parse")],
            metrics: vec![MetricFilter {
                key: "ac.value".to_string(),
                r#match: MetricMatch::Gte { value: 18.0 },
            }],
            ..SimpleSearchFilter::default()
        }
        .into_filter_node()
        .expect("simple filter should lower");

        assert_eq!(
            filter,
            Some(SearchFilterNode::all_of(vec![
                SearchFilterNode::price(NumericMatch::Between {
                    min: 100.0,
                    max: 500.0,
                }),
                SearchFilterNode::links_to(RecordKey::parse("spells:fireball").unwrap()),
                SearchFilterNode::metric("ac.value", MetricMatch::Gte { value: 18.0 }),
            ]))
        );
    }

    #[test]
    fn simple_filter_rejects_invalid_numeric_values() {
        let error = SimpleSearchFilter {
            level: Some(NumericMatch::Between { min: 5.0, max: 1.0 }),
            ..SimpleSearchFilter::default()
        }
        .into_filter_node()
        .expect_err("inverted range should be rejected");

        assert_eq!(
            error,
            SimpleSearchFilterError::InvalidRange {
                field: "level".to_string(),
                min: 5.0,
                max: 1.0,
            }
        );
    }

    #[test]
    fn simple_filter_json_round_trips() {
        let filter = SimpleSearchFilter {
            kinds: vec![RecordKind::Spell],
            pack_names: vec!["spells-srd".to_string()],
            level: Some(NumericMatch::Between { min: 1.0, max: 3.0 }),
            traits_any: vec!["healing".to_string(), "vitality".to_string()],
            links_to: vec![RecordKey::parse("rules:healing").expect("key should parse")],
            metrics: vec![MetricFilter {
                key: "rank.value".to_string(),
                r#match: MetricMatch::Gte { value: 2.0 },
            }],
            ..SimpleSearchFilter::default()
        };

        let json = serde_json::to_string(&filter).expect("simple filter serializes");
        let decoded: SimpleSearchFilter =
            serde_json::from_str(&json).expect("simple filter deserializes");

        assert_eq!(decoded, filter);
        assert!(json.contains("\"kinds\""));
        assert!(json.contains("\"spells-srd\""));
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
