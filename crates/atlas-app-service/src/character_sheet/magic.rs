use super::*;
use crate::presentation::attribute_label;

pub(super) fn groups(sheet: &PathbuilderSheet, abilities: &AbilityKeys<'_>) -> Vec<SheetGroupView> {
    let mut groups = Vec::new();
    if let Some(points) = shown(&sheet.focus_points) {
        groups.push(summary("Focus pool", vec![fact_value("Points", points)]));
    }
    for caster in values(&sheet.spell_casters) {
        let caster_label = name(&caster.name, "Unnamed caster");
        let mut facts = Vec::new();
        add_facts(
            &mut facts,
            [
                ("Tradition", title_case(&caster.magic_tradition)),
                ("Casting type", title_case(&caster.spellcasting_type)),
                (
                    "Key ability",
                    source_label(&caster.ability, attribute_label),
                ),
                (
                    "Proficiency",
                    value(&caster.proficiency).map(|value| rank(*value)),
                ),
                ("Focus points", shown(&caster.focus_points)),
                ("Innate", yes_no(&caster.innate)),
            ],
        );
        let mut rows = Vec::new();
        if !facts.is_empty() {
            rows.push(row("Spellcasting", facts));
        }
        let mut ranks = std::collections::BTreeMap::<i64, Vec<SheetRowView>>::new();
        for (index, slots) in value(&caster.per_day).into_iter().flatten().enumerate() {
            if let Some(slots) = value(slots) {
                ranks.entry(index as i64).or_default().push(row(
                    rank_label(index as i64),
                    vec![fact_value("Per day", slots)],
                ));
            }
        }
        let mut unranked = Vec::new();
        for (source, label) in [(&caster.spells, "Known"), (&caster.prepared, "Prepared")] {
            for spell_rank in values(source) {
                let spell_level = value(&spell_rank.spell_level).copied();
                let spell_rows = spell_level
                    .map(|level| ranks.entry(level).or_default())
                    .unwrap_or(&mut unranked);
                for spell in values(&spell_rank.list) {
                    let mut facts = vec![fact_value("List", label)];
                    if let Some(level) = spell_level {
                        facts.push(fact_value("Rank", rank_label(level)));
                    }
                    spell_rows.push(ability_row(
                        abilities,
                        PcAbilityKind::Spell,
                        spell,
                        None,
                        facts,
                    ));
                }
            }
        }
        rows.extend(ranks.into_values().flatten());
        rows.extend(unranked);
        groups.push(group(caster_label, rows));
    }
    for (tradition, casting) in value(&sheet.focus).into_iter().flatten() {
        for (ability, focus) in value(casting).into_iter().flatten() {
            if let Some(focus) = value(focus) {
                let mut facts = Vec::new();
                add_facts(
                    &mut facts,
                    [
                        ("Ability bonus", signed(&focus.ability_bonus)),
                        (
                            "Proficiency",
                            value(&focus.proficiency).map(|value| rank(*value)),
                        ),
                        ("Item bonus", signed(&focus.item_bonus)),
                    ],
                );
                let mut rows = Vec::new();
                if !facts.is_empty() {
                    rows.push(row(attribute_label(ability), facts));
                }
                for (source, label) in [
                    (&focus.focus_cantrips, "Focus cantrip"),
                    (&focus.focus_spells, "Focus spell"),
                ] {
                    for spell in values(source) {
                        rows.push(ability_row(
                            abilities,
                            PcAbilityKind::FocusSpell,
                            spell,
                            None,
                            vec![fact_value("Type", label)],
                        ));
                    }
                }
                let title = format!(
                    "{} focus · {}",
                    sentence_case(tradition),
                    attribute_label(ability)
                );
                groups.push(group(title, rows));
            }
        }
    }
    groups
}

fn rank_label(level: i64) -> String {
    if level == 0 {
        "Cantrips".into()
    } else {
        format!("Rank {level}")
    }
}
