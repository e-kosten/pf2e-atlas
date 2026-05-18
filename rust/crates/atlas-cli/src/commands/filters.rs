use atlas_domain::metadata::{
    MetadataEnumStringField, MetadataNumberField, MetadataNumberMatch, MetadataPredicate,
    MetadataSetField, MetadataSetMatch, MetadataStringMatch, MetadataTextMatch,
    MetadataTextStringField,
};
use atlas_domain::{NumericMatch, RecordFamily, SearchFilterNode};
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
            || !self.packs.is_empty()
            || !self.rarities.is_empty()
            || !self.sources.is_empty()
            || self.level.is_some()
            || self.min_level.is_some()
            || self.max_level.is_some()
            || !self.traits.is_empty()
            || !self.any_traits.is_empty()
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
        &options.packs,
    );
    push_enum_string_filter(
        &mut children,
        MetadataEnumStringField::Rarity,
        &options.rarities,
    );
    push_text_string_filter(
        &mut children,
        MetadataTextStringField::PublicationTitle,
        &options.sources,
    );
    push_level_filter(
        &mut children,
        options.level.as_deref(),
        options.min_level,
        options.max_level,
    )?;
    for value in &options.traits {
        children.push(trait_filter(value));
    }
    if !options.any_traits.is_empty() {
        children.push(any_or_single(
            options.any_traits.iter().map(|value| trait_filter(value)),
        ));
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
    if level.is_some() && (min_level.is_some() || max_level.is_some()) {
        return Err(CliFilterError {
            code: "invalid_filter",
            message: "--level cannot be combined with --min-level or --max-level".to_string(),
        });
    }
    let Some(r#match) = (match level {
        Some(level) => Some(parse_level_match(level)?),
        None => match (min_level, max_level) {
            (Some(min), Some(max)) => Some(MetadataNumberMatch::Between { min, max }),
            (Some(value), None) => Some(MetadataNumberMatch::Gte { value }),
            (None, Some(value)) => Some(MetadataNumberMatch::Lte { value }),
            (None, None) => None,
        },
    }) else {
        return Ok(());
    };
    children.push(SearchFilterNode::metadata(MetadataPredicate::Number {
        field: MetadataNumberField::Level,
        r#match,
    }));
    Ok(())
}

fn parse_level_match(value: &str) -> Result<MetadataNumberMatch, CliFilterError> {
    if let Some((min, max)) = value.split_once("..") {
        if min.is_empty() || max.is_empty() {
            return Err(CliFilterError {
                code: "invalid_filter",
                message: "--level range must include both bounds, such as 1..5".to_string(),
            });
        }
        return Ok(MetadataNumberMatch::Between {
            min: parse_level_bound(min, "--level")?,
            max: parse_level_bound(max, "--level")?,
        });
    }
    Ok(NumericMatch::Eq {
        value: parse_level_bound(value, "--level")?,
    }
    .into())
}

fn parse_level_bound(value: &str, flag: &str) -> Result<f64, CliFilterError> {
    value.parse::<f64>().map_err(|error| CliFilterError {
        code: "invalid_filter",
        message: format!("invalid {flag} value `{value}`: {error}"),
    })
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
    let nodes = nodes.into_iter().collect::<Vec<_>>();
    if nodes.len() == 1 {
        nodes.into_iter().next().expect("one node")
    } else {
        SearchFilterNode::any_of(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::RecordFamily;

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
            packs: vec!["actions".to_string()],
            ..FilterOptions::default()
        };

        let error =
            build_filter(Some(r#"{"kind":"record_family","value":"rule"}"#), &options).unwrap_err();
        assert_eq!(error.code, "invalid_filter");
    }
}
