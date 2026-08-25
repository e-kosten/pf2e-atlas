use std::fmt;

use serde_json::Value;

use super::creature_core::{LoreSource, parse_lore_source};
use super::embedded::{NpcEmbeddedItemSource, parse_npc_embedded_item};
use super::value::serialized_object;
use super::{
    RawSourceJson, SerializedSourceObject, SourceDiagnostic, SourceDiagnosticKind, SourceIdentity,
    SourcePresence, SourceVersionMetadata, actual_shape, optional_array_of_objects,
    optional_integer, optional_object, optional_string, required_object, required_string,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemType {
    Action,
    Affliction,
    Ancestry,
    Armor,
    Background,
    Backpack,
    Book,
    CampaignFeature,
    Class,
    Condition,
    Consumable,
    Deity,
    Effect,
    Equipment,
    Feat,
    Heritage,
    Kit,
    Lore,
    Melee,
    Shield,
    Spell,
    SpellcastingEntry,
    Treasure,
    Weapon,
}

impl ItemType {
    pub const ALL: [Self; 24] = [
        Self::Action,
        Self::Affliction,
        Self::Ancestry,
        Self::Armor,
        Self::Background,
        Self::Backpack,
        Self::Book,
        Self::CampaignFeature,
        Self::Class,
        Self::Condition,
        Self::Consumable,
        Self::Deity,
        Self::Effect,
        Self::Equipment,
        Self::Feat,
        Self::Heritage,
        Self::Kit,
        Self::Lore,
        Self::Melee,
        Self::Shield,
        Self::Spell,
        Self::SpellcastingEntry,
        Self::Treasure,
        Self::Weapon,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Action => "action",
            Self::Affliction => "affliction",
            Self::Ancestry => "ancestry",
            Self::Armor => "armor",
            Self::Background => "background",
            Self::Backpack => "backpack",
            Self::Book => "book",
            Self::CampaignFeature => "campaignFeature",
            Self::Class => "class",
            Self::Condition => "condition",
            Self::Consumable => "consumable",
            Self::Deity => "deity",
            Self::Effect => "effect",
            Self::Equipment => "equipment",
            Self::Feat => "feat",
            Self::Heritage => "heritage",
            Self::Kit => "kit",
            Self::Lore => "lore",
            Self::Melee => "melee",
            Self::Shield => "shield",
            Self::Spell => "spell",
            Self::SpellcastingEntry => "spellcastingEntry",
            Self::Treasure => "treasure",
            Self::Weapon => "weapon",
        }
    }

    fn parse(
        value: &str,
        identity: &SourceIdentity,
        json_path: &str,
    ) -> Result<Self, SourceDiagnostic> {
        Self::ALL
            .into_iter()
            .find(|item_type| item_type.as_str() == value)
            .ok_or_else(|| {
                SourceDiagnostic::new(
                    SourceDiagnosticKind::UnknownDiscriminator,
                    identity,
                    json_path,
                    format!(
                        "one of [{}]",
                        Self::ALL
                            .iter()
                            .map(|item_type| item_type.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    format!("unknown Item discriminator {value:?}"),
                )
            })
    }

    fn is_valid_for_parent(self, parent: &SourceParentContext) -> bool {
        match parent {
            SourceParentContext::NpcItems => matches!(
                self,
                Self::Action
                    | Self::Affliction
                    | Self::Armor
                    | Self::Backpack
                    | Self::Book
                    | Self::Condition
                    | Self::Consumable
                    | Self::Effect
                    | Self::Equipment
                    | Self::Lore
                    | Self::Melee
                    | Self::Shield
                    | Self::Spell
                    | Self::SpellcastingEntry
                    | Self::Treasure
                    | Self::Weapon
            ),
        }
    }
}

impl fmt::Display for ItemType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceParentContext {
    NpcItems,
}

impl SourceParentContext {
    pub const fn npc_items() -> Self {
        Self::NpcItems
    }

    pub const fn relationship_path(&self) -> &'static str {
        match self {
            Self::NpcItems => "items",
        }
    }

    fn display(&self) -> String {
        match self {
            Self::NpcItems => format!("Actor[npc].{}", self.relationship_path()),
        }
    }
}

/// Full serialized Foundry Item source envelope.
///
/// `serialized` and `system` retain the complete source tree. Product-backed
/// fields are promoted into explicit type-specific DTOs in B3/B4; downstream
/// code must not query these trees as a fallback semantic API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullItemSource {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub image: SourcePresence<String>,
    pub folder: SourcePresence<String>,
    pub sort: SourcePresence<i64>,
    pub effects: SourcePresence<Vec<SerializedSourceObject>>,
    pub flags: SourcePresence<SerializedSourceObject>,
    pub parent_context: Option<SourceParentContext>,
    pub lore: Option<LoreSource>,
    pub(crate) compendium_source: SourcePresence<String>,
    pub(crate) stable_source_locators: Vec<EmbeddedStableLocatorSource>,
    pub(crate) embedded_relationships: Vec<EmbeddedRelationshipSource>,
    pub(crate) relationship_unsupported: Vec<super::embedded::ValueSummary>,
    pub(crate) embedded: Option<NpcEmbeddedItemSource>,
    serialized: SerializedSourceObject,
    system: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmbeddedStableLocatorSource {
    pub(crate) source_path: &'static str,
    pub(crate) value: String,
    pub(crate) precedence: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmbeddedRelationshipKindSource {
    GrantedBy,
    ItemGrant,
    LinkedWeapon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmbeddedRelationshipSource {
    pub(crate) kind: EmbeddedRelationshipKindSource,
    pub(crate) target_id: String,
    pub(crate) source_path: String,
    pub(crate) contextual_label: SourcePresence<String>,
    pub(crate) lifecycle: SourcePresence<String>,
}

impl FullItemSource {
    #[cfg(test)]
    pub(super) fn serialized(&self) -> &SerializedSourceObject {
        &self.serialized
    }

    #[cfg(test)]
    pub(super) fn system(&self) -> &SerializedSourceObject {
        &self.system
    }
}

macro_rules! item_source_variants {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum ItemSource {
            $($variant(FullItemSource)),+
        }

        impl ItemSource {
            pub fn source(&self) -> &FullItemSource {
                match self {
                    $(Self::$variant(source) => source),+
                }
            }

            pub fn item_type(&self) -> ItemType {
                self.source().item_type
            }
        }
    };
}

item_source_variants!(
    Action,
    Affliction,
    Ancestry,
    Armor,
    Background,
    Backpack,
    Book,
    CampaignFeature,
    Class,
    Condition,
    Consumable,
    Deity,
    Effect,
    Equipment,
    Feat,
    Heritage,
    Kit,
    Lore,
    Melee,
    Shield,
    Spell,
    SpellcastingEntry,
    Treasure,
    Weapon,
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedItemSource {
    pub version: SourceVersionMetadata,
    pub source: ItemSource,
    provenance: RawSourceJson,
}

impl VersionedItemSource {
    /// The immutable original JSON for explicit provenance/audit workflows.
    pub fn raw_json_for_audit(&self) -> &Value {
        self.provenance.for_audit()
    }
}

pub fn parse_item_source(
    version: SourceVersionMetadata,
    identity: SourceIdentity,
    parent_context: Option<SourceParentContext>,
    raw: Value,
) -> Result<VersionedItemSource, SourceDiagnostic> {
    let source = parse_item(&identity, parent_context, &raw, "$")?;
    Ok(VersionedItemSource {
        version,
        source,
        provenance: RawSourceJson::new(raw),
    })
}

pub(crate) fn parse_item(
    identity: &SourceIdentity,
    parent_context: Option<SourceParentContext>,
    raw: &Value,
    json_path: &str,
) -> Result<ItemSource, SourceDiagnostic> {
    let map = raw.as_object().ok_or_else(|| {
        SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            identity,
            json_path,
            "Item source object",
            actual_shape(raw),
        )
    })?;
    let id = required_string(map, "_id", identity, &format!("{json_path}._id"))?;
    let name = required_string(map, "name", identity, &format!("{json_path}.name"))?;
    let item_type_path = format!("{json_path}.type");
    let item_type_value = required_string(map, "type", identity, &item_type_path)?;
    let item_type = ItemType::parse(&item_type_value, identity, &item_type_path)?;
    if let Some(parent) = &parent_context
        && !item_type.is_valid_for_parent(parent)
    {
        return Err(SourceDiagnostic::new(
            SourceDiagnosticKind::InvalidParentContext,
            identity,
            item_type_path,
            format!("Item discriminator valid under {}", parent.display()),
            format!("Item discriminator {item_type:?}"),
        ));
    }
    let system = required_object(map, "system", identity, &format!("{json_path}.system"))?;
    let lore = (item_type == ItemType::Lore)
        .then(|| parse_lore_source(system, identity, &format!("{json_path}.system")))
        .transpose()?;
    let compendium_source = match map.get("_stats") {
        None => SourcePresence::Missing,
        Some(Value::Null) => SourcePresence::Null,
        Some(Value::Object(stats)) => optional_string(
            stats,
            "compendiumSource",
            identity,
            &format!("{json_path}._stats.compendiumSource"),
        )?,
        Some(other) => {
            return Err(SourceDiagnostic::new(
                SourceDiagnosticKind::MalformedShape,
                identity,
                format!("{json_path}._stats"),
                "object",
                actual_shape(other),
            ));
        }
    };
    let (stable_source_locators, embedded_relationships, relationship_unsupported) =
        embedded_identity_fields(map, system);
    let embedded = parent_context
        .as_ref()
        .is_some_and(|parent| *parent == SourceParentContext::NpcItems)
        .then(|| {
            parse_npc_embedded_item(item_type, system, identity, &format!("{json_path}.system"))
        })
        .transpose()?;
    let source = FullItemSource {
        id,
        name,
        item_type,
        image: optional_string(map, "img", identity, &format!("{json_path}.img"))?,
        folder: optional_string(map, "folder", identity, &format!("{json_path}.folder"))?,
        sort: optional_integer(map, "sort", identity, &format!("{json_path}.sort"))?,
        effects: optional_array_of_objects(
            map,
            "effects",
            identity,
            &format!("{json_path}.effects"),
        )?,
        flags: optional_object(map, "flags", identity, &format!("{json_path}.flags"))?,
        parent_context,
        lore,
        compendium_source,
        stable_source_locators,
        embedded_relationships,
        relationship_unsupported,
        embedded,
        serialized: serialized_object(map),
        system: serialized_object(system),
    };

    Ok(match item_type {
        ItemType::Action => ItemSource::Action(source),
        ItemType::Affliction => ItemSource::Affliction(source),
        ItemType::Ancestry => ItemSource::Ancestry(source),
        ItemType::Armor => ItemSource::Armor(source),
        ItemType::Background => ItemSource::Background(source),
        ItemType::Backpack => ItemSource::Backpack(source),
        ItemType::Book => ItemSource::Book(source),
        ItemType::CampaignFeature => ItemSource::CampaignFeature(source),
        ItemType::Class => ItemSource::Class(source),
        ItemType::Condition => ItemSource::Condition(source),
        ItemType::Consumable => ItemSource::Consumable(source),
        ItemType::Deity => ItemSource::Deity(source),
        ItemType::Effect => ItemSource::Effect(source),
        ItemType::Equipment => ItemSource::Equipment(source),
        ItemType::Feat => ItemSource::Feat(source),
        ItemType::Heritage => ItemSource::Heritage(source),
        ItemType::Kit => ItemSource::Kit(source),
        ItemType::Lore => ItemSource::Lore(source),
        ItemType::Melee => ItemSource::Melee(source),
        ItemType::Shield => ItemSource::Shield(source),
        ItemType::Spell => ItemSource::Spell(source),
        ItemType::SpellcastingEntry => ItemSource::SpellcastingEntry(source),
        ItemType::Treasure => ItemSource::Treasure(source),
        ItemType::Weapon => ItemSource::Weapon(source),
    })
}

fn embedded_identity_fields(
    item: &serde_json::Map<String, Value>,
    system: &serde_json::Map<String, Value>,
) -> (
    Vec<EmbeddedStableLocatorSource>,
    Vec<EmbeddedRelationshipSource>,
    Vec<super::embedded::ValueSummary>,
) {
    let mut locators = Vec::new();
    let mut relationships = Vec::new();
    let mut unsupported = Vec::new();

    capture_locator(
        nested(item, &["flags", "core", "sourceId"]),
        "$.flags.core.sourceId",
        1,
        &mut locators,
        &mut unsupported,
    );
    capture_locator(
        nested(system, &["spell", "flags", "core", "sourceId"]),
        "$.system.spell.flags.core.sourceId",
        3,
        &mut locators,
        &mut unsupported,
    );
    capture_locator(
        nested(system, &["spell", "_stats", "compendiumSource"]),
        "$.system.spell._stats.compendiumSource",
        2,
        &mut locators,
        &mut unsupported,
    );

    if let Some(granted_by) = nested(item, &["flags", "pf2e", "grantedBy"]) {
        capture_relationship_object(
            granted_by,
            EmbeddedRelationshipKindSource::GrantedBy,
            "$.flags.pf2e.grantedBy",
            SourcePresence::Missing,
            &mut relationships,
            &mut unsupported,
        );
    }
    if let Some(item_grants) = nested(item, &["flags", "pf2e", "itemGrants"]) {
        match item_grants {
            Value::Object(grants) => {
                for (label, grant) in grants {
                    capture_relationship_object(
                        grant,
                        EmbeddedRelationshipKindSource::ItemGrant,
                        &format!("$.flags.pf2e.itemGrants.{label}"),
                        SourcePresence::Value(label.clone()),
                        &mut relationships,
                        &mut unsupported,
                    );
                }
            }
            value => unsupported.push(summary("$.flags.pf2e.itemGrants", value)),
        }
    }
    if let Some(value) = nested(item, &["flags", "pf2e", "linkedWeapon"]) {
        match value.as_str() {
            Some(id) => relationships.push(EmbeddedRelationshipSource {
                kind: EmbeddedRelationshipKindSource::LinkedWeapon,
                target_id: id.to_string(),
                source_path: "$.flags.pf2e.linkedWeapon".to_string(),
                contextual_label: SourcePresence::Missing,
                lifecycle: SourcePresence::Missing,
            }),
            None => unsupported.push(summary("$.flags.pf2e.linkedWeapon", value)),
        }
    }
    (locators, relationships, unsupported)
}

fn nested<'a>(map: &'a serde_json::Map<String, Value>, segments: &[&str]) -> Option<&'a Value> {
    let mut value = map.get(*segments.first()?)?;
    for segment in &segments[1..] {
        value = value.as_object()?.get(*segment)?;
    }
    Some(value)
}

