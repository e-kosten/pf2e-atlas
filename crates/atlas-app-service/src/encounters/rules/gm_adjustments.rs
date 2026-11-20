//! Shared selection and stacking for GM modifiers on encounter statistics.
use super::super::stat_block::*;
use super::projection::{apply, apply_modifier};
use atlas_app_model::{AdjustmentModifierType, AdjustmentTarget, EncounterAdjustmentTargetView};
use atlas_local_state::EncounterAdjustment;
use serde_json::Number;

pub(super) fn apply_all(block: &mut StatBlockView, adjustments: &[EncounterAdjustment]) {
    for adjustment in adjustments {
        let modifier = Modifier {
            source: adjustment.adjustment_key.clone(),
            label: adjustment.name.clone(),
            modifier_type: match adjustment.modifier_type {
                AdjustmentModifierType::Stacks => ModifierType::Adjustment,
                AdjustmentModifierType::Item => ModifierType::Item,
                AdjustmentModifierType::Status => ModifierType::Status,
                AdjustmentModifierType::Circumstance => ModifierType::Circumstance,
            },
            value: adjustment.amount,
        };
        if let Some(target) = statistic(&adjustment.target) {
            for stat in block.values.iter_mut().filter(|stat| stat.target == target) {
                apply(stat, modifier.clone());
            }
        } else if let AdjustmentTarget::Speed { mode } = &adjustment.target {
            for speed in block
                .speeds
                .iter_mut()
                .filter(|speed| &speed.movement_type == mode)
            {
                apply_modifier(
                    &mut speed.adjusted_value_feet,
                    &mut speed.modifiers,
                    &mut speed.suppressed_modifiers,
                    modifier.clone(),
                );
            }
        } else {
            for activity in &mut block.activities {
                if matches_activity(&adjustment.target, activity.kind) {
                    if adjustment.target == AdjustmentTarget::SpellDcs {
                        if let Some(dc) = &mut activity.difficulty_class {
                            apply(dc, modifier.clone());
                        }
                    } else {
                        for roll in &mut activity.rolls {
                            apply_modifier(
                                &mut roll.adjusted_value,
                                &mut roll.modifiers,
                                &mut roll.suppressed_modifiers,
                                modifier.clone(),
                            );
                        }
                    }
                }
            }
        }
    }
    // Apply the bounds after stacking so intermediate clamping is order-independent.
    for speed in &mut block.speeds {
        if !speed.modifiers.is_empty()
            && speed.adjusted_value_feet.as_f64().is_some_and(|v| v < 0.0)
        {
            speed.adjusted_value_feet = Number::from(0);
            speed.notes.push(atlas_app_model::RuntimeEffectNoteView {
                source: "speed-minimum".into(),
                label: "Minimum speed".into(),
                reason: "Speed cannot be negative.".into(),
            });
        }
    }
}

fn statistic(target: &AdjustmentTarget) -> Option<StatisticTarget> {
    Some(match target {
        AdjustmentTarget::ArmorClass => StatisticTarget::ArmorClass,
        AdjustmentTarget::Fortitude => StatisticTarget::Save(StatisticSave::Fortitude),
        AdjustmentTarget::Reflex => StatisticTarget::Save(StatisticSave::Reflex),
        AdjustmentTarget::Will => StatisticTarget::Save(StatisticSave::Will),
        AdjustmentTarget::Perception => StatisticTarget::Perception,
        AdjustmentTarget::Skill { key } => StatisticTarget::Skill(key.clone()),
        _ => return None,
    })
}

fn matches_activity(target: &AdjustmentTarget, kind: MechanicActivityKindView) -> bool {
    match target {
        AdjustmentTarget::StrikeAttacks => kind == MechanicActivityKindView::Strike,
        AdjustmentTarget::SpellAttacks | AdjustmentTarget::SpellDcs => {
            matches!(
                kind,
                MechanicActivityKindView::CastingEntry | MechanicActivityKindView::Spell
            )
        }
        _ => false,
    }
}

