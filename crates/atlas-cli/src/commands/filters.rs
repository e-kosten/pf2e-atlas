use atlas_domain::metadata::{
    MetadataEnumStringField, MetadataNumberField, MetadataNumberMatch, MetadataPredicate,
    MetadataSetField, MetadataSetMatch, MetadataStringMatch, MetadataTextMatch,
    MetadataTextStringField,
};
use atlas_domain::{
    MetricMatch, NumericMatch, RecordFamily, RecordKey, ScalarValue, SearchFilterNode,
};
use serde_json::Value;

use crate::FilterOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliFilterError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

pub(crate) fn build_filter(
    filter_json: Option<&str>,
    options: &FilterOptions,
) -> Result<(Option<SearchFilterNode>, Option<Value>), CliFilterError> {
    if filter_json.is_some() && options.has_convenience_filters() {
        return Err(CliFilterError {
            code: "invalid_filter",
            message: "filter convenience flags cannot be combined with --filter-json".to_string(),
        });
    }

    if let Some(filter_json) = filter_json {
        let filter_value =
            serde_json::from_str::<Value>(filter_json).map_err(|error| CliFilterError {
                code: "invalid_filter_json",
                message: format!("failed to parse --filter-json: {error}"),
            })?;
        let filter = serde_json::from_str::<SearchFilterNode>(filter_json).map_err(|error| {
            CliFilterError {
                code: "invalid_filter_json",
                message: format!("failed to parse --filter-json: {error}"),
            }
        })?;
        filter.validate().map_err(|error| CliFilterError {
            code: "invalid_filter",
            message: error.to_string(),
        })?;
        return Ok((Some(filter), Some(filter_value)));
    }

    let filter = build_convenience_filter(options)?;
    let filter_value = filter
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| CliFilterError {
            code: "invalid_filter",
            message: format!("failed to serialize convenience filter: {error}"),
        })?;
    Ok((filter, filter_value))
}

trait FilterOptionExt {
    fn has_convenience_filters(&self) -> bool;
}

impl FilterOptionExt for FilterOptions {
    fn has_convenience_filters(&self) -> bool {
        !self.families.is_empty()
            || !self.pack_names.is_empty()
            || !self.pack_labels.is_empty()
            || !self.rarities.is_empty()
            || !self.publication_titles.is_empty()
            || self.level.is_some()
            || self.min_level.is_some()
            || self.max_level.is_some()
            || self.price.is_some()
            || self.min_price.is_some()
            || self.max_price.is_some()
            || !self.traits.is_empty()
            || !self.any_traits.is_empty()
            || !self.references.is_empty()
            || !self.referenced_by.is_empty()
            || !self.metrics.is_empty()
    }
}

fn build_convenience_filter(
    options: &FilterOptions,
) -> Result<Option<SearchFilterNode>, CliFilterError> {
    let mut children = Vec::new();

    push_repeated_family(&mut children, &options.families)?;
    push_enum_string_filter(
        &mut children,
        MetadataEnumStringField::PackName,
        &options.pack_names,
    );
    push_enum_string_filter(
        &mut children,
        MetadataEnumStringField::PackLabel,
        &options.pack_labels,
    );
    push_enum_string_filter(
        &mut children,
        MetadataEnumStringField::Rarity,
        &options.rarities,
    );
    push_text_string_filter(
        &mut children,
        MetadataTextStringField::PublicationTitle,
        &options.publication_titles,
    );
    push_level_filter(
        &mut children,
        options.level.as_deref(),
        options.min_level,
        options.max_level,
    )?;
    push_number_filter(
        &mut children,
        MetadataNumberField::PriceCp,
        "--price",
        options.price.as_deref(),
        options.min_price,
        options.max_price,
    )?;
    for value in &options.traits {
        children.push(trait_filter(value));
    }
    if !options.any_traits.is_empty() {
        children.push(any_or_single(
            options.any_traits.iter().map(|value| trait_filter(value)),
        ));
    }
    for value in &options.references {
        children.push(SearchFilterNode::links_to(parse_record_key(
            value,
            "--references",
        )?));
    }
    for value in &options.referenced_by {
        children.push(SearchFilterNode::linked_from(parse_record_key(
            value,
            "--referenced-by",
        )?));
    }
    for value in &options.metrics {
        children.push(parse_metric_filter(value)?);
    }

    match children.len() {
        0 => Ok(None),
        1 => Ok(children.into_iter().next()),
        _ => Ok(Some(SearchFilterNode::all_of(children))),
    }
}

fn push_repeated_family(
    children: &mut Vec<SearchFilterNode>,
    values: &[String],
) -> Result<(), CliFilterError> {
    let nodes = values
        .iter()
        .map(|value| {
            let family = serde_plain_record_family(value)?;
            Ok(SearchFilterNode::record_family(family))
        })
        .collect::<Result<Vec<_>, CliFilterError>>()?;
    push_optional_group(children, nodes);
    Ok(())
}

