use std::collections::BTreeMap;

use atlas_app_model::{
    ActivityRollSurfaceView, ActivityRollView, DamageEffectKindView, DamageExpressionView,
    EncounterParticipantVariantView, MechanicActivityKindView, MechanicActivityModeView,
    MechanicActivityUsageView, MechanicActivityView, StatBlockView, StatModifierTypeView,
    StatModifierView, StatValueView, UnappliedEffectView,
};
use atlas_local_state::{EncounterParticipant, EncounterParticipantCondition, ParticipantVariant};
use atlas_record::{
    AbilityKind, ActivityRoll, ActivityRollAbility, ActivityRollSurface, DamageEffectKind,
    DamageExpression, MechanicActivity, MechanicActivityKind, MechanicActivityMode,
    MechanicActivityUsage, MechanicScalar, MechanicSurface, MechanicTarget, MechanicValue,
    MechanicsView, build_mechanics_view,
};

use super::conditions::{ConditionRule, condition_rule_for_key};
use super::projection::participant_variant_view;

#[derive(Debug, Clone)]
struct CandidateModifier {
    target: MechanicTarget,
    source: String,
    label: String,
    modifier_type: StatModifierTypeView,
    value: i64,
}

#[derive(Debug, Clone)]
struct RollModifier {
    source: String,
    label: String,
    modifier_type: StatModifierTypeView,
    value: i64,
}

pub(super) fn participant_stat_block(
    participant: &EncounterParticipant,
    record: &atlas_record::AtlasRecord,
) -> Option<StatBlockView> {
    let mechanics = build_mechanics_view(record)?;
    Some(apply_participant_effects(participant, mechanics))
}

pub(super) fn variant_hp_adjustment_delta(
    old_variant: ParticipantVariant,
    new_variant: ParticipantVariant,
    level: Option<i64>,
) -> i64 {
    variant_hp_delta(new_variant, level).unwrap_or(0)
        - variant_hp_delta(old_variant, level).unwrap_or(0)
}

fn apply_participant_effects(
    participant: &EncounterParticipant,
    mechanics: MechanicsView,
) -> StatBlockView {
    let mut modifiers = variant_modifiers(participant.participant_variant, &mechanics);
    let mut unapplied_effects = variant_unapplied_effects(participant.participant_variant);
    for condition in &participant.conditions {
        let Some(condition_rule) = condition_rule_for_key(condition.condition_key.as_deref())
        else {
            continue;
        };
        modifiers.extend(condition_modifiers(condition, condition_rule, &mechanics));
        unapplied_effects.extend(condition_unapplied_effects(condition, condition_rule));
    }

    let mut by_target = BTreeMap::<MechanicTarget, Vec<CandidateModifier>>::new();
    for modifier in modifiers {
        by_target
            .entry(modifier.target.clone())
            .or_default()
            .push(modifier);
    }

    StatBlockView {
        record_key: mechanics.record_key.to_string(),
        title: mechanics.title,
        level: mechanics.level,
        adjusted_level: adjusted_level(mechanics.level, participant.participant_variant),
        values: mechanics
            .values
            .into_iter()
            .map(|value| {
                let modifiers = by_target.remove(&value.target).unwrap_or_default();
                stat_value_view(value, modifiers)
            })
            .collect(),
        activities: mechanics
            .activities
            .into_iter()
            .map(|activity| activity_view(activity, participant))
            .collect(),
        unapplied_effects,
    }
}

fn activity_view(
    activity: MechanicActivity,
    participant: &EncounterParticipant,
) -> MechanicActivityView {
    let kind = activity.kind;
    let usage = activity.usage;
    MechanicActivityView {
        activity_id: activity.activity_id,
        label: activity.label,
        kind: activity_kind_view(kind),
        usage: activity_usage_view(usage),
        rolls: activity
            .rolls
            .into_iter()
            .map(|roll| activity_roll_view(roll, kind, participant))
            .collect(),
        damage: activity
            .damage
            .into_iter()
            .map(|damage| damage_view(damage, kind, usage, participant))
            .collect(),
        modes: activity
            .modes
            .into_iter()
            .map(|mode| activity_mode_view(mode, kind, usage, participant))
            .collect(),
    }
}

fn activity_mode_view(
    mode: MechanicActivityMode,
    activity_kind: MechanicActivityKind,
    activity_usage: MechanicActivityUsage,
    participant: &EncounterParticipant,
) -> MechanicActivityModeView {
    MechanicActivityModeView {
        mode_id: mode.mode_id,
        label: mode.label,
        target: mode.target,
        range: mode.range,
        time: mode.time,
        damage: mode
            .damage
            .into_iter()
            .map(|damage| damage_view(damage, activity_kind, activity_usage, participant))
            .collect(),
    }
}

fn activity_roll_view(
    roll: ActivityRoll,
    activity_kind: MechanicActivityKind,
    participant: &EncounterParticipant,
) -> ActivityRollView {
    let mut modifiers = Vec::new();
    if let Some(modifier) = variant_roll_modifier(participant.participant_variant) {
        modifiers.push(modifier);
    }
    for condition in &participant.conditions {
        let Some(rule) = condition_rule_for_key(condition.condition_key.as_deref()) else {
            continue;
        };
        modifiers.extend(condition_roll_modifiers(
            condition,
            rule,
            activity_kind,
            &roll,
        ));
    }
    let (applied, suppressed) = stack_roll_modifiers(modifiers);
    let adjusted_value = applied
        .iter()
        .fold(roll.base_value, |total, modifier| total + modifier.value);
    ActivityRollView {
        roll_id: roll.roll_id,
        label: roll.label,
        base_value: roll.base_value,
        adjusted_value,
        surface: activity_roll_surface_view(roll.surface),
        modifiers: applied.into_iter().map(roll_modifier_view).collect(),
        suppressed_modifiers: suppressed.into_iter().map(roll_modifier_view).collect(),
    }
}

