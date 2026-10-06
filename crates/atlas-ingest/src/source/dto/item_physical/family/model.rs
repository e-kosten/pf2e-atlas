use serde_json::Number;

use super::super::super::item_common::ItemNumberValueSource;
use super::super::super::{SerializedSourceObject, SourcePresence};
use super::super::ItemTextValueSource;

/// Refinements compose the common/physical fields in PhysicalItemSource.
/// Pending is modeling status for armor, consumable, shield and weapon; their
/// exact discriminator and system payload stay in the common source owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalFamilySource {
    Equipment(Box<EquipmentFamilySource>),
    Backpack(BackpackFamilySource),
    Book(BookFamilySource),
    Treasure(TreasureFamilySource),
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentFamilySource {
    /// Observed serialized fields outside the current EquipmentSystemSource
    /// declaration. They are retained as source facts without weapon semantics.
    pub legacy: EquipmentLegacySource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentLegacySource {
    pub stowing: SourcePresence<bool>,
    pub ability: SourcePresence<ItemTextValueSource>,
    pub map: SourcePresence<ItemTextValueSource>,
    pub bonus: SourcePresence<ItemNumberValueSource>,
    pub bonus_damage: SourcePresence<ItemNumberValueSource>,
    pub splash_damage: SourcePresence<ItemNumberValueSource>,
    pub damage: SourcePresence<EquipmentLegacyDamageSource>,
    pub range: SourcePresence<ItemTextValueSource>,
    pub reload: SourcePresence<ItemTextValueSource>,
    pub weapon_type: SourcePresence<ItemTextValueSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentLegacyDamageSource {
    pub damage_type: SourcePresence<String>,
    pub dice: SourcePresence<Number>,
    pub die: SourcePresence<String>,
    pub value: SourcePresence<String>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackpackFamilySource {
    pub stowing: SourcePresence<bool>,
    pub collapsed: SourcePresence<bool>,
    pub bulk: SourcePresence<BackpackBulkSource>,
}

/// Extra container bulk fields; shared bulk.value stays in the physical owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackpackBulkSource {
    pub held_or_stowed: SourcePresence<Number>,
    pub capacity: SourcePresence<Number>,
    pub ignored: SourcePresence<Number>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookFamilySource {
    pub category: SourcePresence<BookCategorySource>,
    pub capacity: SourcePresence<Number>,
    /// Authored ItemUUID strings in order, without resolution or deduplication.
    pub contents: SourcePresence<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookCategorySource {
    Formula,
    Spell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreasureFamilySource {
    pub stack_group: SourcePresence<TreasureStackGroupSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreasureStackGroupSource {
    Coins,
    Gems,
}
