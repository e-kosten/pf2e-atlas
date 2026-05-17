use atlas_domain::{MetricDomain, MetricValueType};

use crate::MetricRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricGroup {
    Abilities,
    Defense,
    Health,
    Perception,
    Saves,
    Skills,
    Movement,
    Senses,
    Stealth,
    Items,
    Disable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricLabelTemplate {
    Static(&'static str),
    FoundryI18n {
        key: &'static str,
        fallback: &'static str,
    },
    Template(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricVariableVocabulary {
    Ability,
    Skill,
    Save,
    MovementType,
    SenseType,
    FreeSlug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKeySegment {
    Literal(&'static str),
    Variable {
        name: &'static str,
        vocabulary: MetricVariableVocabulary,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricKeyPattern {
    pub segments: &'static [MetricKeySegment],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticMetricDefinition {
    pub domain: MetricDomain,
    pub key: &'static str,
    pub value_type: MetricValueType,
    pub namespace: &'static str,
    pub label: MetricLabelTemplate,
    pub short_label: Option<MetricLabelTemplate>,
    pub group: MetricGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternMetricDefinition {
    pub domain: MetricDomain,
    pub pattern: MetricKeyPattern,
    pub value_type: MetricValueType,
    pub namespace: &'static str,
    pub label: MetricLabelTemplate,
    pub short_label: Option<MetricLabelTemplate>,
    pub group: MetricGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricDefinition {
    Static(StaticMetricDefinition),
    Pattern(PatternMetricDefinition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCapture {
    pub name: &'static str,
    pub raw: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDefinitionMatch {
    pub definition: &'static MetricDefinition,
    pub captures: Vec<MetricCapture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDisplayLabel {
    pub label: String,
    pub short_label: Option<String>,
    pub known: bool,
}

impl MetricDefinition {
    pub const fn exact_key(self) -> Option<&'static str> {
        match self {
            Self::Static(definition) => Some(definition.key),
            Self::Pattern(_) => None,
        }
    }

    pub const fn domain(self) -> MetricDomain {
        match self {
            Self::Static(definition) => definition.domain,
            Self::Pattern(definition) => definition.domain,
        }
    }

    pub const fn value_type(self) -> MetricValueType {
        match self {
            Self::Static(definition) => definition.value_type,
            Self::Pattern(definition) => definition.value_type,
        }
    }

    pub const fn namespace(self) -> &'static str {
        match self {
            Self::Static(definition) => definition.namespace,
            Self::Pattern(definition) => definition.namespace,
        }
    }

    pub const fn group(self) -> MetricGroup {
        match self {
            Self::Static(definition) => definition.group,
            Self::Pattern(definition) => definition.group,
        }
    }

    const fn label_template(self) -> MetricLabelTemplate {
        match self {
            Self::Static(definition) => definition.label,
            Self::Pattern(definition) => definition.label,
        }
    }

    const fn short_label_template(self) -> Option<MetricLabelTemplate> {
        match self {
            Self::Static(definition) => definition.short_label,
            Self::Pattern(definition) => definition.short_label,
        }
    }
}

impl MetricDefinitionMatch {
    pub fn label(&self) -> MetricDisplayLabel {
        let definition = *self.definition;
        MetricDisplayLabel {
            label: render_template(definition.label_template(), &self.captures),
            short_label: definition
                .short_label_template()
                .map(|template| render_template(template, &self.captures)),
            known: true,
        }
    }
}

pub fn all_definitions() -> &'static [MetricDefinition] {
    DEFINITIONS
}

pub fn definition_for(domain: MetricDomain, key: &str) -> Option<MetricDefinitionMatch> {
    for definition in DEFINITIONS {
        if let MetricDefinition::Static(static_definition) = definition
            && static_definition.domain == domain
            && static_definition.key == key
        {
            return Some(MetricDefinitionMatch {
                definition,
                captures: Vec::new(),
            });
        }
    }

    for definition in DEFINITIONS {
        if let MetricDefinition::Pattern(pattern_definition) = definition
            && pattern_definition.domain == domain
            && let Some(captures) = match_pattern(pattern_definition.pattern, key)
        {
            return Some(MetricDefinitionMatch {
                definition,
                captures,
            });
        }
    }

    None
}

pub fn is_known_key(domain: MetricDomain, key: &str) -> bool {
    definition_for(domain, key).is_some()
}

pub fn label_for_row(row: &MetricRow) -> MetricDisplayLabel {
    definition_for(row.domain, &row.key)
        .map(|matched| matched.label())
        .unwrap_or_else(|| MetricDisplayLabel {
            label: row.key.clone(),
            short_label: None,
            known: false,
        })
}

fn match_pattern(pattern: MetricKeyPattern, key: &str) -> Option<Vec<MetricCapture>> {
    let key_segments = key.split('.').collect::<Vec<_>>();
    if key_segments.len() != pattern.segments.len() {
        return None;
    }

    let mut captures = Vec::new();
    for (pattern_segment, key_segment) in pattern.segments.iter().zip(key_segments) {
        match pattern_segment {
            MetricKeySegment::Literal(literal) if *literal == key_segment => {}
            MetricKeySegment::Literal(_) => return None,
            MetricKeySegment::Variable { name, vocabulary } => {
                let label = format_variable(*vocabulary, key_segment)?;
                captures.push(MetricCapture {
                    name,
                    raw: key_segment.to_string(),
                    label,
                });
            }
        }
    }
    Some(captures)
}

fn format_variable(vocabulary: MetricVariableVocabulary, value: &str) -> Option<String> {
    let label = match vocabulary {
        MetricVariableVocabulary::Ability => match value {
            "str" => "Str",
            "dex" => "Dex",
            "con" => "Con",
            "int" => "Int",
            "wis" => "Wis",
            "cha" => "Cha",
            _ => return None,
        }
        .to_string(),
        MetricVariableVocabulary::Save => match value {
            "fort" => "Fortitude",
            "ref" => "Reflex",
            "will" => "Will",
            _ => return None,
        }
        .to_string(),
        MetricVariableVocabulary::Skill => skill_label(value)
            .map(str::to_string)
            .unwrap_or_else(|| titleize_slug(value)),
        MetricVariableVocabulary::MovementType
        | MetricVariableVocabulary::SenseType
        | MetricVariableVocabulary::FreeSlug => titleize_slug(value),
    };
    Some(label)
}

fn skill_label(value: &str) -> Option<&'static str> {
    match value {
        "acrobatics" => Some("Acrobatics"),
        "arcana" => Some("Arcana"),
        "athletics" => Some("Athletics"),
        "crafting" => Some("Crafting"),
        "deception" => Some("Deception"),
        "diplomacy" => Some("Diplomacy"),
        "intimidation" => Some("Intimidation"),
        "medicine" => Some("Medicine"),
        "nature" => Some("Nature"),
        "occultism" => Some("Occultism"),
        "performance" => Some("Performance"),
        "religion" => Some("Religion"),
        "society" => Some("Society"),
        "stealth" => Some("Stealth"),
        "survival" => Some("Survival"),
        "thievery" => Some("Thievery"),
        _ => None,
    }
}

fn render_template(template: MetricLabelTemplate, captures: &[MetricCapture]) -> String {
    match template {
        MetricLabelTemplate::Static(label) => label.to_string(),
        MetricLabelTemplate::FoundryI18n { fallback, .. } => fallback.to_string(),
        MetricLabelTemplate::Template(template) => {
            let mut rendered = template.to_string();
            for capture in captures {
                rendered = rendered.replace(&format!("{{{}}}", capture.name), &capture.label);
            }
            rendered
        }
    }
}

fn titleize_slug(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .enumerate()
        .map(|(index, part)| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            if index == 0 {
                format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
            } else {
                format!("{first}{}", chars.as_str())
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

const fn static_definition(
    domain: MetricDomain,
    key: &'static str,
    value_type: MetricValueType,
    namespace: &'static str,
    label: MetricLabelTemplate,
    short_label: Option<MetricLabelTemplate>,
    group: MetricGroup,
) -> MetricDefinition {
    MetricDefinition::Static(StaticMetricDefinition {
        domain,
        key,
        value_type,
        namespace,
        label,
        short_label,
        group,
    })
}

const fn pattern_definition(
    domain: MetricDomain,
    pattern: MetricKeyPattern,
    value_type: MetricValueType,
    namespace: &'static str,
    label: MetricLabelTemplate,
    short_label: Option<MetricLabelTemplate>,
    group: MetricGroup,
) -> MetricDefinition {
    MetricDefinition::Pattern(PatternMetricDefinition {
        domain,
        pattern,
        value_type,
        namespace,
        label,
        short_label,
        group,
    })
}

pub mod actor {
    use super::*;

    pub const ARMOR_CLASS: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "ac.value",
        MetricValueType::Number,
        "ac",
        MetricLabelTemplate::FoundryI18n {
            key: "PF2E.ArmorClassLabel",
            fallback: "Armor Class",
        },
        Some(MetricLabelTemplate::FoundryI18n {
            key: "PF2E.ArmorClassShortLabel",
            fallback: "AC",
        }),
        MetricGroup::Defense,
    );
    pub const HARDNESS: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "hardness.value",
        MetricValueType::Number,
        "hardness",
        MetricLabelTemplate::Static("Hardness"),
        None,
        MetricGroup::Defense,
    );
    pub const HP_VALUE: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "hp.value",
        MetricValueType::Number,
        "hp",
        MetricLabelTemplate::Static("Hit Points"),
        Some(MetricLabelTemplate::Static("HP")),
        MetricGroup::Health,
    );
    pub const HP_MAX: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "hp.max",
        MetricValueType::Number,
        "hp",
        MetricLabelTemplate::Static("Maximum Hit Points"),
        Some(MetricLabelTemplate::Static("Max HP")),
        MetricGroup::Health,
    );
    pub const HP_BROKEN_THRESHOLD: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "hp.bt",
        MetricValueType::Number,
        "hp",
        MetricLabelTemplate::Static("Broken Threshold"),
        Some(MetricLabelTemplate::Static("BT")),
        MetricGroup::Health,
    );
    pub const PERCEPTION_MOD: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "perception.mod",
        MetricValueType::Number,
        "perception",
        MetricLabelTemplate::Static("Perception modifier"),
        Some(MetricLabelTemplate::Static("Perception")),
        MetricGroup::Perception,
    );
    pub const STEALTH_MOD: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "stealth.mod",
        MetricValueType::Number,
        "stealth",
        MetricLabelTemplate::Static("Stealth modifier"),
        Some(MetricLabelTemplate::Static("Stealth")),
        MetricGroup::Stealth,
    );
    pub const STEALTH_DC: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "stealth.dc",
        MetricValueType::Number,
        "stealth",
        MetricLabelTemplate::Static("Stealth DC"),
        None,
        MetricGroup::Stealth,
    );

    pub mod ability {
        use super::*;

        const ABILITY_MOD_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("ability"),
            MetricKeySegment::Variable {
                name: "ability",
                vocabulary: MetricVariableVocabulary::Ability,
            },
            MetricKeySegment::Literal("mod"),
        ];

        pub const MOD: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: ABILITY_MOD_SEGMENTS,
            },
            MetricValueType::Number,
            "ability",
            MetricLabelTemplate::Template("{ability} modifier"),
            Some(MetricLabelTemplate::Template("{ability}")),
            MetricGroup::Abilities,
        );

        pub fn mod_key(ability: &str) -> String {
            format!("ability.{ability}.mod")
        }
    }

    pub mod save {
        use super::*;

        const SAVE_MOD_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("save"),
            MetricKeySegment::Variable {
                name: "save",
                vocabulary: MetricVariableVocabulary::Save,
            },
            MetricKeySegment::Literal("mod"),
        ];

        pub const MOD: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: SAVE_MOD_SEGMENTS,
            },
            MetricValueType::Number,
            "save",
            MetricLabelTemplate::Template("{save} modifier"),
            None,
            MetricGroup::Saves,
        );
        pub const BEST: MetricDefinition = static_definition(
            MetricDomain::Actor,
            "save.best",
            MetricValueType::Text,
            "save",
            MetricLabelTemplate::Static("Best save"),
            None,
            MetricGroup::Saves,
        );
        pub const WORST: MetricDefinition = static_definition(
            MetricDomain::Actor,
            "save.worst",
            MetricValueType::Text,
            "save",
            MetricLabelTemplate::Static("Worst save"),
            None,
            MetricGroup::Saves,
        );

        pub fn mod_key(save: &str) -> String {
            format!("save.{save}.mod")
        }
    }

    pub mod skill {
        use super::*;

        const SKILL_MOD_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("skill"),
            MetricKeySegment::Variable {
                name: "skill",
                vocabulary: MetricVariableVocabulary::Skill,
            },
            MetricKeySegment::Literal("mod"),
        ];
        const SKILL_RANK_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("skill"),
            MetricKeySegment::Variable {
                name: "skill",
                vocabulary: MetricVariableVocabulary::Skill,
            },
            MetricKeySegment::Literal("rank"),
        ];
        const SKILL_PROFICIENT_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("skill"),
            MetricKeySegment::Variable {
                name: "skill",
                vocabulary: MetricVariableVocabulary::Skill,
            },
            MetricKeySegment::Literal("proficient"),
        ];

        pub const MOD: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: SKILL_MOD_SEGMENTS,
            },
            MetricValueType::Number,
            "skill",
            MetricLabelTemplate::Template("{skill} modifier"),
            Some(MetricLabelTemplate::Template("{skill}")),
            MetricGroup::Skills,
        );
        pub const RANK: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: SKILL_RANK_SEGMENTS,
            },
            MetricValueType::Number,
            "skill",
            MetricLabelTemplate::Template("{skill} rank"),
            None,
            MetricGroup::Skills,
        );
        pub const PROFICIENT: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: SKILL_PROFICIENT_SEGMENTS,
            },
            MetricValueType::Boolean,
            "skill",
            MetricLabelTemplate::Template("{skill} proficiency"),
            None,
            MetricGroup::Skills,
        );

        pub fn mod_key(skill: &str) -> String {
            format!("skill.{skill}.mod")
        }

        pub fn rank_key(skill: &str) -> String {
            format!("skill.{skill}.rank")
        }

        pub fn proficient_key(skill: &str) -> String {
            format!("skill.{skill}.proficient")
        }
    }

    pub mod speed {
        use super::*;

        const SPEED_VALUE_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("speed"),
            MetricKeySegment::Variable {
                name: "movement",
                vocabulary: MetricVariableVocabulary::MovementType,
            },
            MetricKeySegment::Literal("value"),
        ];

        pub const VALUE: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: SPEED_VALUE_SEGMENTS,
            },
            MetricValueType::Number,
            "speed",
            MetricLabelTemplate::Template("{movement} Speed"),
            None,
            MetricGroup::Movement,
        );

        pub fn value_key(movement: &str) -> String {
            format!("speed.{movement}.value")
        }
    }

    pub mod sense {
        use super::*;

        const SENSE_RANGE_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("sense"),
            MetricKeySegment::Variable {
                name: "sense",
                vocabulary: MetricVariableVocabulary::SenseType,
            },
            MetricKeySegment::Literal("range"),
        ];

        pub const RANGE: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: SENSE_RANGE_SEGMENTS,
            },
            MetricValueType::Number,
            "sense",
            MetricLabelTemplate::Template("{sense} range"),
            None,
            MetricGroup::Senses,
        );

        pub fn range_key(sense: &str) -> String {
            format!("sense.{sense}.range")
        }
    }

    pub mod disable {
        use super::*;

        const DISABLE_SKILL_DC_MIN_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("disable"),
            MetricKeySegment::Variable {
                name: "skill",
                vocabulary: MetricVariableVocabulary::Skill,
            },
            MetricKeySegment::Literal("dc"),
            MetricKeySegment::Literal("min"),
        ];
        const DISABLE_SKILL_DC_MAX_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("disable"),
            MetricKeySegment::Variable {
                name: "skill",
                vocabulary: MetricVariableVocabulary::Skill,
            },
            MetricKeySegment::Literal("dc"),
            MetricKeySegment::Literal("max"),
        ];
        const DISABLE_SKILL_RANK_MIN_SEGMENTS: &[MetricKeySegment] = &[
            MetricKeySegment::Literal("disable"),
            MetricKeySegment::Variable {
                name: "skill",
                vocabulary: MetricVariableVocabulary::Skill,
            },
            MetricKeySegment::Literal("rank"),
            MetricKeySegment::Literal("min"),
        ];

        pub const DC_MIN: MetricDefinition = static_definition(
            MetricDomain::Actor,
            "disable.dc.min",
            MetricValueType::Number,
            "disable",
            MetricLabelTemplate::Static("Minimum disable DC"),
            Some(MetricLabelTemplate::Static("Disable DC")),
            MetricGroup::Disable,
        );
        pub const DC_MAX: MetricDefinition = static_definition(
            MetricDomain::Actor,
            "disable.dc.max",
            MetricValueType::Number,
            "disable",
            MetricLabelTemplate::Static("Maximum disable DC"),
            Some(MetricLabelTemplate::Static("Disable DC max")),
            MetricGroup::Disable,
        );
        pub const SKILL_DC_MIN: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: DISABLE_SKILL_DC_MIN_SEGMENTS,
            },
            MetricValueType::Number,
            "disable",
            MetricLabelTemplate::Template("Minimum {skill} disable DC"),
            Some(MetricLabelTemplate::Template("{skill} disable DC")),
            MetricGroup::Disable,
        );
        pub const SKILL_DC_MAX: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: DISABLE_SKILL_DC_MAX_SEGMENTS,
            },
            MetricValueType::Number,
            "disable",
            MetricLabelTemplate::Template("Maximum {skill} disable DC"),
            Some(MetricLabelTemplate::Template("{skill} disable DC max")),
            MetricGroup::Disable,
        );
        pub const SKILL_RANK_MIN: MetricDefinition = pattern_definition(
            MetricDomain::Actor,
            MetricKeyPattern {
                segments: DISABLE_SKILL_RANK_MIN_SEGMENTS,
            },
            MetricValueType::Number,
            "disable",
            MetricLabelTemplate::Template("Minimum {skill} disable rank"),
            None,
            MetricGroup::Disable,
        );

        pub fn skill_dc_min_key(skill: &str) -> String {
            format!("disable.{skill}.dc.min")
        }

        pub fn skill_dc_max_key(skill: &str) -> String {
            format!("disable.{skill}.dc.max")
        }

        pub fn skill_rank_min_key(skill: &str) -> String {
            format!("disable.{skill}.rank.min")
        }
    }
}

