//! Contextual facets operate on named projections and exact counterfactual sets.
use crate::query::{
    catalog::{Binding, descriptor, descriptors},
    compiler::{bound_field, compile_node, compile_predicate, member_relation},
    validation::error,
};
use crate::{
    IndexError,
    query::{ValidatedQuery, validate_query},
};
use atlas_domain::{
    RecordKey,
    query::{
        QueryCompare, QueryExpression, QueryFieldState, QueryFieldType, QueryLiteral,
        QueryPredicate, QuerySetMatch,
    },
    query_discovery::{QueryFieldCounts, QueryStateCount, QueryValueOption, QueryValueOptions},
};
use rusqlite::{Connection, params_from_iter, types::Value};

/// Search supplies the clause-removed candidate keys for semantic/hybrid facets.
/// The bounded flag prevents labeling their counts as full-corpus totals.
#[derive(Debug, Clone, Default)]
pub struct QueryFacetContext {
    pub eligible_keys: Option<Vec<RecordKey>>,
    pub bounded_candidates: bool,
    pub prefer_remaster: bool,
}
#[derive(Debug, Clone)]
pub struct QueryValuesRequest {
    pub field: String,
    pub clause_id: Option<String>,
    pub query: ValidatedQuery,
    pub context: QueryFacetContext,
    pub text: Option<String>,
    pub offset: usize,
    pub limit: usize,
}
#[derive(Debug, Clone)]
pub struct QueryCountsRequest {
    pub field: String,
    pub clause_id: Option<String>,
    pub query: ValidatedQuery,
    pub context: QueryFacetContext,
}
fn key_filter(context: &QueryFacetContext, parameters: &mut Vec<Value>) -> String {
    match &context.eligible_keys {
        None => "1".to_owned(),
        Some(keys) if keys.is_empty() => "0".to_owned(),
        Some(keys) => {
            let slots = keys
                .iter()
                .map(|key| {
                    parameters.push(Value::Text(key.to_string()));
                    format!("?{}", parameters.len())
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("r.key IN ({slots})")
        }
    }
}
fn field(id: &str) -> Result<&'static crate::query::catalog::Descriptor, IndexError> {
    descriptors()?
        .iter()
        .find(|d| d.definition.id == id)
        .ok_or_else(|| {
            error(
                "unknown_field",
                format!("Unsupported field {id}; use atlas filters fields"),
                Some(id),
            )
            .into()
        })
}
fn scalar_literal(value: Value) -> Result<QueryLiteral, IndexError> {
    match value {
        Value::Integer(i) => Ok(QueryLiteral::Number(i.into())),
        Value::Real(f) => serde_json::Number::from_f64(f)
            .map(QueryLiteral::Number)
            .ok_or_else(|| IndexError::Invalid("nonfinite facet value".into())),
        Value::Text(s) => Ok(QueryLiteral::String(s)),
        _ => Err(IndexError::Invalid("unexpected facet value type".into())),
    }
}
fn occurrences(d: &crate::query::catalog::Descriptor) -> Result<(String, String), IndexError> {
    match d.definition.scope.as_deref() {
        None => Ok(("records r WHERE ".to_owned(), String::new())),
        Some(scope) => {
            let collection = descriptor(None, scope)?
                .ok_or_else(|| IndexError::Invalid("missing catalog collection".into()))?;
            Ok((
                format!(
                    "records r, {} AND ",
                    member_relation(collection, None, "c")?
                ),
                scope.to_owned(),
            ))
        }
    }
}
pub(crate) fn field_counts(
    connection: &Connection,
    request: &QueryCountsRequest,
) -> Result<QueryFieldCounts, IndexError> {
    let id = &request.field;
    let d = field(id)?;
    let (query, member) = if let Some(clause_id) = &request.clause_id {
        let (base, _, member) = remove_clause(&request.query, clause_id, d)?;
        (base, member)
    } else {
        (request.query.clone(), None)
    };
    let b = bound_field(d, d.definition.scope.as_deref());
    let (from, _) = occurrences(d)?;
    let mut compiled = compile_predicate(&query)?;
    let keys = key_filter(&request.context, &mut compiled.parameters);
    let member = member
        .as_ref()
        .map(|p| compile_node(p, d.definition.scope.as_deref(), &mut compiled.parameters))
        .transpose()?
        .unwrap_or_else(|| "1".to_owned());
    let sql = format!(
        "SELECT {},COUNT(*),COUNT(DISTINCT r.record_id) FROM {from} ({}) AND ({keys}) AND ({member}) GROUP BY 1",
        b.state, compiled.expression
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(&compiled.parameters), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u64>(1)?,
                r.get::<_, u64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut states = Vec::new();
    for state in [
        QueryFieldState::Value,
        QueryFieldState::Missing,
        QueryFieldState::Null,
        QueryFieldState::Invalid,
        QueryFieldState::NotApplicable,
    ] {
        let counts = rows.iter().find(|r| r.0 == state.as_str());
        states.push(QueryStateCount {
            state,
            occurrences: counts.map(|r| r.1).unwrap_or(0),
            distinct_roots: counts.map(|r| r.2).unwrap_or(0),
        });
    }
    let (minimum, maximum) = if d.definition.field_type == QueryFieldType::Number {
        let sql = format!(
            "SELECT MIN({}),MAX({}) FROM {from} ({}) AND ({keys}) AND ({member}) AND ({})='value'",
            b.value, b.value, compiled.expression, b.state
        );
        connection
            .query_row(&sql, params_from_iter(&compiled.parameters), |r| {
                Ok((r.get::<_, Value>(0)?, r.get::<_, Value>(1)?))
            })
            .map(|(min, max)| {
                fn number(v: Value) -> Option<serde_json::Number> {
                    match v {
                        Value::Integer(i) => Some(i.into()),
                        Value::Real(f) => serde_json::Number::from_f64(f),
                        _ => None,
                    }
                }
                (number(min), number(max))
            })?
    } else {
        (None, None)
    };
    Ok(QueryFieldCounts {
        field: id.to_owned(),
        counting_scope: d
            .definition
            .scope
            .clone()
            .unwrap_or_else(|| "root".to_owned()),
        states,
        exhaustive: !request.context.bounded_candidates,
        count_basis: if request.context.bounded_candidates {
            "clause-removed semantic candidate window; before remaster suppression"
        } else {
            "clause-removed eligible source roots; before remaster suppression"
        }
        .to_owned(),
        minimum,
        maximum,
    })
}

fn remove_clause(
    query: &ValidatedQuery,
    id: &str,
    expected: &crate::query::catalog::Descriptor,
) -> Result<(ValidatedQuery, QueryPredicate, Option<QueryPredicate>), IndexError> {
    let mut removed = None;
    let mut member_remainder = None;
    fn walk(
        p: &QueryPredicate,
        id: &str,
        scope: Option<&str>,
        allowed: bool,
        removed: &mut Option<QueryPredicate>,
        member: &mut Option<QueryPredicate>,
        expected: &crate::query::catalog::Descriptor,
    ) -> Result<QueryPredicate, IndexError> {
        if p.clause_id.as_deref() == Some(id) {
            if !allowed || scope != expected.definition.scope.as_deref() {
                return Err(error(
                    "unsupported_facet_context",
                    "Contextual facets require a positive conjunction and at most one Exists scope",
                    Some(&expected.definition.id),
                )
                .into());
            }
            let actual = match &p.expression {
                QueryExpression::Compare { field, .. }
                | QueryExpression::In { field, .. }
                | QueryExpression::SetMatch { field, .. }
                | QueryExpression::StateMatch { field, .. } => field,
                _ => {
                    return Err(error(
                        "unsupported_facet_context",
                        "Facet target must be a field clause",
                        Some(&expected.definition.id),
                    )
                    .into());
                }
            };
            if actual != &expected.definition.path
                || matches!(
                    &p.expression,
                    QueryExpression::SetMatch {
                        op: QuerySetMatch::ExcludesAny,
                        ..
                    } | QueryExpression::Compare {
                        op: QueryCompare::Neq,
                        ..
                    }
                )
            {
                return Err(error(
                    "unsupported_facet_context",
                    "Facet target must include the selected field",
                    Some(&expected.definition.id),
                )
                .into());
            }
            *removed = Some(p.clone());
            return Ok(QueryPredicate::boolean(true));
        }
        let mut result = p.clone();
        result.expression = match &p.expression {
            QueryExpression::AllOf { children } => QueryExpression::AllOf {
                children: children
                    .iter()
                    .map(|p| walk(p, id, scope, allowed, removed, member, expected))
                    .collect::<Result<_, _>>()?,
            },
            QueryExpression::AnyOf { children } => QueryExpression::AnyOf {
                children: children
                    .iter()
                    .map(|p| walk(p, id, scope, false, removed, member, expected))
                    .collect::<Result<_, _>>()?,
            },
            QueryExpression::Not { predicate } => QueryExpression::Not {
                predicate: Box::new(walk(
                    predicate, id, scope, false, removed, member, expected,
                )?),
            },
            QueryExpression::Exists {
                collection,
                scope_id,
                predicate,
            } => {
                let previously_removed = removed.is_some();
                let child = walk(
                    predicate,
                    id,
                    Some(collection),
                    allowed,
                    removed,
                    member,
                    expected,
                )?;
                if !previously_removed && removed.is_some() {
                    *member = Some(child.clone());
                }
                QueryExpression::Exists {
                    collection: collection.clone(),
                    scope_id: scope_id.clone(),
                    predicate: Box::new(child),
                }
            }
            expression => expression.clone(),
        };
        Ok(result)
    }
    let base = walk(
        query.predicate(),
        id,
        None,
        true,
        &mut removed,
        &mut member_remainder,
        expected,
    )?;
    let target = removed.ok_or_else(|| {
        error(
            "clause_id",
            "The target clause ID does not exist in the request",
            Some(&expected.definition.id),
        )
    })?;
    Ok((validate_query(&base)?, target, member_remainder))
}
/// The same supported self-exclusion used by counts/values supplies search's
/// clause-removed candidate universe. Physical SQL bindings stay private.
pub fn facet_base_query(
    field_id: &str,
    clause_id: Option<&str>,
    query: &ValidatedQuery,
) -> Result<ValidatedQuery, IndexError> {
    let descriptor = field(field_id)?;
    match clause_id {
        Some(id) => remove_clause(query, id, descriptor).map(|(base, _, _)| base),
        None => Ok(query.clone()),
    }
}
fn replace_clause(p: &QueryPredicate, id: &str, replacement: &QueryPredicate) -> QueryPredicate {
    if p.clause_id.as_deref() == Some(id) {
        return replacement.clone();
    }
    let mut p = p.clone();
    p.expression = match p.expression {
        QueryExpression::AllOf { children } => QueryExpression::AllOf {
            children: children
                .iter()
                .map(|p| replace_clause(p, id, replacement))
                .collect(),
        },
        QueryExpression::AnyOf { children } => QueryExpression::AnyOf {
            children: children
                .iter()
                .map(|p| replace_clause(p, id, replacement))
                .collect(),
        },
        QueryExpression::Not { predicate } => QueryExpression::Not {
            predicate: Box::new(replace_clause(&predicate, id, replacement)),
        },
        QueryExpression::Exists {
            collection,
            scope_id,
            predicate,
        } => QueryExpression::Exists {
            collection,
            scope_id,
            predicate: Box::new(replace_clause(&predicate, id, replacement)),
        },
        other => other,
    };
    p
}
fn candidate_clause(
    d: &crate::query::catalog::Descriptor,
    value: &QueryLiteral,
) -> Result<QueryPredicate, IndexError> {
    Ok(QueryPredicate::new(
        if d.definition.field_type == QueryFieldType::Set {
            let QueryLiteral::String(value) = value else {
                return Err(IndexError::Invalid(
                    "set facet has a non-string catalog value".into(),
                ));
            };
            QueryExpression::SetMatch {
                field: d.definition.path.clone(),
                op: QuerySetMatch::Includes,
                values: vec![value.clone()],
            }
        } else {
            QueryExpression::Compare {
                field: d.definition.path.clone(),
                op: QueryCompare::Eq,
                value: value.clone(),
            }
        },
    ))
}
fn selected(target: Option<&QueryPredicate>) -> Vec<QueryLiteral> {
    match target.map(|p| &p.expression) {
        Some(QueryExpression::Compare { value, .. }) => vec![value.clone()],
        Some(QueryExpression::In { values, .. }) => values.clone(),
        Some(QueryExpression::SetMatch { values, .. }) => {
            values.iter().cloned().map(QueryLiteral::String).collect()
        }
        _ => Vec::new(),
    }
}
pub(crate) fn field_values(
    connection: &Connection,
    request: &QueryValuesRequest,
) -> Result<QueryValueOptions, IndexError> {
    if request.limit == 0 || request.limit > 200 || request.offset > 100_000 {
        return Err(error(
            "discovery_limit",
            "Value discovery requires limit1..=200 and bounded offset",
            Some(&request.field),
        )
        .into());
    }
    let d = field(&request.field)?;
    if matches!(
        d.definition.field_type,
        QueryFieldType::Number | QueryFieldType::Collection
    ) {
        return Err(error(
            "discovery_type",
            "Use field availability/numeric statistics or scoped member values for this descriptor",
            Some(&request.field),
        )
        .into());
    }
    let (base, target, member) = if let Some(id) = &request.clause_id {
        let (base, target, member) = remove_clause(&request.query, id, d)?;
        (base, Some(target), member)
    } else {
        (request.query.clone(), None, None)
    };
    let mut compiled = compile_predicate(&base)?;
    let keys = key_filter(&request.context, &mut compiled.parameters);
    let (from, _) = occurrences(d)?;
    let b = bound_field(d, d.definition.scope.as_deref());
    let member = member
        .as_ref()
        .map(|p| compile_node(p, d.definition.scope.as_deref(), &mut compiled.parameters))
        .transpose()?
        .unwrap_or_else(|| "1".to_owned());
    // SQLite has no LATERAL subqueries. A correlated json_group_array carries
    // the finite relational values to json_each without parsing source bodies.
    let values_expr = if matches!(&d.binding, Binding::Set { .. }) {
        format!(
            "(SELECT json_group_array(t.value) FROM {})",
            member_relation(d, d.definition.scope.as_deref(), "t")?
        )
    } else {
        format!("json_array({})", b.value)
    };
    let sql = format!(
        "WITH known AS (SELECT {values_expr} AS values_json FROM {from} ({}) AND ({keys}) AND ({member}) AND ({})='value') SELECT DISTINCT v.value FROM known,json_each(known.values_json) v ORDER BY v.value",
        compiled.expression, b.state
    );
    let mut statement = connection.prepare(&sql)?;
    let mut values = statement
        .query_map(params_from_iter(&compiled.parameters), |r| {
            r.get::<_, Value>(0)
        })?
        .map(|r| {
            scalar_literal(r?).map(|v| {
                if d.definition.field_type == QueryFieldType::Boolean {
                    match v {
                        QueryLiteral::Number(n) => QueryLiteral::Boolean(n.as_i64() == Some(1)),
                        other => other,
                    }
                } else {
                    v
                }
            })
        })
        .collect::<Result<Vec<_>, IndexError>>()?;
    // Closed choices absent from this artifact and selected absent open values
    // remain discoverable rather than being silently erased by row enumeration.
    for value in d
        .definition
        .choices
        .iter()
        .cloned()
        .map(QueryLiteral::String)
        .chain(if d.definition.field_type == QueryFieldType::Boolean {
            vec![QueryLiteral::Boolean(false), QueryLiteral::Boolean(true)]
        } else {
            vec![]
        })
        .chain(selected(target.as_ref()))
    {
        if !values.contains(&value) {
            values.push(value);
        }
    }
    values.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
    let selected = selected(target.as_ref());
    if let Some(text) = &request.text {
        values.retain(|value|selected.contains(value) || matches!(value,QueryLiteral::String(s) if s.to_lowercase().contains(&text.to_lowercase())));
    }
    let total_values = values.len() as u64;
    let mut options = Vec::new();
    let mut page = values
        .into_iter()
        .skip(request.offset)
        .take(request.limit)
        .collect::<Vec<_>>();
    // Selected options are state, not merely a page of suggestions. Retain them
    // even when text search or pagination would otherwise hide them.
    for value in &selected {
        if !page.contains(value) {
            page.push(value.clone());
        }
    }
    for value in page {
        let candidate = candidate_clause(d, &value)?;
        let p = if let Some(id) = &request.clause_id {
            replace_clause(request.query.predicate(), id, &candidate)
        } else {
            let candidate = if let Some(scope) = &d.definition.scope {
                QueryPredicate::new(QueryExpression::Exists {
                    collection: scope.clone(),
                    scope_id: "discovery-option".to_owned(),
                    predicate: Box::new(candidate),
                })
            } else {
                candidate
            };
            QueryPredicate::new(QueryExpression::AllOf {
                children: vec![request.query.predicate().clone(), candidate],
            })
        };
        let accepted = validate_query(&p)?;
        let mut compiled = compile_predicate(&accepted)?;
        let keys = key_filter(&request.context, &mut compiled.parameters);
        let preference = if request.context.prefer_remaster {
            "WHERE NOT EXISTS(SELECT 1 FROM remaster_pairs p JOIN matches replacement ON replacement.record_id=p.remaster_record_id WHERE p.legacy_record_id=m.record_id)"
        } else {
            ""
        };
        let sql = format!(
            "WITH matches AS (SELECT r.record_id FROM records r WHERE ({}) AND ({keys})) SELECT COUNT(*) FROM matches m {preference}",
            compiled.expression
        );
        let count = connection.query_row(&sql, params_from_iter(&compiled.parameters), |r| {
            r.get::<_, u64>(0)
        })?;
        options.push(QueryValueOption {
            selected: selected.contains(&value),
            value,
            distinct_roots: count,
        });
    }
    Ok(QueryValueOptions {
        field: request.field.clone(),
        options,
        total_values,
        exhaustive: !request.context.bounded_candidates,
        count_basis: if request.context.bounded_candidates {
            "clause-removed semantic candidate window"
        } else if request.clause_id.is_some() {
            "exact counterfactual displayed roots"
        } else {
            "unscoped value enumeration with active root eligibility"
        }
        .to_owned(),
    })
}
