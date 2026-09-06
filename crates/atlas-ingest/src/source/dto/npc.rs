use std::fmt;

use serde_json::Value;

use super::creature_core::parse_npc_core;
use super::embedded::{ActorSpellcastingSource, parse_actor_spellcasting};
use super::item::parse_item;
use super::{
    ItemSource, NpcCoreSource, RawSourceJson, SerializedSourceMember, SerializedSourceObject,
    SerializedSourceValue, SourceDiagnostic, SourceDiagnosticKind, SourceIdentity,
    SourceParentContext, SourcePresence, SourceVersionMetadata, actual_shape,
    optional_array_of_objects, optional_integer, optional_object, optional_string, required_object,
    required_string,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorType {
    Army,
    Character,
    Familiar,
    Hazard,
    Loot,
    Npc,
    Party,
    Vehicle,
}

impl ActorType {
    pub const ALL: [Self; 8] = [
        Self::Army,
        Self::Character,
        Self::Familiar,
        Self::Hazard,
        Self::Loot,
        Self::Npc,
        Self::Party,
        Self::Vehicle,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Army => "army",
            Self::Character => "character",
            Self::Familiar => "familiar",
            Self::Hazard => "hazard",
            Self::Loot => "loot",
            Self::Npc => "npc",
            Self::Party => "party",
            Self::Vehicle => "vehicle",
        }
    }

    fn parse(
        value: &str,
        identity: &SourceIdentity,
        json_path: &str,
    ) -> Result<Self, SourceDiagnostic> {
        Self::ALL
            .into_iter()
            .find(|actor_type| actor_type.as_str() == value)
            .ok_or_else(|| {
                SourceDiagnostic::new(
                    SourceDiagnosticKind::UnknownDiscriminator,
                    identity,
                    json_path,
                    format!(
                        "one of [{}]",
                        Self::ALL
                            .iter()
                            .map(|actor_type| actor_type.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    format!("unknown Actor discriminator {value:?}"),
                )
            })
    }
}

impl fmt::Display for ActorType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Persisted NPC source before Foundry prepares defaults or derived data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSource {
    pub id: String,
    pub name: String,
    pub actor_type: ActorType,
    pub image: SourcePresence<String>,
    pub folder: SourcePresence<String>,
    pub sort: SourcePresence<i64>,
    pub(crate) ownership: SourcePresence<SerializedSourceObject>,
    pub(crate) effects: SourcePresence<Vec<SerializedSourceObject>>,
    pub items: SourcePresence<Vec<ItemSource>>,
    pub core: NpcCoreSource,
    pub(crate) spellcasting: SourcePresence<ActorSpellcastingSource>,
    serialized: SerializedSourceObject,
    system: SerializedSourceObject,
}

impl NpcSource {
    #[cfg(test)]
    pub(super) fn serialized(&self) -> &SerializedSourceObject {
        &self.serialized
    }

    #[cfg(test)]
    pub(super) fn system(&self) -> &SerializedSourceObject {
        &self.system
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedNpcSource {
    pub version: SourceVersionMetadata,
    pub source: NpcSource,
    provenance: RawSourceJson,
}

impl VersionedNpcSource {
    /// The immutable original JSON for explicit provenance/audit workflows.
    pub fn raw_json_for_audit(&self) -> &Value {
        self.provenance.for_audit()
    }
}

pub fn parse_npc_source(
    version: SourceVersionMetadata,
    identity: SourceIdentity,
    raw: Value,
) -> Result<VersionedNpcSource, SourceDiagnostic> {
    let serialized = SerializedSourceObject::from_json(&raw).ok_or_else(|| {
        SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            &identity,
            "$",
            "Actor source object",
            actual_shape(&raw),
        )
    })?;
    parse_npc_source_from_serialized(version, identity, raw, &serialized)
}

pub(crate) fn parse_npc_source_from_serialized(
    version: SourceVersionMetadata,
    identity: SourceIdentity,
    raw: Value,
    serialized: &SerializedSourceObject,
) -> Result<VersionedNpcSource, SourceDiagnostic> {
    let map = raw.as_object().ok_or_else(|| {
        SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            &identity,
            "$",
            "Actor source object",
            actual_shape(&raw),
        )
    })?;
    let id = required_string(map, "_id", &identity, "$._id")?;
    let name = required_string(map, "name", &identity, "$.name")?;
    let actor_type_value = required_string(map, "type", &identity, "$.type")?;
    let actor_type = ActorType::parse(&actor_type_value, &identity, "$.type")?;
    if actor_type != ActorType::Npc {
        return Err(SourceDiagnostic::new(
            SourceDiagnosticKind::InvalidParentContext,
            &identity,
            "$.type",
            "Actor discriminator \"npc\"",
            format!("Actor discriminator {actor_type:?}"),
        ));
    }
    let system = required_object(map, "system", &identity, "$.system")?;
    let serialized_system = match serialized.member("system") {
        SerializedSourceMember::Value(SerializedSourceValue::Object(system)) => system,
        _ => {
            return Err(SourceDiagnostic::new(
                SourceDiagnosticKind::MalformedShape,
                &identity,
                "$.system",
                "one source object",
                "missing, null, non-object, or duplicate source member",
            ));
        }
    };
    let core = parse_npc_core(system, &identity)?;
    let serialized_items = match serialized.member("items") {
        SerializedSourceMember::Value(SerializedSourceValue::Array(items)) => {
            Some(items.as_slice())
        }
        _ => None,
    };
    let items = match map.get("items") {
        None => SourcePresence::Missing,
        Some(Value::Null) => SourcePresence::Null,
        Some(Value::Array(items)) => SourcePresence::Value(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let serialized_item = serialized_items
                        .and_then(|items| items.get(index))
                        .and_then(SerializedSourceValue::object)
                        .ok_or_else(|| {
                            SourceDiagnostic::new(
                                SourceDiagnosticKind::MalformedShape,
                                &identity,
                                format!("$.items[{index}]"),
                                "one source object",
                                "missing or non-object source member",
                            )
                        })?;
                    parse_item(
                        &identity,
                        Some(SourceParentContext::npc_items()),
                        item,
                        serialized_item,
                        &format!("$.items[{index}]"),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Some(other) => {
            return Err(SourceDiagnostic::new(
                SourceDiagnosticKind::MalformedShape,
                &identity,
                "$.items",
                "array of full Item source objects",
                actual_shape(other),
            ));
        }
    };

    let source = NpcSource {
        id,
        name,
        actor_type,
        image: optional_string(map, "img", &identity, "$.img")?,
        folder: optional_string(map, "folder", &identity, "$.folder")?,
        sort: optional_integer(map, "sort", &identity, "$.sort")?,
        ownership: optional_object(map, "ownership", &identity, "$.ownership")?,
        effects: optional_array_of_objects(map, "effects", &identity, "$.effects")?,
        items,
        core,
        spellcasting: parse_actor_spellcasting(system),
        serialized: serialized.clone(),
        system: serialized_system.clone(),
    };

    Ok(VersionedNpcSource {
        version,
        source,
        provenance: RawSourceJson::new(raw),
    })
}
