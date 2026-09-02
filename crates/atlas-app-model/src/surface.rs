use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::EncounterRuntimeView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceView {
    pub metadata: RecordSurfaceMetadataView,
    pub profile: RecordSurfaceProfileView,
    pub presentation: RecordSurfacePresentationView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub encounter: Option<EncounterRuntimeView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RecordSurfaceProfileView {
    SearchCompact,
    RecordDetail,
    EncounterParticipant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceMetadataView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record_key: Option<String>,
    pub title: String,
    pub kind: String,
    pub kind_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rarity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source: Option<RecordSurfaceSourceView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceSourceView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub publication_title: Option<String>,
    pub pack_label: String,
    pub document_type: String,
    pub record_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_contract_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_system_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_upstream_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "presentation_type", rename_all = "snake_case")]
#[ts(tag = "presentation_type", rename_all = "snake_case")]
pub enum RecordSurfacePresentationView {
    Creature { body: Box<CreatureSurfaceView> },
    Unavailable { unavailable: SurfaceUnavailableView },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SurfaceUnavailableView {
    pub reason: SurfaceUnavailableReasonView,
    pub requested_kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SurfaceUnavailableReasonView {
    RecordFamilyNotMigrated,
    RecordUnavailable,
    ManualParticipant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub teaser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub size: Option<CreatureSurfaceSizeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub adjustment: Option<CreatureSurfaceAdjustmentView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<CreatureSurfaceInitiativeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub vitals: Option<CreatureSurfaceVitalsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub defenses: Option<CreatureSurfaceDefensesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub saves: Option<CreatureSurfaceSavesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub awareness: Option<CreatureSurfaceAwarenessView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub abilities: Option<CreatureSurfaceAbilitiesView>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub skills: Option<Vec<CreatureSurfaceSkillView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub unmodeled_skills: Option<Vec<CreatureSurfaceUnmodeledSkillView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub movement: Option<Vec<CreatureSurfaceMovementView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub resources: Option<Vec<CreatureSurfaceResourceView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rituals: Option<CreatureSurfaceRitualsView>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub equipment: Option<Vec<CreatureSurfaceEquipmentView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub lore: Option<Vec<CreatureSurfaceLoreView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub spellcasting: Option<Vec<CreatureSurfaceSpellcastingView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub standalone_spells: Option<Vec<CreatureSurfaceSpellView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub activities: Option<Vec<CreatureSurfaceActivityView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub content: Option<Vec<CreatureSurfaceContentView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub relationships: Option<Vec<CreatureSurfaceRelationshipView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub unavailable_domains: Option<CreatureSurfaceUnavailableDomainsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub provenance: Option<CreatureSurfaceProvenanceView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceUnavailableDomainsView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub classification: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub vitals: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub defenses: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub saves: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub awareness: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub abilities: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub skills: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub movement: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub resources: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub equipment: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub lore: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spellcasting: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub activities: Option<CreatureSurfaceDomainUnavailableView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub relationships: Option<CreatureSurfaceDomainUnavailableView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceDomainUnavailableView {
    pub causes: Vec<CreatureSurfaceUnavailableCauseView>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceUnavailableCauseView {
    pub state: CreatureSurfaceUnavailableStateView,
    pub field: CreatureSurfaceUnavailableFieldView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub component_id: Option<String>,
    pub provenance: CreatureSurfaceFactProvenanceView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub unmodeled_skill: Option<CreatureSurfaceUnmodeledSkillView>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceUnavailableStateView {
    Missing,
    Null,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceUnavailableFieldView {
    Size,
    Adjustment,
    InitiativeStatistic,
    Defenses,
    HitPoints,
    ArmorClass,
    ShieldArmorClassBonus,
    ShieldBrokenThreshold,
    ShieldHardness,
    ShieldMaximumHitPoints,
    Saves,
    Immunities,
    Resistances,
    Weaknesses,
    IwrAmount,
    IwrExceptions,
    IwrDoubleVs,
    Perception,
    Senses,
    SenseAcuity,
    Languages,
    Skills,
    SkillModifier,
    SkillVariantModifier,
    SkillVariantPredicate,
    UnmodeledSkill,
    LegacyAbilities,
    Movement,
    MovementMode,
    MovementSpeed,
    Resources,
    ResourceMaximum,
    EmbeddedEntities,
    ActivityTraits,
    ActivityActionCost,
    ActionFrequencyMaximum,
    ActionFrequencyPeriod,
    ActionUsesMaximum,
    ActivityRoll,
    ActivityDamage,
    ActivityContent,
    DamageFormula,
    DamageType,
    SpellPreparation,
    SpellTradition,
    SpellAttack,
    SpellDifficultyClass,
    SpellTraits,
    SpellRank,
    SpellUsesMaximum,
    SpellSlotMaximum,
    RitualDifficultyClass,
    SpellContent,
    Equipment,
    EquipmentUsesMaximum,
    Lore,
    LoreModifier,
    Relationships,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSizeView {
    pub value: CreatureSurfaceSizeValueView,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceSizeValueView {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceAdjustmentView {
    pub value: CreatureSurfaceAdjustmentValueView,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceAdjustmentValueView {
    Elite,
    Weak,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceInitiativeView {
    pub statistic: String,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceVitalsView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub hit_points: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<String>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceDefensesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub armor_class: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub armor_class_details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub hardness: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub shield: Option<CreatureSurfaceShieldView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub immunities: Vec<CreatureSurfaceIwrView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resistances: Vec<CreatureSurfaceIwrView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weaknesses: Vec<CreatureSurfaceIwrView>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceShieldView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub armor_class_bonus: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub broken_threshold: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub hardness: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub maximum_hit_points: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceIwrView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub double_vs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSavesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fortitude: Option<CreatureSurfaceSaveView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reflex: Option<CreatureSurfaceSaveView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub will: Option<CreatureSurfaceSaveView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub all_saves_note: Option<String>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSaveView {
    pub component_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceAwarenessView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub perception: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub has_vision: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub senses: Vec<CreatureSurfaceSenseView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub language_details: Option<String>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSenseView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub acuity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub range_feet: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceAbilitiesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub strength: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub dexterity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub constitution: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub intelligence: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub wisdom: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub charisma: Option<i64>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSkillView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub source_entries: Option<Vec<CreatureSurfaceSkillSourceEntryView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub variants: Option<Vec<CreatureSurfaceSkillVariantView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSkillSourceEntryView {
    pub authored_order: u32,
    pub authored_key: String,
    pub modifier: CreatureSurfaceIntegerPresenceView,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(tag = "state", rename_all = "snake_case")]
pub enum CreatureSurfaceIntegerPresenceView {
    Missing,
    Null,
    Value {
        #[serde(with = "crate::json_integer")]
        #[ts(type = "number")]
        value: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceUnmodeledSkillView {
    pub component_id: String,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub source_entries: Option<Vec<CreatureSurfaceSkillSourceEntryView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_item_id: Option<String>,
    pub authored_key: String,
    pub base: CreatureSurfaceIntegerPresenceView,
    pub reason: CreatureSurfaceUnmodeledSkillReasonView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceUnmodeledSkillReasonView {
    UnknownAuthoredKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSkillVariantView {
    pub component_id: String,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub predicates: Option<Vec<CreatureSurfaceSkillPredicateView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "predicate_type", rename_all = "snake_case")]
#[ts(tag = "predicate_type", rename_all = "snake_case")]
pub enum CreatureSurfaceSkillPredicateView {
    Term {
        term: String,
    },
    Not {
        term: String,
    },
    Any {
        terms: Vec<String>,
    },
    AtLeast {
        term: String,
        #[serde(with = "crate::json_integer")]
        #[ts(type = "number")]
        minimum: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceMovementView {
    pub component_id: String,
    pub authored_order: u32,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub speed_feet: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceResourceView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub maximum: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceActivityView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub provenance: CreatureSurfaceOccurrenceProvenanceView,
    pub activity_type: CreatureSurfaceActivityTypeView,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub action_cost: Option<CreatureSurfaceActionCostView>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub attack_effects: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub frequency: Option<CreatureSurfaceFrequencyView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub requirements: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cost: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub uses: Option<CreatureSurfaceUsesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub self_effect: Option<CreatureSurfaceSelfEffectView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rolls: Vec<CreatureSurfaceRollView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub damage: Vec<CreatureSurfaceDamageView>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub content: Option<Vec<CreatureSurfaceContentView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceFrequencyView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub period: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceUsesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub maximum: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSelfEffectView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceActivityTypeView {
    Strike,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "cost_type", rename_all = "snake_case")]
#[ts(tag = "cost_type", rename_all = "snake_case")]
pub enum CreatureSurfaceActionCostView {
    Passive,
    Reaction,
    FreeAction,
    Actions { count: u8 },
    Time { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceRollView {
    pub roll_id: String,
    pub label: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub modifier: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceDamageView {
    pub damage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub damage_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSpellcastingView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub provenance: CreatureSurfaceOccurrenceProvenanceView,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preparation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tradition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub attack_modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub difficulty_class: Option<i64>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub slots: Option<Vec<CreatureSurfaceSpellSlotView>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spells: Vec<CreatureSurfaceSpellView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSpellSlotView {
    #[serde(with = "crate::json_integer")]
    #[ts(type = "number")]
    pub rank: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub maximum: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSpellView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub provenance: CreatureSurfaceOccurrenceProvenanceView,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_record_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub rank: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub context: Option<CreatureSurfaceSpellOccurrenceContextView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub content: Option<Vec<CreatureSurfaceContentView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSpellOccurrenceContextView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub slot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub uses: Option<CreatureSurfaceUsesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub contextual_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceRitualsView {
    #[serde(with = "crate::json_integer")]
    #[ts(type = "number")]
    pub difficulty_class: i64,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceEquipmentView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub provenance: CreatureSurfaceOccurrenceProvenanceView,
    pub label: String,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub traits: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub usage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub quantity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub uses: Option<CreatureSurfaceUsesView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceLoreView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub provenance: CreatureSurfaceOccurrenceProvenanceView,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub modifier: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceOccurrenceProvenanceView {
    pub identity_stability: CreatureSurfaceOccurrenceIdentityStabilityView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub nested_source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub stable_source_locator: Option<String>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub source_locators: Option<Vec<CreatureSurfaceSourceLocatorView>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceOccurrenceIdentityStabilityView {
    StableNestedSourceId,
    UnstableOwnerFamilyOrdinal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSourceLocatorView {
    pub locator: String,
    pub precedence: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceContentView {
    pub content_key: String,
    pub role: CreatureSurfaceContentRoleView,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    pub blocks: Vec<CreatureSurfaceContentBlockView>,
    pub content_hash: String,
    pub visibility: String,
    pub provenance: CreatureSurfaceContentProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "block_type", rename_all = "snake_case")]
#[ts(tag = "block_type", rename_all = "snake_case")]
pub enum CreatureSurfaceContentBlockView {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        spans: Vec<CreatureSurfaceContentInlineView>,
    },
    List {
        ordered: bool,
        items: Vec<CreatureSurfaceContentListItemView>,
    },
    Table {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        caption: Option<String>,
        rows: Vec<CreatureSurfaceContentTableRowView>,
    },
    Divider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceContentListItemView {
    pub blocks: Vec<CreatureSurfaceContentBlockView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceContentTableRowView {
    pub cells: Vec<Vec<CreatureSurfaceContentBlockView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "span_type", rename_all = "snake_case")]
#[ts(tag = "span_type", rename_all = "snake_case")]
pub enum CreatureSurfaceContentInlineView {
    Text {
        text: String,
    },
    Strong {
        spans: Vec<CreatureSurfaceContentInlineView>,
    },
    Emphasis {
        spans: Vec<CreatureSurfaceContentInlineView>,
    },
    Code {
        text: String,
    },
    Reference {
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        record_key: Option<String>,
        embedded: bool,
    },
    Check {
        display: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        statistic: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, with = "crate::json_integer::optional")]
        #[ts(optional, type = "number")]
        difficulty_class: Option<i64>,
    },
    LineBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceContentRoleView {
    PrimaryDescription,
    Summary,
    SupplementalRules,
    EmbeddedCapability,
    JournalPage,
    TableResult,
    GeneratedNarrative,
    Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceContentProvenanceView {
    pub source_record_key: String,
    pub relative_source_path: String,
    pub field_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub nested_source_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceRelationshipView {
    pub source_occurrence_id: String,
    pub kind: CreatureSurfaceRelationshipKindView,
    pub target: CreatureSurfaceRelationshipTargetView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub contextual_label: Option<String>,
    pub provenance_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceRelationshipKindView {
    GrantedBy,
    ItemGrant,
    LinkedWeapon,
    PreparedSpell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "target_type", rename_all = "snake_case")]
#[ts(tag = "target_type", rename_all = "snake_case")]
pub enum CreatureSurfaceRelationshipTargetView {
    Occurrence { occurrence_id: String },
    UnresolvedSource { source_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceProvenanceView {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceFactProvenanceView {
    pub owner: CreatureSurfaceFactOwnerView,
    pub field: CreatureSurfaceSourceFieldView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceFactOwnerView {
    CanonicalCreature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceSourceFieldView {
    Size,
    Adjustment,
    Initiative,
    Defenses,
    Perception,
    Languages,
    Skills,
    LegacyAbilities,
    Movement,
    Resources,
    EmbeddedEntities,
}

fn optional_vec_is_empty<T>(values: &Option<Vec<T>>) -> bool {
    values.as_ref().is_none_or(Vec::is_empty)
}
