//! Explicit choice values bound to size/modifier rules, not guessed from key names.
use super::*;
use atlas_foundry_model::{PredicateInput, PredicateStatement, SourceValue};

type Choice = SourceFromSchemaChoiceSetSchemaChoicesAlternative1EntryPickableThing;
fn compare(
    input: &SourcePresence<(String, StringOrNumber)>,
    rank: u8,
    compare: impl FnOnce(f64, f64) -> bool,
) -> Option<bool> {
    let (name, StringOrNumber::Number(bound)) = input.as_value()? else {
        return None;
    };
    (name == "parent:level").then_some(())?;
    Some(compare(f64::from(rank), bound.as_f64()?))
}
fn statement(predicate: &PredicateStatement, rank: u8) -> Option<bool> {
    match predicate {
        PredicateStatement::GreaterThanEqualTo(p) => compare(&p.gte, rank, |a, b| a >= b),
        PredicateStatement::LessThanEqualTo(p) => compare(&p.lte, rank, |a, b| a <= b),
        PredicateStatement::GreaterThan(p) => compare(&p.gt, rank, |a, b| a > b),
        PredicateStatement::LessThan(p) => compare(&p.lt, rank, |a, b| a < b),
        PredicateStatement::EqualTo(p) => compare(&p.eq, rank, |a, b| a == b),
        PredicateStatement::Disjunction(p) => {
            p.or.as_value()?
                .iter()
                .map(|p| statement(p, rank))
                .collect::<Option<Vec<_>>>()
                .map(|v| v.into_iter().any(|v| v))
        }
        PredicateStatement::Conjunction(p) => p
            .and
            .as_value()?
            .iter()
            .map(|p| statement(p, rank))
            .collect::<Option<Vec<_>>>()
            .map(|v| v.into_iter().all(|v| v)),
        PredicateStatement::Negation(p) => statement(p.not.as_value()?, rank).map(|v| !v),
        _ => None,
    }
}
fn predicate(input: &SourcePresence<PredicateInput>, rank: u8) -> Option<bool> {
    match input {
        SourcePresence::Missing => Some(true),
        SourcePresence::Value(PredicateInput::Array(values)) => values
            .iter()
            .map(|p| statement(p, rank))
            .collect::<Option<Vec<_>>>()
            .map(|v| v.into_iter().all(|v| v)),
        SourcePresence::Value(PredicateInput::PredicateStatement(p)) => statement(p, rank),
        _ => None,
    }
}
fn bound<'a>(
    input: &'a SourcePresence<JSONValue>,
    flag: &str,
    choice: &'a Choice,
) -> Result<Option<&'a SourceValue>, String> {
    let Some(JSONValue::String(reference)) = input.as_value() else {
        return Ok(None);
    };
    let prefix = format!("{{item|flags.system.rulesSelections.{flag}.");
    let Some(field) = reference
        .strip_prefix(&prefix)
        .and_then(|s| s.strip_suffix('}'))
    else {
        return Ok(None);
    };
    if field.is_empty() || field.contains('.') {
        return Err("The selected form needs additional context.".into());
    }
    let Some(StringOrNumberOrObject::Object(SourceValue::Object(value))) = choice.value.as_value()
    else {
        return Err("The choice's values are unavailable.".into());
    };
    let mut members = value.fields().iter().filter(|(key, _)| key == field);
    let found = members.next().map(|(_, v)| v);
    if members.next().is_some() {
        return Err("The choice's values are ambiguous.".into());
    }
    found
        .map(Some)
        .ok_or_else(|| "A choice statistic is unavailable.".into())
}
fn scalar_value(value: &SourceValue) -> Result<String, String> {
    match value {
        SourceValue::String(v) => Ok(v.clone()),
        SourceValue::Number(v) => Ok(v.to_string()),
        _ => Err("A choice statistic is unavailable.".into()),
    }
}
fn size(value: &SourceValue) -> Result<String, String> {
    let SourceValue::String(size) = value else {
        return Err("The choice's size is unavailable.".into());
    };
    Ok(match size.as_str() {
        "lg" | "large" => "Large",
        "huge" => "Huge",
        "med" | "medium" => "Medium",
        "sm" | "small" => "Small",
        "tiny" => "Tiny",
        "grg" | "gargantuan" => "Gargantuan",
        _ => return Err("The choice's size is unavailable.".into()),
    }
    .into())
}
fn facts(
    rules: &[RuleSource],
    flag: &str,
    choice: &Choice,
) -> Result<Vec<SpellEffectFactView>, String> {
    let mut output = vec![];
    for rule in rules {
        match rule {
            RuleSource::CreatureSize(rule) => {
                if rule.ignored.as_value() == Some(&true) {
                    continue;
                }
                if !matches!(&rule.predicate, SourcePresence::Missing)
                    && !rule.predicate.as_value().is_some_and(Vec::is_empty)
                {
                    return Err(
                        "The size depends on additional conditions; read the effect's rules."
                            .into(),
                    );
                }
                if let Some(v) = bound(&rule.value, flag, choice)? {
                    output.push(rank_fact("Size", value(size(v)?), None, None));
                }
                if let Some(reach) = rule.reach.as_value() {
                    for (label, input) in [
                        ("Reach (fixed)", &reach.r#override),
                        ("Reach increase", &reach.add),
                        ("Minimum reach", &reach.upgrade),
                    ] {
                        if let Some(v) = bound(input, flag, choice)? {
                            output.push(rank_fact(
                                label,
                                value(format!("{} feet", scalar_value(v)?)),
                                None,
                                None,
                            ));
                        }
                    }
                }
            }
            RuleSource::FlatModifier(rule) => {
                if rule.ignored.as_value() == Some(&true) {
                    continue;
                }
                if !matches!(&rule.predicate, SourcePresence::Missing)
                    && !rule.predicate.as_value().is_some_and(Vec::is_empty)
                {
                    return Err(
                        "A modifier depends on additional conditions; read the effect's rules."
                            .into(),
                    );
                }
                if let Some(v) = bound(&rule.value, flag, choice)? {
                    let selector = match rule.selector.as_value() {
                        Some(SourceFromSchemaAdjustModifierSchemaSelectors::String(v)) => v.clone(),
                        Some(SourceFromSchemaAdjustModifierSchemaSelectors::Array(v)) => {
                            v.join(", ")
                        }
                        _ => return Err("The modifier's affected statistic is unavailable.".into()),
                    };
                    let label = match selector.as_str() {
                        "melee-strike-damage" => "Melee damage".into(),
                        _ => selector.replace('-', " "),
                    };
                    let v = scalar_value(v)?;
                    let kind = rule
                        .r#type
                        .as_value()
                        .and_then(token)
                        .ok_or("The modifier type is unavailable.")?;
                    output.push(rank_fact(
                        label,
                        value(format!(
                            "{}{v} {kind}",
                            if v.starts_with('-') { "" } else { "+" }
                        )),
                        None,
                        None,
                    ));
                }
            }
            _ => {}
        }
    }
    if output.is_empty() {
        return Err("These choices need additional context; read the effect's rules.".into());
    }
    Ok(output)
}
fn eligible(choices: &[Choice], rank: u8) -> Result<Vec<&Choice>, String> {
    let mut out = vec![];
    for choice in choices {
        match predicate(&choice.predicate, rank) {
            Some(true) => out.push(choice),
            Some(false) => {}
            None => {
                return Err(
                    "Available choices depend on additional conditions; read the effect's rules."
                        .into(),
                );
            }
        }
    }
    Ok(out)
}
pub(super) fn project(
    rules: &[RuleSource],
    rank: u8,
    preview: &mut SpellEffectPreviewView,
) -> Result<(), String> {
    let choices = rules
        .iter()
        .filter_map(|r| match r {
            RuleSource::ChoiceSet(r) => Some(r.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [rule] = choices.as_slice() else {
        return Err("The effect's rank statistics are unavailable.".into());
    };
    if rule.ignored.as_value() == Some(&true)
        || (!matches!(rule.predicate, SourcePresence::Missing)
            && !rule.predicate.as_value().is_some_and(Vec::is_empty))
    {
        return Err(
            "These choices depend on additional conditions; read the effect's rules.".into(),
        );
    }
    let Some(SourceFromSchemaChoiceSetSchemaChoices::Array(choices)) = rule.choices.as_value()
    else {
        return Err("The effect's choices are unavailable.".into());
    };
    let flag = rule
        .flag
        .as_value()
        .ok_or("The effect's choice is unavailable.")?;
    let baseline = eligible(choices, preview.base_rank)?;
    let base_facts = if let [choice] = baseline.as_slice() {
        facts(rules, flag, choice)?
    } else {
        vec![]
    };
    for choice in eligible(choices, rank)? {
        let mut current = facts(rules, flag, choice)?;
        for fact in &mut current {
            fact.base_value = base_facts
                .iter()
                .find(|b| b.label == fact.label && b.value != fact.value)
                .and_then(|b| b.value.value.clone());
        }
        let label = current
            .iter()
            .find(|f| f.label == "Size")
            .and_then(|f| f.value.value.clone())
            .ok_or("The size choice is unavailable.")?;
        preview.choices.push(SpellEffectChoiceView {
            label,
            facts: current,
        });
    }
    if preview.choices.len() == 1 {
        preview.facts = preview.choices.remove(0).facts;
    }
    if preview.choices.is_empty() && preview.facts.is_empty() {
        return Err("No size choice is available at this rank.".into());
    }
    Ok(())
}