pub(in crate::encounters) fn target_label(target: &AdjustmentTarget) -> String {
    match target {
        AdjustmentTarget::ArmorClass => "AC".into(),
        AdjustmentTarget::Fortitude => "Fortitude".into(),
        AdjustmentTarget::Reflex => "Reflex".into(),
        AdjustmentTarget::Will => "Will".into(),
        AdjustmentTarget::Perception => "Perception".into(),
        AdjustmentTarget::Skill { key } => capitalize(key),
        AdjustmentTarget::Speed { mode } => format!("{} speed", capitalize(mode)),
        AdjustmentTarget::StrikeAttacks => "Strike attacks".into(),
        AdjustmentTarget::SpellAttacks => "Spell attacks".into(),
        AdjustmentTarget::SpellDcs => "Spell DCs".into(),
    }
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |c| {
        format!("{}{}", c.to_ascii_uppercase(), chars.as_str())
    })
}

pub(in crate::encounters) fn targets(block: &StatBlockView) -> Vec<EncounterAdjustmentTargetView> {
    use AdjustmentTarget::*;
    let candidates = [ArmorClass, Fortitude, Reflex, Will, Perception]
        .into_iter()
        .chain(block.values.iter().filter_map(|value| match &value.target {
            StatisticTarget::Skill(key) => Some(Skill { key: key.clone() }),
            _ => None,
        }))
        .chain(block.speeds.iter().map(|speed| Speed {
            mode: speed.movement_type.clone(),
        }))
        .chain([StrikeAttacks, SpellAttacks, SpellDcs]);
    let mut targets: Vec<EncounterAdjustmentTargetView> = Vec::new();
    for target in candidates {
        if target.is_supported()
            && observe(block, block, &target, "")
                .iter()
                .any(|row| row.value.is_some())
            && !targets.iter().any(|entry| entry.target == target)
        {
            targets.push(EncounterAdjustmentTargetView {
                label: target_label(&target),
                target,
            });
        }
    }
    targets
}

pub(in crate::encounters) struct Observation {
    pub key: String,
    pub label: String,
    pub value: Option<Number>,
    pub suppressed: bool,
}

pub(in crate::encounters) fn observe(
    block: &StatBlockView,
    authored: &StatBlockView,
    target: &AdjustmentTarget,
    adjustment_key: &str,
) -> Vec<Observation> {
    let suppressed = |modifiers: &[Modifier]| {
        modifiers
            .iter()
            .any(|modifier| modifier.source == adjustment_key)
    };
    if let Some(statistic) = statistic(target) {
        let Some(stat) = block.values.iter().find(|stat| stat.target == statistic) else {
            return vec![];
        };
        return vec![Observation {
            key: statistic.key().into_owned(),
            label: target_label(target),
            value: block.effective_stat(&statistic).cloned(),
            suppressed: suppressed(&stat.suppressed_modifiers)
                || block
                    .replacements
                    .iter()
                    .any(|entry| entry.target == statistic),
        }];
    }
    if let AdjustmentTarget::Speed { mode } = target {
        return block
            .speeds
            .iter()
            .filter(|speed| &speed.movement_type == mode)
            .map(|speed| Observation {
                key: mode.clone(),
                label: target_label(target),
                value: Some(speed.adjusted_value_feet.clone()),
                suppressed: suppressed(&speed.suppressed_modifiers),
            })
            .collect();
    }
    authored
        .activities
        .iter()
        .filter(|activity| {
            activity.kind != MechanicActivityKindView::Spell
                && matches_activity(target, activity.kind)
        })
        .map(|source| {
            let effective = block.activities.iter().find(|activity| {
                activity.navigation == source.navigation
                    && activity.calculation_unavailable.is_none()
            });
            let value = effective.and_then(|activity| {
                if *target == AdjustmentTarget::SpellDcs {
                    let dc = activity.difficulty_class.as_ref()?;
                    Some((&dc.adjusted_value, &dc.suppressed_modifiers))
                } else {
                    let roll = activity.rolls.first()?;
                    Some((&roll.adjusted_value, &roll.suppressed_modifiers))
                }
            });
            Observation {
                key: source.activity_id.clone(),
                label: source.label.clone(),
                value: value.map(|(value, _)| value.clone()),
                suppressed: value.is_some_and(|(_, modifiers)| suppressed(modifiers)),
            }
        })
        .collect()
}
