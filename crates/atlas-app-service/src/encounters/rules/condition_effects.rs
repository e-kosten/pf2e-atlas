//! Evaluate generated condition declarations against the application's typed domains.
use super::super::conditions::ConditionRule;
use super::super::stat_block::{Ability, StatisticSave, StatisticTarget};
use super::modifiers::ModifierType;
use atlas_foundry_model::generated::condition_definitions::{
    ConditionModifierType, ConditionPredicate, ConditionValue,
};

pub(super) fn value(
    expression: ConditionValue,
    magnitude: Option<i64>,
    level: Option<i64>,
) -> Option<i64> {
    match expression {
        ConditionValue::Fixed(value) => Some(value),
        ConditionValue::Magnitude(coefficient) => {
            coefficient.checked_mul(magnitude.filter(|v| *v >= 0)?)
        }
        ConditionValue::LevelMagnitude {
            coefficient,
            minimum_level,
        } => {
            let scaled = coefficient.checked_mul(magnitude.filter(|v| *v >= 0)?)?;
            if scaled == 0 {
                Some(0)
            } else {
                scaled.checked_mul(level?.max(minimum_level))
            }
        }
    }
}

pub(super) enum Target<'a> {
    Statistic(&'a StatisticTarget),
    Attack,
    Damage { ability_known: bool },
    Speed,
}
pub(super) struct Context<'a> {
    pub target: Target<'a>,
    pub ability: Option<Ability>,
    pub initiative: bool,
    /// None means an item's auditory trait is unknown; a generic check has no item.
    pub auditory: Option<bool>,
}
impl<'a> Context<'a> {
    pub fn statistic(target: &'a StatisticTarget, ability: Option<Ability>) -> Self {
        Self {
            target: Target::Statistic(target),
            ability,
            initiative: false,
            auditory: Some(false),
        }
    }
    fn selector(&self, selector: &str) -> Option<bool> {
        let statistic = match self.target {
            Target::Statistic(target) => Some(target),
            _ => None,
        };
        let check = matches!(self.target, Target::Attack)
            || statistic.is_some_and(StatisticTarget::check_or_dc);
        let ability_based = check || matches!(self.target, Target::Damage { .. });
        let ability = match selector {
            "str-based" | "str-damage" => Some(Ability::Strength),
            "dex-based" => Some(Ability::Dexterity),
            "con-based" => Some(Ability::Constitution),
            "int-based" => Some(Ability::Intelligence),
            "wis-based" => Some(Ability::Wisdom),
            "cha-based" => Some(Ability::Charisma),
            _ => None,
        };
        if let Some(ability) = ability {
            if !ability_based
                || (selector == "str-damage" && !matches!(self.target, Target::Damage { .. }))
            {
                return Some(false);
            }
            if matches!(
                self.target,
                Target::Damage {
                    ability_known: false
                }
            ) {
                return None;
            }
            return self.ability.map(|actual| actual == ability).or_else(|| {
                matches!(
                    self.target,
                    Target::Damage {
                        ability_known: true
                    }
                )
                .then_some(false)
            });
        }
        Some(match selector {
            "all" => check,
            "all-speeds" => matches!(self.target, Target::Speed),
            "attack-roll" => matches!(self.target, Target::Attack),
            "ac" => statistic == Some(&StatisticTarget::ArmorClass),
            "hp" => statistic == Some(&StatisticTarget::HitPointMaximum),
            "perception" => statistic == Some(&StatisticTarget::Perception),
            "reflex" => statistic == Some(&StatisticTarget::Save(StatisticSave::Reflex)),
            "skill-check" => statistic.is_some_and(StatisticTarget::skill),
            "saving-throw" => statistic.is_some_and(StatisticTarget::save),
            _ => false,
        })
    }
    pub fn predicate(&self, predicate: ConditionPredicate) -> Option<bool> {
        match predicate {
            ConditionPredicate::Atom(atom) => match atom {
                "check:type:skill" => Some(matches!(
                    self.target,
                    Target::Statistic(StatisticTarget::Skill(_))
                )),
                "item:trait:auditory" => self.auditory,
                "check:statistic:base:perception" => Some(matches!(
                    self.target,
                    Target::Statistic(StatisticTarget::Perception)
                )),
                "check:statistic:initiative" => Some(self.initiative),
                _ => None,
            },
            ConditionPredicate::Not(term) => self.predicate(*term).map(|v| !v),
            ConditionPredicate::All(terms) => {
                let mut known = true;
                for term in terms {
                    match self.predicate(*term) {
                        Some(false) => return Some(false),
                        None => known = false,
                        _ => {}
                    }
                }
                known.then_some(true)
            }
            ConditionPredicate::Any(terms) => {
                let mut known = true;
                for term in terms {
                    match self.predicate(*term) {
                        Some(true) => return Some(true),
                        None => known = false,
                        _ => {}
                    }
                }
                known.then_some(false)
            }
        }
    }
}