fn damage_view(
    damage: DamageExpression,
    activity_kind: MechanicActivityKind,
    activity_usage: MechanicActivityUsage,
    participant: &EncounterParticipant,
) -> DamageExpressionView {
    let mut modifiers = variant_damage_modifier(
        participant.participant_variant,
        activity_usage,
        damage.effect_kind,
    )
    .into_iter()
    .collect::<Vec<_>>();
    for condition in &participant.conditions {
        let Some(rule) = condition_rule_for_key(condition.condition_key.as_deref()) else {
            continue;
        };
        modifiers.extend(condition_damage_modifiers(
            condition,
            rule,
            activity_kind,
            &damage,
        ));
    }
    let modifier_total = modifiers.iter().map(|modifier| modifier.value).sum::<i64>();
    let adjusted_formula =
        (modifier_total != 0).then(|| adjusted_formula(&damage.formula, modifier_total));
    DamageExpressionView {
        damage_id: damage.damage_id,
        label: damage.label,
        formula: damage.formula,
        adjusted_formula,
        damage_type: damage.damage_type,
        effect_kind: damage_effect_kind_view(damage.effect_kind),
        modifiers,
    }
}

fn condition_damage_modifiers(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
    activity_kind: MechanicActivityKind,
    damage: &DamageExpression,
) -> Vec<StatModifierView> {
    if rule != ConditionRule::Enfeebled
        || activity_kind != MechanicActivityKind::Strike
        || damage.effect_kind != DamageEffectKind::Damage
        || damage_ability(damage) != Some(AbilityKind::Strength)
    {
        return Vec::new();
    }
    let source = condition_source(condition);
    vec![StatModifierView {
        source: source.clone(),
        label: source,
        modifier_type: StatModifierTypeView::Status,
        value: -condition_value(condition),
    }]
}

fn damage_ability(damage: &DamageExpression) -> Option<AbilityKind> {
    match damage.ability? {
        ActivityRollAbility::Strength => Some(AbilityKind::Strength),
        ActivityRollAbility::Dexterity => Some(AbilityKind::Dexterity),
        ActivityRollAbility::Constitution => Some(AbilityKind::Constitution),
        ActivityRollAbility::Intelligence => Some(AbilityKind::Intelligence),
        ActivityRollAbility::Wisdom => Some(AbilityKind::Wisdom),
        ActivityRollAbility::Charisma => Some(AbilityKind::Charisma),
    }
}

fn variant_damage_modifier(
    variant: ParticipantVariant,
    usage: MechanicActivityUsage,
    effect_kind: DamageEffectKind,
) -> Option<StatModifierView> {
    if effect_kind != DamageEffectKind::Damage {
        return None;
    }
    let direction = match variant {
        ParticipantVariant::Normal => return None,
        ParticipantVariant::Elite => 1,
        ParticipantVariant::Weak => -1,
    };
    let magnitude = match usage {
        MechanicActivityUsage::Unlimited => 2,
        MechanicActivityUsage::Limited => 4,
        MechanicActivityUsage::Ambiguous => return None,
    };
    let source = variant_source(variant).to_string();
    Some(StatModifierView {
        source: source.clone(),
        label: format!("{source} damage adjustment"),
        modifier_type: StatModifierTypeView::Adjustment,
        value: direction * magnitude,
    })
}

fn variant_roll_modifier(variant: ParticipantVariant) -> Option<RollModifier> {
    let value = variant_stat_delta(variant)?;
    let source = variant_source(variant).to_string();
    Some(RollModifier {
        source: source.clone(),
        label: format!("{source} adjustment"),
        modifier_type: StatModifierTypeView::Adjustment,
        value,
    })
}

fn condition_roll_modifiers(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
    activity_kind: MechanicActivityKind,
    roll: &ActivityRoll,
) -> Vec<RollModifier> {
    let amount = condition_value(condition);
    let source = condition_source(condition);
    let status_penalty = |value: i64| RollModifier {
        source: source.clone(),
        label: source.clone(),
        modifier_type: StatModifierTypeView::Status,
        value: -value,
    };
    match rule {
        ConditionRule::Frightened | ConditionRule::Sickened => vec![status_penalty(amount)],
        ConditionRule::Clumsy if roll_ability(roll) == Some(AbilityKind::Dexterity) => {
            vec![status_penalty(amount)]
        }
        ConditionRule::Enfeebled if roll_ability(roll) == Some(AbilityKind::Strength) => {
            vec![status_penalty(amount)]
        }
        ConditionRule::Stupefied if activity_kind == MechanicActivityKind::Spell => {
            vec![status_penalty(amount)]
        }
        ConditionRule::OffGuard
        | ConditionRule::Clumsy
        | ConditionRule::Enfeebled
        | ConditionRule::Stupefied => Vec::new(),
    }
}

