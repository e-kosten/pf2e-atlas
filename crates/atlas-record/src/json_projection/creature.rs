use atlas_domain::DetailLevel;
use serde::Serialize;

use crate::{
    ActivityRollAbility, ContentOwner, ContentRole, CreatureActionCost, CreatureAdjustment,
    CreatureCapability, CreatureContentAssociationFailure, CreatureContentPlacement,
    CreatureDamage, CreatureDamageKind, CreatureEntity, CreatureEntityOccurrence,
    CreatureEntityRelationshipKind, CreatureEntityTarget, CreatureFrequency, CreatureIwr,
    CreatureMovementMode, CreatureNumber, CreatureOccurrenceContext, CreatureOccurrenceParent,
    CreaturePreparedSpellSlot, CreatureRecord, CreatureRelationshipTarget, CreatureResourceAmount,
    CreatureRoll, CreatureRollKind, CreatureSave, CreatureSkill, CreatureSkillVariant,
    CreatureSourceScalar, CreatureSpellPreparation, CreatureSpellSlot,
    CreatureUnmodeledSkillReason, CreatureUseLimit, FactValue, PresentationContent,
    ResourceCurrentPolicy, UnsupportedMechanicNote, UnsupportedSourceValue, place_creature_content,
    project_presentation_content,
};

