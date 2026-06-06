use atlas_app_model::{BasicSearchFilter, FilterClauseOperator, FilterRange, MetricComparison};
use atlas_domain::{
    MetadataEnumStringField, MetadataPredicate, MetadataSetField, MetadataSetMatch,
    MetadataStringMatch, MetadataTextMatch, MetadataTextStringField, MetricFilter, MetricMatch,
    NumericMatch, RecordKind, ScalarValue, SearchFilterNode, SimpleSearchFilter,
};

use crate::{AppServiceError, AppServiceResult};

pub(crate) fn lower_basic_filter(
    filter: Option<&BasicSearchFilter>,
) -> AppServiceResult<Option<SearchFilterNode>> {
    let Some(filter) = filter else {
        return Ok(None);
    };
    if filter.clauses.is_empty() {
        return Ok(None);
    }
    let mut simple = SimpleSearchFilter::default();
    let mut direct_nodes = Vec::new();

    for clause in &filter.clauses {
        if is_empty_clause(clause) {
            continue;
        }
        match clause.operator {
            FilterClauseOperator::IncludeAny => lower_include_any(
                &mut simple,
                &mut direct_nodes,
                &clause.field,
                &clause.values,
            )?,
            FilterClauseOperator::IncludeAll => lower_include_all(
                &mut simple,
                &mut direct_nodes,
                &clause.field,
                &clause.values,
            )?,
            FilterClauseOperator::ExcludeAny => {
                direct_nodes.push(SearchFilterNode::not_of(any_value_nodes(
                    &clause.field,
                    &clause.values,
                )?));
            }
            FilterClauseOperator::Range => {
                lower_range(&mut simple, &clause.field, clause.range)?;
            }
            FilterClauseOperator::MetricCompare => {
                lower_metric(&mut simple, clause.metric.as_ref())?;
            }
        }
    }

    let mut nodes = Vec::new();
    if let Some(simple_node) = simple
        .into_filter_node()
        .map_err(|error| AppServiceError::invalid_request(error.to_string()))?
    {
        nodes.push(simple_node);
    }
    nodes.extend(direct_nodes);

    Ok(match nodes.len() {
        0 => None,
        1 => nodes.into_iter().next(),
        _ => Some(SearchFilterNode::all_of(nodes)),
    })
}

fn is_empty_clause(clause: &atlas_app_model::FilterClause) -> bool {
    clause.values.is_empty() && clause.range.is_none() && clause.metric.is_none()
}

fn lower_include_any(
    simple: &mut SimpleSearchFilter,
    direct_nodes: &mut Vec<SearchFilterNode>,
    field: &str,
    values: &[String],
) -> AppServiceResult<()> {
    match canonical_field(field).as_str() {
        "kind" | "record_kind" => {
            simple.kinds.extend(
                values
                    .iter()
                    .map(|value| {
                        RecordKind::from_input(value).ok_or_else(|| {
                            AppServiceError::invalid_request(format!(
                                "unknown record kind `{value}`"
                            ))
                        })
                    })
                    .collect::<AppServiceResult<Vec<_>>>()?,
            );
        }
        "trait" | "traits" => simple.traits_any.extend(values.iter().cloned()),
        "rarity" => simple.rarities.extend(values.iter().cloned()),
        "pack" | "pack_name" => simple.pack_names.extend(values.iter().cloned()),
        "pack_label" => simple.pack_labels.extend(values.iter().cloned()),
        "source" | "publication" | "publication_title" => {
            simple.publication_titles.extend(values.iter().cloned());
        }
        other => {
            direct_nodes.push(any_value_nodes(other, values)?);
        }
    }
    Ok(())
}

fn lower_include_all(
    simple: &mut SimpleSearchFilter,
    direct_nodes: &mut Vec<SearchFilterNode>,
    field: &str,
    values: &[String],
) -> AppServiceResult<()> {
    match canonical_field(field).as_str() {
        "trait" | "traits" => simple.traits_all.extend(values.iter().cloned()),
        other => direct_nodes.push(SearchFilterNode::all_of(value_nodes(other, values)?)),
    }
    Ok(())
}

