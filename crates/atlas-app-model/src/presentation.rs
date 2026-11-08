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

#[cfg(feature = "openapi")]
impl<T: utoipa::ToSchema> utoipa::ToSchema for FactView<T> {
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        use atlas_domain::QueryFieldState;
        use utoipa::PartialSchema;
        schemas.push((
            <QueryFieldState as utoipa::ToSchema>::name().into_owned(),
            QueryFieldState::schema(),
        ));
        T::schemas(schemas);
    }
}
#[cfg(feature = "openapi")]
impl<T: utoipa::ToSchema> utoipa::__dev::ComposeSchema for FactView<T> {
    fn compose(
        mut generics: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
    ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        use utoipa::openapi::{
            Ref,
            schema::{AnyOfBuilder, ObjectBuilder, Type},
        };
        let value = generics.pop().unwrap_or_else(T::schema);
        ObjectBuilder::new()
            .property(
                "state",
                Ref::from_schema_name(<atlas_domain::QueryFieldState as utoipa::ToSchema>::name()),
            )
            .required("state")
            .property(
                "value",
                AnyOfBuilder::new()
                    .item(value)
                    .item(ObjectBuilder::new().schema_type(Type::Null)),
            )
            .required("value")
            .into()
    }
}

/// Authored costs have a glyph only when their fixed meaning is known.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ActionGlyphView {
    OneAction,
    TwoActions,
    ThreeActions,
    Reaction,
    FreeAction,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActionCostView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub glyph: Option<ActionGlyphView>,
    pub label: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct NumberFactView {
    pub state: atlas_domain::QueryFieldState,
    #[ts(type = "number | null")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<f64>, required))]
    pub value: Option<serde_json::Number>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub adjustment: Option<NumberAdjustmentView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct NumberAdjustmentView {
    #[ts(type = "number")]
    #[cfg_attr(feature = "openapi", schema(value_type = f64))]
    pub authored: serde_json::Number,
    pub applied: Vec<StatModifierView>,
    pub suppressed: Vec<StatModifierView>,
    pub notes: Vec<RuntimeEffectNoteView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordPresentationView {
    pub identity: RecordSummaryView,
    pub classification: String,
    pub content: Vec<PreparedContentFieldView>,
    #[serde(rename = "embedded")]
    pub owned: Vec<EmbeddedRecordLinkView>,
    pub body: RecordBodyView,
    /// Declared recipient destinations not already presented in this document.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub grants: Option<RecordGrantsView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordGrantsView {
    pub entries: Vec<RecordGrantLinkView>,
    pub unavailable_count: usize,
    pub truncated: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordGrantLinkView {
    pub title: String,
    /// Root summaries are absent for exact embedded destinations.
    #[cfg_attr(feature = "openapi", schema(required))]
    pub record: Option<RecordSummaryView>,
    pub navigation: RecordNavigationView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RecordBodyView {
    Creature(Box<ActorPresentationView>),
    Hazard(Box<HazardPresentationView>),
    Spell(Box<SpellPresentationView>),
    Activity(Box<ActorActivityView>),
    PhysicalReference(Box<PhysicalPresentationView>),
    Ancestry(Box<crate::AncestryReferenceView>),
    Heritage(Box<crate::HeritageReferenceView>),
    Class(Box<crate::ClassReferenceView>),
    Deity(Box<crate::DeityReferenceView>),
    Vehicle(Box<crate::VehicleReferenceView>),
    Army(Box<crate::ArmyReferenceView>),
    RollTable(Box<RollTablePresentationView>),
    TableResult(Box<TableResultPresentationView>),
    Journal(Box<JournalPresentationView>),
    JournalPage(Box<JournalPagePresentationView>),
    Content,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EmbeddedRecordLinkView {
    pub navigation: RecordNavigationView,
    pub title: String,
    pub family: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub usage: Option<String>,
    pub action_cost: FactView<ActionCostView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorPresentationView {
    pub size: FactView<crate::ReferenceValueView>,
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
    pub movement_details: FactView<String>,
    pub land_speed: NumberFactView,
    pub immunities: FactView<Vec<IwrEntryView>>,
    pub weaknesses: FactView<Vec<IwrEntryView>>,
    pub resistances: FactView<Vec<IwrEntryView>>,
    pub senses: FactView<Vec<ActorSenseView>>,
    pub perception_details: FactView<String>,
    pub languages: FactView<Vec<crate::ReferenceValueView>>,
    pub language_details: FactView<String>,
    pub activities: FactView<Vec<ActorActivityView>>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub runtime: Option<ActorRuntimeView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorDefenseNotesView {
    pub armor_class: FactView<String>,
    pub hit_points: FactView<String>,
    pub all_saves: FactView<String>,
    pub fortitude: FactView<String>,
    pub reflex: FactView<String>,
    pub will: FactView<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActivityFrequencyView {
    pub maximum: NumberFactView,
    pub interval: FactView<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub repeats_description: Option<bool>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorSavesView {
    pub fortitude: NumberFactView,
    pub reflex: NumberFactView,
    pub will: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorModifierView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub help: Option<crate::ReferenceHelpView>,
    pub key: String,
    pub label: String,
    pub modifier: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorSkillView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub help: Option<crate::ReferenceHelpView>,
    pub key: String,
    pub label: String,
    pub modifier: NumberFactView,
    pub note: FactView<String>,
    pub conditional: FactView<Vec<ConditionalSkillView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ConditionalSkillView {
    pub qualification: FactView<String>,
    pub label: FactView<String>,
    pub modifier: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorMovementView {
    pub movement_type: FactView<String>,
    pub feet: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorSenseView {
    pub sense: FactView<crate::ReferenceValueView>,
    pub acuity: FactView<String>,
    pub range_feet: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct IwrEntryView {
    pub damage_type: FactView<crate::ReferenceValueView>,
    pub magnitude: NumberFactView,
    pub exceptions: FactView<Vec<String>>,
    pub double_against: FactView<Vec<String>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorRuntimeView {
    pub senses: Vec<EncounterSenseView>,
    pub contextual_armor_class: Vec<EncounterContextualArmorClassView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub action_budget: Option<ActionBudgetView>,
    pub unapplied_effects: Vec<UnappliedEffectView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct HazardPresentationView {
    pub detection_dc: NumberFactView,
    pub broken_threshold: NumberFactView,
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
/// Reading groups derived from authored activation/category, never prose.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ActorAbilityGroupView {
    Activation,
    Primary,
    Reaction,
    Supporting,
    Unclassified,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct StrikeRangeView {
    pub increment: NumberFactView,
    pub maximum: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActorActivityView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub help: Option<crate::ReferenceHelpView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub spell_effect: Option<SpellEffectPreviewView>,
    /// Display order within a reading group; never the runtime item ordinal.
    pub reading_order: usize,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub ability_group: Option<ActorAbilityGroupView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub first_level_only: Option<FactView<bool>>,
    pub is_ranged: FactView<bool>,
    pub strike_range: FactView<StrikeRangeView>,
    pub effect_duration: FactView<EffectDurationView>,
    pub self_effect: FactView<ActivityEffectReferenceView>,
    pub strike_effects: FactView<Vec<ActivityEffectReferenceView>>,
    pub focus_capacity: NumberFactView,
    pub frequency: FactView<ActivityFrequencyView>,
    pub casting_ranks: Vec<u8>,
    // Whether opening the selected document can add details beyond this summary.
    pub has_reference_details: bool,
    pub navigation: RecordNavigationView,
    pub title: String,
    pub family: String,
    pub kind: ActorActivityKindView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub usage: Option<String>,
    pub action_cost: FactView<ActionCostView>,
    pub traits: Vec<crate::RecordBadgeView>,
    pub attack: NumberFactView,
    pub attack_options: Vec<crate::StrikeAttackOptionView>,
    pub lore_modifier: NumberFactView,
    pub damage: FactView<Vec<DamageComponentView>>,
    pub difficulty_class: NumberFactView,
    pub casting_tradition: FactView<crate::ReferenceValueView>,
    pub preparation: FactView<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub casting_entry: Option<RecordNavigationView>,
    pub association: FactView<String>,
    pub base_rank: NumberFactView,
    pub authored_cast_rank: NumberFactView,
    #[cfg_attr(feature = "openapi", schema(no_recursion, required))]
    pub casting_context: Option<CastingEntryContextView>,
    pub notes: Vec<RuntimeEffectNoteView>,
}
/// Read-only source statistics at a selected casting rank; never applies an effect.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellEffectPreviewView {
    pub rank: u8,
    pub base_rank: u8,
    pub facts: Vec<SpellEffectFactView>,
    pub strikes: Vec<SpellEffectStrikeView>,
    pub choices: Vec<SpellEffectChoiceView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unavailable: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellEffectFactView {
    pub label: String,
    pub value: FactView<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub base_value: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub qualifier: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellEffectStrikeView {
    pub id: String,
    pub title: String,
    pub traits: Vec<crate::RecordBadgeView>,
    pub attack: SpellEffectFactView,
    pub damage: SpellEffectFactView,
    pub damage_type: FactView<crate::ReferenceValueView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellEffectChoiceView {
    pub label: String,
    pub facts: Vec<SpellEffectFactView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct CastingEntryContextView {
    pub parent: RecordNavigationView,
    pub parent_title: String,
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub spells: FactView<Vec<ActorActivityView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DamageComponentView {
    pub id: String,
    pub formula: FactView<String>,
    pub damage_type: FactView<crate::ReferenceValueView>,
    pub kinds: FactView<Vec<String>>,
    pub category: FactView<String>,
    pub materials: FactView<Vec<String>>,
    pub apply_modifier: FactView<bool>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellPresentationView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub runtime_attack: Option<NumberFactView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub runtime_difficulty_class: Option<NumberFactView>,
    pub ritual: FactView<RitualCastingView>,
    pub counteraction: FactView<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub rank_note: Option<String>,
    // Original rank, before the selected read-time heightening.
    pub base_rank: NumberFactView,
    /// Verified differences from this owned spell's authored base, not coverage claims.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub rank_changes: Option<Vec<SpellRankChangeView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub runtime_notes: Option<Vec<crate::RuntimeEffectNoteView>>,
    pub rank: NumberFactView,
    pub traditions: FactView<Vec<crate::ReferenceValueView>>,
    pub cast: FactView<ActionCostView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub requirements: Option<FactView<String>>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub cost: Option<FactView<String>>,
    pub range: FactView<String>,
    pub target: FactView<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub area: Option<FactView<SpellAreaView>>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub defense: Option<FactView<SpellDefenseView>>,
    pub duration: FactView<String>,
    pub sustained: FactView<bool>,
    pub damage: FactView<Vec<DamageComponentView>>,
    pub heightening: FactView<SpellHeighteningView>,
    pub forms: FactView<Vec<SpellFormView>>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub casting_entry: Option<FactView<String>>,
    pub authored_cast_rank: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpellRankFactView {
    Cast,
    Requirements,
    Cost,
    Range,
    Target,
    Area,
    Defense,
    Duration,
    Sustained,
    Traits,
    Traditions,
    Rarity,
    Counteraction,
    Damage { component_id: String },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellRankChangeView {
    pub fact: SpellRankFactView,
    pub base_rank: u8,
    pub base_value: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub explanation: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellAreaView {
    pub shape: FactView<String>,
    pub size: FactView<String>,
    pub details: FactView<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellDefenseView {
    pub statistic: FactView<String>,
    pub basic: FactView<bool>,
    pub passive: FactView<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct HeighteningDamageView {
    pub id: String,
    pub formula: FactView<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellFixedLevelView {
    pub rank: u8,
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub changes: FactView<SpellChangeView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellChangeView {
    pub cast: FactView<ActionCostView>,
    pub range: FactView<String>,
    pub target: FactView<String>,
    pub area: FactView<SpellAreaView>,
    pub defense: FactView<SpellDefenseView>,
    pub duration: FactView<String>,
    pub sustained: FactView<bool>,
    pub traits: FactView<Vec<crate::ReferenceValueView>>,
    pub traditions: FactView<Vec<crate::ReferenceValueView>>,
    pub damage: FactView<Vec<DamageComponentView>>,
    pub heightening: FactView<SpellHeighteningChangeView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellHeighteningChangeView {
    pub kind: FactView<String>,
    pub interval: NumberFactView,
    pub area: NumberFactView,
    pub damage: FactView<Vec<HeighteningDamageView>>,
    pub levels: FactView<Vec<SpellFixedLevelView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SpellFormView {
    pub id: String,
    pub name: FactView<String>,
    pub sort: NumberFactView,
    pub changes: FactView<SpellChangeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub rank_changes: Option<Vec<SpellRankChangeView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub rank_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub effective_rank: Option<u8>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PhysicalPresentationView {
    pub base_type: FactView<PhysicalBaseTypeView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub details: Option<PhysicalDetailsView>,
    pub quantity: NumberFactView,
    pub material: FactView<PhysicalMaterialReferenceView>,
    pub size: FactView<crate::ReferenceValueView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub consumable: Option<ConsumablePresentationView>,
    pub usage: FactView<String>,
    pub bulk: NumberFactView,
    pub price: FactView<PhysicalPriceView>,
    pub price_per: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PhysicalBaseTypeView {
    pub value: String,
    pub label: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponReferenceView {
    pub category: FactView<String>,
    pub group: FactView<crate::ReferenceValueView>,
    pub damage: FactView<WeaponDamageReferenceView>,
    pub range_increment: NumberFactView,
    pub maximum_range: NumberFactView,
    pub reload: FactView<String>,
    pub splash_damage: FactView<String>,
    pub ammunition: FactView<WeaponAmmunitionReferenceView>,
    pub runes: FactView<WeaponRuneReferenceView>,
    pub melee_usage: FactView<WeaponMeleeReferenceView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponDamageReferenceView {
    pub dice: NumberFactView,
    pub die: FactView<String>,
    pub damage_type: FactView<crate::ReferenceValueView>,
    pub modifier: NumberFactView,
    pub persistent: FactView<WeaponPersistentReferenceView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponPersistentReferenceView {
    pub number: NumberFactView,
    pub faces: NumberFactView,
    pub damage_type: FactView<crate::ReferenceValueView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponAmmunitionReferenceView {
    pub base_type: FactView<String>,
    pub capacity: NumberFactView,
    pub built_in: FactView<bool>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponRuneReferenceView {
    pub potency: NumberFactView,
    pub striking: FactView<String>,
    pub property: FactView<Vec<String>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WeaponMeleeReferenceView {
    pub die: FactView<String>,
    pub damage_type: FactView<crate::ReferenceValueView>,
    pub group: FactView<crate::ReferenceValueView>,
    pub traits: FactView<Vec<crate::ReferenceValueView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ArmorReferenceView {
    pub category: FactView<String>,
    pub group: FactView<crate::ReferenceValueView>,
    pub ac_bonus: NumberFactView,
    pub dex_cap: NumberFactView,
    pub check_penalty: NumberFactView,
    pub speed_penalty: NumberFactView,
    pub strength: NumberFactView,
    pub potency: NumberFactView,
    pub resilient: FactView<String>,
    pub property_runes: FactView<Vec<String>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ShieldReferenceView {
    pub ac_bonus: NumberFactView,
    pub speed_penalty: NumberFactView,
    pub hardness: NumberFactView,
    pub hit_points: NumberFactView,
    pub durability_in_table: bool,
    pub reinforcing: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContainerReferenceView {
    pub capacity: NumberFactView,
    pub ignored_bulk: NumberFactView,
    pub held_or_stowed_bulk: NumberFactView,
    pub stowing: FactView<bool>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct AmmunitionReferenceView {
    pub maximum_uses: NumberFactView,
    pub source_uses: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct KitReferenceView {
    pub items: FactView<Vec<KitEntryReferenceView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct KitEntryReferenceView {
    pub name: FactView<String>,
    pub quantity: NumberFactView,
    #[cfg_attr(feature = "openapi", schema(no_recursion))]
    pub items: FactView<Vec<KitEntryReferenceView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PhysicalPriceView {
    pub pp: NumberFactView,
    pub gp: NumberFactView,
    pub sp: NumberFactView,
    pub cp: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ConsumablePresentationView {
    pub maximum_hp: NumberFactView,
    pub hardness: NumberFactView,
    pub auto_destroy: FactView<bool>,
    pub spell: FactView<EmbeddedRecordLinkView>,
    pub spell_presented_inline: bool,
    pub category: FactView<String>,
    pub damage: FactView<ConsumableDamageView>,
    pub maximum_uses: NumberFactView,
    pub source_uses: NumberFactView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ConsumableDamageView {
    pub formula: FactView<String>,
    pub kind: FactView<String>,
    pub damage_type: FactView<crate::ReferenceValueView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollTablePresentationView {
    pub roll_note: String,
    pub formula: FactView<String>,
    pub results: FactView<Vec<RollTableRowView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TableResultPresentationView {
    pub range: FactView<String>,
    pub weight: NumberFactView,
    pub result_type: FactView<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollTableRowView {
    pub link: EmbeddedRecordLinkView,
    #[cfg_attr(feature = "openapi", schema(schema_with = roll_table_range_schema))]
    pub range: FactView<[f64; 2]>,
}

#[cfg(feature = "openapi")]
fn roll_table_range_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{ArrayBuilder, Schema};
    let range = Schema::Array(
        ArrayBuilder::new()
            .items(<f64 as utoipa::PartialSchema>::schema())
            .min_items(Some(2))
            .max_items(Some(2))
            .build(),
    );
    <FactView<Vec<f64>> as utoipa::__dev::ComposeSchema>::compose(vec![range.into()])
}

/// An authored effect reference; navigation exists only for a verified destination.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ActivityEffectReferenceView {
    pub name: FactView<String>,
    pub authored_target: FactView<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub navigation: Option<RecordNavigationView>,
}

/// Authored ritual checks; no DC or eligibility inference.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RitualCastingView {
    pub primary_check: FactView<String>,
    pub secondary_casters: NumberFactView,
    pub secondary_checks: FactView<String>,
}
/// Reference duration, distinct from an applied effect's remaining time.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EffectDurationView {
    pub duration: FactView<String>,
    pub sustained: FactView<bool>,
    pub expiry: FactView<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct JournalPresentationView {
    pub pages: FactView<Vec<EmbeddedRecordLinkView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct JournalPagePresentationView {
    pub show_body_heading: FactView<bool>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub media_notice: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterSenseView {
    pub sense_type: crate::ReferenceValueView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub precision: Option<String>,
    #[cfg_attr(feature = "openapi", schema(value_type = Option<f64>, required))]
    pub range_feet: Option<serde_json::Number>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub available: Option<bool>,
    #[cfg_attr(feature = "openapi", schema(value_type = Option<f64>, required))]
    pub perception: Option<serde_json::Number>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub reason: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterContextualArmorClassView {
    pub opponent_key: String,
    #[cfg_attr(feature = "openapi", schema(value_type = FactView<f64>))]
    pub value: FactView<serde_json::Number>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PhysicalMaterialReferenceView {
    pub kind: FactView<String>,
    pub grade: FactView<String>,
}
