use std::collections::{BTreeMap, BTreeSet};

use atlas_app_model::{
    ActionBudgetView, ActivityRollSurfaceView, ActivityRollView, DamageEffectKindView,
    DamageExpressionView, EncounterParticipantVariantView, MechanicActivityKindView,
    MechanicActivityModeView, MechanicActivityUsageView, MechanicActivityView, MovementSpeedView,
    RuntimeAdjustmentView, RuntimeCapabilityView, RuntimeCountSegmentView, RuntimeCountView,
    RuntimeEffectNoteView, StatBlockView, StatModifierTypeView, StatModifierView, StatValueView,
    UnappliedEffectView,
};
use atlas_local_state::{EncounterParticipant, EncounterParticipantCondition, ParticipantVariant};
use atlas_record::{
    AbilityKind, ActivityRoll, ActivityRollAbility, ActivityRollSurface, CanonicalMechanicActivity,
    CanonicalMechanicsProjection, CreatureActionCost, CreatureDamage, CreatureDamageKind,
    CreatureFrequency, CreatureNumber, CreatureResourceAmount, CreatureRoll, CreatureRollKind,
    CreatureSourceScalar, CreatureUseLimit, DamageEffectKind, DamageExpression, FactValue,
    MechanicActivity, MechanicActivityFamily, MechanicActivityKind, MechanicActivityMode,
    MechanicActivityUsage, MechanicBaseValue, MechanicFact, MechanicScalar, MechanicSurface,
    MechanicTarget, MechanicValue, MechanicsView, MovementSpeed, RecordBody, RetrievedRecord,
    SaveKind, UnsupportedMechanic, UnsupportedMechanicValue, UnsupportedSourceReason,
    UnsupportedSourceShape, UnsupportedSourceValue, build_mechanics_view,
    project_creature_mechanics,
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

#[derive(Debug, Clone)]
struct RuntimeAdjustment {
    source: String,
    label: String,
    value: i64,
    reason: Option<String>,
    floor: Option<i64>,
}

#[derive(Debug, Clone)]
struct RuntimeNote {
    source: String,
    label: String,
    reason: String,
}

pub(super) fn participant_stat_block(
    participant: &EncounterParticipant,
    retrieved: &RetrievedRecord,
) -> Option<StatBlockView> {
    let Some(RecordBody::Creature(creature)) = retrieved.body.as_ref() else {
        return None;
    };
    let mechanics = canonical_participant_mechanics(
        project_creature_mechanics(creature),
        creature.identity.name.clone(),
    );
    Some(apply_participant_effects(
        participant,
        mechanics.view,
        mechanics.unapplied_effects,
        mechanics.variant_damage_blocked_activity_ids,
    ))
}

pub(crate) fn record_stat_block(record: &atlas_record::AtlasRecord) -> Option<StatBlockView> {
    let mechanics = build_mechanics_view(record)?;
    Some(base_mechanics_block(mechanics))
}

pub(super) fn participant_runtime_block(participant: &EncounterParticipant) -> StatBlockView {
    StatBlockView {
        record_key: participant.participant_key.clone(),
        title: participant.display_name.clone(),
        level: None,
        adjusted_level: None,
        values: Vec::new(),
        speeds: Vec::new(),
        action_budget: Some(action_budget_view(participant)),
        activities: Vec::new(),
        unapplied_effects: participant_runtime_notes(participant)
            .into_iter()
            .map(unapplied_runtime_note_view)
            .collect(),
    }
}

fn base_mechanics_block(mechanics: MechanicsView) -> StatBlockView {
    StatBlockView {
        record_key: mechanics.record_key.to_string(),
        title: mechanics.title,
        level: mechanics.level,
        adjusted_level: mechanics.level,
        values: mechanics
            .values
            .into_iter()
            .map(|value| stat_value_view(value, Vec::new()))
            .collect(),
        speeds: mechanics.speeds.into_iter().map(base_speed_view).collect(),
        action_budget: None,
        activities: mechanics
            .activities
            .into_iter()
            .map(base_activity_view)
            .collect(),
        unapplied_effects: Vec::new(),
    }
}

fn base_speed_view(speed: MovementSpeed) -> MovementSpeedView {
    MovementSpeedView {
        movement_type: speed.movement_type,
        label: speed.label,
        base_value_feet: speed.value_feet,
        adjusted_value_feet: speed.value_feet,
        adjustments: Vec::new(),
        suppressed_adjustments: Vec::new(),
        notes: Vec::new(),
    }
}

fn base_activity_view(activity: MechanicActivity) -> MechanicActivityView {
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
            .map(base_activity_roll_view)
            .collect(),
        damage: activity.damage.into_iter().map(base_damage_view).collect(),
        modes: activity
            .modes
            .into_iter()
            .map(base_activity_mode_view)
            .collect(),
    }
}

fn base_activity_mode_view(mode: MechanicActivityMode) -> MechanicActivityModeView {
    MechanicActivityModeView {
        mode_id: mode.mode_id,
        label: mode.label,
        target: mode.target,
        range: mode.range,
        time: mode.time,
        damage: mode.damage.into_iter().map(base_damage_view).collect(),
    }
}

fn base_activity_roll_view(roll: ActivityRoll) -> ActivityRollView {
    ActivityRollView {
        roll_id: roll.roll_id,
        label: roll.label,
        base_value: roll.base_value,
        adjusted_value: roll.base_value,
        surface: activity_roll_surface_view(roll.surface),
        modifiers: Vec::new(),
        suppressed_modifiers: Vec::new(),
    }
}

fn base_damage_view(damage: DamageExpression) -> DamageExpressionView {
    DamageExpressionView {
        damage_id: damage.damage_id,
        label: damage.label,
        formula: damage.formula,
        adjusted_formula: None,
        damage_type: damage.damage_type,
        effect_kind: damage_effect_kind_view(damage.effect_kind),
        modifiers: Vec::new(),
    }
}

struct ParticipantMechanics {
    view: MechanicsView,
    unapplied_effects: Vec<UnappliedEffectView>,
    variant_damage_blocked_activity_ids: BTreeSet<String>,
}

fn canonical_participant_mechanics(
    projection: CanonicalMechanicsProjection,
    title: String,
) -> ParticipantMechanics {
    let level = fact_integer(&projection.level);
    let mut values = Vec::new();
    let mut speeds = Vec::new();
    for fact in projection.facts {
        match &fact.target {
            MechanicTarget::Movement { speed_id } => {
                if let Some(value_feet) = mechanic_integer(&fact.value) {
                    speeds.push(MovementSpeed {
                        movement_type: speed_id.as_str().to_string(),
                        label: fact.label,
                        value_feet,
                    });
                }
            }
            _ => {
                if let Some(base_value) = mechanic_integer(&fact.value) {
                    values.push(MechanicValue {
                        target: fact.target,
                        label: fact.label,
                        base_value: MechanicScalar::Number(base_value),
                        facets: fact.facets,
                    });
                }
            }
        }
    }
    let mut unapplied_effects = projection
        .unsupported
        .into_iter()
        .map(canonical_unsupported_effect)
        .collect::<Vec<_>>();
    let mut activities = Vec::new();
    let mut variant_damage_blocked_activity_ids = BTreeSet::new();
    for activity in projection.activities {
        unapplied_effects.extend(
            activity
                .unsupported
                .iter()
                .cloned()
                .map(canonical_unsupported_effect),
        );
        let disposition = canonical_activity(activity);
        if disposition.variant_damage_blocked {
            variant_damage_blocked_activity_ids.insert(disposition.activity.activity_id.clone());
        }
        values.extend(disposition.values);
        unapplied_effects.extend(disposition.unapplied_effects);
        activities.push(disposition.activity);
    }

    ParticipantMechanics {
        view: MechanicsView {
            record_key: projection.record_key,
            kind: atlas_domain::RecordKind::Creature,
            title,
            level,
            values,
            speeds,
            activities,
        },
        unapplied_effects,
        variant_damage_blocked_activity_ids,
    }
}

struct CanonicalActivityDisposition {
    activity: MechanicActivity,
    values: Vec<MechanicValue>,
    unapplied_effects: Vec<UnappliedEffectView>,
    variant_damage_blocked: bool,
}

fn canonical_activity(activity: CanonicalMechanicActivity) -> CanonicalActivityDisposition {
    let ability = activity_attack_ability(&activity);
    let usage = canonical_activity_usage(&activity);
    let variant_damage_blocked = canonical_variant_damage_blocked(&activity);
    let kind = match activity.family {
        MechanicActivityFamily::Strike => MechanicActivityKind::Strike,
        MechanicActivityFamily::Spell | MechanicActivityFamily::SpellcastingEntry => {
            MechanicActivityKind::Spell
        }
        MechanicActivityFamily::Action | MechanicActivityFamily::Unsupported => {
            MechanicActivityKind::Other
        }
    };
    let rolls = activity
        .facts
        .iter()
        .filter_map(canonical_activity_roll)
        .collect();
    let damage = activity
        .facts
        .iter()
        .filter_map(|fact| canonical_activity_damage(fact, ability))
        .collect();
    let values = activity
        .facts
        .iter()
        .filter_map(canonical_activity_value)
        .collect();
    let unapplied_effects = activity
        .facts
        .iter()
        .filter_map(canonical_activity_unapplied_effect)
        .collect();
    CanonicalActivityDisposition {
        activity: MechanicActivity {
            activity_id: activity.occurrence_id.as_str().to_string(),
            label: activity.label,
            kind,
            traits: Vec::new(),
            compendium_source: None,
            usage,
            rolls,
            damage,
            modes: Vec::new(),
        },
        values,
        unapplied_effects,
        variant_damage_blocked,
    }
}

fn canonical_variant_damage_blocked(activity: &CanonicalMechanicActivity) -> bool {
    activity
        .facts
        .iter()
        .filter_map(|fact| match (&fact.target, &fact.value) {
            (MechanicTarget::ActivityDamage { .. }, MechanicBaseValue::Damage(damage)) => {
                Some(damage)
            }
            _ => None,
        })
        .find(|damage| canonical_damage_effect_kind(damage) == DamageEffectKind::Damage)
        .is_some_and(|damage| damage.formula.as_value().is_none())
}

fn canonical_activity_roll(fact: &MechanicFact) -> Option<ActivityRoll> {
    match (&fact.target, &fact.value) {
        (MechanicTarget::ActivityRoll { roll_id, .. }, MechanicBaseValue::Roll(roll)) => {
            let surface = match roll.kind {
                CreatureRollKind::Attack => ActivityRollSurface::AttackRoll,
                CreatureRollKind::DifficultyClass => ActivityRollSurface::Dc,
                // The encounter DTO has no generic check surface. The complete typed fact is
                // retained as an explicit unapplied note by `canonical_activity_unapplied_effect`.
                CreatureRollKind::Check => return None,
            };
            Some(ActivityRoll {
                roll_id: roll_id.clone(),
                label: fact.label.clone(),
                base_value: fact_integer(&roll.value)?,
                surface,
                ability: fact_activity_ability(&roll.ability),
            })
        }
        (MechanicTarget::SpellcastingAttack { .. }, _) => Some(ActivityRoll {
            roll_id: fact.target.id(),
            label: fact.label.clone(),
            base_value: mechanic_integer(&fact.value)?,
            surface: ActivityRollSurface::AttackRoll,
            ability: None,
        }),
        (MechanicTarget::SpellcastingDc { .. }, _) => Some(ActivityRoll {
            roll_id: fact.target.id(),
            label: fact.label.clone(),
            base_value: mechanic_integer(&fact.value)?,
            surface: ActivityRollSurface::Dc,
            ability: None,
        }),
        _ => None,
    }
}

fn canonical_activity_value(fact: &MechanicFact) -> Option<MechanicValue> {
    let MechanicTarget::SpellSlotMaximum { .. } = &fact.target else {
        return None;
    };
    let MechanicBaseValue::SourceInteger(FactValue::Value(CreatureSourceScalar::Value(value))) =
        &fact.value
    else {
        return None;
    };
    Some(MechanicValue {
        target: fact.target.clone(),
        label: fact.label.clone(),
        base_value: MechanicScalar::Number(*value),
        facets: fact.facets.clone(),
    })
}

fn canonical_activity_unapplied_effect(fact: &MechanicFact) -> Option<UnappliedEffectView> {
    let disposition = match (&fact.target, &fact.value) {
        (
            MechanicTarget::ActivityRoll { .. },
            MechanicBaseValue::Roll(
                roll @ CreatureRoll {
                    kind: CreatureRollKind::Check,
                    ..
                },
            ),
        ) => format!(
            "check roll has no encounter roll surface; id={:?}; label={:?}; value={}; ability={}",
            roll.id,
            roll.label,
            fact_i64(&roll.value),
            fact_roll_ability(&roll.ability)
        ),
        (MechanicTarget::ActivityActionCost { .. }, MechanicBaseValue::ActionCost(value)) => {
            format!(
                "action cost has no exact encounter activity field; value={}",
                action_cost(value)
            )
        }
        (MechanicTarget::ActivityFrequency { .. }, MechanicBaseValue::Frequency(value)) => {
            format!(
                "frequency is represented only by coarse activity usage; value={}",
                frequency(value)
            )
        }
        (MechanicTarget::ActivityUses { .. }, MechanicBaseValue::Uses(value)) => format!(
            "uses are represented only by coarse activity usage; value={}",
            uses(value)
        ),
        (target @ MechanicTarget::ActivityDamage { .. }, MechanicBaseValue::Damage(damage))
            if damage.formula.as_value().is_none() =>
        {
            format!(
                "damage formula cannot populate a structured encounter damage expression; target={}; id={:?}; formula={}",
                target.id(),
                damage.id,
                fact_string(&damage.formula)
            )
        }
        (
            MechanicTarget::SpellSlotMaximum { .. },
            MechanicBaseValue::SourceInteger(FactValue::Value(CreatureSourceScalar::Value(_))),
        ) => return None,
        (MechanicTarget::SpellSlotMaximum { .. }, MechanicBaseValue::SourceInteger(value)) => {
            format!(
                "spell-slot maximum cannot populate a numeric encounter value; value={}",
                source_i64(value)
            )
        }
        _ => return None,
    };
    Some(UnappliedEffectView {
        source: "Canonical source".to_string(),
        label: format!(
            "{} retained without exact encounter field",
            fact.target.id()
        ),
        reason: format!("{}: {disposition}", fact.label),
    })
}

fn fact_i64(value: &FactValue<i64>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!("value({value})"),
    }
}

fn fact_string(value: &FactValue<String>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!("value({value:?})"),
    }
}

