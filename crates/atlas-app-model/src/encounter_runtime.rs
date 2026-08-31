use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::EncounterParticipantVariantView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub level: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub vitals: Option<EncounterRuntimeVitalsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub defenses: Option<EncounterRuntimeDefensesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub saves: Option<EncounterRuntimeSavesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub awareness: Option<EncounterRuntimeAwarenessView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub abilities: Option<EncounterRuntimeAbilitiesView>,
    pub skills: Vec<EncounterRuntimeSkillView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub movement: Option<EncounterRuntimeMovementView>,
    pub resources: Vec<EncounterRuntimeResourceView>,
    pub spellcasting: Vec<EncounterRuntimeSpellcastingView>,
    pub activities: Vec<EncounterRuntimeActivityView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub action_budget: Option<EncounterRuntimeActionBudgetView>,
    pub conditions: Vec<EncounterRuntimeConditionView>,
    pub automation_limitations: Vec<EncounterRuntimeAutomationLimitationView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeVitalsView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub maximum_hp: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeDefensesView {
    pub armor_class: RuntimeNumberView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeSavesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fortitude: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reflex: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub will: Option<RuntimeNumberView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeAwarenessView {
    pub perception: RuntimeNumberView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeAbilitiesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub strength: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub dexterity: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub constitution: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub intelligence: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub wisdom: Option<RuntimeNumberView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub charisma: Option<RuntimeNumberView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeSkillView {
    pub skill_id: String,
    pub label: String,
    pub kind: EncounterRuntimeSkillKindView,
    pub modifier: RuntimeNumberView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(tag = "kind", rename_all = "snake_case")]
pub enum EncounterRuntimeSkillKindView {
    Standard { slug: String },
    Lore { slug: String },
    Legacy { slug: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeMovementView {
    pub speeds: Vec<RuntimeDistanceView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeDistanceView {
    pub movement_type: String,
    pub label: String,
    pub base_value_feet: i64,
    pub adjusted_value_feet: i64,
    pub adjustments: Vec<RuntimeAdjustmentView>,
    pub suppressed_adjustments: Vec<RuntimeAdjustmentView>,
    pub notes: Vec<RuntimeEffectNoteView>,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeResourceView {
    pub resource_id: String,
    pub label: String,
    pub maximum: RuntimeNumberView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current: Option<RuntimeNumberView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeSpellcastingView {
    pub entry_id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub attack: Option<RuntimeRollView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub dc: Option<RuntimeRollView>,
    pub slots: Vec<EncounterRuntimeSpellSlotView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeSpellSlotView {
    pub rank: i64,
    pub maximum: RuntimeCountView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeNumberView {
    pub label: String,
    pub base_value: i64,
    pub adjusted_value: i64,
    pub modifiers: Vec<RuntimeModifierView>,
    pub suppressed_modifiers: Vec<RuntimeModifierView>,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum StatModifierTypeView {
    Adjustment,
    Status,
    Circumstance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeModifierView {
    pub provenance: RuntimeFactProvenanceView,
    pub label: String,
    pub modifier_type: StatModifierTypeView,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeAdjustmentView {
    pub provenance: RuntimeFactProvenanceView,
    pub label: String,
    pub value: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCountView {
    pub label: String,
    pub base_value: i64,
    pub adjusted_value: i64,
    pub segments: Vec<RuntimeCountSegmentView>,
    pub adjustments: Vec<RuntimeAdjustmentView>,
    pub suppressed_adjustments: Vec<RuntimeAdjustmentView>,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCountSegmentView {
    pub label: String,
    pub value: i64,
    pub restricted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCapabilityView {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub provenance: Option<RuntimeFactProvenanceView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeEffectNoteView {
    pub provenance: RuntimeFactProvenanceView,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeActionBudgetView {
    pub actions: RuntimeCountView,
    pub reactions: RuntimeCountView,
    pub can_act: RuntimeCapabilityView,
    pub can_react: RuntimeCapabilityView,
    pub notes: Vec<RuntimeEffectNoteView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeConditionView {
    pub condition_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub condition_key: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_participant_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub duration_rounds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeAutomationLimitationView {
    pub code: EncounterRuntimeAutomationLimitationCodeView,
    pub target: EncounterRuntimeAutomationLimitationTargetView,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterRuntimeAutomationLimitationCodeView {
    ActivityCheckNotAutomated,
    ConditionAttackAdjustmentPartial,
    ConditionDamageAdjustmentPartial,
    ExplorationActivityRestrictionNotAutomated,
    ManipulateActionCheckNotAutomated,
    ProneContextRequiresAdjudication,
    RestrictedActionExceptionsNotAutomated,
    SpellDisruptionCheckNotAutomated,
    StunnedTimingRequiresAdjudication,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "target_type", rename_all = "snake_case")]
#[ts(tag = "target_type", rename_all = "snake_case")]
pub enum EncounterRuntimeAutomationLimitationTargetView {
    Participant,
    Condition { condition_id: i64 },
    Activity { activity_id: String },
    Spellcasting { entry_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeFactProvenanceView {
    pub source: RuntimeFactSourceView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub canonical_target: Option<RuntimeCanonicalTargetView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "source_type", rename_all = "snake_case")]
#[ts(tag = "source_type", rename_all = "snake_case")]
pub enum RuntimeFactSourceView {
    CanonicalRecord,
    ParticipantState,
    ParticipantVariant {
        variant: EncounterParticipantVariantView,
    },
    Condition {
        condition_id: i64,
        condition_ref: String,
        label: String,
    },
    RuntimeRule {
        rule: RuntimeRuleView,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RuntimeRuleView {
    ActionBudget,
    Movement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "target_type", rename_all = "snake_case")]
#[ts(tag = "target_type", rename_all = "snake_case")]
pub enum RuntimeCanonicalTargetView {
    Level,
    ArmorClass,
    MaximumHp,
    Perception,
    Save {
        save: RuntimeSaveKindView,
    },
    AbilityModifier {
        ability: RuntimeAbilityKindView,
    },
    Skill {
        skill_id: String,
    },
    Movement {
        speed_id: String,
    },
    ResourceMaximum {
        resource_id: String,
    },
    ActivityActionCost {
        activity_id: String,
    },
    ActivityFrequency {
        activity_id: String,
    },
    ActivityUses {
        activity_id: String,
    },
    ActivityRoll {
        activity_id: String,
        roll_id: String,
    },
    ActivityDamage {
        activity_id: String,
        damage_id: String,
    },
    SpellcastingAttack {
        entry_id: String,
    },
    SpellcastingDc {
        entry_id: String,
    },
    SpellSlotMaximum {
        entry_id: String,
        rank: i64,
    },
    RitualDc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RuntimeSaveKindView {
    Fortitude,
    Reflex,
    Will,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RuntimeAbilityKindView {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeActivityView {
    pub activity_id: String,
    pub label: String,
    pub kind: EncounterRuntimeActivityKindView,
    pub usage: EncounterRuntimeActivityUsageView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub action_cost: Option<EncounterRuntimeActionCostView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub frequency: Option<EncounterRuntimeFrequencyView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub uses: Option<EncounterRuntimeUsesView>,
    pub rolls: Vec<RuntimeRollView>,
    pub damage: Vec<RuntimeFormulaView>,
    pub modes: Vec<EncounterRuntimeActivityModeView>,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeActionCostView {
    pub value: EncounterRuntimeActionCostKindView,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(tag = "kind", rename_all = "snake_case")]
pub enum EncounterRuntimeActionCostKindView {
    Passive,
    Reaction,
    FreeAction,
    Actions { count: i64 },
    Time { value: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeFrequencyView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub serialized_value: Option<i64>,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeUsesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub serialized_value: Option<i64>,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterRuntimeActivityModeView {
    pub mode_id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub time: Option<String>,
    pub damage: Vec<RuntimeFormulaView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterRuntimeActivityKindView {
    Strike,
    Spell,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterRuntimeActivityUsageView {
    Unlimited,
    Limited,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeRollView {
    pub roll_id: String,
    pub label: String,
    pub base_value: i64,
    pub adjusted_value: i64,
    pub surface: RuntimeRollSurfaceView,
    pub modifiers: Vec<RuntimeModifierView>,
    pub suppressed_modifiers: Vec<RuntimeModifierView>,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RuntimeRollSurfaceView {
    AttackRoll,
    Dc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeFormulaView {
    pub damage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    pub formula: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub adjusted_formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub damage_type: Option<String>,
    pub effect_kind: RuntimeDamageEffectKindView,
    pub modifiers: Vec<RuntimeModifierView>,
    pub provenance: RuntimeFactProvenanceView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RuntimeDamageEffectKindView {
    Damage,
    Healing,
    DamageOrHealing,
    Unknown,
}
