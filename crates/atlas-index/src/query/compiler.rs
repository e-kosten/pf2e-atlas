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
    pub proofs: Vec<(String, String)>,
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
    let owner = if super::catalog::is_embedded_scope(scope) {
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
            Binding::Embedded { .. } => binding(
                &Binding::Family {
                    table: "actor_projection",
                    column: "items",
                    physical_child: false,
                },
                None,
            ),
            Binding::EmbeddedKind { members } => {
                let cases = members
                    .iter()
                    .filter_map(|m| {
                        atlas_record::source_record::source_family_record_kind(m)
                            .map(|k| format!("WHEN '{m}' THEN '{}'", k.as_str()))
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let value = if cases.is_empty() {
                    "NULL".to_owned()
                } else {
                    format!("CASE c.source_type {cases} ELSE NULL END")
                };
                BoundField {
                    state: format!(
                        "CASE WHEN c.source_type_state!='value' THEN c.source_type_state WHEN ({value}) IS NULL THEN 'not_applicable' ELSE 'value' END"
                    ),
                    value,
                    owner_id: "c.id".to_owned(),
                }
            }
            Binding::UnitPriceGp => {
                let mut result = binding(
                    &Binding::Family {
                        table: "physical_projection",
                        column: "price_per_item_cp",
                        physical_child: false,
                    },
                    scope,
                );
                result.value = format!("({} / 100.0)", result.value);
                result
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
            Binding::LevelOrRank => BoundField {
                value: format!(
                    "CASE WHEN r.source_type='spell' THEN {} ELSE r.level END",
                    family_field("spell_projection", "rank", false, None)
                ),
                state: format!(
                    "CASE WHEN r.source_type='spell' THEN {} ELSE r.level_state END",
                    family_field("spell_projection", "rank_state", false, None)
                ),
                owner_id: "r.record_id".to_owned(),
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
    if let Binding::Embedded { members } = &d.binding {
        let types = members
            .iter()
            .map(|s| format!("'{s}'"))
            .collect::<Vec<_>>()
            .join(",");
        return Ok(format!(
            "actor_items {alias} WHERE {alias}.record_id=r.record_id AND ({alias}.source_type_state!='value' OR {alias}.source_type IN ({types}))"
        ));
    }
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
pub(crate) fn parameter(parameters: &mut Vec<Value>, value: Value) -> String {
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
    let tree = compile_tree(query.predicate(), None, &mut parameters)?;
    let mut proofs = Vec::new();
    tree.positive_proofs(true, "1", &mut proofs);
    let expression = tree.truth;
    Ok(CompiledPredicate {
        expression: format!("(r.document_kind!='Macro' AND ({expression}))"),
        parameters,
        proofs,
    })
}
#[derive(Clone, Copy)]
enum BooleanNode {
    All,
    Any,
    Not,
    Leaf,
}
struct CompiledTruth {
    truth: String,
    kind: BooleanNode,
    children: Vec<CompiledTruth>,
    relationship_id: Option<String>,
}
impl CompiledTruth {
    fn positive_proofs(&self, required: bool, guard: &str, out: &mut Vec<(String, String)>) {
        match self.kind {
            BooleanNode::Not => self.children[0].positive_proofs(!required, guard, out),
            BooleanNode::All | BooleanNode::Any => {
                for child in &self.children {
                    let alternative = matches!(self.kind, BooleanNode::All) && !required
                        || matches!(self.kind, BooleanNode::Any) && required;
                    let guard = if alternative {
                        format!(
                            "({guard}) AND ({}) IS {}",
                            child.truth,
                            if required { "TRUE" } else { "FALSE" }
                        )
                    } else {
                        guard.into()
                    };
                    child.positive_proofs(required, &guard, out);
                }
            }
            BooleanNode::Leaf => {
                if required && let Some(id) = &self.relationship_id {
                    out.push((
                        id.clone(),
                        format!("({guard}) AND ({}) IS TRUE", self.truth),
                    ));
                }
            }
        }
    }
}
pub(crate) fn compile_node(
    p: &QueryPredicate,
    scope: Option<&str>,
    parameters: &mut Vec<Value>,
) -> Result<String, crate::IndexError> {
    Ok(compile_tree(p, scope, parameters)?.truth)
}
fn compile_tree(
    p: &QueryPredicate,
    scope: Option<&str>,
    parameters: &mut Vec<Value>,
) -> Result<CompiledTruth, crate::IndexError> {
    let mut children_trees = Vec::new();
    let guarded = |state: String, value: String| {
        format!("CASE WHEN ({state})='value' THEN ({value}) ELSE NULL END")
    };
    let truth = match &p.expression {
        QueryExpression::Relationship { relationship } => {
            crate::relationship_query::compile_relationship(relationship, parameters)?
        }
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
            let child = compile_tree(predicate, Some(collection), parameters)?.truth;
            let child = if matches!(d.binding, Binding::Embedded { .. }) {
                format!("CASE WHEN c.source_type_state='value' THEN ({child}) ELSE NULL END")
            } else {
                child
            };
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
                    .map(|c| {
                        let tree = compile_tree(c, scope, parameters)?;
                        let truth = tree.truth.clone();
                        children_trees.push(tree);
                        Ok::<_, crate::IndexError>(truth)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(op)
            )
        }
        QueryExpression::Not { predicate } => {
            let tree = compile_tree(predicate, scope, parameters)?;
            let truth = format!("NOT ({})", tree.truth);
            children_trees.push(tree);
            truth
        }
    };
    let kind = match &p.expression {
        QueryExpression::AllOf { .. } => BooleanNode::All,
        QueryExpression::AnyOf { .. } => BooleanNode::Any,
        QueryExpression::Not { .. } => BooleanNode::Not,
        _ => BooleanNode::Leaf,
    };
    Ok(CompiledTruth {
        truth,
        kind,
        children: children_trees,
        relationship_id: if matches!(p.expression, QueryExpression::Relationship { .. }) {
            p.clause_id.clone()
        } else {
            None
        },
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

/// Native copper retains the stored per-item numeric domain for ordering and display projection.
pub(crate) fn bound_unit_price_cp() -> Result<BoundField, crate::IndexError> {
    let mut field = bound_field(
        super::catalog::descriptor(None, "physical.unit_price_gp")?.ok_or_else(|| {
            crate::IndexError::Invalid("unit-price binding is unavailable".into())
        })?,
        None,
    );
    field.value = family_field("physical_projection", "price_per_item_cp", false, None);
    Ok(field)
}
