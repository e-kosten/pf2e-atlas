use super::catalog::{Descriptor, descriptor};
use atlas_domain::{
    QueryCompare, QueryError, QueryExpression, QueryFieldType, QueryLimits, QueryLiteral,
    QueryPredicate, QuerySetMatch,
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ValidatedQuery {
    pub(super) predicate: QueryPredicate,
}
impl ValidatedQuery {
    pub fn predicate(&self) -> &QueryPredicate {
        &self.predicate
    }
}
pub(crate) fn error(code: &str, message: impl Into<String>, field: Option<&str>) -> QueryError {
    QueryError {
        code: code.to_owned(),
        message: message.into(),
        field: field.map(str::to_owned),
        clause_id: None,
        source_span: None,
    }
}
pub(super) fn numeric_literal(
    number: &serde_json::Number,
) -> Result<rusqlite::types::Value, QueryError> {
    crate::numeric::numeric_value(number).map_err(|e| error("numeric_domain", e.to_string(), None))
}
pub(super) fn literal(d: &Descriptor, value: &QueryLiteral) -> Result<(), QueryError> {
    use QueryFieldType::*;
    match (&d.definition.field_type, value) {
        (String, QueryLiteral::String(value)) => {
            if !d.definition.choices.is_empty() && !d.definition.choices.contains(value) {
                Err(error(
                    "invalid_choice",
                    format!(
                        "{value:?} is not declared for {}; discover allowed choices with atlas filters fields",
                        d.definition.id
                    ),
                    Some(&d.definition.id),
                ))
            } else {
                Ok(())
            }
        }
        (Boolean, QueryLiteral::Boolean(_)) => Ok(()),
        (Number, QueryLiteral::Number(number)) => numeric_literal(number).map(|_| ()),
        _ => Err(error(
            "literal_type",
            format!(
                "{} requires a {:?} literal",
                d.definition.id, d.definition.field_type
            ),
            Some(&d.definition.id),
        )),
    }
}
pub fn validate_query(predicate: &QueryPredicate) -> Result<ValidatedQuery, QueryError> {
    let mut nodes = 0;
    let mut bytes = 0;
    let mut ids = HashSet::new();
    let mut scopes = HashSet::new();
    fn visit(
        p: &QueryPredicate,
        scope: Option<&str>,
        depth: usize,
        nodes: &mut usize,
        ids: &mut HashSet<String>,
        scopes: &mut HashSet<String>,
        bytes: &mut usize,
    ) -> Result<(), QueryError> {
        let limits = QueryLimits::default();
        *nodes += 1;
        if *nodes > limits.nodes || depth > limits.depth {
            return Err(error(
                "query_limit",
                "Query exceeds the node/depth limit",
                None,
            ));
        }
        if let Some(id) = &p.clause_id {
            if id.is_empty() || id.len() > 128 || !ids.insert(id.clone()) {
                return Err(error(
                    "clause_id",
                    "Clause IDs must be nonempty, bounded and unique",
                    None,
                ));
            }
            *bytes += id.len();
        }
        let literal_bytes = |v: &QueryLiteral| match v {
            QueryLiteral::String(s) => s.len(),
            QueryLiteral::Number(n) => n.to_string().len(),
            QueryLiteral::Boolean(_) => 5,
        };
        *bytes += match &p.expression {
            QueryExpression::Compare { field, value, .. } => field.len() + literal_bytes(value),
            QueryExpression::In { field, values } => {
                field.len() + values.iter().map(literal_bytes).sum::<usize>()
            }
            QueryExpression::SetMatch { field, values, .. } => {
                field.len() + values.iter().map(String::len).sum::<usize>()
            }
            QueryExpression::StateMatch { field, .. } => field.len(),
            QueryExpression::Exists {
                collection,
                scope_id,
                ..
            } => collection.len() + scope_id.len(),
            _ => 0,
        };
        if *bytes > limits.source_bytes {
            return Err(error(
                "query_limit",
                "Structured query exceeds the literal/identifier byte budget",
                None,
            ));
        }
        let resolve = |field: &str| {
            descriptor(scope, field)?.ok_or_else(|| {
                error(
                    "unknown_field",
                    format!(
                        "Unsupported field {field:?} in {} scope; use atlas filters fields",
                        scope.unwrap_or("root")
                    ),
                    Some(field),
                )
            })
        };
        let bounded = |n: usize| {
            if n == 0 || n > limits.literal_list {
                Err(error(
                    "literal_list",
                    "Literal lists must contain 1..=128 values",
                    None,
                ))
            } else {
                Ok(())
            }
        };
        let result = (|| match &p.expression {
            QueryExpression::BooleanConstant { .. } => Ok(()),
            QueryExpression::Compare { field, op, value } => {
                let d = resolve(field)?;
                literal(d, value)?;
                if d.definition.field_type != QueryFieldType::Number
                    && !matches!(op, QueryCompare::Eq | QueryCompare::Neq)
                {
                    Err(error(
                        "operator_type",
                        "Only equality/inequality is supported for this field",
                        Some(field),
                    ))
                } else {
                    Ok(())
                }
            }
            QueryExpression::In { field, values } => {
                bounded(values.len())?;
                let d = resolve(field)?;
                if d.definition.field_type != QueryFieldType::String {
                    return Err(error(
                        "operator_type",
                        "In requires a string field",
                        Some(field),
                    ));
                }
                for v in values {
                    literal(d, v)?;
                }
                Ok(())
            }
            QueryExpression::SetMatch { field, op, values } => {
                bounded(values.len())?;
                let d = resolve(field)?;
                if d.definition.field_type != QueryFieldType::Set
                    || (*op == QuerySetMatch::Includes && values.len() != 1)
                {
                    return Err(error(
                        "operator_type",
                        "Set membership requires a set; Includes accepts one literal",
                        Some(field),
                    ));
                }
                for value in values {
                    if !d.definition.choices.is_empty() && !d.definition.choices.contains(value) {
                        return Err(error(
                            "invalid_choice",
                            format!("Undeclared set choice {value:?}"),
                            Some(field),
                        ));
                    }
                }
                Ok(())
            }
            QueryExpression::StateMatch { field, .. } => resolve(field).map(|_| ()),
            QueryExpression::Exists {
                collection,
                scope_id,
                predicate,
            } => {
                if scope.is_some() {
                    return Err(error(
                        "nested_exists",
                        "Nested collection scopes are unsupported",
                        Some(collection),
                    ));
                }
                let d = resolve(collection)?;
                if d.definition.field_type != QueryFieldType::Collection {
                    return Err(error(
                        "operator_type",
                        "Exists requires a selected collection",
                        Some(collection),
                    ));
                }
                if scope_id.is_empty() || scope_id.len() > 128 || !scopes.insert(scope_id.clone()) {
                    return Err(error(
                        "scope_id",
                        "Scope IDs must be nonempty, bounded and unique",
                        Some(collection),
                    ));
                }
                visit(
                    predicate,
                    Some(collection),
                    depth + 1,
                    nodes,
                    ids,
                    scopes,
                    bytes,
                )
            }
            QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
                if children.is_empty() {
                    return Err(error(
                        "empty_boolean",
                        "Boolean groups must be nonempty",
                        None,
                    ));
                }
                for child in children {
                    visit(child, scope, depth + 1, nodes, ids, scopes, bytes)?;
                }
                Ok(())
            }
            QueryExpression::Not { predicate } => {
                visit(predicate, scope, depth + 1, nodes, ids, scopes, bytes)
            }
        })();
        result.map_err(|mut e| {
            if e.clause_id.is_none() {
                e.clause_id = p.clause_id.clone();
            }
            e
        })
    }
    visit(
        predicate,
        None,
        0,
        &mut nodes,
        &mut ids,
        &mut scopes,
        &mut bytes,
    )?;
    Ok(ValidatedQuery {
        predicate: predicate.clone(),
    })
}