fn roll_ability(roll: &ActivityRoll) -> Option<AbilityKind> {
    match roll.ability? {
        ActivityRollAbility::Strength => Some(AbilityKind::Strength),
        ActivityRollAbility::Dexterity => Some(AbilityKind::Dexterity),
        ActivityRollAbility::Constitution => Some(AbilityKind::Constitution),
        ActivityRollAbility::Intelligence => Some(AbilityKind::Intelligence),
        ActivityRollAbility::Wisdom => Some(AbilityKind::Wisdom),
        ActivityRollAbility::Charisma => Some(AbilityKind::Charisma),
    }
}

fn stack_roll_modifiers(modifiers: Vec<RollModifier>) -> (Vec<RollModifier>, Vec<RollModifier>) {
    let mut applied = Vec::new();
    let mut suppressed = Vec::new();
    let mut typed = BTreeMap::<(StatModifierTypeView, i8), RollModifier>::new();
    for modifier in modifiers {
        if modifier.modifier_type == StatModifierTypeView::Adjustment {
            applied.push(modifier);
            continue;
        }
        let sign = modifier.value.signum() as i8;
        let key = (modifier.modifier_type, sign);
        if let Some(existing) = typed.remove(&key) {
            let replace = if sign < 0 {
                modifier.value < existing.value
            } else {
                modifier.value > existing.value
            };
            if replace {
                suppressed.push(existing);
                typed.insert(key, modifier);
            } else {
                suppressed.push(modifier);
                typed.insert(key, existing);
            }
        } else {
            typed.insert(key, modifier);
        }
    }
    applied.extend(typed.into_values());
    applied.sort_by(|left, right| left.label.cmp(&right.label));
    suppressed.sort_by(|left, right| left.label.cmp(&right.label));
    (applied, suppressed)
}

fn roll_modifier_view(modifier: RollModifier) -> StatModifierView {
    StatModifierView {
        source: modifier.source,
        label: modifier.label,
        modifier_type: modifier.modifier_type,
        value: modifier.value,
    }
}

fn activity_kind_view(kind: MechanicActivityKind) -> MechanicActivityKindView {
    match kind {
        MechanicActivityKind::Strike => MechanicActivityKindView::Strike,
        MechanicActivityKind::Spell => MechanicActivityKindView::Spell,
        MechanicActivityKind::Other => MechanicActivityKindView::Other,
    }
}

fn activity_roll_surface_view(surface: ActivityRollSurface) -> ActivityRollSurfaceView {
    match surface {
        ActivityRollSurface::AttackRoll => ActivityRollSurfaceView::AttackRoll,
        ActivityRollSurface::Dc => ActivityRollSurfaceView::Dc,
    }
}

fn activity_usage_view(usage: MechanicActivityUsage) -> MechanicActivityUsageView {
    match usage {
        MechanicActivityUsage::Unlimited => MechanicActivityUsageView::Unlimited,
        MechanicActivityUsage::Limited => MechanicActivityUsageView::Limited,
        MechanicActivityUsage::Ambiguous => MechanicActivityUsageView::Ambiguous,
    }
}

fn damage_effect_kind_view(effect_kind: DamageEffectKind) -> DamageEffectKindView {
    match effect_kind {
        DamageEffectKind::Damage => DamageEffectKindView::Damage,
        DamageEffectKind::Healing => DamageEffectKindView::Healing,
        DamageEffectKind::DamageOrHealing => DamageEffectKindView::DamageOrHealing,
        DamageEffectKind::Unknown => DamageEffectKindView::Unknown,
    }
}

fn adjusted_formula(formula: &str, delta: i64) -> String {
    let trimmed = formula.trim();
    if trimmed.is_empty() || delta == 0 {
        return trimmed.to_string();
    }
    let (body, existing_constant) = split_formula_constant(trimmed);
    let new_constant = existing_constant + delta;
    if new_constant == 0 {
        body.trim().to_string()
    } else if new_constant > 0 {
        format!("{} + {}", body.trim(), new_constant)
    } else {
        format!("{} - {}", body.trim(), new_constant.abs())
    }
}

fn split_formula_constant(formula: &str) -> (&str, i64) {
    let trimmed = formula.trim_end();
    let bytes = trimmed.as_bytes();
    let mut index = bytes.len();
    while index > 0 && bytes[index - 1].is_ascii_digit() {
        index -= 1;
    }
    if index == bytes.len() {
        return (trimmed, 0);
    }
    let number = trimmed[index..].parse::<i64>().unwrap_or(0);
    let prefix = trimmed[..index].trim_end();
    let Some(operator) = prefix.as_bytes().last().copied() else {
        return (trimmed, 0);
    };
    if operator != b'+' && operator != b'-' {
        return (trimmed, 0);
    }
    let body = prefix[..prefix.len() - 1].trim_end();
    if body.is_empty() {
        return (trimmed, 0);
    }
    if operator == b'+' {
        (body, number)
    } else {
        (body, -number)
    }
}

fn stat_value_view(value: MechanicValue, modifiers: Vec<CandidateModifier>) -> StatValueView {
    let MechanicScalar::Number(base_value) = value.base_value;
    let (applied, suppressed) = stack_modifiers(modifiers);
    let adjusted_value = applied
        .iter()
        .fold(base_value, |total, modifier| total + modifier.value);
    StatValueView {
        target: value.target.id(),
        label: value.label,
        base_value,
        adjusted_value,
        modifiers: applied.into_iter().map(modifier_view).collect(),
        suppressed_modifiers: suppressed.into_iter().map(modifier_view).collect(),
    }
}

