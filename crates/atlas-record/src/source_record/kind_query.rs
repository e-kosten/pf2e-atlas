//! Character creation and other kind-specific authored query facts.
use super::{
    FieldAvailability, ItemSourceView, SourceContentOutcome, SourceContentRole,
    SourceContentStatus, SourceFieldView, SourceNodeView,
};
use crate::source_content::{ContentInteractionKind, ContentVisibilityRule};
use atlas_domain::RecordKey;
use atlas_foundry_model::{
    ActorSourcePF2e, RuleSource, SourceContext, SourcePresence, SourceValue, generated::*,
};
use serde::{Deserialize, de::value::StrDeserializer};
use serde_json::Number;

type Attribute = PhysicalSystemSourceApexAttribute;

const ATTRIBUTES: [Attribute; 6] = [
    Attribute::Str,
    Attribute::Dex,
    Attribute::Con,
    Attribute::Int,
    Attribute::Wis,
    Attribute::Cha,
];

pub struct AttributeSlotFacts<'a> {
    pub restricted: SourceFieldView<'a, Vec<Attribute>>,
    pub has_free: SourceFieldView<'a, bool>,
}

fn slot_facts<'a>(
    slots: SourceFieldView<'a, &'a [(String, SourceFromSchemaAttributeSlotSchema)]>,
) -> AttributeSlotFacts<'a> {
    let SourceFieldView::Value(slots) = slots else {
        return AttributeSlotFacts {
            restricted: slots.map(|_| Vec::new()),
            has_free: slots.map(|_| false),
        };
    };
    let mut restricted = Vec::new();
    let mut has_free = false;
    let mut unavailable = SourceFieldView::Value(());
    for (_, slot) in slots {
        match SourceFieldView::from(&slot.value) {
            SourceFieldView::Value(values) if ATTRIBUTES.iter().all(|a| values.contains(a)) => {
                has_free = true;
            }
            SourceFieldView::Value(values) => {
                for &attribute in values {
                    if !restricted.contains(&attribute) {
                        restricted.push(attribute);
                    }
                }
            }
            other if unavailable.value().is_some() => unavailable = other.map(|_| ()),
            _ => {}
        }
    }
    AttributeSlotFacts {
        restricted: unavailable.map(|()| restricted),
        has_free: if has_free {
            SourceFieldView::Value(true)
        } else {
            unavailable.map(|()| false)
        },
    }
}

fn skill_slug(value: &str) -> Option<SkillSlug> {
    SkillSlug::deserialize(StrDeserializer::<serde::de::value::Error>::new(value)).ok()
}

fn active(ignored: &SourcePresence<bool>) -> bool {
    matches!(
        ignored,
        SourcePresence::Value(false) | SourcePresence::Missing
    )
}