fn fact_roll_ability(value: &FactValue<ActivityRollAbility>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!("value({})", roll_ability_name(*value)),
    }
}

fn roll_ability_name(value: ActivityRollAbility) -> &'static str {
    match value {
        ActivityRollAbility::Strength => "strength",
        ActivityRollAbility::Dexterity => "dexterity",
        ActivityRollAbility::Constitution => "constitution",
        ActivityRollAbility::Intelligence => "intelligence",
        ActivityRollAbility::Wisdom => "wisdom",
        ActivityRollAbility::Charisma => "charisma",
    }
}

fn action_cost(value: &CreatureActionCost) -> String {
    match value {
        CreatureActionCost::Passive => "passive".to_string(),
        CreatureActionCost::Reaction => "reaction".to_string(),
        CreatureActionCost::FreeAction => "free_action".to_string(),
        CreatureActionCost::Actions(value) => format!("actions({value})"),
        CreatureActionCost::Time(value) => format!("time({value:?})"),
        CreatureActionCost::Unsupported(value) => unsupported_source_value(value),
    }
}

fn frequency(value: &FactValue<CreatureFrequency>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!(
            "maximum={}, period={}, serialized_value={}",
            fact_i64(&value.maximum),
            fact_string(&value.period),
            fact_i64(&value.serialized_value)
        ),
    }
}

fn uses(value: &FactValue<CreatureUseLimit>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!(
            "maximum={}, serialized_value={}",
            fact_i64(&value.maximum),
            fact_i64(&value.serialized_value)
        ),
    }
}

fn source_i64(value: &FactValue<CreatureSourceScalar<i64>>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(CreatureSourceScalar::Value(value)) => format!("value({value})"),
        FactValue::Value(CreatureSourceScalar::Unsupported(value)) => {
            unsupported_source_value(value)
        }
    }
}

fn unsupported_source_value(value: &UnsupportedSourceValue) -> String {
    format!(
        "unsupported(shape={}, value={:?}, reason={})",
        unsupported_source_shape(value.shape),
        value.value,
        unsupported_source_reason(value.reason)
    )
}

fn unsupported_source_shape(value: UnsupportedSourceShape) -> &'static str {
    match value {
        UnsupportedSourceShape::Missing => "missing",
        UnsupportedSourceShape::Null => "null",
        UnsupportedSourceShape::String => "string",
        UnsupportedSourceShape::Number => "number",
        UnsupportedSourceShape::Boolean => "boolean",
        UnsupportedSourceShape::Array => "array",
        UnsupportedSourceShape::Object => "object",
    }
}

fn unsupported_source_reason(value: UnsupportedSourceReason) -> &'static str {
    match value {
        UnsupportedSourceReason::OpenVocabulary => "open_vocabulary",
        UnsupportedSourceReason::AmbiguousLegacyShape => "ambiguous_legacy_shape",
        UnsupportedSourceReason::InvalidPredicate => "invalid_predicate",
        UnsupportedSourceReason::NonCanonicalRuntimeValue => "non_canonical_runtime_value",
        UnsupportedSourceReason::SourceFieldDrift => "source_field_drift",
    }
}

fn canonical_activity_damage(
    fact: &MechanicFact,
    activity_ability: Option<ActivityRollAbility>,
) -> Option<DamageExpression> {
    let MechanicBaseValue::Damage(damage) = &fact.value else {
        return None;
    };
    Some(DamageExpression {
        damage_id: damage.id.clone(),
        label: (fact.label != damage.id).then(|| fact.label.clone()),
        formula: damage.formula.as_value()?.clone(),
        damage_type: damage.damage_type.as_value().cloned(),
        effect_kind: canonical_damage_effect_kind(damage),
        ability: damage_applies_modifier(damage)
            .then_some(activity_ability)
            .flatten(),
    })
}

fn activity_attack_ability(activity: &CanonicalMechanicActivity) -> Option<ActivityRollAbility> {
    activity.facts.iter().find_map(|fact| {
        let MechanicBaseValue::Roll(roll) = &fact.value else {
            return None;
        };
        (roll.kind == CreatureRollKind::Attack)
            .then(|| fact_activity_ability(&roll.ability))
            .flatten()
    })
}

fn canonical_activity_usage(activity: &CanonicalMechanicActivity) -> MechanicActivityUsage {
    match activity.family {
        MechanicActivityFamily::Strike => MechanicActivityUsage::Unlimited,
        MechanicActivityFamily::Spell | MechanicActivityFamily::Action => {
            if activity.facts.iter().any(fact_has_limited_use) {
                MechanicActivityUsage::Limited
            } else {
                MechanicActivityUsage::Unlimited
            }
        }
        MechanicActivityFamily::SpellcastingEntry => MechanicActivityUsage::Limited,
        MechanicActivityFamily::Unsupported => MechanicActivityUsage::Ambiguous,
    }
}

fn fact_has_limited_use(fact: &MechanicFact) -> bool {
    match &fact.value {
        MechanicBaseValue::Frequency(value) => value
            .as_value()
            .and_then(|frequency| frequency.maximum.as_value())
            .is_some(),
        MechanicBaseValue::Uses(value) => value
            .as_value()
            .and_then(|uses| uses.maximum.as_value())
            .is_some(),
        _ => false,
    }
}

fn canonical_damage_effect_kind(damage: &CreatureDamage) -> DamageEffectKind {
    let Some(kinds) = damage.kinds.as_value() else {
        return DamageEffectKind::Unknown;
    };
    let has_damage = kinds.contains(&CreatureDamageKind::Damage);
    let has_healing = kinds.contains(&CreatureDamageKind::Healing);
    match (has_damage, has_healing) {
        (true, false) => DamageEffectKind::Damage,
        (false, true) => DamageEffectKind::Healing,
        (true, true) => DamageEffectKind::DamageOrHealing,
        (false, false) => DamageEffectKind::Unknown,
    }
}

fn damage_applies_modifier(damage: &CreatureDamage) -> bool {
    matches!(
        damage.apply_modifier,
        FactValue::Value(CreatureSourceScalar::Value(true))
    )
}

fn mechanic_integer(value: &MechanicBaseValue) -> Option<i64> {
    match value {
        MechanicBaseValue::Integer(value) => fact_integer(value),
        MechanicBaseValue::Number(value) => match value.as_value()? {
            CreatureNumber::Integer(value) => Some(*value),
            CreatureNumber::Unsupported(_) => None,
        },
        MechanicBaseValue::ResourceAmount(value) => match value.as_value()? {
            CreatureResourceAmount::Integer(value) => Some(*value),
            CreatureResourceAmount::Unsupported(_) => None,
        },
        MechanicBaseValue::SourceInteger(value) => match value.as_value()? {
            CreatureSourceScalar::Value(value) => Some(*value),
            CreatureSourceScalar::Unsupported(_) => None,
        },
        MechanicBaseValue::Roll(value) => fact_integer(&value.value),
        MechanicBaseValue::ActionCost(_)
        | MechanicBaseValue::Frequency(_)
        | MechanicBaseValue::Uses(_)
        | MechanicBaseValue::Damage(_) => None,
    }
}

fn fact_integer(value: &FactValue<i64>) -> Option<i64> {
    value.as_value().copied()
}

fn fact_activity_ability(value: &FactValue<ActivityRollAbility>) -> Option<ActivityRollAbility> {
    value.as_value().copied()
}

fn canonical_unsupported_effect(unsupported: UnsupportedMechanic) -> UnappliedEffectView {
    let target = unsupported
        .target
        .as_ref()
        .map(MechanicTarget::id)
        .unwrap_or_else(|| "unmodeled mechanic".to_string());
    UnappliedEffectView {
        source: "Canonical source".to_string(),
        label: format!("{target} retained without automation"),
        reason: format!(
            "{}: {}",
            unsupported.source_path,
            unsupported_value(&unsupported.value)
        ),
    }
}

fn unsupported_value(value: &UnsupportedMechanicValue) -> String {
    match value {
        UnsupportedMechanicValue::Source(value) => value.value.clone(),
        UnsupportedMechanicValue::Note(note) => note.value.value.clone(),
        UnsupportedMechanicValue::Capability {
            source_item_type,
            source_slug,
        } => source_slug
            .as_value()
            .map(|slug| format!("{source_item_type}: {slug}"))
            .unwrap_or_else(|| source_item_type.clone()),
        UnsupportedMechanicValue::PreparedSlot(value) => value.value.clone(),
        UnsupportedMechanicValue::ResourceDrift(value) => value.value.value.clone(),
    }
}

pub(super) fn variant_hp_adjustment_delta(
    old_variant: ParticipantVariant,
    new_variant: ParticipantVariant,
    level: Option<i64>,
) -> i64 {
    variant_hp_delta(new_variant, level).unwrap_or(0)
        - variant_hp_delta(old_variant, level).unwrap_or(0)
}

pub(super) fn canonical_creature_level(retrieved: &RetrievedRecord) -> Option<i64> {
    let Some(RecordBody::Creature(creature)) = retrieved.body.as_ref() else {
        return None;
    };
    fact_integer(&creature.level.value)
}

fn apply_participant_effects(
    participant: &EncounterParticipant,
    mechanics: MechanicsView,
    mut unapplied_effects: Vec<UnappliedEffectView>,
    variant_damage_blocked_activity_ids: BTreeSet<String>,
) -> StatBlockView {
    let mut modifiers = variant_modifiers(participant.participant_variant, &mechanics);
    unapplied_effects.extend(variant_unapplied_effects(participant.participant_variant));
    for (condition, rule) in participant_condition_rules(participant) {
        modifiers.extend(condition_modifiers(condition, rule, &mechanics));
        unapplied_effects.extend(condition_unapplied_effects(condition, rule));
    }

    let mut by_target = BTreeMap::<MechanicTarget, Vec<CandidateModifier>>::new();
    for modifier in modifiers {
        by_target
            .entry(modifier.target.clone())
            .or_default()
            .push(modifier);
    }

    let values = mechanics
        .values
        .into_iter()
        .map(|value| {
            let modifiers = by_target.remove(&value.target).unwrap_or_default();
            stat_value_view(value, modifiers)
        })
        .collect();
    unapplied_effects.extend(
        by_target
            .into_values()
            .flatten()
            .map(unmatched_modifier_effect),
    );

    StatBlockView {
        record_key: mechanics.record_key.to_string(),
        title: mechanics.title,
        level: mechanics.level,
        adjusted_level: adjusted_level(mechanics.level, participant.participant_variant),
        values,
        speeds: mechanics
            .speeds
            .into_iter()
            .map(|speed| speed_view(speed, participant))
            .collect(),
        action_budget: Some(action_budget_view(participant)),
        activities: mechanics
            .activities
            .into_iter()
            .map(|activity| {
                let variant_damage_blocked =
                    variant_damage_blocked_activity_ids.contains(&activity.activity_id);
                activity_view(activity, participant, variant_damage_blocked)
            })
            .collect(),
        unapplied_effects,
    }
}

fn speed_view(speed: MovementSpeed, participant: &EncounterParticipant) -> MovementSpeedView {
    let base_value = speed.value_feet;
    let (adjustments, suppressed_adjustments) = speed_adjustments(participant);
    let adjusted_value = apply_runtime_adjustments(base_value, &adjustments);
    let notes = speed_notes(participant);
    MovementSpeedView {
        movement_type: speed.movement_type,
        label: speed.label,
        base_value_feet: base_value,
        adjusted_value_feet: adjusted_value,
        adjustments: adjustments
            .into_iter()
            .map(runtime_adjustment_view)
            .collect(),
        suppressed_adjustments: suppressed_adjustments
            .into_iter()
            .map(runtime_adjustment_view)
            .collect(),
        notes: notes.into_iter().map(runtime_note_view).collect(),
    }
}

fn action_budget_view(participant: &EncounterParticipant) -> ActionBudgetView {
    let action_projection = action_projection(participant);
    let base_actions = 3;
    let base_reactions = 1;
    ActionBudgetView {
        actions: RuntimeCountView {
            label: "Actions".to_string(),
            base_value: base_actions,
            adjusted_value: action_projection.adjusted_actions,
            segments: action_projection.action_segments,
            adjustments: action_projection
                .adjustments
                .into_iter()
                .map(runtime_adjustment_view)
                .collect(),
            suppressed_adjustments: action_projection
                .suppressed_adjustments
                .into_iter()
                .map(runtime_adjustment_view)
                .collect(),
        },
        reactions: RuntimeCountView {
            label: "Reactions".to_string(),
            base_value: base_reactions,
            adjusted_value: base_reactions,
            segments: vec![RuntimeCountSegmentView {
                label: "Base".to_string(),
                value: base_reactions,
                restricted: false,
                reason: None,
            }],
            adjustments: Vec::new(),
            suppressed_adjustments: Vec::new(),
        },
        can_act: action_projection.can_act,
        can_react: action_projection.can_react,
        notes: action_projection
            .notes
            .into_iter()
            .map(runtime_note_view)
            .collect(),
    }
}

struct ActionProjection {
    adjusted_actions: i64,
    action_segments: Vec<RuntimeCountSegmentView>,
    adjustments: Vec<RuntimeAdjustment>,
    suppressed_adjustments: Vec<RuntimeAdjustment>,
    can_act: RuntimeCapabilityView,
    can_react: RuntimeCapabilityView,
    notes: Vec<RuntimeNote>,
}

