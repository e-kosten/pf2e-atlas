use atlas_domain::{DetailLevel, MetricDomain};
use serde::Serialize;

use crate::{
    ActivityRollAbility, ActivityRollSurface, AtlasRecord, DamageEffectKind, MechanicActivity,
    MechanicActivityKind, MechanicActivityMode, MechanicActivityUsage, MetricValue,
    SpellcastingPreparation, definition_for, metrics,
};

use super::{CreaturePerceptionJson, RecordPresentationJson};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureDefensesJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac: Option<CreatureArmorClassJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp: Option<CreatureHitPointsJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardness: Option<i64>,
    pub saves: CreatureSavesJson,
    pub immunities: Vec<CreatureIwrJson>,
    pub resistances: Vec<CreatureIwrJson>,
    pub weaknesses: Vec<CreatureIwrJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureArmorClassJson {
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureHitPointsJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broken_threshold: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureSavesJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fortitude: Option<CreatureSaveJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflex: Option<CreatureSaveJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will: Option<CreatureSaveJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSaveJson {
    pub id: &'static str,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureIwrJson {
    pub id: String,
    pub order: usize,
    pub iwr_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSenseJson {
    pub id: String,
    pub order: usize,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_feet: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSkillJson {
    pub id: String,
    pub order: usize,
    pub slug: String,
    pub label: String,
    pub modifier: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proficient: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureMovementJson {
    pub modes: Vec<CreatureMovementModeJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureMovementModeJson {
    pub id: String,
    pub order: usize,
    pub mode: String,
    pub label: String,
    pub value_feet: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureResourceJson {
    pub id: String,
    pub order: usize,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureStrikeJson {
    pub id: String,
    pub order: usize,
    pub label: String,
    pub traits: Vec<String>,
    pub usage: &'static str,
    pub rolls: Vec<CreatureRollJson>,
    pub damage: Vec<CreatureDamageJson>,
    pub modes: Vec<CreatureActivityModeJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureActionJson {
    pub id: String,
    pub order: usize,
    pub label: String,
    pub traits: Vec<String>,
    pub usage: &'static str,
    pub rolls: Vec<CreatureRollJson>,
    pub damage: Vec<CreatureDamageJson>,
    pub modes: Vec<CreatureActivityModeJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureSpellcastingJson {
    pub entries: Vec<CreatureSpellcastingEntryJson>,
    pub spells: Vec<CreatureSpellJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellcastingEntryJson {
    pub id: String,
    pub order: usize,
    pub label: String,
    pub preparation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellJson {
    pub id: String,
    pub order: usize,
    pub label: String,
    pub traits: Vec<String>,
    pub usage: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compendium_source: Option<String>,
    pub rolls: Vec<CreatureRollJson>,
    pub damage: Vec<CreatureDamageJson>,
    pub modes: Vec<CreatureActivityModeJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureRollJson {
    pub id: String,
    pub label: String,
    pub value: i64,
    pub surface: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureDamageJson {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub formula: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage_type: Option<String>,
    pub effect: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureActivityModeJson {
    pub id: String,
    pub order: usize,
    pub label: String,
    pub sort: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    pub damage: Vec<CreatureDamageJson>,
}

pub(super) fn creature_presentation(
    record: &AtlasRecord,
    detail: DetailLevel,
) -> RecordPresentationJson {
    let scans_mechanics = matches!(
        detail,
        DetailLevel::Preview | DetailLevel::Standard | DetailLevel::Full
    );
    let normal_mechanics = matches!(detail, DetailLevel::Standard | DetailLevel::Full);

    let (defenses, perception, languages, skills, movement, resources) = if scans_mechanics {
        (
            defenses(record),
            perception(record),
            record
                .mechanics
                .actor()
                .map(|actor| actor.languages.clone())
                .unwrap_or_default(),
            skills(record),
            movement(record),
            resources(record),
        )
    } else {
        (
            CreatureDefensesJson::default(),
            None,
            Vec::new(),
            Vec::new(),
            CreatureMovementJson::default(),
            Vec::new(),
        )
    };

    let mut strikes = Vec::new();
    let mut actions = Vec::new();
    let mut spells = Vec::new();
    if scans_mechanics {
        for (order, activity) in record.mechanics.activities.iter().enumerate() {
            match activity.kind {
                MechanicActivityKind::Strike => {
                    strikes.push(strike(activity, order, normal_mechanics))
                }
                MechanicActivityKind::Other => {
                    actions.push(action(activity, order, normal_mechanics))
                }
                MechanicActivityKind::Spell => {
                    spells.push(spell(activity, order, normal_mechanics))
                }
            }
        }
    }
    let entries = if scans_mechanics {
        record
            .mechanics
            .spellcasting_entries
            .iter()
            .enumerate()
            .map(|(order, entry)| CreatureSpellcastingEntryJson {
                id: entry.entry_id.clone(),
                order,
                label: entry.label.clone(),
                preparation: preparation(&entry.preparation),
                attack: entry.spell_attack,
                dc: entry.spell_dc,
            })
            .collect()
    } else {
        Vec::new()
    };

    RecordPresentationJson::Creature {
        defenses,
        perception,
        languages,
        skills,
        movement,
        resources,
        strikes,
        actions,
        spellcasting: CreatureSpellcastingJson { entries, spells },
    }
}

fn defenses(record: &AtlasRecord) -> CreatureDefensesJson {
    let actor = record.mechanics.actor();
    CreatureDefensesJson {
        ac: metric_i64(
            record,
            metrics::actor::ARMOR_CLASS.exact_key().expect("static key"),
        )
        .map(|value| CreatureArmorClassJson { value }),
        hp: hp(record),
        hardness: metric_i64(
            record,
            metrics::actor::HARDNESS.exact_key().expect("static key"),
        ),
        saves: CreatureSavesJson {
            fortitude: save(record, "fort", "fortitude"),
            reflex: save(record, "ref", "reflex"),
            will: save(record, "will", "will"),
        },
        immunities: actor
            .map(|actor| iwr("immunity", &actor.immunities))
            .unwrap_or_default(),
        resistances: actor
            .map(|actor| iwr("resistance", &actor.resistances))
            .unwrap_or_default(),
        weaknesses: actor
            .map(|actor| iwr("weakness", &actor.weaknesses))
            .unwrap_or_default(),
    }
}

fn hp(record: &AtlasRecord) -> Option<CreatureHitPointsJson> {
    let value = metric_i64(
        record,
        metrics::actor::HP_VALUE.exact_key().expect("static key"),
    );
    let maximum = metric_i64(
        record,
        metrics::actor::HP_MAX.exact_key().expect("static key"),
    );
    let broken_threshold = metric_i64(
        record,
        metrics::actor::HP_BROKEN_THRESHOLD
            .exact_key()
            .expect("static key"),
    );
    (value.is_some() || maximum.is_some() || broken_threshold.is_some()).then_some(
        CreatureHitPointsJson {
            value,
            maximum,
            broken_threshold,
        },
    )
}

fn save(record: &AtlasRecord, slug: &str, id: &'static str) -> Option<CreatureSaveJson> {
    metric_i64(record, &metrics::actor::save::mod_key(slug))
        .map(|value| CreatureSaveJson { id, value })
}

fn iwr(prefix: &str, values: &[String]) -> Vec<CreatureIwrJson> {
    values
        .iter()
        .enumerate()
        .map(|(order, value)| CreatureIwrJson {
            id: format!("{prefix}-{order}"),
            order,
            iwr_type: value.clone(),
        })
        .collect()
}

fn perception(record: &AtlasRecord) -> Option<CreaturePerceptionJson> {
    let modifier = metric_i64(
        record,
        metrics::actor::PERCEPTION_MOD
            .exact_key()
            .expect("static key"),
    );
    let senses = record
        .mechanics
        .actor()
        .map(|actor| {
            actor
                .senses
                .iter()
                .enumerate()
                .map(|(order, sense)| {
                    let kind = stable_slug(sense);
                    CreatureSenseJson {
                        id: format!("sense-{order}"),
                        order,
                        range_feet: metric_i64(record, &metrics::actor::sense::range_key(&kind)),
                        kind,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    (modifier.is_some() || !senses.is_empty())
        .then_some(CreaturePerceptionJson { modifier, senses })
}

fn skills(record: &AtlasRecord) -> Vec<CreatureSkillJson> {
    let mut skills = Vec::new();
    for metric in &record.mechanics.metrics {
        if metric.domain != MetricDomain::Actor {
            continue;
        }
        let Some(found) = definition_for(metric.domain, &metric.key) else {
            continue;
        };
        if *found.definition != metrics::actor::skill::MOD {
            continue;
        }
        let Some(capture) = found.captures.first() else {
            continue;
        };
        let MetricValue::Number(value) = metric.value else {
            continue;
        };
        if value.fract() != 0.0 {
            continue;
        }
        let order = skills.len();
        skills.push(CreatureSkillJson {
            id: format!("skill:{}", capture.raw),
            order,
            slug: capture.raw.clone(),
            label: capture.label.clone(),
            modifier: value as i64,
            rank: metric_i64(record, &metrics::actor::skill::rank_key(&capture.raw)),
            proficient: metric_bool(record, &metrics::actor::skill::proficient_key(&capture.raw)),
        });
    }
    skills
}

fn movement(record: &AtlasRecord) -> CreatureMovementJson {
    let mut modes = Vec::new();
    for metric in &record.mechanics.metrics {
        if metric.domain != MetricDomain::Actor {
            continue;
        }
        let Some(found) = definition_for(metric.domain, &metric.key) else {
            continue;
        };
        if *found.definition != metrics::actor::speed::VALUE {
            continue;
        }
        let Some(capture) = found.captures.first() else {
            continue;
        };
        let MetricValue::Number(value) = metric.value else {
            continue;
        };
        if value.fract() != 0.0 || value <= 0.0 {
            continue;
        }
        let order = modes.len();
        modes.push(CreatureMovementModeJson {
            id: format!("movement:{}", capture.raw),
            order,
            mode: capture.raw.clone(),
            label: format!("{} Speed", capture.label),
            value_feet: value as i64,
        });
    }
    CreatureMovementJson { modes }
}

fn resources(_record: &AtlasRecord) -> Vec<CreatureResourceJson> {
    // The relational CLI record currently carries no canonical resource rows.
    // Keep the direct typed collection empty rather than infer resource meaning
    // from undefined metric-key patterns or raw source.
    Vec::new()
}

fn strike(activity: &MechanicActivity, order: usize, full: bool) -> CreatureStrikeJson {
    CreatureStrikeJson {
        id: activity.activity_id.clone(),
        order,
        label: activity.label.clone(),
        traits: activity.traits.clone(),
        usage: usage(activity.usage),
        rolls: if full { rolls(activity) } else { Vec::new() },
        damage: if full {
            damage(&activity.damage)
        } else {
            Vec::new()
        },
        modes: if full {
            modes(&activity.modes)
        } else {
            Vec::new()
        },
    }
}

fn action(activity: &MechanicActivity, order: usize, full: bool) -> CreatureActionJson {
    CreatureActionJson {
        id: activity.activity_id.clone(),
        order,
        label: activity.label.clone(),
        traits: activity.traits.clone(),
        usage: usage(activity.usage),
        rolls: if full { rolls(activity) } else { Vec::new() },
        damage: if full {
            damage(&activity.damage)
        } else {
            Vec::new()
        },
        modes: if full {
            modes(&activity.modes)
        } else {
            Vec::new()
        },
    }
}

fn spell(activity: &MechanicActivity, order: usize, full: bool) -> CreatureSpellJson {
    CreatureSpellJson {
        id: activity.activity_id.clone(),
        order,
        label: activity.label.clone(),
        traits: activity.traits.clone(),
        usage: usage(activity.usage),
        compendium_source: activity.compendium_source.clone(),
        rolls: if full { rolls(activity) } else { Vec::new() },
        damage: if full {
            damage(&activity.damage)
        } else {
            Vec::new()
        },
        modes: if full {
            modes(&activity.modes)
        } else {
            Vec::new()
        },
    }
}

fn rolls(activity: &MechanicActivity) -> Vec<CreatureRollJson> {
    activity
        .rolls
        .iter()
        .map(|roll| CreatureRollJson {
            id: roll.roll_id.clone(),
            label: roll.label.clone(),
            value: roll.base_value,
            surface: match roll.surface {
                ActivityRollSurface::AttackRoll => "attack_roll",
                ActivityRollSurface::Dc => "dc",
            },
            ability: roll.ability.map(ability),
        })
        .collect()
}

fn damage(values: &[crate::DamageExpression]) -> Vec<CreatureDamageJson> {
    values
        .iter()
        .map(|value| CreatureDamageJson {
            id: value.damage_id.clone(),
            label: value.label.clone(),
            formula: value.formula.clone(),
            damage_type: value.damage_type.clone(),
            effect: match value.effect_kind {
                DamageEffectKind::Damage => "damage",
                DamageEffectKind::Healing => "healing",
                DamageEffectKind::DamageOrHealing => "damage_or_healing",
                DamageEffectKind::Unknown => "unknown",
            },
            ability: value.ability.map(ability),
        })
        .collect()
}

fn modes(values: &[MechanicActivityMode]) -> Vec<CreatureActivityModeJson> {
    values
        .iter()
        .enumerate()
        .map(|(order, mode)| CreatureActivityModeJson {
            id: mode.mode_id.clone(),
            order,
            label: mode.label.clone(),
            sort: mode.sort,
            target: mode.target.clone(),
            range: mode.range.clone(),
            time: mode.time.clone(),
            damage: damage(&mode.damage),
        })
        .collect()
}

fn usage(value: MechanicActivityUsage) -> &'static str {
    match value {
        MechanicActivityUsage::Unlimited => "unlimited",
        MechanicActivityUsage::Limited => "limited",
        MechanicActivityUsage::Ambiguous => "ambiguous",
    }
}

fn preparation(value: &SpellcastingPreparation) -> String {
    match value {
        SpellcastingPreparation::Prepared => "prepared".to_string(),
        SpellcastingPreparation::Spontaneous => "spontaneous".to_string(),
        SpellcastingPreparation::Focus => "focus".to_string(),
        SpellcastingPreparation::Innate => "innate".to_string(),
        SpellcastingPreparation::Other(value) => value.clone(),
    }
}

fn ability(value: ActivityRollAbility) -> &'static str {
    match value {
        ActivityRollAbility::Strength => "strength",
        ActivityRollAbility::Dexterity => "dexterity",
        ActivityRollAbility::Constitution => "constitution",
        ActivityRollAbility::Intelligence => "intelligence",
        ActivityRollAbility::Wisdom => "wisdom",
        ActivityRollAbility::Charisma => "charisma",
    }
}

fn metric_i64(record: &AtlasRecord, key: &str) -> Option<i64> {
    record.mechanics.metrics.iter().find_map(|metric| {
        if metric.domain != MetricDomain::Actor || metric.key != key {
            return None;
        }
        let MetricValue::Number(value) = metric.value else {
            return None;
        };
        (value.fract() == 0.0).then_some(value as i64)
    })
}

fn metric_bool(record: &AtlasRecord, key: &str) -> Option<bool> {
    record.mechanics.metrics.iter().find_map(|metric| {
        if metric.domain != MetricDomain::Actor || metric.key != key {
            return None;
        }
        let MetricValue::Boolean(value) = metric.value else {
            return None;
        };
        Some(value)
    })
}

fn stable_slug(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
