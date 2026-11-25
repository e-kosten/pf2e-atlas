use super::*;
use crate::{
    definitions::{HelpContext, record_navigation},
    presentation::attribute_label,
};
use atlas_app_model::{ReferenceDestinationView, ReferenceHelpView};
use atlas_domain::{DefinitionKey, DefinitionNamespace};

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

fn identity(
    sheet: &PathbuilderSheet,
    retrieval: Option<&AtlasRetrievalService>,
) -> AppServiceResult<SheetGroupView> {
    let mut rows = Vec::new();
    for (label, field, source_type) in [
        ("Ancestry", &sheet.ancestry, "ancestry"),
        ("Heritage", &sheet.heritage, "heritage"),
        ("Class", &sheet.class, "class"),
        ("Background", &sheet.background, "background"),
    ] {
        let Some(name) = value(field)
            .map(|name| name.trim())
            .filter(|name| !name.is_empty())
        else {
            continue;
        };
        let mut row = compact_row(label, Some(name.to_owned()), Vec::new());
        if let Some(retrieval) = retrieval {
            let key = unique_pf2e_record(retrieval, name, |record| {
                record
                    .source_type
                    .as_value()
                    .is_some_and(|kind| kind == source_type)
            })?;
            row.summary_help = key.map(|key| ReferenceHelpView {
                definition_key: None,
                destination: ReferenceDestinationView::Record {
                    navigation: record_navigation(key, vec![], retrieval.source_fingerprint())
                        .into(),
                },
            });
        }
        rows.push(row);
    }
    Ok(group("Character details", rows))
}

fn key_ability(
    sheet: &PathbuilderSheet,
    retrieval: Option<&AtlasRetrievalService>,
) -> AppServiceResult<Option<SheetRowView>> {
    let Some(ability) = value(&sheet.key_ability)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let mut row = compact_row(
        "Key ability",
        Some(attribute_label(ability).to_owned()),
        Vec::new(),
    );
    if let Some(retrieval) = retrieval
        && let Ok(key) = DefinitionKey::new(DefinitionNamespace::Ability, ability.to_owned())
    {
        row.summary_help = HelpContext::load(retrieval, [key.clone()].into())?.help(&key);
    }
    Ok(Some(row))
}

#[derive(Clone)]
struct Identity {
    details: SheetGroupView,
    key_ability: Option<SheetRowView>,
    languages: Option<SheetRowView>,
    resistances: Option<SheetRowView>,
    biography: Vec<SheetRowView>,
}

impl Identity {
    fn read(
        sheet: &PathbuilderSheet,
        retrieval: Option<&AtlasRetrievalService>,
    ) -> AppServiceResult<Self> {
        let languages = values(&sheet.languages)
            .filter(|language| !language.trim().eq_ignore_ascii_case("None selected"))
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let row = |label, value: Option<String>| {
            value
                .filter(|s| !s.is_empty())
                .map(|v| compact_row(label, Some(v), Vec::new()))
        };
        Ok(Self {
            details: identity(sheet, retrieval)?,
            key_ability: key_ability(sheet, retrieval)?,
            languages: row("Languages", Some(languages)),
            resistances: row("Resistances", text_list(&sheet.resistances)),
            biography: [
                (
                    "Size",
                    shown(&sheet.size_name).or_else(|| shown(&sheet.size)),
                ),
                ("Age", biography(&sheet.age)),
                ("Gender", biography(&sheet.gender)),
                ("Deity", biography(&sheet.deity)),
            ]
            .into_iter()
            .filter_map(|(label, v)| row(label, v))
            .collect(),
        })
    }
    fn into_context(mut self) -> Vec<SheetGroupView> {
        self.details.rows.extend(self.key_ability);
        let rows = self
            .languages
            .into_iter()
            .chain(self.resistances)
            .chain(self.biography)
            .collect();
        vec![self.details, group("Background & details", rows)]
    }
}

#[derive(Clone)]
struct Statistics {
    defenses: SheetGroupView,
    saves: SheetGroupView,
    scores: SheetGroupView,
    skills: SheetGroupView,
    bonuses: SheetGroupView,
}