fn action_projection(participant: &EncounterParticipant) -> ActionProjection {
    let mut quickened = Vec::new();
    let mut slowed = Vec::new();
    let mut stunned = Vec::new();
    let mut notes = Vec::new();
    for (condition, rule) in participant_condition_rules(participant) {
        match rule {
            ConditionRule::Quickened => quickened.push(RuntimeAdjustment {
                source: condition_source(condition),
                label: "Restricted bonus action".to_string(),
                value: 1,
                reason: Some("Use is restricted by the quickened source.".to_string()),
                floor: None,
            }),
            ConditionRule::Slowed => {
                let amount = condition_value(condition);
                let adjustment = RuntimeAdjustment {
                    source: condition_source(condition),
                    label: "Reduced actions regained".to_string(),
                    value: -amount,
                    reason: Some("Applied to the next action-regain step.".to_string()),
                    floor: Some(0),
                };
                slowed.push(adjustment);
            }
            ConditionRule::Stunned => {
                if let Some(amount) = positive_condition_value(condition) {
                    let consumed = amount.min(3);
                    let adjustment = RuntimeAdjustment {
                        source: condition_source(condition),
                        label: "Actions lost while stunned".to_string(),
                        value: -consumed,
                        reason: Some(
                            "Stunned reduces actions regained, then reduces its condition value."
                                .to_string(),
                        ),
                        floor: Some(0),
                    };
                    stunned.push((adjustment, amount - consumed, amount));
                }
                if let Some((label, reason)) = stunned_contextual_disposition(condition) {
                    notes.push(RuntimeNote {
                        source: condition_source(condition),
                        label,
                        reason,
                    });
                }
            }
            ConditionRule::Prone => notes.push(RuntimeNote {
                source: condition_source(condition),
                label: "Prone contextual limits".to_string(),
                reason: "Prone limits movement choices such as Crawl and Stand; cover and falling consequences remain contextual; speed is unchanged."
                    .to_string(),
            }),
            ConditionRule::Immobilized => notes.push(RuntimeNote {
                source: condition_source(condition),
                label: "Move actions forbidden".to_string(),
                reason: "The participant cannot use actions with the move trait; numeric Speed is unchanged."
                    .to_string(),
            }),
            _ => {}
        }
    }

    let mut adjustments = Vec::new();
    let mut suppressed_adjustments = Vec::new();
    quickened.sort_by(|left, right| left.source.cmp(&right.source));
    if let Some(applied_quickened) = quickened.first().cloned() {
        adjustments.push(applied_quickened);
        suppressed_adjustments.extend(quickened.into_iter().skip(1).map(|adjustment| {
            RuntimeAdjustment {
                reason: Some(
                    "Only one restricted bonus action from quickened applies.".to_string(),
                ),
                ..adjustment
            }
        }));
    }

    slowed.sort_by(|left, right| {
        left.value
            .cmp(&right.value)
            .then_with(|| left.source.cmp(&right.source))
    });
    let strongest_slowed = slowed.first().cloned();
    suppressed_adjustments.extend(
        slowed
            .into_iter()
            .skip(1)
            .map(|adjustment| RuntimeAdjustment {
                reason: Some("A stronger slowed penalty applies.".to_string()),
                ..adjustment
            }),
    );

    stunned.sort_by(|left, right| {
        right
            .2
            .cmp(&left.2)
            .then_with(|| left.0.source.cmp(&right.0.source))
    });
    let strongest_stunned = stunned.first().cloned();
    if let Some((adjustment, _, _)) = strongest_stunned.as_ref() {
        notes.push(RuntimeNote {
            source: adjustment.source.clone(),
            label: "Own-turn stunned timing".to_string(),
            reason: "If stunned is applied during this participant's turn, finish the current action or activity, then lose remaining actions immediately to reduce stunned."
                .to_string(),
        });
    }
    suppressed_adjustments.extend(stunned.into_iter().skip(1).map(|(adjustment, _, _)| {
        RuntimeAdjustment {
            reason: Some("A stronger stunned value applies.".to_string()),
            ..adjustment
        }
    }));

    let stunned_remaining = strongest_stunned
        .as_ref()
        .map(|(_, remaining, _)| *remaining);
    match (strongest_stunned, strongest_slowed) {
        (Some((stunned_adjustment, _, _)), Some(slowed_adjustment)) => {
            let stunned_loss = -stunned_adjustment.value;
            let slowed_loss = -slowed_adjustment.value;
            let overlap = stunned_loss.min(slowed_loss);
            let remaining_slowed_loss = slowed_loss - overlap;
            adjustments.push(stunned_adjustment);
            if remaining_slowed_loss > 0 {
                adjustments.push(RuntimeAdjustment {
                    value: -remaining_slowed_loss,
                    reason: Some(format!(
                        "{overlap} action loss already counts toward slowed; {remaining_slowed_loss} additional slowed action loss applies."
                    )),
                    ..slowed_adjustment
                });
            } else {
                suppressed_adjustments.push(RuntimeAdjustment {
                    reason: Some(format!(
                        "All {slowed_loss} slowed action loss is already counted by stunned."
                    )),
                    ..slowed_adjustment
                });
            }
        }
        (Some((stunned_adjustment, _, _)), None) => adjustments.push(stunned_adjustment),
        (None, Some(slowed_adjustment)) => adjustments.push(slowed_adjustment),
        (None, None) => {}
    }

    let adjusted_actions = apply_runtime_adjustments(3, &adjustments);
    let mut action_segments = vec![RuntimeCountSegmentView {
        label: "Base".to_string(),
        value: (adjusted_actions - restricted_bonus_total(&adjustments)).max(0),
        restricted: false,
        reason: None,
    }];
    for adjustment in adjustments.iter().filter(|adjustment| adjustment.value > 0) {
        action_segments.push(RuntimeCountSegmentView {
            label: adjustment.source.clone(),
            value: adjustment.value,
            restricted: adjustment.reason.is_some(),
            reason: adjustment.reason.clone(),
        });
    }

    let stunned_still_active = stunned_remaining.is_some_and(|remaining| remaining > 0);
    let stunned_capability = if stunned_still_active {
        let source = adjustments
            .iter()
            .find(|adjustment| adjustment.source.starts_with("Stunned"))
            .map(|adjustment| adjustment.source.clone());
        RuntimeCapabilityView {
            available: false,
            source,
            reason: Some("Cannot act while stunned remains active.".to_string()),
        }
    } else {
        RuntimeCapabilityView {
            available: true,
            source: None,
            reason: None,
        }
    };

    ActionProjection {
        adjusted_actions,
        action_segments,
        adjustments,
        suppressed_adjustments,
        can_act: stunned_capability.clone(),
        can_react: stunned_capability,
        notes,
    }
}

fn restricted_bonus_total(adjustments: &[RuntimeAdjustment]) -> i64 {
    adjustments
        .iter()
        .filter(|adjustment| adjustment.value > 0)
        .map(|adjustment| adjustment.value)
        .sum()
}

fn apply_runtime_adjustments(base_value: i64, adjustments: &[RuntimeAdjustment]) -> i64 {
    adjustments.iter().fold(base_value, |value, adjustment| {
        let adjusted = value + adjustment.value;
        adjustment
            .floor
            .map_or(adjusted, |floor| adjusted.max(floor.min(base_value)))
    })
}

fn activity_view(
    activity: MechanicActivity,
    participant: &EncounterParticipant,
    variant_damage_blocked: bool,
) -> MechanicActivityView {
    let kind = activity.kind;
    let usage = activity.usage;
    let mut variant_damage_available = !variant_damage_blocked;
    let damage = activity
        .damage
        .into_iter()
        .map(|damage| {
            damage_view(
                damage,
                kind,
                usage,
                participant,
                &mut variant_damage_available,
            )
        })
        .collect();
    let modes = activity
        .modes
        .into_iter()
        .map(|mode| activity_mode_view(mode, kind, usage, participant))
        .collect();
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
        damage,
        modes,
    }
}