fn skill_binding(path: &str) -> Option<(&str, bool)> {
    let selection = path
        .strip_prefix("system.skills.{item|flags.")?
        .strip_suffix("}.rank")?;
    let binding = selection
        .strip_prefix("system.rulesSelections.")
        .or_else(|| selection.strip_prefix("pf2e.rulesSelections."))?;
    let (flag, nested) = binding
        .strip_suffix(".skill")
        .map_or((binding, false), |flag| (flag, true));
    (!flag.is_empty() && flag.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'))
        .then_some((flag, nested))
}

fn choice_skill(value: &StringOrNumberOrObject, nested: bool) -> Option<SkillSlug> {
    match (value, nested) {
        (StringOrNumberOrObject::String(skill), false) => skill_slug(skill),
        (StringOrNumberOrObject::Object(SourceValue::Object(object)), true) => {
            let mut values = object.fields().iter().filter(|(key, _)| key == "skill");
            match (values.next(), values.next()) {
                (Some((_, SourceValue::String(skill))), None) => skill_slug(skill),
                _ => None,
            }
        }
        _ => None,
    }
}

fn add_skill_choices(skills: &mut Vec<SkillSlug>, rules: &[RuleElementSource], key: &RecordKey) {
    let interpreted = rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| {
            if !matches!(
                rule.key.as_value().map(String::as_str),
                Some("ChoiceSet" | "ActiveEffectLike")
            ) {
                return None;
            }
            rule.interpret(SourceContext::new(
                key.to_string(),
                "checked source",
                format!("$.system.rules[{index}]"),
            ))
            .ok()
        })
        .collect::<Vec<_>>();
    for rule in &interpreted {
        let RuleSource::ActiveEffectLike(upgrade) = rule else {
            continue;
        };
        if !active(&upgrade.ignored)
            || upgrade.mode.as_value() != Some(&SourceFromSchemaAELikeSchemaMode::Upgrade)
            || !matches!(upgrade.value.as_value(), Some(JSONValue::Number(rank)) if rank.as_u64() == Some(1))
        {
            continue;
        }
        let Some((flag, nested)) = upgrade.path.as_value().and_then(|path| skill_binding(path))
        else {
            continue;
        };
        let mut candidates = interpreted.iter().filter_map(|rule| match rule {
            RuleSource::ChoiceSet(choice)
                if active(&choice.ignored)
                    && active(&choice.actor_flag)
                    && choice.flag.as_value().is_some_and(|value| value == flag) =>
            {
                Some(choice)
            }
            _ => None,
        });
        let (Some(choice), None) = (candidates.next(), candidates.next()) else {
            continue;
        };
        let Some(SourceFromSchemaChoiceSetSchemaChoices::Array(options)) =
            choice.choices.as_value()
        else {
            continue;
        };
        let options = options
            .iter()
            .map(|option| choice_skill(option.value.as_value()?, nested))
            .collect::<Option<Vec<_>>>();
        if let Some(options) = options {
            skills.extend(options);
        }
    }
}

#[derive(Clone, Copy)]
pub enum ClassProficiency {
    Simple,
    Martial,
    Advanced,
    Unarmed,
    Unarmored,
    Light,
    Medium,
    Heavy,
}

pub const PROFICIENCY_RANKS: [&str; 5] = ["untrained", "trained", "expert", "master", "legendary"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellRange {
    pub touch: bool,
    pub feet: Option<i64>,
}

fn parse_spell_range(value: &str) -> Option<SpellRange> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized == "touch" {
        return Some(SpellRange {
            touch: true,
            feet: None,
        });
    }
    let (distance, touch) = normalized
        .strip_prefix("touch or ")
        .map_or((normalized.as_str(), false), |distance| (distance, true));
    let (amount, unit) = distance.split_once(' ')?;
    let mut groups = amount.split(',');
    let first = groups.next()?;
    if first.is_empty()
        || !first.bytes().all(|c| c.is_ascii_digit())
        || amount.contains(',')
            && (first.len() > 3
                || groups
                    .any(|group| group.len() != 3 || !group.bytes().all(|c| c.is_ascii_digit())))
    {
        return None;
    }
    let amount = amount.replace(',', "").parse::<i64>().ok()?;
    let feet = match unit {
        "foot" | "feet" => amount,
        "mile" | "miles" => amount.checked_mul(5280)?,
        _ => return None,
    };
    (feet > 0).then_some(SpellRange {
        touch,
        feet: Some(feet),
    })
}

macro_rules! item_fact {
    ($name:ident, $family:ident, $return:ty, $system:ident => $value:expr) => {
        pub fn $name(self) -> SourceFieldView<'a, $return> {
            let Self::$family(source) = self else {
                return SourceFieldView::NotApplicable;
            };
            SourceFieldView::from(&source.system).and_then(|$system| $value)
        }
    };
}