pub(super) fn modifiers<'a>(
    rule: ConditionRule,
    magnitude: Option<i64>,
    level: Option<i64>,
    context: &'a Context<'a>,
) -> impl Iterator<Item = (ModifierType, Option<i64>)> + 'a {
    rule.definition()
        .modifiers
        .iter()
        .filter_map(move |effect| {
            let mut selected = Some(false);
            for selector in effect.selectors {
                match context.selector(selector) {
                    Some(true) => {
                        selected = Some(true);
                        break;
                    }
                    None => selected = None,
                    Some(false) => {}
                }
            }
            let known = match (selected, context.predicate(effect.predicate)) {
                (Some(false), _) | (_, Some(false)) => return None,
                (Some(true), Some(true)) => Some(true),
                _ => None,
            };
            let modifier_type = match effect.modifier_type {
                ConditionModifierType::Status => ModifierType::Status,
                ConditionModifierType::Circumstance => ModifierType::Circumstance,
                ConditionModifierType::Untyped => ModifierType::Adjustment,
            };
            Some((
                modifier_type,
                known.and_then(|_| value(effect.value, magnitude, level)),
            ))
        })
}

pub(in crate::encounters) fn hp_loss(
    rule: ConditionRule,
    magnitude: i64,
    level: Option<i64>,
) -> Option<i64> {
    value(rule.definition().hp_loss?.value, Some(magnitude), level)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_strength_selectors_apply_once_to_each_matching_target() {
        let context = Context {
            target: Target::Damage {
                ability_known: true,
            },
            ability: Some(Ability::Strength),
            initiative: false,
            auditory: Some(false),
        };
        assert_eq!(
            modifiers(ConditionRule::Enfeebled, Some(2), None, &context).collect::<Vec<_>>(),
            vec![(ModifierType::Status, Some(-2))]
        );
        let context = Context {
            target: Target::Attack,
            ..context
        };
        assert_eq!(
            modifiers(ConditionRule::Enfeebled, Some(2), None, &context).collect::<Vec<_>>(),
            vec![(ModifierType::Status, Some(-2))]
        );
        assert_eq!(
            modifiers(
                ConditionRule::Frightened,
                Some(2),
                None,
                &Context {
                    target: Target::Damage {
                        ability_known: true
                    },
                    ..context
                }
            )
            .count(),
            0
        );
    }
    #[test]
    fn deafened_predicate_distinguishes_general_perception_initiative_and_auditory_skills() {
        let mut context = Context::statistic(&StatisticTarget::Perception, Some(Ability::Wisdom));
        assert_eq!(
            modifiers(ConditionRule::Deafened, None, None, &context).count(),
            0
        );
        context.initiative = true;
        assert_eq!(
            modifiers(ConditionRule::Deafened, None, None, &context).collect::<Vec<_>>(),
            vec![(ModifierType::Status, Some(-2))]
        );
        let skill = StatisticTarget::Skill("performance".into());
        let context = Context {
            target: Target::Statistic(&skill),
            auditory: None,
            ..context
        };
        assert_eq!(
            modifiers(ConditionRule::Deafened, None, None, &context).collect::<Vec<_>>(),
            vec![(ModifierType::Status, None)]
        );
    }
    #[test]
    fn generated_drained_expressions_preserve_unknowns_level_floor_and_checked_arithmetic() {
        assert_eq!(hp_loss(ConditionRule::Drained, 2, Some(-1)), Some(2));
        assert_eq!(hp_loss(ConditionRule::Drained, 2, Some(5)), Some(10));
        assert_eq!(hp_loss(ConditionRule::Drained, 2, None), None);
        assert_eq!(hp_loss(ConditionRule::Drained, i64::MAX, Some(2)), None);
    }
}
