use atlas_record::{CreatureFrequencyPeriod, CreatureSize};
use serde_json::Number;

use super::super::item_common::{CommonItemSource, ItemParentSource};
use super::super::{
    SerializedSourceObject, SerializedSourceValue, SourceIdentity, SourcePresence,
    SourceVersionMetadata,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedPhysicalItemSource {
    pub version: SourceVersionMetadata,
    pub identity: SourceIdentity,
    pub parent: Option<ItemParentSource>,
    pub source: PhysicalItemSource,
    pub(super) serialized: SerializedSourceObject,
}

impl VersionedPhysicalItemSource {
    /// Ordered source evidence, not proof of typed family completeness.
    pub fn source_json_for_audit(&self) -> String {
        self.serialized.compact_json()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalItemSource {
    /// Common fields use the same parser as other Items. Its pending system
    /// fields exclude the physical fields modeled below.
    pub common: CommonItemSource,
    pub physical: PhysicalSystemSource,
}

/// Shared PhysicalSystemSource. Presence is retained before defaults; concrete
/// family requirements, forbidden members and narrower types are still pending.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalSystemSource {
    pub quantity: SourcePresence<Number>,
    pub base_item: SourcePresence<String>,
    pub bulk: SourcePresence<PhysicalBulkSource>,
    pub hp: SourcePresence<PhysicalHitPointsSource>,
    pub hardness: SourcePresence<Number>,
    pub price: SourcePresence<ItemPriceSource>,
    pub equipped: SourcePresence<ItemEquippedSource>,
    pub identification: SourcePresence<ItemIdentificationSource>,
    pub container_id: SourcePresence<String>,
    pub material: SourcePresence<ItemMaterialSource>,
    /// The existing size vocabulary has the same meaning and six source tokens
    /// for creatures and equipment; no item-specific copy is needed.
    pub size: SourcePresence<CreatureSize>,
    pub usage: SourcePresence<PhysicalUsageSource>,
    pub activations: SourcePresence<Vec<(String, SourcePresence<ItemActivationSource>)>>,
    pub temporary: SourcePresence<bool>,
    /// Recursive shared physical fields; child family bodies remain pending.
    pub subitems: SourcePresence<Vec<PhysicalItemSource>>,
    pub apex: SourcePresence<SourceApex>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalBulkSource {
    pub value: SourcePresence<Number>,
    /// Container-specific bulk fields are modeled with that family's refinement.
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalHitPointsSource {
    pub value: SourcePresence<Number>,
    pub max: SourcePresence<Number>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPriceSource {
    pub value: SourcePresence<ItemCoinsSource>,
    pub per: SourcePresence<Number>,
    pub size_sensitive: SourcePresence<bool>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemCoinsSource {
    pub pp: SourcePresence<Number>,
    pub gp: SourcePresence<Number>,
    pub sp: SourcePresence<Number>,
    pub cp: SourcePresence<Number>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemEquippedSource {
    pub carry_type: SourcePresence<ItemCarryTypeSource>,
    pub in_slot: SourcePresence<bool>,
    pub hands_held: SourcePresence<u8>,
    pub invested: SourcePresence<bool>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCarryTypeSource {
    Attached,
    Dropped,
    Held,
    Stowed,
    Worn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemIdentificationSource {
    pub status: SourcePresence<ItemIdentificationStatusSource>,
    pub unidentified: SourcePresence<ItemMystifiedSource>,
    /// Upstream declares `object`, not a second MystifiedData structure. Arrays
    /// are also TypeScript objects. This declared payload intentionally stays open.
    pub misidentified: SourcePresence<SerializedSourceValue>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemIdentificationStatusSource {
    Identified,
    Unidentified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemMystifiedSource {
    pub name: SourcePresence<String>,
    pub image: SourcePresence<String>,
    pub data: SourcePresence<ItemMystifiedDataSource>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemMystifiedDataSource {
    pub description: SourcePresence<ItemTextValueSource>,
    pub additional_fields: SerializedSourceObject,
}

/// Exact `{ value: string }` structure used by activation and mystified descriptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTextValueSource {
    pub value: SourcePresence<String>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemMaterialSource {
    pub grade: SourcePresence<ItemMaterialGradeSource>,
    pub material_type: SourcePresence<ItemMaterialTypeSource>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemMaterialGradeSource {
    Low,
    Standard,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemMaterialTypeSource {
    Abysium,
    Adamantine,
    ColdIron,
    Duskwood,
    Djezet,
    Dragonhide,
    Dreamweb,
    GrisantianPelt,
    Inubrix,
    KeepStone,
    Dawnsilver,
    Noqual,
    Orichalcum,
    Peachwood,
    Siccatite,
    Silver,
    Sisterstone,
    SisterstoneDusk,
    SisterstoneScarlet,
    Sloughstone,
    SovereignSteel,
    Warpglass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalUsageSource {
    pub value: SourcePresence<String>,
    /// Includes weapon `canBeAmmo`, whose family refinement remains pending.
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceApex {
    pub attribute: SourcePresence<ApexAttributeSource>,
    pub selected: SourcePresence<bool>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApexAttributeSource {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemActivationSource {
    pub id: SourcePresence<String>,
    pub description: SourcePresence<ItemTextValueSource>,
    pub action_cost: SourcePresence<SourceActionCost>,
    pub components: SourcePresence<ItemActivationComponentsSource>,
    pub frequency: SourcePresence<ItemFrequencySource>,
    pub traits: SourcePresence<ItemActivationTraitsSource>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceActionCost {
    pub action_type: SourcePresence<SourceActionType>,
    pub value: SourcePresence<u8>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceActionType {
    Action,
    Reaction,
    Free,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemActivationComponentsSource {
    pub command: SourcePresence<bool>,
    pub envision: SourcePresence<bool>,
    pub interact: SourcePresence<bool>,
    pub cast: SourcePresence<bool>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFrequencySource {
    pub value: SourcePresence<Number>,
    pub max: SourcePresence<Number>,
    pub per: SourcePresence<CreatureFrequencyPeriod>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemActivationTraitsSource {
    pub value: SourcePresence<Vec<String>>,
    pub additional_fields: SerializedSourceObject,
}
