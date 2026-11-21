use super::*;
use crate::presentation::attribute_label;

const SKILLS: &[(&str, &str, usize)] = &[
    ("acrobatics", "Acrobatics", 1),
    ("arcana", "Arcana", 3),
    ("athletics", "Athletics", 0),
    ("crafting", "Crafting", 3),
    ("deception", "Deception", 5),
    ("diplomacy", "Diplomacy", 5),
    ("intimidation", "Intimidation", 5),
    ("medicine", "Medicine", 4),
    ("nature", "Nature", 4),
    ("occultism", "Occultism", 3),
    ("performance", "Performance", 5),
    ("religion", "Religion", 4),
    ("society", "Society", 3),
    ("stealth", "Stealth", 1),
    ("survival", "Survival", 4),
    ("thievery", "Thievery", 1),
];
const SAVES: &[(&str, &str, usize)] = &[
    ("perception", "Perception", 4),
    ("fortitude", "Fortitude", 2),
    ("reflex", "Reflex", 1),
    ("will", "Will", 4),
];

pub(super) fn groups(sheet: &PathbuilderSheet) -> Vec<SheetGroupView> {
    let mut identity = Vec::new();
    add_facts(
        &mut identity,
        [
            ("Ancestry", shown(&sheet.ancestry)),
            ("Heritage", shown(&sheet.heritage)),
            ("Class", shown(&sheet.class)),
            ("Background", shown(&sheet.background)),
            (
                "Key ability",
                source_label(&sheet.key_ability, attribute_label),
            ),
        ],
    );
    let mut background = Vec::new();
    let languages: Vec<_> = values(&sheet.languages)
        .filter(|language| !language.trim().eq_ignore_ascii_case("None selected"))
        .map(String::as_str)
        .collect();
    add_facts(
        &mut background,
        [
            ("XP", shown(&sheet.xp)),
            ("Languages", Some(languages.join(", "))),
            ("Resistances", text_list(&sheet.resistances)),
            (
                "Size",
                shown(&sheet.size_name).or_else(|| shown(&sheet.size)),
            ),
            ("Age", biography(&sheet.age)),
            ("Gender", biography(&sheet.gender)),
            ("Alignment", biography(&sheet.alignment)),
            ("Deity", biography(&sheet.deity)),
        ],
    );
    let mut scores = Vec::new();
    let mut modifiers = [None; 6];
    if let Some(abilities) = value(&sheet.abilities) {
        for (i, (label, score)) in [
            ("Strength", &abilities.str),
            ("Dexterity", &abilities.dex),
            ("Constitution", &abilities.con),
            ("Intelligence", &abilities.int),
            ("Wisdom", &abilities.wis),
            ("Charisma", &abilities.cha),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(score) = value(score) {
                let mut facts = vec![fact_value("Score", score)];
                let modifier = score.checked_sub(10).map(|value| value.div_euclid(2));
                modifiers[i] = modifier;
                let summary = modifier.map_or_else(
                    || score.to_string(),
                    |modifier| format!("{score} ({modifier:+})"),
                );
                if let Some(modifier) = modifier {
                    facts.push(fact_value("Modifier", format!("{modifier:+}")));
                }
                scores.push(compact_row(label, Some(summary), facts));
            }
        }
    }
    let mut vitals = Vec::new();
    if let Some(ac) = value(&sheet.ac_total) {
        let mut facts = Vec::new();
        add_facts(
            &mut facts,
            [
                ("Total", shown(&ac.ac_total)),
                ("Proficiency", signed(&ac.ac_prof_bonus)),
                ("Ability", signed(&ac.ac_ability_bonus)),
                ("Item", signed(&ac.ac_item_bonus)),
                ("Shield bonus", signed(&ac.shield_bonus)),
            ],
        );
        vitals.push(compact_row("AC", shown(&ac.ac_total), facts));
    }
    if let Some(attributes) = value(&sheet.attributes) {
        let max_hp = || {
            let ancestry = *value(&attributes.ancestry_hp)?;
            let hp_per_level = value(&attributes.class_hp)?
                .checked_add(modifiers[2]?)?
                .checked_add(*value(&attributes.bonus_hp_per_level)?)?;
            hp_per_level
                .checked_mul(sheet.level)?
                .checked_add(ancestry)?
                .checked_add(*value(&attributes.bonus_hp)?)
        };
        let mut hp_facts = vec![fact_value(
            "Max HP",
            max_hp()
                .filter(|hp| *hp > 0)
                .map_or_else(|| "Unavailable".into(), |hp| hp.to_string()),
        )];
        add_facts(
            &mut hp_facts,
            [
                ("Ancestry", shown(&attributes.ancestry_hp)),
                ("Class per level", shown(&attributes.class_hp)),
                (
                    "Constitution per level",
                    modifiers[2].map(|modifier| format!("{modifier:+}")),
                ),
                ("Bonus", signed(&attributes.bonus_hp)),
                ("Bonus per level", signed(&attributes.bonus_hp_per_level)),
            ],
        );
        vitals.push(compact_row(
            "Imported max HP",
            Some(hp_facts[0].value.clone()),
            hp_facts,
        ));
        let total_speed = value(&attributes.speed)
            .zip(value(&attributes.speed_bonus))
            .and_then(|(base, bonus)| base.checked_add(*bonus));
        let mut speed = Vec::new();
        add_facts(
            &mut speed,
            [
                ("Total", total_speed.map(|speed| format!("{speed} ft."))),
                (
                    "Base",
                    value(&attributes.speed).map(|speed| format!("{speed} ft.")),
                ),
                (
                    "Bonus",
                    value(&attributes.speed_bonus).map(|bonus| format!("{bonus:+} ft.")),
                ),
            ],
        );
        if !speed.is_empty() {
            vitals.push(compact_row(
                "Speed",
                total_speed.map(|speed| format!("{speed} ft.")),
                speed,
            ));
        }
    }
    let mut skills = Vec::new();
    let mut saves = Vec::new();
    let mut proficiencies = Vec::new();
    let mut labels = std::collections::HashSet::new();
    let ranks = value(&sheet.proficiencies);
    for (entries, rows) in [(SKILLS, &mut skills), (SAVES, &mut saves)] {
        for &(key, label, ability) in entries {
            let rank_value = ranks
                .and_then(|ranks| ranks.get(key))
                .and_then(value)
                .copied();
            let entry = proficiency_row(sheet, label, rank_value, modifiers[ability]);
            if !entry.facts.is_empty() {
                rows.push(entry);
                labels.insert(label.to_owned());
            }
        }
    }
    for (key, score) in ranks.into_iter().flatten() {
        if SKILLS.iter().chain(SAVES).any(|(known, _, _)| known == key) {
            continue;
        }
        if let Some(score) = value(score) {
            let label = proficiency_label(key);
            proficiencies.push(compact_row(label, Some(rank(*score)), Vec::new()));
        }
    }
    let mut lore = Vec::new();
    for (name, rank) in values(&sheet.lores) {
        let label = if name.trim().is_empty() {
            "Unnamed Lore".into()
        } else {
            format!("{name} Lore")
        };
        labels.insert(label.clone());
        lore.push(proficiency_row(sheet, &label, Some(*rank), modifiers[3]));
    }
    let mut other_bonuses = Vec::new();
    for (label, bonuses) in value(&sheet.mods).into_iter().flatten() {
        if !labels.contains(label) {
            other_bonuses.push(row(label, bonus_facts(value(bonuses))));
        }
    }
    for (label, entries) in value(&sheet.specific_proficiencies).into_iter().flatten() {
        if let Some(entries) = text_list(entries) {
            proficiencies.push(row(
                proficiency_label(label),
                vec![fact_value("Specific proficiencies", entries)],
            ));
        }
    }
    vec![
        summary("Character details", identity),
        group("Defenses & movement", vitals),
        group("Perception & saves", saves),
        group("Ability scores", scores),
        group("Skills", skills),
        group("Lore", lore),
        summary("Background & details", background),
        group("Other proficiencies", proficiencies),
        group("Other exported bonuses", other_bonuses),
    ]
}

