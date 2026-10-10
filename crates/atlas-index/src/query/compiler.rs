//! SQL identifiers are selected exclusively from the compiled catalog.
use super::{
    catalog::{Binding, Descriptor, descriptor},
    validation::{ValidatedQuery, numeric_literal},
};
use atlas_domain::{QueryCompare, QueryExpression, QueryLiteral, QueryPredicate, QuerySetMatch};
use rusqlite::types::Value;

#[derive(Debug, Clone)]
pub(crate) struct CompiledPredicate {
    pub expression: String,
    pub parameters: Vec<Value>,
}
#[derive(Debug, Clone)]
pub(crate) struct BoundField {
    pub value: String,
    pub state: String,
    pub owner_id: String,
}

fn source_alias(scope: Option<&str>) -> &'static str {
    if scope.is_some() { "c" } else { "r" }
}
fn family_field(table: &str, column: &str, physical_child: bool, scope: Option<&str>) -> String {
    if table == "actor_projection" {
        return format!(
            "(SELECT p.{column} FROM actor_projection p WHERE p.record_id=r.record_id)"
        );
    }
    let owner = if scope == Some("actor.items") {
        "actor_item_id=c.id"
    } else {
        "root_record_id=r.record_id"
    };
    if physical_child {
        format!(
            "(SELECT f.{column} FROM {table} f JOIN physical_projection p ON p.id=f.physical_id WHERE p.{owner})"
        )
    } else {
        format!("(SELECT p.{column} FROM {table} p WHERE p.{owner})")
    }
}
pub(crate) fn bound_field(d: &Descriptor, scope: Option<&str>) -> BoundField {
    fn binding(b: &Binding, scope: Option<&str>) -> BoundField {
        let a = source_alias(scope);
        match b {
            Binding::Common { column, known } => {
                let value = if *column == "pack_label" {
                    "(SELECT p.label FROM packs p WHERE p.pack_id=r.pack_id)".to_owned()
                } else {
                    format!("{a}.{column}")
                };
                BoundField {
                    value,
                    state: if *known {
                        "'value'".to_owned()
                    } else {
                        format!("{a}.{column}_state")
                    },
                    owner_id: if scope.is_some() {
                        "c.id"
                    } else {
                        "r.record_id"
                    }
                    .to_owned(),
                }
            }
            Binding::Family {
                table,
                column,
                physical_child,
            } => BoundField {
                value: family_field(table, column, *physical_child, scope),
                state: family_field(table, &format!("{column}_state"), *physical_child, scope),
                owner_id: if *table == "actor_projection" {
                    "r.record_id".to_owned()
                } else {
                    family_field(
                        if *physical_child {
                            "physical_projection"
                        } else {
                            table
                        },
                        "id",
                        false,
                        scope,
                    )
                },
            },
            Binding::Set { parent, .. } | Binding::Collection { parent, .. } => {
                binding(parent, scope)
            }
            Binding::Member { column } => BoundField {
                value: format!("c.{column}"),
                state: format!("c.{column}_state"),
                owner_id: "c.id".to_owned(),
            },
            Binding::Cantrip => BoundField {
                value: format!(
                    "EXISTS(SELECT 1 FROM {} t WHERE t.{}={} AND t.value='cantrip')",
                    if scope.is_some() {
                        "actor_item_traits"
                    } else {
                        "record_traits"
                    },
                    if scope.is_some() {
                        "item_id"
                    } else {
                        "record_id"
                    },
                    if scope.is_some() {
                        "c.id"
                    } else {
                        "r.record_id"
                    }
                ),
                state: format!("{a}.traits_state"),
                owner_id: if scope.is_some() {
                    "c.id"
                } else {
                    "r.record_id"
                }
                .to_owned(),
            },
            Binding::Duration { unit } => {
                let unit_state =
                    family_field("effect_projection", "duration_unit_state", false, scope);
                let unit_value = family_field("effect_projection", "duration_unit", false, scope);
                BoundField {
                    value: family_field("effect_projection", "duration_value", false, scope),
                    state: format!(
                        "CASE WHEN {unit_state}!='value' THEN {unit_state} WHEN {unit_value}='{unit}' THEN {} ELSE 'not_applicable' END",
                        family_field("effect_projection", "duration_value_state", false, scope)
                    ),
                    owner_id: family_field("effect_projection", "id", false, scope),
                }
            }
        }
    }
    let mut result = binding(&d.binding, scope);
    if !d.definition.family_types.is_empty() {
        let guard = d
            .definition
            .family_types
            .iter()
            .map(|s| format!("'{s}'"))
            .collect::<Vec<_>>()
            .join(",");
        result.state = format!(
            "CASE WHEN {}.source_type IN ({guard}) THEN {} ELSE 'not_applicable' END",
            source_alias(scope),
            result.state
        );
    }
    result
}
pub(crate) fn member_relation(
    d: &Descriptor,
    scope: Option<&str>,
    alias: &str,
) -> Result<String, crate::IndexError> {
    let (parent, table, owner, kind) = match &d.binding {
        Binding::Set {
            parent,
            table,
            owner,
            kind,
        }
        | Binding::Collection {
            parent,
            table,
            owner,
            kind,
        } => (parent, table, owner, kind),
        _ => {
            return Err(crate::IndexError::Invalid(
                "compiler collection binding is inconsistent with validated catalog".into(),
            ));
        }
    };
    let parent_descriptor = Descriptor {
        definition: d.definition.clone(),
        binding: (**parent).clone(),
    };
    let parent_id = bound_field(&parent_descriptor, scope).owner_id;
    Ok(format!(
        "{table} {alias} WHERE {alias}.{owner}={parent_id}{}",
        kind.map(|k| format!(" AND {alias}.kind='{k}'"))
            .unwrap_or_default()
    ))
}
fn parameter(parameters: &mut Vec<Value>, value: Value) -> String {
    parameters.push(value);
    format!("?{}", parameters.len())
}
fn scalar(value: &QueryLiteral) -> Result<Value, crate::IndexError> {
    Ok(match value {
        QueryLiteral::String(v) => Value::Text(v.clone()),
        QueryLiteral::Boolean(v) => Value::Integer(i64::from(*v)),
        QueryLiteral::Number(v) => numeric_literal(v)?,
    })
}
pub(crate) fn compile_predicate(
    query: &ValidatedQuery,
) -> Result<CompiledPredicate, crate::IndexError> {
    let mut parameters = Vec::new();
    let expression = compile_node(query.predicate(), None, &mut parameters)?;
    Ok(CompiledPredicate {
        expression: format!("(r.document_kind!='Macro' AND ({expression}))"),
        parameters,
    })
}
pub(crate) fn compile_node(
    p: &QueryPredicate,
    scope: Option<&str>,
    parameters: &mut Vec<Value>,
) -> Result<String, crate::IndexError> {
    let guarded = |state: String, value: String| {
        format!("CASE WHEN ({state})='value' THEN ({value}) ELSE NULL END")
    };
    Ok(match &p.expression {
        QueryExpression::BooleanConstant { value } => if *value { "1" } else { "0" }.to_owned(),
        QueryExpression::Compare { field, op, value } => {
            let d = checked_descriptor(scope, field)?;
            let b = bound_field(d, scope);
            let op = match op {
                QueryCompare::Eq => "=",
                QueryCompare::Neq => "!=",
                QueryCompare::Lt => "<",
                QueryCompare::Lte => "<=",
                QueryCompare::Gt => ">",
                QueryCompare::Gte => ">=",
            };
            guarded(
                b.state,
                format!("{} {op} {}", b.value, parameter(parameters, scalar(value)?)),
            )
        }
        QueryExpression::In { field, values } => {
            let b = bound_field(checked_descriptor(scope, field)?, scope);
            let values = values
                .iter()
                .map(|v| Ok(parameter(parameters, scalar(v)?)))
                .collect::<Result<Vec<_>, crate::IndexError>>()?
                .join(",");
            guarded(b.state, format!("{} IN ({values})", b.value))
        }
        QueryExpression::StateMatch { field, state } => {
            let b = bound_field(checked_descriptor(scope, field)?, scope);
            format!(
                "({})={}",
                b.state,
                parameter(parameters, Value::Text(state.as_str().to_owned()))
            )
        }
        QueryExpression::SetMatch { field, op, values } => {
            let d = checked_descriptor(scope, field)?;
            let b = bound_field(d, scope);
            let checks = values
                .iter()
                .map(|v| {
                    Ok(format!(
                        "EXISTS(SELECT 1 FROM {} AND t.value={})",
                        member_relation(d, scope, "t")?,
                        parameter(parameters, Value::Text(v.clone()))
                    ))
                })
                .collect::<Result<Vec<_>, crate::IndexError>>()?;
            let check = checks.join(if *op == QuerySetMatch::IncludesAll {
                " AND "
            } else {
                " OR "
            });
            guarded(
                b.state,
                if *op == QuerySetMatch::ExcludesAny {
                    format!("NOT ({check})")
                } else {
                    check
                },
            )
        }
        QueryExpression::Exists {
            collection,
            predicate,
            ..
        } => {
            let d = checked_descriptor(None, collection)?;
            let state = bound_field(d, None).state;
            let relation = member_relation(d, None, "c")?;
            let child = compile_node(predicate, Some(collection), parameters)?;
            // The true witness wins regardless of child order or other unknown
            // witnesses, matching CEL specification rather than cel's interpreter.
            format!(
                "CASE WHEN ({state}) IS NOT 'value' THEN NULL WHEN EXISTS(SELECT 1 FROM {relation} AND ({child}) IS TRUE) THEN 1 WHEN EXISTS(SELECT 1 FROM {relation} AND ({child}) IS NULL) THEN NULL ELSE 0 END"
            )
        }
        QueryExpression::AllOf { children } | QueryExpression::AnyOf { children } => {
            let op = if matches!(p.expression, QueryExpression::AllOf { .. }) {
                " AND "
            } else {
                " OR "
            };
            format!(
                "({})",
                children
                    .iter()
                    .map(|c| compile_node(c, scope, parameters))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(op)
            )
        }
        QueryExpression::Not { predicate } => {
            format!("NOT ({})", compile_node(predicate, scope, parameters)?)
        }
    })
}
fn checked_descriptor(
    scope: Option<&str>,
    field: &str,
) -> Result<&'static Descriptor, crate::IndexError> {
    descriptor(scope, field)?.ok_or_else(|| {
        crate::IndexError::Invalid(format!(
            "validated field {field} is absent from compiled catalog"
        ))
    })
}