impl Statistics {
    fn into_context(self) -> Vec<SheetGroupView> {
        vec![
            self.defenses,
            self.saves,
            self.scores,
            self.skills,
            self.bonuses,
        ]
    }
    fn into_sheet(mut self, mut identity: Identity) -> Vec<SheetGroupView> {
        self.defenses.rows.extend(identity.resistances);
        self.scores.rows.extend(identity.key_ability);
        identity.details.rows.extend(identity.languages);
        identity.details.rows.extend(identity.biography);
        vec![
            self.defenses,
            self.saves,
            identity.details,
            self.scores,
            self.skills,
            self.bonuses,
        ]
    }
}

pub(super) fn project(
    sheet: &PathbuilderSheet,
    retrieval: Option<&AtlasRetrievalService>,
    layout: SheetLayout<'_>,
    output: &mut SheetGroups,
) -> AppServiceResult<()> {
    use atlas_app_model::CharacterSection::{
        Identity as IdentitySection, Statistics as StatsSection,
    };
    if layout.includes_sheet() {
        let identity = Identity::read(sheet, retrieval)?;
        let stats = statistics(sheet);
        if layout.selected(IdentitySection) {
            output
                .context
                .insert(IdentitySection, identity.clone().into_context());
        }
        if layout.selected(StatsSection) {
            output
                .context
                .insert(StatsSection, stats.clone().into_context());
        }
        output
            .sections
            .push(section(SheetSectionKind::Stats, stats.into_sheet(identity)));
    } else {
        if layout.selected(IdentitySection) {
            output.context.insert(
                IdentitySection,
                Identity::read(sheet, retrieval)?.into_context(),
            );
        }
        if layout.selected(StatsSection) {
            output
                .context
                .insert(StatsSection, statistics(sheet).into_context());
        }
    }
    Ok(())
}

fn statistics(sheet: &PathbuilderSheet) -> Statistics {
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
            "Max HP",
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
    for (name, rank) in values(&sheet.lores) {
        let label = if name.trim().is_empty() {
            "Unnamed Lore".into()
        } else {
            format!("{name} Lore")
        };
        labels.insert(label.clone());
        skills.push(proficiency_row(sheet, &label, Some(*rank), modifiers[3]));
    }
    let mut other_bonuses = Vec::new();
    for (label, bonuses) in value(&sheet.mods).into_iter().flatten() {
        if !labels.contains(label) {
            other_bonuses.push(row(label, bonus_facts(value(bonuses))));
        }
    }
    skills.sort_by_cached_key(|row| normalize_record_name(&row.label));
    let mut skills = group("Skills", skills);
    skills.kind = Some(SheetGroupKind::Skills);
    Statistics {
        defenses: group("Defenses & movement", vitals),
        saves: group("Perception & saves", saves),
        scores: group("Ability scores", scores),
        skills,
        bonuses: group("Other exported bonuses", other_bonuses),
    }
}

pub(super) fn proficiencies(sheet: &PathbuilderSheet) -> Vec<SheetGroupView> {
    let mut rows = Vec::new();
    for (key, score) in value(&sheet.proficiencies).into_iter().flatten() {
        if !SKILLS.iter().chain(SAVES).any(|(known, _, _)| known == key)
            && let Some(score) = value(score)
        {
            rows.push(compact_row(
                proficiency_label(key),
                Some(rank(*score)),
                Vec::new(),
            ));
        }
    }
    for (label, entries) in value(&sheet.specific_proficiencies).into_iter().flatten() {
        if let Some(entries) = text_list(entries) {
            rows.push(row(
                proficiency_label(label),
                vec![fact_value("Specific proficiencies", entries)],
            ));
        }
    }
    rows.sort_by_cached_key(|row| normalize_record_name(&row.label));
    let mut proficiencies = group("Other proficiencies", rows);
    proficiencies.kind = Some(SheetGroupKind::Proficiencies);
    vec![proficiencies]
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
    } else if modifier.is_some() {
        facts.push(fact_value("Rank", "Unavailable"));
    }
    if let Some(proficiency) = proficiency {
        facts.push(fact_value("Proficiency", format!("{proficiency:+}")));
    }
    if let Some(modifier) = modifier {
        facts.push(fact_value("Ability modifier", format!("{modifier:+}")));
    }
    let bonuses = value(&sheet.mods)
        .and_then(|mods| mods.get(label))
        .and_then(value);
    facts.extend(bonus_facts(bonuses));
    let summary = base
        .or(modifier)
        .map_or_else(|| "—".into(), |value| format!("{value:+}"));
    compact_row(label, Some(summary), facts)
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
        "computers" => "Computers",
        "piloting" => "Piloting",
        _ => key,
    }
    .to_owned()
}