fn proficiency_row(
    sheet: &PathbuilderSheet,
    label: &str,
    rank_value: Option<i64>,
    modifier: Option<i64>,
) -> SheetRowView {
    let mut facts = Vec::new();
    // The rank alone does not establish an untrained proficiency bonus.
    // For example, Untrained Improvisation can contribute the character's level.
    let proficiency = rank_value
        .filter(|rank| matches!(rank, 2 | 4 | 6 | 8))
        .and_then(|rank| sheet.level.checked_add(rank));
    let base = modifier
        .zip(proficiency)
        .and_then(|(ability, proficiency)| ability.checked_add(proficiency));
    if let Some(base) = base {
        facts.push(fact_value("Base modifier", format!("{base:+}")));
    }
    if let Some(rank_value) = rank_value {
        facts.push(fact_value("Rank", rank(rank_value)));
    }
    if let Some(proficiency) = proficiency {
        facts.push(fact_value("Proficiency", format!("{proficiency:+}")));
    } else if let Some(modifier) = modifier {
        facts.push(fact_value("Ability modifier", format!("{modifier:+}")));
    }
    let bonuses = value(&sheet.mods)
        .and_then(|mods| mods.get(label))
        .and_then(value);
    facts.extend(bonus_facts(bonuses));
    let summary = base
        .map(|base| format!("{base:+} base"))
        .or_else(|| rank_value.map(rank));
    compact_row(label, summary, facts)
}

fn bonus_facts(
    bonuses: Option<&std::collections::BTreeMap<String, ImportedField<i64>>>,
) -> Vec<SheetFactView> {
    bonuses
        .into_iter()
        .flatten()
        .filter_map(|(label, number)| signed(number).map(|number| fact_value(label, number)))
        .collect()
}

fn biography(field: &ImportedField<String>) -> Option<String> {
    value(field)
        .filter(|value| !value.trim().eq_ignore_ascii_case("Not set"))
        .cloned()
}

fn proficiency_label(key: &str) -> String {
    match key {
        "classDC" => "Class DC",
        "castingArcane" => "Arcane casting",
        "castingDivine" => "Divine casting",
        "castingOccult" => "Occult casting",
        "castingPrimal" => "Primal casting",
        "unarmored" => "Unarmored defense",
        "heavy" => "Heavy armor",
        "medium" => "Medium armor",
        "light" => "Light armor",
        "advanced" => "Advanced weapons",
        "martial" => "Martial weapons",
        "simple" => "Simple weapons",
        "unarmed" => "Unarmed attacks",
        _ => key,
    }
    .to_owned()
}
