//! Transient semantic facts for reference and encounter renderers.
use crate::{
    ActionBudgetView, PreparedContentFieldView, RecordNavigationView, RecordSummaryView,
    RuntimeEffectNoteView, StatModifierView, UnappliedEffectView,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct FactView<T> {
    pub state: atlas_domain::QueryFieldState,
    pub value: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct NumberFactView {
    pub state: atlas_domain::QueryFieldState,
    #[ts(type = "number | null")]
    pub value: Option<serde_json::Number>,
    pub adjustment: Option<NumberAdjustmentView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct NumberAdjustmentView {
    #[ts(type = "number")]
    pub authored: serde_json::Number,
    pub applied: Vec<StatModifierView>,
    pub suppressed: Vec<StatModifierView>,
    pub notes: Vec<RuntimeEffectNoteView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordPresentationView {
    pub identity: RecordSummaryView,
    pub content: Vec<PreparedContentFieldView>,
    pub owned: Vec<OwnedRecordLinkView>,
    pub body: RecordBodyView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RecordBodyView {
    Creature(Box<ActorPresentationView>),
    Hazard(Box<HazardPresentationView>),
    Spell(Box<SpellPresentationView>),
    Activity(Box<ActorActivityView>),
    PhysicalReference(Box<PhysicalPresentationView>),
    RollTable(Box<RollTablePresentationView>),
    TableResult(Box<TableResultPresentationView>),
    Content,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OwnedRecordLinkView {
    pub navigation: RecordNavigationView,
    pub title: String,
    pub family: String,
    pub usage: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorPresentationView {
    pub level: NumberFactView,
    pub armor_class: NumberFactView,
    pub maximum_hp: NumberFactView,
    pub perception: NumberFactView,
    pub saves: ActorSavesView,
    pub abilities: FactView<Vec<ActorModifierView>>,
    pub skills: FactView<Vec<ActorSkillView>>,
    pub movement: FactView<Vec<ActorMovementView>>,
    pub land_speed: NumberFactView,
    pub immunities: FactView<Vec<IwrEntryView>>,
    pub weaknesses: FactView<Vec<IwrEntryView>>,
    pub resistances: FactView<Vec<IwrEntryView>>,
    pub senses: FactView<Vec<ActorSenseView>>,
    pub perception_details: FactView<String>,
    pub languages: FactView<Vec<String>>,
    pub language_details: FactView<String>,
    pub activities: FactView<Vec<ActorActivityView>>,
    pub runtime: Option<ActorRuntimeView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorSavesView {
    pub fortitude: NumberFactView,
    pub reflex: NumberFactView,
    pub will: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorModifierView {
    pub key: String,
    pub label: String,
    pub modifier: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorSkillView {
    pub key: String,
    pub label: String,
    pub modifier: NumberFactView,
    pub note: FactView<String>,
    pub conditional: FactView<Vec<ConditionalSkillView>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ConditionalSkillView {
    pub label: FactView<String>,
    pub modifier: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorMovementView {
    pub movement_type: FactView<String>,
    pub feet: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorSenseView {
    pub sense: FactView<String>,
    pub acuity: FactView<String>,
    pub range_feet: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct IwrEntryView {
    pub damage_type: FactView<String>,
    pub magnitude: NumberFactView,
    pub exceptions: FactView<Vec<String>>,
    pub double_against: FactView<Vec<String>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorRuntimeView {
    pub action_budget: Option<ActionBudgetView>,
    pub unapplied_effects: Vec<UnappliedEffectView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct HazardPresentationView {
    pub actor: ActorPresentationView,
    pub complex: FactView<bool>,
    pub stealth: NumberFactView,
    pub hardness: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ActorActivityKindView {
    Strike,
    Ability,
    Spell,
    CastingEntry,
    Lore,
    Gear,
    Other,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorActivityView {
    pub navigation: RecordNavigationView,
    pub title: String,
    pub family: String,
    pub kind: ActorActivityKindView,
    pub usage: Option<String>,
    pub traits: Vec<crate::RecordBadgeView>,
    pub attack: NumberFactView,
    pub lore_modifier: NumberFactView,
    pub damage: FactView<Vec<DamageComponentView>>,
    pub difficulty_class: NumberFactView,
    pub casting_tradition: FactView<String>,
    pub preparation: FactView<String>,
    pub casting_entry: Option<RecordNavigationView>,
    pub association: FactView<String>,
    pub notes: Vec<RuntimeEffectNoteView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DamageComponentView {
    pub id: String,
    pub formula: FactView<String>,
    pub damage_type: FactView<String>,
    pub kinds: FactView<Vec<String>>,
    pub category: FactView<String>,
    pub materials: FactView<Vec<String>>,
    pub apply_modifier: FactView<bool>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellPresentationView {
    pub rank: NumberFactView,
    pub traditions: FactView<Vec<String>>,
    pub cast: FactView<String>,
    pub requirements: FactView<String>,
    pub cost: FactView<String>,
    pub range: FactView<String>,
    pub target: FactView<String>,
    pub area: FactView<SpellAreaView>,
    pub defense: FactView<SpellDefenseView>,
    pub duration: FactView<String>,
    pub sustained: FactView<bool>,
    pub damage: FactView<Vec<DamageComponentView>>,
    pub heightening: FactView<SpellHeighteningView>,
    pub forms: FactView<Vec<SpellFormView>>,
    pub casting_entry: FactView<String>,
    pub authored_cast_rank: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellAreaView {
    pub shape: FactView<String>,
    pub size: FactView<String>,
    pub details: FactView<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellDefenseView {
    pub statistic: FactView<String>,
    pub basic: FactView<bool>,
    pub passive: FactView<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpellHeighteningView {
    Interval {
        interval: NumberFactView,
        area: Box<NumberFactView>,
        damage: FactView<Vec<HeighteningDamageView>>,
    },
    Fixed {
        levels: FactView<Vec<SpellFixedLevelView>>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct HeighteningDamageView {
    pub id: String,
    pub formula: FactView<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellFixedLevelView {
    pub rank: u8,
    pub changes: FactView<SpellChangeView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellChangeView {
    pub cast: FactView<String>,
    pub range: FactView<String>,
    pub target: FactView<String>,
    pub area: FactView<SpellAreaView>,
    pub defense: FactView<SpellDefenseView>,
    pub duration: FactView<String>,
    pub sustained: FactView<bool>,
    pub traits: FactView<Vec<String>>,
    pub traditions: FactView<Vec<String>>,
    pub damage: FactView<Vec<DamageComponentView>>,
    pub heightening: FactView<SpellHeighteningChangeView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellHeighteningChangeView {
    pub kind: FactView<String>,
    pub interval: NumberFactView,
    pub area: NumberFactView,
    pub damage: FactView<Vec<HeighteningDamageView>>,
    pub levels: FactView<Vec<SpellFixedLevelView>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellFormView {
    pub id: String,
    pub name: FactView<String>,
    pub sort: NumberFactView,
    pub changes: FactView<SpellChangeView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PhysicalPresentationView {
    pub usage: FactView<String>,
    pub bulk: NumberFactView,
    pub price: FactView<String>,
    pub price_per: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollTablePresentationView {
    pub formula: FactView<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TableResultPresentationView {
    pub range: FactView<String>,
    pub weight: NumberFactView,
    pub result_type: FactView<String>,
}
