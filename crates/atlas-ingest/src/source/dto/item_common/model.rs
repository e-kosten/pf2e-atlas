use atlas_domain::Rarity;
use serde_json::Number;

use super::super::{ActorType, ItemType, SourceIdentity, SourcePresence, SourceVersionMetadata};
use super::{SerializedSourceObject, SerializedSourceValue};

/// Context of an item in an Actor's authored `items` array, without game rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemParentSource {
    pub actor_type: ActorType,
    pub item_ordinal: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedCommonItemSource {
    pub version: SourceVersionMetadata,
    pub identity: SourceIdentity,
    pub parent: Option<ItemParentSource>,
    pub source: CommonItemSource,
    pub(super) serialized: SerializedSourceObject,
}

impl VersionedCommonItemSource {
    /// Ordered source JSON for explicit audits, including repeated members.
    /// Typed-model tests must inspect `source`, not use this as a round-trip proof.
    pub fn source_json_for_audit(&self) -> String {
        self.serialized.compact_json()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonItemSource {
    pub envelope: ItemSourceEnvelope,
    pub system: ItemSystemSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSourceEnvelope {
    /// Foundry DocumentIdField permits null before identity is assigned.
    pub id: SourcePresence<String>,
    pub name: String,
    pub item_type: ItemType,
    pub image: SourcePresence<String>,
    pub folder: SourcePresence<String>,
    pub sort: SourcePresence<i64>,
    pub ownership: SourcePresence<Vec<(String, SourcePresence<ItemOwnershipLevel>)>>,
    pub flags: SourcePresence<ItemFlagsSource>,
    pub stats: SourcePresence<ItemStatsSource>,
    /// Embedded ActiveEffect modeling is pending; order and complete values survive.
    pub effects: SourcePresence<Vec<SerializedSourceObject>>,
    pub additional_fields: SerializedSourceObject,
}

/// Source presence is retained before Foundry supplies defaults. Missing and
/// explicit null are evidence states, not defaults or prepared product values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSystemSource {
    pub level: SourcePresence<ItemLevelSource>,
    pub description: SourcePresence<ItemDescriptionSource>,
    pub traits: SourcePresence<ItemTraitsSource>,
    /// Rule-specific typed models are pending; this is not rule execution.
    pub rules: SourcePresence<Vec<SerializedSourceObject>>,
    pub slug: SourcePresence<String>,
    pub publication: SourcePresence<ItemPublicationSource>,
    pub migration: SourcePresence<ItemMigrationSource>,
    pub legacy_schema: SourcePresence<SerializedSourceValue>,
    /// Includes family-specific and previously unknown system fields.
    pub pending_family_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemLevelSource {
    pub value: SourcePresence<Number>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDescriptionSource {
    pub gm: SourcePresence<String>,
    pub value: SourcePresence<String>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemTraitsShape {
    ValuesAndRarity,
    ValuesOnly,
    RarityOnly,
    OtherTagsOnly,
}

/// The four upstream trait shapes share a payload. Family refinements can later
/// forbid value/rarity; this common parser preserves pre-default presence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTraitsSource {
    pub value: SourcePresence<Vec<String>>,
    pub rarity: SourcePresence<Rarity>,
    pub other_tags: SourcePresence<Vec<String>>,
    pub additional_fields: SerializedSourceObject,
}

impl ItemTraitsSource {
    /// Observed layout based on member presence, including explicit null.
    /// This does not certify a family's narrower trait constraints.
    pub fn shape(&self) -> ItemTraitsShape {
        match (!self.value.is_missing(), !self.rarity.is_missing()) {
            (true, true) => ItemTraitsShape::ValuesAndRarity,
            (true, false) => ItemTraitsShape::ValuesOnly,
            (false, true) => ItemTraitsShape::RarityOnly,
            (false, false) => ItemTraitsShape::OtherTagsOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemLicenseSource {
    Orc,
    Ogl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPublicationSource {
    pub title: SourcePresence<String>,
    pub authors: SourcePresence<String>,
    pub license: SourcePresence<ItemLicenseSource>,
    pub remaster: SourcePresence<bool>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemMigrationSource {
    pub version: SourcePresence<Number>,
    pub previous: SourcePresence<ItemMigrationPreviousSource>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemMigrationPreviousSource {
    pub foundry: SourcePresence<String>,
    pub system: SourcePresence<String>,
    pub schema: SourcePresence<Number>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemOwnershipLevel {
    Inherit,
    None,
    Limited,
    Observer,
    Owner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStatsSource {
    pub system_id: SourcePresence<String>,
    pub system_version: SourcePresence<String>,
    pub core_version: SourcePresence<String>,
    pub created_time: SourcePresence<Number>,
    pub modified_time: SourcePresence<Number>,
    pub last_modified_by: SourcePresence<String>,
    pub compendium_source: SourcePresence<String>,
    pub duplicate_source: SourcePresence<String>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFlagsSource {
    pub pf2e: SourcePresence<Pf2eItemFlagsSource>,
    /// Other Foundry/module namespaces are open source values.
    pub extensions: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pf2eItemFlagsSource {
    pub rules_selections: SourcePresence<Vec<(String, SourcePresence<ItemRuleSelectionSource>)>>,
    pub item_grants: SourcePresence<Vec<(String, SourcePresence<ItemGranterSource>)>>,
    pub granted_by: SourcePresence<ItemGrantSource>,
    pub extensions: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemRuleSelectionSource {
    String(String),
    Number(Number),
    Object(SerializedSourceObject),
    /// TypeScript's `object` source alternative includes JSON arrays.
    Array(Vec<SerializedSourceValue>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemGrantDeleteAction {
    Cascade,
    Detach,
    Restrict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemGrantSource {
    pub id: String,
    pub on_delete: SourcePresence<ItemGrantDeleteAction>,
    pub additional_fields: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemGranterSource {
    pub grant: ItemGrantSource,
    pub nested: SourcePresence<bool>,
}