pub mod item {
    use super::*;

    pub mod armor {
        use super::*;

        pub const AC_BONUS: MetricDefinition = static_definition(
            MetricDomain::Item,
            "armor.ac_bonus",
            MetricValueType::Number,
            "armor",
            MetricLabelTemplate::Static("Armor AC bonus"),
            Some(MetricLabelTemplate::Static("AC bonus")),
            MetricGroup::Items,
        );
        pub const DEX_CAP: MetricDefinition = static_definition(
            MetricDomain::Item,
            "armor.dex_cap",
            MetricValueType::Number,
            "armor",
            MetricLabelTemplate::Static("Armor Dexterity cap"),
            Some(MetricLabelTemplate::Static("Dex cap")),
            MetricGroup::Items,
        );
        pub const STRENGTH: MetricDefinition = static_definition(
            MetricDomain::Item,
            "armor.strength",
            MetricValueType::Number,
            "armor",
            MetricLabelTemplate::Static("Armor Strength requirement"),
            Some(MetricLabelTemplate::Static("Strength")),
            MetricGroup::Items,
        );
        pub const CHECK_PENALTY: MetricDefinition = static_definition(
            MetricDomain::Item,
            "armor.check_penalty",
            MetricValueType::Number,
            "armor",
            MetricLabelTemplate::Static("Armor check penalty"),
            Some(MetricLabelTemplate::Static("Check penalty")),
            MetricGroup::Items,
        );
        pub const SPEED_PENALTY: MetricDefinition = static_definition(
            MetricDomain::Item,
            "armor.speed_penalty",
            MetricValueType::Number,
            "armor",
            MetricLabelTemplate::Static("Armor Speed penalty"),
            Some(MetricLabelTemplate::Static("Speed penalty")),
            MetricGroup::Items,
        );
    }

