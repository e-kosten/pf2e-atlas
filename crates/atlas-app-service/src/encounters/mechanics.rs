//! Runtime overlays on authored typed statistics; unknown inputs stay unknown.
use super::conditions::{ConditionRule, condition_rule_for_key};
pub(crate) use super::stat_block::*;
use atlas_app_model::*;
use atlas_local_state::{EncounterParticipant, ParticipantVariant};
use atlas_record::source_record::{
    SourceBackedRecord, SourceFieldView, SourceNpcAdjustment, SourceQueryView,
};
use serde_json::Number;

#[cfg(test)]
thread_local! {
    static BASELINES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub(super) fn baseline_count(reset: bool) -> usize {
    BASELINES.with(|count| if reset { count.replace(0) } else { count.get() })
}

pub(super) fn authored_variant(record: &SourceBackedRecord) -> Option<ParticipantVariant> {
    SourceQueryView::new(record.source(), record.key().pack().as_str(), "")
        .actor()
        .authored_adjustment()
        .value()
        .map(|v| match v {
            SourceNpcAdjustment::Normal => ParticipantVariant::Normal,
            SourceNpcAdjustment::Elite => ParticipantVariant::Elite,
            SourceNpcAdjustment::Weak => ParticipantVariant::Weak,
        })
}
pub(super) fn initial_variant(
    record: &SourceBackedRecord,
) -> (
    Option<ParticipantVariant>,
    atlas_local_state::ParticipantVariantOrigin,
) {
    use atlas_local_state::ParticipantVariantOrigin;
    if let Some(variant) = authored_variant(record) {
        return (Some(variant), ParticipantVariantOrigin::InheritedKnown);
    }
    if SourceQueryView::new(record.source(), record.key().pack().as_str(), "")
        .actor()
        .npc_adjustment_is_unset()
    {
        (
            Some(ParticipantVariant::Normal),
            ParticipantVariantOrigin::DefaultUnadjusted,
        )
    } else {
        (None, ParticipantVariantOrigin::InheritedUnknown)
    }
}
pub(super) fn integer(value: &Number) -> Option<i64> {
    if let Some(i) = value.as_i64() {
        return Some(i);
    }
    if value.is_u64() {
        return None;
    }
    let n = value.as_f64()?;
    (n.is_finite() && n.fract() == 0.0 && n >= i64::MIN as f64 && n < 9223372036854775808.0)
        .then_some(n as i64)
}
pub(super) fn hp_delta(variant: ParticipantVariant, level: i64) -> i64 {
    match variant {
        ParticipantVariant::Normal => 0,
        ParticipantVariant::Elite => {
            if level >= 20 {
                30
            } else if level >= 5 {
                20
            } else if level >= 2 {
                15
            } else {
                10
            }
        }
        ParticipantVariant::Weak => {
            -if level >= 21 {
                30
            } else if level >= 6 {
                20
            } else if level >= 3 {
                15
            } else if level == 1 || level == 2 {
                10
            } else {
                0
            }
        }
    }
}
pub(super) fn adjusted_npc_level(level: i64, variant: ParticipantVariant) -> i64 {
    let base = level.clamp(-1, 100);
    match variant {
        ParticipantVariant::Normal => base,
        ParticipantVariant::Elite => base + if base < 1 { 2 } else { 1 },
        ParticipantVariant::Weak => base - if base == 1 { 2 } else { 1 },
    }
}
pub(super) fn effective_max(
    record: &SourceBackedRecord,
    variant: Option<ParticipantVariant>,
) -> Option<i64> {
    let actor = SourceQueryView::new(record.source(), record.key().pack().as_str(), "").actor();
    let hp = integer(actor.hp_maximum().value()?)?;
    if matches!(actor.authored_adjustment(), SourceFieldView::NotApplicable) {
        return Some(hp);
    }
    let variant = variant?;
    if variant == ParticipantVariant::Normal {
        return Some(hp);
    }
    let level = integer(actor.level().value()?)?;
    hp.checked_add(hp_delta(variant, level))
}
pub(super) fn participant_stat_block(
    participant: &EncounterParticipant,
    baseline: &StatBlockView,
    adjustment_applicable: bool,
) -> StatBlockView {
    let mut block = baseline.clone();
    let variant = participant.participant_variant;
    let delta = match variant {
        ParticipantVariant::Normal => 0,
        ParticipantVariant::Elite => 2,
        ParticipantVariant::Weak => -2,
    };
    if adjustment_applicable
        && participant.variant_origin
            != atlas_local_state::ParticipantVariantOrigin::InheritedUnknown
    {
        if let Some(level) = block.level.as_ref().and_then(integer) {
            block.adjusted_level = Some(adjusted_npc_level(level, variant).into());
        }
        for value in &mut block.values {
            let amount = if value.target == "hp.max" {
                block
                    .level
                    .as_ref()
                    .and_then(integer)
                    .map(|l| hp_delta(variant, l))
            } else if value.target == "ac"
                || value.target.starts_with("save.")
                || value.target == "perception"
                || value.target.starts_with("skill.")
            {
                Some(delta)
            } else {
                None
            };
            if let Some(amount) = amount.filter(|v| *v != 0) {
                apply(
                    value,
                    StatModifierView {
                        source: "npc-adjustment".into(),
                        label: format!("{variant:?}"),
                        modifier_type: StatModifierTypeView::Adjustment,
                        value: amount,
                    },
                );
            }
        }
        for activity in &mut block.activities {
            for roll in &mut activity.rolls {
                if delta != 0 {
                    let m = StatModifierView {
                        source: "npc-adjustment".into(),
                        label: format!("{variant:?}"),
                        modifier_type: StatModifierTypeView::Adjustment,
                        value: delta,
                    };
                    if let Some(n) = add(&roll.adjusted_value, delta) {
                        roll.adjusted_value = n;
                        roll.modifiers.push(m);
                    }
                }
            }
            if delta != 0 && !activity.damage.is_empty() {
                activity.notes.push(RuntimeEffectNoteView{source:"npc-adjustment".into(),label:format!("{variant:?}"),reason:"Damage adjustment requires authored usage context; the original formulas remain available.".into()});
            }
        }
    } else if adjustment_applicable {
        block.unapplied_effects.push(UnappliedEffectView{source:"npc-adjustment".into(),label:"Source adjustment unavailable".into(),reason:"Missing or invalid adjustment does not establish Normal; choose an explicit variant to apply an adjustment.".into()});
    }
    let mut movement_restriction = None;
    for c in &participant.conditions {
        let Some(rule) = condition_rule_for_key(c.condition_key.as_deref()) else {
            block.unapplied_effects.push(UnappliedEffectView {
                source: c.name.clone(),
                label: c.name.clone(),
                reason: "This condition is tracked without an automated rule.".into(),
            });
            continue;
        };
        let amount = c.value.unwrap_or(1).max(0);
        let mut applied = false;
        for v in &mut block.values {
            let matched = match rule {
                ConditionRule::Frightened | ConditionRule::Sickened => {
                    v.target == "ac"
                        || v.target.starts_with("save.")
                        || v.target == "perception"
                        || v.target == "hazard.stealth"
                        || v.target.starts_with("skill.")
                }
                ConditionRule::OffGuard
                | ConditionRule::Grabbed
                | ConditionRule::Restrained
                | ConditionRule::Prone => v.target == "ac",
                ConditionRule::Clumsy | ConditionRule::Encumbered => matches!(
                    v.target.as_str(),
                    "ac" | "save.reflex"
                        | "skill.acrobatics"
                        | "skill.stealth"
                        | "skill.thievery"
                        | "ability.dex"
                ),
                ConditionRule::Enfeebled => {
                    matches!(v.target.as_str(), "ability.str" | "skill.athletics")
                }
                ConditionRule::Stupefied => matches!(
                    v.target.as_str(),
                    "save.will"
                        | "perception"
                        | "ability.int"
                        | "ability.wis"
                        | "ability.cha"
                        | "skill.arcana"
                        | "skill.crafting"
                        | "skill.occultism"
                        | "skill.society"
                        | "skill.nature"
                        | "skill.religion"
                        | "skill.medicine"
                        | "skill.survival"
                        | "skill.deception"
                        | "skill.diplomacy"
                        | "skill.intimidation"
                        | "skill.performance"
                ),
                _ => false,
            };
            if matched {
                let value = if matches!(
                    rule,
                    ConditionRule::OffGuard
                        | ConditionRule::Grabbed
                        | ConditionRule::Restrained
                        | ConditionRule::Prone
                ) {
                    -2
                } else if rule == ConditionRule::Encumbered {
                    -1
                } else {
                    -amount
                };
                apply(
                    v,
                    StatModifierView {
                        source: c.condition_key.clone().unwrap_or_else(|| c.name.clone()),
                        label: c.name.clone(),
                        modifier_type: if matches!(
                            rule,
                            ConditionRule::OffGuard
                                | ConditionRule::Grabbed
                                | ConditionRule::Restrained
                                | ConditionRule::Prone
                        ) {
                            StatModifierTypeView::Circumstance
                        } else {
                            StatModifierTypeView::Status
                        },
                        value,
                    },
                );
                applied = true;
            }
        }
        if matches!(rule, ConditionRule::Frightened | ConditionRule::Sickened) {
            for a in &mut block.activities {
                for roll in &mut a.rolls {
                    apply_modifier(
                        &mut roll.adjusted_value,
                        &mut roll.modifiers,
                        &mut roll.suppressed_modifiers,
                        StatModifierView {
                            source: c.name.clone(),
                            label: c.name.clone(),
                            modifier_type: StatModifierTypeView::Status,
                            value: -amount,
                        },
                    );
                    applied = true;
                }
            }
        }
        if matches!(
            rule,
            ConditionRule::Immobilized
                | ConditionRule::Grabbed
                | ConditionRule::Restrained
                | ConditionRule::Encumbered
        ) {
            if movement_restriction.is_none() || rule != ConditionRule::Encumbered {
                movement_restriction = Some((rule, c));
            }
            applied |= !block.speeds.is_empty();
        }
        if !applied
            || matches!(
                rule,
                ConditionRule::Enfeebled
                    | ConditionRule::Clumsy
                    | ConditionRule::Stupefied
                    | ConditionRule::Prone
                    | ConditionRule::Quickened
                    | ConditionRule::Stunned
            )
        {
            block.unapplied_effects.push(UnappliedEffectView{source:c.name.clone(),label:c.name.clone(),reason:"Attack ability, spellcasting, restricted actions or situational effects require additional authored/runtime context; unsupported effects remain unapplied.".into()});
        }
    }
    if let Some((rule, condition)) = movement_restriction {
        for speed in &mut block.speeds {
            let baseline = &speed.base_value_feet;
            let restricted = if rule == ConditionRule::Encumbered {
                if let Some(n) = baseline.as_i64() {
                    Some(Number::from(if n > 0 { (n - 10).max(5) } else { n }))
                } else if baseline.is_u64() {
                    None
                } else {
                    baseline.as_f64().and_then(|n| {
                        Number::from_f64(if n > 0.0 { (n - 10.0).max(5.0) } else { n })
                    })
                }
            } else {
                Some(Number::from(0))
            };
            if let Some(restricted) = restricted {
                speed.adjusted_value_feet = restricted;
                speed.notes.push(RuntimeEffectNoteView {
                    source: condition.name.clone(),
                    label: condition.name.clone(),
                    reason: "Movement restriction applied once to the known authored speed.".into(),
                });
            }
        }
    }
    block.action_budget = Some(action_budget(participant));
    block
}
pub(super) fn arithmetic_baseline(presentation: &RecordPresentationView) -> Option<StatBlockView> {
    let a = match &presentation.body {
        RecordBodyView::Creature(a) => a.as_ref(),
        RecordBodyView::Hazard(h) => &h.actor,
        _ => return None,
    };
    #[cfg(test)]
    BASELINES.with(|count| count.set(count.get() + 1));
    let mut values = vec![];
    let mut push = |target: String, label: String, f: &NumberFactView| {
        if let Some(n) = &f.value {
            values.push(StatValueView {
                target,
                label,
                base_value: n.clone(),
                adjusted_value: n.clone(),
                modifiers: vec![],
                suppressed_modifiers: vec![],
            });
        }
    };
    push("ac".into(), "AC".into(), &a.armor_class);
    push("hp.max".into(), "Maximum HP".into(), &a.maximum_hp);
    push("perception".into(), "Perception".into(), &a.perception);
    for (key, f) in [
        ("fortitude", &a.saves.fortitude),
        ("reflex", &a.saves.reflex),
        ("will", &a.saves.will),
    ] {
        push(format!("save.{key}"), key.into(), f);
    }
    if let RecordBodyView::Hazard(h) = &presentation.body {
        push("hazard.stealth".into(), "Stealth".into(), &h.stealth);
        push("hazard.hardness".into(), "Hardness".into(), &h.hardness);
    }
    for v in a.abilities.value.iter().flatten() {
        push(format!("ability.{}", v.key), v.label.clone(), &v.modifier);
    }
    for v in a.skills.value.iter().flatten() {
        push(format!("skill.{}", v.key), v.label.clone(), &v.modifier);
    }
    for (index, activity) in a.activities.value.iter().flatten().enumerate() {
        push(
            format!("skill.lore.{index}"),
            activity.title.clone(),
            &activity.lore_modifier,
        );
    }
    let mut notes = vec![];
    for v in a.skills.value.iter().flatten() {
        if let Some(note) = &v.note.value {
            notes.push(UnappliedEffectView {
                source: format!("skill.{}", v.key),
                label: v.label.clone(),
                reason: note.clone(),
            });
        }
        if let Some(c) = &v.conditional.value {
            for e in c {
                if let (Some(label), Some(n)) = (&e.label.value, &e.modifier.value) {
                    notes.push(UnappliedEffectView{source:format!("skill.{}",v.key),label:label.clone(),reason:format!("Conditional modifier {n}; apply only when its authored predicate matches.")});
                }
            }
        }
    }
    let speed = |kind: &str, n: &Number| MovementSpeedView {
        movement_type: kind.into(),
        label: kind.into(),
        base_value_feet: n.clone(),
        adjusted_value_feet: n.clone(),
        adjustments: vec![],
        suppressed_adjustments: vec![],
        notes: vec![],
    };
    let mut speeds = vec![];
    if let Some(n) = &a.land_speed.value {
        speeds.push(speed("land", n));
    }
    for v in a.movement.value.iter().flatten() {
        if let (Some(kind), Some(n)) = (&v.movement_type.value, &v.feet.value) {
            speeds.push(speed(kind, n));
        }
    }
    for activity in a.activities.value.iter().flatten() {
        for n in &activity.notes {
            notes.push(UnappliedEffectView {
                source: n.source.clone(),
                label: n.label.clone(),
                reason: n.reason.clone(),
            });
        }
    }
    let activities = a
        .activities
        .value
        .iter()
        .flatten()
        .filter(|a| {
            matches!(
                a.kind,
                ActorActivityKindView::Strike
                    | ActorActivityKindView::Spell
                    | ActorActivityKindView::Ability
                    | ActorActivityKindView::CastingEntry
            )
        })
        .cloned()
        .map(|a| MechanicActivityView {
            activity_id: a
                .navigation
                .owners
                .first()
                .map(|o| format!("{:?}", o.identity))
                .unwrap_or_default(),
            label: a.title,
            kind: match a.kind {
                ActorActivityKindView::Strike => MechanicActivityKindView::Strike,
                ActorActivityKindView::Spell => MechanicActivityKindView::Spell,
                _ => MechanicActivityKindView::Other,
            },
            navigation: a.navigation,
            rolls: a
                .attack
                .value
                .filter(|_| a.kind == ActorActivityKindView::Strike)
                .map(|n| ActivityRollView {
                    roll_id: "attack".into(),
                    label: "Attack".into(),
                    base_value: n.clone(),
                    adjusted_value: n,
                    surface: ActivityRollSurfaceView::AttackRoll,
                    modifiers: vec![],
                    suppressed_modifiers: vec![],
                })
                .into_iter()
                .collect(),
            damage: a
                .damage
                .value
                .unwrap_or_default()
                .into_iter()
                .filter_map(|d| {
                    Some(DamageExpressionView {
                        damage_id: d.id,
                        label: None,
                        formula: d.formula.value?,
                        adjusted_formula: None,
                        damage_type: d.damage_type.value,
                        effect_kind: DamageEffectKindView::Unknown,
                        modifiers: vec![],
                    })
                })
                .collect(),
            notes: a.notes,
        })
        .collect();
    Some(StatBlockView {
        record_key: presentation.identity.record_key.clone(),
        title: presentation.identity.title.clone(),
        level: a.level.value.clone(),
        adjusted_level: a.level.value.clone(),
        values,
        speeds,
        action_budget: None,
        activities,
        unapplied_effects: notes,
    })
}
fn add(n: &Number, delta: i64) -> Option<Number> {
    if let Some(i) = n.as_i64() {
        return i.checked_add(delta).map(Number::from);
    }
    if n.is_u64() {
        return None;
    }
    Number::from_f64(n.as_f64()? + delta as f64)
}
fn apply(v: &mut StatValueView, m: StatModifierView) {
    apply_modifier(
        &mut v.adjusted_value,
        &mut v.modifiers,
        &mut v.suppressed_modifiers,
        m,
    );
}
fn apply_modifier(
    adjusted_value: &mut Number,
    modifiers: &mut Vec<StatModifierView>,
    suppressed_modifiers: &mut Vec<StatModifierView>,
    m: StatModifierView,
) {
    let same = modifiers.iter().position(|old| {
        old.modifier_type == m.modifier_type && m.modifier_type != StatModifierTypeView::Adjustment
    });
    let delta = if let Some(i) = same {
        if modifiers[i].value <= m.value {
            suppressed_modifiers.push(m);
            return;
        }
        m.value - modifiers[i].value
    } else {
        m.value
    };
    if let Some(n) = add(adjusted_value, delta) {
        if let Some(i) = same {
            suppressed_modifiers.push(modifiers.remove(i));
        }
        *adjusted_value = n;
        modifiers.push(m);
    }
}
pub(super) fn participant_runtime_block(p: &EncounterParticipant) -> StatBlockView {
    StatBlockView {
        record_key: p.participant_key.clone(),
        title: p.display_name.clone(),
        level: None,
        adjusted_level: None,
        values: vec![],
        speeds: vec![],
        action_budget: Some(action_budget(p)),
        activities: vec![],
        unapplied_effects: vec![],
    }
}
fn action_budget(p: &EncounterParticipant) -> ActionBudgetView {
    let mut loss = 0;
    let mut quick = false;
    let mut stunned = false;
    for c in &p.conditions {
        match condition_rule_for_key(c.condition_key.as_deref()) {
            Some(ConditionRule::Slowed) => loss = loss.max(c.value.unwrap_or(1).max(0)),
            Some(ConditionRule::Stunned) => {
                loss = loss.max(c.value.unwrap_or(1).max(0));
                stunned = true;
            }
            Some(ConditionRule::Quickened) => quick = true,
            _ => {}
        }
    }
    let actions = (3 - loss).max(0) + i64::from(quick);
    let count = |label: &str, base: i64, value: i64| RuntimeCountView {
        label: label.into(),
        base_value: base,
        adjusted_value: value,
        segments: vec![],
        adjustments: vec![],
        suppressed_adjustments: vec![],
    };
    ActionBudgetView {
        actions: count("Actions", 3, actions),
        reactions: count("Reactions", 1, i64::from(!stunned)),
        can_act: RuntimeCapabilityView {
            available: actions > 0,
            source: None,
            reason: None,
        },
        can_react: RuntimeCapabilityView {
            available: !stunned,
            source: None,
            reason: None,
        },
        notes: if quick {
            vec![RuntimeEffectNoteView {
                source: "quickened".into(),
                label: "Quickened action".into(),
                reason: "One added action is restricted by the effect that granted it.".into(),
            }]
        } else {
            vec![]
        },
    }
}
