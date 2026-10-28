//! CLI conveniences lower to the same validated predicate as CEL and the web editor.
use crate::cli::args::FilterOptions;
use atlas_app_model::{AppError, AppErrorCode, RecordRefResolutionView};
use atlas_app_service::AtlasAppService;
use atlas_domain::{
    QueryExpression, QueryLiteral, QueryPredicate, QueryRecordSelection, QueryRelationship,
    QueryRelationshipDirection, QuerySetMatch, RecordKey, assign_relationship_clause_ids,
};
use serde_json::Value;

pub(crate) fn build_filter(
    options: &FilterOptions,
) -> Result<(Option<QueryPredicate>, Option<Value>), AppError> {
    build_filter_using(options, |reference| {
        RecordKey::parse(reference).map_err(|_| {
            AppError::new(
                AppErrorCode::FilterInvalid,
                "--print-filter requires canonical keys for link filters. Resolve the name with `atlas record resolve <NAME> --json` and use a returned key.",
            )
        })
    })
}

pub(crate) fn resolve_filter(
    options: &FilterOptions,
    service: &AtlasAppService,
) -> Result<Option<QueryPredicate>, AppError> {
    build_filter_using(options, |reference| resolve_reference(service, reference))
        .map(|(predicate, _)| predicate)
}

pub(crate) fn resolve_reference(
    service: &AtlasAppService,
    reference: &str,
) -> Result<RecordKey, AppError> {
    match service
        .resolve_record_ref(reference.to_owned(), None)
        .map_err(|error| error.into_app_error())?
    {
        RecordRefResolutionView::Key(key) => Ok(key),
        RecordRefResolutionView::Miss => Err(AppError::new(
            AppErrorCode::RecordResolutionMiss,
            "No strict name or verified alias matched the selected record.",
        )),
        RecordRefResolutionView::Ambiguous(matches) => Err(AppError::new(
            AppErrorCode::RecordResolutionAmbiguous,
            "The selected record name is ambiguous. Resolve the name and use a returned key.",
        )
        .with_details(
            serde_json::json!({"candidates":matches.iter().map(|m| &m.record).collect::<Vec<_>>()}),
        )),
    }
}

fn build_filter_using(
    options: &FilterOptions,
    mut resolve: impl FnMut(&str) -> Result<RecordKey, AppError>,
) -> Result<(Option<QueryPredicate>, Option<Value>), AppError> {
    let invalid = |error: atlas_domain::QueryError| {
        AppError::new(AppErrorCode::FilterInvalid, error.to_string())
            .with_details(serde_json::json!({"query_error":error}))
    };
    let mut children = Vec::new();
    if let Some(expression) = &options.where_expression {
        children.push(
            atlas_index::parse_where(expression)
                .map_err(invalid)?
                .predicate()
                .clone(),
        );
    }
    for (field, values) in [
        ("kind", &options.kinds),
        ("source.pack.id", &options.pack_names),
        ("rarity", &options.rarities),
    ] {
        if !values.is_empty() {
            children.push(QueryPredicate::new(QueryExpression::In {
                field: field.into(),
                values: values.iter().cloned().map(QueryLiteral::String).collect(),
            }));
        }
    }
    if !options.traits.is_empty() {
        children.push(QueryPredicate::new(QueryExpression::SetMatch {
            field: "traits".into(),
            op: QuerySetMatch::IncludesAll,
            values: options.traits.clone(),
        }));
    }
    for (references, direction, include_contents) in [
        (
            &options.links_to,
            QueryRelationshipDirection::Incoming,
            true,
        ),
        (
            &options.linked_from,
            QueryRelationshipDirection::Outgoing,
            true,
        ),
        (
            &options.links_to_main_entry,
            QueryRelationshipDirection::Incoming,
            false,
        ),
        (
            &options.linked_from_main_entry,
            QueryRelationshipDirection::Outgoing,
            false,
        ),
    ] {
        for reference in references {
            children.push(QueryPredicate::new(QueryExpression::Relationship {
                relationship: QueryRelationship {
                    direction,
                    selection: QueryRecordSelection {
                        record_key: resolve(reference)?,
                        owners: Vec::new(),
                        field: None,
                        passage: None,
                        source_fingerprint: None,
                        spell_rank: None,
                    },
                    include_contents,
                },
            }));
        }
    }
    let mut predicate = match children.len() {
        0 => None,
        1 => children.pop(),
        _ => Some(QueryPredicate::new(QueryExpression::AllOf { children })),
    };
    if let Some(predicate) = &mut predicate {
        assign_relationship_clause_ids(predicate).map_err(invalid)?;
        atlas_index::validate_query(predicate).map_err(invalid)?;
    }
    let value = predicate
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| AppError::new(AppErrorCode::InternalError, error.to_string()))?;
    Ok((predicate, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn convenience_and_cel_share_validated_predicate() {
        let (predicate, _) = build_filter(&FilterOptions {
            where_expression: Some("defenses.hp.maximum >= 80".into()),
            kinds: vec!["creature".into()],
            traits: vec!["undead".into()],
            ..Default::default()
        })
        .expect("valid");
        let expected: Vec<QueryPredicate> = serde_json::from_value(serde_json::json!([
            {"kind":"compare", "field":"defenses.hp.maximum", "op":"gte", "value":80},
            {"kind":"in", "field":"kind", "values":["creature"]},
            {"kind":"set_match", "field":"traits", "op":"includes_all", "values":["undead"]}
        ]))
        .unwrap();
        let QueryExpression::AllOf { children } = predicate.unwrap().expression else {
            panic!("expected all constraints")
        };
        assert_eq!(
            children
                .iter()
                .map(|child| &child.expression)
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|child| &child.expression)
                .collect::<Vec<_>>()
        );
        assert!(children[0].clause_id.is_some());
    }
    #[test]
    fn unknown_fields_and_unsupported_syntax_fail() {
        for expression in ["missing.field == 2", "defenses.hp.maximum + 3 > 5"] {
            assert!(
                build_filter(&FilterOptions {
                    where_expression: Some(expression.into()),
                    ..Default::default()
                })
                .is_err()
            );
        }
    }
    #[test]
    fn named_flags_are_resolved_independently_and_keep_each_direction_and_scope() {
        let mut references = Vec::new();
        let (predicate, _) = build_filter_using(
            &FilterOptions {
                links_to: vec!["Armor".into()],
                linked_from_main_entry: vec!["Guardian".into()],
                ..Default::default()
            },
            |reference| {
                references.push(reference.to_owned());
                Ok(RecordKey::parse("actions:aaaaaaaaaaaaaaaa").unwrap())
            },
        )
        .unwrap();
        assert_eq!(references, ["Armor", "Guardian"]);
        let QueryExpression::AllOf { children } = predicate.unwrap().expression else {
            panic!("expected AND")
        };
        assert!(
            matches!(&children[0].expression, QueryExpression::Relationship {relationship} if relationship.direction == QueryRelationshipDirection::Incoming && relationship.include_contents)
        );
        assert!(
            matches!(&children[1].expression, QueryExpression::Relationship {relationship} if relationship.direction == QueryRelationshipDirection::Outgoing && !relationship.include_contents)
        );
        assert_ne!(children[0].clause_id, children[1].clause_id);
    }
    #[test]
    fn print_filter_does_not_resolve_names() {
        let error = build_filter(&FilterOptions {
            links_to: vec!["Leather Armor".into()],
            ..Default::default()
        })
        .unwrap_err();
        assert_eq!(error.code, AppErrorCode::FilterInvalid);
        assert!(error.message.contains("atlas record resolve"));
    }
}
