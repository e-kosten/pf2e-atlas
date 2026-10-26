//! Rank provenance describes observed source-backed changes, never overall coverage.
use super::*;
use atlas_foundry_model::{SourcePresence, generated::*};

fn value<T>(fact: &FactView<T>, format: impl FnOnce(&T) -> String) -> String {
    match (fact.state, fact.value.as_ref()) {
        (QueryFieldState::Value, Some(value)) => {
            let text = format(value);
            if text.trim().is_empty() {
                "Empty".into()
            } else {
                text
            }
        }
        (QueryFieldState::Null, _) => "Not specified".into(),
        (QueryFieldState::Missing | QueryFieldState::NotApplicable, _) => "Not listed".into(),
        _ => "Unavailable".into(),
    }
}

fn changed<T: PartialEq>(
    changes: &mut Vec<SpellRankChangeView>,
    base_rank: u8,
    field: SpellRankFactView,
    before: &FactView<T>,
    after: &FactView<T>,
    format: impl FnOnce(&T) -> String,
) {
    if before != after && matches!(after.state, QueryFieldState::Value | QueryFieldState::Null) {
        changes.push(SpellRankChangeView {
            fact: field,
            base_rank,
            base_value: value(before, format),
            explanation: None,
        });
    }
}

fn area(area: &SpellAreaView) -> String {
    [&area.size, &area.shape, &area.details]
        .into_iter()
        .filter(|fact| fact.state == QueryFieldState::Value)
        .filter_map(|fact| fact.value.as_deref())
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn defense(defense: &SpellDefenseView) -> String {
    let mut parts = vec![];
    if defense.basic.value == Some(true) {
        parts.push("Basic".into());
    }
    for fact in [&defense.statistic, &defense.passive] {
        if let Some(value) = &fact.value
            && !value.trim().is_empty()
        {
            parts.push(value.clone());
        }
    }
    parts.join(" ")
}

fn damage(damage: &DamageComponentView) -> String {
    let mut text = value(&damage.formula, Clone::clone);
    if let Some(kind) = &damage.damage_type.value
        && !kind.is_empty()
    {
        text.push(' ');
        text.push_str(kind);
    }
    if let Some(qualifier) = &damage.category.value
        && !qualifier.trim().is_empty()
    {
        text.push_str("; ");
        text.push_str(qualifier);
    }
    if let Some(materials) = &damage.materials.value
        && !materials.is_empty()
    {
        text.push_str("; ");
        text.push_str(&materials.join(", "));
    }
    if let Some(kinds) = &damage.kinds.value
        && !(kinds.len() == 1 && kinds[0] == "damage")
        && !kinds.is_empty()
    {
        text.push_str("; ");
        text.push_str(&kinds.join(", "));
    }
    if damage.apply_modifier.value == Some(true) {
        text.push_str("; applies modifier");
    }
    text
}

/// Compare the exact selected copy with its base, independent of prior UI rank choices.
pub(super) fn rank_changes(
    source: &SpellSource,
    effective: &SpellSource,
    current: &SpellPresentationView,
    rank: u8,
) -> Vec<SpellRankChangeView> {
    let Some(base_rank) = spell_context::base_rank(source).filter(|base| rank > *base) else {
        return vec![];
    };
    let base = spell::authored_spell(source);
    let mut result = vec![];
    macro_rules! compare {
        ($field:ident, $kind:ident, $format:expr) => {
            changed(
                &mut result,
                base_rank,
                SpellRankFactView::$kind,
                &base.$field,
                &current.$field,
                $format,
            );
        };
    }
    compare!(cast, Cast, |v: &ActionCostView| v.label.clone());
    compare!(range, Range, Clone::clone);
    compare!(target, Target, Clone::clone);
    compare!(duration, Duration, Clone::clone);
    compare!(sustained, Sustained, |v: &bool| if *v {
        "Yes"
    } else {
        "No"
    }
    .into());
    compare!(counteraction, Counteraction, |v: &bool| if *v {
        "Yes"
    } else {
        "No"
    }
    .into());
    let traits = |spell: &SpellSource| {
        identifiers(
            SourceFieldView::from(&spell.system)
                .and_then(|s| (&s.traits).into())
                .and_then(|t| (&t.value).into()),
        )
    };
    changed(
        &mut result,
        base_rank,
        SpellRankFactView::Traits,
        &traits(source),
        &traits(effective),
        |v: &Vec<String>| v.join(", "),
    );
    compare!(traditions, Traditions, |v: &Vec<String>| v.join(", "));
    for (kind, before, after) in [
        (SpellRankFactView::Cost, &base.cost, &current.cost),
        (
            SpellRankFactView::Requirements,
            &base.requirements,
            &current.requirements,
        ),
    ] {
        if before != after {
            changed(
                &mut result,
                base_rank,
                kind,
                before.as_ref().unwrap_or(&unavailable()),
                after.as_ref().unwrap_or(&fact(SourceFieldView::Null)),
                Clone::clone,
            );
        }
    }
    changed(
        &mut result,
        base_rank,
        SpellRankFactView::Area,
        base.area.as_ref().unwrap_or(&fact(SourceFieldView::Null)),
        current
            .area
            .as_ref()
            .unwrap_or(&fact(SourceFieldView::Null)),
        area,
    );
    changed(
        &mut result,
        base_rank,
        SpellRankFactView::Defense,
        base.defense
            .as_ref()
            .unwrap_or(&fact(SourceFieldView::Null)),
        current
            .defense
            .as_ref()
            .unwrap_or(&fact(SourceFieldView::Null)),
        defense,
    );
    let rarity = |spell: &SpellSource| {
        identifier(
            SourceFieldView::from(&spell.system)
                .and_then(|s| (&s.traits).into())
                .and_then(|t| (&t.rarity).into()),
        )
    };
    changed(
        &mut result,
        base_rank,
        SpellRankFactView::Rarity,
        &rarity(source),
        &rarity(effective),
        Clone::clone,
    );
    if let Some(components) = &current.damage.value {
        for component in components {
            let before = base
                .damage
                .value
                .as_ref()
                .and_then(|values| values.iter().find(|v| v.id == component.id));
            if before != Some(component) && component.formula.state == QueryFieldState::Value {
                result.push(SpellRankChangeView {
                    fact: SpellRankFactView::Damage {
                        component_id: component.id.clone(),
                    },
                    base_rank,
                    base_value: before.map(damage).unwrap_or_else(|| "Not listed".into()),
                    explanation: None,
                });
            }
        }
    }
    if let Some(SpellSystemSourceHeightening::SpellHeighteningInterval(interval)) = source
        .system
        .as_value()
        .and_then(|s| s.heightening.as_value())
    {
        let step = interval
            .interval
            .as_value()
            .and_then(serde_json::Number::as_u64);
        if let Some(step) = step.filter(|step| *step > 0) {
            let frequency = if step == 1 {
                "per additional rank".into()
            } else {
                format!("per {step} additional ranks")
            };
            for change in &mut result {
                match &change.fact {
                    SpellRankFactView::Damage { component_id } => {
                        let formula = interval
                            .damage
                            .as_value()
                            .and_then(|d| d.entries.iter().find(|(id, _)| id == component_id))
                            .map(|(_, f)| f);
                        if let Some(formula) = formula {
                            change.explanation = Some(format!("+{formula} {frequency}"));
                        }
                    }
                    SpellRankFactView::Area => {
                        if let SourcePresence::Value(increment) = &interval.area {
                            change.explanation = Some(format!("+{increment} feet {frequency}"));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn changes(system: serde_json::Value, rank: u8) -> Vec<SpellRankChangeView> {
        let record = atlas_search::test_support::record(
            "spells",
            "Item",
            json!({
                "_id":"aaaaaaaaaaaaaaaa","type":"spell","system":system
            }),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            record.record.node_at(&[]).unwrap()
        else {
            panic!("spell");
        };
        let original = source.clone();
        let effective = spell_rank::heighten(source, rank).unwrap();
        let result = rank_changes(source, &effective, &spell::authored_spell(&effective), rank);
        assert_eq!(
            source, &original,
            "reading must preserve the selected owned source"
        );
        result
    }

    #[test]
    fn interval_markers_keep_exact_component_and_customized_base() {
        let system = json!({"level":{"value":3},"damage":{
            "custom":{"formula":"8d6","type":"fire","kinds":["damage"]},
            "ongoing":{"formula":"1d4","type":"fire","category":"persistent"}
        },"heightening":{"type":"interval","interval":1,"damage":{"custom":"2d6"}}});
        for rank in [5, 4, 3, 6] {
            let changes = changes(system.clone(), rank);
            if rank == 3 {
                assert!(changes.is_empty());
                continue;
            }
            assert_eq!(changes.len(), 1);
            assert_eq!(
                changes[0].fact,
                SpellRankFactView::Damage {
                    component_id: "custom".into()
                }
            );
            assert_eq!(changes[0].base_rank, 3);
            assert_eq!(changes[0].base_value, "8d6 fire");
            assert_eq!(
                changes[0].explanation.as_deref(),
                Some("+2d6 per additional rank")
            );
        }
    }

    #[test]
    fn fixed_changes_do_not_treat_prose_or_rank_selection_as_applied_mechanics() {
        let system = json!({"level":{"value":2},"target":{"value":"1 willing creature"},
            "description":{"value":"Heightened (4th) additional rules. Heightened (6th) choose a version."},
            "heightening":{"type":"fixed","levels":{"4":{},"6":{"target":{"value":"10 willing creatures"}}}}
        });
        assert!(changes(system.clone(), 4).is_empty());
        let changes = changes(system, 6);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].fact, SpellRankFactView::Target);
        assert_eq!(changes[0].base_value, "1 willing creature");
        assert_eq!(changes[0].explanation, None);
    }

    #[test]
    fn unchanged_values_and_unavailable_replacements_get_no_markers() {
        let system = json!({"level":{"value":1},"target":{"value":"1 creature"},
        "range":{"value":"30 feet"},"heightening":{"type":"fixed","levels":{"2":{
            "target":{"value":"1 creature"},"range":{"value":42}
        }}}});
        assert!(changes(system, 2).is_empty());
    }

    #[test]
    fn casting_traits_and_area_have_their_own_base_explanations() {
        let system = json!({"level":{"value":2},"time":{"value":"2"},
        "area":{"type":"burst","value":10},"traits":{"value":["manipulate"]},
        "heightening":{"type":"fixed","levels":{"5":{
            "time":{"value":"3"},"area":{"type":"burst","value":20},
            "traits":{"value":["concentrate","manipulate"]}
        }}}});
        let changes = changes(system, 5);
        assert_eq!(changes.len(), 3);
        assert_eq!(
            changes
                .iter()
                .find(|c| c.fact == SpellRankFactView::Cast)
                .unwrap()
                .base_value,
            "2 actions"
        );
        assert_eq!(
            changes
                .iter()
                .find(|c| c.fact == SpellRankFactView::Traits)
                .unwrap()
                .base_value,
            "manipulate"
        );
        assert_eq!(
            changes
                .iter()
                .find(|c| c.fact == SpellRankFactView::Area)
                .unwrap()
                .base_value,
            "10 feet burst"
        );
    }
}
