//! Contextual facets operate on named projections and exact counterfactual sets.
use crate::query::{
    catalog::{Binding, descriptor, descriptors},
    compiler::{CompiledPredicate, bound_field, compile_node, member_relation},
    validation::error,
};
use crate::{
    IndexError,
    query::{ValidatedQuery, validate_query},
};
use atlas_domain::{
    FacetUnavailableReason, QueryFacetOutcome, QueryVocabulary, QueryVocabularyBasis,
    QueryVocabularyOption, RecordKey,
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
    pub only_matching: bool,
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
#[derive(Debug, Clone)]
pub struct QueryValueCountsRequest {
    pub field: String,
    pub clause_id: Option<String>,
    pub query: ValidatedQuery,
    pub context: QueryFacetContext,
    pub values: Vec<QueryLiteral>,
}
#[derive(Debug, Clone)]
pub struct QueryVocabularyRequest {
    pub field: String,
    pub eligible_keys: Option<Vec<RecordKey>>,
    pub text: Option<String>,
    pub offset: usize,
    pub limit: usize,
    pub selected_values: Vec<QueryLiteral>,
}

/// Validate value discovery before optional capability classification or model work.
pub fn validate_value_discovery(
    field_id: &str,
    offset: usize,
    limit: usize,
    text: Option<&str>,
) -> Result<(), IndexError> {
    if limit == 0 || limit > 200 || offset > 100_000 {
        return Err(error(
            "discovery_limit",
            "Value discovery requires limit 1..=200 and bounded offset",
            Some(field_id),
        )
        .into());
    }
    if text.is_some_and(|text| text.len() > atlas_domain::QueryLimits::default().source_bytes) {
        return Err(error(
            "discovery_limit",
            "Vocabulary search text exceeds the bounded request size",
            Some(field_id),
        )
        .into());
    }
    let d = field(field_id)?;
    if matches!(
        d.definition.field_type,
        QueryFieldType::Number | QueryFieldType::Collection
    ) {
        return Err(error(
            "discovery_type",
            "Use availability/numeric statistics or scoped member values for this field",
            Some(field_id),
        )
        .into());
    }
    Ok(())
}
fn compile_eligibility(
    query: &ValidatedQuery,
    context: &QueryFacetContext,
) -> Result<CompiledPredicate, IndexError> {
    let (eligible, parameters) =
        crate::read_units::eligible_sql(query, context.eligible_keys.as_deref())?;
    Ok(CompiledPredicate {
        expression: format!("r.record_id IN ({eligible})"),
        parameters,
        proofs: Vec::new(),
    })
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
) -> Result<QueryFacetOutcome<QueryFieldCounts>, IndexError> {
    let id = &request.field;
    let d = field(id)?;

    let (query, member) = if let Some(clause_id) = &request.clause_id {
        let (base, _, member) = match remove_clause(&request.query, clause_id, d)? {
            QueryFacetOutcome::Available { value } => value,
            QueryFacetOutcome::Unavailable { reason } => {
                return Ok(QueryFacetOutcome::Unavailable { reason });
            }
        };
        (base, member)
    } else {
        (request.query.clone(), None)
    };
    let b = bound_field(d, d.definition.scope.as_deref());
    let (from, _) = occurrences(d)?;
    let mut compiled = compile_eligibility(&query, &request.context)?;
    let member = member
        .as_ref()
        .map(|p| compile_node(p, d.definition.scope.as_deref(), &mut compiled.parameters))
        .transpose()?
        .unwrap_or_else(|| "1".to_owned());
    let sql = format!(
        "SELECT {},COUNT(*),COUNT(DISTINCT r.record_id) FROM {from} ({}) AND ({member}) GROUP BY 1",
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
            "SELECT MIN({}),MAX({}) FROM {from} ({}) AND ({member}) AND ({})='value'",
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
    Ok(QueryFacetOutcome::Available {
        value: QueryFieldCounts {
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
        },
    })
}

type RemovedClause = (ValidatedQuery, QueryPredicate, Option<QueryPredicate>);
struct ClauseRemoval {
    target: Option<QueryPredicate>,
    member: Option<QueryPredicate>,
    unavailable: Option<FacetUnavailableReason>,
}
fn remove_clause(
    query: &ValidatedQuery,
    id: &str,
    expected: &crate::query::catalog::Descriptor,
) -> Result<QueryFacetOutcome<RemovedClause>, IndexError> {
    let mut removal = ClauseRemoval {
        target: None,
        member: None,
        unavailable: None,
    };
    fn walk(
        p: &QueryPredicate,
        id: &str,
        scope: Option<&str>,
        allowed: bool,
        removal: &mut ClauseRemoval,
        expected: &crate::query::catalog::Descriptor,
    ) -> Result<QueryPredicate, IndexError> {
        if p.clause_id.as_deref() == Some(id) {
            if scope != expected.definition.scope.as_deref() {
                return Err(error(
                    "facet_target_scope",
                    "The target condition does not use the requested field scope",
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
                        "facet_target_type",
                        "Facet target must be a field clause",
                        Some(&expected.definition.id),
                    )
                    .into());
                }
            };
            if actual != &expected.definition.path {
                return Err(error(
                    "facet_target_field",
                    "The target condition does not use the requested field",
                    Some(&expected.definition.id),
                )
                .into());
            }
            if !allowed {
                removal.unavailable = Some(FacetUnavailableReason::NonConjunctiveContext);
            } else if matches!(
                &p.expression,
                QueryExpression::SetMatch {
                    op: QuerySetMatch::ExcludesAny,
                    ..
                } | QueryExpression::Compare {
                    op: QueryCompare::Neq,
                    ..
                }
            ) {
                removal.unavailable = Some(FacetUnavailableReason::NegativeCondition);
            }
            removal.target = Some(p.clone());
            return Ok(QueryPredicate::boolean(true));
        }
        let mut result = p.clone();
        result.expression = match &p.expression {
            QueryExpression::AllOf { children } => QueryExpression::AllOf {
                children: children
                    .iter()
                    .map(|p| walk(p, id, scope, allowed, removal, expected))
                    .collect::<Result<_, _>>()?,
            },
            QueryExpression::AnyOf { children } => QueryExpression::AnyOf {
                children: children
                    .iter()
                    .map(|p| walk(p, id, scope, false, removal, expected))
                    .collect::<Result<_, _>>()?,
            },
            QueryExpression::Not { predicate } => QueryExpression::Not {
                predicate: Box::new(walk(predicate, id, scope, false, removal, expected)?),
            },
            QueryExpression::Exists {
                collection,
                scope_id,
                predicate,
            } => {
                let previously_removed = removal.target.is_some();
                let child = walk(predicate, id, Some(collection), allowed, removal, expected)?;
                if !previously_removed && removal.target.is_some() {
                    removal.member = Some(child.clone());
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
    let base = walk(query.predicate(), id, None, true, &mut removal, expected)?;
    let target = removal.target.ok_or_else(|| {
        error(
            "clause_id",
            "The target clause ID does not exist in the request",
            Some(&expected.definition.id),
        )
    })?;
    if expected.definition.scope.is_none() {
        return match root_field_context(query.predicate(), &expected.definition.path) {
            Some(base) => Ok(QueryFacetOutcome::Available {
                value: (validate_query(&base)?, target, None),
            }),
            None => Ok(QueryFacetOutcome::Unavailable {
                reason: FacetUnavailableReason::NonConjunctiveContext,
            }),
        };
    }
    if let Some(reason) = removal.unavailable {
        return Ok(QueryFacetOutcome::Unavailable { reason });
    }
    Ok(QueryFacetOutcome::Available {
        value: (validate_query(&base)?, target, removal.member),
    })
}
/// The same supported self-exclusion used by counts/values supplies search's
/// clause-removed candidate universe. Physical SQL bindings stay private.
pub fn facet_base_query(
    field_id: &str,
    clause_id: Option<&str>,
    query: &ValidatedQuery,
) -> Result<QueryFacetOutcome<ValidatedQuery>, IndexError> {
    let descriptor = field(field_id)?;

    match clause_id {
        Some(id) => Ok(remove_clause(query, id, descriptor)?.map(|(base, _, _)| base)),
        None => Ok(QueryFacetOutcome::Available {
            value: query.clone(),
        }),
    }
}
// Remove a root field's complete selection only through conjunctions. A NOT
// around one field leaf is a negative selection; compound NOT/OR is not a facet.
fn root_field_context(p: &QueryPredicate, field: &str) -> Option<QueryPredicate> {
    fn is_field(p: &QueryPredicate, expected: &str) -> bool {
        matches!(&p.expression,
            QueryExpression::Compare { field, .. } | QueryExpression::In { field, .. }
            | QueryExpression::SetMatch { field, .. } | QueryExpression::StateMatch { field, .. }
            if field == expected)
    }
    fn contains(p: &QueryPredicate, field: &str) -> bool {
        if is_field(p, field) {
            return true;
        }
        match &p.expression {
            QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
                children.iter().any(|child| contains(child, field))
            }
            QueryExpression::Not { predicate } => contains(predicate, field),
            _ => false, // Exists belongs to a different scope.
        }
    }
    if is_field(p, field) {
        return Some(QueryPredicate::boolean(true));
    }
    match &p.expression {
        QueryExpression::AllOf { children } => Some(QueryPredicate::new(QueryExpression::AllOf {
            children: children
                .iter()
                .map(|child| root_field_context(child, field))
                .collect::<Option<Vec<_>>>()?,
        })),
        QueryExpression::Not { predicate } if is_field(predicate, field) => {
            Some(QueryPredicate::boolean(true))
        }
        QueryExpression::Not { .. } | QueryExpression::AnyOf { .. } if contains(p, field) => None,
        _ => Some(p.clone()),
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
                return Err(error(
                    "literal_type",
                    "Set vocabulary selections must be strings",
                    Some(&d.definition.id),
                )
                .into());
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
/// Validate every requested literal before optional discovery availability.
pub fn validate_value_counts(field_id: &str, values: &[QueryLiteral]) -> Result<(), IndexError> {
    validate_value_discovery(field_id, 0, 1, None)?;
    if values.len() > atlas_domain::QueryLimits::default().literal_list {
        return Err(error(
            "discovery_limit",
            "Too many values for contextual counts",
            Some(field_id),
        )
        .into());
    }
    let d = field(field_id)?;
    for value in values {
        let candidate = candidate_clause(d, value)?;
        let candidate = if let Some(scope) = &d.definition.scope {
            QueryPredicate::new(QueryExpression::Exists {
                collection: scope.clone(),
                scope_id: "count-value".into(),
                predicate: Box::new(candidate),
            })
        } else {
            candidate
        };
        validate_query(&candidate)?;
    }
    Ok(())
}
pub(crate) fn field_value_counts(
    connection: &Connection,
    request: &QueryValueCountsRequest,
) -> Result<QueryFacetOutcome<QueryValueOptions>, IndexError> {
    validate_value_counts(&request.field, &request.values)?;
    let d = field(&request.field)?;

    let target = match request.clause_id.as_deref() {
        Some(id) => match remove_clause(&request.query, id, d)? {
            QueryFacetOutcome::Available {
                value: (_, target, _),
            } => Some(target),
            QueryFacetOutcome::Unavailable { reason } => {
                return Ok(QueryFacetOutcome::Unavailable { reason });
            }
        },
        None => None,
    };
    let mut values = Vec::new();
    for value in &request.values {
        if !values.contains(value) {
            values.push(value.clone());
        }
    }
    let total_values = values.len() as u64;
    Ok(QueryFacetOutcome::Available {
        value: QueryValueOptions {
            field: request.field.clone(),
            options: count_options(
                connection,
                d,
                &request.query,
                request.clause_id.as_deref(),
                &request.context,
                values,
                &selected(target.as_ref()),
            )?,
            total_values,
            exhaustive: !request.context.bounded_candidates,
            count_basis: "exact counterfactual displayed roots".into(),
        },
    })
}
fn count_options(
    connection: &Connection,
    d: &crate::query::catalog::Descriptor,
    query: &ValidatedQuery,
    clause_id: Option<&str>,
    context: &QueryFacetContext,
    values: Vec<QueryLiteral>,
    selected: &[QueryLiteral],
) -> Result<Vec<QueryValueOption>, IndexError> {
    let root_base = if clause_id.is_some() && d.definition.scope.is_none() {
        Some(
            root_field_context(query.predicate(), &d.definition.path).ok_or_else(|| {
                IndexError::Invalid("option counts require a validated facet context".into())
            })?,
        )
    } else {
        None
    };
    let mut options = Vec::new();
    for value in values {
        let candidate = candidate_clause(d, &value)?;
        let p = if let Some(base) = &root_base {
            QueryPredicate::new(QueryExpression::AllOf {
                children: vec![base.clone(), candidate],
            })
        } else if let Some(id) = clause_id {
            replace_clause(query.predicate(), id, &candidate)
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
                children: vec![query.predicate().clone(), candidate],
            })
        };
        let accepted = validate_query(&p)?;
        let (eligible, parameters) =
            crate::read_units::eligible_sql(&accepted, context.eligible_keys.as_deref())?;
        let preference = if context.prefer_remaster {
            "WHERE NOT EXISTS(SELECT 1 FROM remaster_pairs p JOIN matches replacement ON replacement.record_id=p.remaster_record_id WHERE p.legacy_record_id=m.record_id)"
        } else {
            ""
        };
        let sql =
            format!("WITH matches AS ({eligible}) SELECT COUNT(*) FROM matches m {preference}");
        let count =
            connection.query_row(&sql, params_from_iter(&parameters), |r| r.get::<_, u64>(0))?;
        options.push(QueryValueOption {
            selected: selected.contains(&value),
            value,
            distinct_roots: count,
        });
    }
    Ok(options)
}
pub(crate) fn field_values(
    connection: &Connection,
    request: &QueryValuesRequest,
) -> Result<QueryFacetOutcome<QueryValueOptions>, IndexError> {
    validate_value_discovery(
        &request.field,
        request.offset,
        request.limit,
        request.text.as_deref(),
    )?;
    let d = field(&request.field)?;

    let (base, target, member) = if let Some(id) = &request.clause_id {
        let (base, target, member) = match remove_clause(&request.query, id, d)? {
            QueryFacetOutcome::Available { value } => value,
            QueryFacetOutcome::Unavailable { reason } => {
                return Ok(QueryFacetOutcome::Unavailable { reason });
            }
        };
        (base, Some(target), member)
    } else {
        (request.query.clone(), None, None)
    };
    let mut values = enumerate_values(connection, d, &base, &request.context, member.as_ref())?;
    // Closed choices absent from this artifact and selected absent open values
    // remain discoverable rather than being silently erased by row enumeration.
    for value in d
        .definition
        .choices
        .iter()
        .filter(|_| !request.only_matching)
        .cloned()
        .map(QueryLiteral::String)
        .chain(
            if d.definition.field_type == QueryFieldType::Boolean && !request.only_matching {
                vec![QueryLiteral::Boolean(false), QueryLiteral::Boolean(true)]
            } else {
                vec![]
            },
        )
        .chain(selected(target.as_ref()))
    {
        if !values.contains(&value) {
            values.push(value);
        }
    }
    super::value_order::sort_values(&d.definition.path, &mut values);
    let selected = selected(target.as_ref());
    if let Some(text) = &request.text {
        values.retain(|value|selected.contains(value) || matches!(value,QueryLiteral::String(s) if s.to_lowercase().contains(&text.to_lowercase())));
    }
    let total_values = values.len() as u64;
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
    let options = count_options(
        connection,
        d,
        &request.query,
        request.clause_id.as_deref(),
        &request.context,
        page,
        &selected,
    )?;
    Ok(QueryFacetOutcome::Available {
        value: QueryValueOptions {
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
        },
    })
}

fn enumerate_values(
    connection: &Connection,
    d: &crate::query::catalog::Descriptor,
    base: &ValidatedQuery,
    context: &QueryFacetContext,
    member: Option<&QueryPredicate>,
) -> Result<Vec<QueryLiteral>, IndexError> {
    let mut compiled = compile_eligibility(base, context)?;
    let (from, _) = occurrences(d)?;
    let b = bound_field(d, d.definition.scope.as_deref());
    let member = member
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
        "WITH known AS (SELECT {values_expr} AS values_json FROM {from} ({}) AND ({member}) AND ({})='value') SELECT DISTINCT v.value FROM known,json_each(known.values_json) v ORDER BY v.value",
        compiled.expression, b.state
    );
    let mut statement = connection.prepare(&sql)?;
    let values = statement
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

    Ok(values)
}

pub(crate) fn field_vocabulary(
    connection: &Connection,
    request: &QueryVocabularyRequest,
) -> Result<QueryVocabulary, IndexError> {
    validate_value_discovery(
        &request.field,
        request.offset,
        request.limit,
        request.text.as_deref(),
    )?;
    let d = field(&request.field)?;
    if request.selected_values.len() > atlas_domain::QueryLimits::default().literal_list {
        return Err(error(
            "discovery_limit",
            "Too many selected vocabulary values",
            Some(&request.field),
        )
        .into());
    }
    // The shared predicate validator remains the literal/choice authority, also
    // for member fields. A vocabulary request has no query to discard.
    let candidate = QueryPredicate::new(QueryExpression::AllOf {
        children: request
            .selected_values
            .iter()
            .map(|value| candidate_clause(d, value))
            .collect::<Result<_, _>>()?,
    });
    let candidate = if let Some(scope) = &d.definition.scope {
        QueryPredicate::new(QueryExpression::Exists {
            collection: scope.clone(),
            scope_id: "vocabulary-selection".into(),
            predicate: Box::new(candidate),
        })
    } else {
        candidate
    };
    if !request.selected_values.is_empty() {
        validate_query(&candidate)?;
    }
    let base = validate_query(&QueryPredicate::boolean(true))?;
    let context = QueryFacetContext {
        eligible_keys: request.eligible_keys.clone(),
        bounded_candidates: false,
        prefer_remaster: false,
    };
    let mut values = enumerate_values(connection, d, &base, &context, None)?;
    if let Some(text) = &request.text {
        let text = text.to_lowercase();
        values.retain(|value| match value {
            QueryLiteral::String(s) => s.to_lowercase().contains(&text),
            QueryLiteral::Boolean(b) => b.to_string().contains(&text),
            QueryLiteral::Number(_) => false,
        });
    }
    super::value_order::sort_values(&d.definition.path, &mut values);
    let total_values = values.len() as u64;
    let options = values
        .into_iter()
        .skip(request.offset)
        .take(request.limit)
        .map(|value| QueryVocabularyOption { value })
        .collect();
    let mut selected_values = Vec::new();
    for value in &request.selected_values {
        if !selected_values.contains(value) {
            selected_values.push(value.clone());
        }
    }
    Ok(QueryVocabulary {
        field: request.field.clone(),
        options,
        selected_values,
        total_values,
        basis: if request.eligible_keys.is_some() {
            QueryVocabularyBasis::ProductKeyScopeBeforeEditionPreference
        } else {
            QueryVocabularyBasis::ProductArtifactBeforeEditionPreference
        },
    })
}