impl<'a> ItemSourceView<'a> {
    pub fn background_boosts(self) -> AttributeSlotFacts<'a> {
        let slots = match self {
            Self::BackgroundSource(source) => SourceFieldView::from(&source.system)
                .and_then(|system| (&system.boosts).into())
                .map(|boosts| boosts.indexed_fields.entries.as_slice()),
            _ => SourceFieldView::NotApplicable,
        };
        slot_facts(slots)
    }

    pub fn background_skill_options(self, key: &RecordKey) -> SourceFieldView<'a, Vec<SkillSlug>> {
        let Self::BackgroundSource(source) = self else {
            return SourceFieldView::NotApplicable;
        };
        SourceFieldView::from(&source.system).and_then(|system| {
            SourceFieldView::from(&system.trained_skills)
                .and_then(|skills| (&skills.value).into())
                .map(|skills| {
                    let mut options = skills.clone();
                    if let Some(rules) = system.rules.as_value() {
                        add_skill_choices(&mut options, rules, key);
                    }
                    options
                })
        })
    }

    item_fact!(ancestry_vision, AncestrySource, &'a AncestryVision, system => (&system.vision).into());
    item_fact!(ancestry_land_speed, AncestrySource, &'a Number, system => (&system.speed).into());
    item_fact!(ancestry_starting_hp, AncestrySource, &'a Number, system => (&system.hp).into());

    pub fn ancestry_default_attributes(self, flaws: bool) -> SourceFieldView<'a, Vec<Attribute>> {
        let Self::AncestrySource(source) = self else {
            return SourceFieldView::NotApplicable;
        };
        let slots = SourceFieldView::from(&source.system)
            .and_then(|system| {
                if flaws {
                    (&system.flaws).into()
                } else {
                    (&system.boosts).into()
                }
            })
            .map(|slots| slots.entries.as_slice());
        slot_facts(slots).restricted
    }

    item_fact!(class_key_attributes, ClassSource, &'a Vec<Attribute>, system =>
        SourceFieldView::from(&system.key_ability).and_then(|key| (&key.value).into())
    );

    pub fn class_starting_proficiency(
        self,
        proficiency: ClassProficiency,
    ) -> SourceFieldView<'a, &'static str> {
        let Self::ClassSource(source) = self else {
            return SourceFieldView::NotApplicable;
        };
        SourceFieldView::from(&source.system).and_then(|system| {
            use ClassProficiency::*;
            let rank = match proficiency {
                Simple | Martial | Advanced | Unarmed => SourceFieldView::from(&system.attacks)
                    .and_then(|ranks| {
                        match proficiency {
                            Simple => &ranks.simple,
                            Martial => &ranks.martial,
                            Advanced => &ranks.advanced,
                            _ => &ranks.unarmed,
                        }
                        .into()
                    }),
                Unarmored | Light | Medium | Heavy => SourceFieldView::from(&system.defenses)
                    .and_then(|ranks| {
                        match proficiency {
                            Unarmored => &ranks.unarmored,
                            Light => &ranks.light,
                            Medium => &ranks.medium,
                            _ => &ranks.heavy,
                        }
                        .into()
                    }),
            };
            rank.and_then(|rank| {
                match rank
                    .as_u64()
                    .and_then(|rank| usize::try_from(rank).ok())
                    .and_then(|rank| PROFICIENCY_RANKS.get(rank))
                {
                    Some(&rank) => SourceFieldView::Value(rank),
                    None => SourceFieldView::ProjectionInvalid {
                        source_path: "/system",
                        reason: "Starting proficiency must be an integer from 0 through 4",
                    },
                }
            })
        })
    }

    item_fact!(deity_category, DeitySource, &'a DeityCategory, system => (&system.category).into());
    item_fact!(deity_skills, DeitySource, &'a Vec<SkillSlug>, system => (&system.skill).into());
    item_fact!(deity_weapons, DeitySource, &'a DeitySystemSourceWeapons, system => (&system.weapons).into());

    item_fact!(deity_sanctification_requirement, DeitySource, &'static str, system =>
        match SourceFieldView::from(&system.sanctification) {
            SourceFieldView::Null => SourceFieldView::Value("none"),
            value => value.and_then(|value| SourceFieldView::from(&value.modal)).map(|modal| match modal {
                DeitySanctificationModal::Can => "optional",
                DeitySanctificationModal::Must => "required",
            }),
        }
    );
    item_fact!(deity_sanctification_options, DeitySource, &'a [Sanctification], system =>
        match SourceFieldView::from(&system.sanctification) {
            SourceFieldView::Null => SourceFieldView::Value(&[] as &[Sanctification]),
            value => value.and_then(|value| SourceFieldView::from(&value.what)).map(Vec::as_slice),
        }
    );
    item_fact!(spell_cast_range, SpellSource, SpellRange, system =>
        SourceFieldView::from(&system.range).and_then(|range| SourceFieldView::from(&range.value))
            .and_then(|value| match parse_spell_range(value) {
                Some(range) => SourceFieldView::Value(range),
                None if value.trim().is_empty() => SourceFieldView::Missing,
                None => SourceFieldView::ProjectionInvalid {
                    source_path: "/system/range/value",
                    reason: "Cast range is not an explicit distance or touch range",
                },
            })
    );
}

pub fn hazard_disable_skills<'a>(
    source: SourceNodeView<'a>,
    content: &[SourceContentOutcome],
) -> SourceFieldView<'a, Vec<SkillSlug>> {
    let SourceNodeView::Actor(ActorSourcePF2e::HazardSource(hazard)) = source else {
        return SourceFieldView::NotApplicable;
    };
    SourceFieldView::from(&hazard.system)
        .and_then(|system| SourceFieldView::from(&system.details))
        .and_then(|details| SourceFieldView::from(&details.disable))
        .and_then(|authored| {
            let outcome = content.iter().find(|outcome| {
                outcome.role == SourceContentRole::Disable
                    && outcome.locator().owners.is_empty()
                    && outcome.locator().field == "/system/details/disable"
                    && outcome.visibility == ContentVisibilityRule::All
                    && outcome.visibility_availability == FieldAvailability::Value
            });
            match outcome.map(|outcome| &outcome.status) {
                Some(SourceContentStatus::Prepared(prepared)) => SourceFieldView::Value(
                    prepared
                        .interactions
                        .iter()
                        .filter_map(|interaction| match &interaction.kind {
                            ContentInteractionKind::Check {
                                statistic: Some(statistic),
                                ..
                            } => Some(statistic.as_str()),
                            _ => None,
                        })
                        .flat_map(|statistic| statistic.split(','))
                        .filter_map(|skill| skill_slug(skill.trim()))
                        .collect(),
                ),
                Some(
                    SourceContentStatus::PreparationFailed { .. }
                    | SourceContentStatus::UnsupportedFormat { .. },
                ) => SourceFieldView::ProjectionInvalid {
                    source_path: "/system/details/disable",
                    reason: "Disable check preparation failed",
                },
                Some(SourceContentStatus::FormatUnavailable { availability, .. }) => {
                    match availability {
                        FieldAvailability::Null => SourceFieldView::Null,
                        FieldAvailability::Invalid { .. } => SourceFieldView::ProjectionInvalid {
                            source_path: "/system/details/disable",
                            reason: "Disable check format is unavailable",
                        },
                        FieldAvailability::NotApplicable => SourceFieldView::NotApplicable,
                        _ => SourceFieldView::Missing,
                    }
                }
                None if authored.trim().is_empty() => SourceFieldView::Value(Vec::new()),
                None => SourceFieldView::Missing,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_explicit_casting_ranges() {
        for (text, touch, feet) in [
            ("Touch", true, None),
            (" 30 feet ", false, Some(30)),
            ("1 mile", false, Some(5280)),
            ("1,000 feet", false, Some(1000)),
            ("touch or 30 feet", true, Some(30)),
        ] {
            assert_eq!(
                parse_spell_range(text),
                Some(SpellRange { touch, feet }),
                "{text}"
            );
        }
        for text in [
            "",
            "self",
            "varies",
            "30",
            "30 feet (burst only)",
            "30-foot emanation",
            "1,2 feet",
            "0 feet",
        ] {
            assert_eq!(parse_spell_range(text), None, "{text}");
        }
    }
}