fn activity_mode_view(
    mode: MechanicActivityMode,
    activity_kind: MechanicActivityKind,
    activity_usage: MechanicActivityUsage,
    participant: &EncounterParticipant,
) -> MechanicActivityModeView {
    let mut variant_damage_available = true;
    MechanicActivityModeView {
        mode_id: mode.mode_id,
        label: mode.label,
        target: mode.target,
        range: mode.range,
        time: mode.time,
        damage: mode
            .damage
            .into_iter()
            .map(|damage| {
                damage_view(
                    damage,
                    activity_kind,
                    activity_usage,
                    participant,
                    &mut variant_damage_available,
                )
            })
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
        for rule in expanded_condition_rules(rule) {
            modifiers.extend(condition_roll_modifiers(
                condition,
                rule,
                activity_kind,
                &roll,
            ));
        }
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
    variant_damage_available: &mut bool,
) -> DamageExpressionView {
    let apply_variant_damage = *variant_damage_available
        && damage.effect_kind == DamageEffectKind::Damage
        && matches!(
            activity_kind,
            MechanicActivityKind::Strike | MechanicActivityKind::Spell
        );
    if apply_variant_damage {
        *variant_damage_available = false;
    }
    let mut modifiers = variant_damage_modifier(
        participant.participant_variant,
        activity_kind,
        activity_usage,
        damage.effect_kind,
        apply_variant_damage,
    )
    .into_iter()
    .collect::<Vec<_>>();
    for condition in &participant.conditions {
        let Some(rule) = condition_rule_for_key(condition.condition_key.as_deref()) else {
            continue;
        };
        for rule in expanded_condition_rules(rule) {
            modifiers.extend(condition_damage_modifiers(
                condition,
                rule,
                activity_kind,
                &damage,
            ));
        }
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
    activity_kind: MechanicActivityKind,
    usage: MechanicActivityUsage,
    effect_kind: DamageEffectKind,
    apply_variant_damage: bool,
) -> Option<StatModifierView> {
    if !apply_variant_damage || effect_kind != DamageEffectKind::Damage {
        return None;
    }
    let direction = match variant {
        ParticipantVariant::Normal => return None,
        ParticipantVariant::Elite => 1,
        ParticipantVariant::Weak => -1,
    };
    let magnitude = match (activity_kind, usage) {
        (MechanicActivityKind::Strike, _) => 2,
        (MechanicActivityKind::Spell, MechanicActivityUsage::Unlimited) => 2,
        (MechanicActivityKind::Spell, MechanicActivityUsage::Limited) => 4,
        (MechanicActivityKind::Spell, MechanicActivityUsage::Ambiguous)
        | (MechanicActivityKind::Other, _) => return None,
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
        ConditionRule::Prone if roll.surface == ActivityRollSurface::AttackRoll => {
            vec![RollModifier {
                source: source.clone(),
                label: source,
                modifier_type: StatModifierTypeView::Circumstance,
                value: -2,
            }]
        }
        ConditionRule::OffGuard
        | ConditionRule::Fatigued
        | ConditionRule::Clumsy
        | ConditionRule::Enfeebled
        | ConditionRule::Stupefied
        | ConditionRule::Slowed
        | ConditionRule::Quickened
        | ConditionRule::Stunned
        | ConditionRule::Immobilized
        | ConditionRule::Grabbed
        | ConditionRule::Restrained
        | ConditionRule::Encumbered
        | ConditionRule::Prone => Vec::new(),
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
    let adjusted_value = if value.target == MechanicTarget::MaxHp {
        adjusted_value.max(1)
    } else {
        adjusted_value
    };
    StatValueView {
        target: value.target.id(),
        label: value.label,
        base_value,
        adjusted_value,
        modifiers: applied.into_iter().map(modifier_view).collect(),
        suppressed_modifiers: suppressed.into_iter().map(modifier_view).collect(),
    }
}

fn unmatched_modifier_effect(modifier: CandidateModifier) -> UnappliedEffectView {
    UnappliedEffectView {
        source: modifier.source,
        label: format!("{} was not applied", modifier.label),
        reason: format!(
            "No supported base value was available for target {}.",
            modifier.target.id()
        ),
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
        .filter(|value| value.facets.surface.is_check_or_dc())
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
        ParticipantVariant::Weak if level <= 0 => Some(0),
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
        ConditionRule::Fatigued => mechanics
            .values
            .iter()
            .filter(|value| {
                matches!(
                    value.target,
                    MechanicTarget::ArmorClass
                        | MechanicTarget::Save {
                            save: SaveKind::Fortitude | SaveKind::Reflex | SaveKind::Will
                        }
                )
            })
            .map(|value| status_penalty(value.target.clone(), source.clone(), 1))
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
        ConditionRule::Slowed
        | ConditionRule::Quickened
        | ConditionRule::Stunned
        | ConditionRule::Immobilized
        | ConditionRule::Grabbed
        | ConditionRule::Restrained
        | ConditionRule::Encumbered
        | ConditionRule::Prone => Vec::new(),
    }
}

fn speed_adjustments(
    participant: &EncounterParticipant,
) -> (Vec<RuntimeAdjustment>, Vec<RuntimeAdjustment>) {
    let mut penalties = Vec::new();
    for (condition, rule) in participant_condition_rules(participant) {
        let source = condition_source(condition);
        if rule == ConditionRule::Encumbered {
            penalties.push(RuntimeAdjustment {
                source: source.clone(),
                label: "Speed penalty".to_string(),
                value: -10,
                reason: Some(
                    "Encumbered reduces speeds by 10 feet, to a minimum of 5 feet.".to_string(),
                ),
                floor: Some(5),
            });
        }
    }
    penalties.sort_by(|left, right| left.source.cmp(&right.source));

    let mut adjustments = Vec::new();
    let mut suppressed = Vec::new();
    if let Some(penalty) = penalties.first().cloned() {
        adjustments.push(penalty);
        suppressed.extend(
            penalties
                .into_iter()
                .skip(1)
                .map(|adjustment| RuntimeAdjustment {
                    reason: Some("An equivalent speed penalty already applies.".to_string()),
                    ..adjustment
                }),
        );
    }
    (adjustments, suppressed)
}

fn speed_notes(participant: &EncounterParticipant) -> Vec<RuntimeNote> {
    let mut notes = Vec::new();
    for (condition, rule) in participant_condition_rules(participant) {
        let source = condition_source(condition);
        match rule {
            ConditionRule::Grabbed => notes.push(RuntimeNote {
                source: source.clone(),
                label: "Grabbed restrictions".to_string(),
                reason: "Grabbed makes the participant off-guard, forbids move-trait actions through immobilized, and can affect manipulate actions; numeric Speed is unchanged."
                    .to_string(),
            }),
            ConditionRule::Restrained => notes.push(RuntimeNote {
                source: source.clone(),
                label: "Restrained restrictions".to_string(),
                reason:
                    "Restrained makes the participant off-guard, forbids move-trait actions through immobilized, and restricts attack and manipulate actions; numeric Speed is unchanged."
                        .to_string(),
            }),
            ConditionRule::Immobilized => notes.push(RuntimeNote {
                source: source.clone(),
                label: "Move actions forbidden".to_string(),
                reason: "The participant cannot use actions with the move trait; numeric Speed is unchanged."
                    .to_string(),
            }),
            ConditionRule::Prone => notes.push(RuntimeNote {
                source: source.clone(),
                label: "Prone contextual limits".to_string(),
                reason:
                    "Prone limits movement choices such as Crawl and Stand; cover and falling consequences remain contextual; numeric Speed is unchanged."
                        .to_string(),
            }),
            _ => {}
        }
    }
    notes
}

fn condition_unapplied_effects(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
) -> Vec<UnappliedEffectView> {
    let source = condition_source(condition);
    match rule {
        ConditionRule::Fatigued => vec![unapplied_runtime_note_view(
            fatigued_exploration_note(condition),
        )],
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
        ConditionRule::Grabbed => vec![UnappliedEffectView {
            source: source.clone(),
            label: format!("{source} manipulate-action flat check"),
            reason: "Manipulate actions require the Grabbed flat check; that contextual roll and action loss are not automated."
                .to_string(),
        }],
        ConditionRule::Restrained => vec![UnappliedEffectView {
            source: source.clone(),
            label: format!("{source} attack and manipulate restrictions"),
            reason: "Attack and manipulate actions remain unavailable except for the contextual actions that can remove the restraint; those exceptions are not automated."
                .to_string(),
        }],
        ConditionRule::Stunned => stunned_contextual_disposition(condition)
            .map(|(label, reason)| UnappliedEffectView {
                source: source.clone(),
                label,
                reason,
            })
            .into_iter()
            .collect(),
        ConditionRule::Prone => vec![UnappliedEffectView {
            source: source.clone(),
            label: format!("{source} contextual limits"),
            reason: "Prone movement, cover, and falling consequences are tracked as notes rather than numeric adjustments."
                .to_string(),
        }],
        ConditionRule::Frightened
        | ConditionRule::Sickened
        | ConditionRule::OffGuard
        | ConditionRule::Slowed
        | ConditionRule::Quickened
        | ConditionRule::Immobilized
        | ConditionRule::Encumbered => Vec::new(),
    }
}

fn participant_runtime_notes(participant: &EncounterParticipant) -> Vec<RuntimeNote> {
    let mut notes = Vec::new();
    notes.extend(action_projection(participant).notes);
    notes.extend(speed_notes(participant));
    notes.extend(
        participant_condition_rules(participant)
            .filter(|(_, rule)| *rule == ConditionRule::Fatigued)
            .map(|(condition, _)| fatigued_exploration_note(condition)),
    );
    notes
}

fn fatigued_exploration_note(condition: &EncounterParticipantCondition) -> RuntimeNote {
    RuntimeNote {
        source: condition_source(condition),
        label: "Fatigued exploration activity restriction".to_string(),
        reason: "Travel exploration activities are restricted, but exploration context is not automated by this encounter follow-up."
            .to_string(),
    }
}

fn participant_condition_rules(
    participant: &EncounterParticipant,
) -> impl Iterator<Item = (&EncounterParticipantCondition, ConditionRule)> {
    let restrained_present = participant.conditions.iter().any(|condition| {
        condition_rule_for_key(condition.condition_key.as_deref())
            == Some(ConditionRule::Restrained)
    });
    let mut rules = Vec::new();
    let mut suppressed_grabbed = Vec::new();
    for condition in &participant.conditions {
        let Some(rule) = condition_rule_for_key(condition.condition_key.as_deref()) else {
            continue;
        };
        if restrained_present && rule == ConditionRule::Grabbed {
            suppressed_grabbed.push(condition);
            continue;
        }
        rules.extend(
            expanded_condition_rules(rule)
                .into_iter()
                .map(|rule| (condition, rule)),
        );
    }
    rules.extend(
        suppressed_grabbed
            .into_iter()
            .map(|condition| (condition, ConditionRule::OffGuard)),
    );
    rules.into_iter()
}

fn expanded_condition_rules(rule: ConditionRule) -> Vec<ConditionRule> {
    match rule {
        ConditionRule::Grabbed => vec![
            ConditionRule::Grabbed,
            ConditionRule::OffGuard,
            ConditionRule::Immobilized,
        ],
        ConditionRule::Restrained => vec![
            ConditionRule::Restrained,
            ConditionRule::OffGuard,
            ConditionRule::Immobilized,
        ],
        ConditionRule::Encumbered => vec![ConditionRule::Encumbered, ConditionRule::Clumsy],
        ConditionRule::Prone => vec![ConditionRule::Prone, ConditionRule::OffGuard],
        _ => vec![rule],
    }
}

fn runtime_adjustment_view(adjustment: RuntimeAdjustment) -> RuntimeAdjustmentView {
    RuntimeAdjustmentView {
        source: adjustment.source,
        label: adjustment.label,
        value: adjustment.value,
        reason: adjustment.reason,
    }
}

fn runtime_note_view(note: RuntimeNote) -> RuntimeEffectNoteView {
    RuntimeEffectNoteView {
        source: note.source,
        label: note.label,
        reason: note.reason,
    }
}

fn unapplied_runtime_note_view(note: RuntimeNote) -> UnappliedEffectView {
    UnappliedEffectView {
        source: note.source,
        label: note.label,
        reason: note.reason,
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

fn positive_condition_value(condition: &EncounterParticipantCondition) -> Option<i64> {
    condition.value.filter(|value| *value > 0)
}

fn optional_i64(value: Option<i64>) -> String {
    value.map_or_else(|| "missing".to_string(), |value| format!("value({value})"))
}

fn stunned_contextual_disposition(
    condition: &EncounterParticipantCondition,
) -> Option<(String, String)> {
    let numeric_value = positive_condition_value(condition);
    if numeric_value.is_some() && condition.duration_rounds.is_none() {
        return None;
    }
    let action_disposition = if numeric_value.is_some() {
        "the positive numeric value is applied to this action-regain step"
    } else {
        "no numeric action loss is applied"
    };
    Some((
        "Stunned duration/value disposition".to_string(),
        format!(
            "Stunned value={}; duration_rounds={}; {action_disposition}; duration timing and lifecycle remain contextual.",
            optional_i64(condition.value),
            optional_i64(condition.duration_rounds)
        ),
    ))
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
        AtlasRecord, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, MechanicFacets,
        MetricDefinition, MetricRow, MetricValue, RecordClassification, RecordIdentity,
        RecordProvenance, metrics,
    };

    fn project_legacy(
        participant: &EncounterParticipant,
        record: &AtlasRecord,
    ) -> Option<StatBlockView> {
        let mechanics = build_mechanics_view(record)?;
        Some(apply_participant_effects(
            participant,
            mechanics,
            Vec::new(),
            BTreeSet::new(),
        ))
    }

    fn project_canonical(participant: &EncounterParticipant) -> StatBlockView {
        project_canonical_projection(participant, canonical_projection())
    }

    fn project_canonical_projection(
        participant: &EncounterParticipant,
        projection: CanonicalMechanicsProjection,
    ) -> StatBlockView {
        let mechanics =
            canonical_participant_mechanics(projection, "Canonical Creature".to_string());
        apply_participant_effects(
            participant,
            mechanics.view,
            mechanics.unapplied_effects,
            mechanics.variant_damage_blocked_activity_ids,
        )
    }

    fn activity_fact_mut<'a>(
        projection: &'a mut CanonicalMechanicsProjection,
        target: &MechanicTarget,
    ) -> &'a mut MechanicFact {
        projection
            .activities
            .iter_mut()
            .flat_map(|activity| activity.facts.iter_mut())
            .find(|fact| &fact.target == target)
            .expect("canonical activity fact should exist")
    }

    fn canonical_activity_mut<'a>(
        projection: &'a mut CanonicalMechanicsProjection,
        occurrence_id: &str,
    ) -> &'a mut CanonicalMechanicActivity {
        projection
            .activities
            .iter_mut()
            .find(|activity| activity.occurrence_id.as_str() == occurrence_id)
            .expect("canonical activity should exist")
    }

    fn canonical_strike_with_unsupported_first_formula(
        formula: FactValue<String>,
    ) -> CanonicalMechanicsProjection {
        let mut projection = canonical_projection();
        let strike = canonical_activity_mut(&mut projection, "strike-claw");
        let main_index = strike
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. } if damage_id == "main"
                )
            })
            .expect("main damage should exist");
        let main = strike.facts[main_index].clone();

        let mut unsupported_first = main.clone();
        unsupported_first.target = MechanicTarget::ActivityDamage {
            occurrence_id: strike.occurrence_id.clone(),
            damage_id: "unsupported-first".to_string(),
        };
        unsupported_first.label = "unsupported-first".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut unsupported_first.value {
            damage.id = "unsupported-first".to_string();
            damage.formula = formula;
        }
        strike.facts.insert(main_index, unsupported_first);

        let mut secondary = main;
        secondary.target = MechanicTarget::ActivityDamage {
            occurrence_id: strike.occurrence_id.clone(),
            damage_id: "secondary".to_string(),
        };
        secondary.label = "secondary".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut secondary.value {
            damage.id = "secondary".to_string();
            damage.formula = FactValue::Value("1d4".to_string());
        }
        strike.facts.push(secondary);
        projection
    }

    fn assert_unsupported_damage_formula_disposition(
        projection: &StatBlockView,
        expected_formula: &str,
    ) {
        let target = MechanicTarget::ActivityDamage {
            occurrence_id: atlas_record::CreatureOccurrenceId::new("strike-claw")
                .expect("occurrence id should be valid"),
            damage_id: "unsupported-first".to_string(),
        };
        let expected_label = format!("{} retained without exact encounter field", target.id());
        let expected_reason = format!(
            "unsupported-first: damage formula cannot populate a structured encounter damage expression; target={}; id=\"unsupported-first\"; formula={expected_formula}",
            target.id()
        );
        let matching = projection
            .unapplied_effects
            .iter()
            .filter(|effect| effect.label == expected_label)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].source, "Canonical source");
        assert_eq!(matching[0].reason, expected_reason);
    }

    fn canonical_projection() -> CanonicalMechanicsProjection {
        let skill_id =
            atlas_record::CreatureComponentId::new("athletics").expect("skill id should be valid");
        let speed_id =
            atlas_record::CreatureComponentId::new("land").expect("speed id should be valid");
        let resource_id =
            atlas_record::CreatureComponentId::new("focus").expect("resource id should be valid");
        let strike_id = atlas_record::CreatureOccurrenceId::new("strike-claw")
            .expect("occurrence id should be valid");
        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");
        let action_id = atlas_record::CreatureOccurrenceId::new("action-breath")
            .expect("occurrence id should be valid");
        CanonicalMechanicsProjection {
            record_key: RecordKey::parse("actors:canonical").expect("record key should parse"),
            level: FactValue::Value(5),
            facts: vec![
                MechanicFact {
                    target: MechanicTarget::ArmorClass,
                    label: "AC".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(22)),
                    facets: MechanicFacets::armor_class(),
                },
                MechanicFact {
                    target: MechanicTarget::MaxHp,
                    label: "Max HP".to_string(),
                    value: MechanicBaseValue::Number(FactValue::Value(CreatureNumber::Integer(60))),
                    facets: MechanicFacets::hit_points(),
                },
                MechanicFact {
                    target: MechanicTarget::Perception,
                    label: "Perception".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(13)),
                    facets: MechanicFacets::perception(),
                },
                MechanicFact {
                    target: MechanicTarget::Save {
                        save: atlas_record::SaveKind::Reflex,
                    },
                    label: "Reflex".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(12)),
                    facets: MechanicFacets::saving_throw(atlas_record::SaveKind::Reflex),
                },
                MechanicFact {
                    target: MechanicTarget::CreatureSkill {
                        skill_id,
                        kind: atlas_record::CreatureSkillKind::Athletics,
                    },
                    label: "Athletics".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(9)),
                    facets: MechanicFacets::creature_skill(
                        atlas_record::CreatureSkillKind::Athletics,
                    ),
                },
                MechanicFact {
                    target: MechanicTarget::Movement { speed_id },
                    label: "Land Speed".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(25)),
                    facets: MechanicFacets::movement(),
                },
                MechanicFact {
                    target: MechanicTarget::ResourceMaximum { resource_id },
                    label: "Focus".to_string(),
                    value: MechanicBaseValue::ResourceAmount(FactValue::Value(
                        CreatureResourceAmount::Integer(2),
                    )),
                    facets: MechanicFacets::resource(),
                },
            ],
            activities: vec![
                CanonicalMechanicActivity {
                    occurrence_id: strike_id.clone(),
                    family: MechanicActivityFamily::Strike,
                    label: "Claw".to_string(),
                    authored_order: 0,
                    facts: vec![
                        MechanicFact {
                            target: MechanicTarget::ActivityRoll {
                                occurrence_id: strike_id.clone(),
                                roll_id: "attack".to_string(),
                            },
                            label: "Attack".to_string(),
                            value: MechanicBaseValue::Roll(atlas_record::CreatureRoll {
                                id: "attack".to_string(),
                                label: "Attack".to_string(),
                                kind: CreatureRollKind::Attack,
                                value: FactValue::Value(12),
                                ability: FactValue::Value(ActivityRollAbility::Strength),
                            }),
                            facets: MechanicFacets::roll(
                                atlas_record::MechanicSourceFamily::Strike,
                                CreatureRollKind::Attack,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityDamage {
                                occurrence_id: strike_id.clone(),
                                damage_id: "main".to_string(),
                            },
                            label: "main".to_string(),
                            value: MechanicBaseValue::Damage(CreatureDamage {
                                id: "main".to_string(),
                                formula: FactValue::Value("1d6+4".to_string()),
                                damage_type: FactValue::Value("slashing".to_string()),
                                category: FactValue::Missing,
                                kinds: FactValue::Value(vec![CreatureDamageKind::Damage]),
                                apply_modifier: FactValue::Value(CreatureSourceScalar::Value(true)),
                            }),
                            facets: MechanicFacets::damage(
                                atlas_record::MechanicSourceFamily::Strike,
                            ),
                        },
                    ],
                    unsupported: vec![UnsupportedMechanic {
                        target: None,
                        activity_occurrence_id: Some(strike_id),
                        source_path: "activities.strike-claw.unsupported".to_string(),
                        value: UnsupportedMechanicValue::Capability {
                            source_item_type: "unmodeled-rule".to_string(),
                            source_slug: FactValue::Missing,
                        },
                    }],
                },
                CanonicalMechanicActivity {
                    occurrence_id: spellcasting_id.clone(),
                    family: MechanicActivityFamily::SpellcastingEntry,
                    label: "Arcane Prepared Spells".to_string(),
                    authored_order: 1,
                    facts: vec![
                        MechanicFact {
                            target: MechanicTarget::SpellcastingDc {
                                entry_occurrence_id: spellcasting_id.clone(),
                            },
                            label: "Spell DC".to_string(),
                            value: MechanicBaseValue::Integer(FactValue::Value(22)),
                            facets: MechanicFacets::spellcasting_dc(),
                        },
                        MechanicFact {
                            target: MechanicTarget::SpellcastingAttack {
                                entry_occurrence_id: spellcasting_id.clone(),
                            },
                            label: "Spell Attack".to_string(),
                            value: MechanicBaseValue::Integer(FactValue::Value(14)),
                            facets: MechanicFacets::spellcasting_attack(),
                        },
                        MechanicFact {
                            target: MechanicTarget::SpellSlotMaximum {
                                entry_occurrence_id: spellcasting_id.clone(),
                                rank: 3,
                            },
                            label: "Rank 3 slots".to_string(),
                            value: MechanicBaseValue::SourceInteger(FactValue::Value(
                                CreatureSourceScalar::Value(2),
                            )),
                            facets: MechanicFacets::spell_slot(),
                        },
                        MechanicFact {
                            target: MechanicTarget::SpellSlotMaximum {
                                entry_occurrence_id: spellcasting_id.clone(),
                                rank: 1,
                            },
                            label: "Rank 1 slots".to_string(),
                            value: MechanicBaseValue::SourceInteger(FactValue::Value(
                                CreatureSourceScalar::Value(4),
                            )),
                            facets: MechanicFacets::spell_slot(),
                        },
                        MechanicFact {
                            target: MechanicTarget::SpellSlotMaximum {
                                entry_occurrence_id: spellcasting_id,
                                rank: 4,
                            },
                            label: "Rank 4 slots".to_string(),
                            value: MechanicBaseValue::SourceInteger(FactValue::Missing),
                            facets: MechanicFacets::spell_slot(),
                        },
                    ],
                    unsupported: Vec::new(),
                },
                CanonicalMechanicActivity {
                    occurrence_id: action_id.clone(),
                    family: MechanicActivityFamily::Action,
                    label: "Breath Weapon".to_string(),
                    authored_order: 2,
                    facts: vec![
                        MechanicFact {
                            target: MechanicTarget::ActivityActionCost {
                                occurrence_id: action_id.clone(),
                            },
                            label: "Action cost".to_string(),
                            value: MechanicBaseValue::ActionCost(CreatureActionCost::Actions(2)),
                            facets: MechanicFacets::action_economy(
                                atlas_record::MechanicSourceFamily::Action,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityUses {
                                occurrence_id: action_id.clone(),
                            },
                            label: "Uses".to_string(),
                            value: MechanicBaseValue::Uses(FactValue::Value(CreatureUseLimit {
                                maximum: FactValue::Value(2),
                                serialized_value: FactValue::Value(1),
                            })),
                            facets: MechanicFacets::uses(
                                atlas_record::MechanicSourceFamily::Action,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityFrequency {
                                occurrence_id: action_id.clone(),
                            },
                            label: "Frequency".to_string(),
                            value: MechanicBaseValue::Frequency(FactValue::Value(
                                atlas_record::CreatureFrequency {
                                    maximum: FactValue::Value(1),
                                    period: FactValue::Value("day".to_string()),
                                    serialized_value: FactValue::Missing,
                                },
                            )),
                            facets: MechanicFacets::frequency(
                                atlas_record::MechanicSourceFamily::Action,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityRoll {
                                occurrence_id: action_id.clone(),
                                roll_id: "recall".to_string(),
                            },
                            label: "Recall Knowledge".to_string(),
                            value: MechanicBaseValue::Roll(CreatureRoll {
                                id: "recall".to_string(),
                                label: "Recall Knowledge".to_string(),
                                kind: CreatureRollKind::Check,
                                value: FactValue::Value(18),
                                ability: FactValue::Value(ActivityRollAbility::Intelligence),
                            }),
                            facets: MechanicFacets::roll(
                                atlas_record::MechanicSourceFamily::Action,
                                CreatureRollKind::Check,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityDamage {
                                occurrence_id: action_id,
                                damage_id: "fire".to_string(),
                            },
                            label: "fire".to_string(),
                            value: MechanicBaseValue::Damage(CreatureDamage {
                                id: "fire".to_string(),
                                formula: FactValue::Value("4d6".to_string()),
                                damage_type: FactValue::Value("fire".to_string()),
                                category: FactValue::Missing,
                                kinds: FactValue::Value(vec![CreatureDamageKind::Damage]),
                                apply_modifier: FactValue::Value(CreatureSourceScalar::Value(
                                    false,
                                )),
                            }),
                            facets: MechanicFacets::damage(
                                atlas_record::MechanicSourceFamily::Action,
                            ),
                        },
                    ],
                    unsupported: Vec::new(),
                },
            ],
            unsupported: Vec::new(),
        }
    }

    #[test]
    fn canonical_targets_receive_variants_without_adjusting_resources() {
        let projection = project_canonical(&participant(ParticipantVariant::Elite, Vec::new()));

        assert_eq!(projection.adjusted_level, Some(6));
        assert_stat(&projection, "ac", 22, 24, "Elite adjustment");
        assert_stat(&projection, "hp.max", 60, 80, "Elite HP adjustment");
        let resource = projection
            .values
            .iter()
            .find(|value| value.label == "Focus")
            .expect("canonical resource should project");
        assert!(
            resource
                .target
                .starts_with("mechanic-target/v1/resource-maximum/")
        );
        assert_eq!(resource.base_value, 2);
        assert_eq!(resource.adjusted_value, 2);
        assert!(resource.modifiers.is_empty());
        assert_eq!(speed(&projection, "land").base_value_feet, 25);
        assert!(
            projection
                .values
                .iter()
                .all(|value| !value.target.starts_with("skill."))
        );
    }

    #[test]
    fn canonical_weak_hp_adjustment_has_a_minimum_of_one() {
        let mut projection = canonical_projection();
        projection.level = FactValue::Value(1);
        let hp = projection
            .facts
            .iter_mut()
            .find(|fact| fact.target == MechanicTarget::MaxHp)
            .expect("max hp should exist");
        hp.value = MechanicBaseValue::Number(FactValue::Value(CreatureNumber::Integer(5)));
        let mechanics = canonical_participant_mechanics(projection, "Fragile".to_string());
        let block = apply_participant_effects(
            &participant(ParticipantVariant::Weak, Vec::new()),
            mechanics.view,
            mechanics.unapplied_effects,
            mechanics.variant_damage_blocked_activity_ids,
        );

        assert_stat(&block, "hp.max", 5, 1, "Weak HP adjustment");
    }

    #[test]
    fn canonical_weak_hp_bands_cover_every_boundary() {
        let cases = [
            (-1, 100),
            (0, 100),
            (1, 90),
            (2, 90),
            (3, 85),
            (5, 85),
            (6, 80),
            (20, 80),
            (21, 70),
        ];
        for (level, expected_hp) in cases {
            let mut canonical = canonical_projection();
            canonical.level = FactValue::Value(level);
            let hp = canonical
                .facts
                .iter_mut()
                .find(|fact| fact.target == MechanicTarget::MaxHp)
                .expect("max hp should exist");
            hp.value = MechanicBaseValue::Number(FactValue::Value(CreatureNumber::Integer(100)));
            let projection = project_canonical_projection(
                &participant(ParticipantVariant::Weak, Vec::new()),
                canonical,
            );
            assert_eq!(
                value(&projection, "hp.max").adjusted_value,
                expected_hp,
                "unexpected weak HP at starting level {level}"
            );
        }

        let mut level_zero = canonical_projection();
        level_zero.level = FactValue::Value(0);
        let hp = level_zero
            .facts
            .iter_mut()
            .find(|fact| fact.target == MechanicTarget::MaxHp)
            .expect("max hp should exist");
        hp.value = MechanicBaseValue::Number(FactValue::Value(CreatureNumber::Integer(5)));
        let unchanged = project_canonical_projection(
            &participant(ParticipantVariant::Weak, Vec::new()),
            level_zero,
        );
        assert_eq!(value(&unchanged, "hp.max").adjusted_value, 5);
    }

    #[test]
    fn weak_hp_variant_transitions_use_the_starting_level_band() {
        let cases = [
            (ParticipantVariant::Normal, ParticipantVariant::Weak, 5, -15),
            (ParticipantVariant::Weak, ParticipantVariant::Normal, 5, 15),
            (ParticipantVariant::Normal, ParticipantVariant::Weak, 6, -20),
            (ParticipantVariant::Weak, ParticipantVariant::Normal, 6, 20),
            (
                ParticipantVariant::Normal,
                ParticipantVariant::Weak,
                20,
                -20,
            ),
            (ParticipantVariant::Weak, ParticipantVariant::Normal, 20, 20),
            (
                ParticipantVariant::Normal,
                ParticipantVariant::Weak,
                21,
                -30,
            ),
            (ParticipantVariant::Weak, ParticipantVariant::Normal, 21, 30),
            (ParticipantVariant::Weak, ParticipantVariant::Elite, 5, 35),
            (ParticipantVariant::Elite, ParticipantVariant::Weak, 6, -40),
            (ParticipantVariant::Weak, ParticipantVariant::Elite, 20, 50),
            (ParticipantVariant::Elite, ParticipantVariant::Weak, 21, -60),
        ];

        for (old_variant, new_variant, level, expected_delta) in cases {
            assert_eq!(
                variant_hp_adjustment_delta(old_variant, new_variant, Some(level)),
                expected_delta,
                "unexpected {old_variant:?} -> {new_variant:?} HP delta at starting level {level}"
            );
        }
    }

    #[test]
    fn canonical_condition_targets_stack_and_suppress_by_type() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Frightened", Some(1)),
                condition("Sickened", Some(2)),
                condition("Off-Guard", None),
            ],
        ));

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
        let skill = projection
            .values
            .iter()
            .find(|value| value.label == "Athletics")
            .expect("canonical skill should project");
        assert_eq!(skill.adjusted_value, 7);
        assert!(
            skill
                .target
                .starts_with("mechanic-target/v1/creature-skill/")
        );
    }

    #[test]
    fn fatigued_applies_fixed_penalty_only_to_ac_and_all_saves() {
        let projection = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Fatigued", Some(9))],
            ),
            &record(),
        )
        .expect("stat block");

        for (target, base) in [
            ("ac", 22),
            ("save.fort", 15),
            ("save.ref", 12),
            ("save.will", 12),
        ] {
            let stat = value(&projection, target);
            assert_eq!(stat.adjusted_value, base - 1, "unexpected {target}");
            assert_eq!(stat.modifiers.len(), 1, "unexpected {target}");
            assert!(stat.modifiers.iter().any(|modifier| {
                modifier.source == "Fatigued 9"
                    && modifier.modifier_type == StatModifierTypeView::Status
                    && modifier.value == -1
            }));
        }

        for target in [
            "hp.max",
            "perception",
            "skill.athletics",
            "ability.str",
            "ability.dex",
        ] {
            let stat = value(&projection, target);
            assert_eq!(stat.adjusted_value, stat.base_value, "unexpected {target}");
            assert!(stat.modifiers.is_empty(), "unexpected {target}");
            assert!(stat.suppressed_modifiers.is_empty(), "unexpected {target}");
        }
        for speed in &projection.speeds {
            assert_eq!(speed.adjusted_value_feet, speed.base_value_feet);
            assert!(speed.adjustments.is_empty());
            assert!(speed.suppressed_adjustments.is_empty());
            assert!(speed.notes.is_empty());
        }
        let budget = projection.action_budget.as_ref().expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 3);
        assert!(budget.actions.adjustments.is_empty());
        assert!(budget.actions.suppressed_adjustments.is_empty());
        assert!(budget.notes.is_empty());
        for activity in &projection.activities {
            for roll in &activity.rolls {
                assert!(
                    roll.modifiers
                        .iter()
                        .chain(&roll.suppressed_modifiers)
                        .all(|modifier| modifier.source != "Fatigued 9")
                );
            }
            for damage in &activity.damage {
                assert!(
                    damage
                        .modifiers
                        .iter()
                        .all(|modifier| modifier.source != "Fatigued 9")
                );
            }
            for mode in &activity.modes {
                for damage in &mode.damage {
                    assert!(
                        damage
                            .modifiers
                            .iter()
                            .all(|modifier| modifier.source != "Fatigued 9")
                    );
                }
            }
        }

        let canonical = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![condition("Fatigued", None)],
        ));
        let resource = canonical
            .values
            .iter()
            .find(|value| value.label == "Focus")
            .expect("canonical resource should project");
        assert_eq!(resource.adjusted_value, resource.base_value);
        assert!(resource.modifiers.is_empty());
        assert!(resource.suppressed_modifiers.is_empty());

        let note = projection
            .unapplied_effects
            .iter()
            .find(|effect| effect.label == "Fatigued exploration activity restriction")
            .expect("travel restriction should remain explicit");
        assert_eq!(note.source, "Fatigued 9");
        assert_eq!(
            note.reason,
            "Travel exploration activities are restricted, but exploration context is not automated by this encounter follow-up."
        );
    }

    #[test]
    fn fatigued_uses_existing_status_stacking_and_suppression() {
        let projection = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![
                    condition("Fatigued", None),
                    condition("Frightened", Some(2)),
                    condition("Off-Guard", None),
                ],
            ),
            &record(),
        )
        .expect("stat block");

        for target in ["ac", "save.fort", "save.ref", "save.will"] {
            let stat = value(&projection, target);
            assert!(stat.modifiers.iter().any(|modifier| {
                modifier.source == "Frightened 2"
                    && modifier.modifier_type == StatModifierTypeView::Status
                    && modifier.value == -2
            }));
            assert!(stat.suppressed_modifiers.iter().any(|modifier| {
                modifier.source == "Fatigued"
                    && modifier.modifier_type == StatModifierTypeView::Status
                    && modifier.value == -1
            }));
        }
        let ac = value(&projection, "ac");
        assert!(ac.modifiers.iter().any(|modifier| {
            modifier.source == "Off-Guard"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
                && modifier.value == -2
        }));
        let perception = value(&projection, "perception");
        assert!(
            perception
                .modifiers
                .iter()
                .chain(&perception.suppressed_modifiers)
                .all(|modifier| modifier.source != "Fatigued")
        );
    }

    #[test]
    fn fatigued_requires_the_canonical_key_and_preserves_note_order() {
        let mut first = condition("Fatigued", None);
        first.name = "First fatigue".to_string();
        let mut second = condition("Fatigued", None);
        second.name = "Second fatigue".to_string();
        let annotation = unmodeled_condition("Fatigued", None);

        let ordered = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![first.clone(), annotation.clone(), second.clone()],
            ),
            &record(),
        )
        .expect("stat block");
        assert_eq!(
            ordered
                .unapplied_effects
                .iter()
                .filter(|effect| effect.label == "Fatigued exploration activity restriction")
                .map(|effect| effect.source.as_str())
                .collect::<Vec<_>>(),
            vec!["First fatigue", "Second fatigue"]
        );
        assert_eq!(value(&ordered, "ac").adjusted_value, 21);

        let reversed = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![second.clone(), first.clone()],
            ),
            &record(),
        )
        .expect("stat block");
        assert_eq!(
            reversed
                .unapplied_effects
                .iter()
                .filter(|effect| effect.label == "Fatigued exploration activity restriction")
                .map(|effect| effect.source.as_str())
                .collect::<Vec<_>>(),
            vec!["Second fatigue", "First fatigue"]
        );

        first.condition_key = None;
        let key_mutation = project_legacy(
            &participant(ParticipantVariant::Normal, vec![first, second]),
            &record(),
        )
        .expect("stat block");
        assert_eq!(
            key_mutation
                .unapplied_effects
                .iter()
                .filter(|effect| effect.label == "Fatigued exploration activity restriction")
                .map(|effect| effect.source.as_str())
                .collect::<Vec<_>>(),
            vec!["Second fatigue"]
        );

        let annotation_only = project_legacy(
            &participant(ParticipantVariant::Normal, vec![annotation]),
            &record(),
        )
        .expect("stat block");
        assert_eq!(value(&annotation_only, "ac").adjusted_value, 22);
        assert!(annotation_only.unapplied_effects.is_empty());
    }

    #[test]
    fn runtime_only_fatigued_keeps_context_without_changing_actions() {
        let projection = participant_runtime_block(&participant(
            ParticipantVariant::Normal,
            vec![condition("Fatigued", None)],
        ));
        let budget = projection.action_budget.as_ref().expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 3);
        assert!(budget.actions.adjustments.is_empty());
        assert_eq!(projection.unapplied_effects.len(), 1);
        assert_eq!(
            projection.unapplied_effects[0].label,
            "Fatigued exploration activity restriction"
        );
        assert_eq!(
            projection.unapplied_effects[0].reason,
            "Travel exploration activities are restricted, but exploration context is not automated by this encounter follow-up."
        );
    }

    #[test]
    fn canonical_activity_targets_keep_roll_damage_and_unapplied_context() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Elite,
            vec![condition("Enfeebled", Some(2))],
        ));

        assert_roll(
            &projection,
            "strike-claw",
            "attack",
            12,
            12,
            "Elite adjustment",
        );
        assert!(
            roll(&projection, "strike-claw", "attack")
                .modifiers
                .iter()
                .any(|modifier| modifier.label == "Enfeebled 2")
        );
        let strike_damage = damage(&projection, "strike-claw", "main");
        assert_eq!(strike_damage.adjusted_formula, None);
        assert_eq!(strike_damage.modifiers.len(), 2);
        assert_no_damage_modifier(&projection, "action-breath", "fire");
        assert!(projection.unapplied_effects.iter().any(|effect| {
            effect.reason.contains("activities.strike-claw.unsupported")
                && effect.reason.contains("unmodeled-rule")
        }));
    }

    #[test]
    fn canonical_activity_facts_have_exact_existing_surface_or_unapplied_dispositions() {
        let projection = project_canonical(&participant(ParticipantVariant::Normal, Vec::new()));
        let action_id = atlas_record::CreatureOccurrenceId::new("action-breath")
            .expect("occurrence id should be valid");
        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");
        let action = projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == action_id.as_str())
            .expect("action should project");
        assert_eq!(action.usage, MechanicActivityUsageView::Limited);
        assert!(
            action.rolls.iter().all(|roll| roll.roll_id != "recall"),
            "a check must not be mislabeled as the existing attack or DC surface"
        );

        let expected_unapplied = [
            (
                MechanicTarget::ActivityActionCost {
                    occurrence_id: action_id.clone(),
                },
                "Action cost: action cost has no exact encounter activity field; value=actions(2)",
            ),
            (
                MechanicTarget::ActivityUses {
                    occurrence_id: action_id.clone(),
                },
                "Uses: uses are represented only by coarse activity usage; value=maximum=value(2), serialized_value=value(1)",
            ),
            (
                MechanicTarget::ActivityFrequency {
                    occurrence_id: action_id.clone(),
                },
                "Frequency: frequency is represented only by coarse activity usage; value=maximum=value(1), period=value(\"day\"), serialized_value=missing",
            ),
            (
                MechanicTarget::ActivityRoll {
                    occurrence_id: action_id,
                    roll_id: "recall".to_string(),
                },
                "Recall Knowledge: check roll has no encounter roll surface; id=\"recall\"; label=\"Recall Knowledge\"; value=value(18); ability=value(intelligence)",
            ),
        ];
        for (target, reason) in expected_unapplied {
            let label = format!("{} retained without exact encounter field", target.id());
            assert!(projection.unapplied_effects.iter().any(|effect| {
                effect.source == "Canonical source"
                    && effect.label == label
                    && effect.reason == reason
            }));
        }

        let slot_values = projection
            .values
            .iter()
            .filter(|value| {
                value
                    .target
                    .starts_with("mechanic-target/v1/spell-slot-maximum/")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            slot_values
                .iter()
                .map(|value| (value.label.as_str(), value.base_value, value.adjusted_value))
                .collect::<Vec<_>>(),
            vec![("Rank 3 slots", 2, 2), ("Rank 1 slots", 4, 4)],
            "representable spell-slot maxima should retain canonical fact order"
        );
        let missing_slot = MechanicTarget::SpellSlotMaximum {
            entry_occurrence_id: spellcasting_id,
            rank: 4,
        };
        assert!(projection.unapplied_effects.iter().any(|effect| {
            effect.label
                == format!(
                    "{} retained without exact encounter field",
                    missing_slot.id()
                )
                && effect.reason
                    == "Rank 4 slots: spell-slot maximum cannot populate a numeric encounter value; value=missing"
        }));
    }

    #[test]
    fn canonical_unapplied_activity_facts_preserve_order_and_duplicates() {
        let mut canonical = canonical_projection();
        let action_id = atlas_record::CreatureOccurrenceId::new("action-breath")
            .expect("occurrence id should be valid");
        let action = canonical
            .activities
            .iter_mut()
            .find(|activity| activity.occurrence_id == action_id)
            .expect("action should exist");
        let mut duplicate_uses = action
            .facts
            .iter()
            .find(|fact| matches!(fact.target, MechanicTarget::ActivityUses { .. }))
            .expect("uses fact should exist")
            .clone();
        duplicate_uses.label = "Secondary uses".to_string();
        duplicate_uses.value = MechanicBaseValue::Uses(FactValue::Value(CreatureUseLimit {
            maximum: FactValue::Value(3),
            serialized_value: FactValue::Value(2),
        }));
        action.facts.insert(2, duplicate_uses);

        let projection = project_canonical_projection(
            &participant(ParticipantVariant::Normal, Vec::new()),
            canonical,
        );
        let action_prefixes = [
            "mechanic-target/v1/activity-action-cost/",
            "mechanic-target/v1/activity-uses/",
            "mechanic-target/v1/activity-frequency/",
            "mechanic-target/v1/activity-roll/",
        ];
        let reasons = projection
            .unapplied_effects
            .iter()
            .filter(|effect| {
                action_prefixes
                    .iter()
                    .any(|prefix| effect.label.starts_with(prefix))
            })
            .map(|effect| effect.reason.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            reasons,
            vec![
                "Action cost: action cost has no exact encounter activity field; value=actions(2)",
                "Uses: uses are represented only by coarse activity usage; value=maximum=value(2), serialized_value=value(1)",
                "Secondary uses: uses are represented only by coarse activity usage; value=maximum=value(3), serialized_value=value(2)",
                "Frequency: frequency is represented only by coarse activity usage; value=maximum=value(1), period=value(\"day\"), serialized_value=missing",
                "Recall Knowledge: check roll has no encounter roll surface; id=\"recall\"; label=\"Recall Knowledge\"; value=value(18); ability=value(intelligence)",
            ]
        );
    }

    #[test]
    fn canonical_activity_fact_mutations_change_their_exact_dispositions() {
        let normal = participant(ParticipantVariant::Normal, Vec::new());
        let action_id = atlas_record::CreatureOccurrenceId::new("action-breath")
            .expect("occurrence id should be valid");
        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");

        let mut check = canonical_projection();
        activity_fact_mut(
            &mut check,
            &MechanicTarget::ActivityRoll {
                occurrence_id: action_id.clone(),
                roll_id: "recall".to_string(),
            },
        )
        .value = MechanicBaseValue::Roll(CreatureRoll {
            id: "recall".to_string(),
            label: "Recall Knowledge".to_string(),
            kind: CreatureRollKind::Check,
            value: FactValue::Null,
            ability: FactValue::Value(ActivityRollAbility::Wisdom),
        });
        assert!(
            project_canonical_projection(&normal, check)
                .unapplied_effects
                .iter()
                .any(|effect| effect.reason.contains("value=null; ability=value(wisdom)"))
        );

        let mut action_cost_projection = canonical_projection();
        activity_fact_mut(
            &mut action_cost_projection,
            &MechanicTarget::ActivityActionCost {
                occurrence_id: action_id.clone(),
            },
        )
        .value = MechanicBaseValue::ActionCost(CreatureActionCost::Reaction);
        assert!(
            project_canonical_projection(&normal, action_cost_projection)
                .unapplied_effects
                .iter()
                .any(|effect| effect.reason.ends_with("value=reaction"))
        );

        let mut uses_projection = canonical_projection();
        activity_fact_mut(
            &mut uses_projection,
            &MechanicTarget::ActivityUses {
                occurrence_id: action_id.clone(),
            },
        )
        .value = MechanicBaseValue::Uses(FactValue::Null);
        assert!(
            project_canonical_projection(&normal, uses_projection)
                .unapplied_effects
                .iter()
                .any(|effect| effect.reason.ends_with("value=null"))
        );

        let mut frequency_projection = canonical_projection();
        activity_fact_mut(
            &mut frequency_projection,
            &MechanicTarget::ActivityFrequency {
                occurrence_id: action_id,
            },
        )
        .value = MechanicBaseValue::Frequency(FactValue::Value(CreatureFrequency {
            maximum: FactValue::Value(2),
            period: FactValue::Value("round".to_string()),
            serialized_value: FactValue::Value(1),
        }));
        assert!(
            project_canonical_projection(&normal, frequency_projection)
                .unapplied_effects
                .iter()
                .any(|effect| effect.reason.contains(
                    "maximum=value(2), period=value(\"round\"), serialized_value=value(1)"
                ))
        );

        let mut slot_projection = canonical_projection();
        activity_fact_mut(
            &mut slot_projection,
            &MechanicTarget::SpellSlotMaximum {
                entry_occurrence_id: spellcasting_id,
                rank: 3,
            },
        )
        .value = MechanicBaseValue::SourceInteger(FactValue::Value(CreatureSourceScalar::Value(5)));
        let projection = project_canonical_projection(&normal, slot_projection);
        let slot = projection
            .values
            .iter()
            .find(|value| value.label == "Rank 3 slots")
            .expect("mutated spell-slot maximum should remain represented");
        assert_eq!((slot.base_value, slot.adjusted_value), (5, 5));
    }

    #[test]
    fn canonical_ability_and_spell_targets_receive_deterministic_conditions() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Clumsy", Some(1)),
                condition("Enfeebled", Some(2)),
                condition("Stupefied", Some(1)),
            ],
        ));

        assert_eq!(value(&projection, "save.ref").adjusted_value, 11);
        assert_eq!(value(&projection, "perception").adjusted_value, 12);
        let athletics = projection
            .values
            .iter()
            .find(|value| value.label == "Athletics")
            .expect("athletics should project");
        assert_eq!(athletics.adjusted_value, 7);
        assert_roll(&projection, "strike-claw", "attack", 12, 10, "Enfeebled 2");
        let spellcasting = projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == "spellcasting-arcane")
            .expect("spellcasting entry should project");
        assert_eq!(spellcasting.rolls[0].base_value, 22);
        assert_eq!(spellcasting.rolls[0].adjusted_value, 21);
        assert!(
            spellcasting.rolls[0]
                .modifiers
                .iter()
                .any(|modifier| modifier.label == "Stupefied 1")
        );
        assert_damage_modifier(&projection, "strike-claw", "main", -2);
    }

    #[test]
    fn canonical_variant_damage_adjusts_only_first_melee_or_spell_damage() {
        let mut strike_projection = canonical_projection();
        let strike = canonical_activity_mut(&mut strike_projection, "strike-claw");
        let main = strike
            .facts
            .iter()
            .find(|fact| matches!(fact.target, MechanicTarget::ActivityDamage { .. }))
            .expect("strike damage should exist")
            .clone();
        let mut unsupported_first = main.clone();
        unsupported_first.target = MechanicTarget::ActivityDamage {
            occurrence_id: strike.occurrence_id.clone(),
            damage_id: "unsupported-first".to_string(),
        };
        unsupported_first.label = "unsupported-first".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut unsupported_first.value {
            damage.id = "unsupported-first".to_string();
            damage.formula = FactValue::Value("contextual damage".to_string());
            damage.kinds = FactValue::Value(vec![CreatureDamageKind::Unsupported(
                UnsupportedSourceValue {
                    shape: UnsupportedSourceShape::String,
                    value: "\"contextual\"".to_string(),
                    reason: UnsupportedSourceReason::OpenVocabulary,
                },
            )]);
        }
        let main_index = strike
            .facts
            .iter()
            .position(|fact| matches!(fact.target, MechanicTarget::ActivityDamage { .. }))
            .expect("main damage should exist");
        strike.facts.insert(main_index, unsupported_first);
        let mut secondary = main;
        secondary.target = MechanicTarget::ActivityDamage {
            occurrence_id: strike.occurrence_id.clone(),
            damage_id: "secondary".to_string(),
        };
        secondary.label = "secondary".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut secondary.value {
            damage.id = "secondary".to_string();
            damage.formula = FactValue::Value("1d4".to_string());
        }
        strike.facts.push(secondary);
        let elite_strike = project_canonical_projection(
            &participant(ParticipantVariant::Elite, Vec::new()),
            strike_projection,
        );
        assert_no_damage_modifier(&elite_strike, "strike-claw", "unsupported-first");
        assert_eq!(
            damage(&elite_strike, "strike-claw", "unsupported-first").adjusted_formula,
            None
        );
        assert_damage_modifier(&elite_strike, "strike-claw", "main", 2);
        assert_no_damage_modifier(&elite_strike, "strike-claw", "secondary");
        assert!(
            elite_strike
                .unapplied_effects
                .iter()
                .any(|effect| { effect.label == "Elite ambiguous offensive damage adjustments" })
        );

        let mut limited_spell_projection = canonical_projection();
        let limited = canonical_activity_mut(&mut limited_spell_projection, "action-breath");
        limited.family = MechanicActivityFamily::Spell;
        let mut secondary = limited
            .facts
            .iter()
            .find(|fact| matches!(fact.target, MechanicTarget::ActivityDamage { .. }))
            .expect("spell damage should exist")
            .clone();
        secondary.target = MechanicTarget::ActivityDamage {
            occurrence_id: limited.occurrence_id.clone(),
            damage_id: "cold".to_string(),
        };
        secondary.label = "cold".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut secondary.value {
            damage.id = "cold".to_string();
            damage.formula = FactValue::Value("2d6".to_string());
            damage.damage_type = FactValue::Value("cold".to_string());
        }
        limited.facts.push(secondary);
        let elite_limited = project_canonical_projection(
            &participant(ParticipantVariant::Elite, Vec::new()),
            limited_spell_projection.clone(),
        );
        assert_damage_modifier(&elite_limited, "action-breath", "fire", 4);
        assert_no_damage_modifier(&elite_limited, "action-breath", "cold");

        let at_will = canonical_activity_mut(&mut limited_spell_projection, "action-breath");
        at_will.facts.retain(|fact| {
            !matches!(
                fact.target,
                MechanicTarget::ActivityUses { .. } | MechanicTarget::ActivityFrequency { .. }
            )
        });
        let fire_index = at_will
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. } if damage_id == "fire"
                )
            })
            .expect("fire damage should exist");
        let cold_index = at_will
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. } if damage_id == "cold"
                )
            })
            .expect("cold damage should exist");
        at_will.facts.swap(fire_index, cold_index);
        let weak_at_will = project_canonical_projection(
            &participant(ParticipantVariant::Weak, Vec::new()),
            limited_spell_projection,
        );
        assert_damage_modifier(&weak_at_will, "action-breath", "cold", -2);
        assert_no_damage_modifier(&weak_at_will, "action-breath", "fire");
    }

    #[test]
    fn unsupported_first_damage_formula_retains_exact_ordered_disposition() {
        for (formula, expected_formula) in
            [(FactValue::Missing, "missing"), (FactValue::Null, "null")]
        {
            let projection = canonical_strike_with_unsupported_first_formula(formula);
            let projected = project_canonical_projection(
                &participant(ParticipantVariant::Elite, Vec::new()),
                projection,
            );
            let strike = projected
                .activities
                .iter()
                .find(|activity| activity.activity_id == "strike-claw")
                .expect("strike should project");
            assert_eq!(
                strike
                    .damage
                    .iter()
                    .map(|damage| damage.damage_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["main", "secondary"]
            );
            assert_no_damage_modifier(&projected, "strike-claw", "main");
            assert_no_damage_modifier(&projected, "strike-claw", "secondary");
            assert_unsupported_damage_formula_disposition(&projected, expected_formula);
        }

        let mut reordered = canonical_strike_with_unsupported_first_formula(FactValue::Missing);
        let strike = canonical_activity_mut(&mut reordered, "strike-claw");
        let unsupported_index = strike
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. }
                        if damage_id == "unsupported-first"
                )
            })
            .expect("unsupported damage should exist");
        let main_index = strike
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. } if damage_id == "main"
                )
            })
            .expect("main damage should exist");
        strike.facts.swap(unsupported_index, main_index);

        let projected = project_canonical_projection(
            &participant(ParticipantVariant::Elite, Vec::new()),
            reordered,
        );
        assert_damage_modifier(&projected, "strike-claw", "main", 2);
        assert_no_damage_modifier(&projected, "strike-claw", "secondary");
        assert_unsupported_damage_formula_disposition(&projected, "missing");
    }

    #[test]
    fn canonical_prone_applies_off_guard_and_typed_attack_penalty_only() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Prone", None),
                condition("Prone", None),
                condition("Frightened", Some(1)),
            ],
        ));
        assert_eq!(value(&projection, "ac").adjusted_value, 19);
        assert_eq!(value(&projection, "ac").modifiers.len(), 2);
        assert_eq!(value(&projection, "ac").suppressed_modifiers.len(), 1);
        let attack = roll(&projection, "strike-claw", "attack");
        assert_eq!(attack.adjusted_value, 9);
        assert!(attack.modifiers.iter().any(|modifier| {
            modifier.source == "Prone"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
                && modifier.value == -2
        }));
        assert!(attack.modifiers.iter().any(|modifier| {
            modifier.source == "Frightened 1"
                && modifier.modifier_type == StatModifierTypeView::Status
                && modifier.value == -1
        }));
        assert!(attack.suppressed_modifiers.iter().any(|modifier| {
            modifier.source == "Prone"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
        }));
        let spell_dc = projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == "spellcasting-arcane")
            .and_then(|activity| activity.rolls.iter().find(|roll| roll.label == "Spell DC"))
            .expect("spell DC should project");
        assert_eq!(spell_dc.adjusted_value, 21);
        assert!(
            spell_dc
                .modifiers
                .iter()
                .all(|modifier| modifier.source != "Prone")
        );
        let spell_attack = projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == "spellcasting-arcane")
            .and_then(|activity| {
                activity
                    .rolls
                    .iter()
                    .find(|roll| roll.label == "Spell Attack")
            })
            .expect("spell attack should project");
        assert_eq!(spell_attack.surface, ActivityRollSurfaceView::AttackRoll);
        assert_eq!(spell_attack.base_value, 14);
        assert_eq!(spell_attack.adjusted_value, 11);
        assert!(spell_attack.modifiers.iter().any(|modifier| {
            modifier.source == "Prone"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
                && modifier.value == -2
        }));
        assert!(spell_attack.modifiers.iter().any(|modifier| {
            modifier.source == "Frightened 1"
                && modifier.modifier_type == StatModifierTypeView::Status
                && modifier.value == -1
        }));
        assert!(spell_attack.suppressed_modifiers.iter().any(|modifier| {
            modifier.source == "Prone"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
        }));
        assert!(projection.unapplied_effects.iter().any(|effect| {
            effect
                .reason
                .contains("check roll has no encounter roll surface")
                && effect.reason.contains("Recall Knowledge")
        }));
        let action_notes = &projection
            .action_budget
            .as_ref()
            .expect("action budget")
            .notes;
        assert!(action_notes.iter().any(|note| {
            note.source == "Prone"
                && note.reason.contains("cover")
                && note.reason.contains("falling")
        }));

        let mut dc_mutation = canonical_projection();
        let strike_id = atlas_record::CreatureOccurrenceId::new("strike-claw")
            .expect("occurrence id should be valid");
        let roll_fact = activity_fact_mut(
            &mut dc_mutation,
            &MechanicTarget::ActivityRoll {
                occurrence_id: strike_id,
                roll_id: "attack".to_string(),
            },
        );
        if let MechanicBaseValue::Roll(roll) = &mut roll_fact.value {
            roll.kind = CreatureRollKind::DifficultyClass;
        }
        let dc_projection = project_canonical_projection(
            &participant(ParticipantVariant::Normal, vec![condition("Prone", None)]),
            dc_mutation,
        );
        let mutated = roll(&dc_projection, "strike-claw", "attack");
        assert_eq!(mutated.adjusted_value, 12);
        assert!(mutated.modifiers.is_empty());
    }

    #[test]
    fn canonical_runtime_rules_explain_action_and_movement_suppression() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Quickened", None),
                condition("Quickened", None),
                condition("Slowed", Some(1)),
                condition("Slowed", Some(2)),
                condition("Grabbed", None),
                condition("Encumbered", None),
            ],
        ));

        let budget = projection.action_budget.as_ref().expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 2);
        assert_eq!(budget.actions.adjustments.len(), 2);
        assert_eq!(budget.actions.suppressed_adjustments.len(), 2);
        let land = speed(&projection, "land");
        assert_eq!(land.adjusted_value_feet, 15);
        assert_eq!(land.adjustments.len(), 1);
        assert!(land.suppressed_adjustments.is_empty());
        assert!(
            budget
                .notes
                .iter()
                .any(|note| { note.source == "Grabbed" && note.label == "Move actions forbidden" })
        );
    }

    #[test]
    fn participant_projection_never_falls_back_to_sparse_record_mechanics() {
        let retrieved = RetrievedRecord {
            record: record(),
            body: None,
        };

        assert!(
            participant_stat_block(
                &participant(ParticipantVariant::Elite, Vec::new()),
                &retrieved,
            )
            .is_none()
        );
    }

    #[test]
    fn elite_adjusts_projected_creature_stats_and_hp_by_level_band() {
        let participant = participant(ParticipantVariant::Elite, Vec::new());
        let projection = project_legacy(&participant, &record()).expect("stat block");
        assert_eq!(projection.adjusted_level, Some(6));
        assert_stat(&projection, "ac", 22, 24, "Elite adjustment");
        assert_stat(&projection, "hp.max", 60, 80, "Elite HP adjustment");
    }

    #[test]
    fn weak_adjusts_projected_creature_stats_and_hp_by_level_band() {
        let participant = participant(ParticipantVariant::Weak, Vec::new());
        let projection = project_legacy(&participant, &record()).expect("stat block");
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
        let projection = project_legacy(&participant, &record()).expect("stat block");
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
        let projection = project_legacy(&participant, &record()).expect("stat block");
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
        let projection = project_legacy(&participant, &record()).expect("stat block");

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
        let projection = project_legacy(&participant, &record()).expect("stat block");

        assert_eq!(value(&projection, "ac").adjusted_value, 22);
        assert_eq!(value(&projection, "perception").adjusted_value, 13);
        assert_eq!(value(&projection, "skill.athletics").adjusted_value, 9);
    }

    #[test]
    fn elite_and_weak_project_structured_activity_damage_adjustments() {
        let elite = project_legacy(
            &participant(ParticipantVariant::Elite, Vec::new()),
            &record(),
        )
        .expect("stat block");
        assert_damage_modifier(&elite, "claw", "main", 2);
        assert_no_damage_modifier(&elite, "claw", "secondary");
        assert_mode_damage_modifier(&elite, "claw", "sweep", "mode-primary", 2);
        assert_no_mode_damage_modifier(&elite, "claw", "sweep", "mode-persistent");
        assert_eq!(
            damage(&elite, "claw", "main").adjusted_formula.as_deref(),
            Some("1d6 + 6")
        );
        assert_damage_modifier(&elite, "fireball", "0", 4);
        assert_no_damage_modifier(&elite, "fireball", "persistent");
        assert_damage_modifier(&elite, "ignition", "fire", 2);
        assert_no_damage_modifier(&elite, "ignition", "persistent");
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

        let weak = project_legacy(
            &participant(ParticipantVariant::Weak, Vec::new()),
            &record(),
        )
        .expect("stat block");
        assert_damage_modifier(&weak, "claw", "main", -2);
        assert_no_damage_modifier(&weak, "claw", "secondary");
        assert_mode_damage_modifier(&weak, "claw", "sweep", "mode-primary", -2);
        assert_no_mode_damage_modifier(&weak, "claw", "sweep", "mode-persistent");
        assert_damage_modifier(&weak, "fireball", "0", -4);
        assert_no_damage_modifier(&weak, "fireball", "persistent");
        assert_damage_modifier(&weak, "ignition", "fire", -2);
        assert_no_damage_modifier(&weak, "ignition", "persistent");
        assert_no_damage_modifier(&weak, "breath", "0");
        assert_no_damage_modifier(&weak, "heal", "0");
    }

    #[test]
    fn activity_roll_surfaces_receive_variant_and_condition_modifiers() {
        let elite = project_legacy(
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
        let projection = project_legacy(
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

    #[test]
    fn action_conditions_project_runtime_action_budget() {
        let slowed = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Slowed", Some(1))],
            ),
            &record(),
        )
        .expect("stat block");
        let slowed_budget = slowed.action_budget.as_ref().expect("action budget");
        assert_eq!(slowed_budget.actions.adjusted_value, 2);
        assert!(slowed_budget.can_react.available);

        let quickened = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Quickened", None)],
            ),
            &record(),
        )
        .expect("stat block");
        let quickened_budget = quickened.action_budget.as_ref().expect("action budget");
        assert_eq!(quickened_budget.actions.adjusted_value, 4);
        assert!(
            quickened_budget
                .actions
                .segments
                .iter()
                .any(|segment| segment.restricted && segment.value == 1)
        );

        let stunned_one = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Stunned", Some(1))],
            ),
            &record(),
        )
        .expect("stat block");
        let stunned_one_budget = stunned_one.action_budget.as_ref().expect("action budget");
        assert_eq!(stunned_one_budget.actions.adjusted_value, 2);
        assert!(stunned_one_budget.can_react.available);

        let stunned_four = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Stunned", Some(4))],
            ),
            &record(),
        )
        .expect("stat block");
        let stunned_four_budget = stunned_four.action_budget.as_ref().expect("action budget");
        assert_eq!(stunned_four_budget.actions.adjusted_value, 0);
        assert!(!stunned_four_budget.can_act.available);
        assert!(!stunned_four_budget.can_react.available);

        let stunned_and_slowed = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Stunned", Some(1)), condition("Slowed", Some(2))],
            ),
            &record(),
        )
        .expect("stat block");
        let budget = stunned_and_slowed
            .action_budget
            .as_ref()
            .expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 1);
        assert!(
            budget
                .actions
                .adjustments
                .iter()
                .any(|adjustment| { adjustment.source == "Stunned 1" && adjustment.value == -1 })
        );
        assert!(budget.actions.adjustments.iter().any(|adjustment| {
            adjustment.source == "Slowed 2"
                && adjustment.value == -1
                && adjustment.reason.as_deref()
                    == Some(
                        "1 action loss already counts toward slowed; 1 additional slowed action loss applies.",
                    )
        }));
        assert!(
            budget
                .actions
                .suppressed_adjustments
                .iter()
                .all(|adjustment| adjustment.source != "Slowed 2")
        );

        let stronger_stunned = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Slowed", Some(1)), condition("Stunned", Some(2))],
            ),
            &record(),
        )
        .expect("stat block");
        let stronger_budget = stronger_stunned
            .action_budget
            .as_ref()
            .expect("action budget");
        assert_eq!(stronger_budget.actions.adjusted_value, 1);
        assert!(
            stronger_budget
                .actions
                .adjustments
                .iter()
                .any(|adjustment| { adjustment.source == "Stunned 2" && adjustment.value == -2 })
        );
        assert!(
            stronger_budget
                .actions
                .suppressed_adjustments
                .iter()
                .any(|adjustment| {
                    adjustment.source == "Slowed 1"
                        && adjustment.reason.as_deref()
                            == Some("All 1 slowed action loss is already counted by stunned.")
                })
        );

        let duplicate_conditions = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![
                    condition("Quickened", None),
                    condition("Quickened", None),
                    condition("Slowed", Some(1)),
                    condition("Slowed", Some(2)),
                ],
            ),
            &record(),
        )
        .expect("stat block");
        let duplicate_budget = duplicate_conditions
            .action_budget
            .as_ref()
            .expect("action budget");
        assert_eq!(duplicate_budget.actions.adjusted_value, 2);
        assert_eq!(duplicate_budget.actions.adjustments.len(), 2);
        assert_eq!(duplicate_budget.actions.suppressed_adjustments.len(), 2);
    }

    #[test]
    fn duration_form_stunned_is_retained_without_value_coercion() {
        let duration_only = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition_with_duration("Stunned", None, Some(2))],
            ),
            &record(),
        )
        .expect("stat block");
        let budget = duration_only.action_budget.as_ref().expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 3);
        assert!(budget.actions.adjustments.is_empty());
        assert!(budget.can_act.available);
        let expected_duration_reason = "Stunned value=missing; duration_rounds=value(2); no numeric action loss is applied; duration timing and lifecycle remain contextual.";
        assert!(budget.notes.iter().any(|note| {
            note.source == "Stunned"
                && note.label == "Stunned duration/value disposition"
                && note.reason == expected_duration_reason
        }));
        assert!(duration_only.unapplied_effects.iter().any(|effect| {
            effect.source == "Stunned"
                && effect.label == "Stunned duration/value disposition"
                && effect.reason == expected_duration_reason
        }));

        let numeric = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Stunned", Some(1))],
            ),
            &record(),
        )
        .expect("stat block");
        assert_eq!(
            numeric
                .action_budget
                .as_ref()
                .expect("action budget")
                .actions
                .adjusted_value,
            2
        );
        assert!(
            numeric
                .unapplied_effects
                .iter()
                .all(|effect| { effect.label != "Stunned duration/value disposition" })
        );

        let mut duration_mutation = condition_with_duration("Stunned", None, Some(2));
        duration_mutation.duration_rounds = Some(4);
        let mutated = project_legacy(
            &participant(ParticipantVariant::Normal, vec![duration_mutation]),
            &record(),
        )
        .expect("stat block");
        assert_eq!(
            mutated
                .action_budget
                .as_ref()
                .expect("action budget")
                .actions
                .adjusted_value,
            3
        );
        assert!(mutated.unapplied_effects.iter().any(|effect| {
            effect.reason == "Stunned value=missing; duration_rounds=value(4); no numeric action loss is applied; duration timing and lifecycle remain contextual."
        }));
        assert!(
            mutated
                .unapplied_effects
                .iter()
                .all(|effect| effect.reason != expected_duration_reason)
        );

        let invalid_numeric = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition_with_duration("Stunned", Some(0), None)],
            ),
            &record(),
        )
        .expect("stat block");
        assert_eq!(
            invalid_numeric
                .action_budget
                .as_ref()
                .expect("action budget")
                .actions
                .adjusted_value,
            3
        );
        assert!(invalid_numeric.unapplied_effects.iter().any(|effect| {
            effect.reason == "Stunned value=value(0); duration_rounds=missing; no numeric action loss is applied; duration timing and lifecycle remain contextual."
        }));

        let value_and_duration = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition_with_duration("Stunned", Some(2), Some(3))],
            ),
            &record(),
        )
        .expect("stat block");
        assert_eq!(
            value_and_duration
                .action_budget
                .as_ref()
                .expect("action budget")
                .actions
                .adjusted_value,
            1
        );
        assert!(value_and_duration.unapplied_effects.iter().any(|effect| {
            effect.reason == "Stunned value=value(2); duration_rounds=value(3); the positive numeric value is applied to this action-regain step; duration timing and lifecycle remain contextual."
        }));
    }

    #[test]
    fn restrained_overrides_grabbed_independent_of_condition_order() {
        for conditions in [
            vec![condition("Grabbed", None), condition("Restrained", None)],
            vec![condition("Restrained", None), condition("Grabbed", None)],
        ] {
            let projection = project_legacy(
                &participant(ParticipantVariant::Normal, conditions),
                &record(),
            )
            .expect("stat block");
            let ac = value(&projection, "ac");
            assert_eq!(ac.adjusted_value, 20);
            assert!(ac.modifiers.iter().any(|modifier| {
                modifier.source == "Restrained"
                    && modifier.modifier_type == StatModifierTypeView::Circumstance
                    && modifier.value == -2
            }));
            assert!(ac.suppressed_modifiers.iter().any(|modifier| {
                modifier.source == "Grabbed"
                    && modifier.modifier_type == StatModifierTypeView::Circumstance
                    && modifier.value == -2
            }));
            assert!(projection.unapplied_effects.iter().any(|effect| {
                effect.source == "Restrained"
                    && effect.label == "Restrained attack and manipulate restrictions"
            }));
            assert!(
                projection
                    .unapplied_effects
                    .iter()
                    .all(|effect| effect.source != "Grabbed")
            );
            let budget = projection.action_budget.as_ref().expect("action budget");
            assert!(budget.notes.iter().any(|note| {
                note.source == "Restrained" && note.label == "Move actions forbidden"
            }));
            assert!(budget.notes.iter().all(|note| note.source != "Grabbed"));
            assert!(
                speed(&projection, "land")
                    .notes
                    .iter()
                    .all(|note| note.source != "Grabbed")
            );
        }

        let grabbed_only = project_legacy(
            &participant(ParticipantVariant::Normal, vec![condition("Grabbed", None)]),
            &record(),
        )
        .expect("stat block");
        assert!(
            value(&grabbed_only, "ac")
                .modifiers
                .iter()
                .any(|modifier| modifier.source == "Grabbed")
        );
        assert!(grabbed_only.unapplied_effects.iter().any(|effect| {
            effect.source == "Grabbed" && effect.label == "Grabbed manipulate-action flat check"
        }));
    }

    #[test]
    fn movement_conditions_project_speeds_and_chained_effects() {
        let encumbered = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Encumbered", None)],
            ),
            &record(),
        )
        .expect("stat block");
        assert_eq!(speed(&encumbered, "land").adjusted_value_feet, 15);
        assert_eq!(speed(&encumbered, "fly").adjusted_value_feet, 5);
        assert_eq!(value(&encumbered, "save.ref").adjusted_value, 11);

        let grabbed = project_legacy(
            &participant(ParticipantVariant::Normal, vec![condition("Grabbed", None)]),
            &record(),
        )
        .expect("stat block");
        assert_eq!(speed(&grabbed, "land").adjusted_value_feet, 25);
        assert_eq!(value(&grabbed, "ac").adjusted_value, 20);
        assert!(
            grabbed
                .action_budget
                .as_ref()
                .expect("action budget")
                .notes
                .iter()
                .any(|note| note.source == "Grabbed" && note.label == "Move actions forbidden")
        );

        let grabbed_and_encumbered = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Grabbed", None), condition("Encumbered", None)],
            ),
            &record(),
        )
        .expect("stat block");
        let land = speed(&grabbed_and_encumbered, "land");
        assert_eq!(land.adjusted_value_feet, 15);
        assert_eq!(land.adjustments.len(), 1);
        assert!(land.suppressed_adjustments.is_empty());

        for condition_name in ["Immobilized", "Grabbed", "Restrained"] {
            let restricted = project_legacy(
                &participant(
                    ParticipantVariant::Normal,
                    vec![condition(condition_name, None)],
                ),
                &record(),
            )
            .expect("stat block");
            assert_eq!(speed(&restricted, "land").adjusted_value_feet, 25);
            assert_eq!(speed(&restricted, "fly").adjusted_value_feet, 10);
            assert!(speed(&restricted, "land").notes.iter().any(|note| {
                note.source == condition_name
                    && note.label == "Move actions forbidden"
                    && note.reason.contains("numeric Speed is unchanged")
            }));
            assert!(
                restricted
                    .action_budget
                    .as_ref()
                    .expect("action budget")
                    .notes
                    .iter()
                    .any(|note| {
                        note.source == condition_name
                            && note.label == "Move actions forbidden"
                            && note.reason.contains("move trait")
                    })
            );
            match condition_name {
                "Grabbed" => assert!(restricted.unapplied_effects.iter().any(|effect| {
                    effect.source == "Grabbed"
                        && effect.label == "Grabbed manipulate-action flat check"
                })),
                "Restrained" => assert!(restricted.unapplied_effects.iter().any(|effect| {
                    effect.source == "Restrained"
                        && effect.label == "Restrained attack and manipulate restrictions"
                })),
                _ => {}
            }
        }
        let unrestricted = project_legacy(
            &participant(ParticipantVariant::Normal, Vec::new()),
            &record(),
        )
        .expect("stat block");
        assert!(
            unrestricted
                .action_budget
                .as_ref()
                .expect("action budget")
                .notes
                .iter()
                .all(|note| note.label != "Move actions forbidden")
        );

        let prone = project_legacy(
            &participant(ParticipantVariant::Normal, vec![condition("Prone", None)]),
            &record(),
        )
        .expect("stat block");
        assert_eq!(speed(&prone, "land").adjusted_value_feet, 25);
        assert_eq!(value(&prone, "ac").adjusted_value, 20);
        assert!(
            prone
                .unapplied_effects
                .iter()
                .any(|effect| effect.label == "Prone contextual limits")
        );

        let zero_speed = project_legacy(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Encumbered", None)],
            ),
            &record_with_speeds(&[("land", 0), ("fly", 10)]),
        )
        .expect("stat block");
        assert!(
            zero_speed
                .speeds
                .iter()
                .all(|speed| speed.movement_type != "land")
        );
        assert_eq!(speed(&zero_speed, "fly").adjusted_value_feet, 5);
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

    fn speed<'a>(projection: &'a StatBlockView, movement_type: &str) -> &'a MovementSpeedView {
        projection
            .speeds
            .iter()
            .find(|speed| speed.movement_type == movement_type)
            .expect("speed should exist")
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

    fn condition_with_duration(
        name: &str,
        value: Option<i64>,
        duration_rounds: Option<i64>,
    ) -> EncounterParticipantCondition {
        EncounterParticipantCondition {
            duration_rounds,
            ..condition(name, value)
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
            "slowed" => "conditionitems:xYTAsEpcJE1Ccni3",
            "quickened" => "conditionitems:nlCjDvLMf2EkV2dl",
            "stunned" => "conditionitems:dfCMdR4wnpbYNTix",
            "immobilized" => "conditionitems:eIcWbB5o3pP6OIMe",
            "grabbed" => "conditionitems:kWc1fhmv9LBiTuei",
            "restrained" => "conditionitems:VcDeM8A5oI6VqhbM",
            "encumbered" => "conditionitems:D5mg6Tc7Jzrj6ro7",
            "prone" => "conditionitems:j91X7x0XSomq8d60",
            "fatigued" => "conditionitems:HL2l2VRSaQHu9lUw",
            _ => "conditionitems:unsupported",
        }
    }

    fn record() -> AtlasRecord {
        record_with_speeds(&[("land", 25), ("fly", 10)])
    }

    fn record_with_speeds(speeds: &[(&str, i64)]) -> AtlasRecord {
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
        record.mechanics.metrics.extend(
            speeds
                .iter()
                .map(|(speed, value)| metric(&format!("speed.{speed}.value"), *value as f64)),
        );
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
                damage: vec![
                    atlas_record::DamageExpression {
                        damage_id: "main".to_string(),
                        label: None,
                        formula: "1d6+4".to_string(),
                        damage_type: Some("slashing".to_string()),
                        effect_kind: atlas_record::DamageEffectKind::Damage,
                        ability: Some(atlas_record::ActivityRollAbility::Strength),
                    },
                    atlas_record::DamageExpression {
                        damage_id: "secondary".to_string(),
                        label: Some("Persistent bleed".to_string()),
                        formula: "1d4".to_string(),
                        damage_type: Some("bleed".to_string()),
                        effect_kind: atlas_record::DamageEffectKind::Damage,
                        ability: None,
                    },
                ],
                modes: vec![atlas_record::MechanicActivityMode {
                    mode_id: "sweep".to_string(),
                    label: "Claw sweep".to_string(),
                    sort: 1,
                    target: None,
                    range: None,
                    time: None,
                    damage: vec![
                        atlas_record::DamageExpression {
                            damage_id: "mode-primary".to_string(),
                            label: None,
                            formula: "3d6".to_string(),
                            damage_type: Some("slashing".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: None,
                        },
                        atlas_record::DamageExpression {
                            damage_id: "mode-persistent".to_string(),
                            label: Some("Persistent bleed".to_string()),
                            formula: "1d6".to_string(),
                            damage_type: Some("bleed".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: None,
                        },
                    ],
                }],
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
                damage: vec![
                    atlas_record::DamageExpression {
                        damage_id: "0".to_string(),
                        label: None,
                        formula: "6d6".to_string(),
                        damage_type: Some("fire".to_string()),
                        effect_kind: atlas_record::DamageEffectKind::Damage,
                        ability: None,
                    },
                    atlas_record::DamageExpression {
                        damage_id: "persistent".to_string(),
                        label: Some("Persistent fire".to_string()),
                        formula: "1d6".to_string(),
                        damage_type: Some("fire".to_string()),
                        effect_kind: atlas_record::DamageEffectKind::Damage,
                        ability: None,
                    },
                ],
                modes: Vec::new(),
            },
            atlas_record::MechanicActivity {
                activity_id: "ignition".to_string(),
                label: "Ignition".to_string(),
                kind: atlas_record::MechanicActivityKind::Spell,
                traits: vec!["cantrip".to_string()],
                compendium_source: None,
                usage: atlas_record::MechanicActivityUsage::Unlimited,
                rolls: Vec::new(),
                damage: vec![
                    atlas_record::DamageExpression {
                        damage_id: "fire".to_string(),
                        label: None,
                        formula: "2d4".to_string(),
                        damage_type: Some("fire".to_string()),
                        effect_kind: atlas_record::DamageEffectKind::Damage,
                        ability: None,
                    },
                    atlas_record::DamageExpression {
                        damage_id: "persistent".to_string(),
                        label: Some("Persistent fire".to_string()),
                        formula: "1d4".to_string(),
                        damage_type: Some("fire".to_string()),
                        effect_kind: atlas_record::DamageEffectKind::Damage,
                        ability: None,
                    },
                ],
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
