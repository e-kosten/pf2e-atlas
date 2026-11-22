use super::*;
use crate::{pathbuilder_sheet::ImportedSpellRank, presentation::attribute_label};
use atlas_app_model::{SheetSpellRankView, SheetSpellView};
use std::collections::BTreeMap;

pub(super) fn groups(sheet: &PathbuilderSheet, abilities: &AbilityKeys<'_>) -> Vec<SheetGroupView> {
    let mut groups = Vec::new();
    if let Some(points) = value(&sheet.focus_points).filter(|points| **points != 0) {
        groups.push(typed_group(
            SheetGroupKind::FocusPool,
            "Focus pool",
            vec![compact_row("Points", Some(points.to_string()), Vec::new())],
        ));
    }
    for caster in values(&sheet.spell_casters) {
        let mut ranks = BTreeMap::<Option<i64>, SheetSpellRankView>::new();
        for (index, slots) in value(&caster.per_day).into_iter().flatten().enumerate() {
            let index = Some(index as i64);
            if let Some(slots) = value(slots).filter(|slots| **slots != 0) {
                ranks
                    .entry(index)
                    .or_insert_with(|| empty_rank(index))
                    .slots = Some(*slots);
            }
        }
        for (source, prepared) in [(&caster.spells, false), (&caster.prepared, true)] {
            for entry in values(source) {
                let Some(spells) = nonempty_list(entry) else {
                    continue;
                };
                let rank = value(&entry.spell_level).copied();
                let group = ranks.entry(rank).or_insert_with(|| empty_rank(rank));
                let list = if prepared {
                    &mut group.prepared
                } else {
                    &mut group.known
                };
                list.extend(spells);
            }
        }
        let mut ranks = ranks.into_iter().collect::<Vec<_>>();
        ranks.sort_by_key(|(rank, _)| rank.unwrap_or(i64::MAX));
        let has_magic = !ranks.is_empty()
            || value(&caster.innate) == Some(&true)
            || value(&caster.focus_points).is_some_and(|points| *points != 0);
        if !has_magic {
            continue;
        }
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
        let mut caster = summary(name(&caster.name, "Unnamed caster"), facts);
        caster.spell_ranks = ranks
            .into_iter()
            .map(|(_, mut rank)| {
                rank.known = count_spells(rank.known, abilities, PcAbilityKind::Spell);
                rank.prepared = count_spells(rank.prepared, abilities, PcAbilityKind::Spell);
                rank
            })
            .collect();
        groups.push(caster);
    }

    for (tradition, casting) in value(&sheet.focus).into_iter().flatten() {
        for (ability, focus) in value(casting).into_iter().flatten() {
            let Some(focus) = value(focus) else { continue };
            let ranks = [
                ("Focus cantrips", &focus.focus_cantrips),
                ("Focus spells", &focus.focus_spells),
            ]
            .into_iter()
            .filter_map(|(label, field)| {
                let known = count_spells(
                    values(field).map(|spell| occurrence(spell)),
                    abilities,
                    PcAbilityKind::FocusSpell,
                );
                (!known.is_empty()).then(|| SheetSpellRankView {
                    label: label.into(),
                    slots: None,
                    known,
                    prepared: Vec::new(),
                })
            })
            .collect::<Vec<_>>();
            if ranks.is_empty() {
                continue;
            }
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
            let title = format!(
                "{} focus · {}",
                sentence_case(tradition),
                attribute_label(ability)
            );
            let mut group = summary(&title, facts);
            group.spell_ranks = ranks;
            groups.push(group);
        }
    }
    groups
}

fn nonempty_list(entry: &ImportedSpellRank) -> Option<Vec<SheetSpellView>> {
    let spells: Vec<_> = values(&entry.list)
        .filter(|spell| !spell.trim().is_empty())
        .map(|spell| occurrence(spell))
        .collect();
    (!spells.is_empty()).then_some(spells)
}

fn occurrence(spell: &str) -> SheetSpellView {
    SheetSpellView {
        label: spell.to_owned(),
        count: 1,
        ability_key: None,
    }
}

fn count_spells(
    spells: impl IntoIterator<Item = SheetSpellView>,
    abilities: &AbilityKeys<'_>,
    kind: PcAbilityKind,
) -> Vec<SheetSpellView> {
    let mut positions = HashMap::<String, usize>::new();
    let mut unique = Vec::<SheetSpellView>::new();
    for mut spell in spells {
        if let Some(index) = positions.get(&spell.label) {
            unique[*index].count += spell.count;
        } else {
            positions.insert(spell.label.clone(), unique.len());
            spell.ability_key = abilities
                .get(&ability_identity(kind, &spell.label, None))
                .map(|key| (*key).to_owned());
            unique.push(spell);
        }
    }
    unique
}

fn empty_rank(rank: Option<i64>) -> SheetSpellRankView {
    let label = match rank {
        Some(0) => "Cantrips".into(),
        Some(rank) => format!("Rank {rank}"),
        None => "Rank unavailable".into(),
    };
    SheetSpellRankView {
        label,
        slots: None,
        known: Vec::new(),
        prepared: Vec::new(),
    }
}

pub(super) fn comparison_rows(
    ranks: &[SheetSpellRankView],
) -> impl Iterator<Item = SheetRowView> + '_ {
    ranks.iter().flat_map(|rank| {
        let slots = rank
            .slots
            .map(|count| row(&rank.label, vec![fact_value("Slots", count)]));
        slots.into_iter().chain(
            [("Known", &rank.known), ("Prepared", &rank.prepared)]
                .into_iter()
                .flat_map(move |(list, spells)| {
                    spells.iter().flat_map(move |spell| {
                        std::iter::repeat_with(move || {
                            row(
                                &spell.label,
                                vec![fact_value("Rank", &rank.label), fact_value("List", list)],
                            )
                        })
                        .take(spell.count)
                    })
                }),
        )
    })
}
