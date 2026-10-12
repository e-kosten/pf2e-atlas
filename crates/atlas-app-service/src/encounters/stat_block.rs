//! Internal mutable overlay workspace; it is not a transport or source authority.
use atlas_app_model::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatBlockView {
    pub record_key: String,
    pub title: String,
    pub level: Option<serde_json::Number>,
    pub adjusted_level: Option<serde_json::Number>,
    pub values: Vec<StatValueView>,
    pub speeds: Vec<MovementSpeedView>,
    pub action_budget: Option<ActionBudgetView>,
    pub activities: Vec<MechanicActivityView>,
    pub unapplied_effects: Vec<UnappliedEffectView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovementSpeedView {
    pub movement_type: String,
    pub label: String,
    pub base_value_feet: serde_json::Number,
    pub adjusted_value_feet: serde_json::Number,
    pub adjustments: Vec<RuntimeAdjustmentView>,
    pub suppressed_adjustments: Vec<RuntimeAdjustmentView>,
    pub notes: Vec<RuntimeEffectNoteView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatValueView {
    pub target: String,
    pub label: String,
    pub base_value: serde_json::Number,
    pub adjusted_value: serde_json::Number,
    pub modifiers: Vec<StatModifierView>,
    pub suppressed_modifiers: Vec<StatModifierView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MechanicActivityView {
    pub activity_id: String,
    pub label: String,
    pub kind: MechanicActivityKindView,
    pub navigation: RecordNavigationView,
    pub notes: Vec<RuntimeEffectNoteView>,
    pub rolls: Vec<ActivityRollView>,
    pub damage: Vec<DamageExpressionView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MechanicActivityKindView {
    Strike,
    Spell,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityRollView {
    pub roll_id: String,
    pub label: String,
    pub base_value: serde_json::Number,
    pub adjusted_value: serde_json::Number,
    pub surface: ActivityRollSurfaceView,
    pub modifiers: Vec<StatModifierView>,
    pub suppressed_modifiers: Vec<StatModifierView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityRollSurfaceView {
    AttackRoll,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageExpressionView {
    pub damage_id: String,
    pub label: Option<String>,
    pub formula: String,
    pub adjusted_formula: Option<String>,
    pub damage_type: Option<String>,
    pub effect_kind: DamageEffectKindView,
    pub modifiers: Vec<StatModifierView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageEffectKindView {
    Unknown,
}
