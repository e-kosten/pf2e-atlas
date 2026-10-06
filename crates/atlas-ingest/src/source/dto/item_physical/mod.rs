//! Shared persisted physical Item fields, before defaults or product conversion.
//!
//! Equipment, backpack, book and treasure have typed family refinements. The
//! other four family bodies, rules and effects remain pending. This parser
//! retains pre-default source presence rather than full Foundry admission checks.

mod family;
mod model;
mod parse;

pub use model::{
    ApexAttributeSource, ItemActivationComponentsSource, ItemActivationSource,
    ItemActivationTraitsSource, ItemCarryTypeSource, ItemCoinsSource, ItemEquippedSource,
    ItemFrequencySource, ItemIdentificationSource, ItemIdentificationStatusSource,
    ItemMaterialGradeSource, ItemMaterialSource, ItemMaterialTypeSource, ItemMystifiedDataSource,
    ItemMystifiedSource, ItemPriceSource, ItemTextValueSource, PhysicalHitPointsSource,
    PhysicalItemSource, PhysicalSystemSource, PhysicalUsageSource, SourceActionCost,
    SourceActionType, SourceApex, VersionedPhysicalItemSource,
};
pub use parse::parse_physical_item_source;

pub use family::{
    BackpackBulkSource, BackpackFamilySource, BookCategorySource, BookFamilySource,
    EquipmentFamilySource, EquipmentLegacyDamageSource, EquipmentLegacySource,
    PhysicalFamilySource, TreasureFamilySource, TreasureStackGroupSource,
};

use parse::text_value;
