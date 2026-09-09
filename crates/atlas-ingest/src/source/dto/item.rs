use std::fmt;

use atlas_record::UnsupportedSourceShape;
use serde_json::Value;

use super::creature_core::{LoreSource, parse_lore_source};
use super::embedded::{NpcEmbeddedItemSource, parse_npc_embedded_item};
use super::{
    RawSourceJson, SerializedSourceMember, SerializedSourceObject, SerializedSourceValue,
    SourceDiagnostic, SourceDiagnosticKind, SourceIdentity, SourcePresence, SourceVersionMetadata,
    actual_shape, optional_array_of_objects, optional_integer, optional_object, optional_string,
    required_object, required_string,
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
            SourceParentContext::CharacterItems | SourceParentContext::HazardItems => {
                self == Self::Consumable
            }
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
    CharacterItems,
    HazardItems,
}

impl SourceParentContext {
    pub const fn npc_items() -> Self {
        Self::NpcItems
    }

    pub const fn relationship_path(&self) -> &'static str {
        match self {
            Self::NpcItems | Self::CharacterItems | Self::HazardItems => "items",
        }
    }

    fn display(&self) -> String {
        match self {
            Self::NpcItems => format!("Actor[npc].{}", self.relationship_path()),
            Self::CharacterItems => format!("Actor[character].{}", self.relationship_path()),
            Self::HazardItems => format!("Actor[hazard].{}", self.relationship_path()),
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
    pub(crate) effects: SourcePresence<Vec<SerializedSourceObject>>,
    pub(crate) flags: SourcePresence<SerializedSourceObject>,
    pub parent_context: Option<SourceParentContext>,
    pub lore: Option<LoreSource>,
    pub(crate) consumable: Option<ConsumableItemSource>,
    pub(crate) compendium_source: SourcePresence<String>,
    pub(crate) stable_source_locators: Vec<EmbeddedStableLocatorSource>,
    pub(crate) embedded_relationships: Vec<EmbeddedRelationshipSource>,
    pub(crate) relationship_unsupported: Vec<super::embedded::ValueSummary>,
    pub(crate) embedded: Option<NpcEmbeddedItemSource>,
    serialized: SerializedSourceObject,
    system: SerializedSourceObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumableItemSource {
    pub(crate) slug: ConsumableSourceFact<String>,
    pub(crate) level: ConsumableSourceFact<i64>,
    pub(crate) category: ConsumableSourceFact<String>,
    pub(crate) rarity: ConsumableSourceFact<String>,
    pub(crate) traits: ConsumableSourceFact<Vec<String>>,
    pub(crate) other_tags: ConsumableSourceFact<Vec<String>>,
    pub(crate) base_item: ConsumableSourceFact<String>,
    pub(crate) bulk: ConsumableSourceFact<String>,
    pub(crate) size: ConsumableSourceFact<String>,
    pub(crate) stack_group: ConsumableSourceFact<String>,
    pub(crate) quantity: ConsumableSourceFact<i64>,
    pub(crate) usage: ConsumableSourceFact<String>,
    pub(crate) maximum_uses: ConsumableSourceFact<i64>,
    pub(crate) current_uses: ConsumableSourceFact<i64>,
    pub(crate) auto_destroy: ConsumableSourceFact<bool>,
    pub(crate) container_id: ConsumableSourceFact<String>,
    pub(crate) equipped: ConsumableSourceFact<ConsumableEquippedSource>,
    pub(crate) maximum_hp: ConsumableSourceFact<i64>,
    pub(crate) current_hp: ConsumableSourceFact<i64>,
    pub(crate) hardness: ConsumableSourceFact<i64>,
    pub(crate) material: ConsumableSourceFact<ConsumableMaterialSource>,
    pub(crate) price: ConsumableSourceFact<ConsumablePriceSource>,
    pub(crate) damage: ConsumableSourceFact<ConsumableDamageSource>,
    pub(crate) publication: ConsumableSourceFact<ConsumablePublicationSource>,
    pub(crate) rules: ConsumableSourceFact<Vec<ConsumableUnsupportedSource>>,
    pub(crate) target_locator: ConsumableSourceFact<String>,
    pub(crate) unsupported_content: Vec<ConsumableUnsupportedSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConsumableSourceFact<T> {
    Missing,
    Null,
    Value(T),
    Unsupported(ConsumableUnsupportedSource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumableUnsupportedSource {
    pub(crate) shape: UnsupportedSourceShape,
    pub(crate) value: String,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumableEquippedSource {
    pub(crate) carry_type: ConsumableSourceFact<String>,
    pub(crate) hands_held: ConsumableSourceFact<i64>,
    pub(crate) in_slot: ConsumableSourceFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumableMaterialSource {
    pub(crate) grade: ConsumableSourceFact<String>,
    pub(crate) material_type: ConsumableSourceFact<String>,
    pub(crate) effects: ConsumableSourceFact<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumablePriceSource {
    pub(crate) denominations: ConsumableSourceFact<Vec<(String, i64)>>,
    pub(crate) per: ConsumableSourceFact<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumableDamageSource {
    pub(crate) formula: ConsumableSourceFact<String>,
    pub(crate) category: ConsumableSourceFact<String>,
    pub(crate) damage_type: ConsumableSourceFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumablePublicationSource {
    pub(crate) title: ConsumableSourceFact<String>,
    pub(crate) license: ConsumableSourceFact<String>,
    pub(crate) remaster: ConsumableSourceFact<bool>,
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
    let serialized = SerializedSourceObject::from_json(&raw).ok_or_else(|| {
        SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            &identity,
            "$",
            "Item source object",
            actual_shape(&raw),
        )
    })?;
    parse_item_source_from_serialized(version, identity, parent_context, raw, &serialized)
}

pub(crate) fn parse_item_source_from_serialized(
    version: SourceVersionMetadata,
    identity: SourceIdentity,
    parent_context: Option<SourceParentContext>,
    raw: Value,
    serialized: &SerializedSourceObject,
) -> Result<VersionedItemSource, SourceDiagnostic> {
    let source = parse_item(&identity, parent_context, &raw, serialized, "$")?;
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
    serialized: &SerializedSourceObject,
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
    let serialized_system = match serialized.member("system") {
        SerializedSourceMember::Value(SerializedSourceValue::Object(system)) => system,
        _ => {
            return Err(SourceDiagnostic::new(
                SourceDiagnosticKind::MalformedShape,
                identity,
                format!("{json_path}.system"),
                "one source object",
                "missing, null, non-object, or duplicate source member",
            ));
        }
    };
    let lore = (item_type == ItemType::Lore)
        .then(|| parse_lore_source(system, identity, &format!("{json_path}.system")))
        .transpose()?;
    let consumable = (item_type == ItemType::Consumable)
        .then(|| parse_consumable_source(serialized, serialized_system));
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
        consumable,
        compendium_source,
        stable_source_locators,
        embedded_relationships,
        relationship_unsupported,
        embedded,
        serialized: serialized.clone(),
        system: serialized_system.clone(),
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

fn parse_consumable_source(
    item: &SerializedSourceObject,
    system: &SerializedSourceObject,
) -> ConsumableItemSource {
    let uses =
        strict_consumable_object(system, "uses", &["max", "value", "autoDestroy"], |object| {
            (
                source_i64(object, &["max"]),
                source_i64(object, &["value"]),
                source_bool(object, &["autoDestroy"]),
            )
        });
    let (maximum_uses, current_uses, auto_destroy) = split_three_resource_facts(uses);
    let hp = strict_consumable_object(system, "hp", &["max", "value"], |object| {
        (source_i64(object, &["max"]), source_i64(object, &["value"]))
    });
    let (maximum_hp, current_hp) = split_two_resource_facts(hp);

    ConsumableItemSource {
        slug: source_string(system, &["slug"]),
        level: source_i64(system, &["level", "value"]),
        category: source_string(system, &["category"]),
        rarity: source_string(system, &["traits", "rarity"]),
        traits: source_string_array(system, &["traits", "value"]),
        other_tags: source_string_array(system, &["traits", "otherTags"]),
        base_item: source_string(system, &["baseItem"]),
        bulk: source_decimal(system, &["bulk", "value"]),
        size: source_string(system, &["size"]),
        stack_group: source_string(system, &["stackGroup"]),
        quantity: source_i64(system, &["quantity"]),
        usage: source_string(system, &["usage", "value"]),
        maximum_uses,
        current_uses,
        auto_destroy,
        container_id: source_string(system, &["containerId"]),
        equipped: strict_consumable_object(
            system,
            "equipped",
            &["carryType", "handsHeld", "inSlot"],
            |object| ConsumableEquippedSource {
                carry_type: source_string(object, &["carryType"]),
                hands_held: source_i64(object, &["handsHeld"]),
                in_slot: source_bool(object, &["inSlot"]),
            },
        ),
        maximum_hp,
        current_hp,
        hardness: source_i64(system, &["hardness"]),
        material: strict_consumable_object(
            system,
            "material",
            &["grade", "type", "effects"],
            |object| ConsumableMaterialSource {
                grade: source_string(object, &["grade"]),
                material_type: source_string(object, &["type"]),
                effects: source_string_array(object, &["effects"]),
            },
        ),
        price: strict_consumable_object(system, "price", &["value", "per"], |object| {
            ConsumablePriceSource {
                denominations: source_object(object, &["value"], |values| {
                    if let Some(unsupported) = unsupported_object_members(
                        values,
                        &["cp", "sp", "gp", "pp"],
                        "unsupported price denomination",
                    ) {
                        return Err(unsupported);
                    }
                    let mut denominations = Vec::with_capacity(values.fields().len());
                    for (denomination, value) in values.fields() {
                        let SerializedSourceValue::Number(amount) = value else {
                            return Err(ConsumableUnsupportedSource {
                                shape: UnsupportedSourceShape::Object,
                                value: values.compact_json(),
                                reason: "price denominations must be integers",
                            });
                        };
                        let Some(amount) = amount.as_i64() else {
                            return Err(ConsumableUnsupportedSource {
                                shape: UnsupportedSourceShape::Object,
                                value: values.compact_json(),
                                reason: "price denominations must be integers",
                            });
                        };
                        denominations.push((denomination.clone(), amount));
                    }
                    Ok(denominations)
                })
                .and_then_result(),
                per: source_i64(object, &["per"]),
            }
        }),
        damage: strict_consumable_object(
            system,
            "damage",
            &["formula", "kind", "type"],
            |object| ConsumableDamageSource {
                formula: source_string(object, &["formula"]),
                category: source_string(object, &["kind"]),
                damage_type: source_string(object, &["type"]),
            },
        ),
        publication: strict_consumable_object(
            system,
            "publication",
            &["title", "license", "remaster"],
            |object| ConsumablePublicationSource {
                title: source_string(object, &["title"]),
                license: source_string(object, &["license"]),
                remaster: source_bool(object, &["remaster"]),
            },
        ),
        rules: source_array(system, &["rules"], |values| {
            values
                .iter()
                .map(|value| ConsumableUnsupportedSource {
                    shape: value.shape(),
                    value: value.compact_json(),
                    reason: "rule execution is outside H5",
                })
                .collect()
        }),
        target_locator: consumable_target_locator(item),
        unsupported_content: consumable_unsupported_content(system),
    }
}

fn consumable_unsupported_content(
    system: &SerializedSourceObject,
) -> Vec<ConsumableUnsupportedSource> {
    match source_member(system, &["description"]) {
        ConsumableSourceFact::Missing | ConsumableSourceFact::Null => Vec::new(),
        ConsumableSourceFact::Unsupported(value) => vec![value],
        ConsumableSourceFact::Value(SerializedSourceValue::Object(description)) => {
            if let Some(unsupported) = unsupported_object_members(
                description,
                &["value", "gm"],
                "unrecognized or duplicate consumable description member",
            ) {
                return vec![unsupported];
            }
            ["value", "gm"]
                .into_iter()
                .filter_map(|member| match source_string(description, &[member]) {
                    ConsumableSourceFact::Unsupported(mut value) => {
                        value.reason = "consumable description must be a string";
                        Some(value)
                    }
                    ConsumableSourceFact::Missing
                    | ConsumableSourceFact::Null
                    | ConsumableSourceFact::Value(_) => None,
                })
                .collect()
        }
        ConsumableSourceFact::Value(value) => vec![unsupported(
            value,
            "consumable description must be an object",
        )],
    }
}

fn strict_consumable_object<T>(
    object: &SerializedSourceObject,
    member: &str,
    allowed_members: &[&str],
    project: impl FnOnce(&SerializedSourceObject) -> T,
) -> ConsumableSourceFact<T> {
    match source_member(object, &[member]) {
        ConsumableSourceFact::Missing => ConsumableSourceFact::Missing,
        ConsumableSourceFact::Null => ConsumableSourceFact::Null,
        ConsumableSourceFact::Unsupported(value) => ConsumableSourceFact::Unsupported(value),
        ConsumableSourceFact::Value(SerializedSourceValue::Object(value)) => {
            if let Some(unsupported) = unsupported_object_members(
                value,
                allowed_members,
                "unrecognized or duplicate nested source member",
            ) {
                ConsumableSourceFact::Unsupported(unsupported)
            } else {
                ConsumableSourceFact::Value(project(value))
            }
        }
        ConsumableSourceFact::Value(value) => {
            ConsumableSourceFact::Unsupported(unsupported(value, "expected nested resource object"))
        }
    }
}

fn unsupported_object_members(
    object: &SerializedSourceObject,
    allowed_members: &[&str],
    reason: &'static str,
) -> Option<ConsumableUnsupportedSource> {
    let mut observed = std::collections::BTreeSet::new();
    let unsupported = object
        .fields()
        .iter()
        .any(|(member, _)| !allowed_members.contains(&member.as_str()) || !observed.insert(member));
    unsupported.then(|| ConsumableUnsupportedSource {
        shape: UnsupportedSourceShape::Object,
        value: object.compact_json(),
        reason,
    })
}

fn split_two_resource_facts<A, B>(
    resource: ConsumableSourceFact<(ConsumableSourceFact<A>, ConsumableSourceFact<B>)>,
) -> (ConsumableSourceFact<A>, ConsumableSourceFact<B>) {
    match resource {
        ConsumableSourceFact::Missing => {
            (ConsumableSourceFact::Missing, ConsumableSourceFact::Missing)
        }
        ConsumableSourceFact::Null => (ConsumableSourceFact::Null, ConsumableSourceFact::Null),
        ConsumableSourceFact::Unsupported(value) => (
            ConsumableSourceFact::Unsupported(value.clone()),
            ConsumableSourceFact::Unsupported(value),
        ),
        ConsumableSourceFact::Value(values) => values,
    }
}

fn split_three_resource_facts<A, B, C>(
    resource: ConsumableSourceFact<(
        ConsumableSourceFact<A>,
        ConsumableSourceFact<B>,
        ConsumableSourceFact<C>,
    )>,
) -> (
    ConsumableSourceFact<A>,
    ConsumableSourceFact<B>,
    ConsumableSourceFact<C>,
) {
    match resource {
        ConsumableSourceFact::Missing => (
            ConsumableSourceFact::Missing,
            ConsumableSourceFact::Missing,
            ConsumableSourceFact::Missing,
        ),
        ConsumableSourceFact::Null => (
            ConsumableSourceFact::Null,
            ConsumableSourceFact::Null,
            ConsumableSourceFact::Null,
        ),
        ConsumableSourceFact::Unsupported(value) => (
            ConsumableSourceFact::Unsupported(value.clone()),
            ConsumableSourceFact::Unsupported(value.clone()),
            ConsumableSourceFact::Unsupported(value),
        ),
        ConsumableSourceFact::Value(values) => values,
    }
}

fn consumable_target_locator(item: &SerializedSourceObject) -> ConsumableSourceFact<String> {
    let compendium = source_string(item, &["_stats", "compendiumSource"]);
    let source_id = source_string(item, &["flags", "core", "sourceId"]);
    match (compendium, source_id) {
        (ConsumableSourceFact::Value(left), ConsumableSourceFact::Value(right))
            if left == right =>
        {
            ConsumableSourceFact::Value(left)
        }
        (
            ConsumableSourceFact::Value(left),
            ConsumableSourceFact::Missing | ConsumableSourceFact::Null,
        )
        | (
            ConsumableSourceFact::Missing | ConsumableSourceFact::Null,
            ConsumableSourceFact::Value(left),
        ) => ConsumableSourceFact::Value(left),
        (ConsumableSourceFact::Value(left), ConsumableSourceFact::Value(right)) => {
            ConsumableSourceFact::Unsupported(ConsumableUnsupportedSource {
                shape: UnsupportedSourceShape::Object,
                value: format!(
                    r#"{{"_stats.compendiumSource":{left:?},"flags.core.sourceId":{right:?}}}"#
                ),
                reason: "conflicting consumable target locators",
            })
        }
        (ConsumableSourceFact::Unsupported(value), _)
        | (_, ConsumableSourceFact::Unsupported(value)) => ConsumableSourceFact::Unsupported(value),
        (ConsumableSourceFact::Null, _) | (_, ConsumableSourceFact::Null) => {
            ConsumableSourceFact::Null
        }
        (ConsumableSourceFact::Missing, ConsumableSourceFact::Missing) => {
            ConsumableSourceFact::Missing
        }
    }
}

trait ConsumableSourceFactResult<T> {
    fn and_then_result(self) -> ConsumableSourceFact<T>;
}

impl<T> ConsumableSourceFactResult<T>
    for ConsumableSourceFact<Result<T, ConsumableUnsupportedSource>>
{
    fn and_then_result(self) -> ConsumableSourceFact<T> {
        match self {
            ConsumableSourceFact::Missing => ConsumableSourceFact::Missing,
            ConsumableSourceFact::Null => ConsumableSourceFact::Null,
            ConsumableSourceFact::Unsupported(value) => ConsumableSourceFact::Unsupported(value),
            ConsumableSourceFact::Value(Ok(value)) => ConsumableSourceFact::Value(value),
            ConsumableSourceFact::Value(Err(value)) => ConsumableSourceFact::Unsupported(value),
        }
    }
}

fn source_member<'a>(
    object: &'a SerializedSourceObject,
    path: &[&str],
) -> ConsumableSourceFact<&'a SerializedSourceValue> {
    let mut current = object;
    for (index, segment) in path.iter().enumerate() {
        match current.member(segment) {
            SerializedSourceMember::Missing => return ConsumableSourceFact::Missing,
            SerializedSourceMember::Null => return ConsumableSourceFact::Null,
            SerializedSourceMember::Duplicate(values) => {
                return ConsumableSourceFact::Unsupported(ConsumableUnsupportedSource {
                    shape: UnsupportedSourceShape::Array,
                    value: format!(
                        "[{}]",
                        values
                            .iter()
                            .map(|value| value.compact_json())
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    reason: "duplicate source member",
                });
            }
            SerializedSourceMember::Value(value) if index + 1 == path.len() => {
                return ConsumableSourceFact::Value(value);
            }
            SerializedSourceMember::Value(SerializedSourceValue::Object(next)) => current = next,
            SerializedSourceMember::Value(value) => {
                return ConsumableSourceFact::Unsupported(unsupported(
                    value,
                    "non-object ancestor for requested descendant",
                ));
            }
        }
    }
    ConsumableSourceFact::Missing
}

fn source_string(object: &SerializedSourceObject, path: &[&str]) -> ConsumableSourceFact<String> {
    project_source(
        source_member(object, path),
        |value| match value {
            SerializedSourceValue::String(value) => Some(value.clone()),
            _ => None,
        },
        "expected string",
    )
}

fn source_i64(object: &SerializedSourceObject, path: &[&str]) -> ConsumableSourceFact<i64> {
    project_source(
        source_member(object, path),
        |value| match value {
            SerializedSourceValue::Number(value) => value.as_i64(),
            _ => None,
        },
        "expected integer",
    )
}

fn source_bool(object: &SerializedSourceObject, path: &[&str]) -> ConsumableSourceFact<bool> {
    project_source(
        source_member(object, path),
        |value| match value {
            SerializedSourceValue::Boolean(value) => Some(*value),
            _ => None,
        },
        "expected boolean",
    )
}

fn source_decimal(object: &SerializedSourceObject, path: &[&str]) -> ConsumableSourceFact<String> {
    project_source(
        source_member(object, path),
        |value| match value {
            SerializedSourceValue::Number(value) => Some(value.to_string()),
            _ => None,
        },
        "expected exact decimal number",
    )
}

fn source_string_array(
    object: &SerializedSourceObject,
    path: &[&str],
) -> ConsumableSourceFact<Vec<String>> {
    project_source(
        source_member(object, path),
        |value| match value {
            SerializedSourceValue::Array(values) => values
                .iter()
                .map(|value| value.string().map(ToOwned::to_owned))
                .collect::<Option<Vec<_>>>(),
            _ => None,
        },
        "expected string array",
    )
}

fn source_object<T>(
    object: &SerializedSourceObject,
    path: &[&str],
    project: impl FnOnce(&SerializedSourceObject) -> T,
) -> ConsumableSourceFact<T> {
    project_source(
        source_member(object, path),
        |value| value.object().map(project),
        "expected object",
    )
}

fn source_array<T>(
    object: &SerializedSourceObject,
    path: &[&str],
    project: impl FnOnce(&[SerializedSourceValue]) -> T,
) -> ConsumableSourceFact<T> {
    project_source(
        source_member(object, path),
        |value| match value {
            SerializedSourceValue::Array(values) => Some(project(values)),
            _ => None,
        },
        "expected array",
    )
}

fn project_source<T>(
    source: ConsumableSourceFact<&SerializedSourceValue>,
    project: impl FnOnce(&SerializedSourceValue) -> Option<T>,
    reason: &'static str,
) -> ConsumableSourceFact<T> {
    match source {
        ConsumableSourceFact::Missing => ConsumableSourceFact::Missing,
        ConsumableSourceFact::Null => ConsumableSourceFact::Null,
        ConsumableSourceFact::Unsupported(value) => ConsumableSourceFact::Unsupported(value),
        ConsumableSourceFact::Value(value) => project(value)
            .map(ConsumableSourceFact::Value)
            .unwrap_or_else(|| ConsumableSourceFact::Unsupported(unsupported(value, reason))),
    }
}

fn unsupported(value: &SerializedSourceValue, reason: &'static str) -> ConsumableUnsupportedSource {
    ConsumableUnsupportedSource {
        shape: value.shape(),
        value: value.compact_json(),
        reason,
    }
}

#[cfg(test)]
mod consumable_tests {
    use super::{ConsumableSourceFact, parse_consumable_source};
    use crate::source::dto::{
        SerializedSourceMember, SerializedSourceValue, parse_serialized_source_object,
    };

    fn source(bytes: &[u8]) -> super::ConsumableItemSource {
        let root = parse_serialized_source_object(bytes).expect("serialized source object");
        let system = match root.member("system") {
            SerializedSourceMember::Value(SerializedSourceValue::Object(value)) => value.clone(),
            _ => panic!("fixture system object"),
        };
        parse_consumable_source(&root, &system)
    }

    #[test]
    fn consumable_descendants_distinguish_missing_null_malformed_and_duplicate() {
        let missing = source(br#"{"system":{}}"#);
        assert!(matches!(
            missing.current_uses,
            ConsumableSourceFact::Missing
        ));

        let null = source(br#"{"system":{"uses":{"value":null}}}"#);
        assert!(matches!(null.current_uses, ConsumableSourceFact::Null));

        let malformed = source(br#"{"system":{"uses":7}}"#);
        let ConsumableSourceFact::Unsupported(maximum_uses) = &malformed.maximum_uses else {
            panic!("scalar ancestor must be unsupported");
        };
        assert_eq!(maximum_uses.value, "7");
        assert_eq!(maximum_uses.reason, "expected nested resource object");
        let ConsumableSourceFact::Unsupported(current_uses) = &malformed.current_uses else {
            panic!("malformed resource evidence must reach current uses");
        };
        let ConsumableSourceFact::Unsupported(auto_destroy) = &malformed.auto_destroy else {
            panic!("malformed resource evidence must reach auto-destroy");
        };
        assert_eq!(current_uses.value, "7");
        assert_eq!(auto_destroy.value, "7");

        let duplicate = source(br#"{"system":{"uses":{"value":1,"value":2}}}"#);
        let ConsumableSourceFact::Unsupported(maximum_uses) = &duplicate.maximum_uses else {
            panic!("duplicate resource must be unsupported once");
        };
        assert_eq!(maximum_uses.value, r#"{"value":1,"value":2}"#);
        assert_eq!(
            maximum_uses.reason,
            "unrecognized or duplicate nested source member"
        );
        let ConsumableSourceFact::Unsupported(current_uses) = &duplicate.current_uses else {
            panic!("duplicate resource evidence must reach current uses");
        };
        assert_eq!(current_uses.value, r#"{"value":1,"value":2}"#);

        let unknown = source(br#"{"system":{"uses":{"max":3,"unexpected":7}}}"#);
        let ConsumableSourceFact::Unsupported(maximum_uses) = &unknown.maximum_uses else {
            panic!("unknown resource member must retain the whole ordered object");
        };
        assert_eq!(maximum_uses.value, r#"{"max":3,"unexpected":7}"#);
        let ConsumableSourceFact::Unsupported(current_uses) = &unknown.current_uses else {
            panic!("unknown resource evidence must reach current uses");
        };
        let ConsumableSourceFact::Unsupported(auto_destroy) = &unknown.auto_destroy else {
            panic!("unknown resource evidence must reach auto-destroy");
        };
        assert_eq!(current_uses.value, r#"{"max":3,"unexpected":7}"#);
        assert_eq!(auto_destroy.value, r#"{"max":3,"unexpected":7}"#);

        let equipped =
            source(br#"{"system":{"equipped":{"carryType":"held","handsHeld":1,"slot":"belt"}}}"#);
        let ConsumableSourceFact::Unsupported(equipped) = equipped.equipped else {
            panic!("unknown equipped member must retain the whole ordered object");
        };
        assert_eq!(
            equipped.value,
            r#"{"carryType":"held","handsHeld":1,"slot":"belt"}"#
        );
    }

    #[test]
    fn malformed_nested_resources_and_price_maps_retain_exact_evidence() {
        let malformed_hp = source(br#"{"system":{"hp":false}}"#);
        let ConsumableSourceFact::Unsupported(maximum_hp) = malformed_hp.maximum_hp else {
            panic!("malformed HP evidence must reach maximum HP");
        };
        let ConsumableSourceFact::Unsupported(current_hp) = malformed_hp.current_hp else {
            panic!("malformed HP evidence must reach current HP");
        };
        assert_eq!(maximum_hp.value, "false");
        assert_eq!(current_hp.value, "false");

        let malformed_resources = source(
            br#"{"system":{"hp":{"max":10,"value":{"bad":1}},"price":{"value":{"gp":2,"sp":"three"}}}}"#,
        );
        let ConsumableSourceFact::Unsupported(current_hp) = malformed_resources.current_hp else {
            panic!("malformed current HP must be unsupported");
        };
        assert_eq!(current_hp.value, r#"{"bad":1}"#);

        let ConsumableSourceFact::Value(price) = malformed_resources.price else {
            panic!("price object remains present");
        };
        let ConsumableSourceFact::Unsupported(denominations) = price.denominations else {
            panic!("one malformed denomination invalidates the typed denomination map");
        };
        assert_eq!(denominations.value, r#"{"gp":2,"sp":"three"}"#);
        assert_eq!(denominations.reason, "price denominations must be integers");

        let unknown_price = source(br#"{"system":{"price":{"value":{"credits":4},"per":1}}}"#);
        let ConsumableSourceFact::Value(price) = unknown_price.price else {
            panic!("price object remains present");
        };
        let ConsumableSourceFact::Unsupported(denominations) = price.denominations else {
            panic!("unknown denomination must be retained as one exact map");
        };
        assert_eq!(denominations.value, r#"{"credits":4}"#);

        let unknown_damage = source(
            br#"{"system":{"damage":{"formula":"1d6","kind":"damage","type":"fire","unexpected":true}}}"#,
        );
        let ConsumableSourceFact::Unsupported(damage) = unknown_damage.damage else {
            panic!("unknown damage member must retain the complete damage map");
        };
        assert_eq!(
            damage.value,
            r#"{"formula":"1d6","kind":"damage","type":"fire","unexpected":true}"#
        );

        let damage = source(
            br#"{"system":{"damage":{"formula":"2d8+5","kind":"healing","type":"vitality"}}}"#,
        );
        let ConsumableSourceFact::Value(damage) = damage.damage else {
            panic!("structured consumable damage must remain typed");
        };
        assert_eq!(
            damage.formula,
            ConsumableSourceFact::Value("2d8+5".to_string())
        );
        assert_eq!(
            damage.category,
            ConsumableSourceFact::Value("healing".to_string())
        );
        assert_eq!(
            damage.damage_type,
            ConsumableSourceFact::Value("vitality".to_string())
        );
    }

    #[test]
    fn consumable_locator_prefers_exact_lineage_and_rejects_conflicts() {
        let stats = source(
            br#"{"_stats":{"compendiumSource":"Compendium.pf2e.equipment-srd.Item.target"},"system":{}}"#,
        );
        assert_eq!(
            stats.target_locator,
            ConsumableSourceFact::Value("Compendium.pf2e.equipment-srd.Item.target".to_string())
        );

        let flags = source(
            br#"{"flags":{"core":{"sourceId":"Compendium.pf2e.equipment-srd.Item.target"}},"system":{}}"#,
        );
        assert_eq!(stats.target_locator, flags.target_locator);

        let conflict = source(
            br#"{"_stats":{"compendiumSource":"Compendium.pf2e.equipment-srd.Item.first"},"flags":{"core":{"sourceId":"Compendium.pf2e.equipment-srd.Item.second"}},"system":{}}"#,
        );
        let ConsumableSourceFact::Unsupported(conflict) = conflict.target_locator else {
            panic!("conflicting exact locators must fail closed");
        };
        assert_eq!(conflict.reason, "conflicting consumable target locators");
        assert!(conflict.value.contains("first"));
        assert!(conflict.value.contains("second"));
    }

    #[test]
    fn malformed_consumable_content_is_retained_once() {
        for (bytes, exact) in [
            (
                &br#"{"system":{"description":{"value":"first","value":"second"}}}"#[..],
                r#"{"value":"first","value":"second"}"#,
            ),
            (&br#"{"system":{"description":7}}"#[..], "7"),
        ] {
            let source = source(bytes);
            assert_eq!(source.unsupported_content.len(), 1);
            assert_eq!(source.unsupported_content[0].value, exact);
        }
    }
}