fn capture_locator(
    value: Option<&Value>,
    source_path: &'static str,
    precedence: u8,
    locators: &mut Vec<EmbeddedStableLocatorSource>,
    unsupported: &mut Vec<super::embedded::ValueSummary>,
) {
    let Some(value) = value else { return };
    match value.as_str() {
        Some(value) => locators.push(EmbeddedStableLocatorSource {
            source_path,
            value: value.to_string(),
            precedence,
        }),
        None => unsupported.push(summary(source_path, value)),
    }
}

fn capture_relationship_object(
    value: &Value,
    kind: EmbeddedRelationshipKindSource,
    source_path: &str,
    contextual_label: SourcePresence<String>,
    relationships: &mut Vec<EmbeddedRelationshipSource>,
    unsupported: &mut Vec<super::embedded::ValueSummary>,
) {
    let Some(map) = value.as_object() else {
        unsupported.push(summary(source_path, value));
        return;
    };
    let Some(id) = map.get("id").and_then(Value::as_str) else {
        if let Some(value) = map.get("id") {
            unsupported.push(summary(&format!("{source_path}.id"), value));
        }
        return;
    };
    let lifecycle = match map.get("onDelete") {
        None => SourcePresence::Missing,
        Some(Value::Null) => SourcePresence::Null,
        Some(Value::String(value)) => SourcePresence::Value(value.clone()),
        Some(value) => {
            unsupported.push(summary(&format!("{source_path}.onDelete"), value));
            SourcePresence::Missing
        }
    };
    relationships.push(EmbeddedRelationshipSource {
        kind,
        target_id: id.to_string(),
        source_path: format!("{source_path}.id"),
        contextual_label,
        lifecycle,
    });
}

fn summary(source_path: &str, value: &Value) -> super::embedded::ValueSummary {
    super::embedded::ValueSummary {
        source_path: source_path.to_string(),
        shape: actual_shape(value).to_string(),
        value: value.to_string(),
    }
}
