//! Runtime overlays on authored typed statistics; unknown inputs stay unknown.
use super::conditions::{ConditionRule, condition_rule_for_key};
use atlas_app_model::*;
use atlas_local_state::{EncounterParticipant, ParticipantVariant};
use atlas_record::source_record::{
    ActorSave, SourceBackedRecord, SourceFieldView, SourceNodeView, SourceNpcAdjustment,
    SourceQueryView,
};
use serde_json::Number;

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
    record: &SourceBackedRecord,
    fingerprint: &str,
) -> Option<StatBlockView> {
    let mut block = record_stat_block(record, fingerprint)?;
    let adjustment_applicable = !matches!(
        SourceQueryView::new(record.source(), record.key().pack().as_str(), "")
            .actor()
            .authored_adjustment(),
        SourceFieldView::NotApplicable
    );
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
                    let old = roll
                        .modifiers
                        .iter()
                        .filter(|m| m.modifier_type == StatModifierTypeView::Status)
                        .map(|m| m.value)
                        .min()
                        .unwrap_or(0);
                    let penalty = -amount;
                    if penalty < old
                        && let Some(n) = add(&roll.adjusted_value, penalty - old)
                    {
                        roll.adjusted_value = n;
                        roll.modifiers.push(StatModifierView {
                            source: c.name.clone(),
                            label: c.name.clone(),
                            modifier_type: StatModifierTypeView::Status,
                            value: penalty,
                        });
                        applied = true;
                    }
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
    Some(block)
}
pub(crate) fn record_stat_block(
    record: &SourceBackedRecord,
    fingerprint: &str,
) -> Option<StatBlockView> {
    let query = SourceQueryView::new(record.source(), record.key().pack().as_str(), "");
    if !matches!(query.source, SourceNodeView::Actor(_)) {
        return None;
    }
    let actor = query.actor();
    let mut values = vec![];
    let mut push = |target: &str, label: &str, v: SourceFieldView<'_, &Number>| {
        if let Some(n) = v.value() {
            values.push(StatValueView {
                target: target.into(),
                label: label.into(),
                base_value: n.clone(),
                adjusted_value: n.clone(),
                modifiers: vec![],
                suppressed_modifiers: vec![],
            });
        }
    };
    push("ac", "AC", actor.armor_class());
    push("hp.max", "Maximum HP", actor.hp_maximum());
    push("perception", "Perception", actor.perception());
    push("hazard.hardness", "Hardness", actor.hazard_hardness());
    push("hazard.stealth", "Stealth", actor.hazard_stealth());
    for (save, name) in [
        (ActorSave::Fortitude, "fortitude"),
        (ActorSave::Reflex, "reflex"),
        (ActorSave::Will, "will"),
    ] {
        push(&format!("save.{name}"), name, actor.save(save));
    }
    actor.visit_abilities(|key, v| push(&format!("ability.{key}"), key, v));
    let mut notes = vec![];
    actor.visit_skills(|key,v|{
        if let Some(skill)=v.value(){
            push(&format!("skill.{key}"),key,(&skill.base).into());
            if let Some(note)=SourceFieldView::from(&skill.note).value(){notes.push(UnappliedEffectView{source:format!("skill.{key}"),label:key.into(),reason:note.clone()});}

            if let Some(special)=SourceFieldView::from(&skill.special).value(){for entry in special{if let (Some(label),Some(base))=(SourceFieldView::from(&entry.label).value(),SourceFieldView::from(&entry.base).value()){notes.push(UnappliedEffectView{source:format!("skill.{key}"),label:label.clone(),reason:format!("Conditional modifier {base}; apply only when its authored predicate matches.")});}}}
        }
    });
    let mut speeds = vec![];
    if let Some(n) = actor.land_speed().value() {
        speeds.push(speed("land", n));
    }
    if let Some(other) = actor.speeds().value() {
        for s in other.iter() {
            if let (Some(t), Some(n)) = (s.speed_type().value(), s.value().value()) {
                let kind = serde_json::to_value(t)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default();
                speeds.push(speed(&kind, n));
            }
        }
    }
    let mut activities = vec![];
    record.visit_immediate_actor_items(|index, owner, item| {
        let name = SourceNodeView::Item(item)
            .name()
            .value()
            .cloned()
            .unwrap_or_else(|| item.family().into());
        if let Some(n) = item.lore_modifier().value() {
            values.push(StatValueView {
                target: format!("skill.lore.{index}"),
                label: name.clone(),
                base_value: n.clone(),
                adjusted_value: n.clone(),
                modifiers: vec![],
                suppressed_modifiers: vec![],
            });
            if let Some(variants) = item.lore_variants().value() {
                for (_, v) in &variants.entries {
                    if let Some(label) = SourceFieldView::from(&v.label).value() {
                        notes.push(UnappliedEffectView {
                            source: name.clone(),
                            label: label.clone(),
                            reason: SourceFieldView::from(&v.options)
                                .value()
                                .cloned()
                                .unwrap_or_else(|| "Lore variant context is unavailable.".into()),
                        });
                    }
                }
            }
        }
        let kind = match item.family() {
            "melee" => MechanicActivityKindView::Strike,
            "spell" => MechanicActivityKindView::Spell,
            "action" | "effect" | "spellcastingEntry" => MechanicActivityKindView::Other,
            _ => return,
        };
        let mut rolls = vec![];
        if let Some(n) = item.melee_attack().value() {
            rolls.push(ActivityRollView {
                roll_id: "attack".into(),
                label: "Attack".into(),
                base_value: n.clone(),
                adjusted_value: n.clone(),
                surface: ActivityRollSurfaceView::AttackRoll,
                modifiers: vec![],
                suppressed_modifiers: vec![],
            });
        }
        let mut damage = vec![];
        if let Some(entries) = item.melee_damage().value() {
            for (id, v) in &entries.entries {
                if let Some(formula) = SourceFieldView::from(&v.damage).value() {
                    damage.push(damage_view(
                        id,
                        formula,
                        SourceFieldView::from(&v.damage_type)
                            .value()
                            .and_then(enum_text),
                    ));
                }
            }
        }
        if let Some(entries) = item.spell_damage().value() {
            for (id, v) in &entries.entries {
                if let Some(formula) = SourceFieldView::from(&v.formula).value() {
                    damage.push(damage_view(
                        id,
                        formula,
                        SourceFieldView::from(&v.r#type).value().and_then(enum_text),
                    ));
                }
            }
        }
        let owners = vec![owner.clone()];
        activities.push(MechanicActivityView {
            activity_id: format!("owned-item-{index}"),
            label: name,
            kind,
            navigation: RecordNavigationView {
                record_key: record.key().to_string(),
                source_fingerprint: crate::projection::navigation_fingerprint(&owners, fingerprint),
                owners,
                field: None,
                passage: None,
            },
            rolls,
            damage,
            notes: vec![],
        });
    });
    Some(StatBlockView {
        record_key: record.key().to_string(),
        title: query
            .source
            .name()
            .value()
            .cloned()
            .unwrap_or_else(|| record.key().to_string()),
        level: actor.level().value().cloned(),
        adjusted_level: actor.level().value().cloned(),
        values,
        speeds,
        action_budget: None,
        activities,
        unapplied_effects: notes,
    })
}
fn enum_text(v: &impl serde::Serialize) -> Option<String> {
    serde_json::to_value(v).ok()?.as_str().map(str::to_owned)
}
fn speed(kind: &str, n: &Number) -> MovementSpeedView {
    MovementSpeedView {
        movement_type: kind.into(),
        label: kind.into(),
        base_value_feet: n.clone(),
        adjusted_value_feet: n.clone(),
        adjustments: vec![],
        suppressed_adjustments: vec![],
        notes: vec![],
    }
}
fn damage_view(id: &str, formula: &str, kind: Option<String>) -> DamageExpressionView {
    DamageExpressionView {
        damage_id: id.into(),
        label: None,
        formula: formula.into(),
        adjusted_formula: None,
        damage_type: kind,
        effect_kind: DamageEffectKindView::Unknown,
        modifiers: vec![],
    }
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
    let same = v.modifiers.iter().position(|old| {
        old.modifier_type == m.modifier_type && m.modifier_type != StatModifierTypeView::Adjustment
    });
    let delta = if let Some(i) = same {
        if v.modifiers[i].value <= m.value {
            v.suppressed_modifiers.push(m);
            return;
        }
        let old = v.modifiers.remove(i);
        let delta = m.value - old.value;
        v.suppressed_modifiers.push(old);
        delta
    } else {
        m.value
    };
    if let Some(n) = add(&v.adjusted_value, delta) {
        v.adjusted_value = n;
        v.modifiers.push(m);
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