fn stack_modifiers(
    modifiers: Vec<CandidateModifier>,
) -> (Vec<CandidateModifier>, Vec<CandidateModifier>) {
    let mut applied = Vec::new();
    let mut suppressed = Vec::new();
    let mut typed = BTreeMap::<(StatModifierTypeView, i8), CandidateModifier>::new();
    for modifier in modifiers {
        if modifier.modifier_type == StatModifierTypeView::Adjustment {
            applied.push(modifier);
            continue;
        }
        let sign = modifier.value.signum() as i8;
        let key = (modifier.modifier_type, sign);
        if let Some(existing) = typed.remove(&key) {
            let replace = if sign < 0 {
                modifier.value < existing.value
            } else {
                modifier.value > existing.value
            };
            if replace {
                suppressed.push(existing);
                typed.insert(key, modifier);
            } else {
                suppressed.push(modifier);
                typed.insert(key, existing);
            }
        } else {
            typed.insert(key, modifier);
        }
    }
    applied.extend(typed.into_values());
    applied.sort_by(|left, right| left.label.cmp(&right.label));
    suppressed.sort_by(|left, right| left.label.cmp(&right.label));
    (applied, suppressed)
}

fn modifier_view(modifier: CandidateModifier) -> StatModifierView {
    StatModifierView {
        source: modifier.source,
        label: modifier.label,
        modifier_type: modifier.modifier_type,
        value: modifier.value,
    }
}

fn variant_modifiers(
    variant: ParticipantVariant,
    mechanics: &MechanicsView,
) -> Vec<CandidateModifier> {
    let Some(value_delta) = variant_stat_delta(variant) else {
        return Vec::new();
    };
    let source = variant_source(variant);
    let mut modifiers = mechanics
        .values
        .iter()
        .filter(|value| {
            !matches!(
                value.facets.surface,
                MechanicSurface::HitPoints | MechanicSurface::RawModifier
            )
        })
        .map(|value| CandidateModifier {
            target: value.target.clone(),
            source: source.to_string(),
            label: format!("{source} adjustment"),
            modifier_type: StatModifierTypeView::Adjustment,
            value: value_delta,
        })
        .collect::<Vec<_>>();
    if let Some(hp_delta) = variant_hp_delta(variant, mechanics.level) {
        modifiers.push(CandidateModifier {
            target: MechanicTarget::MaxHp,
            source: source.to_string(),
            label: format!("{source} HP adjustment"),
            modifier_type: StatModifierTypeView::Adjustment,
            value: hp_delta,
        });
    }
    modifiers
}

fn variant_unapplied_effects(variant: ParticipantVariant) -> Vec<UnappliedEffectView> {
    match variant {
        ParticipantVariant::Normal => Vec::new(),
        ParticipantVariant::Elite | ParticipantVariant::Weak => {
            let source = variant_source(variant).to_string();
            vec![UnappliedEffectView {
                source: source.clone(),
                label: format!("{source} ambiguous offensive damage adjustments"),
                reason: "Ambiguous, prose-only, or unsupported offensive damage remains unapplied."
                    .to_string(),
            }]
        }
    }
}

fn adjusted_level(level: Option<i64>, variant: ParticipantVariant) -> Option<i64> {
    let level = level?;
    Some(match variant {
        ParticipantVariant::Normal => level,
        ParticipantVariant::Elite if level <= 0 => level + 2,
        ParticipantVariant::Elite => level + 1,
        ParticipantVariant::Weak if level == 1 => level - 2,
        ParticipantVariant::Weak => level - 1,
    })
}

fn variant_stat_delta(variant: ParticipantVariant) -> Option<i64> {
    match variant {
        ParticipantVariant::Normal => None,
        ParticipantVariant::Elite => Some(2),
        ParticipantVariant::Weak => Some(-2),
    }
}

fn variant_hp_delta(variant: ParticipantVariant, level: Option<i64>) -> Option<i64> {
    let level = level?;
    match variant {
        ParticipantVariant::Normal => None,
        ParticipantVariant::Elite if level <= 1 => Some(10),
        ParticipantVariant::Elite if level <= 4 => Some(15),
        ParticipantVariant::Elite if level <= 19 => Some(20),
        ParticipantVariant::Elite => Some(30),
        ParticipantVariant::Weak if level <= 2 => Some(-10),
        ParticipantVariant::Weak if level <= 5 => Some(-15),
        ParticipantVariant::Weak if level <= 20 => Some(-20),
        ParticipantVariant::Weak => Some(-30),
    }
}

fn variant_source(variant: ParticipantVariant) -> &'static str {
    match participant_variant_view(variant) {
        EncounterParticipantVariantView::Normal => "Normal",
        EncounterParticipantVariantView::Elite => "Elite",
        EncounterParticipantVariantView::Weak => "Weak",
    }
}

