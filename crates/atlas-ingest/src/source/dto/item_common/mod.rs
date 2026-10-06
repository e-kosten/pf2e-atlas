//! Common persisted Item fields, independently callable before product conversion.
//!
//! Family-specific systems, rule elements and embedded effects are retained for
//! inspection but are not yet fully typed by this parser.

mod model;
mod parse;

pub use super::value::{SerializedSourceObject, SerializedSourceValue};
pub use model::{
    CommonItemSource, ItemDescriptionSource, ItemFlagsSource, ItemGrantDeleteAction,
    ItemGrantSource, ItemGranterSource, ItemLevelSource, ItemLicenseSource,
    ItemMigrationPreviousSource, ItemMigrationSource, ItemOwnershipLevel, ItemParentSource,
    ItemPublicationSource, ItemRuleSelectionSource, ItemSourceEnvelope, ItemStatsSource,
    ItemSystemSource, ItemTraitsShape, ItemTraitsSource, Pf2eItemFlagsSource,
    VersionedCommonItemSource,
};
pub use parse::parse_common_item_source;