fn serde_plain_record_family(value: &str) -> Result<RecordFamily, CliFilterError> {
    serde_json::from_value::<RecordFamily>(Value::String(value.to_string())).map_err(|error| {
        CliFilterError {
            code: "invalid_filter",
            message: format!("invalid --family value `{value}`: {error}"),
        }
    })
}

fn push_enum_string_filter(
    children: &mut Vec<SearchFilterNode>,
    field: MetadataEnumStringField,
    values: &[String],
) {
    let nodes = values.iter().map(|value| {
        SearchFilterNode::metadata(MetadataPredicate::EnumString {
            field,
            r#match: MetadataStringMatch::Eq {
                value: value.clone(),
            },
        })
    });
    push_optional_group(children, nodes.collect());
}

fn push_text_string_filter(
    children: &mut Vec<SearchFilterNode>,
    field: MetadataTextStringField,
    values: &[String],
) {
    let nodes = values.iter().map(|value| {
        SearchFilterNode::metadata(MetadataPredicate::Text {
            field,
            r#match: MetadataTextMatch::Eq {
                value: value.clone(),
            },
        })
    });
    push_optional_group(children, nodes.collect());
}

fn push_level_filter(
    children: &mut Vec<SearchFilterNode>,
    level: Option<&str>,
    min_level: Option<f64>,
    max_level: Option<f64>,
) -> Result<(), CliFilterError> {
    push_number_filter(
        children,
        MetadataNumberField::Level,
        "--level",
        level,
        min_level,
        max_level,
    )
}

fn push_number_filter(
    children: &mut Vec<SearchFilterNode>,
    field: MetadataNumberField,
    flag: &'static str,
    exact_or_range: Option<&str>,
    min_value: Option<f64>,
    max_value: Option<f64>,
) -> Result<(), CliFilterError> {
    if exact_or_range.is_some() && (min_value.is_some() || max_value.is_some()) {
        return Err(CliFilterError {
            code: "invalid_filter",
            message: format!(
                "{flag} cannot be combined with --min-* or --max-* for the same field"
            ),
        });
    }
    let Some(r#match) = (match exact_or_range {
        Some(value) => Some(parse_number_match(value, flag)?),
        None => match (min_value, max_value) {
            (Some(min), Some(max)) => Some(MetadataNumberMatch::Between { min, max }),
            (Some(value), None) => Some(MetadataNumberMatch::Gte { value }),
            (None, Some(value)) => Some(MetadataNumberMatch::Lte { value }),
            (None, None) => None,
        },
    }) else {
        return Ok(());
    };
    children.push(SearchFilterNode::metadata(MetadataPredicate::Number {
        field,
        r#match,
    }));
    Ok(())
}

fn parse_number_match(value: &str, flag: &str) -> Result<MetadataNumberMatch, CliFilterError> {
    if let Some((min, max)) = value.split_once("..") {
        if min.is_empty() || max.is_empty() {
            return Err(CliFilterError {
                code: "invalid_filter",
                message: format!("{flag} range must include both bounds, such as 1..5"),
            });
        }
        return Ok(MetadataNumberMatch::Between {
            min: parse_number_bound(min, flag)?,
            max: parse_number_bound(max, flag)?,
        });
    }
    Ok(NumericMatch::Eq {
        value: parse_number_bound(value, flag)?,
    }
    .into())
}

fn parse_number_bound(value: &str, flag: &str) -> Result<f64, CliFilterError> {
    value.parse::<f64>().map_err(|error| CliFilterError {
        code: "invalid_filter",
        message: format!("invalid {flag} value `{value}`: {error}"),
    })
}

fn parse_record_key(value: &str, flag: &str) -> Result<RecordKey, CliFilterError> {
    RecordKey::parse(value).map_err(|error| CliFilterError {
        code: "invalid_filter",
        message: format!("invalid {flag} record key `{value}`: {error}"),
    })
}