use super::{
    CreaturePerceptionJson, RecordEditionContextJson, RecordPresentationJson,
    RecordRelationshipLookupJson,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureDefensesJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac: Option<CreatureArmorClassJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp: Option<CreatureHitPointsJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardness: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shield: Option<CreatureShieldJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saves: Option<CreatureSavesJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_saves_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immunities: Option<Vec<CreatureIwrJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resistances: Option<Vec<CreatureIwrJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weaknesses: Option<Vec<CreatureIwrJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureShieldJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armor_class_bonus: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broken_threshold: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardness: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_hit_points: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_hit_points: Option<i64>,
    pub current_policy: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureInitiativeJson {
    pub statistic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureAbilitiesJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dexterity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constitution: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intelligence: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wisdom: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charisma: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureEquipmentJson {
    #[serde(skip)]
    pub id: String,
    pub order: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<CreatureUseLimitJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_record_key: Option<String>,
    #[serde(skip)]
    pub target_entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureOccurrenceProvenanceJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<CreatureContentJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureLoreJson {
    #[serde(skip)]
    pub id: String,
    pub order: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_record_key: Option<String>,
    #[serde(skip)]
    pub target_entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureOccurrenceProvenanceJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<CreatureContentJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureOccurrenceProvenanceJson {
    pub identity_stability: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested_source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stable_source_locator: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_locators: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureRitualsJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty_class: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureContentJson {
    pub content_key: String,
    #[serde(skip)]
    pub owner: CreatureContentOwnerJson,
    pub role: &'static str,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub content_hash: String,
    pub visibility: &'static str,
    pub document: PresentationContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureContentProvenanceJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "owner_type", rename_all = "snake_case")]
pub enum CreatureContentOwnerJson {
    Record { record_key: String },
    Entity { entity_id: String },
    Occurrence { occurrence_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureContentProvenanceJson {
    pub source_record_key: String,
    pub relative_source_path: String,
    pub field_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested_source_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureRelationshipJson {
    pub source_occurrence_id: String,
    pub kind: &'static str,
    pub target: CreatureRelationshipTargetJson,
    pub source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "target_type", rename_all = "snake_case")]
pub enum CreatureRelationshipTargetJson {
    Occurrence { occurrence_id: String },
    UnresolvedNestedSource { source_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureAvailabilityJson {
    pub state: CreatureAvailabilityStateJson,
    pub field: CreatureAvailabilityFieldJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authored_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_value: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureAvailabilityEvidenceJson {
    pub state: CreatureAvailabilityStateJson,
    pub field: CreatureAvailabilityFieldJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authored_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_shape: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatureAvailabilityStateJson {
    Missing,
    Null,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatureAvailabilityFieldJson {
    Size,
    Defenses,
    Perception,
    EmbeddedEntities,
    Adjustment,
    InitiativeStatistic,
    SourceAlliance,
    HitPointsValue,
    SenseAcuity,
    SkillPredicate,
    MovementMode,
    ResourceMaximum,
    ResourceSerializedValue,
    ResourceSourceDrift,
    RitualDifficultyClass,
    ActionCost,
    SpellPreparation,
    SpellSlotMaximum,
    SpellSlotSerializedValue,
    PreparedSpellSlot,
    SpellRitualSecondaryCasters,
    SpellDefenseSave,
    DamageKind,
    DamageApplyModifier,
    UnsupportedMechanic,
    UnmodeledSkill,
    UnsupportedCapability,
    ContentAssociation,
}

impl CreatureAvailabilityFieldJson {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::Defenses => "defenses",
            Self::Perception => "perception",
            Self::EmbeddedEntities => "embedded_entities",
            Self::Adjustment => "adjustment",
            Self::InitiativeStatistic => "initiative_statistic",
            Self::SourceAlliance => "source_alliance",
            Self::HitPointsValue => "hit_points_value",
            Self::SenseAcuity => "sense_acuity",
            Self::SkillPredicate => "skill_predicate",
            Self::MovementMode => "movement_mode",
            Self::ResourceMaximum => "resource_maximum",
            Self::ResourceSerializedValue => "resource_serialized_value",
            Self::ResourceSourceDrift => "resource_source_drift",
            Self::RitualDifficultyClass => "ritual_difficulty_class",
            Self::ActionCost => "action_cost",
            Self::SpellPreparation => "spell_preparation",
            Self::SpellSlotMaximum => "spell_slot_maximum",
            Self::SpellSlotSerializedValue => "spell_slot_serialized_value",
            Self::PreparedSpellSlot => "prepared_spell_slot",
            Self::SpellRitualSecondaryCasters => "spell_ritual_secondary_casters",
            Self::SpellDefenseSave => "spell_defense_save",
            Self::DamageKind => "damage_kind",
            Self::DamageApplyModifier => "damage_apply_modifier",
            Self::UnsupportedMechanic => "unsupported_mechanic",
            Self::UnmodeledSkill => "unmodeled_skill",
            Self::UnsupportedCapability => "unsupported_capability",
            Self::ContentAssociation => "content_association",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureProvenanceJson {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
    pub facts: CreatureFactProvenanceSetJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureFactProvenanceSetJson {
    pub level: CreatureFactProvenanceJson,
    pub rarity: CreatureFactProvenanceJson,
    pub traits: CreatureFactProvenanceJson,
    pub size: CreatureFactProvenanceJson,
    pub publication: CreatureFactProvenanceJson,
    pub adjustment: CreatureFactProvenanceJson,
    pub source_alliance: CreatureFactProvenanceJson,
    pub perception: CreatureFactProvenanceJson,
    pub initiative: CreatureFactProvenanceJson,
    pub languages: CreatureFactProvenanceJson,
    pub skills: CreatureFactProvenanceJson,
    pub abilities: CreatureFactProvenanceJson,
    pub defenses: CreatureFactProvenanceJson,
    pub movement: CreatureFactProvenanceJson,
    pub resources: CreatureFactProvenanceJson,
    pub embedded_entities: CreatureFactProvenanceJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CreatureFactProvenanceJson {
    Source { field: &'static str },
    Derived { derivation: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureArmorClassJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureHitPointsJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary_maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureSavesJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fortitude: Option<CreatureSaveJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflex: Option<CreatureSaveJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will: Option<CreatureSaveJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSaveJson {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureIwrJson {
    pub id: String,
    pub order: u32,
    pub iwr_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub double_vs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_once: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSenseJson {
    pub id: String,
    pub order: u32,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acuity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_feet: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSkillJson {
    pub id: String,
    pub order: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_entries: Vec<CreatureSkillSourceEntryJson>,
    pub slug: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<CreatureSkillVariantJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmodeled: Option<CreatureUnmodeledSkillJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSkillSourceEntryJson {
    pub authored_key: String,
    pub modifier: CreatureIntegerPresenceJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureUnmodeledSkillJson {
    pub authored_key: String,
    pub base: CreatureIntegerPresenceJson,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureUnmodeledSkillAvailabilityJson {
    pub skill_id: String,
    pub authored_order: u32,
    pub authored_key: String,
    pub modifier: CreatureIntegerPresenceJson,
    pub message: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum CreatureIntegerPresenceJson {
    Missing,
    Null,
    Value(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSkillVariantJson {
    pub id: String,
    pub order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub predicates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureMovementJson {
    pub modes: Vec<CreatureMovementModeJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureMovementModeJson {
    pub id: String,
    pub order: u32,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_feet: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureResourceJson {
    pub id: String,
    pub order: u32,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_value: Option<i64>,
    pub current_policy: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureActionCostJson {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsupported: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureFrequencyJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_value: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureStrikeJson {
    #[serde(skip)]
    pub id: String,
    pub order: u32,
    pub label: String,
    pub traits: Vec<String>,
    pub action_cost: CreatureActionCostJson,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attack_effects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rolls: Option<Vec<CreatureRollJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<Vec<CreatureDamageJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<CreatureContentJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_record_key: Option<String>,
    #[serde(skip)]
    pub target_entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureOccurrenceProvenanceJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureActionJson {
    #[serde(skip)]
    pub id: String,
    pub order: u32,
    pub label: String,
    pub traits: Vec<String>,
    pub action_cost: CreatureActionCostJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<CreatureFrequencyJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rolls: Option<Vec<CreatureRollJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<Vec<CreatureDamageJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<CreatureUseLimitJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_effect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_effect_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<CreatureContentJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_record_key: Option<String>,
    #[serde(skip)]
    pub target_entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureOccurrenceProvenanceJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureSpellcastingJson {
    pub entries: Vec<CreatureSpellcastingEntryJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub standalone_spells: Vec<CreatureSpellJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellcastingEntryJson {
    #[serde(skip)]
    pub id: String,
    pub order: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preparation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tradition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slots: Option<Vec<CreatureSpellSlotJson>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spells: Vec<CreatureSpellJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_record_key: Option<String>,
    #[serde(skip)]
    pub target_entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureOccurrenceProvenanceJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<CreatureContentJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellSlotJson {
    pub rank: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepared: Option<Vec<CreaturePreparedSpellJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreaturePreparedSpellJson {
    pub order: u32,
    #[serde(skip)]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expended: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepared: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureUseLimitJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_value: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureOccurrenceContextJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip)]
    pub slot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<CreatureUseLimitJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellJson {
    #[serde(skip)]
    pub id: String,
    pub order: u32,
    pub label: String,
    pub traits: Vec<String>,
    pub action_cost: CreatureActionCostJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_record_key: Option<String>,
    #[serde(skip)]
    pub target_entity_id: Option<String>,
    #[serde(skip)]
    pub parent_entry_id: Option<String>,
    pub context: CreatureOccurrenceContextJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_rank: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counteraction: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ritual: Option<CreatureSpellRitualJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<CreatureSpellAreaJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<CreatureSpellDurationJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defense: Option<CreatureSpellDefenseJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<Vec<CreatureDamageJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<CreatureContentJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CreatureOccurrenceProvenanceJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellRitualJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_casters: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_checks: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellAreaJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellDurationJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sustained: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellDefenseJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureRollJson {
    pub id: String,
    pub label: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureDamageJson {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_modifier: Option<bool>,
}

pub(super) fn creature_presentation(
    creature: &CreatureRecord,
    detail: DetailLevel,
    edition: Option<RecordEditionContextJson>,
    record_relationships: Option<RecordRelationshipLookupJson>,
    include_provenance_evidence: bool,
    teaser: Option<String>,
) -> RecordPresentationJson {
    let include_scan = matches!(
        detail,
        DetailLevel::Preview | DetailLevel::Standard | DetailLevel::Full
    );
    let include_details = matches!(detail, DetailLevel::Standard | DetailLevel::Full);
    let include_content = matches!(detail, DetailLevel::Description | DetailLevel::Full);
    let placement = place_creature_content(creature);
    if detail == DetailLevel::Summary {
        return RecordPresentationJson::Creature {
            teaser: None,
            size: None,
            adjustment: None,
            initiative: None,
            abilities: None,
            defenses: None,
            perception: None,
            languages: None,
            skills: None,
            movement: None,
            resources: None,
            strikes: None,
            actions: None,
            spellcasting: None,
            rituals: None,
            equipment: None,
            lore: None,
            content: None,
            relationships: None,
            provenance: None,
            edition: None,
            record_relationships: None,
            availability: Vec::new(),
            unmodeled_skill_availability: Vec::new(),
            availability_evidence: None,
        };
    }

    let mut strikes = Vec::new();
    let mut actions = Vec::new();
    let mut entries = Vec::new();
    let mut spells = Vec::new();
    if let Some(embedded) = creature.embedded_entities.value.as_value() {
        for occurrence in &embedded.occurrences {
            let label = occurrence_label(occurrence, &embedded.entities);
            match &occurrence.capability {
                CreatureCapability::Strike(capability) => strikes.push(CreatureStrikeJson {
                    id: occurrence.id.as_str().to_string(),
                    order: occurrence.authored_order,
                    label,
                    traits: strings(&capability.traits),
                    action_cost: action_cost(&capability.action_cost),
                    attack_effects: strings(&capability.attack_effects),
                    rolls: include_details.then(|| rolls(&capability.rolls)),
                    damage: include_details
                        .then(|| damage(&capability.damage))
                        .flatten(),
                    content: include_content
                        .then(|| {
                            occurrence_content(
                                creature,
                                &placement,
                                occurrence,
                                detail,
                                include_provenance_evidence,
                            )
                        })
                        .flatten(),
                    target_record_key: target_record_key(occurrence),
                    target_entity_id: target_entity_id(occurrence),
                    provenance: (detail == DetailLevel::Full && include_provenance_evidence)
                        .then(|| occurrence_provenance(occurrence)),
                }),
                CreatureCapability::Action(capability) => actions.push(CreatureActionJson {
                    id: occurrence.id.as_str().to_string(),
                    order: occurrence.authored_order,
                    label,
                    traits: strings(&capability.traits),
                    action_cost: action_cost(&capability.action_cost),
                    category: text(&capability.category),
                    frequency: capability.frequency.as_value().map(frequency),
                    requirements: include_details
                        .then(|| text(&capability.requirements))
                        .flatten(),
                    cost: include_details.then(|| text(&capability.cost)).flatten(),
                    rolls: include_details.then(|| rolls(&capability.rolls)),
                    damage: include_details
                        .then(|| damage(&capability.damage))
                        .flatten(),
                    uses: include_details
                        .then(|| occurrence.context.uses.as_value().map(use_limit))
                        .flatten(),
                    self_effect: include_details
                        .then(|| text(&capability.self_effect))
                        .flatten(),
                    self_effect_label: include_details
                        .then(|| text(&capability.self_effect_label))
                        .flatten(),
                    content: include_content
                        .then(|| {
                            occurrence_content(
                                creature,
                                &placement,
                                occurrence,
                                detail,
                                include_provenance_evidence,
                            )
                        })
                        .flatten(),
                    target_record_key: target_record_key(occurrence),
                    target_entity_id: target_entity_id(occurrence),
                    provenance: (detail == DetailLevel::Full && include_provenance_evidence)
                        .then(|| occurrence_provenance(occurrence)),
                }),
                CreatureCapability::SpellcastingEntry(capability) => {
                    entries.push(CreatureSpellcastingEntryJson {
                        id: occurrence.id.as_str().to_string(),
                        order: occurrence.authored_order,
                        label,
                        preparation: capability.preparation.as_value().map(preparation),
                        tradition: text(&capability.tradition),
                        attack: integer(&capability.attack),
                        dc: integer(&capability.dc),
                        slots: include_details
                            .then(|| spell_slots(&capability.slots))
                            .flatten(),
                        spells: Vec::new(),
                        target_record_key: target_record_key(occurrence),
                        target_entity_id: target_entity_id(occurrence),
                        provenance: (detail == DetailLevel::Full && include_provenance_evidence)
                            .then(|| occurrence_provenance(occurrence)),
                        content: include_content
                            .then(|| {
                                occurrence_content(
                                    creature,
                                    &placement,
                                    occurrence,
                                    detail,
                                    include_provenance_evidence,
                                )
                            })
                            .flatten(),
                    });
                }
                CreatureCapability::Spell(capability) => spells.push(CreatureSpellJson {
                    id: occurrence.id.as_str().to_string(),
                    order: occurrence.authored_order,
                    label,
                    traits: strings(&capability.traits),
                    action_cost: action_cost(&capability.action_cost),
                    target_record_key: match &occurrence.target {
                        CreatureEntityTarget::CanonicalRecord(key) => Some(key.to_string()),
                        CreatureEntityTarget::ActorOwned(_) => None,
                    },
                    target_entity_id: target_entity_id(occurrence),
                    parent_entry_id: match &occurrence.parent {
                        CreatureOccurrenceParent::SpellcastingEntry(parent) => {
                            Some(parent.as_str().to_string())
                        }
                        CreatureOccurrenceParent::Creature => None,
                    },
                    context: occurrence_context(&occurrence.context),
                    base_rank: integer(&capability.base_rank),
                    signature: boolean(&capability.signature),
                    traditions: strings(&capability.traditions),
                    requirements: include_details
                        .then(|| text(&capability.requirements))
                        .flatten(),
                    cost: include_details.then(|| text(&capability.cost)).flatten(),
                    target: include_details.then(|| text(&capability.target)).flatten(),
                    range: include_details.then(|| text(&capability.range)).flatten(),
                    time: include_details.then(|| text(&capability.time)).flatten(),
                    counteraction: include_details
                        .then(|| boolean(&capability.counteraction))
                        .flatten(),
                    ritual: include_details
                        .then(|| {
                            capability
                                .ritual
                                .as_value()
                                .map(|value| CreatureSpellRitualJson {
                                    primary_check: text(&value.primary_check),
                                    secondary_casters: value.secondary_casters.as_value().and_then(
                                        |value| match value {
                                            CreatureSourceScalar::Value(value) => Some(*value),
                                            CreatureSourceScalar::Unsupported(_) => None,
                                        },
                                    ),
                                    secondary_checks: text(&value.secondary_checks),
                                })
                        })
                        .flatten(),
                    area: include_details
                        .then(|| {
                            capability
                                .area
                                .as_value()
                                .map(|value| CreatureSpellAreaJson {
                                    area_type: text(&value.area_type),
                                    value: integer(&value.value),
                                })
                        })
                        .flatten(),
                    duration: include_details
                        .then(|| {
                            capability
                                .duration
                                .as_value()
                                .map(|value| CreatureSpellDurationJson {
                                    value: text(&value.value),
                                    sustained: boolean(&value.sustained),
                                })
                        })
                        .flatten(),
                    defense: include_details
                        .then(|| {
                            capability
                                .defense
                                .as_value()
                                .map(|value| CreatureSpellDefenseJson {
                                    save: value.save.as_value().and_then(|save| match save {
                                        crate::CreatureSpellSave::Fortitude => Some("fortitude"),
                                        crate::CreatureSpellSave::Reflex => Some("reflex"),
                                        crate::CreatureSpellSave::Will => Some("will"),
                                        crate::CreatureSpellSave::Unsupported(_) => None,
                                    }),
                                    basic: boolean(&value.basic),
                                })
                        })
                        .flatten(),
                    damage: include_details
                        .then(|| damage(&capability.damage))
                        .flatten(),
                    content: include_content
                        .then(|| {
                            occurrence_content(
                                creature,
                                &placement,
                                occurrence,
                                detail,
                                include_provenance_evidence,
                            )
                        })
                        .flatten(),
                    provenance: (detail == DetailLevel::Full && include_provenance_evidence)
                        .then(|| occurrence_provenance(occurrence)),
                }),
                CreatureCapability::Equipment(_)
                | CreatureCapability::Lore(_)
                | CreatureCapability::Unsupported(_) => {}
            }
        }
    }
    strikes.sort_by_key(|item| item.order);
    actions.sort_by_key(|item| item.order);
    entries.sort_by_key(|item| item.order);
    spells.sort_by_key(|item| item.order);
    let mut standalone_spells = Vec::new();
    for spell in spells {
        if let Some(parent_entry_id) = spell.parent_entry_id.as_deref()
            && let Some(entry) = entries.iter_mut().find(|entry| entry.id == parent_entry_id)
        {
            entry.spells.push(spell);
        } else {
            standalone_spells.push(spell);
        }
    }

    let availability_evidence = availability_evidence(creature, &placement, detail);
    let availability = product_availability(&availability_evidence, detail);
    let unmodeled_skill_availability = unmodeled_skill_availability(creature, detail);
    RecordPresentationJson::Creature {
        teaser,
        size: include_scan
            .then(|| {
                creature
                    .size
                    .value
                    .as_value()
                    .map(|value| value.as_source().to_string())
            })
            .flatten(),
        adjustment: include_scan
            .then(|| adjustment(&creature.adjustment.value))
            .flatten(),
        initiative: include_scan
            .then(|| initiative(&creature.initiative.value))
            .flatten(),
        abilities: include_scan
            .then(|| abilities(&creature.legacy_abilities.value))
            .flatten(),
        defenses: include_scan
            .then(|| creature.defenses.value.as_value().map(defenses))
            .flatten(),
        perception: include_scan
            .then(|| creature.perception.value.as_value().map(perception))
            .flatten(),
        languages: include_scan
            .then(|| {
                creature
                    .languages
                    .value
                    .as_value()
                    .and_then(|languages| languages.values.as_value())
                    .map(|values| {
                        values
                            .iter()
                            .map(|value| value.as_str().to_string())
                            .collect()
                    })
            })
            .flatten(),
        skills: include_scan
            .then(|| {
                creature
                    .skills
                    .value
                    .as_value()
                    .map(|values| values.iter().map(skill).collect())
            })
            .flatten(),
        movement: include_scan
            .then(|| {
                creature
                    .movement
                    .value
                    .as_value()
                    .map(|values| CreatureMovementJson {
                        modes: values.iter().map(movement).collect(),
                    })
            })
            .flatten(),
        resources: include_scan
            .then(|| {
                creature
                    .resources
                    .value
                    .as_value()
                    .map(|values| values.iter().map(resource).collect())
            })
            .flatten(),
        strikes: include_scan
            .then(|| creature.embedded_entities.value.as_value().map(|_| strikes))
            .flatten(),
        actions: include_scan
            .then(|| creature.embedded_entities.value.as_value().map(|_| actions))
            .flatten(),
        spellcasting: include_scan
            .then(|| {
                creature
                    .embedded_entities
                    .value
                    .as_value()
                    .map(|_| CreatureSpellcastingJson {
                        entries,
                        standalone_spells,
                    })
            })
            .flatten(),
        rituals: include_details.then(|| rituals(creature)).flatten(),
        equipment: include_scan
            .then(|| equipment(creature, &placement, detail, include_provenance_evidence))
            .flatten(),
        lore: include_scan
            .then(|| lore(creature, &placement, detail, include_provenance_evidence))
            .flatten(),
        content: include_content
            .then(|| all_content(creature, &placement, detail, include_provenance_evidence))
            .flatten(),
        relationships: include_provenance_evidence
            .then(|| relationships(&placement))
            .flatten(),
        provenance: (detail == DetailLevel::Full && include_provenance_evidence).then(|| {
            CreatureProvenanceJson {
                source_path: creature.provenance.source_path.clone(),
                source_contract_version: creature.provenance.source_contract_version.clone(),
                source_system_version: creature.provenance.source_system_version.clone(),
                source_upstream_commit: creature.provenance.source_upstream_commit.clone(),
                facts: fact_provenance_set(creature),
            }
        }),
        edition,
        record_relationships,
        availability,
        unmodeled_skill_availability,
        availability_evidence: include_provenance_evidence
            .then_some(availability_evidence)
            .filter(|evidence| !evidence.is_empty()),
    }
}

fn fact_provenance_set(creature: &CreatureRecord) -> CreatureFactProvenanceSetJson {
    CreatureFactProvenanceSetJson {
        level: fact_provenance(&creature.level.provenance),
        rarity: fact_provenance(&creature.rarity.provenance),
        traits: fact_provenance(&creature.traits.provenance),
        size: fact_provenance(&creature.size.provenance),
        publication: fact_provenance(&creature.publication.provenance),
        adjustment: fact_provenance(&creature.adjustment.provenance),
        source_alliance: fact_provenance(&creature.source_alliance.provenance),
        perception: fact_provenance(&creature.perception.provenance),
        initiative: fact_provenance(&creature.initiative.provenance),
        languages: fact_provenance(&creature.languages.provenance),
        skills: fact_provenance(&creature.skills.provenance),
        abilities: fact_provenance(&creature.legacy_abilities.provenance),
        defenses: fact_provenance(&creature.defenses.provenance),
        movement: fact_provenance(&creature.movement.provenance),
        resources: fact_provenance(&creature.resources.provenance),
        embedded_entities: fact_provenance(&creature.embedded_entities.provenance),
    }
}

fn fact_provenance(value: &crate::CreatureFactProvenance) -> CreatureFactProvenanceJson {
    match value {
        crate::CreatureFactProvenance::Source(field) => CreatureFactProvenanceJson::Source {
            field: match field {
                crate::CreatureSourceField::Identity => "identity",
                crate::CreatureSourceField::Level => "level",
                crate::CreatureSourceField::Rarity => "rarity",
                crate::CreatureSourceField::Traits => "traits",
                crate::CreatureSourceField::Size => "size",
                crate::CreatureSourceField::Publication => "publication",
                crate::CreatureSourceField::Adjustment => "adjustment",
                crate::CreatureSourceField::SourceAlliance => "source_alliance",
                crate::CreatureSourceField::Perception => "perception",
                crate::CreatureSourceField::Initiative => "initiative",
                crate::CreatureSourceField::Languages => "languages",
                crate::CreatureSourceField::Skills => "skills",
                crate::CreatureSourceField::LegacyAbilities => "legacy_abilities",
                crate::CreatureSourceField::Defenses => "defenses",
                crate::CreatureSourceField::Movement => "movement",
                crate::CreatureSourceField::Resources => "resources",
                crate::CreatureSourceField::EmbeddedEntities => "embedded_entities",
            },
        },
        crate::CreatureFactProvenance::Derived(derivation) => CreatureFactProvenanceJson::Derived {
            derivation: match derivation {
                crate::CreatureDerivation::RecordClassificationProjection => {
                    "record_classification_projection"
                }
                crate::CreatureDerivation::LegacyActorProjection => "legacy_actor_projection",
                crate::CreatureDerivation::DisplayLabel => "display_label",
            },
        },
    }
}

fn defenses(value: &crate::CreatureDefenses) -> CreatureDefensesJson {
    CreatureDefensesJson {
        ac: value
            .armor_class
            .as_value()
            .map(|ac| CreatureArmorClassJson {
                value: integer(&ac.value),
                details: note(&ac.details),
            }),
        hp: value.hit_points.as_value().map(|hp| CreatureHitPointsJson {
            value: hp.value.as_value().and_then(|value| match value {
                CreatureNumber::Integer(value) => Some(*value),
                CreatureNumber::Unsupported(_) => None,
            }),
            maximum: integer(&hp.maximum),
            temporary: integer(&hp.temporary),
            temporary_maximum: integer(&hp.temporary_maximum),
            details: note(&hp.details),
        }),
        hardness: integer(&value.hardness),
        shield: value.shield.as_value().map(|shield| CreatureShieldJson {
            armor_class_bonus: integer(&shield.armor_class_bonus),
            broken_threshold: integer(&shield.broken_threshold),
            hardness: integer(&shield.hardness),
            maximum_hit_points: integer(&shield.maximum_hit_points),
            serialized_hit_points: integer(&shield.serialized_hit_points),
            current_policy: "serialized_hit_points_are_provenance_only",
        }),
        saves: value.saves.as_value().map(|saves| CreatureSavesJson {
            fortitude: saves.fortitude.as_value().map(save),
            reflex: saves.reflex.as_value().map(save),
            will: saves.will.as_value().map(save),
        }),
        all_saves_note: note(&value.all_saves_note),
        immunities: iwr_values(&value.immunities),
        resistances: iwr_values(&value.resistances),
        weaknesses: iwr_values(&value.weaknesses),
    }
}

fn adjustment(value: &FactValue<CreatureAdjustment>) -> Option<String> {
    match value.as_value()? {
        CreatureAdjustment::Elite => Some("elite".to_string()),
        CreatureAdjustment::Weak => Some("weak".to_string()),
        CreatureAdjustment::Unsupported(_) => None,
    }
}

fn initiative(value: &FactValue<crate::CreatureInitiative>) -> Option<CreatureInitiativeJson> {
    match value.as_value()?.statistic.as_value()? {
        crate::CreatureInitiativeStatistic::Named(value) => Some(CreatureInitiativeJson {
            statistic: value.as_str().to_string(),
        }),
        crate::CreatureInitiativeStatistic::Unsupported(_) => None,
    }
}

fn abilities(value: &FactValue<crate::CreatureLegacyAbilities>) -> Option<CreatureAbilitiesJson> {
    let value = value.as_value()?;
    Some(CreatureAbilitiesJson {
        strength: integer(&value.strength),
        dexterity: integer(&value.dexterity),
        constitution: integer(&value.constitution),
        intelligence: integer(&value.intelligence),
        wisdom: integer(&value.wisdom),
        charisma: integer(&value.charisma),
    })
}

fn rituals(creature: &CreatureRecord) -> Option<CreatureRitualsJson> {
    let embedded = creature.embedded_entities.value.as_value()?;
    let context = embedded.actor_spellcasting.as_value()?;
    Some(CreatureRitualsJson {
        difficulty_class: context.rituals_dc.as_value().and_then(|value| match value {
            CreatureSourceScalar::Value(value) => Some(*value),
            CreatureSourceScalar::Unsupported(_) => None,
        }),
    })
}

fn equipment(
    creature: &CreatureRecord,
    placement: &CreatureContentPlacement,
    detail: DetailLevel,
    include_provenance_evidence: bool,
) -> Option<Vec<CreatureEquipmentJson>> {
    let embedded = creature.embedded_entities.value.as_value()?;
    let mut values = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| {
            let CreatureCapability::Equipment(value) = &occurrence.capability else {
                return None;
            };
            Some(CreatureEquipmentJson {
                id: occurrence.id.as_str().to_string(),
                order: occurrence.authored_order,
                label: occurrence_label(occurrence, &embedded.entities),
                traits: strings(&value.traits),
                level: integer(&value.level),
                usage: text(&value.usage),
                quantity: integer(&value.quantity),
                uses: value.uses.as_value().map(use_limit),
                target_record_key: target_record_key(occurrence),
                target_entity_id: target_entity_id(occurrence),
                provenance: (detail == DetailLevel::Full && include_provenance_evidence)
                    .then(|| occurrence_provenance(occurrence)),
                content: (detail == DetailLevel::Full)
                    .then(|| {
                        occurrence_content(
                            creature,
                            placement,
                            occurrence,
                            detail,
                            include_provenance_evidence,
                        )
                    })
                    .flatten(),
            })
        })
        .collect::<Vec<_>>();
    values.sort_by_key(|value| value.order);
    (!values.is_empty()).then_some(values)
}

fn lore(
    creature: &CreatureRecord,
    placement: &CreatureContentPlacement,
    detail: DetailLevel,
    include_provenance_evidence: bool,
) -> Option<Vec<CreatureLoreJson>> {
    let embedded = creature.embedded_entities.value.as_value()?;
    let mut values = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| {
            let CreatureCapability::Lore(value) = &occurrence.capability else {
                return None;
            };
            Some(CreatureLoreJson {
                id: occurrence.id.as_str().to_string(),
                order: occurrence.authored_order,
                label: occurrence_label(occurrence, &embedded.entities),
                modifier: integer(&value.modifier),
                target_record_key: target_record_key(occurrence),
                target_entity_id: target_entity_id(occurrence),
                provenance: (detail == DetailLevel::Full && include_provenance_evidence)
                    .then(|| occurrence_provenance(occurrence)),
                content: (detail == DetailLevel::Full)
                    .then(|| {
                        occurrence_content(
                            creature,
                            placement,
                            occurrence,
                            detail,
                            include_provenance_evidence,
                        )
                    })
                    .flatten(),
            })
        })
        .collect::<Vec<_>>();
    values.sort_by_key(|value| value.order);
    (!values.is_empty()).then_some(values)
}

fn occurrence_content(
    creature: &CreatureRecord,
    placement: &CreatureContentPlacement,
    occurrence: &CreatureEntityOccurrence,
    detail: DetailLevel,
    include_provenance_evidence: bool,
) -> Option<Vec<CreatureContentJson>> {
    let values = placement
        .documents_for_occurrence(creature, &occurrence.id)
        .filter_map(|document| {
            content_json(
                document,
                detail == DetailLevel::Full && include_provenance_evidence,
            )
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then_some(values)
}

fn all_content(
    creature: &CreatureRecord,
    placement: &CreatureContentPlacement,
    detail: DetailLevel,
    include_provenance_evidence: bool,
) -> Option<Vec<CreatureContentJson>> {
    let mut documents = if detail == DetailLevel::Description {
        placement.all_documents(creature).collect::<Vec<_>>()
    } else {
        placement.general_documents(creature).collect::<Vec<_>>()
    };
    documents.sort_by_key(|document| (document.authored_order, document.id.content_key.as_str()));
    let values = documents
        .into_iter()
        .filter_map(|document| {
            content_json(
                document,
                detail == DetailLevel::Full && include_provenance_evidence,
            )
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then_some(values)
}

fn content_json(
    document: &crate::OwnedRichContentDocument,
    full: bool,
) -> Option<CreatureContentJson> {
    Some(CreatureContentJson {
        content_key: document.id.content_key.as_str().to_string(),
        owner: match &document.owner {
            ContentOwner::Record(key) => CreatureContentOwnerJson::Record {
                record_key: key.to_string(),
            },
            ContentOwner::CreatureEntity(id) => CreatureContentOwnerJson::Entity {
                entity_id: id.as_str().to_string(),
            },
            ContentOwner::CreatureOccurrence(id) => CreatureContentOwnerJson::Occurrence {
                occurrence_id: id.as_str().to_string(),
            },
            ContentOwner::HazardEntity(_)
            | ContentOwner::HazardOccurrence(_)
            | ContentOwner::Child(_) => return None,
        },
        role: content_role(document.role),
        authored_order: document.authored_order,
        label: document.label.clone(),
        content_hash: document.content_hash.as_str().to_string(),
        visibility: document.visibility.as_str(),
        document: project_presentation_content(&document.document),
        provenance: full.then(|| CreatureContentProvenanceJson {
            source_record_key: document.provenance.source_record_key.to_string(),
            relative_source_path: document.provenance.relative_source_path.clone(),
            field_family: document.provenance.field_or_pointer_family.clone(),
            nested_source_id: document.provenance.nested_source_id.clone(),
        }),
    })
}

fn content_role(value: ContentRole) -> &'static str {
    match value {
        ContentRole::PrimaryDescription => "primary_description",
        ContentRole::Summary => "summary",
        ContentRole::SupplementalRules => "supplemental_rules",
        ContentRole::EmbeddedCapability => "embedded_capability",
        ContentRole::JournalPage => "journal_page",
        ContentRole::TableResult => "table_result",
        ContentRole::GeneratedNarrative => "generated_narrative",
        ContentRole::Provenance => "provenance",
    }
}

fn relationships(placement: &CreatureContentPlacement) -> Option<Vec<CreatureRelationshipJson>> {
    let values = placement
        .relationships()
        .iter()
        .map(|value| CreatureRelationshipJson {
            source_occurrence_id: value.source.as_str().to_string(),
            kind: match value.kind {
                CreatureEntityRelationshipKind::GrantedBy => "granted_by",
                CreatureEntityRelationshipKind::ItemGrant => "item_grant",
                CreatureEntityRelationshipKind::LinkedWeapon => "linked_weapon",
                CreatureEntityRelationshipKind::PreparedSpell => "prepared_spell",
            },
            target: match &value.target {
                CreatureRelationshipTarget::Occurrence(id) => {
                    CreatureRelationshipTargetJson::Occurrence {
                        occurrence_id: id.as_str().to_string(),
                    }
                }
                CreatureRelationshipTarget::UnresolvedNestedSourceId(id) => {
                    CreatureRelationshipTargetJson::UnresolvedNestedSource {
                        source_id: id.as_str().to_string(),
                    }
                }
            },
            source_path: value.source_path.clone(),
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then_some(values)
}

fn availability_evidence(
    creature: &CreatureRecord,
    placement: &CreatureContentPlacement,
    detail: DetailLevel,
) -> Vec<CreatureAvailabilityEvidenceJson> {
    if detail == DetailLevel::Summary {
        return Vec::new();
    }
    let mut values = Vec::new();
    if detail != DetailLevel::Description
        && let Some(skills) = creature.skills.value.as_value()
    {
        for skill in skills {
            if let Some(unmodeled) = skill.unmodeled.as_value() {
                values.push(CreatureAvailabilityEvidenceJson {
                    state: CreatureAvailabilityStateJson::Unsupported,
                    field: CreatureAvailabilityFieldJson::UnmodeledSkill,
                    component_id: Some(skill.id.as_str().to_string()),
                    authored_key: Some(unmodeled.authored_key.clone()),
                    source_value: None,
                    source_shape: None,
                    source_reason: None,
                    source_path: None,
                    message: "The source supplied an unrecognized skill key.".to_string(),
                });
            }
        }
    }
    if detail != DetailLevel::Description {
        collect_unsupported_availability(
            creature,
            matches!(detail, DetailLevel::Standard | DetailLevel::Full),
            &mut values,
        );
    }
    if detail != DetailLevel::Description {
        for (field, label, value) in [
            (
                CreatureAvailabilityFieldJson::Size,
                "size",
                fact_state(&creature.size.value),
            ),
            (
                CreatureAvailabilityFieldJson::Defenses,
                "defenses",
                fact_state(&creature.defenses.value),
            ),
            (
                CreatureAvailabilityFieldJson::Perception,
                "perception",
                fact_state(&creature.perception.value),
            ),
            (
                CreatureAvailabilityFieldJson::EmbeddedEntities,
                "embedded_entities",
                fact_state(&creature.embedded_entities.value),
            ),
        ] {
            if let Some(state) = value {
                values.push(CreatureAvailabilityEvidenceJson {
                    state,
                    field,
                    component_id: None,
                    authored_key: None,
                    source_value: None,
                    source_shape: None,
                    source_reason: None,
                    source_path: None,
                    message: format!("{label} data is unavailable."),
                });
            }
        }
    }
    if let Some(embedded) = creature.embedded_entities.value.as_value() {
        for occurrence in &embedded.occurrences {
            if let Some(failure) = placement.failure(&occurrence.id) {
                values.push(CreatureAvailabilityEvidenceJson {
                    state: CreatureAvailabilityStateJson::Unsupported,
                    field: CreatureAvailabilityFieldJson::ContentAssociation,
                    component_id: Some(occurrence.id.as_str().to_string()),
                    authored_key: None,
                    source_value: None,
                    source_shape: None,
                    source_reason: None,
                    source_path: None,
                    message: match failure {
                        CreatureContentAssociationFailure::DuplicateOccurrenceIdentity => "Content is unavailable because the occurrence identity is duplicated.",
                        CreatureContentAssociationFailure::AmbiguousEntityTarget => "Content is unavailable because the entity target is ambiguous.",
                        CreatureContentAssociationFailure::DuplicateContentIdentity => "Content is unavailable because the stable content identity is duplicated.",
                        CreatureContentAssociationFailure::AmbiguousOwnerAssociation => "Content is unavailable because the authored owner association is ambiguous.",
                    }.to_string(),
                });
            }
        }
    }
    values.sort_by(|left, right| {
        left.field
            .cmp(&right.field)
            .then_with(|| left.state.cmp(&right.state))
            .then_with(|| left.component_id.cmp(&right.component_id))
            .then_with(|| left.authored_key.cmp(&right.authored_key))
            .then_with(|| left.source_value.cmp(&right.source_value))
            .then_with(|| left.source_shape.cmp(&right.source_shape))
            .then_with(|| left.source_reason.cmp(&right.source_reason))
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.message.cmp(&right.message))
    });
    values
}

fn product_availability(
    evidence: &[CreatureAvailabilityEvidenceJson],
    detail: DetailLevel,
) -> Vec<CreatureAvailabilityJson> {
    if detail == DetailLevel::Preview {
        return Vec::new();
    }
    let mut values = evidence
        .iter()
        .filter(|cause| {
            !matches!(
                cause.field,
                CreatureAvailabilityFieldJson::UnmodeledSkill
                    | CreatureAvailabilityFieldJson::ResourceSerializedValue
                    | CreatureAvailabilityFieldJson::ResourceSourceDrift
                    | CreatureAvailabilityFieldJson::SpellSlotSerializedValue
                    | CreatureAvailabilityFieldJson::UnsupportedMechanic
            )
        })
        .map(|cause| CreatureAvailabilityJson {
            state: cause.state,
            field: cause.field,
            authored_key: cause.authored_key.clone(),
            source_value: product_source_value(cause),
            message: cause.message.clone(),
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.field
            .cmp(&right.field)
            .then_with(|| left.state.cmp(&right.state))
            .then_with(|| left.authored_key.cmp(&right.authored_key))
            .then_with(|| left.source_value.cmp(&right.source_value))
            .then_with(|| left.message.cmp(&right.message))
    });
    values.dedup();
    values
}

fn unmodeled_skill_availability(
    creature: &CreatureRecord,
    detail: DetailLevel,
) -> Vec<CreatureUnmodeledSkillAvailabilityJson> {
    if !matches!(detail, DetailLevel::Standard | DetailLevel::Full) {
        return Vec::new();
    }
    creature
        .skills
        .value
        .as_value()
        .into_iter()
        .flatten()
        .filter_map(|skill| {
            let unmodeled = skill.unmodeled.as_value()?;
            Some(CreatureUnmodeledSkillAvailabilityJson {
                skill_id: skill.id.as_str().to_string(),
                authored_order: skill.authored_order,
                authored_key: unmodeled.authored_key.clone(),
                modifier: integer_presence(&unmodeled.base),
                message: "The source supplied an unrecognized skill key.",
            })
        })
        .collect()
}

fn product_source_value(cause: &CreatureAvailabilityEvidenceJson) -> Option<String> {
    let value = cause.source_value.as_deref()?;
    (cause.source_shape == Some("string") && !matches!(value, "" | "false" | "null" | "0"))
        .then(|| value.to_string())
}

fn collect_unsupported_availability(
    creature: &CreatureRecord,
    include_details: bool,
    values: &mut Vec<CreatureAvailabilityEvidenceJson>,
) {
    if let Some(CreatureAdjustment::Unsupported(source)) = creature.adjustment.value.as_value() {
        push_unsupported(
            values,
            CreatureAvailabilityFieldJson::Adjustment,
            None,
            source,
            None,
            "The source supplied an unsupported creature adjustment.",
        );
    }
    if let Some(crate::CreatureInitiative {
        statistic: FactValue::Value(crate::CreatureInitiativeStatistic::Unsupported(source)),
    }) = creature.initiative.value.as_value()
    {
        push_unsupported(
            values,
            CreatureAvailabilityFieldJson::InitiativeStatistic,
            None,
            source,
            None,
            "The source supplied an unsupported initiative statistic.",
        );
    }
    if let Some(crate::CreatureSourceAlliance::Unsupported(source)) =
        creature.source_alliance.value.as_value()
    {
        push_unsupported(
            values,
            CreatureAvailabilityFieldJson::SourceAlliance,
            None,
            source,
            None,
            "The source supplied an unsupported alliance value.",
        );
    }
    if let Some(defenses) = creature.defenses.value.as_value()
        && let Some(hit_points) = defenses.hit_points.as_value()
        && let Some(CreatureNumber::Unsupported(source)) = hit_points.value.as_value()
    {
        push_unsupported(
            values,
            CreatureAvailabilityFieldJson::HitPointsValue,
            Some("hit_points".to_string()),
            source,
            None,
            "The source supplied an unsupported hit point value.",
        );
    }
    if let Some(perception) = creature.perception.value.as_value()
        && let Some(senses) = perception.senses.as_value()
    {
        for sense in senses {
            if let Some(crate::SenseAcuity::Unsupported(source)) = sense.acuity.as_value() {
                push_unsupported(
                    values,
                    CreatureAvailabilityFieldJson::SenseAcuity,
                    Some(sense.id.as_str().to_string()),
                    source,
                    None,
                    "The source supplied an unsupported sense acuity.",
                );
            }
        }
    }
    if let Some(skills) = creature.skills.value.as_value() {
        for skill in skills {
            if let Some(variants) = skill.variants.as_value() {
                for variant in variants {
                    if let Some(predicates) = variant.predicate.as_value() {
                        for predicate in predicates {
                            if let crate::CreaturePredicate::Unsupported(source) = predicate {
                                push_unsupported(
                                    values,
                                    CreatureAvailabilityFieldJson::SkillPredicate,
                                    Some(format!("{}/{}", skill.id.as_str(), variant.id.as_str())),
                                    source,
                                    None,
                                    "The source supplied an unsupported skill predicate.",
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(movement) = creature.movement.value.as_value() {
        for mode in movement {
            if let CreatureMovementMode::Unsupported(source) = &mode.mode {
                push_unsupported(
                    values,
                    CreatureAvailabilityFieldJson::MovementMode,
                    Some(mode.id.as_str().to_string()),
                    source,
                    None,
                    "The source supplied an unsupported movement mode.",
                );
            }
        }
    }
    if let Some(resources) = creature.resources.value.as_value() {
        for resource in resources {
            if let Some(CreatureResourceAmount::Unsupported(source)) = resource.maximum.as_value() {
                push_unsupported(
                    values,
                    CreatureAvailabilityFieldJson::ResourceMaximum,
                    Some(resource.id.as_str().to_string()),
                    source,
                    None,
                    "The source supplied an unsupported resource maximum.",
                );
            }
            if let Some(CreatureResourceAmount::Unsupported(source)) =
                resource.serialized_value.as_value()
            {
                push_unsupported(
                    values,
                    CreatureAvailabilityFieldJson::ResourceSerializedValue,
                    Some(resource.id.as_str().to_string()),
                    source,
                    None,
                    "The source supplied an unsupported serialized resource value.",
                );
            }
            if let Some(drift) = resource.source_drift.as_value() {
                for fact in drift {
                    push_unsupported(
                        values,
                        CreatureAvailabilityFieldJson::ResourceSourceDrift,
                        Some(resource.id.as_str().to_string()),
                        &fact.value,
                        None,
                        "The source supplied a drifting resource value.",
                    );
                }
            }
        }
    }
    let Some(embedded) = creature.embedded_entities.value.as_value() else {
        return;
    };
    if let Some(actor_spellcasting) = embedded.actor_spellcasting.as_value() {
        if include_details
            && let Some(CreatureSourceScalar::Unsupported(source)) =
                actor_spellcasting.rituals_dc.as_value()
        {
            push_unsupported(
                values,
                CreatureAvailabilityFieldJson::RitualDifficultyClass,
                Some("actor_spellcasting".to_string()),
                source,
                None,
                "The source supplied an unsupported ritual difficulty class.",
            );
        }
        push_unsupported_notes(
            values,
            "actor_spellcasting",
            &actor_spellcasting.unsupported_notes,
        );
    }
    for occurrence in &embedded.occurrences {
        let occurrence_id = occurrence.id.as_str();
        push_unsupported_notes(
            values,
            occurrence_id,
            capability_unsupported_notes(&occurrence.capability),
        );
        match &occurrence.capability {
            CreatureCapability::Strike(capability) => {
                collect_action_cost(values, occurrence_id, &capability.action_cost);
                if include_details {
                    collect_damage(values, occurrence_id, &capability.damage);
                }
            }
            CreatureCapability::Action(capability) => {
                collect_action_cost(values, occurrence_id, &capability.action_cost);
                if include_details {
                    collect_damage(values, occurrence_id, &capability.damage);
                }
            }
            CreatureCapability::SpellcastingEntry(capability) => {
                if let Some(CreatureSpellPreparation::Unsupported(source)) =
                    capability.preparation.as_value()
                {
                    push_unsupported(
                        values,
                        CreatureAvailabilityFieldJson::SpellPreparation,
                        Some(occurrence_id.to_string()),
                        source,
                        None,
                        "The source supplied an unsupported spell preparation.",
                    );
                }
                if include_details {
                    collect_spell_slots(values, occurrence_id, &capability.slots);
                }
            }
            CreatureCapability::Spell(capability) => {
                collect_action_cost(values, occurrence_id, &capability.action_cost);
                if include_details
                    && let Some(ritual) = capability.ritual.as_value()
                    && let Some(CreatureSourceScalar::Unsupported(source)) =
                        ritual.secondary_casters.as_value()
                {
                    push_unsupported(
                        values,
                        CreatureAvailabilityFieldJson::SpellRitualSecondaryCasters,
                        Some(occurrence_id.to_string()),
                        source,
                        None,
                        "The source supplied an unsupported ritual secondary-caster value.",
                    );
                }
                if include_details
                    && let Some(defense) = capability.defense.as_value()
                    && let Some(crate::CreatureSpellSave::Unsupported(source)) =
                        defense.save.as_value()
                {
                    push_unsupported(
                        values,
                        CreatureAvailabilityFieldJson::SpellDefenseSave,
                        Some(occurrence_id.to_string()),
                        source,
                        None,
                        "The source supplied an unsupported spell defense save.",
                    );
                }
                if include_details {
                    collect_damage(values, occurrence_id, &capability.damage);
                }
            }
            CreatureCapability::Unsupported(capability) => {
                values.push(CreatureAvailabilityEvidenceJson {
                    state: CreatureAvailabilityStateJson::Unsupported,
                    field: CreatureAvailabilityFieldJson::UnsupportedCapability,
                    component_id: Some(occurrence_id.to_string()),
                    authored_key: Some(capability.source_item_type.clone()),
                    source_value: None,
                    source_shape: None,
                    source_reason: None,
                    source_path: None,
                    message: "The source supplied an unsupported creature capability.".to_string(),
                })
            }
            CreatureCapability::Equipment(_) | CreatureCapability::Lore(_) => {}
        }
    }
}

fn collect_action_cost(
    values: &mut Vec<CreatureAvailabilityEvidenceJson>,
    occurrence_id: &str,
    action_cost: &CreatureActionCost,
) {
    if let CreatureActionCost::Unsupported(source) = action_cost {
        push_unsupported(
            values,
            CreatureAvailabilityFieldJson::ActionCost,
            Some(occurrence_id.to_string()),
            source,
            None,
            "The source supplied an unsupported action cost.",
        );
    }
}

fn collect_spell_slots(
    values: &mut Vec<CreatureAvailabilityEvidenceJson>,
    occurrence_id: &str,
    slots: &FactValue<Vec<CreatureSpellSlot>>,
) {
    let Some(slots) = slots.as_value() else {
        return;
    };
    for slot in slots {
        let component_id = format!("{occurrence_id}/rank-{}", slot.rank);
        if let Some(CreatureSourceScalar::Unsupported(source)) = slot.maximum.as_value() {
            push_unsupported(
                values,
                CreatureAvailabilityFieldJson::SpellSlotMaximum,
                Some(component_id.clone()),
                source,
                None,
                "The source supplied an unsupported spell slot maximum.",
            );
        }
        if let Some(CreatureSourceScalar::Unsupported(source)) = slot.serialized_value.as_value() {
            push_unsupported(
                values,
                CreatureAvailabilityFieldJson::SpellSlotSerializedValue,
                Some(component_id.clone()),
                source,
                None,
                "The source supplied an unsupported serialized spell slot value.",
            );
        }
        if let Some(prepared) = slot.prepared.as_value() {
            for (index, prepared) in prepared.iter().enumerate() {
                if let CreaturePreparedSpellSlot::Unsupported(source) = prepared {
                    push_unsupported(
                        values,
                        CreatureAvailabilityFieldJson::PreparedSpellSlot,
                        Some(format!("{component_id}/prepared-{index}")),
                        source,
                        None,
                        "The source supplied an unsupported prepared spell slot.",
                    );
                }
            }
        }
    }
}

fn collect_damage(
    values: &mut Vec<CreatureAvailabilityEvidenceJson>,
    occurrence_id: &str,
    damage: &FactValue<Vec<CreatureDamage>>,
) {
    let Some(damage) = damage.as_value() else {
        return;
    };
    for part in damage {
        let component_id = format!("{occurrence_id}/{}", part.id);
        if let Some(kinds) = part.kinds.as_value() {
            for kind in kinds {
                if let CreatureDamageKind::Unsupported(source) = kind {
                    push_unsupported(
                        values,
                        CreatureAvailabilityFieldJson::DamageKind,
                        Some(component_id.clone()),
                        source,
                        None,
                        "The source supplied an unsupported damage kind.",
                    );
                }
            }
        }
        if let Some(CreatureSourceScalar::Unsupported(source)) = part.apply_modifier.as_value() {
            push_unsupported(
                values,
                CreatureAvailabilityFieldJson::DamageApplyModifier,
                Some(component_id),
                source,
                None,
                "The source supplied an unsupported damage modifier policy.",
            );
        }
    }
}

fn capability_unsupported_notes(capability: &CreatureCapability) -> &[UnsupportedMechanicNote] {
    match capability {
        CreatureCapability::Strike(value) => &value.unsupported_notes,
        CreatureCapability::Action(value) => &value.unsupported_notes,
        CreatureCapability::SpellcastingEntry(value) => &value.unsupported_notes,
        CreatureCapability::Spell(value) => &value.unsupported_notes,
        CreatureCapability::Equipment(value) => &value.unsupported_notes,
        CreatureCapability::Lore(value) => &value.unsupported_notes,
        CreatureCapability::Unsupported(value) => &value.unsupported_notes,
    }
}

fn push_unsupported_notes(
    values: &mut Vec<CreatureAvailabilityEvidenceJson>,
    component_id: &str,
    notes: &[UnsupportedMechanicNote],
) {
    for note in notes {
        push_unsupported(
            values,
            CreatureAvailabilityFieldJson::UnsupportedMechanic,
            Some(component_id.to_string()),
            &note.value,
            Some(note.source_path.clone()),
            "The source supplied an unsupported mechanic.",
        );
    }
}

fn push_unsupported(
    values: &mut Vec<CreatureAvailabilityEvidenceJson>,
    field: CreatureAvailabilityFieldJson,
    component_id: Option<String>,
    source: &UnsupportedSourceValue,
    source_path: Option<String>,
    message: &str,
) {
    values.push(CreatureAvailabilityEvidenceJson {
        state: CreatureAvailabilityStateJson::Unsupported,
        field,
        component_id,
        authored_key: None,
        source_value: Some(source.value.clone()),
        source_shape: Some(unsupported_shape(source.shape)),
        source_reason: Some(unsupported_reason(source.reason)),
        source_path,
        message: message.to_string(),
    });
}

fn unsupported_shape(value: crate::UnsupportedSourceShape) -> &'static str {
    match value {
        crate::UnsupportedSourceShape::Missing => "missing",
        crate::UnsupportedSourceShape::Null => "null",
        crate::UnsupportedSourceShape::String => "string",
        crate::UnsupportedSourceShape::Number => "number",
        crate::UnsupportedSourceShape::Boolean => "boolean",
        crate::UnsupportedSourceShape::Array => "array",
        crate::UnsupportedSourceShape::Object => "object",
    }
}

fn unsupported_reason(value: crate::UnsupportedSourceReason) -> &'static str {
    match value {
        crate::UnsupportedSourceReason::OpenVocabulary => "open_vocabulary",
        crate::UnsupportedSourceReason::AmbiguousLegacyShape => "ambiguous_legacy_shape",
        crate::UnsupportedSourceReason::InvalidPredicate => "invalid_predicate",
        crate::UnsupportedSourceReason::NonCanonicalRuntimeValue => "non_canonical_runtime_value",
        crate::UnsupportedSourceReason::SourceFieldDrift => "source_field_drift",
    }
}

fn fact_state<T>(value: &FactValue<T>) -> Option<CreatureAvailabilityStateJson> {
    match value {
        FactValue::Missing => Some(CreatureAvailabilityStateJson::Missing),
        FactValue::Null => Some(CreatureAvailabilityStateJson::Null),
        FactValue::Value(_) => None,
    }
}

fn save(value: &CreatureSave) -> CreatureSaveJson {
    CreatureSaveJson {
        id: value.id.as_str().to_string(),
        value: integer(&value.value),
        details: note(&value.details),
    }
}

fn iwr_values(values: &FactValue<Vec<CreatureIwr>>) -> Option<Vec<CreatureIwrJson>> {
    values.as_value().map(|values| {
        values
            .iter()
            .map(|value| CreatureIwrJson {
                id: value.id.as_str().to_string(),
                order: value.authored_order,
                iwr_type: value.iwr_type.as_str().to_string(),
                value: integer(&value.value),
                exceptions: value
                    .exceptions
                    .as_value()
                    .map(|values| {
                        values
                            .iter()
                            .map(|value| value.as_str().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
                double_vs: value
                    .double_vs
                    .as_value()
                    .map(|values| {
                        values
                            .iter()
                            .map(|value| value.as_str().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
                apply_once: boolean(&value.apply_once),
            })
            .collect()
    })
}

fn perception(value: &crate::CreaturePerception) -> CreaturePerceptionJson {
    CreaturePerceptionJson {
        modifier: integer(&value.modifier),
        details: note(&value.details),
        has_vision: boolean(&value.has_vision),
        senses: value.senses.as_value().map(|values| {
            values
                .iter()
                .map(|sense| CreatureSenseJson {
                    id: sense.id.as_str().to_string(),
                    order: sense.authored_order,
                    kind: sense.sense_type.as_str().to_string(),
                    acuity: sense.acuity.as_value().map(|acuity| match acuity {
                        crate::SenseAcuity::Precise => "precise".to_string(),
                        crate::SenseAcuity::Imprecise => "imprecise".to_string(),
                        crate::SenseAcuity::Vague => "vague".to_string(),
                        crate::SenseAcuity::Unsupported(value) => value.value.clone(),
                    }),
                    range_feet: integer(&sense.range),
                })
                .collect()
        }),
    }
}

fn skill(value: &CreatureSkill) -> CreatureSkillJson {
    CreatureSkillJson {
        id: value.id.as_str().to_string(),
        order: value.authored_order,
        source_entries: value
            .source_entries
            .iter()
            .map(|entry| CreatureSkillSourceEntryJson {
                authored_key: entry.authored_key.clone(),
                modifier: integer_presence(&entry.modifier),
            })
            .collect(),
        slug: value.kind.source_slug().to_string(),
        label: value.label.clone(),
        modifier: value
            .unmodeled
            .as_value()
            .is_none()
            .then(|| integer(&value.modifier))
            .flatten(),
        note: note(&value.note),
        variants: value
            .variants
            .as_value()
            .map(|values| values.iter().map(skill_variant).collect())
            .unwrap_or_default(),
        source_item_id: value
            .source_item_id
            .as_value()
            .map(|value| value.as_str().to_string()),
        unmodeled: value
            .unmodeled
            .as_value()
            .map(|unmodeled| CreatureUnmodeledSkillJson {
                authored_key: unmodeled.authored_key.clone(),
                base: integer_presence(&unmodeled.base),
                reason: match unmodeled.reason {
                    CreatureUnmodeledSkillReason::UnknownAuthoredKey => "unknown_authored_key",
                },
            }),
    }
}

fn integer_presence(value: &FactValue<i64>) -> CreatureIntegerPresenceJson {
    match value {
        FactValue::Missing => CreatureIntegerPresenceJson::Missing,
        FactValue::Null => CreatureIntegerPresenceJson::Null,
        FactValue::Value(value) => CreatureIntegerPresenceJson::Value(*value),
    }
}

fn skill_variant(value: &CreatureSkillVariant) -> CreatureSkillVariantJson {
    CreatureSkillVariantJson {
        id: value.id.as_str().to_string(),
        order: value.authored_order,
        modifier: integer(&value.modifier),
        label: text(&value.label),
        predicates: value
            .predicate
            .as_value()
            .map(|values| values.iter().map(predicate).collect())
            .unwrap_or_default(),
    }
}

fn predicate(value: &crate::CreaturePredicate) -> String {
    match value {
        crate::CreaturePredicate::Term(term) => term.as_str().to_string(),
        crate::CreaturePredicate::Not(term) => format!("not:{}", term.as_str()),
        crate::CreaturePredicate::Any(terms) => format!(
            "any:{}",
            terms
                .iter()
                .map(|term| term.as_str())
                .collect::<Vec<_>>()
                .join("|")
        ),
        crate::CreaturePredicate::AtLeast { term, minimum } => {
            format!("at_least:{minimum}:{}", term.as_str())
        }
        crate::CreaturePredicate::Unsupported(value) => value.value.clone(),
    }
}

fn movement(value: &crate::CreatureSpeed) -> CreatureMovementModeJson {
    CreatureMovementModeJson {
        id: value.id.as_str().to_string(),
        order: value.authored_order,
        mode: match &value.mode {
            CreatureMovementMode::Land => "land".to_string(),
            CreatureMovementMode::Burrow => "burrow".to_string(),
            CreatureMovementMode::Climb => "climb".to_string(),
            CreatureMovementMode::Fly => "fly".to_string(),
            CreatureMovementMode::Swim => "swim".to_string(),
            CreatureMovementMode::Unsupported(value) => value.value.clone(),
        },
        label: text(&value.label),
        value_feet: integer(&value.value),
        details: note(&value.details),
    }
}

fn resource(value: &crate::CreatureResource) -> CreatureResourceJson {
    CreatureResourceJson {
        id: value.id.as_str().to_string(),
        order: value.authored_order,
        kind: value.kind.as_str().to_string(),
        label: value.label.clone(),
        maximum: resource_amount(&value.maximum),
        serialized_value: resource_amount(&value.serialized_value),
        current_policy: match value.current_policy {
            ResourceCurrentPolicy::SerializedValueIsProvenanceOnly => {
                "serialized_value_is_provenance_only"
            }
        },
    }
}

fn resource_amount(value: &FactValue<CreatureResourceAmount>) -> Option<i64> {
    value.as_value().and_then(|value| match value {
        CreatureResourceAmount::Integer(value) => Some(*value),
        CreatureResourceAmount::Unsupported(_) => None,
    })
}

fn occurrence_label(occurrence: &CreatureEntityOccurrence, entities: &[CreatureEntity]) -> String {
    if let Some(label) = occurrence.context.contextual_label.as_value() {
        return label.clone();
    }
    if let CreatureEntityTarget::ActorOwned(id) = &occurrence.target
        && let Some(entity) = entities.iter().find(|entity| &entity.id == id)
    {
        return entity.label.clone();
    }
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(key) => key.to_string(),
        CreatureEntityTarget::ActorOwned(id) => id.as_str().to_string(),
    }
}

fn target_record_key(occurrence: &CreatureEntityOccurrence) -> Option<String> {
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(key) => Some(key.to_string()),
        CreatureEntityTarget::ActorOwned(_) => None,
    }
}

fn target_entity_id(occurrence: &CreatureEntityOccurrence) -> Option<String> {
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(_) => None,
        CreatureEntityTarget::ActorOwned(id) => Some(id.as_str().to_string()),
    }
}

fn occurrence_provenance(
    occurrence: &CreatureEntityOccurrence,
) -> CreatureOccurrenceProvenanceJson {
    CreatureOccurrenceProvenanceJson {
        identity_stability: match occurrence.identity_stability {
            crate::OccurrenceIdentityStability::StableNestedSourceId => "stable_nested_source_id",
            crate::OccurrenceIdentityStability::UnstableOwnerFamilyOrdinal => {
                "unstable_owner_family_ordinal"
            }
        },
        nested_source_id: occurrence
            .source_identity
            .nested_source_id
            .as_value()
            .map(|value| value.as_str().to_string()),
        stable_source_locator: occurrence
            .source_identity
            .stable_source_locator
            .as_value()
            .map(|value| value.as_str().to_string()),
        source_locators: occurrence
            .source_identity
            .source_locators
            .iter()
            .map(|value| value.locator.as_str().to_string())
            .collect(),
    }
}

fn occurrence_context(value: &CreatureOccurrenceContext) -> CreatureOccurrenceContextJson {
    CreatureOccurrenceContextJson {
        group: text(&value.group),
        rank: integer(&value.rank),
        location: text(&value.location),
        slot: text(&value.slot),
        uses: value.uses.as_value().map(use_limit),
        contextual_label: text(&value.contextual_label),
    }
}

fn use_limit(value: &CreatureUseLimit) -> CreatureUseLimitJson {
    CreatureUseLimitJson {
        maximum: integer(&value.maximum),
        serialized_value: integer(&value.serialized_value),
    }
}

fn action_cost(value: &CreatureActionCost) -> CreatureActionCostJson {
    let (kind, actions, time, unsupported) = match value {
        CreatureActionCost::Passive => ("passive", None, None, None),
        CreatureActionCost::Reaction => ("reaction", None, None, None),
        CreatureActionCost::FreeAction => ("free_action", None, None, None),
        CreatureActionCost::Actions(actions) => ("actions", Some(*actions), None, None),
        CreatureActionCost::Time(time) => ("time", None, Some(time.clone()), None),
        CreatureActionCost::Unsupported(value) => {
            ("unsupported", None, None, Some(value.value.clone()))
        }
    };
    CreatureActionCostJson {
        kind,
        actions,
        time,
        unsupported,
    }
}

fn frequency(value: &CreatureFrequency) -> CreatureFrequencyJson {
    let maximum = integer(&value.maximum);
    let period = text(&value.period);
    let display_period = period
        .as_deref()
        .and_then(crate::CreatureFrequencyPeriod::from_source_token);
    CreatureFrequencyJson {
        maximum,
        period,
        display: crate::format_creature_frequency(maximum, display_period),
        serialized_value: integer(&value.serialized_value),
    }
}

fn preparation(value: &CreatureSpellPreparation) -> String {
    match value {
        CreatureSpellPreparation::Prepared => "prepared".to_string(),
        CreatureSpellPreparation::Spontaneous => "spontaneous".to_string(),
        CreatureSpellPreparation::Focus => "focus".to_string(),
        CreatureSpellPreparation::Innate => "innate".to_string(),
        CreatureSpellPreparation::Ritual => "ritual".to_string(),
        CreatureSpellPreparation::Unsupported(value) => value.value.clone(),
    }
}

fn spell_slots(values: &FactValue<Vec<CreatureSpellSlot>>) -> Option<Vec<CreatureSpellSlotJson>> {
    values
        .as_value()
        .map(|values| values.iter().map(spell_slot).collect())
}

fn spell_slot(value: &CreatureSpellSlot) -> CreatureSpellSlotJson {
    CreatureSpellSlotJson {
        rank: value.rank,
        maximum: source_integer(&value.maximum),
        serialized_value: source_integer(&value.serialized_value),
        prepared: value.prepared.as_value().map(|values| {
            values
                .iter()
                .filter_map(|value| match value {
                    CreaturePreparedSpellSlot::Spell {
                        id,
                        name,
                        expended,
                        prepared,
                        authored_order,
                    } => Some(CreaturePreparedSpellJson {
                        order: *authored_order,
                        id: id.as_value().map(|value| value.as_str().to_string()),
                        name: text(name),
                        expended: boolean(expended),
                        prepared: boolean(prepared),
                    }),
                    CreaturePreparedSpellSlot::Unsupported(_) => None,
                })
                .collect()
        }),
    }
}

fn source_integer(value: &FactValue<CreatureSourceScalar<i64>>) -> Option<i64> {
    value.as_value().and_then(|value| match value {
        CreatureSourceScalar::Value(value) => Some(*value),
        CreatureSourceScalar::Unsupported(_) => None,
    })
}

fn rolls(values: &[CreatureRoll]) -> Vec<CreatureRollJson> {
    values
        .iter()
        .map(|value| CreatureRollJson {
            id: value.id.clone(),
            label: value.label.clone(),
            kind: match value.kind {
                CreatureRollKind::Attack => "attack",
                CreatureRollKind::DifficultyClass => "difficulty_class",
                CreatureRollKind::Check => "check",
            },
            value: integer(&value.value),
            ability: value.ability.as_value().copied().map(ability),
        })
        .collect()
}

fn damage(values: &FactValue<Vec<CreatureDamage>>) -> Option<Vec<CreatureDamageJson>> {
    values.as_value().map(|values| {
        values
            .iter()
            .map(|value| CreatureDamageJson {
                id: value.id.clone(),
                formula: text(&value.formula),
                damage_type: text(&value.damage_type),
                category: text(&value.category),
                kinds: value
                    .kinds
                    .as_value()
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|kind| match kind {
                                CreatureDamageKind::Damage => Some("damage"),
                                CreatureDamageKind::Healing => Some("healing"),
                                CreatureDamageKind::Unsupported(_) => None,
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                apply_modifier: value
                    .apply_modifier
                    .as_value()
                    .and_then(|value| match value {
                        CreatureSourceScalar::Value(value) => Some(*value),
                        CreatureSourceScalar::Unsupported(_) => None,
                    }),
            })
            .collect()
    })
}

fn integer(value: &FactValue<i64>) -> Option<i64> {
    value.as_value().copied()
}

fn boolean(value: &FactValue<bool>) -> Option<bool> {
    value.as_value().copied()
}

fn text(value: &FactValue<String>) -> Option<String> {
    value.as_value().cloned()
}

fn note(value: &FactValue<crate::CreatureNote>) -> Option<String> {
    value.as_value().map(|value| value.as_str().to_string())
}

fn strings(value: &FactValue<Vec<String>>) -> Vec<String> {
    value.as_value().cloned().unwrap_or_default()
}

fn ability(value: ActivityRollAbility) -> &'static str {
    match value {
        ActivityRollAbility::Strength => "strength",
        ActivityRollAbility::Dexterity => "dexterity",
        ActivityRollAbility::Constitution => "constitution",
        ActivityRollAbility::Intelligence => "intelligence",
        ActivityRollAbility::Wisdom => "wisdom",
        ActivityRollAbility::Charisma => "charisma",
    }
}
