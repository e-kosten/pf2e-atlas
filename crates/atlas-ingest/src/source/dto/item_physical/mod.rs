//! Shared persisted physical Item fields, before defaults or product conversion.
//!
//! Concrete family refinements, rules and effects remain pending. This parser
//! recognizes the physical union; it does not certify full family admission.

mod model;
mod parse;

pub use model::{
    ApexAttributeSource, ItemActivationComponentsSource, ItemActivationSource,
    ItemActivationTraitsSource, ItemCarryTypeSource, ItemCoinsSource, ItemEquippedSource,
    ItemFrequencySource, ItemIdentificationSource, ItemIdentificationStatusSource,
    ItemMaterialGradeSource, ItemMaterialSource, ItemMaterialTypeSource, ItemMystifiedDataSource,
    ItemMystifiedSource, ItemPriceSource, ItemTextValueSource, PhysicalBulkSource,
    PhysicalHitPointsSource, PhysicalItemSource, PhysicalSystemSource, PhysicalUsageSource,
    SourceActionCost, SourceActionType, SourceApex, VersionedPhysicalItemSource,
};
pub use parse::parse_physical_item_source;