fn lower_range(
    simple: &mut SimpleSearchFilter,
    field: &str,
    range: Option<FilterRange>,
) -> AppServiceResult<()> {
    let Some(range) = range else {
        return Err(AppServiceError::invalid_request(
            "range clause is missing range",
        ));
    };
    let numeric_match = numeric_match(range)?;
    match canonical_field(field).as_str() {
        "level" | "rank" => simple.level = Some(numeric_match),
        "price" | "price_cp" => simple.price_cp = Some(numeric_match),
        other => {
            return Err(AppServiceError::invalid_request(format!(
                "range filters are not supported for field `{other}`"
            )));
        }
    }
    Ok(())
}

fn lower_metric(
    simple: &mut SimpleSearchFilter,
    metric: Option<&MetricComparison>,
) -> AppServiceResult<()> {
    let Some(metric) = metric else {
        return Err(AppServiceError::invalid_request(
            "metric clause is missing metric comparison",
        ));
    };
    simple.metrics.push(MetricFilter {
        key: metric.key.clone(),
        r#match: match metric.op.as_str() {
            "gt" => MetricMatch::Gt {
                value: metric.value,
            },
            "gte" => MetricMatch::Gte {
                value: metric.value,
            },
            "lt" => MetricMatch::Lt {
                value: metric.value,
            },
            "lte" => MetricMatch::Lte {
                value: metric.value,
            },
            "eq" => MetricMatch::Eq {
                value: ScalarValue::Number(metric.value),
            },
            other => {
                return Err(AppServiceError::invalid_request(format!(
                    "unsupported metric operator `{other}`"
                )));
            }
        },
    });
    Ok(())
}

fn any_value_nodes(field: &str, values: &[String]) -> AppServiceResult<SearchFilterNode> {
    let mut nodes = value_nodes(field, values)?;
    Ok(match nodes.len() {
        0 => {
            return Err(AppServiceError::invalid_request(
                "filter clause has no values",
            ));
        }
        1 => nodes.remove(0),
        _ => SearchFilterNode::any_of(nodes),
    })
}

fn value_nodes(field: &str, values: &[String]) -> AppServiceResult<Vec<SearchFilterNode>> {
    values
        .iter()
        .map(|value| value_node(field, value))
        .collect()
}

fn value_node(field: &str, value: &str) -> AppServiceResult<SearchFilterNode> {
    Ok(match canonical_field(field).as_str() {
        "kind" | "record_kind" => {
            SearchFilterNode::record_kind(RecordKind::from_input(value).ok_or_else(|| {
                AppServiceError::invalid_request(format!("unknown record kind `{value}`"))
            })?)
        }
        "trait" | "traits" => SearchFilterNode::metadata(MetadataPredicate::Set {
            field: MetadataSetField::Traits,
            r#match: MetadataSetMatch::Includes {
                value: value.to_string(),
            },
        }),
        "rarity" => SearchFilterNode::rarity(atlas_domain::NullableStringMatch::Eq {
            value: value.to_string(),
        }),
        "pack" | "pack_name" => SearchFilterNode::pack(value.to_string()),
        "pack_label" => SearchFilterNode::metadata(MetadataPredicate::EnumString {
            field: MetadataEnumStringField::PackLabel,
            r#match: MetadataStringMatch::Eq {
                value: value.to_string(),
            },
        }),
        "source" | "publication" | "publication_title" => {
            SearchFilterNode::metadata(MetadataPredicate::Text {
                field: MetadataTextStringField::PublicationTitle,
                r#match: MetadataTextMatch::Contains {
                    value: value.to_string(),
                },
            })
        }
        other => {
            return Err(AppServiceError::invalid_request(format!(
                "unsupported filter field `{other}`"
            )));
        }
    })
}

fn numeric_match(range: FilterRange) -> AppServiceResult<NumericMatch> {
    match (range.min, range.max) {
        (Some(min), Some(max)) if min <= max => Ok(NumericMatch::Between { min, max }),
        (Some(_), Some(_)) => Err(AppServiceError::invalid_request(
            "range minimum must be less than or equal to range maximum",
        )),
        (Some(value), None) => Ok(NumericMatch::Gte { value }),
        (None, Some(value)) => Ok(NumericMatch::Lte { value }),
        (None, None) => Err(AppServiceError::invalid_request(
            "range clause must include a minimum or maximum",
        )),
    }
}

