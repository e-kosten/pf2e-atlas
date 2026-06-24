use std::collections::BTreeMap;

use atlas_app_model::{
    DamageExpressionView, EncounterParticipantVariantView, MechanicActivityKindView,
    MechanicActivityUsageView, MechanicActivityView, StatBlockView, StatModifierTypeView,
    StatModifierView, StatValueView, UnappliedEffectView,
};
use atlas_local_state::{EncounterParticipant, EncounterParticipantCondition, ParticipantVariant};
use atlas_record::{
    AbilityKind, DamageExpression, MechanicActivity, MechanicActivityKind, MechanicActivityUsage,
    MechanicScalar, MechanicSurface, MechanicTarget, MechanicValue, MechanicsView,
    build_mechanics_view,
};

use super::projection::participant_variant_view;

#[derive(Debug, Clone)]
struct CandidateModifier {
    target: MechanicTarget,
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
        let Some(condition_rule) = ConditionRule::from_condition(condition) else {
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
            .map(|activity| activity_view(activity, participant.participant_variant))
            .collect(),
        unapplied_effects,
    }
}

fn activity_view(activity: MechanicActivity, variant: ParticipantVariant) -> MechanicActivityView {
    let damage_modifier = variant_damage_modifier(variant, activity.usage);
    MechanicActivityView {
        activity_id: activity.activity_id,
        label: activity.label,
        kind: activity_kind_view(activity.kind),
        usage: activity_usage_view(activity.usage),
        damage: activity
            .damage
            .into_iter()
            .map(|damage| damage_view(damage, damage_modifier.clone()))
            .collect(),
    }
}

fn damage_view(
    damage: DamageExpression,
    modifier: Option<StatModifierView>,
) -> DamageExpressionView {
    DamageExpressionView {
        damage_id: damage.damage_id,
        label: damage.label,
        formula: damage.formula,
        damage_type: damage.damage_type,
        modifiers: modifier.into_iter().collect(),
    }
}

fn variant_damage_modifier(
    variant: ParticipantVariant,
    usage: MechanicActivityUsage,
) -> Option<StatModifierView> {
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

fn activity_kind_view(kind: MechanicActivityKind) -> MechanicActivityKindView {
    match kind {
        MechanicActivityKind::Strike => MechanicActivityKindView::Strike,
        MechanicActivityKind::Spell => MechanicActivityKindView::Spell,
        MechanicActivityKind::Other => MechanicActivityKindView::Other,
    }
}

fn activity_usage_view(usage: MechanicActivityUsage) -> MechanicActivityUsageView {
    match usage {
        MechanicActivityUsage::Unlimited => MechanicActivityUsageView::Unlimited,
        MechanicActivityUsage::Limited => MechanicActivityUsageView::Limited,
        MechanicActivityUsage::Ambiguous => MechanicActivityUsageView::Ambiguous,
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

#[derive(Debug, Clone, Copy)]
enum ConditionRule {
    Frightened,
    Sickened,
    OffGuard,
    Clumsy,
    Enfeebled,
    Stupefied,
}

impl ConditionRule {
    fn from_condition(condition: &EncounterParticipantCondition) -> Option<Self> {
        match condition_slug(condition).as_str() {
            "frightened" => Some(Self::Frightened),
            "sickened" => Some(Self::Sickened),
            "off-guard" | "offguard" => Some(Self::OffGuard),
            "clumsy" => Some(Self::Clumsy),
            "enfeebled" => Some(Self::Enfeebled),
            "stupefied" => Some(Self::Stupefied),
            _ => None,
        }
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
            label: format!("{source} ranged attack penalty"),
            reason: "Ranged attack modifiers are not typed yet.".to_string(),
        }],
        ConditionRule::Enfeebled => vec![UnappliedEffectView {
            source: source.clone(),
            label: format!("{source} melee attack and damage penalty"),
            reason: "Strength-based attacks and damage are not typed yet.".to_string(),
        }],
        ConditionRule::Stupefied => vec![UnappliedEffectView {
            source: source.clone(),
            label: format!("{source} spellcasting penalty"),
            reason: "Spell attacks, spell DCs, and flat-check disruption are not typed yet."
                .to_string(),
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

fn condition_slug(condition: &EncounterParticipantCondition) -> String {
    slugify(&condition.name)
}

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
    fn elite_and_weak_project_structured_activity_damage_adjustments() {
        let elite = participant_stat_block(
            &participant(ParticipantVariant::Elite, Vec::new()),
            &record(),
        )
        .expect("stat block");
        assert_damage_modifier(&elite, "claw", "main", 2);
        assert_damage_modifier(&elite, "fireball", "0", 4);
        assert_no_damage_modifier(&elite, "breath", "0");

        let weak = participant_stat_block(
            &participant(ParticipantVariant::Weak, Vec::new()),
            &record(),
        )
        .expect("stat block");
        assert_damage_modifier(&weak, "claw", "main", -2);
        assert_damage_modifier(&weak, "fireball", "0", -4);
        assert_no_damage_modifier(&weak, "breath", "0");
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
            condition_key: Some(format!("conditionitems:{}", slugify(name))),
            name: name.to_string(),
            value,
            source_participant_key: None,
            duration_rounds: None,
            note: None,
            source_note: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
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
                damage: vec![atlas_record::DamageExpression {
                    damage_id: "main".to_string(),
                    label: None,
                    formula: "1d6+4".to_string(),
                    damage_type: Some("slashing".to_string()),
                }],
            },
            atlas_record::MechanicActivity {
                activity_id: "fireball".to_string(),
                label: "Fireball".to_string(),
                kind: atlas_record::MechanicActivityKind::Spell,
                traits: Vec::new(),
                compendium_source: None,
                usage: atlas_record::MechanicActivityUsage::Limited,
                damage: vec![atlas_record::DamageExpression {
                    damage_id: "0".to_string(),
                    label: None,
                    formula: "6d6".to_string(),
                    damage_type: Some("fire".to_string()),
                }],
            },
            atlas_record::MechanicActivity {
                activity_id: "breath".to_string(),
                label: "Breath Weapon".to_string(),
                kind: atlas_record::MechanicActivityKind::Other,
                traits: Vec::new(),
                compendium_source: None,
                usage: atlas_record::MechanicActivityUsage::Ambiguous,
                damage: vec![atlas_record::DamageExpression {
                    damage_id: "0".to_string(),
                    label: None,
                    formula: "4d6".to_string(),
                    damage_type: Some("fire".to_string()),
                }],
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