fn parse_metric_filter(value: &str) -> Result<SearchFilterNode, CliFilterError> {
    let (metric, operator, raw_value) = split_predicate(value, "--metric")?;
    if metric.is_empty() {
        return Err(CliFilterError {
            code: "invalid_filter",
            message: format!("invalid --metric predicate `{value}`: missing metric key"),
        });
    }
    let r#match = match operator {
        PredicateOperator::Eq | PredicateOperator::Colon => MetricMatch::Eq {
            value: parse_scalar_value(raw_value),
        },
        PredicateOperator::Gt => MetricMatch::Gt {
            value: parse_number_bound(raw_value, "--metric")?,
        },
        PredicateOperator::Gte => MetricMatch::Gte {
            value: parse_number_bound(raw_value, "--metric")?,
        },
        PredicateOperator::Lt => MetricMatch::Lt {
            value: parse_number_bound(raw_value, "--metric")?,
        },
        PredicateOperator::Lte => MetricMatch::Lte {
            value: parse_number_bound(raw_value, "--metric")?,
        },
    };
    Ok(SearchFilterNode::metric(metric, r#match))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PredicateOperator {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Colon,
}

fn split_predicate<'a>(
    value: &'a str,
    flag: &str,
) -> Result<(&'a str, PredicateOperator, &'a str), CliFilterError> {
    if value.contains("!=") {
        return Err(CliFilterError {
            code: "invalid_filter",
            message: format!("invalid {flag} predicate `{value}`: `!=` is not supported"),
        });
    }
    for (token, operator) in [
        (">=", PredicateOperator::Gte),
        ("<=", PredicateOperator::Lte),
        (">", PredicateOperator::Gt),
        ("<", PredicateOperator::Lt),
        ("=", PredicateOperator::Eq),
        (":", PredicateOperator::Colon),
    ] {
        if let Some((left, right)) = value.split_once(token) {
            if left.is_empty() || right.is_empty() {
                return Err(CliFilterError {
                    code: "invalid_filter",
                    message: format!("invalid {flag} predicate `{value}`"),
                });
            }
            return Ok((left, operator, right));
        }
    }
    Err(CliFilterError {
        code: "invalid_filter",
        message: format!("invalid {flag} predicate `{value}`: missing operator"),
    })
}

fn parse_scalar_value(value: &str) -> ScalarValue {
    match value {
        "true" => ScalarValue::Boolean(true),
        "false" => ScalarValue::Boolean(false),
        _ => value
            .parse::<f64>()
            .map(ScalarValue::Number)
            .unwrap_or_else(|_| ScalarValue::String(value.to_string())),
    }
}

fn trait_filter(value: &str) -> SearchFilterNode {
    SearchFilterNode::metadata(MetadataPredicate::Set {
        field: MetadataSetField::Traits,
        r#match: MetadataSetMatch::Includes {
            value: value.to_string(),
        },
    })
}

fn push_optional_group(children: &mut Vec<SearchFilterNode>, nodes: Vec<SearchFilterNode>) {
    if !nodes.is_empty() {
        children.push(any_or_single(nodes));
    }
}