fn condition_modifiers(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
    mechanics: &MechanicsView,
) -> Vec<CandidateModifier> {
    let amount = condition_value(condition);
    let source = condition_source(condition);
    let status_penalty = |target: MechanicTarget, label: String, value: i64| CandidateModifier {
        target,
        source: source.clone(),
        label,
        modifier_type: StatModifierTypeView::Status,
        value: -value,
    };
    match rule {
        ConditionRule::Frightened | ConditionRule::Sickened => mechanics
            .values
            .iter()
            .filter(|value| value.facets.surface.is_check_or_dc())
            .map(|value| status_penalty(value.target.clone(), source.clone(), amount))
            .collect(),
        ConditionRule::OffGuard => vec![CandidateModifier {
            target: MechanicTarget::ArmorClass,
            source: source.clone(),
            label: source,
            modifier_type: StatModifierTypeView::Circumstance,
            value: -2,
        }],
        ConditionRule::Clumsy => ability_targets(mechanics, AbilityKind::Dexterity)
            .into_iter()
            .map(|target| status_penalty(target, source.clone(), amount))
            .collect(),
        ConditionRule::Enfeebled => ability_targets(mechanics, AbilityKind::Strength)
            .into_iter()
            .map(|target| status_penalty(target, source.clone(), amount))
            .collect(),
        ConditionRule::Stupefied => mental_targets(mechanics)
            .into_iter()
            .map(|target| status_penalty(target, source.clone(), amount))
            .collect(),
    }
}

fn condition_unapplied_effects(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
) -> Vec<UnappliedEffectView> {
    let source = condition_source(condition);
    match rule {
        ConditionRule::Clumsy => vec![UnappliedEffectView {
            source: source.clone(),
            label: format!("{source} unmodeled Dexterity attack penalties"),
            reason: "Only structured activity attack rolls are adjusted.".to_string(),
        }],
        ConditionRule::Enfeebled => vec![UnappliedEffectView {
            source: source.clone(),
            label: format!("{source} untyped Strength-based damage penalties"),
            reason: "Only structured Strength-based strike damage is adjusted.".to_string(),
        }],
        ConditionRule::Stupefied => vec![UnappliedEffectView {
            source: source.clone(),
            label: format!("{source} spell disruption flat check"),
            reason: "Flat-check spell disruption is not automated yet.".to_string(),
        }],
        ConditionRule::Frightened | ConditionRule::Sickened | ConditionRule::OffGuard => Vec::new(),
    }
}

fn ability_targets(mechanics: &MechanicsView, ability: AbilityKind) -> Vec<MechanicTarget> {
    mechanics
        .values
        .iter()
        .filter(|value| {
            value.facets.surface != MechanicSurface::RawModifier
                && value.facets.ability == Some(ability)
        })
        .map(|value| value.target.clone())
        .collect()
}

fn mental_targets(mechanics: &MechanicsView) -> Vec<MechanicTarget> {
    mechanics
        .values
        .iter()
        .filter(|value| {
            value.facets.surface != MechanicSurface::RawModifier
                && matches!(
                    value.facets.ability,
                    Some(AbilityKind::Intelligence | AbilityKind::Wisdom | AbilityKind::Charisma)
                )
        })
        .map(|value| value.target.clone())
        .collect()
}

fn condition_value(condition: &EncounterParticipantCondition) -> i64 {
    condition.value.unwrap_or(1).max(1)
}

fn condition_source(condition: &EncounterParticipantCondition) -> String {
    let name = condition.name.trim();
    if name.is_empty() {
        return "Condition".to_string();
    }
    match condition.value {
        Some(value) if value > 0 => format!("{name} {value}"),
        _ => name.to_string(),
    }
}

