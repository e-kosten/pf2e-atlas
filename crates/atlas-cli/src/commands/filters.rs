use atlas_domain::{
    MetricFilter, MetricMatch, NumericMatch, RecordKey, RecordKind, ScalarValue, SearchFilterNode,
    SimpleSearchFilter,
};
use serde_json::Value;

use crate::cli::args::FilterOptions;

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

    let filter = simple_filter_from_options(options)?
        .into_filter_node()
        .map_err(|error| CliFilterError {
            code: "invalid_filter",
            message: error.to_string(),
        })?;
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
        !self.kinds.is_empty()
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

fn simple_filter_from_options(
    options: &FilterOptions,
) -> Result<SimpleSearchFilter, CliFilterError> {
    Ok(SimpleSearchFilter {
        kinds: parse_record_kinds(&options.kinds)?,
        pack_names: options.pack_names.clone(),
        pack_labels: options.pack_labels.clone(),
        rarities: options.rarities.clone(),
        publication_titles: options.publication_titles.clone(),
        level: parse_numeric_filter(
            "--level",
            options.level.as_deref(),
            options.min_level,
            options.max_level,
        )?,
        price_cp: parse_numeric_filter(
            "--price",
            options.price.as_deref(),
            options.min_price,
            options.max_price,
        )?,
        traits_all: options.traits.clone(),
        traits_any: options.any_traits.clone(),
        links_to: parse_record_keys(&options.references, "--references")?,
        linked_from: parse_record_keys(&options.referenced_by, "--referenced-by")?,
        metrics: options
            .metrics
            .iter()
            .map(|value| parse_metric_filter(value))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn parse_record_kinds(values: &[String]) -> Result<Vec<RecordKind>, CliFilterError> {
    values
        .iter()
        .map(|value| serde_plain_record_kind(value))
        .collect()
}

fn serde_plain_record_kind(value: &str) -> Result<RecordKind, CliFilterError> {
    serde_json::from_value::<RecordKind>(Value::String(value.to_string())).map_err(|error| {
        CliFilterError {
            code: "invalid_filter",
            message: format!("invalid --kind value `{value}`: {error}"),
        }
    })
}

fn parse_numeric_filter(
    flag: &'static str,
    exact_or_range: Option<&str>,
    min_level: Option<f64>,
    max_level: Option<f64>,
) -> Result<Option<NumericMatch>, CliFilterError> {
    if exact_or_range.is_some() && (min_level.is_some() || max_level.is_some()) {
        return Err(CliFilterError {
            code: "invalid_filter",
            message: format!(
                "{flag} cannot be combined with --min-* or --max-* for the same field"
            ),
        });
    }
    Ok(match exact_or_range {
        Some(value) => Some(parse_number_match(value, flag)?),
        None => match (min_level, max_level) {
            (Some(min), Some(max)) => Some(NumericMatch::Between { min, max }),
            (Some(value), None) => Some(NumericMatch::Gte { value }),
            (None, Some(value)) => Some(NumericMatch::Lte { value }),
            (None, None) => None,
        },
    })
}

fn parse_number_match(value: &str, flag: &str) -> Result<NumericMatch, CliFilterError> {
    if let Some((min, max)) = value.split_once("..") {
        if min.is_empty() || max.is_empty() {
            return Err(CliFilterError {
                code: "invalid_filter",
                message: format!("{flag} range must include both bounds, such as 1..5"),
            });
        }
        return Ok(NumericMatch::Between {
            min: parse_number_bound(min, flag)?,
            max: parse_number_bound(max, flag)?,
        });
    }
    Ok(NumericMatch::Eq {
        value: parse_number_bound(value, flag)?,
    })
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

fn parse_record_keys(
    values: &[String],
    flag: &'static str,
) -> Result<Vec<RecordKey>, CliFilterError> {
    values
        .iter()
        .map(|value| parse_record_key(value, flag))
        .collect()
}

fn parse_metric_filter(value: &str) -> Result<MetricFilter, CliFilterError> {
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
    Ok(MetricFilter {
        key: metric.to_string(),
        r#match,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::metadata::{
        MetadataEnumStringField, MetadataNumberField, MetadataNumberMatch, MetadataPredicate,
        MetadataStringMatch,
    };
    use atlas_domain::{RecordKind, ScalarValue};

    #[test]
    fn repeated_scalar_flags_or_within_field_and_and_across_fields() {
        let options = FilterOptions {
            kinds: vec!["spell".to_string(), "feat".to_string()],
            rarities: vec!["rare".to_string(), "unique".to_string()],
            ..FilterOptions::default()
        };

        let (filter, _) = build_filter(None, &options).expect("filter builds");
        assert_eq!(
            filter,
            Some(SearchFilterNode::all_of(vec![
                SearchFilterNode::any_of(vec![
                    SearchFilterNode::record_kind(RecordKind::Spell),
                    SearchFilterNode::record_kind(RecordKind::Feat),
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
            build_filter(Some(r#"{"kind":"record_kind","value":"rule"}"#), &options).unwrap_err();
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
