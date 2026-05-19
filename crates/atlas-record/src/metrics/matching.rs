use atlas_domain::MetricDomain;

use crate::MetricRow;

use super::model::{
    MetricCapture, MetricDefinition, MetricDefinitionMatch, MetricDisplayLabel, MetricKeyPattern,
    MetricKeySegment, MetricLabelTemplate, MetricVariableVocabulary,
};
use super::{actor, item};

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

pub(crate) fn match_pattern(pattern: MetricKeyPattern, key: &str) -> Option<Vec<MetricCapture>> {
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
