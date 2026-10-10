use crate::cli::args::FilterOptions;
use atlas_domain::{QueryExpression, QueryLiteral, QueryPredicate, QuerySetMatch};
use serde_json::Value;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliFilterError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}
pub(crate) fn build_filter(
    options: &FilterOptions,
) -> Result<(Option<QueryPredicate>, Option<Value>), CliFilterError> {
    let fail = |message: String| CliFilterError {
        code: "invalid_filter",
        message,
    };
    let mut children = Vec::new();
    if let Some(expression) = &options.where_expression {
        children.push(
            atlas_index::parse_where(expression)
                .map_err(|e| fail(e.to_string()))?
                .predicate()
                .clone(),
        );
    }
    for (field, values) in [
        ("record.kind", &options.kinds),
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
    let predicate = match children.len() {
        0 => None,
        1 => children.pop(),
        _ => Some(QueryPredicate::new(QueryExpression::AllOf { children })),
    };
    if let Some(p) = &predicate {
        atlas_index::validate_query(p).map_err(|e| fail(e.to_string()))?;
    }
    let value = predicate
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|e| fail(e.to_string()))?;
    Ok((predicate, value))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn convenience_and_cel_share_validated_predicate() {
        let (p, _) = build_filter(&FilterOptions {
            where_expression: Some("actor.hp.maximum >= 80".into()),
            kinds: vec!["creature".into()],
            traits: vec!["undead".into()],
            ..Default::default()
        })
        .expect("valid");
        assert!(
            matches!(p.expect("predicate").expression,QueryExpression::AllOf{children} if children.len()==3)
        );
    }
    #[test]
    fn unknown_fields_and_unsupported_syntax_fail() {
        for s in ["missing.field == 2", "actor.hp.maximum + 3 > 5"] {
            assert!(
                build_filter(&FilterOptions {
                    where_expression: Some(s.into()),
                    ..Default::default()
                })
                .is_err()
            );
        }
    }
}
