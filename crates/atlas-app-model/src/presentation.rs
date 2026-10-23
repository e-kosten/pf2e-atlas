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

/// Authored costs have a glyph only when their fixed meaning is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ActionGlyphView {
    OneAction,
    TwoActions,
    ThreeActions,
    Reaction,
    FreeAction,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActionCostView {
    pub glyph: Option<ActionGlyphView>,
    pub label: String,
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
    pub classification: String,
    pub content: Vec<PreparedContentFieldView>,
    #[serde(rename = "embedded")]
    pub owned: Vec<EmbeddedRecordLinkView>,
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
    Journal(Box<JournalPresentationView>),
    JournalPage(Box<JournalPagePresentationView>),
    Content,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EmbeddedRecordLinkView {
    pub navigation: RecordNavigationView,
    pub title: String,
    pub family: String,
    pub usage: Option<String>,
    pub action_cost: FactView<ActionCostView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorPresentationView {
    pub size: FactView<String>,
    pub hardness: NumberFactView,
    pub ritual_difficulty_class: NumberFactView,
    pub defense_notes: ActorDefenseNotesView,
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
pub struct ActorDefenseNotesView {
    pub armor_class: FactView<String>,
    pub hit_points: FactView<String>,
    pub all_saves: FactView<String>,
    pub fortitude: FactView<String>,
    pub reflex: FactView<String>,
    pub will: FactView<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActivityFrequencyView {
    pub maximum: NumberFactView,
    pub interval: FactView<String>,
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
    pub senses: Vec<EncounterSenseView>,
    pub contextual_armor_class: Vec<EncounterContextualArmorClassView>,
    pub action_budget: Option<ActionBudgetView>,
    pub unapplied_effects: Vec<UnappliedEffectView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct HazardPresentationView {
    pub actor: ActorPresentationView,
    // Known authored HP max and AC determine the primary HP/AC/hardness block.
    // Raw defense facts and independently applicable saves remain available.
    pub defense_block: FactView<bool>,
    // Reading-only omission of the exact all-zero, blank-detail legacy save block.
    // The authored actor save facts remain intact.
    pub show_saves: bool,
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
    pub is_ranged: FactView<bool>,
    pub effect_duration: FactView<EffectDurationView>,
    pub self_effect: FactView<ActivityEffectReferenceView>,
    pub strike_effects: FactView<Vec<String>>,
    pub focus_capacity: NumberFactView,
    pub frequency: FactView<ActivityFrequencyView>,
    pub casting_ranks: Vec<u8>,
    // Whether opening the selected document can add details beyond this summary.
    pub has_reference_details: bool,
    pub navigation: RecordNavigationView,
    pub title: String,
    pub family: String,
    pub kind: ActorActivityKindView,
    pub usage: Option<String>,
    pub action_cost: FactView<ActionCostView>,
    pub traits: Vec<crate::RecordBadgeView>,
    pub attack: NumberFactView,
    pub attack_options: Vec<crate::StrikeAttackOptionView>,
    pub lore_modifier: NumberFactView,
    pub damage: FactView<Vec<DamageComponentView>>,
    pub difficulty_class: NumberFactView,
    pub casting_tradition: FactView<String>,
    pub preparation: FactView<String>,
    pub casting_entry: Option<RecordNavigationView>,
    pub association: FactView<String>,
    pub base_rank: NumberFactView,
    pub authored_cast_rank: NumberFactView,
    pub casting_context: Option<CastingEntryContextView>,
    pub notes: Vec<RuntimeEffectNoteView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct CastingEntryContextView {
    pub parent: RecordNavigationView,
    pub parent_title: String,
    pub spells: FactView<Vec<ActorActivityView>>,
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
    pub runtime_attack: Option<NumberFactView>,
    pub runtime_difficulty_class: Option<NumberFactView>,
    pub ritual: FactView<RitualCastingView>,
    pub counteraction: FactView<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rank_note: Option<String>,
    // Original rank, before the selected read-time heightening.
    pub base_rank: NumberFactView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub runtime_notes: Option<Vec<crate::RuntimeEffectNoteView>>,
    pub rank: NumberFactView,
    pub traditions: FactView<Vec<String>>,
    pub cast: FactView<ActionCostView>,
    pub requirements: Option<FactView<String>>,
    pub cost: Option<FactView<String>>,
    pub range: FactView<String>,
    pub target: FactView<String>,
    pub area: Option<FactView<SpellAreaView>>,
    pub defense: Option<FactView<SpellDefenseView>>,
    pub duration: FactView<String>,
    pub sustained: FactView<bool>,
    pub damage: FactView<Vec<DamageComponentView>>,
    pub heightening: FactView<SpellHeighteningView>,
    pub forms: FactView<Vec<SpellFormView>>,
    pub casting_entry: Option<FactView<String>>,
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
    pub cast: FactView<ActionCostView>,
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
    pub details: Option<PhysicalDetailsView>,
    pub quantity: NumberFactView,
    pub material: FactView<PhysicalMaterialReferenceView>,
    pub size: FactView<String>,
    pub consumable: Option<ConsumablePresentationView>,
    pub usage: FactView<String>,
    pub bulk: NumberFactView,
    pub price: FactView<PhysicalPriceView>,
    pub price_per: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PhysicalDetailsView {
    Weapon(Box<WeaponReferenceView>),
    Armor(Box<ArmorReferenceView>),
    Shield(Box<ShieldReferenceView>),
    Container(Box<ContainerReferenceView>),
    Ammunition(Box<AmmunitionReferenceView>),
    Kit(Box<KitReferenceView>),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponReferenceView {
    pub category: FactView<String>,
    pub group: FactView<String>,
    pub damage: FactView<WeaponDamageReferenceView>,
    pub range_increment: NumberFactView,
    pub maximum_range: NumberFactView,
    pub reload: FactView<String>,
    pub splash_damage: FactView<String>,
    pub ammunition: FactView<WeaponAmmunitionReferenceView>,
    pub runes: FactView<WeaponRuneReferenceView>,
    pub melee_usage: FactView<WeaponMeleeReferenceView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponDamageReferenceView {
    pub dice: NumberFactView,
    pub die: FactView<String>,
    pub damage_type: FactView<String>,
    pub modifier: NumberFactView,
    pub persistent: FactView<WeaponPersistentReferenceView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponPersistentReferenceView {
    pub number: NumberFactView,
    pub faces: NumberFactView,
    pub damage_type: FactView<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponAmmunitionReferenceView {
    pub base_type: FactView<String>,
    pub capacity: NumberFactView,
    pub built_in: FactView<bool>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponRuneReferenceView {
    pub potency: NumberFactView,
    pub striking: FactView<String>,
    pub property: FactView<Vec<String>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponMeleeReferenceView {
    pub die: FactView<String>,
    pub damage_type: FactView<String>,
    pub group: FactView<String>,
    pub traits: FactView<Vec<String>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ArmorReferenceView {
    pub category: FactView<String>,
    pub group: FactView<String>,
    pub ac_bonus: NumberFactView,
    pub dex_cap: NumberFactView,
    pub check_penalty: NumberFactView,
    pub speed_penalty: NumberFactView,
    pub strength: NumberFactView,
    pub potency: NumberFactView,
    pub resilient: FactView<String>,
    pub property_runes: FactView<Vec<String>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ShieldReferenceView {
    pub ac_bonus: NumberFactView,
    pub speed_penalty: NumberFactView,
    pub hardness: NumberFactView,
    pub hit_points: NumberFactView,
    pub durability_in_table: bool,
    pub reinforcing: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContainerReferenceView {
    pub capacity: NumberFactView,
    pub ignored_bulk: NumberFactView,
    pub held_or_stowed_bulk: NumberFactView,
    pub stowing: FactView<bool>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct AmmunitionReferenceView {
    pub maximum_uses: NumberFactView,
    pub source_uses: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct KitReferenceView {
    pub items: FactView<Vec<KitEntryReferenceView>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct KitEntryReferenceView {
    pub name: FactView<String>,
    pub quantity: NumberFactView,
    pub items: FactView<Vec<KitEntryReferenceView>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PhysicalPriceView {
    pub pp: NumberFactView,
    pub gp: NumberFactView,
    pub sp: NumberFactView,
    pub cp: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ConsumablePresentationView {
    pub spell: FactView<EmbeddedRecordLinkView>,
    pub spell_presented_inline: bool,
    pub category: FactView<String>,
    pub damage: FactView<ConsumableDamageView>,
    pub maximum_uses: NumberFactView,
    pub source_uses: NumberFactView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ConsumableDamageView {
    pub formula: FactView<String>,
    pub kind: FactView<String>,
    pub damage_type: FactView<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollTablePresentationView {
    pub roll_note: String,
    pub formula: FactView<String>,
    pub results: FactView<Vec<RollTableRowView>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TableResultPresentationView {
    pub range: FactView<String>,
    pub weight: NumberFactView,
    pub result_type: FactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollTableRowView {
    pub link: EmbeddedRecordLinkView,
    pub range: FactView<[f64; 2]>,
}

/// An authored effect reference; navigation exists only for a verified destination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActivityEffectReferenceView {
    pub name: FactView<String>,
    pub authored_target: FactView<String>,
    pub navigation: Option<RecordNavigationView>,
}

/// Authored ritual checks; no DC or eligibility inference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RitualCastingView {
    pub primary_check: FactView<String>,
    pub secondary_casters: NumberFactView,
    pub secondary_checks: FactView<String>,
}
/// Reference duration, distinct from an applied effect's remaining time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EffectDurationView {
    pub duration: FactView<String>,
    pub sustained: FactView<bool>,
    pub expiry: FactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct JournalPresentationView {
    pub pages: FactView<Vec<EmbeddedRecordLinkView>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct JournalPagePresentationView {
    pub show_body_heading: FactView<bool>,
    pub media_notice: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterSenseView {
    pub sense_type: String,
    pub precision: Option<String>,
    pub range_feet: Option<serde_json::Number>,
    pub available: Option<bool>,
    pub perception: Option<serde_json::Number>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterContextualArmorClassView {
    pub opponent_key: String,
    pub value: FactView<serde_json::Number>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PhysicalMaterialReferenceView {
    pub kind: FactView<String>,
    pub grade: FactView<String>,
}
