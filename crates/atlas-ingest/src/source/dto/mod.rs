//! Versioned DTO boundary for persisted PF2e/Foundry source documents.
//!
//! These types describe serialized `Source`, not Foundry's prepared `Data`.
//! Canonical record/entity conversion is intentionally owned by later ingest
//! phases. The original JSON is retained only behind an explicitly named audit
//! accessor.

mod creature_core;
mod diagnostic;
mod embedded;
mod hazard;
mod item;
mod npc;
mod presence;
mod spell;
mod value;
mod version;

pub use diagnostic::{SourceDiagnostic, SourceDiagnosticKind, SourceIdentity};
pub use item::{
    FullItemSource, ItemSource, ItemType, SourceParentContext, VersionedItemSource,
    parse_item_source,
};
pub use npc::{ActorType, NpcSource, VersionedNpcSource, parse_npc_source};
pub use presence::SourcePresence;
pub use version::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SIGNATURE,
    PF2E_SOURCE_PINNED_SYSTEM_ID, PF2E_SOURCE_PINNED_SYSTEM_VERSION, SourceVersionMetadata,
    pinned_source_version_metadata, validate_pinned_source_version,
};

pub(crate) use embedded::{
    ActionSource, ActorSpellcastingSource, DamageSource, EmbeddedCommonSource,
    EmbeddedSourceScalar, EquipmentSource, NpcEmbeddedItemSource, PreparedSlotSource,
    SourceTypeDrift, SpellDefenseSource, SpellSource, SpellcastingEntrySource, StrikeSource,
    UseLimitSource, ValueSummary,
};
pub(crate) use hazard::{
    HazardDamageSource, HazardDefensesSource, HazardDetectionSource, HazardEmitsSoundSource,
    HazardFrequencySource, HazardHitPointsSource, HazardItemSource, HazardIwrSource,
    HazardLifecycleSource, HazardPublicationSource, HazardSaveSource, HazardSavesSource,
    HazardSelfEffectSource, HazardSource, HazardSourceField, HazardSourceValue,
    VersionedHazardSource, parse_hazard_source,
};
pub(crate) use item::{
    ConsumableDamageSource, ConsumableEquippedSource, ConsumableItemSource,
    ConsumableMaterialSource, ConsumablePriceSource, ConsumablePublicationSource,
    ConsumableSourceFact, ConsumableUnsupportedSource, EmbeddedRelationshipKindSource,
    EmbeddedRelationshipSource, EmbeddedStableLocatorSource, parse_item_source_from_serialized,
};
pub(crate) use npc::parse_npc_source_from_serialized;
pub(crate) use spell::{
    ConsumableSpellChildSource, SpellDocumentSource, SpellItemSource, parse_spell_document_source,
};
pub(crate) use value::{
    LegacyDuplicateDisposition, RawSourceJson, SerializedSourceMember, SerializedSourceObject,
    SerializedSourceValue, actual_shape, optional_array_of_objects, optional_integer,
    optional_object, optional_string, parse_serialized_source_object, required_object,
    required_string,
};

#[cfg(test)]
mod tests;
pub use creature_core::{
    NpcCoreSource, NpcIwrSource, NpcLegacyAbilitySource, NpcPredicateSource,
    NpcResourceAmountSource, NpcResourceSource, NpcSaveSource, NpcSavesSource, NpcSkillSource,
    NpcSkillVariantSource, SourceInteger,
};
