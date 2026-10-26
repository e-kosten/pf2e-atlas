//! Read-only, source-backed statistics for an effect at a selected casting rank.
mod battle_form;
mod choices;
use super::*;
use atlas_foundry_model::generated::*;
use atlas_foundry_model::{SourceContext, SourcePresence};

pub(crate) fn supports(item: ItemSourceView<'_>) -> bool {
    item.rules().value().is_some_and(|rules| {
        let has = |key| {
            rules
                .iter()
                .any(|rule| rule.key.as_value().is_some_and(|v| v == key))
        };
        has("BattleForm") || (has("ChoiceSet") && has("CreatureSize"))
    })
}

pub(crate) fn base_rank(source: &EffectSource) -> Option<u8> {
    source
        .system
        .as_value()
        .and_then(|s| s.level.as_value())
        .and_then(|v| v.value.as_value())
        .and_then(|v| v.as_u64())
        .filter(|v| (1..=10).contains(v))
        .map(|v| v as u8)
}

pub(super) fn apply_rank(
    source: &EffectSource,
    requested: Option<u8>,
    presentation: &mut RecordPresentationView,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> crate::AppServiceResult<Option<u8>> {
    let item = ItemSourceView::EffectSource(source);
    if !supports(item) {
        if requested.is_some() {
            return Err(crate::AppServiceError::invalid_request(
                "This effect has no rank preview.",
            ));
        }
        return Ok(None);
    }
    let base = base_rank(source);
    let Some(base_rank) = base else {
        if requested.is_some() {
            return Err(crate::AppServiceError::invalid_request(
                "The effect's base rank is unavailable.",
            ));
        }
        return Ok(None);
    };
    let rank = requested.unwrap_or(base_rank);
    if !(base_rank..=10).contains(&rank) {
        return Err(crate::AppServiceError::invalid_request(
            "Choose a rank from the effect's base rank through 10.",
        ));
    }
    let mut preview = SpellEffectPreviewView {
        rank,
        base_rank,
        facts: vec![],
        strikes: vec![],
        choices: vec![],
        unavailable: None,
    };
    let rules = item.rules().value().unwrap_or_default();
    let typed = rules
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            matches!(
                r.key.as_value().map(String::as_str),
                Some("BattleForm" | "ChoiceSet" | "CreatureSize" | "FlatModifier")
            )
        })
        .map(|(index, rule)| {
            rule.interpret(SourceContext::new(
                &presentation.identity.record_key,
                "checked source",
                format!("$.system.rules[{index}]"),
            ))
        })
        .collect::<Result<Vec<_>, _>>();
    let projected = typed
        .map_err(|_| "The effect's rank statistics are unavailable.".to_owned())
        .and_then(|rules| {
            let forms = rules
                .iter()
                .filter_map(|r| match r {
                    RuleSource::BattleForm(r) => Some(r.as_ref()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if forms.len() == 1 {
                battle_form::project(forms[0], rank, base_rank, &mut preview, retrieval)
            } else if !forms.is_empty() {
                Err(
                    "This effect has multiple battle forms; read its rules to choose a form."
                        .into(),
                )
            } else {
                choices::project(&rules, rank, &mut preview)
            }
        });
    if let Err(reason) = projected {
        preview.facts.clear();
        preview.strikes.clear();
        preview.choices.clear();
        preview.unavailable = Some(reason);
    }
    if let RecordBodyView::Activity(activity) = &mut presentation.body {
        activity.spell_effect = Some(preview);
        activity.navigation.spell_rank = Some(rank);
        presentation.classification = format!("Effect · Rank {rank}");
        presentation.identity.level_label = Some(format!("Rank {rank}"));
    }
    Ok(Some(rank))
}

pub(super) fn rank_fact(
    label: impl Into<String>,
    value: FactView<String>,
    base: Option<&FactView<String>>,
    qualifier: Option<String>,
) -> SpellEffectFactView {
    let base_value = base
        .filter(|base| {
            base.state == QueryFieldState::Value
                && value.state == QueryFieldState::Value
                && base.value != value.value
        })
        .and_then(|base| base.value.clone());
    SpellEffectFactView {
        label: label.into(),
        value,
        base_value,
        qualifier,
    }
}

pub(super) fn value(text: impl Into<String>) -> FactView<String> {
    FactView {
        state: QueryFieldState::Value,
        value: Some(text.into()),
    }
}

pub(super) fn scalar(source: &SourcePresence<JSONValue>) -> FactView<String> {
    fact(SourceFieldView::from(source).and_then(scalar_text))
}
pub(super) fn scalar_text(v: &JSONValue) -> SourceFieldView<'_, String> {
    match v {
        JSONValue::String(v) => SourceFieldView::Value(v.replace("@actor.level", "your level")),
        JSONValue::Number(v) => SourceFieldView::Value(v.to_string()),
        _ => SourceFieldView::ProjectionInvalid {
            source_path: "system.rules",
            reason: "A scalar statistic is required",
        },
    }
}

pub(super) fn modifier(source: &SourcePresence<StringOrNumber>) -> FactView<String> {
    fact(SourceFieldView::from(source).map(|v| match v {
        StringOrNumber::String(v) => v.replace("@actor.level", "your level"),
        StringOrNumber::Number(v) => {
            if v.as_f64().is_some_and(|v| v >= 0.0) {
                format!("+{v}")
            } else {
                v.to_string()
            }
        }
    }))
}

#[cfg(test)]
mod tests;
