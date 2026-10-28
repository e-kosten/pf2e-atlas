use super::*;
use atlas_foundry_model::SourceObject;
use atlas_foundry_model::generated::{AlternativeDenialNand, PredicateStatement, StringOrNumber};

pub(super) fn project(value: SourceFieldView<'_, &AlternativeDenialNand>) -> FactView<String> {
    fact(value.and_then(|terms| match group(terms, "all of", 0) {
        Some(value) => SourceFieldView::Value(value),
        None => SourceFieldView::ProjectionInvalid {
            source_path: "/system/skills/special/predicate",
            reason: "Skill qualification cannot be represented completely",
        },
    }))
}

fn group(terms: &[PredicateStatement], label: &str, depth: usize) -> Option<String> {
    let values = terms
        .iter()
        .map(|term| statement(term, depth + 1))
        .collect::<Option<Vec<_>>>()?;
    Some(match values.as_slice() {
        [] if label == "all of" => String::new(),
        [value] if label == "all of" => value.clone(),
        _ => format!("{label} ({})", values.join("; ")),
    })
}

fn statement(term: &PredicateStatement, depth: usize) -> Option<String> {
    if depth > 64 {
        return None;
    }
    macro_rules! group {
        ($value:expr, $field:ident, $label:literal) => {{
            let value = $value;
            clean(&value.additional_fields)?;
            group(value.$field.as_value()?, $label, depth)
        }};
    }
    macro_rules! comparison {
        ($value:expr, $field:ident, $operator:literal) => {{
            let value = $value;
            clean(&value.additional_fields)?;
            let (left, right) = value.$field.as_value()?;
            let right = match right {
                StringOrNumber::String(value) => value.clone(),
                StringOrNumber::Number(value) => value.to_string(),
            };
            Some(format!("{left} {} {right}", $operator))
        }};
    }
    match term {
        PredicateStatement::String(value) => (!value.trim().is_empty()).then(|| value.clone()),
        PredicateStatement::Conjunction(value) => group!(value, and, "all of"),
        PredicateStatement::Disjunction(value) => group!(value, or, "any of"),
        PredicateStatement::AlternativeDenial(value) => group!(value, nand, "not all of"),
        PredicateStatement::JointDenial(value) => group!(value, nor, "none of"),
        PredicateStatement::ExclusiveDisjunction(value) => group!(value, xor, "exactly one of"),
        PredicateStatement::Biconditional(value) => group!(value, iff, "all true or all false"),
        PredicateStatement::Negation(value) => {
            clean(&value.additional_fields)?;
            Some(format!(
                "not ({})",
                statement(value.not.as_value()?, depth + 1)?
            ))
        }
        PredicateStatement::Conditional(value) => {
            clean(&value.additional_fields)?;
            Some(format!(
                "if ({}) then ({})",
                statement(value.r#if.as_value()?, depth + 1)?,
                statement(value.then.as_value()?, depth + 1)?
            ))
        }
        PredicateStatement::EqualTo(value) => comparison!(value, eq, "="),
        PredicateStatement::GreaterThan(value) => comparison!(value, gt, ">"),
        PredicateStatement::GreaterThanEqualTo(value) => comparison!(value, gte, "≥"),
        PredicateStatement::LessThan(value) => comparison!(value, lt, "<"),
        PredicateStatement::LessThanEqualTo(value) => comparison!(value, lte, "≤"),
    }
}

fn clean(fields: &SourceObject) -> Option<()> {
    fields.fields().is_empty().then_some(())
}
