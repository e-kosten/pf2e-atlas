//! Versioned DTO boundary for persisted PF2e/Foundry source documents.
//!
//! These types describe serialized `Source`, not Foundry's prepared `Data`.
//! Canonical record/entity conversion is intentionally owned by later ingest
//! phases. The original JSON is retained only behind an explicitly named audit
//! accessor.

mod diagnostic;
mod item;
mod npc;
mod presence;
mod value;
mod version;

pub use diagnostic::{SourceDiagnostic, SourceDiagnosticKind, SourceIdentity};
pub use item::{
    FullItemSource, ItemSource, ItemType, SourceParentContext, VersionedItemSource,
    parse_item_source,
};
pub use npc::{ActorType, NpcSource, VersionedNpcSource, parse_npc_source};
pub use presence::SourcePresence;
pub use value::SerializedSourceObject;
pub use version::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SYSTEM_ID,
    PF2E_SOURCE_PINNED_SYSTEM_VERSION, SourceVersionMetadata, pinned_source_version_metadata,
    validate_pinned_source_version,
};

pub(crate) use value::{
    RawSourceJson, actual_shape, optional_array_of_objects, optional_integer, optional_object,
    optional_string, required_object, required_string,
};

#[cfg(test)]
mod tests;
