//! Accepted-root lowering exposes relational indexes without changing nullable truth.
use super::{
    ValidatedQuery,
    catalog::{Binding, Descriptor},
    compiler::{
        checked_descriptor, comparison_operator, compile_node, family_applicability, parameter,
        scalar,
    },
};
use atlas_domain::{QueryExpression, QueryPredicate, QuerySetMatch};
use rusqlite::types::Value;

pub(crate) fn compile_root_membership(
    query: &ValidatedQuery,
) -> Result<(String, Vec<Value>), crate::IndexError> {
    let mut parameters = Vec::new();
    let expression = accepted(query.predicate(), &mut parameters)?;
    Ok((expression, parameters))
}

fn accepted(
    predicate: &QueryPredicate,
    parameters: &mut Vec<Value>,
) -> Result<String, crate::IndexError> {
    let optimized = match &predicate.expression {
        QueryExpression::Compare { field, op, value } => {
            let descriptor = checked_descriptor(None, field)?;
            scalar_membership(descriptor, parameters, |value_sql, parameters| {
                Ok(format!(
                    "{value_sql} {} {}",
                    comparison_operator(*op),
                    parameter(parameters, scalar(value)?)
                ))
            })?
        }
        QueryExpression::In { field, values } => {
            let descriptor = checked_descriptor(None, field)?;
            scalar_membership(descriptor, parameters, |value_sql, parameters| {
                let slots = values
                    .iter()
                    .map(|value| Ok(parameter(parameters, scalar(value)?)))
                    .collect::<Result<Vec<_>, crate::IndexError>>()?;
                Ok(format!("{value_sql} IN ({})", slots.join(",")))
            })?
        }
        QueryExpression::SetMatch { field, op, values } if *op != QuerySetMatch::ExcludesAny => {
            let descriptor = checked_descriptor(None, field)?;
            set_membership(descriptor, *op, values, parameters)?
        }
        QueryExpression::AllOf { children } => Some(format!(
            "({})",
            children
                .iter()
                .map(|child| accepted(child, parameters))
                .collect::<Result<Vec<_>, _>>()?
                .join(" AND ")
        )),
        _ => None,
    };
    match optimized {
        Some(expression) => Ok(expression),
        None => Ok(format!(
            "({}) IS TRUE",
            compile_node(predicate, None, parameters)?
        )),
    }
}

fn applicable(descriptor: &Descriptor, expression: String) -> String {
    match family_applicability(descriptor, None) {
        Some(guard) => format!("({guard}) AND ({expression})"),
        None => expression,
    }
}

fn scalar_membership(
    descriptor: &Descriptor,
    parameters: &mut Vec<Value>,
    comparison: impl FnOnce(&str, &mut Vec<Value>) -> Result<String, crate::IndexError>,
) -> Result<Option<String>, crate::IndexError> {
    if let Binding::Common { column, known } = &descriptor.binding {
        if *column == "pack_label" {
            return Ok(None);
        }
        let comparison = comparison(&format!("r.{column}"), parameters)?;
        let expression = if *known {
            comparison
        } else {
            format!("r.{column}_state='value' AND ({comparison})")
        };
        return Ok(Some(applicable(descriptor, expression)));
    }
    let Some(relation) = RootRelation::for_binding(&descriptor.binding) else {
        return Ok(None);
    };
    let comparison = comparison(&relation.value, parameters)?;
    Ok(Some(applicable(
        descriptor,
        relation.membership(&comparison),
    )))
}

struct RootRelation {
    from: String,
    record: String,
    owner: String,
    value: String,
    state: String,
}
impl RootRelation {
    fn for_binding(binding: &Binding) -> Option<Self> {
        match binding {
            Binding::Common { column, known } if *column != "pack_label" => Some(Self {
                from: "records p".into(),
                record: "p.record_id".into(),
                owner: "p.record_id".into(),
                value: format!("p.{column}"),
                state: if *known {
                    "'value'".into()
                } else {
                    format!("p.{column}_state")
                },
            }),
            Binding::Family {
                table,
                column,
                physical_child,
            } => {
                let actor = *table == "actor_projection";
                let alias = if *physical_child { "f" } else { "p" };
                Some(Self {
                    from: if *physical_child {
                        format!("{table} f JOIN physical_projection p ON p.id=f.physical_id")
                    } else {
                        format!("{table} p")
                    },
                    record: if actor {
                        "p.record_id".into()
                    } else {
                        "p.root_record_id".into()
                    },
                    owner: if actor {
                        "p.record_id".into()
                    } else {
                        "p.id".into()
                    },
                    value: format!("{alias}.{column}"),
                    state: format!("{alias}.{column}_state"),
                })
            }
            Binding::UnitPriceGp => {
                let mut relation = Self::for_binding(&Binding::Family {
                    table: "physical_projection",
                    column: "price_per_item_cp",
                    physical_child: false,
                })?;
                relation.value = format!("({} / 100.0)", relation.value);
                Some(relation)
            }
            _ => None,
        }
    }
    fn membership(&self, comparison: &str) -> String {
        format!(
            "r.record_id IN (SELECT {} FROM {} WHERE {}='value' AND ({comparison}))",
            self.record, self.from, self.state
        )
    }
}

fn set_membership(
    descriptor: &Descriptor,
    op: QuerySetMatch,
    values: &[String],
    parameters: &mut Vec<Value>,
) -> Result<Option<String>, crate::IndexError> {
    let Binding::Set {
        parent,
        table,
        owner,
        kind,
    } = &descriptor.binding
    else {
        return Ok(None);
    };
    let Some(relation) = RootRelation::for_binding(parent) else {
        return Ok(None);
    };
    let kind = kind
        .map(|kind| format!(" AND t.kind='{kind}'"))
        .unwrap_or_default();
    let membership = |comparison: &str| {
        applicable(
            descriptor,
            format!(
                "r.record_id IN (SELECT {} FROM {} JOIN {table} t ON t.{owner}={} WHERE {}='value'{kind} AND ({comparison}))",
                relation.record, relation.from, relation.owner, relation.state
            ),
        )
    };
    if op == QuerySetMatch::IncludesAll {
        let checks = values
            .iter()
            .map(|value| {
                let slot = parameter(parameters, Value::Text(value.clone()));
                membership(&format!("t.value={slot}"))
            })
            .collect::<Vec<_>>();
        Ok(Some(format!("({})", checks.join(" AND "))))
    } else {
        let slots = values
            .iter()
            .map(|value| parameter(parameters, Value::Text(value.clone())))
            .collect::<Vec<_>>();
        Ok(Some(membership(&format!(
            "t.value IN ({})",
            slots.join(",")
        ))))
    }
}