    pub mod shield {
        use super::*;

        pub const AC_BONUS: MetricDefinition = static_definition(
            MetricDomain::Item,
            "shield.ac_bonus",
            MetricValueType::Number,
            "shield",
            MetricLabelTemplate::Static("Shield AC bonus"),
            Some(MetricLabelTemplate::Static("AC bonus")),
            MetricGroup::Items,
        );
        pub const HARDNESS: MetricDefinition = static_definition(
            MetricDomain::Item,
            "shield.hardness",
            MetricValueType::Number,
            "shield",
            MetricLabelTemplate::Static("Shield Hardness"),
            Some(MetricLabelTemplate::Static("Hardness")),
            MetricGroup::Items,
        );
        pub const HP: MetricDefinition = static_definition(
            MetricDomain::Item,
            "shield.hp",
            MetricValueType::Number,
            "shield",
            MetricLabelTemplate::Static("Shield Hit Points"),
            Some(MetricLabelTemplate::Static("HP")),
            MetricGroup::Items,
        );
        pub const BROKEN_THRESHOLD: MetricDefinition = static_definition(
            MetricDomain::Item,
            "shield.bt",
            MetricValueType::Number,
            "shield",
            MetricLabelTemplate::Static("Shield Broken Threshold"),
            Some(MetricLabelTemplate::Static("BT")),
            MetricGroup::Items,
        );
    }