fn any_or_single(nodes: impl IntoIterator<Item = SearchFilterNode>) -> SearchFilterNode {
    let mut nodes = nodes.into_iter().collect::<Vec<_>>();
    match nodes.len() {
        1 => nodes.remove(0),
        _ => SearchFilterNode::any_of(nodes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::{RecordFamily, ScalarValue};

    #[test]
    fn repeated_scalar_flags_or_within_field_and_and_across_fields() {
        let options = FilterOptions {
            families: vec!["spell".to_string(), "feat".to_string()],
            rarities: vec!["rare".to_string(), "unique".to_string()],
            ..FilterOptions::default()
        };

        let (filter, _) = build_filter(None, &options).expect("filter builds");
        assert_eq!(
            filter,
            Some(SearchFilterNode::all_of(vec![
                SearchFilterNode::any_of(vec![
                    SearchFilterNode::record_family(RecordFamily::Spell),
                    SearchFilterNode::record_family(RecordFamily::Feat),
                ]),
                SearchFilterNode::any_of(vec![
                    SearchFilterNode::metadata(MetadataPredicate::EnumString {
                        field: MetadataEnumStringField::Rarity,
                        r#match: MetadataStringMatch::Eq {
                            value: "rare".to_string(),
                        },
                    }),
                    SearchFilterNode::metadata(MetadataPredicate::EnumString {
                        field: MetadataEnumStringField::Rarity,
                        r#match: MetadataStringMatch::Eq {
                            value: "unique".to_string(),
                        },
                    }),
                ]),
            ]))
        );
    }

    #[test]
    fn trait_flags_support_all_and_any_groups() {
        let options = FilterOptions {
            traits: vec!["healing".to_string(), "vitality".to_string()],
            any_traits: vec!["fire".to_string(), "cold".to_string()],
            ..FilterOptions::default()
        };

        let (filter, _) = build_filter(None, &options).expect("filter builds");
        assert!(matches!(filter, Some(SearchFilterNode::AllOf { .. })));
        let Some(SearchFilterNode::AllOf { children }) = filter else {
            panic!("expected all_of");
        };
        assert_eq!(children.len(), 3);
        assert!(matches!(
            children.last(),
            Some(SearchFilterNode::AnyOf { children }) if children.len() == 2
        ));
    }

    #[test]
    fn level_flags_reject_conflicting_shapes() {
        let options = FilterOptions {
            level: Some("2".to_string()),
            min_level: Some(1.0),
            ..FilterOptions::default()
        };

        let error = build_filter(None, &options).unwrap_err();
        assert_eq!(error.code, "invalid_filter");
    }

    #[test]
    fn filter_json_cannot_combine_with_convenience_flags() {
        let options = FilterOptions {
            pack_names: vec!["actions".to_string()],
            ..FilterOptions::default()
        };

        let error =
            build_filter(Some(r#"{"kind":"record_family","value":"rule"}"#), &options).unwrap_err();
        assert_eq!(error.code, "invalid_filter");
    }

    #[test]
    fn reference_flags_lower_to_link_filters() {
        let options = FilterOptions {
            references: vec!["spells:fireball".to_string()],
            referenced_by: vec!["actions:activate".to_string()],
            ..FilterOptions::default()
        };

        let (filter, _) = build_filter(None, &options).expect("filter builds");
        assert_eq!(
            filter,
            Some(SearchFilterNode::all_of(vec![
                SearchFilterNode::links_to(RecordKey::parse("spells:fireball").unwrap()),
                SearchFilterNode::linked_from(RecordKey::parse("actions:activate").unwrap()),
            ]))
        );
    }

    #[test]
    fn price_and_metric_flags_lower_to_canonical_filters() {
        let options = FilterOptions {
            price: Some("100..500".to_string()),
            metrics: vec!["ac.value>=18".to_string(), "hp.value:40".to_string()],
            ..FilterOptions::default()
        };

        let (filter, _) = build_filter(None, &options).expect("filter builds");
        let Some(SearchFilterNode::AllOf { children }) = filter else {
            panic!("expected all_of");
        };
        assert_eq!(children.len(), 3);
        assert_eq!(
            children[0],
            SearchFilterNode::metadata(MetadataPredicate::Number {
                field: MetadataNumberField::PriceCp,
                r#match: MetadataNumberMatch::Between {
                    min: 100.0,
                    max: 500.0
                },
            })
        );
        assert_eq!(
            children[1],
            SearchFilterNode::metric("ac.value", MetricMatch::Gte { value: 18.0 })
        );
        assert_eq!(
            children[2],
            SearchFilterNode::metric(
                "hp.value",
                MetricMatch::Eq {
                    value: ScalarValue::Number(40.0),
                },
            )
        );
    }

    #[test]
    fn exact_min_and_max_price_flags_lower_to_number_filters() {
        let exact = FilterOptions {
            price: Some("250".to_string()),
            ..FilterOptions::default()
        };
        let min = FilterOptions {
            min_price: Some(100.0),
            ..FilterOptions::default()
        };
        let max = FilterOptions {
            max_price: Some(500.0),
            ..FilterOptions::default()
        };

        assert_eq!(
            build_filter(None, &exact).expect("filter builds").0,
            Some(SearchFilterNode::metadata(MetadataPredicate::Number {
                field: MetadataNumberField::PriceCp,
                r#match: MetadataNumberMatch::Eq { value: 250.0 },
            }))
        );
        assert_eq!(
            build_filter(None, &min).expect("filter builds").0,
            Some(SearchFilterNode::metadata(MetadataPredicate::Number {
                field: MetadataNumberField::PriceCp,
                r#match: MetadataNumberMatch::Gte { value: 100.0 },
            }))
        );
        assert_eq!(
            build_filter(None, &max).expect("filter builds").0,
            Some(SearchFilterNode::metadata(MetadataPredicate::Number {
                field: MetadataNumberField::PriceCp,
                r#match: MetadataNumberMatch::Lte { value: 500.0 },
            }))
        );
    }

    #[test]
    fn price_flags_reject_conflicting_shapes() {
        let options = FilterOptions {
            price: Some("250".to_string()),
            max_price: Some(500.0),
            ..FilterOptions::default()
        };

        let error = build_filter(None, &options).unwrap_err();
        assert_eq!(error.code, "invalid_filter");
    }

    #[test]
    fn invalid_reference_key_is_rejected() {
        let options = FilterOptions {
            references: vec!["not-a-key".to_string()],
            ..FilterOptions::default()
        };

        let error = build_filter(None, &options).unwrap_err();
        assert_eq!(error.code, "invalid_filter");
    }

    #[test]
    fn malformed_metric_predicates_are_rejected() {
        for metric in ["ac.value", "ac.value>=high", "ac.value!=18"] {
            let options = FilterOptions {
                metrics: vec![metric.to_string()],
                ..FilterOptions::default()
            };

            let error = build_filter(None, &options).unwrap_err();
            assert_eq!(error.code, "invalid_filter");
        }
    }
}
