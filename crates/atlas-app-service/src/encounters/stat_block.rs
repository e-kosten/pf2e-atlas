//! Internal mutable overlay workspace; it is not a transport or source authority.
pub(crate) use super::rules::modifiers::{Modifier, ModifierType};
use atlas_app_model::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatBlockView {
    pub(crate) source_actions: super::rules::source_action_conditions::SourceActionConditions,
    pub record_key: String,
    pub title: String,
    pub level: Option<serde_json::Number>,
    pub adjusted_level: Option<serde_json::Number>,
    pub values: Vec<StatValueView>,
    pub replacements: Vec<StatReplacement>,
    pub unresolved_stats: Vec<StatisticTarget>,
    pub senses: Vec<EncounterSenseView>,
    pub senses_known: bool,
    pub sense_facts: Vec<super::rules::senses::SenseFacts>,
    pub contextual_armor_class: Vec<EncounterContextualArmorClassView>,
    pub speeds: Vec<MovementSpeedView>,
    pub action_budget: Option<ActionBudgetView>,
    pub activities: Vec<MechanicActivityView>,
    pub unapplied_effects: Vec<UnappliedEffectView>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MovementSpeedView {
    pub movement_type: String,
    pub label: String,
    pub base_value_feet: serde_json::Number,
    pub adjusted_value_feet: serde_json::Number,
    pub adjustments: Vec<RuntimeAdjustmentView>,
    pub suppressed_adjustments: Vec<RuntimeAdjustmentView>,
    pub notes: Vec<RuntimeEffectNoteView>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatValueView {
    pub target: StatisticTarget,
    pub label: String,
    pub ability: Option<Ability>,
    pub base_value: serde_json::Number,
    pub adjusted_value: serde_json::Number,
    pub modifiers: Vec<Modifier>,
    pub suppressed_modifiers: Vec<Modifier>,
    pub notes: Vec<RuntimeEffectNoteView>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MechanicActivityView {
    pub traits_known: bool,
    pub calculation_unavailable: Option<String>,
    pub is_ranged: Option<bool>,
    pub attack_ability: Option<Ability>,
    pub difficulty_class: Option<StatValueView>,
    pub spell_usage: Option<SpellUsage>,
    pub activity_id: String,
    pub label: String,
    pub kind: MechanicActivityKindView,
    pub traits: Vec<String>,
    pub navigation: RecordNavigationView,
    pub notes: Vec<RuntimeEffectNoteView>,
    pub rolls: Vec<ActivityRollView>,
    pub damage: Vec<DamageExpressionView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MechanicActivityKindView {
    Strike,
    Spell,
    CastingEntry,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActivityRollView {
    pub roll_id: String,
    pub label: String,
    pub base_value: serde_json::Number,
    pub adjusted_value: serde_json::Number,
    pub surface: ActivityRollSurfaceView,
    pub modifiers: Vec<Modifier>,
    pub suppressed_modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActivityRollSurfaceView {
    AttackRoll,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DamageExpressionView {
    pub ability: Option<Ability>,
    pub ability_known: bool,
    pub unavailable_reason: Option<String>,
    pub source_ordinal: usize,
    pub damage_id: String,
    pub label: Option<String>,
    pub formula: String,
    pub adjusted_formula: Option<String>,
    pub damage_type: Option<String>,
    pub category: Option<String>,
    pub effect_kind: DamageEffectKindView,
    pub modifiers: Vec<Modifier>,
    pub suppressed_modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DamageEffectKindView {
    Damage,
    Healing,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}
impl Ability {
    pub(crate) fn from_key(key: &str) -> Option<Self> {
        match key {
            "str" => Some(Self::Strength),
            "dex" => Some(Self::Dexterity),
            "con" => Some(Self::Constitution),
            "int" => Some(Self::Intelligence),
            "wis" => Some(Self::Wisdom),
            "cha" => Some(Self::Charisma),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpellUsage {
    AtWill,
    Limited,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatReplacement {
    pub target: StatisticTarget,
    pub value: serde_json::Number,
    pub condition_id: i64,
    pub condition_name: String,
}
impl StatBlockView {
    pub(crate) fn effective_stat(&self, target: &StatisticTarget) -> Option<&serde_json::Number> {
        if self
            .unresolved_stats
            .iter()
            .any(|unresolved| unresolved == target)
        {
            return None;
        }
        self.replacements
            .iter()
            .find(|replacement| &replacement.target == target)
            .map(|replacement| &replacement.value)
            .or_else(|| {
                self.values
                    .iter()
                    .find(|value| &value.target == target)
                    .map(|value| &value.adjusted_value)
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatisticSave {
    Fortitude,
    Reflex,
    Will,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StatisticTarget {
    ArmorClass,
    HitPointMaximum,
    Perception,
    Save(StatisticSave),
    Skill(String),
    Ability(Ability),
    HazardStealth,
    Hardness,
    SpellDc,
}
impl StatisticTarget {
    pub(crate) fn key(&self) -> std::borrow::Cow<'_, str> {
        use std::borrow::Cow;
        match self {
            Self::ArmorClass => Cow::Borrowed("ac"),
            Self::HitPointMaximum => Cow::Borrowed("hp.max"),
            Self::Perception => Cow::Borrowed("perception"),
            Self::Save(StatisticSave::Fortitude) => Cow::Borrowed("save.fortitude"),
            Self::Save(StatisticSave::Reflex) => Cow::Borrowed("save.reflex"),
            Self::Save(StatisticSave::Will) => Cow::Borrowed("save.will"),
            Self::Skill(key) => Cow::Owned(format!("skill.{key}")),
            Self::Ability(ability) => Cow::Owned(format!("ability.{}", ability.key())),
            Self::HazardStealth => Cow::Borrowed("hazard.stealth"),
            Self::Hardness => Cow::Borrowed("hazard.hardness"),
            Self::SpellDc => Cow::Borrowed("spell.dc"),
        }
    }
    pub(crate) fn check_or_dc(&self) -> bool {
        matches!(
            self,
            Self::ArmorClass
                | Self::Perception
                | Self::Save(_)
                | Self::Skill(_)
                | Self::HazardStealth
                | Self::SpellDc
        )
    }
    pub(crate) fn skill(&self) -> bool {
        matches!(self, Self::Skill(_))
    }
    pub(crate) fn save(&self) -> bool {
        matches!(self, Self::Save(_))
    }
}
impl std::fmt::Display for StatisticTarget {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.key())
    }
}
impl Ability {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Strength => "str",
            Self::Dexterity => "dex",
            Self::Constitution => "con",
            Self::Intelligence => "int",
            Self::Wisdom => "wis",
            Self::Charisma => "cha",
        }
    }
}