fn canonical_field(field: &str) -> String {
    field.trim().to_ascii_lowercase().replace('-', "_")
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        BasicSearchFilter, FilterClause, FilterClauseOperator, FilterRange, MetricComparison,
    };
    use atlas_domain::{
        MetadataNumberField, MetadataNumberMatch, MetadataPredicate, MetadataSetField,
        MetadataSetMatch, MetricMatch, NumericMatch, ScalarValue, SearchFilterNode,
    };

    use super::lower_basic_filter;

    #[test]
    fn lowers_required_traits_as_all_predicates() {
        let node = lower_basic_filter(Some(&BasicSearchFilter {
            clauses: vec![clause(
                "traits",
                FilterClauseOperator::IncludeAll,
                ["fire", "healing"],
            )],
        }))
        .expect("filter lowers")
        .expect("filter node");

        assert_eq!(
            node,
            SearchFilterNode::all_of(vec![trait_node("fire"), trait_node("healing")])
        );
    }

    #[test]
    fn lowers_any_traits_as_any_group() {
        let node = lower_basic_filter(Some(&BasicSearchFilter {
            clauses: vec![clause(
                "traits",
                FilterClauseOperator::IncludeAny,
                ["fire", "healing"],
            )],
        }))
        .expect("filter lowers")
        .expect("filter node");

        assert_eq!(
            node,
            SearchFilterNode::any_of(vec![trait_node("fire"), trait_node("healing")])
        );
    }

    #[test]
    fn lowers_excluded_rarity_as_not_any_group() {
        let node = lower_basic_filter(Some(&BasicSearchFilter {
            clauses: vec![clause(
                "rarity",
                FilterClauseOperator::ExcludeAny,
                ["rare", "unique"],
            )],
        }))
        .expect("filter lowers")
        .expect("filter node");

        assert_eq!(
            node,
            SearchFilterNode::not_of(SearchFilterNode::any_of(vec![
                SearchFilterNode::rarity(atlas_domain::NullableStringMatch::Eq {
                    value: "rare".to_string()
                }),
                SearchFilterNode::rarity(atlas_domain::NullableStringMatch::Eq {
                    value: "unique".to_string()
                })
            ]))
        );
    }

    #[test]
    fn lowers_level_range_and_metric_comparison() {
        let node = lower_basic_filter(Some(&BasicSearchFilter {
            clauses: vec![
                FilterClause {
                    id: "level".to_string(),
                    field: "level".to_string(),
                    operator: FilterClauseOperator::Range,
                    values: Vec::new(),
                    range: Some(FilterRange {
                        min: Some(1.0),
                        max: Some(5.0),
                    }),
                    metric: None,
                },
                FilterClause {
                    id: "metric".to_string(),
                    field: "metric".to_string(),
                    operator: FilterClauseOperator::MetricCompare,
                    values: Vec::new(),
                    range: None,
                    metric: Some(MetricComparison {
                        key: "hp.value".to_string(),
                        op: "eq".to_string(),
                        value: 40.0,
                    }),
                },
            ],
        }))
        .expect("filter lowers")
        .expect("filter node");

        assert_eq!(
            node,
            SearchFilterNode::all_of(vec![
                SearchFilterNode::metadata(MetadataPredicate::Number {
                    field: MetadataNumberField::Level,
                    r#match: MetadataNumberMatch::Between { min: 1.0, max: 5.0 },
                }),
                SearchFilterNode::metric(
                    "hp.value",
                    MetricMatch::Eq {
                        value: ScalarValue::Number(40.0),
                    }
                )
            ])
        );
    }

    #[test]
    fn rejects_inverted_range() {
        let error = lower_basic_filter(Some(&BasicSearchFilter {
            clauses: vec![FilterClause {
                id: "level".to_string(),
                field: "level".to_string(),
                operator: FilterClauseOperator::Range,
                values: Vec::new(),
                range: Some(FilterRange {
                    min: Some(5.0),
                    max: Some(1.0),
                }),
                metric: None,
            }],
        }))
        .expect_err("inverted range should fail");

        assert!(error.to_string().contains("range minimum"));
    }

    fn clause(
        field: &str,
        operator: FilterClauseOperator,
        values: impl IntoIterator<Item = &'static str>,
    ) -> FilterClause {
        FilterClause {
            id: field.to_string(),
            field: field.to_string(),
            operator,
            values: values.into_iter().map(str::to_string).collect(),
            range: None,
            metric: None,
        }
    }

    fn trait_node(value: &str) -> SearchFilterNode {
        SearchFilterNode::metadata(MetadataPredicate::Set {
            field: MetadataSetField::Traits,
            r#match: MetadataSetMatch::Includes {
                value: value.to_string(),
            },
        })
    }

    #[allow(dead_code)]
    fn _assert_numeric_match_is_exported(_: NumericMatch) {}
}