#[cfg(test)]
fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_separator = false;
        } else if !previous_separator && !slug.is_empty() {
            slug.push('-');
            previous_separator = true;
        }
    }
    if previous_separator {
        slug.pop();
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::{
        AtlasRecord, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, MetricDefinition,
        MetricRow, MetricValue, RecordClassification, RecordIdentity, RecordProvenance, metrics,
    };

    #[test]
    fn elite_adjusts_projected_creature_stats_and_hp_by_level_band() {
        let participant = participant(ParticipantVariant::Elite, Vec::new());
        let projection = participant_stat_block(&participant, &record()).expect("stat block");
        assert_eq!(projection.adjusted_level, Some(6));
        assert_stat(&projection, "ac", 22, 24, "Elite adjustment");
        assert_stat(&projection, "hp.max", 60, 80, "Elite HP adjustment");
    }

    #[test]
    fn weak_adjusts_projected_creature_stats_and_hp_by_level_band() {
        let participant = participant(ParticipantVariant::Weak, Vec::new());
        let projection = participant_stat_block(&participant, &record()).expect("stat block");
        assert_eq!(projection.adjusted_level, Some(4));
        assert_stat(&projection, "perception", 13, 11, "Weak adjustment");
        assert_stat(&projection, "hp.max", 60, 45, "Weak HP adjustment");
    }

    #[test]
    fn condition_penalties_stack_by_modifier_type() {
        let conditions = vec![
            condition("Frightened", Some(1)),
            condition("Sickened", Some(2)),
            condition("Off-Guard", None),
        ];
        let participant = participant(ParticipantVariant::Normal, conditions);
        let projection = participant_stat_block(&participant, &record()).expect("stat block");
        let ac = value(&projection, "ac");
        assert_eq!(ac.adjusted_value, 18);
        assert!(
            ac.modifiers
                .iter()
                .any(|modifier| modifier.label == "Sickened 2")
        );
        assert!(
            ac.modifiers
                .iter()
                .any(|modifier| modifier.label == "Off-Guard")
        );
        assert!(
            ac.suppressed_modifiers
                .iter()
                .any(|modifier| modifier.label == "Frightened 1")
        );
    }

    #[test]
    fn targeted_conditions_project_supported_stats_and_report_unapplied_effects() {
        let conditions = vec![
            condition("Clumsy", None),
            condition("Enfeebled", Some(2)),
            condition("Stupefied", Some(1)),
        ];
        let participant = participant(ParticipantVariant::Normal, conditions);
        let projection = participant_stat_block(&participant, &record()).expect("stat block");
        assert_eq!(value(&projection, "save.ref").adjusted_value, 11);
        assert_eq!(value(&projection, "skill.athletics").adjusted_value, 7);
        assert_eq!(value(&projection, "save.will").adjusted_value, 11);
        assert_eq!(projection.unapplied_effects.len(), 3);
    }

    #[test]
    fn frightened_penalizes_checks_and_dcs_but_not_raw_ability_modifiers() {
        let participant = participant(
            ParticipantVariant::Normal,
            vec![condition("Frightened", Some(1))],
        );
        let projection = participant_stat_block(&participant, &record()).expect("stat block");

        assert_eq!(value(&projection, "ac").adjusted_value, 21);
        assert_eq!(value(&projection, "perception").adjusted_value, 12);
        assert_eq!(value(&projection, "save.will").adjusted_value, 11);
        assert_eq!(value(&projection, "skill.athletics").adjusted_value, 8);
        assert_eq!(value(&projection, "ability.str").adjusted_value, 4);
        assert_eq!(value(&projection, "ability.dex").adjusted_value, 3);
        assert!(value(&projection, "ability.str").modifiers.is_empty());
    }

    #[test]
    fn condition_names_without_modeled_keys_remain_annotation_only() {
        let participant = participant(
            ParticipantVariant::Normal,
            vec![unmodeled_condition("Frightened", Some(3))],
        );
        let projection = participant_stat_block(&participant, &record()).expect("stat block");

        assert_eq!(value(&projection, "ac").adjusted_value, 22);
        assert_eq!(value(&projection, "perception").adjusted_value, 13);
        assert_eq!(value(&projection, "skill.athletics").adjusted_value, 9);
    }

    #[test]
    fn elite_and_weak_project_structured_activity_damage_adjustments() {
        let elite = participant_stat_block(
            &participant(ParticipantVariant::Elite, Vec::new()),
            &record(),
        )
        .expect("stat block");
        assert_damage_modifier(&elite, "claw", "main", 2);
        assert_eq!(
            damage(&elite, "claw", "main").adjusted_formula.as_deref(),
            Some("1d6 + 6")
        );
        assert_damage_modifier(&elite, "fireball", "0", 4);
        assert_no_damage_modifier(&elite, "breath", "0");
        assert_no_damage_modifier(&elite, "heal", "0");
        assert_no_mode_damage_modifier(&elite, "heal", "living", "0");
        assert_mode_damage_modifier(&elite, "heal", "undead", "0", 4);
        assert_eq!(
            mode_damage(&elite, "heal", "undead", "0")
                .adjusted_formula
                .as_deref(),
            Some("1d8 + 4")
        );

        let weak = participant_stat_block(
            &participant(ParticipantVariant::Weak, Vec::new()),
            &record(),
        )
        .expect("stat block");
        assert_damage_modifier(&weak, "claw", "main", -2);
        assert_damage_modifier(&weak, "fireball", "0", -4);
        assert_no_damage_modifier(&weak, "breath", "0");
        assert_no_damage_modifier(&weak, "heal", "0");
    }

    #[test]
    fn activity_roll_surfaces_receive_variant_and_condition_modifiers() {
        let elite = participant_stat_block(
            &participant(ParticipantVariant::Elite, Vec::new()),
            &record(),
        )
        .expect("stat block");
        assert_roll(&elite, "claw", "attack", 12, 14, "Elite adjustment");
        assert_roll(&elite, "fireball", "spell.dc", 22, 24, "Elite adjustment");

        let conditions = vec![
            condition("Frightened", Some(1)),
            condition("Enfeebled", Some(2)),
            condition("Stupefied", Some(2)),
        ];
        let projection = participant_stat_block(
            &participant(ParticipantVariant::Normal, conditions),
            &record(),
        )
        .expect("stat block");
        assert_roll(&projection, "claw", "attack", 12, 10, "Enfeebled 2");
        assert_roll(&projection, "fireball", "spell.dc", 22, 20, "Stupefied 2");
        assert!(
            roll(&projection, "fireball", "spell.dc")
                .suppressed_modifiers
                .iter()
                .any(|modifier| modifier.label == "Frightened 1")
        );
        assert_damage_modifier(&projection, "claw", "main", -2);
        assert_no_damage_modifier(&projection, "fireball", "0");
    }

    fn assert_stat(
        projection: &StatBlockView,
        target: &str,
        base: i64,
        adjusted: i64,
        label: &str,
    ) {
        let value = value(projection, target);
        assert_eq!(value.base_value, base);
        assert_eq!(value.adjusted_value, adjusted);
        assert!(
            value
                .modifiers
                .iter()
                .any(|modifier| modifier.label == label)
        );
    }

    fn value<'a>(projection: &'a StatBlockView, target: &str) -> &'a StatValueView {
        projection
            .values
            .iter()
            .find(|value| value.target == target)
            .expect("stat value should exist")
    }

    fn assert_damage_modifier(
        projection: &StatBlockView,
        activity_id: &str,
        damage_id: &str,
        value: i64,
    ) {
        let damage = damage(projection, activity_id, damage_id);
        assert_eq!(damage.modifiers.len(), 1);
        assert_eq!(damage.modifiers[0].value, value);
    }

    fn assert_no_damage_modifier(projection: &StatBlockView, activity_id: &str, damage_id: &str) {
        assert!(
            damage(projection, activity_id, damage_id)
                .modifiers
                .is_empty()
        );
    }

    fn assert_mode_damage_modifier(
        projection: &StatBlockView,
        activity_id: &str,
        mode_id: &str,
        damage_id: &str,
        value: i64,
    ) {
        let damage = mode_damage(projection, activity_id, mode_id, damage_id);
        assert_eq!(damage.modifiers.len(), 1);
        assert_eq!(damage.modifiers[0].value, value);
    }

    fn assert_no_mode_damage_modifier(
        projection: &StatBlockView,
        activity_id: &str,
        mode_id: &str,
        damage_id: &str,
    ) {
        assert!(
            mode_damage(projection, activity_id, mode_id, damage_id)
                .modifiers
                .is_empty()
        );
    }

    fn assert_roll(
        projection: &StatBlockView,
        activity_id: &str,
        roll_id: &str,
        base: i64,
        adjusted: i64,
        label: &str,
    ) {
        let roll = roll(projection, activity_id, roll_id);
        assert_eq!(roll.base_value, base);
        assert_eq!(roll.adjusted_value, adjusted);
        assert!(
            roll.modifiers
                .iter()
                .any(|modifier| modifier.label == label)
        );
    }

    fn roll<'a>(
        projection: &'a StatBlockView,
        activity_id: &str,
        roll_id: &str,
    ) -> &'a ActivityRollView {
        projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == activity_id)
            .and_then(|activity| activity.rolls.iter().find(|roll| roll.roll_id == roll_id))
            .expect("activity roll should exist")
    }

    fn damage<'a>(
        projection: &'a StatBlockView,
        activity_id: &str,
        damage_id: &str,
    ) -> &'a DamageExpressionView {
        projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == activity_id)
            .and_then(|activity| {
                activity
                    .damage
                    .iter()
                    .find(|damage| damage.damage_id == damage_id)
            })
            .expect("damage expression should exist")
    }

    fn mode_damage<'a>(
        projection: &'a StatBlockView,
        activity_id: &str,
        mode_id: &str,
        damage_id: &str,
    ) -> &'a DamageExpressionView {
        projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == activity_id)
            .and_then(|activity| {
                activity
                    .modes
                    .iter()
                    .find(|mode| mode.mode_id == mode_id)
                    .and_then(|mode| {
                        mode.damage
                            .iter()
                            .find(|damage| damage.damage_id == damage_id)
                    })
            })
            .expect("mode damage expression should exist")
    }

    fn participant(
        participant_variant: ParticipantVariant,
        conditions: Vec<EncounterParticipantCondition>,
    ) -> EncounterParticipant {
        EncounterParticipant {
            participant_key: "participant".to_string(),
            record_key: Some("actors:test".to_string()),
            participant_kind: atlas_local_state::ParticipantKind::Creature,
            participant_variant,
            position: 1,
            display_name: "Creature".to_string(),
            record_title_snapshot: Some("Creature".to_string()),
            record_kind_snapshot: Some("creature".to_string()),
            side: atlas_local_state::ParticipantSide::Enemy,
            initiative: None,
            initiative_order: 1,
            max_hp: Some(60),
            current_hp: Some(60),
            temporary_hp: 0,
            defeated: false,
            hidden: false,
            note: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            conditions,
        }
    }

    fn condition(name: &str, value: Option<i64>) -> EncounterParticipantCondition {
        EncounterParticipantCondition {
            condition_id: 1,
            condition_key: Some(condition_key(name).to_string()),
            name: name.to_string(),
            value,
            source_participant_key: None,
            duration_rounds: None,
            note: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn unmodeled_condition(name: &str, value: Option<i64>) -> EncounterParticipantCondition {
        EncounterParticipantCondition {
            condition_id: 1,
            condition_key: None,
            name: name.to_string(),
            value,
            source_participant_key: None,
            duration_rounds: None,
            note: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn condition_key(name: &str) -> &'static str {
        match slugify(name).as_str() {
            "frightened" => "conditionitems:TBSHQspnbcqxsmjL",
            "sickened" => "conditionitems:fesd1n5eVhpCSS18",
            "off-guard" => "conditionitems:AJh5ex99aV6VTggg",
            "clumsy" => "conditionitems:i3OJZU2nk64Df3xm",
            "enfeebled" => "conditionitems:MIRkyAjyBeXivMa7",
            "stupefied" => "conditionitems:e1XGnhKNSQIm5IXg",
            _ => "conditionitems:unsupported",
        }
    }

    fn record() -> AtlasRecord {
        let mut classification = RecordClassification::new(RecordKind::Creature);
        classification.level = Some(5);
        let mut record = AtlasRecord::new(
            RecordIdentity::new(RecordKey::parse("actors:test").expect("key"), "Creature"),
            classification,
            FoundryRecordInfo::new("Actors", FoundryDocumentType::Actor, FoundryRecordType::Npc),
            RecordProvenance::new("test.json"),
        );
        record.mechanics.metrics = vec![
            defined_metric(metrics::actor::ARMOR_CLASS, 22.0),
            defined_metric(metrics::actor::HP_MAX, 60.0),
            defined_metric(metrics::actor::PERCEPTION_MOD, 13.0),
            metric("save.fort.mod", 15.0),
            metric("save.ref.mod", 12.0),
            metric("save.will.mod", 12.0),
            metric("ability.str.mod", 4.0),
            metric("ability.dex.mod", 3.0),
            metric("ability.int.mod", 1.0),
            metric("ability.wis.mod", 2.0),
            metric("ability.cha.mod", 0.0),
            metric("skill.athletics.mod", 9.0),
            metric("skill.stealth.mod", 8.0),
            metric("skill.arcana.mod", 7.0),
        ];
        record.mechanics.activities = vec![
            atlas_record::MechanicActivity {
                activity_id: "claw".to_string(),
                label: "Claw".to_string(),
                kind: atlas_record::MechanicActivityKind::Strike,
                traits: Vec::new(),
                compendium_source: None,
                usage: atlas_record::MechanicActivityUsage::Unlimited,
                rolls: vec![atlas_record::ActivityRoll {
                    roll_id: "attack".to_string(),
                    label: "Attack".to_string(),
                    base_value: 12,
                    surface: atlas_record::ActivityRollSurface::AttackRoll,
                    ability: Some(atlas_record::ActivityRollAbility::Strength),
                }],
                damage: vec![atlas_record::DamageExpression {
                    damage_id: "main".to_string(),
                    label: None,
                    formula: "1d6+4".to_string(),
                    damage_type: Some("slashing".to_string()),
                    effect_kind: atlas_record::DamageEffectKind::Damage,
                    ability: Some(atlas_record::ActivityRollAbility::Strength),
                }],
                modes: Vec::new(),
            },
            atlas_record::MechanicActivity {
                activity_id: "fireball".to_string(),
                label: "Fireball".to_string(),
                kind: atlas_record::MechanicActivityKind::Spell,
                traits: Vec::new(),
                compendium_source: None,
                usage: atlas_record::MechanicActivityUsage::Limited,
                rolls: vec![
                    atlas_record::ActivityRoll {
                        roll_id: "spell.attack".to_string(),
                        label: "Spell Attack".to_string(),
                        base_value: 13,
                        surface: atlas_record::ActivityRollSurface::AttackRoll,
                        ability: None,
                    },
                    atlas_record::ActivityRoll {
                        roll_id: "spell.dc".to_string(),
                        label: "Spell DC".to_string(),
                        base_value: 22,
                        surface: atlas_record::ActivityRollSurface::Dc,
                        ability: None,
                    },
                ],
                damage: vec![atlas_record::DamageExpression {
                    damage_id: "0".to_string(),
                    label: None,
                    formula: "6d6".to_string(),
                    damage_type: Some("fire".to_string()),
                    effect_kind: atlas_record::DamageEffectKind::Damage,
                    ability: None,
                }],
                modes: Vec::new(),
            },
            atlas_record::MechanicActivity {
                activity_id: "heal".to_string(),
                label: "Heal".to_string(),
                kind: atlas_record::MechanicActivityKind::Spell,
                traits: vec!["healing".to_string(), "vitality".to_string()],
                compendium_source: None,
                usage: atlas_record::MechanicActivityUsage::Limited,
                rolls: Vec::new(),
                damage: vec![atlas_record::DamageExpression {
                    damage_id: "0".to_string(),
                    label: None,
                    formula: "1d8".to_string(),
                    damage_type: Some("vitality".to_string()),
                    effect_kind: atlas_record::DamageEffectKind::DamageOrHealing,
                    ability: None,
                }],
                modes: vec![
                    atlas_record::MechanicActivityMode {
                        mode_id: "living".to_string(),
                        label: "Heal (vs. Living)".to_string(),
                        sort: 2,
                        target: Some("1 willing living creature".to_string()),
                        range: Some("30 feet".to_string()),
                        time: Some("2".to_string()),
                        damage: vec![atlas_record::DamageExpression {
                            damage_id: "0".to_string(),
                            label: None,
                            formula: "1d8+8".to_string(),
                            damage_type: Some("vitality".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Healing,
                            ability: None,
                        }],
                    },
                    atlas_record::MechanicActivityMode {
                        mode_id: "undead".to_string(),
                        label: "Heal (vs. Undead)".to_string(),
                        sort: 3,
                        target: Some("1 undead".to_string()),
                        range: Some("30 feet".to_string()),
                        time: Some("2".to_string()),
                        damage: vec![atlas_record::DamageExpression {
                            damage_id: "0".to_string(),
                            label: None,
                            formula: "1d8".to_string(),
                            damage_type: Some("vitality".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: None,
                        }],
                    },
                ],
            },
            atlas_record::MechanicActivity {
                activity_id: "breath".to_string(),
                label: "Breath Weapon".to_string(),
                kind: atlas_record::MechanicActivityKind::Other,
                traits: Vec::new(),
                compendium_source: None,
                usage: atlas_record::MechanicActivityUsage::Ambiguous,
                rolls: Vec::new(),
                damage: vec![atlas_record::DamageExpression {
                    damage_id: "0".to_string(),
                    label: None,
                    formula: "4d6".to_string(),
                    damage_type: Some("fire".to_string()),
                    effect_kind: atlas_record::DamageEffectKind::Damage,
                    ability: None,
                }],
                modes: Vec::new(),
            },
        ];
        record
    }

    fn defined_metric(definition: MetricDefinition, value: f64) -> MetricRow {
        metric(
            definition.exact_key().expect("metric has an exact key"),
            value,
        )
    }

    fn metric(key: &str, value: f64) -> MetricRow {
        MetricRow {
            domain: atlas_domain::MetricDomain::Actor,
            key: key.to_string(),
            value: MetricValue::Number(value),
        }
    }
}