    pub mod weapon {
        use super::*;

        pub const RANGE_INCREMENT: MetricDefinition = static_definition(
            MetricDomain::Item,
            "weapon.range_increment",
            MetricValueType::Number,
            "weapon",
            MetricLabelTemplate::Static("Weapon range increment"),
            Some(MetricLabelTemplate::Static("Range")),
            MetricGroup::Items,
        );
        pub const RELOAD: MetricDefinition = static_definition(
            MetricDomain::Item,
            "weapon.reload",
            MetricValueType::Number,
            "weapon",
            MetricLabelTemplate::Static("Weapon reload"),
            Some(MetricLabelTemplate::Static("Reload")),
            MetricGroup::Items,
        );
        pub const DAMAGE_DICE: MetricDefinition = static_definition(
            MetricDomain::Item,
            "weapon.damage_dice",
            MetricValueType::Number,
            "weapon",
            MetricLabelTemplate::Static("Weapon damage dice"),
            Some(MetricLabelTemplate::Static("Damage dice")),
            MetricGroup::Items,
        );
        pub const DAMAGE_DIE_FACES: MetricDefinition = static_definition(
            MetricDomain::Item,
            "weapon.damage_die_faces",
            MetricValueType::Number,
            "weapon",
            MetricLabelTemplate::Static("Weapon damage die faces"),
            Some(MetricLabelTemplate::Static("Damage die")),
            MetricGroup::Items,
        );
    }
}

static DEFINITIONS: &[MetricDefinition] = &[
    actor::ARMOR_CLASS,
    actor::HARDNESS,
    actor::HP_VALUE,
    actor::HP_MAX,
    actor::HP_BROKEN_THRESHOLD,
    actor::PERCEPTION_MOD,
    actor::STEALTH_MOD,
    actor::STEALTH_DC,
    actor::ability::MOD,
    actor::save::MOD,
    actor::save::BEST,
    actor::save::WORST,
    actor::skill::MOD,
    actor::skill::RANK,
    actor::skill::PROFICIENT,
    actor::speed::VALUE,
    actor::sense::RANGE,
    actor::disable::DC_MIN,
    actor::disable::DC_MAX,
    actor::disable::SKILL_DC_MIN,
    actor::disable::SKILL_DC_MAX,
    actor::disable::SKILL_RANK_MIN,
    item::armor::AC_BONUS,
    item::armor::DEX_CAP,
    item::armor::STRENGTH,
    item::armor::CHECK_PENALTY,
    item::armor::SPEED_PENALTY,
    item::shield::AC_BONUS,
    item::shield::HARDNESS,
    item::shield::HP,
    item::shield::BROKEN_THRESHOLD,
    item::weapon::RANGE_INCREMENT,
    item::weapon::RELOAD,
    item::weapon::DAMAGE_DICE,
    item::weapon::DAMAGE_DIE_FACES,
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::MetricValue;

    #[test]
    fn exact_static_definitions_win_before_patterns() {
        let matched = definition_for(MetricDomain::Actor, "ac.value").expect("metric is known");

        assert_eq!(matched.definition.group(), MetricGroup::Defense);
        assert!(matched.captures.is_empty());
        assert_eq!(matched.label().label, "Armor Class");
        assert_eq!(matched.label().short_label.as_deref(), Some("AC"));
    }

    #[test]
    fn pattern_definitions_capture_and_label_variables() {
        let matched =
            definition_for(MetricDomain::Actor, "skill.arcana.mod").expect("metric is known");

        assert_eq!(matched.definition.group(), MetricGroup::Skills);
        assert_eq!(matched.captures[0].name, "skill");
        assert_eq!(matched.captures[0].raw, "arcana");
        assert_eq!(matched.captures[0].label, "Arcana");
        assert_eq!(matched.label().label, "Arcana modifier");
    }

    #[test]
    fn open_vocabularies_titleize_captured_slugs() {
        let matched =
            definition_for(MetricDomain::Actor, "speed.spider_climb.value").expect("metric known");

        assert_eq!(matched.label().label, "Spider climb Speed");
    }

    #[test]
    fn unknown_rows_fall_back_to_raw_key_labels() {
        let row = MetricRow {
            domain: MetricDomain::Actor,
            key: "unknown.metric".to_string(),
            value: MetricValue::Number(12.0),
        };

        assert_eq!(
            label_for_row(&row),
            MetricDisplayLabel {
                label: "unknown.metric".to_string(),
                short_label: None,
                known: false,
            }
        );
    }

    #[test]
    fn current_emitted_metric_inventory_is_covered() {
        let known_keys = [
            (MetricDomain::Actor, "ability.cha.mod"),
            (MetricDomain::Actor, "ability.con.mod"),
            (MetricDomain::Actor, "ability.dex.mod"),
            (MetricDomain::Actor, "ability.int.mod"),
            (MetricDomain::Actor, "ability.str.mod"),
            (MetricDomain::Actor, "ability.wis.mod"),
            (MetricDomain::Actor, "ac.value"),
            (MetricDomain::Actor, "hardness.value"),
            (MetricDomain::Actor, "hp.bt"),
            (MetricDomain::Actor, "hp.max"),
            (MetricDomain::Actor, "hp.value"),
            (MetricDomain::Actor, "perception.mod"),
            (MetricDomain::Actor, "save.best"),
            (MetricDomain::Actor, "save.fort.mod"),
            (MetricDomain::Actor, "save.ref.mod"),
            (MetricDomain::Actor, "save.will.mod"),
            (MetricDomain::Actor, "save.worst"),
            (MetricDomain::Actor, "skill.arcana.mod"),
            (MetricDomain::Actor, "skill.arcana.proficient"),
            (MetricDomain::Actor, "skill.arcana.rank"),
            (MetricDomain::Actor, "speed.fly.value"),
            (MetricDomain::Actor, "sense.scent.range"),
            (MetricDomain::Actor, "stealth.dc"),
            (MetricDomain::Actor, "stealth.mod"),
            (MetricDomain::Actor, "disable.dc.min"),
            (MetricDomain::Actor, "disable.dc.max"),
            (MetricDomain::Actor, "disable.thievery.dc.min"),
            (MetricDomain::Actor, "disable.thievery.dc.max"),
            (MetricDomain::Actor, "disable.thievery.rank.min"),
            (MetricDomain::Item, "armor.ac_bonus"),
            (MetricDomain::Item, "armor.check_penalty"),
            (MetricDomain::Item, "armor.dex_cap"),
            (MetricDomain::Item, "armor.speed_penalty"),
            (MetricDomain::Item, "armor.strength"),
            (MetricDomain::Item, "shield.ac_bonus"),
            (MetricDomain::Item, "shield.bt"),
            (MetricDomain::Item, "shield.hardness"),
            (MetricDomain::Item, "shield.hp"),
            (MetricDomain::Item, "weapon.damage_dice"),
            (MetricDomain::Item, "weapon.damage_die_faces"),
            (MetricDomain::Item, "weapon.range_increment"),
            (MetricDomain::Item, "weapon.reload"),
        ];

        for (domain, key) in known_keys {
            assert!(
                is_known_key(domain, key),
                "{domain:?}/{key} should be known"
            );
        }
    }

    #[test]
    fn pattern_definitions_do_not_overlap_on_current_inventory() {
        let current_dynamic_keys = [
            "ability.cha.mod",
            "ability.con.mod",
            "ability.dex.mod",
            "ability.int.mod",
            "ability.str.mod",
            "ability.wis.mod",
            "save.fort.mod",
            "save.ref.mod",
            "save.will.mod",
            "skill.arcana.mod",
            "skill.arcana.proficient",
            "skill.arcana.rank",
            "speed.fly.value",
            "speed.spider_climb.value",
            "sense.scent.range",
            "sense.infrared_vision.range",
            "disable.thievery.dc.min",
            "disable.thievery.dc.max",
            "disable.thievery.rank.min",
        ];

        for key in current_dynamic_keys {
            let mut matches_by_domain: BTreeMap<MetricDomain, usize> = BTreeMap::new();
            for definition in DEFINITIONS {
                let MetricDefinition::Pattern(pattern) = definition else {
                    continue;
                };
                if match_pattern(pattern.pattern, key).is_some() {
                    *matches_by_domain.entry(pattern.domain).or_default() += 1;
                }
            }

            for (domain, count) in matches_by_domain {
                assert_eq!(
                    count, 1,
                    "{domain:?}/{key} should not match multiple patterns"
                );
            }
        }
    }
}
