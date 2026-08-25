use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_record::{
    ActivityRoll, ActivityRollAbility, ActivityRollSurface, CreatureActionCapability,
    CreatureActionCost, CreatureActorSpellcastingContext, CreatureCapability, CreatureDamage,
    CreatureDamageKind, CreatureDeltaDisposition, CreatureDeltaValue, CreatureEmbeddedEntities,
    CreatureEntity, CreatureEntityFamily, CreatureEntityId, CreatureEntityOccurrence,
    CreatureEntityRelationship, CreatureEntityRelationshipKind, CreatureEntitySourceIdentity,
    CreatureEntityTarget, CreatureEquipmentCapability, CreatureFact, CreatureLoreCapability,
    CreatureOccurrenceContext, CreatureOccurrenceDelta, CreatureOccurrenceId,
    CreatureOccurrenceParent, CreaturePreparedSpellSlot, CreatureRelationshipExecution,
    CreatureRelationshipTarget, CreatureRitualContext, CreatureRoll, CreatureRollKind,
    CreatureSourceField, CreatureSourceId, CreatureSourceLocator, CreatureSourceScalar,
    CreatureSpellArea, CreatureSpellCapability, CreatureSpellDefense, CreatureSpellDuration,
    CreatureSpellPreparation, CreatureSpellSave, CreatureSpellSlot,
    CreatureSpellcastingEntryCapability, CreatureStrikeCapability, CreatureUnsupportedCapability,
    CreatureUseLimit, DamageEffectKind, DamageExpression, FactValue, MechanicActivity,
    MechanicActivityKind, MechanicActivityUsage, OccurrenceIdentityStability, RecordBody,
    SpellcastingEntryMechanics, SpellcastingPreparation, StableSourceLocator,
    UnsupportedMechanicNote, UnsupportedSourceReason, UnsupportedSourceShape,
    UnsupportedSourceValue,
};
use serde::Serialize;

use crate::records::{LoadedSourceRecord, RecordReferenceIndex};

use super::dto::{
    ActionSource, DamageSource, EmbeddedSourceScalar, EquipmentSource, FullItemSource,
    NpcEmbeddedItemSource, PreparedSlotSource, SourcePresence, SourceTypeDrift, SpellDefenseSource,
    SpellSource, SpellcastingEntrySource, StrikeSource, UseLimitSource, ValueSummary,
    VersionedNpcSource,
};
use super::dto::{EmbeddedRelationshipKindSource, EmbeddedRelationshipSource};

pub(crate) const RETAINED_CAPABILITY_PATHS: [&str; 13] = [
    "$.items[].system.attackEffects.custom",
    "$.items[].system.area.details",
    "$.items[].system.damage.*.materials[]",
    "$.items[].system.defense.passive.statistic",
    "$.items[].system.location.autoHeightenLevel",
    "$.items[].system.prepared.flexible",
    "$.items[].system.prepared.label",
    "$.items[].system.prepared.type",
    "$.items[].system.prepared.validItems",
    "$.items[].system.spelldc.item",
    "$.items[].system.spelldc.label",
    "$.items[].system.spelldc.mod",
    "$.items[].system.spelldc.type",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NpcEmbeddedCandidates {
    pub(crate) items: SourcePresence<Vec<NpcEmbeddedCandidate>>,
    pub(crate) actor_spellcasting: SourcePresence<super::dto::ActorSpellcastingSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NpcEmbeddedCandidate {
    pub(crate) nested_source_id: String,
    pub(crate) label: String,
    pub(crate) compendium_source: SourcePresence<String>,
    pub(crate) sort: SourcePresence<i64>,
    pub(crate) folder: SourcePresence<String>,
    pub(crate) stable_source_locators: Vec<super::dto::EmbeddedStableLocatorSource>,
    pub(crate) relationships: Vec<EmbeddedRelationshipSource>,
    pub(crate) relationship_unsupported: Vec<ValueSummary>,
    pub(crate) source: NpcEmbeddedItemSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct NpcEmbeddedDiagnostic {
    pub(crate) code: &'static str,
    pub(crate) kind: NpcEmbeddedDiagnosticKind,
    pub(crate) record_key: String,
    pub(crate) source_path: String,
    pub(crate) source_item_id: Option<String>,
    pub(crate) source_field: String,
    pub(crate) expected_shape: String,
    pub(crate) observed_shape: String,
    pub(crate) source_value: String,
    pub(crate) disposition: NpcEmbeddedDiagnosticDisposition,
    pub(crate) owner: NpcEmbeddedDiagnosticOwner,
}

impl NpcEmbeddedDiagnostic {
    fn new(
        kind: NpcEmbeddedDiagnosticKind,
        source_item_id: Option<String>,
        source_field: impl Into<String>,
        expected_shape: impl Into<String>,
        observed_shape: impl Into<String>,
        source_value: impl Into<String>,
    ) -> Self {
        Self {
            code: kind.code(),
            kind,
            record_key: String::new(),
            source_path: String::new(),
            source_item_id,
            source_field: source_field.into(),
            expected_shape: expected_shape.into(),
            observed_shape: observed_shape.into(),
            source_value: source_value.into(),
            disposition: kind.disposition(),
            owner: NpcEmbeddedDiagnosticOwner::CreatureEmbeddedEntities,
        }
    }

    fn bind(&mut self, record_key: &RecordKey, source_path: &str) {
        self.record_key = record_key.to_string();
        self.source_path = source_path.to_string();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NpcEmbeddedDiagnosticKind {
    UnresolvedStableSourceIdentity,
    InvalidStableSourceIdentity,
    UnstableOwnerFamilyOrdinalIdentity,
    UnsupportedMechanic,
    SourceTypeDrift,
}

impl NpcEmbeddedDiagnosticKind {
    const fn code(self) -> &'static str {
        match self {
            Self::UnresolvedStableSourceIdentity => {
                "atlas.npc_embedded.unresolved_stable_source_identity.v1"
            }
            Self::InvalidStableSourceIdentity => {
                "atlas.npc_embedded.invalid_stable_source_identity.v1"
            }
            Self::UnstableOwnerFamilyOrdinalIdentity => {
                "atlas.npc_embedded.unstable_owner_family_ordinal_identity.v1"
            }
            Self::UnsupportedMechanic => "atlas.npc_embedded.unsupported_mechanic.v1",
            Self::SourceTypeDrift => "atlas.npc_embedded.source_type_drift.v1",
        }
    }

    const fn disposition(self) -> NpcEmbeddedDiagnosticDisposition {
        match self {
            Self::UnresolvedStableSourceIdentity | Self::InvalidStableSourceIdentity => {
                NpcEmbeddedDiagnosticDisposition::ActorOwnedFallback
            }
            Self::UnstableOwnerFamilyOrdinalIdentity => {
                NpcEmbeddedDiagnosticDisposition::DiagnosedScopedOrdinal
            }
            Self::UnsupportedMechanic | Self::SourceTypeDrift => {
                NpcEmbeddedDiagnosticDisposition::TypedUnsupported
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NpcEmbeddedDiagnosticDisposition {
    #[serde(rename = "preserved_with_actor_owned_fallback")]
    ActorOwnedFallback,
    #[serde(rename = "preserved_with_diagnosed_scoped_ordinal")]
    DiagnosedScopedOrdinal,
    #[serde(rename = "preserved_as_typed_unsupported")]
    TypedUnsupported,
}

impl NpcEmbeddedDiagnosticDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ActorOwnedFallback => "preserved_with_actor_owned_fallback",
            Self::DiagnosedScopedOrdinal => "preserved_with_diagnosed_scoped_ordinal",
            Self::TypedUnsupported => "preserved_as_typed_unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NpcEmbeddedDiagnosticOwner {
    CreatureEmbeddedEntities,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NpcEmbeddedConversion {
    pub(crate) embedded: FactValue<CreatureEmbeddedEntities>,
    pub(crate) diagnostics: Vec<NpcEmbeddedDiagnostic>,
}

pub(crate) fn collect_npc_embedded_candidates(
    source: &VersionedNpcSource,
) -> NpcEmbeddedCandidates {
    let items = match &source.source.items {
        SourcePresence::Missing => SourcePresence::Missing,
        SourcePresence::Null => SourcePresence::Null,
        SourcePresence::Value(items) => SourcePresence::Value(
            items
                .iter()
                .filter_map(|item| candidate(item.source()))
                .collect(),
        ),
    };
    NpcEmbeddedCandidates {
        items,
        actor_spellcasting: source.source.spellcasting.clone(),
    }
}

fn candidate(source: &FullItemSource) -> Option<NpcEmbeddedCandidate> {
    Some(NpcEmbeddedCandidate {
        nested_source_id: source.id.clone(),
        label: source.name.clone(),
        compendium_source: source.compendium_source.clone(),
        sort: source.sort.clone(),
        folder: source.folder.clone(),
        stable_source_locators: source.stable_source_locators.clone(),
        relationships: source.embedded_relationships.clone(),
        relationship_unsupported: source.relationship_unsupported.clone(),
        source: source.embedded.clone()?,
    })
}

pub(crate) fn finalize_npc_embedded_entities(
    records: &mut [LoadedSourceRecord],
    index: &RecordReferenceIndex,
) {
    for loaded in records {
        let Some(candidates) = loaded.facts.npc_embedded_candidates.clone() else {
            continue;
        };
        let owner = loaded.record.identity.key.clone();
        let mut conversion = convert_npc_embedded_entities(owner.clone(), &candidates, |locator| {
            resolve_verified_stable_locator(locator, index)
        });
        for diagnostic in &mut conversion.diagnostics {
            diagnostic.bind(&owner, &loaded.record.provenance.source_path);
        }
        let FactValue::Value(embedded) = &conversion.embedded else {
            if let Some(RecordBody::Creature(creature)) = &mut loaded.facts.canonical_body {
                creature.embedded_entities = CreatureFact::source(
                    conversion.embedded,
                    CreatureSourceField::EmbeddedEntities,
                );
            }
            loaded.facts.npc_embedded_diagnostics = conversion.diagnostics;
            continue;
        };
        let (entries, activities) = legacy_projection(embedded);
        loaded.record.mechanics.spellcasting_entries = entries;
        loaded.record.mechanics.activities = activities;
        if let Some(RecordBody::Creature(creature)) = &mut loaded.facts.canonical_body {
            creature.embedded_entities =
                CreatureFact::source(conversion.embedded, CreatureSourceField::EmbeddedEntities);
        }
        loaded.facts.npc_embedded_diagnostics = conversion.diagnostics;
    }
}

pub(crate) fn convert_npc_embedded_entities(
    owner: RecordKey,
    candidates: &NpcEmbeddedCandidates,
    mut resolve: impl FnMut(&str) -> Option<RecordKey>,
) -> NpcEmbeddedConversion {
    let items = match &candidates.items {
        SourcePresence::Missing => {
            return NpcEmbeddedConversion {
                embedded: FactValue::Missing,
                diagnostics: Vec::new(),
            };
        }
        SourcePresence::Null => {
            return NpcEmbeddedConversion {
                embedded: FactValue::Null,
                diagnostics: Vec::new(),
            };
        }
        SourcePresence::Value(items) => items,
    };
    let mut entities = Vec::new();
    let mut occurrences = Vec::new();
    let mut diagnostics = Vec::new();
    let mut seen_entities = BTreeSet::new();
    let entry_occurrences = items
        .iter()
        .filter_map(|candidate| {
            matches!(
                candidate.source,
                NpcEmbeddedItemSource::SpellcastingEntry(_)
            )
            .then(|| {
                CreatureSourceId::new(candidate.nested_source_id.clone())
                    .ok()
                    .map(|source_id| {
                        (
                            candidate.nested_source_id.clone(),
                            CreatureOccurrenceId::stable_nested(
                                &owner,
                                CreatureEntityFamily::SpellcastingEntry,
                                &source_id,
                            ),
                        )
                    })
            })
            .flatten()
        })
        .collect::<BTreeMap<_, _>>();

    let mut ordered = items.iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by(|(left_index, left), (right_index, right)| {
        match (left.sort.as_value(), right.sort.as_value()) {
            (Some(left_sort), Some(right_sort)) => left_sort
                .cmp(right_sort)
                .then_with(|| left.nested_source_id.cmp(&right.nested_source_id))
                .then_with(|| left_index.cmp(right_index)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left_index.cmp(right_index),
        }
    });
    let occurrence_ids = items
        .iter()
        .enumerate()
        .map(|(ordinal, candidate)| {
            let family = family(&candidate.source);
            let (id, stability, nested) = occurrence_identity(
                &owner,
                family,
                &candidate.nested_source_id,
                ordinal,
                &mut diagnostics,
            );
            (
                candidate.nested_source_id.clone(),
                (id, stability, nested, family),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let prepared_context = prepared_spell_context(items, &owner);

    for (authored_order, (_, candidate)) in ordered.into_iter().enumerate() {
        let family = family(&candidate.source);
        let Some((occurrence_id, stability, nested_source_id, _)) =
            occurrence_ids.get(&candidate.nested_source_id).cloned()
        else {
            continue;
        };
        let source_identity = source_identity(candidate, nested_source_id.clone());
        let preferred_locator = preferred_locator(candidate);
        let target = match preferred_locator {
            SourcePresence::Value(locator) if !locator.trim().is_empty() => {
                if let Some(record_key) = resolve(&locator) {
                    CreatureEntityTarget::CanonicalRecord(record_key)
                } else {
                    diagnostics.push(NpcEmbeddedDiagnostic::new(
                        NpcEmbeddedDiagnosticKind::UnresolvedStableSourceIdentity,
                        Some(candidate.nested_source_id.clone()),
                        "$._stats.compendiumSource",
                        "verified canonical RecordKey",
                        "unresolved stable locator",
                        &locator,
                    ));
                    actor_owned_target(
                        &owner,
                        family,
                        &occurrence_id,
                        &candidate.label,
                        source_identity.clone(),
                        &mut entities,
                        &mut seen_entities,
                    )
                }
            }
            SourcePresence::Value(locator) => {
                diagnostics.push(NpcEmbeddedDiagnostic::new(
                    NpcEmbeddedDiagnosticKind::InvalidStableSourceIdentity,
                    Some(candidate.nested_source_id.clone()),
                    "$._stats.compendiumSource",
                    "non-empty stable locator",
                    "empty string",
                    &locator,
                ));
                actor_owned_target(
                    &owner,
                    family,
                    &occurrence_id,
                    &candidate.label,
                    source_identity.clone(),
                    &mut entities,
                    &mut seen_entities,
                )
            }
            _ => actor_owned_target(
                &owner,
                family,
                &occurrence_id,
                &candidate.label,
                source_identity.clone(),
                &mut entities,
                &mut seen_entities,
            ),
        };
        let mut context = context(candidate);
        let mut parent = match &candidate.source {
            NpcEmbeddedItemSource::Spell(spell) => spell
                .location
                .as_value()
                .and_then(|entry_id| entry_occurrences.get(entry_id))
                .cloned()
                .map(CreatureOccurrenceParent::SpellcastingEntry)
                .unwrap_or(CreatureOccurrenceParent::Creature),
            _ => CreatureOccurrenceParent::Creature,
        };
        if let Some((entry, rank, slot)) = prepared_context.get(&candidate.nested_source_id) {
            parent = CreatureOccurrenceParent::SpellcastingEntry(entry.clone());
            context.rank = FactValue::Value(*rank);
            context.slot = FactValue::Value(slot.clone());
        }
        let deltas = matches!(target, CreatureEntityTarget::CanonicalRecord(_))
            .then(|| occurrence_deltas(candidate))
            .unwrap_or_default();
        let Some(capability) = capability(candidate, &mut diagnostics) else {
            continue;
        };
        occurrences.push(CreatureEntityOccurrence {
            id: occurrence_id,
            identity_stability: stability,
            owner: owner.clone(),
            target,
            family,
            authored_order: authored_order as u32,
            source_sort: presence(&candidate.sort),
            source_folder: presence(&candidate.folder),
            source_identity,
            parent,
            context,
            capability,
            deltas,
        });
    }

    for diagnostic in &mut diagnostics {
        diagnostic.record_key = owner.to_string();
    }
    let relationships = build_relationships(items, &occurrence_ids);
    let mut actor_diagnostics = Vec::new();
    let actor_spellcasting =
        actor_spellcasting_context(&candidates.actor_spellcasting, &mut actor_diagnostics);
    diagnostics.extend(actor_diagnostics);
    NpcEmbeddedConversion {
        embedded: FactValue::Value(CreatureEmbeddedEntities {
            entities,
            occurrences,
            relationships,
            actor_spellcasting,
        }),
        diagnostics,
    }
}

fn stable_locator(source: &SourcePresence<String>) -> FactValue<StableSourceLocator> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(value) => StableSourceLocator::new(value.clone())
            .map(FactValue::Value)
            .unwrap_or(FactValue::Missing),
    }
}

fn preferred_locator(candidate: &NpcEmbeddedCandidate) -> SourcePresence<String> {
    match &candidate.compendium_source {
        SourcePresence::Value(value) if !value.trim().is_empty() => {
            SourcePresence::Value(value.clone())
        }
        SourcePresence::Null => SourcePresence::Null,
        _ => candidate
            .stable_source_locators
            .iter()
            .min_by_key(|locator| locator.precedence)
            .map(|locator| SourcePresence::Value(locator.value.clone()))
            .unwrap_or(SourcePresence::Missing),
    }
}

fn source_identity(
    candidate: &NpcEmbeddedCandidate,
    nested_source_id: FactValue<CreatureSourceId>,
) -> CreatureEntitySourceIdentity {
    let mut source_locators = candidate
        .stable_source_locators
        .iter()
        .filter_map(|locator| {
            StableSourceLocator::new(locator.value.clone())
                .ok()
                .map(|value| CreatureSourceLocator {
                    source_path: locator.source_path.to_string(),
                    locator: value,
                    precedence: locator.precedence,
                })
        })
        .collect::<Vec<_>>();
    if let SourcePresence::Value(value) = &candidate.compendium_source
        && let Ok(locator) = StableSourceLocator::new(value.clone())
    {
        source_locators.push(CreatureSourceLocator {
            source_path: "$._stats.compendiumSource".to_string(),
            locator,
            precedence: 0,
        });
    }
    source_locators.sort_by_key(|locator| locator.precedence);
    CreatureEntitySourceIdentity {
        nested_source_id,
        stable_source_locator: stable_locator(&preferred_locator(candidate)),
        source_locators,
    }
}

fn prepared_spell_context(
    items: &[NpcEmbeddedCandidate],
    owner: &RecordKey,
) -> BTreeMap<String, (CreatureOccurrenceId, i64, String)> {
    let mut context = BTreeMap::new();
    for candidate in items {
        let NpcEmbeddedItemSource::SpellcastingEntry(entry) = &candidate.source else {
            continue;
        };
        let Ok(entry_id) = CreatureSourceId::new(candidate.nested_source_id.clone()) else {
            continue;
        };
        let occurrence = CreatureOccurrenceId::stable_nested(
            owner,
            CreatureEntityFamily::SpellcastingEntry,
            &entry_id,
        );
        let SourcePresence::Value(slots) = &entry.slots else {
            continue;
        };
        for slot in slots {
            let SourcePresence::Value(prepared) = &slot.prepared else {
                continue;
            };
            for (ordinal, prepared) in prepared.iter().enumerate() {
                let PreparedSlotSource::Spell {
                    id: SourcePresence::Value(id),
                    ..
                } = prepared
                else {
                    continue;
                };
                context.entry(id.clone()).or_insert_with(|| {
                    (
                        occurrence.clone(),
                        slot.rank,
                        format!("slot{}:{ordinal}", slot.rank),
                    )
                });
            }
        }
    }
    context
}

fn build_relationships(
    items: &[NpcEmbeddedCandidate],
    occurrence_ids: &BTreeMap<
        String,
        (
            CreatureOccurrenceId,
            OccurrenceIdentityStability,
            FactValue<CreatureSourceId>,
            CreatureEntityFamily,
        ),
    >,
) -> Vec<CreatureEntityRelationship> {
    let mut relationships = Vec::new();
    for candidate in items {
        let Some((source, _, _, _)) = occurrence_ids.get(&candidate.nested_source_id) else {
            continue;
        };
        for relationship in &candidate.relationships {
            let Ok(target_id) = CreatureSourceId::new(relationship.target_id.clone()) else {
                continue;
            };
            let target = occurrence_ids
                .get(&relationship.target_id)
                .map(|(target, _, _, _)| CreatureRelationshipTarget::Occurrence(target.clone()))
                .unwrap_or_else(|| CreatureRelationshipTarget::UnresolvedNestedSourceId(target_id));
            relationships.push(CreatureEntityRelationship {
                source: source.clone(),
                kind: match relationship.kind {
                    EmbeddedRelationshipKindSource::GrantedBy => {
                        CreatureEntityRelationshipKind::GrantedBy
                    }
                    EmbeddedRelationshipKindSource::ItemGrant => {
                        CreatureEntityRelationshipKind::ItemGrant
                    }
                    EmbeddedRelationshipKindSource::LinkedWeapon => {
                        CreatureEntityRelationshipKind::LinkedWeapon
                    }
                },
                target,
                source_path: relationship.source_path.clone(),
                contextual_label: presence(&relationship.contextual_label),
                lifecycle: presence(&relationship.lifecycle),
                execution: CreatureRelationshipExecution::ProvenanceOnly,
            });
        }
        if let NpcEmbeddedItemSource::SpellcastingEntry(entry) = &candidate.source
            && let SourcePresence::Value(slots) = &entry.slots
        {
            for slot in slots {
                let SourcePresence::Value(prepared) = &slot.prepared else {
                    continue;
                };
                for (ordinal, prepared) in prepared.iter().enumerate() {
                    let PreparedSlotSource::Spell {
                        id: SourcePresence::Value(id),
                        ..
                    } = prepared
                    else {
                        continue;
                    };
                    let Ok(target_id) = CreatureSourceId::new(id.clone()) else {
                        continue;
                    };
                    let target = occurrence_ids
                        .get(id)
                        .map(|(target, _, _, _)| {
                            CreatureRelationshipTarget::Occurrence(target.clone())
                        })
                        .unwrap_or_else(|| {
                            CreatureRelationshipTarget::UnresolvedNestedSourceId(target_id)
                        });
                    relationships.push(CreatureEntityRelationship {
                        source: source.clone(),
                        kind: CreatureEntityRelationshipKind::PreparedSpell,
                        target,
                        source_path: format!(
                            "$.system.slots.slot{}.prepared[{ordinal}].id",
                            slot.rank
                        ),
                        contextual_label: FactValue::Missing,
                        lifecycle: FactValue::Missing,
                        execution: CreatureRelationshipExecution::ProvenanceOnly,
                    });
                }
            }
        }
    }
    relationships
}

fn actor_spellcasting_context(
    source: &SourcePresence<super::dto::ActorSpellcastingSource>,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> FactValue<CreatureActorSpellcastingContext> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(source) => FactValue::Value(CreatureActorSpellcastingContext {
            rituals_dc: source_scalar("actor", &source.rituals_dc, diagnostics),
            unsupported_notes: unsupported_notes(
                "actor",
                &SourcePresence::Value(source.unsupported.clone()),
                diagnostics,
            ),
        }),
    }
}

fn resolve_verified_stable_locator(
    locator: &str,
    index: &RecordReferenceIndex,
) -> Option<RecordKey> {
    let parts = locator.split('.').collect::<Vec<_>>();
    let (pack, id) = match parts.as_slice() {
        ["Compendium", "pf2e", pack, "Item", id] => (*pack, *id),
        ["pf2e", pack, id] => (*pack, *id),
        _ => return None,
    };
    index
        .by_pack_id
        .get(&(pack.to_string(), id.to_string()))
        .cloned()
}

pub(crate) fn family(source: &NpcEmbeddedItemSource) -> CreatureEntityFamily {
    match source {
        NpcEmbeddedItemSource::Action(_) => CreatureEntityFamily::Action,
        NpcEmbeddedItemSource::Strike(_) => CreatureEntityFamily::Strike,
        NpcEmbeddedItemSource::SpellcastingEntry(_) => CreatureEntityFamily::SpellcastingEntry,
        NpcEmbeddedItemSource::Spell(_) => CreatureEntityFamily::Spell,
        NpcEmbeddedItemSource::Equipment(_) => CreatureEntityFamily::Equipment,
        NpcEmbeddedItemSource::Lore(_) => CreatureEntityFamily::Lore,
        NpcEmbeddedItemSource::Deferred(deferred) => match deferred.item_type {
            super::dto::ItemType::Affliction => CreatureEntityFamily::Affliction,
            super::dto::ItemType::Armor => CreatureEntityFamily::Armor,
            super::dto::ItemType::Backpack => CreatureEntityFamily::Backpack,
            super::dto::ItemType::Book => CreatureEntityFamily::Book,
            super::dto::ItemType::Condition => CreatureEntityFamily::Condition,
            super::dto::ItemType::Consumable => CreatureEntityFamily::Consumable,
            super::dto::ItemType::Effect => CreatureEntityFamily::Effect,
            super::dto::ItemType::Shield => CreatureEntityFamily::Shield,
            super::dto::ItemType::Treasure => CreatureEntityFamily::Treasure,
            super::dto::ItemType::Weapon => CreatureEntityFamily::Weapon,
            _ => CreatureEntityFamily::Unsupported,
        },
    }
}

fn occurrence_identity(
    owner: &RecordKey,
    family: CreatureEntityFamily,
    nested_id: &str,
    order: usize,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> (
    CreatureOccurrenceId,
    OccurrenceIdentityStability,
    FactValue<CreatureSourceId>,
) {
    match CreatureSourceId::new(nested_id.to_string()) {
        Ok(source_id) => (
            CreatureOccurrenceId::stable_nested(owner, family, &source_id),
            OccurrenceIdentityStability::StableNestedSourceId,
            FactValue::Value(source_id),
        ),
        Err(_) => {
            diagnostics.push(NpcEmbeddedDiagnostic::new(
                NpcEmbeddedDiagnosticKind::UnstableOwnerFamilyOrdinalIdentity,
                None,
                "$._id",
                "stable nested source id",
                "missing or invalid id",
                format!("owner={owner},family={},ordinal={order}", family.as_str()),
            ));
            (
                CreatureOccurrenceId::unstable_ordinal(owner, family, order),
                OccurrenceIdentityStability::UnstableOwnerFamilyOrdinal,
                FactValue::Missing,
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn actor_owned_target(
    owner: &RecordKey,
    family: CreatureEntityFamily,
    occurrence_id: &CreatureOccurrenceId,
    label: &str,
    source_identity: CreatureEntitySourceIdentity,
    entities: &mut Vec<CreatureEntity>,
    seen_entities: &mut BTreeSet<String>,
) -> CreatureEntityTarget {
    let id = CreatureEntityId::actor_owned(owner, family, occurrence_id);
    if seen_entities.insert(id.as_str().to_string()) {
        entities.push(CreatureEntity {
            id: id.clone(),
            family,
            label: label.to_string(),
            source_identity,
        });
    }
    CreatureEntityTarget::ActorOwned(id)
}

fn context(candidate: &NpcEmbeddedCandidate) -> CreatureOccurrenceContext {
    match &candidate.source {
        NpcEmbeddedItemSource::Action(action) => CreatureOccurrenceContext {
            group: presence(&action.category),
            contextual_label: FactValue::Value(candidate.label.clone()),
            ..CreatureOccurrenceContext::default()
        },
        NpcEmbeddedItemSource::Spell(spell) => CreatureOccurrenceContext {
            rank: rank(spell),
            location: presence(&spell.location),
            uses: presence_map(&spell.uses, use_limit),
            contextual_label: FactValue::Value(candidate.label.clone()),
            ..CreatureOccurrenceContext::default()
        },
        NpcEmbeddedItemSource::SpellcastingEntry(entry) => CreatureOccurrenceContext {
            rank: presence(&entry.auto_heighten_level),
            contextual_label: FactValue::Value(candidate.label.clone()),
            ..CreatureOccurrenceContext::default()
        },
        NpcEmbeddedItemSource::Equipment(equipment) => CreatureOccurrenceContext {
            uses: presence_map(&equipment.uses, use_limit),
            contextual_label: FactValue::Value(candidate.label.clone()),
            ..CreatureOccurrenceContext::default()
        },
        _ => CreatureOccurrenceContext {
            contextual_label: FactValue::Value(candidate.label.clone()),
            ..CreatureOccurrenceContext::default()
        },
    }
}

fn rank(spell: &SpellSource) -> FactValue<i64> {
    match &spell.heightened_level {
        SourcePresence::Value(value) => FactValue::Value(*value),
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Missing => presence(&spell.level),
    }
}

fn occurrence_deltas(candidate: &NpcEmbeddedCandidate) -> Vec<CreatureOccurrenceDelta> {
    let mut deltas = vec![delta(
        "label",
        FactValue::Value(CreatureDeltaValue::Text(candidate.label.clone())),
    )];
    if let NpcEmbeddedItemSource::Spell(spell) = &candidate.source {
        deltas.push(delta("rank", delta_integer(&rank(spell))));
        deltas.push(delta("location", delta_text(&presence(&spell.location))));
        deltas.push(delta(
            "uses",
            presence_map(&spell.uses, |uses| {
                CreatureDeltaValue::Uses(use_limit(uses))
            }),
        ));
    }
    if let NpcEmbeddedItemSource::Equipment(equipment) = &candidate.source {
        deltas.push(delta(
            "uses",
            presence_map(&equipment.uses, |uses| {
                CreatureDeltaValue::Uses(use_limit(uses))
            }),
        ));
    }
    deltas
}

fn delta(field_path: &str, source_state: FactValue<CreatureDeltaValue>) -> CreatureOccurrenceDelta {
    CreatureOccurrenceDelta {
        field_path: field_path.to_string(),
        local_value: source_state.clone(),
        source_state,
        canonical_value: FactValue::Missing,
        disposition: CreatureDeltaDisposition::ContextualValue,
    }
}

fn delta_integer(value: &FactValue<i64>) -> FactValue<CreatureDeltaValue> {
    fact_map(value, |value| CreatureDeltaValue::Integer(*value))
}

fn delta_text(value: &FactValue<String>) -> FactValue<CreatureDeltaValue> {
    fact_map(value, |value| CreatureDeltaValue::Text(value.clone()))
}

fn capability(
    candidate: &NpcEmbeddedCandidate,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> Option<CreatureCapability> {
    let mut capability = match &candidate.source {
        NpcEmbeddedItemSource::Action(action) => CreatureCapability::Action(action_capability(
            &candidate.nested_source_id,
            action,
            diagnostics,
        )),
        NpcEmbeddedItemSource::Strike(strike) => CreatureCapability::Strike(strike_capability(
            &candidate.nested_source_id,
            strike,
            diagnostics,
        )),
        NpcEmbeddedItemSource::SpellcastingEntry(entry) => CreatureCapability::SpellcastingEntry(
            entry_capability(&candidate.nested_source_id, entry, diagnostics),
        ),
        NpcEmbeddedItemSource::Spell(spell) => CreatureCapability::Spell(spell_capability(
            &candidate.nested_source_id,
            spell,
            diagnostics,
        )),
        NpcEmbeddedItemSource::Equipment(equipment) => CreatureCapability::Equipment(
            equipment_capability(&candidate.nested_source_id, equipment, diagnostics),
        ),
        NpcEmbeddedItemSource::Lore(lore) => CreatureCapability::Lore(CreatureLoreCapability {
            modifier: presence(&lore.modifier),
            unsupported_notes: common_unsupported_notes(
                &candidate.nested_source_id,
                &lore.common,
                diagnostics,
            ),
        }),
        NpcEmbeddedItemSource::Deferred(deferred) => {
            CreatureCapability::Unsupported(CreatureUnsupportedCapability {
                source_item_type: deferred.item_type.to_string(),
                source_slug: presence(&deferred.common.slug),
                traits: presence(&deferred.common.traits),
                unsupported_notes: common_unsupported_notes(
                    &candidate.nested_source_id,
                    &deferred.common,
                    diagnostics,
                ),
            })
        }
    };
    capability_unsupported_notes(&mut capability)
        .extend(unmodeled_local_notes(candidate, diagnostics));
    Some(capability)
}

fn action_capability(
    item_id: &str,
    source: &ActionSource,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> CreatureActionCapability {
    let mut rolls = Vec::new();
    if !matches!(source.bonus, SourcePresence::Missing) {
        rolls.push(CreatureRoll {
            id: "check".to_string(),
            label: "Check".to_string(),
            kind: CreatureRollKind::Check,
            value: presence(&source.bonus),
            ability: FactValue::Missing,
        });
    }
    if !matches!(source.dc, SourcePresence::Missing) {
        rolls.push(CreatureRoll {
            id: "dc".to_string(),
            label: "DC".to_string(),
            kind: CreatureRollKind::DifficultyClass,
            value: presence(&source.dc),
            ability: FactValue::Missing,
        });
    }
    CreatureActionCapability {
        category: presence(&source.category),
        traits: presence(&source.common.traits),
        action_cost: action_cost(&source.action_type, &source.actions),
        frequency: presence_map(&source.frequency, |frequency| {
            atlas_record::CreatureFrequency {
                maximum: presence(&frequency.maximum),
                period: presence(&frequency.period),
                serialized_value: presence(&frequency.value),
            }
        }),
        self_effect: presence(&source.self_effect),
        self_effect_label: presence(&source.self_effect_label),
        requirements: presence(&source.requirements),
        cost: presence(&source.cost),
        rolls,
        damage: presence_map(&source.damage, |value| damage(item_id, value, diagnostics)),
        unsupported_notes: common_unsupported_notes(item_id, &source.common, diagnostics),
    }
}

fn strike_capability(
    item_id: &str,
    source: &StrikeSource,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> CreatureStrikeCapability {
    let ability = strike_ability(&source.common.traits);
    let rolls = (!matches!(source.bonus, SourcePresence::Missing))
        .then(|| CreatureRoll {
            id: "attack".to_string(),
            label: "Attack".to_string(),
            kind: CreatureRollKind::Attack,
            value: presence(&source.bonus),
            ability: FactValue::Value(ability),
        })
        .into_iter()
        .collect();
    CreatureStrikeCapability {
        traits: presence(&source.common.traits),
        attack_effects: presence(&source.attack_effects),
        rolls,
        damage: presence_map(&source.damage, |value| damage(item_id, value, diagnostics)),
        action_cost: CreatureActionCost::Actions(1),
        unsupported_notes: common_unsupported_notes(item_id, &source.common, diagnostics),
    }
}

fn strike_ability(traits: &SourcePresence<Vec<String>>) -> ActivityRollAbility {
    let traits = traits.as_value().map(Vec::as_slice).unwrap_or_default();
    if traits
        .iter()
        .any(|trait_slug| trait_slug.starts_with("thrown-"))
    {
        return ActivityRollAbility::Strength;
    }
    if traits.iter().any(|trait_slug| {
        trait_slug == "ranged"
            || trait_slug.starts_with("range-")
            || trait_slug.starts_with("reload-")
    }) {
        return ActivityRollAbility::Dexterity;
    }
    ActivityRollAbility::Strength
}

fn entry_capability(
    item_id: &str,
    source: &SpellcastingEntrySource,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> CreatureSpellcastingEntryCapability {
    CreatureSpellcastingEntryCapability {
        preparation: presence_map(&source.preparation, |value| spell_preparation(value)),
        tradition: presence(&source.tradition),
        attack: presence(&source.attack),
        dc: presence(&source.dc),
        slots: presence_map(&source.slots, |slots| {
            slots
                .iter()
                .map(|slot| CreatureSpellSlot {
                    rank: slot.rank,
                    maximum: source_scalar(item_id, &slot.maximum, diagnostics),
                    serialized_value: source_scalar(item_id, &slot.value, diagnostics),
                    prepared: prepared_slots(item_id, &slot.prepared, diagnostics),
                })
                .collect()
        }),
        unsupported_notes: common_unsupported_notes(item_id, &source.common, diagnostics),
    }
}

fn spell_capability(
    item_id: &str,
    source: &SpellSource,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> CreatureSpellCapability {
    let mut notes = common_unsupported_notes(item_id, &source.common, diagnostics);
    notes.extend(unsupported_notes(item_id, &source.overlays, diagnostics));
    CreatureSpellCapability {
        traits: presence(&source.common.traits),
        base_rank: presence(&source.level),
        signature: presence(&source.signature),
        traditions: presence(&source.traditions),
        requirements: presence(&source.requirements),
        cost: presence(&source.cost),
        counteraction: presence(&source.counteraction),
        ritual: presence_map(&source.ritual, |ritual| CreatureRitualContext {
            primary_check: presence(&ritual.primary_check),
            secondary_casters: source_scalar_without_diagnostic(&ritual.secondary_casters),
            secondary_checks: presence(&ritual.secondary_checks),
        }),
        target: presence(&source.target),
        area: presence_map(&source.area, |area| CreatureSpellArea {
            area_type: presence(&area.area_type),
            value: presence(&area.value),
        }),
        range: presence(&source.range),
        time: presence(&source.time),
        duration: presence_map(&source.duration, |duration| CreatureSpellDuration {
            value: presence(&duration.value),
            sustained: presence(&duration.sustained),
        }),
        defense: presence_map(&source.defense, spell_defense),
        damage: presence_map(&source.damage, |value| damage(item_id, value, diagnostics)),
        action_cost: spell_action_cost(&source.time),
        unsupported_notes: notes,
    }
}

fn equipment_capability(
    item_id: &str,
    source: &EquipmentSource,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> CreatureEquipmentCapability {
    CreatureEquipmentCapability {
        traits: presence(&source.common.traits),
        level: presence(&source.level),
        usage: presence(&source.usage),
        quantity: presence(&source.quantity),
        uses: presence_map(&source.uses, use_limit),
        unsupported_notes: common_unsupported_notes(item_id, &source.common, diagnostics),
    }
}

fn action_cost(
    action_type: &SourcePresence<String>,
    actions: &SourcePresence<i64>,
) -> CreatureActionCost {
    match action_type.as_value().map(String::as_str) {
        Some("passive") => CreatureActionCost::Passive,
        Some("reaction") => CreatureActionCost::Reaction,
        Some("free") => CreatureActionCost::FreeAction,
        Some("action") => match actions {
            SourcePresence::Value(value) => u8::try_from(*value)
                .map(CreatureActionCost::Actions)
                .unwrap_or_else(|_| CreatureActionCost::Unsupported(unsupported_number(*value))),
            SourcePresence::Missing => CreatureActionCost::Unsupported(unsupported_string(
                "action count missing for action activity",
            )),
            SourcePresence::Null => CreatureActionCost::Unsupported(unsupported_string(
                "action count null for action activity",
            )),
        },
        Some(value) => CreatureActionCost::Unsupported(unsupported_string(value)),
        None => CreatureActionCost::Unsupported(unsupported_string(match action_type {
            SourcePresence::Missing => "action type missing",
            SourcePresence::Null => "action type null",
            SourcePresence::Value(_) => "unsupported action type",
        })),
    }
}

fn spell_action_cost(time: &SourcePresence<String>) -> CreatureActionCost {
    match time {
        SourcePresence::Value(value) => match value.as_str() {
            "reaction" => CreatureActionCost::Reaction,
            "free" | "free action" => CreatureActionCost::FreeAction,
            _ => value
                .parse::<u8>()
                .map(CreatureActionCost::Actions)
                .unwrap_or_else(|_| CreatureActionCost::Time(value.clone())),
        },
        SourcePresence::Missing | SourcePresence::Null => {
            CreatureActionCost::Unsupported(unsupported_string("missing or null spell time"))
        }
    }
}

fn spell_preparation(value: &str) -> CreatureSpellPreparation {
    match value {
        "prepared" => CreatureSpellPreparation::Prepared,
        "spontaneous" => CreatureSpellPreparation::Spontaneous,
        "focus" => CreatureSpellPreparation::Focus,
        "innate" => CreatureSpellPreparation::Innate,
        "ritual" => CreatureSpellPreparation::Ritual,
        other => CreatureSpellPreparation::Unsupported(unsupported_string(other)),
    }
}

fn spell_defense(source: &SpellDefenseSource) -> CreatureSpellDefense {
    CreatureSpellDefense {
        save: presence_map(&source.statistic, |value| match value.as_str() {
            "fortitude" => CreatureSpellSave::Fortitude,
            "reflex" => CreatureSpellSave::Reflex,
            "will" => CreatureSpellSave::Will,
            other => CreatureSpellSave::Unsupported(unsupported_string(other)),
        }),
        basic: presence(&source.basic),
    }
}

fn damage(
    item_id: &str,
    source: &[DamageSource],
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> Vec<CreatureDamage> {
    source
        .iter()
        .map(|source| CreatureDamage {
            id: source.id.clone(),
            formula: presence(&source.formula),
            damage_type: presence(&source.damage_type),
            category: presence(&source.category),
            kinds: presence_map(&source.kinds, |kinds| {
                kinds
                    .iter()
                    .map(|kind| match kind.as_str() {
                        "damage" => CreatureDamageKind::Damage,
                        "healing" => CreatureDamageKind::Healing,
                        value => CreatureDamageKind::Unsupported(unsupported_string(value)),
                    })
                    .collect()
            }),
            apply_modifier: source_scalar(item_id, &source.apply_modifier, diagnostics),
        })
        .collect()
}

fn source_scalar<T: Clone>(
    item_id: &str,
    source: &SourcePresence<EmbeddedSourceScalar<T>>,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> FactValue<CreatureSourceScalar<T>> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(EmbeddedSourceScalar::Value(value)) => {
            FactValue::Value(CreatureSourceScalar::Value(value.clone()))
        }
        SourcePresence::Value(EmbeddedSourceScalar::Unsupported(drift)) => {
            diagnostics.push(source_type_drift_diagnostic(item_id, drift));
            FactValue::Value(CreatureSourceScalar::Unsupported(UnsupportedSourceValue {
                shape: unsupported_shape(&drift.observed_shape),
                value: drift.value.clone(),
                reason: UnsupportedSourceReason::AmbiguousLegacyShape,
            }))
        }
    }
}

fn source_type_drift_diagnostic(item_id: &str, drift: &SourceTypeDrift) -> NpcEmbeddedDiagnostic {
    NpcEmbeddedDiagnostic::new(
        NpcEmbeddedDiagnosticKind::SourceTypeDrift,
        Some(item_id.to_string()),
        drift.source_path.clone(),
        drift.expected_shape.clone(),
        drift.observed_shape.clone(),
        drift.value.clone(),
    )
}

fn prepared_slots(
    _item_id: &str,
    source: &SourcePresence<Vec<PreparedSlotSource>>,
    _diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> FactValue<Vec<CreaturePreparedSpellSlot>> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(values) => FactValue::Value(
            values
                .iter()
                .enumerate()
                .map(|(authored_order, value)| match value {
                    PreparedSlotSource::Unsupported(drift) => {
                        CreaturePreparedSpellSlot::Unsupported(UnsupportedSourceValue {
                            shape: unsupported_shape(&drift.observed_shape),
                            value: drift.value.clone(),
                            reason: UnsupportedSourceReason::AmbiguousLegacyShape,
                        })
                    }
                    PreparedSlotSource::Spell {
                        id,
                        name,
                        expended,
                        prepared,
                    } => match id {
                        SourcePresence::Value(id) => match CreatureSourceId::new(id.clone()) {
                            Ok(id) => CreaturePreparedSpellSlot::Spell {
                                id: FactValue::Value(id),
                                name: presence(name),
                                expended: presence(expended),
                                prepared: presence(prepared),
                                authored_order: authored_order as u32,
                            },
                            Err(_) => {
                                CreaturePreparedSpellSlot::Unsupported(unsupported_string(id))
                            }
                        },
                        SourcePresence::Missing => CreaturePreparedSpellSlot::Spell {
                            id: FactValue::Missing,
                            name: presence(name),
                            expended: presence(expended),
                            prepared: presence(prepared),
                            authored_order: authored_order as u32,
                        },
                        SourcePresence::Null => CreaturePreparedSpellSlot::Spell {
                            id: FactValue::Null,
                            name: presence(name),
                            expended: presence(expended),
                            prepared: presence(prepared),
                            authored_order: authored_order as u32,
                        },
                    },
                })
                .collect(),
        ),
    }
}

fn source_scalar_without_diagnostic<T: Clone>(
    source: &SourcePresence<EmbeddedSourceScalar<T>>,
) -> FactValue<CreatureSourceScalar<T>> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(EmbeddedSourceScalar::Value(value)) => {
            FactValue::Value(CreatureSourceScalar::Value(value.clone()))
        }
        SourcePresence::Value(EmbeddedSourceScalar::Unsupported(value)) => {
            FactValue::Value(CreatureSourceScalar::Unsupported(UnsupportedSourceValue {
                shape: unsupported_shape(&value.observed_shape),
                value: value.value.clone(),
                reason: UnsupportedSourceReason::OpenVocabulary,
            }))
        }
    }
}

fn unsupported_shape(shape: &str) -> UnsupportedSourceShape {
    match shape {
        "string" => UnsupportedSourceShape::String,
        "integer" | "number" => UnsupportedSourceShape::Number,
        "boolean" => UnsupportedSourceShape::Boolean,
        "array" => UnsupportedSourceShape::Array,
        "null" => UnsupportedSourceShape::Null,
        _ => UnsupportedSourceShape::Object,
    }
}

fn unsupported_notes(
    item_id: &str,
    summaries: &SourcePresence<Vec<ValueSummary>>,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> Vec<UnsupportedMechanicNote> {
    let Some(summaries) = summaries.as_value() else {
        return Vec::new();
    };
    summaries
        .iter()
        .map(|summary| {
            diagnostics.push(NpcEmbeddedDiagnostic::new(
                NpcEmbeddedDiagnosticKind::UnsupportedMechanic,
                Some(item_id.to_string()),
                summary.source_path.clone(),
                "supported B4 mechanic",
                summary.shape.clone(),
                summary.value.clone(),
            ));
            UnsupportedMechanicNote {
                source_path: summary.source_path.clone(),
                value: UnsupportedSourceValue {
                    shape: unsupported_shape(&summary.shape),
                    value: summary.value.clone(),
                    reason: UnsupportedSourceReason::OpenVocabulary,
                },
            }
        })
        .collect()
}

fn common_unsupported_notes(
    item_id: &str,
    common: &super::dto::EmbeddedCommonSource,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> Vec<UnsupportedMechanicNote> {
    unsupported_notes(item_id, &common.rules, diagnostics)
}

fn unmodeled_local_notes(
    candidate: &NpcEmbeddedCandidate,
    diagnostics: &mut Vec<NpcEmbeddedDiagnostic>,
) -> Vec<UnsupportedMechanicNote> {
    let common = match &candidate.source {
        NpcEmbeddedItemSource::Action(value) => &value.common,
        NpcEmbeddedItemSource::Strike(value) => &value.common,
        NpcEmbeddedItemSource::SpellcastingEntry(value) => &value.common,
        NpcEmbeddedItemSource::Spell(value) => &value.common,
        NpcEmbeddedItemSource::Equipment(value) => &value.common,
        NpcEmbeddedItemSource::Lore(value) => &value.common,
        NpcEmbeddedItemSource::Deferred(value) => &value.common,
    };
    common
        .local_unsupported
        .iter()
        .filter(|summary| {
            !modeled_local_path(&candidate.source, &summary.source_path)
                && (!diagnosed_capability_retention_path(&candidate.source, &summary.source_path)
                    || !empty_local_scaffolding(summary))
        })
        .chain(candidate.relationship_unsupported.iter())
        .map(|summary| {
            if diagnosed_capability_retention_path(&candidate.source, &summary.source_path)
                && !empty_local_scaffolding(summary)
            {
                diagnostics.push(NpcEmbeddedDiagnostic::new(
                    NpcEmbeddedDiagnosticKind::UnsupportedMechanic,
                    Some(candidate.nested_source_id.clone()),
                    summary.source_path.clone(),
                    "modeled embedded capability leaf or exact typed unsupported retention",
                    summary.shape.clone(),
                    summary.value.clone(),
                ));
            }
            summary_note(summary)
        })
        .collect()
}

fn empty_local_scaffolding(summary: &ValueSummary) -> bool {
    matches!(
        (summary.shape.as_str(), summary.value.as_str()),
        ("string", "\"\"") | ("array", "[]") | ("object", "{}")
    )
}

fn modeled_local_path(source: &NpcEmbeddedItemSource, path: &str) -> bool {
    let Some((_, relative)) = path.split_once(".system.") else {
        return false;
    };
    if diagnosed_capability_retention_path(source, path) {
        return false;
    }
    if relative.starts_with("spell.system.description.")
        || relative == "spell.flags.core.sourceId"
        || relative == "spell._stats.compendiumSource"
        || relative == "spell.system.counteraction"
        || relative == "spell.system.traits.traditions[]"
        || relative == "publication.remaster"
        || relative == "traits.value[]"
    {
        return true;
    }
    let root = relative.split(['.', '[']).next().unwrap_or(relative);
    match source {
        NpcEmbeddedItemSource::Action(_) => matches!(
            root,
            "actionType"
                | "actions"
                | "frequency"
                | "selfEffect"
                | "requirements"
                | "cost"
                | "bonus"
                | "dc"
                | "damageRolls"
                | "damage"
                | "category"
        ),
        NpcEmbeddedItemSource::Strike(_) => {
            matches!(root, "bonus" | "attackEffects" | "damageRolls")
        }
        NpcEmbeddedItemSource::SpellcastingEntry(_) => matches!(
            root,
            "prepared" | "tradition" | "spelldc" | "slots" | "autoHeightenLevel"
        ),
        NpcEmbeddedItemSource::Spell(_) => {
            matches!(
                root,
                "level"
                    | "location"
                    | "requirements"
                    | "cost"
                    | "counteraction"
                    | "ritual"
                    | "target"
                    | "area"
                    | "range"
                    | "time"
                    | "duration"
                    | "defense"
                    | "damage"
                    | "overlays"
            ) || relative == "traits.traditions[]"
        }
        NpcEmbeddedItemSource::Equipment(_) => {
            matches!(root, "level" | "usage" | "quantity" | "uses")
        }
        NpcEmbeddedItemSource::Lore(_) => root == "mod",
        NpcEmbeddedItemSource::Deferred(_) => false,
    }
}

fn diagnosed_capability_retention_path(source: &NpcEmbeddedItemSource, path: &str) -> bool {
    let Some((_, relative)) = path.split_once(".system.") else {
        return false;
    };
    match source {
        NpcEmbeddedItemSource::Strike(_) => relative == "attackEffects.custom",
        NpcEmbeddedItemSource::Spell(_) => {
            matches!(
                relative,
                "area.details" | "defense.passive.statistic" | "location.autoHeightenLevel"
            ) || unsupported_damage_material_path(relative)
        }
        NpcEmbeddedItemSource::SpellcastingEntry(_) => matches!(
            relative,
            "prepared.flexible"
                | "prepared.label"
                | "prepared.type"
                | "prepared.validItems"
                | "spelldc.item"
                | "spelldc.label"
                | "spelldc.mod"
                | "spelldc.type"
        ),
        _ => false,
    }
}

fn unsupported_damage_material_path(relative: &str) -> bool {
    relative
        .strip_prefix("damage.")
        .and_then(|entry| entry.split_once('.'))
        .is_some_and(|(_, leaf)| leaf == "materials[]")
}

fn summary_note(summary: &ValueSummary) -> UnsupportedMechanicNote {
    UnsupportedMechanicNote {
        source_path: summary.source_path.clone(),
        value: UnsupportedSourceValue {
            shape: unsupported_shape(&summary.shape),
            value: summary.value.clone(),
            reason: UnsupportedSourceReason::OpenVocabulary,
        },
    }
}

fn capability_unsupported_notes(
    capability: &mut CreatureCapability,
) -> &mut Vec<UnsupportedMechanicNote> {
    match capability {
        CreatureCapability::Strike(value) => &mut value.unsupported_notes,
        CreatureCapability::Action(value) => &mut value.unsupported_notes,
        CreatureCapability::SpellcastingEntry(value) => &mut value.unsupported_notes,
        CreatureCapability::Spell(value) => &mut value.unsupported_notes,
        CreatureCapability::Equipment(value) => &mut value.unsupported_notes,
        CreatureCapability::Lore(value) => &mut value.unsupported_notes,
        CreatureCapability::Unsupported(value) => &mut value.unsupported_notes,
    }
}

fn capability_unsupported_notes_ref(capability: &CreatureCapability) -> &[UnsupportedMechanicNote] {
    match capability {
        CreatureCapability::Strike(value) => &value.unsupported_notes,
        CreatureCapability::Action(value) => &value.unsupported_notes,
        CreatureCapability::SpellcastingEntry(value) => &value.unsupported_notes,
        CreatureCapability::Spell(value) => &value.unsupported_notes,
        CreatureCapability::Equipment(value) => &value.unsupported_notes,
        CreatureCapability::Lore(value) => &value.unsupported_notes,
        CreatureCapability::Unsupported(value) => &value.unsupported_notes,
    }
}

pub(crate) fn retained_capability_survival(
    conversion: &NpcEmbeddedConversion,
) -> BTreeMap<&'static str, usize> {
    let FactValue::Value(embedded) = &conversion.embedded else {
        return BTreeMap::new();
    };
    let mut counts = BTreeMap::new();
    for occurrence in &embedded.occurrences {
        for note in capability_unsupported_notes_ref(&occurrence.capability) {
            if let Some(path) = retained_occurrence_capability_family(occurrence.family, note) {
                *counts.entry(path).or_insert(0) += 1;
            }
        }
    }
    counts
}

fn retained_capability_family(path: &str) -> Option<&'static str> {
    let (_, relative) = path.split_once(".system.")?;
    match relative {
        "attackEffects.custom" => Some("$.items[].system.attackEffects.custom"),
        "area.details" => Some("$.items[].system.area.details"),
        "defense.passive.statistic" => Some("$.items[].system.defense.passive.statistic"),
        "location.autoHeightenLevel" => Some("$.items[].system.location.autoHeightenLevel"),
        "prepared.flexible" => Some("$.items[].system.prepared.flexible"),
        "prepared.label" => Some("$.items[].system.prepared.label"),
        "prepared.type" => Some("$.items[].system.prepared.type"),
        "prepared.validItems" => Some("$.items[].system.prepared.validItems"),
        "spelldc.item" => Some("$.items[].system.spelldc.item"),
        "spelldc.label" => Some("$.items[].system.spelldc.label"),
        "spelldc.mod" => Some("$.items[].system.spelldc.mod"),
        "spelldc.type" => Some("$.items[].system.spelldc.type"),
        value if value.starts_with("damage.") && value.ends_with(".materials[]") => {
            Some("$.items[].system.damage.*.materials[]")
        }
        _ => None,
    }
}

fn retained_occurrence_capability_family(
    family: CreatureEntityFamily,
    note: &UnsupportedMechanicNote,
) -> Option<&'static str> {
    if matches!(
        (&note.value.shape, note.value.value.as_str()),
        (UnsupportedSourceShape::String, "\"\"")
            | (UnsupportedSourceShape::Array, "[]")
            | (UnsupportedSourceShape::Object, "{}")
    ) {
        return None;
    }
    let retained = retained_capability_family(&note.source_path)?;
    match (family, retained) {
        (CreatureEntityFamily::Strike, "$.items[].system.attackEffects.custom")
        | (
            CreatureEntityFamily::Spell,
            "$.items[].system.area.details"
            | "$.items[].system.damage.*.materials[]"
            | "$.items[].system.defense.passive.statistic"
            | "$.items[].system.location.autoHeightenLevel",
        )
        | (
            CreatureEntityFamily::SpellcastingEntry,
            "$.items[].system.prepared.flexible"
            | "$.items[].system.prepared.label"
            | "$.items[].system.prepared.type"
            | "$.items[].system.prepared.validItems"
            | "$.items[].system.spelldc.item"
            | "$.items[].system.spelldc.label"
            | "$.items[].system.spelldc.mod"
            | "$.items[].system.spelldc.type",
        ) => Some(retained),
        _ => None,
    }
}

fn use_limit(source: &UseLimitSource) -> CreatureUseLimit {
    CreatureUseLimit {
        maximum: presence(&source.maximum),
        serialized_value: presence(&source.value),
    }
}

fn unsupported_string(value: &str) -> UnsupportedSourceValue {
    UnsupportedSourceValue {
        shape: UnsupportedSourceShape::String,
        value: value.to_string(),
        reason: UnsupportedSourceReason::OpenVocabulary,
    }
}

fn unsupported_number(value: i64) -> UnsupportedSourceValue {
    UnsupportedSourceValue {
        shape: UnsupportedSourceShape::Number,
        value: value.to_string(),
        reason: UnsupportedSourceReason::OpenVocabulary,
    }
}

fn presence<T: Clone>(source: &SourcePresence<T>) -> FactValue<T> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(value) => FactValue::Value(value.clone()),
    }
}

fn presence_map<T, U>(source: &SourcePresence<T>, map: impl FnOnce(&T) -> U) -> FactValue<U> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(value) => FactValue::Value(map(value)),
    }
}

fn fact_map<T, U>(source: &FactValue<T>, map: impl FnOnce(&T) -> U) -> FactValue<U> {
    match source {
        FactValue::Missing => FactValue::Missing,
        FactValue::Null => FactValue::Null,
        FactValue::Value(value) => FactValue::Value(map(value)),
    }
}

fn legacy_projection(
    embedded: &CreatureEmbeddedEntities,
) -> (Vec<SpellcastingEntryMechanics>, Vec<MechanicActivity>) {
    let entries = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| {
            let CreatureCapability::SpellcastingEntry(entry) = &occurrence.capability else {
                return None;
            };
            Some(SpellcastingEntryMechanics {
                entry_id: occurrence_source_id(embedded, occurrence),
                label: context_label(occurrence),
                preparation: match entry.preparation.as_value() {
                    Some(CreatureSpellPreparation::Prepared) => SpellcastingPreparation::Prepared,
                    Some(CreatureSpellPreparation::Spontaneous) => {
                        SpellcastingPreparation::Spontaneous
                    }
                    Some(CreatureSpellPreparation::Focus) => SpellcastingPreparation::Focus,
                    Some(CreatureSpellPreparation::Innate) => SpellcastingPreparation::Innate,
                    Some(CreatureSpellPreparation::Ritual) => {
                        SpellcastingPreparation::Other("ritual".to_string())
                    }
                    Some(CreatureSpellPreparation::Unsupported(value)) => {
                        SpellcastingPreparation::Other(value.value.clone())
                    }
                    None => SpellcastingPreparation::Other("unknown".to_string()),
                },
                spell_attack: entry.attack.as_value().copied(),
                spell_dc: entry.dc.as_value().copied(),
            })
        })
        .collect::<Vec<_>>();
    let activities = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| legacy_activity(occurrence, embedded, &entries))
        .collect();
    (entries, activities)
}

fn legacy_activity(
    occurrence: &CreatureEntityOccurrence,
    embedded: &CreatureEmbeddedEntities,
    entries: &[SpellcastingEntryMechanics],
) -> Option<MechanicActivity> {
    let (kind, traits, usage, rolls, damage) = match &occurrence.capability {
        CreatureCapability::Strike(strike) => (
            MechanicActivityKind::Strike,
            fact_value_or_default(&strike.traits),
            MechanicActivityUsage::Unlimited,
            legacy_rolls(&strike.rolls),
            legacy_damage(
                &strike.damage,
                strike
                    .rolls
                    .first()
                    .and_then(|roll| roll.ability.as_value().copied()),
            ),
        ),
        CreatureCapability::Action(action) => (
            MechanicActivityKind::Other,
            fact_value_or_default(&action.traits),
            if action.frequency.as_value().is_some() {
                MechanicActivityUsage::Limited
            } else {
                MechanicActivityUsage::Ambiguous
            },
            legacy_rolls(&action.rolls),
            legacy_damage(&action.damage, None),
        ),
        CreatureCapability::Spell(spell) => {
            let mut rolls = Vec::new();
            let mut usage = if spell
                .traits
                .as_value()
                .is_some_and(|traits| traits.iter().any(|value| value == "cantrip"))
            {
                MechanicActivityUsage::Unlimited
            } else if occurrence.context.uses.as_value().is_some() {
                MechanicActivityUsage::Limited
            } else {
                MechanicActivityUsage::Ambiguous
            };
            if let CreatureOccurrenceParent::SpellcastingEntry(entry_occurrence) =
                &occurrence.parent
                && let Some(entry) = embedded
                    .occurrences
                    .iter()
                    .find(|candidate| &candidate.id == entry_occurrence)
                && let Some(entry) = entries
                    .iter()
                    .find(|candidate| candidate.entry_id == occurrence_source_id(embedded, entry))
            {
                if matches!(
                    entry.preparation,
                    SpellcastingPreparation::Prepared
                        | SpellcastingPreparation::Spontaneous
                        | SpellcastingPreparation::Focus
                ) {
                    usage = MechanicActivityUsage::Limited;
                }
                if let Some(value) = entry.spell_attack {
                    rolls.push(ActivityRoll {
                        roll_id: "spell.attack".to_string(),
                        label: "Spell Attack".to_string(),
                        base_value: value,
                        surface: ActivityRollSurface::AttackRoll,
                        ability: None,
                    });
                }
                if let Some(value) = entry.spell_dc {
                    rolls.push(ActivityRoll {
                        roll_id: "spell.dc".to_string(),
                        label: "Spell DC".to_string(),
                        base_value: value,
                        surface: ActivityRollSurface::Dc,
                        ability: None,
                    });
                }
            }
            (
                MechanicActivityKind::Spell,
                fact_value_or_default(&spell.traits),
                usage,
                rolls,
                legacy_damage(&spell.damage, None),
            )
        }
        CreatureCapability::SpellcastingEntry(_)
        | CreatureCapability::Equipment(_)
        | CreatureCapability::Lore(_)
        | CreatureCapability::Unsupported(_) => return None,
    };
    Some(MechanicActivity {
        activity_id: occurrence_source_id(embedded, occurrence),
        label: context_label(occurrence),
        kind,
        traits,
        compendium_source: None,
        usage,
        rolls,
        damage,
        modes: Vec::new(),
    })
}

fn occurrence_source_id(
    embedded: &CreatureEmbeddedEntities,
    occurrence: &CreatureEntityOccurrence,
) -> String {
    if let CreatureEntityTarget::ActorOwned(id) = &occurrence.target
        && let Some(source_id) = embedded
            .entities
            .iter()
            .find(|entity| &entity.id == id)
            .and_then(|entity| entity.source_identity.nested_source_id.as_value())
    {
        return source_id.as_str().to_string();
    }
    occurrence
        .id
        .as_str()
        .rsplit(':')
        .next()
        .unwrap_or(occurrence.id.as_str())
        .to_string()
}

fn context_label(occurrence: &CreatureEntityOccurrence) -> String {
    occurrence
        .context
        .contextual_label
        .as_value()
        .cloned()
        .unwrap_or_else(|| occurrence.id.as_str().to_string())
}

fn legacy_rolls(rolls: &[CreatureRoll]) -> Vec<ActivityRoll> {
    rolls
        .iter()
        .filter_map(|roll| {
            Some(ActivityRoll {
                roll_id: roll.id.clone(),
                label: roll.label.clone(),
                base_value: *roll.value.as_value()?,
                surface: match roll.kind {
                    CreatureRollKind::DifficultyClass => ActivityRollSurface::Dc,
                    CreatureRollKind::Attack | CreatureRollKind::Check => {
                        ActivityRollSurface::AttackRoll
                    }
                },
                ability: roll.ability.as_value().copied(),
            })
        })
        .collect()
}

fn legacy_damage(
    source: &FactValue<Vec<CreatureDamage>>,
    ability: Option<ActivityRollAbility>,
) -> Vec<DamageExpression> {
    source
        .as_value()
        .into_iter()
        .flatten()
        .filter_map(|damage| {
            Some(DamageExpression {
                damage_id: damage.id.clone(),
                label: damage.category.as_value().cloned(),
                formula: damage.formula.as_value()?.clone(),
                damage_type: damage.damage_type.as_value().cloned(),
                effect_kind: match damage.kinds.as_value() {
                    Some(kinds)
                        if kinds.contains(&CreatureDamageKind::Damage)
                            && kinds.contains(&CreatureDamageKind::Healing) =>
                    {
                        DamageEffectKind::DamageOrHealing
                    }
                    Some(kinds) if kinds.contains(&CreatureDamageKind::Healing) => {
                        DamageEffectKind::Healing
                    }
                    Some(kinds) if kinds.contains(&CreatureDamageKind::Damage) => {
                        DamageEffectKind::Damage
                    }
                    _ => DamageEffectKind::Unknown,
                },
                ability,
            })
        })
        .collect()
}

fn fact_value_or_default<T: Clone>(value: &FactValue<Vec<T>>) -> Vec<T> {
    value.as_value().cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::Path};

    use atlas_domain::{PackName, RecordId, RecordKey};
    use atlas_record::{
        CreatureCapability, CreatureEntityFamily, CreatureEntityRelationshipKind,
        CreatureEntityTarget, CreaturePreparedSpellSlot, CreatureRelationshipExecution,
        CreatureRelationshipTarget, CreatureSourceScalar, FactValue, RecordBody,
        UnsupportedMechanicNote, UnsupportedSourceShape,
    };
    use serde_json::{Value, json};

    use super::{
        NpcEmbeddedDiagnosticDisposition, NpcEmbeddedDiagnosticKind, NpcEmbeddedDiagnosticOwner,
        capability_unsupported_notes_ref, convert_npc_embedded_entities,
        retained_capability_family, retained_occurrence_capability_family,
    };
    use crate::source::dto::{
        NpcEmbeddedItemSource, SourceIdentity, SourcePresence, parse_npc_source,
        pinned_source_version_metadata,
    };

    #[test]
    fn stable_target_reuse_repetition_reordering_and_fallback_are_explicit() {
        let source = parse_npc_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("bestiary:actor", "packs/bestiary/actor.json"),
            json!({
                "_id": "actor", "name": "Actor", "type": "npc", "system": {},
                "items": [
                    spell("spell-a", "Fireball", "Compendium.pf2e.spells-srd.Item.fireball", "entry", 3),
                    spell("spell-b", "Fireball", "Compendium.pf2e.spells-srd.Item.fireball", "entry", 5),
                    spell("spell-c", "Unknown", "Compendium.pf2e.spells-srd.Item.missing", "entry", 2)
                ]
            }),
        )
        .expect("source");
        let mut candidates = super::collect_npc_embedded_candidates(&source);
        let SourcePresence::Value(items) = &mut candidates.items else {
            panic!("items")
        };
        let NpcEmbeddedItemSource::Spell(second_spell) = &mut items[1].source else {
            panic!("second spell")
        };
        second_spell.location = SourcePresence::Null;
        let owner = key("bestiary", "actor");
        let fireball = key("spells-srd", "fireball");
        let resolve = |locator: &str| locator.ends_with(".fireball").then(|| fireball.clone());
        let first = convert_npc_embedded_entities(owner.clone(), &candidates, resolve);
        assert!(first.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == NpcEmbeddedDiagnosticKind::UnresolvedStableSourceIdentity
        }));
        let FactValue::Value(first) = first.embedded else {
            panic!("embedded")
        };
        assert_eq!(first.occurrences.len(), 3);
        assert_eq!(
            first.occurrences[0].target,
            CreatureEntityTarget::CanonicalRecord(fireball.clone())
        );
        assert_eq!(
            first.occurrences[1].target,
            CreatureEntityTarget::CanonicalRecord(fireball)
        );
        assert_ne!(first.occurrences[0].id, first.occurrences[1].id);
        assert!(matches!(
            first.occurrences[2].target,
            CreatureEntityTarget::ActorOwned(_)
        ));
        assert_eq!(first.entities.len(), 1);
        assert!(first.occurrences[0].deltas.iter().any(|delta| {
            delta.field_path == "location" && matches!(delta.source_state, FactValue::Value(_))
        }));
        assert!(first.occurrences[1].deltas.iter().any(|delta| {
            delta.field_path == "location" && delta.source_state == FactValue::Null
        }));

        let mut reordered = candidates.clone();
        let SourcePresence::Value(items) = &mut reordered.items else {
            panic!("items")
        };
        items.swap(0, 1);
        let second = convert_npc_embedded_entities(owner, &reordered, |locator| {
            locator
                .ends_with(".fireball")
                .then(|| key("spells-srd", "fireball"))
        });
        let FactValue::Value(second) = second.embedded else {
            panic!("embedded")
        };
        assert_eq!(first.occurrences[0].id, second.occurrences[1].id);
        assert_eq!(first.occurrences[1].id, second.occurrences[0].id);
    }

    #[test]
    fn typed_capabilities_preserve_action_economy_limits_rolls_slots_and_lore() {
        let source = parse_npc_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("bestiary:typed", "packs/bestiary/typed.json"),
            json!({
                "_id": "typed", "name": "Typed", "type": "npc", "system": {},
                "items": [
                    {
                        "_id": "action", "name": "Pulse", "type": "action",
                        "system": {
                            "actionType": {"value": "action"}, "actions": {"value": 2},
                            "frequency": {"max": 1, "per": "day"},
                            "bonus": {"value": 17}, "dc": {"value": 26},
                            "damageRolls": {"pulse": {"damage": "2d6", "damageType": "mental", "applyMod": true}},
                            "rules": [{"key": "UnsupportedFixture", "value": 1}]
                        }
                    },
                    {
                        "_id": "strike", "name": "Bolt", "type": "melee",
                        "system": {
                            "bonus": {"value": 19}, "traits": {"value": ["range-60"]},
                            "damageRolls": {"bolt": {"damage": "2d8", "damageType": "electricity"}}
                        }
                    },
                    {
                        "_id": "entry", "name": "Innate Spells", "type": "spellcastingEntry",
                        "system": {
                            "prepared": {"value": "innate"}, "tradition": {"value": "occult"},
                            "spelldc": {"value": 18, "dc": 27},
                            "autoHeightenLevel": {"value": 7},
                            "slots": {"slot4": {"max": 2, "value": 1}}
                        }
                    },
                    {
                        "_id": "spell", "name": "Reactive Spell", "type": "spell",
                        "system": {
                            "level": {"value": 4},
                            "location": {"value": "entry", "heightenedLevel": 6, "uses": {"max": 2, "value": 1}},
                            "time": {"value": "reaction"},
                            "defense": {"save": {"statistic": "custom-save", "basic": false}},
                            "damage": {}
                        }
                    },
                    {
                        "_id": "item-a", "name": "Charm", "type": "equipment",
                        "_stats": {"compendiumSource": "Compendium.pf2e.equipment-srd.Item.charm"},
                        "system": {"level": {"value": 5}, "quantity": 1, "uses": {"max": 3, "value": 2}}
                    },
                    {
                        "_id": "item-b", "name": "Charm", "type": "equipment",
                        "_stats": {"compendiumSource": "Compendium.pf2e.equipment-srd.Item.charm"},
                        "system": {"level": {"value": 5}, "quantity": 1, "uses": {"max": 3, "value": 1}}
                    },
                    {"_id": "lore", "name": "Night Hag Lore", "type": "lore", "system": {"mod": {"value": 21}}}
                ]
            }),
        )
        .expect("source");
        let candidates = super::collect_npc_embedded_candidates(&source);
        let equipment = key("equipment-srd", "charm");
        let converted =
            convert_npc_embedded_entities(key("bestiary", "typed"), &candidates, |locator| {
                locator.ends_with(".charm").then(|| equipment.clone())
            });
        let FactValue::Value(embedded) = converted.embedded else {
            panic!("embedded")
        };

        let action = embedded
            .occurrences_of(CreatureEntityFamily::Action)
            .next()
            .expect("action");
        let atlas_record::CreatureCapability::Action(action) = &action.capability else {
            panic!("action capability")
        };
        assert_eq!(
            action.action_cost,
            atlas_record::CreatureActionCost::Actions(2)
        );
        assert_eq!(action.rolls.len(), 2);
        assert_eq!(action.damage.as_value().map(Vec::len), Some(1));
        assert_eq!(
            action.damage.as_value().expect("damage")[0].apply_modifier,
            FactValue::Value(CreatureSourceScalar::Value(true))
        );
        assert_eq!(
            action
                .frequency
                .as_value()
                .and_then(|value| value.maximum.as_value()),
            Some(&1)
        );
        assert_eq!(action.unsupported_notes.len(), 1);

        let strike = embedded
            .occurrences_of(CreatureEntityFamily::Strike)
            .next()
            .expect("strike");
        let atlas_record::CreatureCapability::Strike(strike) = &strike.capability else {
            panic!("strike capability")
        };
        assert_eq!(
            strike.rolls[0].ability,
            FactValue::Value(atlas_record::ActivityRollAbility::Dexterity)
        );
        assert_eq!(strike.damage.as_value().map(Vec::len), Some(1));

        let entry_occurrence = embedded
            .occurrences_of(CreatureEntityFamily::SpellcastingEntry)
            .next()
            .expect("entry");
        assert_eq!(entry_occurrence.context.rank, FactValue::Value(7));
        let atlas_record::CreatureCapability::SpellcastingEntry(entry) =
            &entry_occurrence.capability
        else {
            panic!("entry capability")
        };
        assert_eq!(entry.attack, FactValue::Value(18));
        assert_eq!(entry.dc, FactValue::Value(27));
        assert_eq!(entry.slots.as_value().map(Vec::len), Some(1));
        let slot = &entry.slots.as_value().expect("slots")[0];
        assert_eq!(
            slot.maximum,
            FactValue::Value(CreatureSourceScalar::Value(2))
        );
        assert_eq!(
            slot.serialized_value,
            FactValue::Value(CreatureSourceScalar::Value(1))
        );

        let spell = embedded
            .occurrences_of(CreatureEntityFamily::Spell)
            .next()
            .expect("spell");
        assert!(matches!(
            spell.parent,
            atlas_record::CreatureOccurrenceParent::SpellcastingEntry(_)
        ));
        assert_eq!(spell.context.rank, FactValue::Value(6));
        assert_eq!(
            spell.context.location,
            FactValue::Value("entry".to_string())
        );
        assert_eq!(
            spell
                .context
                .uses
                .as_value()
                .and_then(|uses| uses.serialized_value.as_value()),
            Some(&1)
        );
        let atlas_record::CreatureCapability::Spell(spell) = &spell.capability else {
            panic!("spell capability")
        };
        assert_eq!(
            spell.action_cost,
            atlas_record::CreatureActionCost::Reaction
        );
        assert!(matches!(
            spell
                .defense
                .as_value()
                .and_then(|defense| defense.save.as_value()),
            Some(atlas_record::CreatureSpellSave::Unsupported(_))
        ));

        let equipment_occurrences = embedded
            .occurrences_of(CreatureEntityFamily::Equipment)
            .collect::<Vec<_>>();
        assert_eq!(equipment_occurrences.len(), 2);
        assert_eq!(
            equipment_occurrences[0].target,
            CreatureEntityTarget::CanonicalRecord(equipment.clone())
        );
        assert_eq!(
            equipment_occurrences[1].target,
            CreatureEntityTarget::CanonicalRecord(equipment)
        );
        assert_ne!(equipment_occurrences[0].id, equipment_occurrences[1].id);
        assert_ne!(
            equipment_occurrences[0].context.uses,
            equipment_occurrences[1].context.uses
        );

        let lore = embedded
            .occurrences_of(CreatureEntityFamily::Lore)
            .next()
            .expect("lore");
        let atlas_record::CreatureCapability::Lore(lore) = &lore.capability else {
            panic!("lore capability")
        };
        assert_eq!(lore.modifier, FactValue::Value(21));
        assert!(converted.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == NpcEmbeddedDiagnosticKind::UnsupportedMechanic
        }));
    }

    #[test]
    fn declared_consumed_capability_leaves_survive_as_exact_occurrence_notes() {
        let source = parse_npc_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("bestiary:retention", "packs/bestiary/retention.json"),
            json!({
                "_id": "retention", "name": "Retention", "type": "npc", "system": {},
                "items": [
                    {
                        "_id": "strike", "name": "Custom Strike", "type": "melee",
                        "system": {
                            "attackEffects": {
                                "value": ["grab"],
                                "custom": "Range Increment 20ft"
                            }
                        }
                    },
                    {
                        "_id": "spell", "name": "Exact Spell", "type": "spell",
                        "system": {
                            "location": {"value": "entry", "autoHeightenLevel": 7},
                            "area": {"type": "burst", "value": 5, "details": "5-foot burst or more"},
                            "damage": {
                                "0": {
                                    "formula": "2d6",
                                    "type": "piercing",
                                    "materials": ["cold-iron"]
                                }
                            },
                            "defense": {
                                "save": {"statistic": "reflex", "basic": true},
                                "passive": {"statistic": "fortitude-dc"}
                            }
                        }
                    },
                    {
                        "_id": "entry", "name": "Prepared Entry", "type": "spellcastingEntry",
                        "system": {
                            "prepared": {
                                "value": "prepared",
                                "flexible": true,
                                "label": "Prepared Arcane Spells",
                                "type": "prepared",
                                "validItems": "All Magic Items"
                            },
                            "spelldc": {
                                "value": 18,
                                "dc": 27,
                                "item": 4,
                                "label": "Arcane Spell DC",
                                "mod": 17,
                                "type": "arcane"
                            }
                        }
                    }
                ]
            }),
        )
        .expect("source");
        let candidates = super::collect_npc_embedded_candidates(&source);
        let converted =
            convert_npc_embedded_entities(key("bestiary", "retention"), &candidates, |_| None);
        let FactValue::Value(embedded) = &converted.embedded else {
            panic!("embedded")
        };

        let expected = [
            ("attackEffects.custom", "\"Range Increment 20ft\""),
            ("area.details", "\"5-foot burst or more\""),
            ("damage.0.materials[]", "\"cold-iron\""),
            ("defense.passive.statistic", "\"fortitude-dc\""),
            ("location.autoHeightenLevel", "7"),
            ("prepared.flexible", "true"),
            ("prepared.label", "\"Prepared Arcane Spells\""),
            ("prepared.type", "\"prepared\""),
            ("prepared.validItems", "\"All Magic Items\""),
            ("spelldc.item", "4"),
            ("spelldc.label", "\"Arcane Spell DC\""),
            ("spelldc.mod", "17"),
            ("spelldc.type", "\"arcane\""),
        ];
        let retained = embedded
            .occurrences
            .iter()
            .flat_map(|occurrence| {
                capability_notes(&occurrence.capability)
                    .iter()
                    .filter(|note| {
                        retained_occurrence_capability_family(occurrence.family, note).is_some()
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(retained.len(), expected.len());
        for (suffix, value) in expected {
            assert!(
                retained.iter().any(|note| {
                    note.source_path.ends_with(suffix) && note.value.value == value
                }),
                "missing exact occurrence note for {suffix}"
            );
        }

        let diagnostics = converted
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.kind == NpcEmbeddedDiagnosticKind::UnsupportedMechanic
                    && retained_capability_family(&diagnostic.source_field).is_some()
            })
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), expected.len());
        assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic.code == "atlas.npc_embedded.unsupported_mechanic.v1"
                && diagnostic.record_key == "bestiary:retention"
                && diagnostic.source_item_id.is_some()
                && diagnostic.disposition == NpcEmbeddedDiagnosticDisposition::TypedUnsupported
                && diagnostic.owner == NpcEmbeddedDiagnosticOwner::CreatureEmbeddedEntities
        }));

        let strike = embedded
            .occurrences_of(CreatureEntityFamily::Strike)
            .next()
            .expect("strike");
        let CreatureCapability::Strike(strike) = &strike.capability else {
            panic!("strike capability")
        };
        assert_eq!(
            strike.attack_effects,
            FactValue::Value(vec!["grab".to_string()])
        );
        let spell = embedded
            .occurrences_of(CreatureEntityFamily::Spell)
            .next()
            .expect("spell");
        let CreatureCapability::Spell(spell) = &spell.capability else {
            panic!("spell capability")
        };
        assert!(matches!(
            &spell.area,
            FactValue::Value(area)
                if area.area_type == FactValue::Value("burst".to_string())
                    && area.value == FactValue::Value(5)
        ));
        let entry = embedded
            .occurrences_of(CreatureEntityFamily::SpellcastingEntry)
            .next()
            .expect("entry");
        let CreatureCapability::SpellcastingEntry(entry) = &entry.capability else {
            panic!("entry capability")
        };
        assert_eq!(entry.dc, FactValue::Value(27));
        assert_eq!(entry.attack, FactValue::Value(18));
    }

    #[test]
    fn individual_scalar_type_drift_preserves_parent_siblings_and_exact_unsupported_values() {
        let source = parse_npc_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("bestiary:drift", "packs/bestiary/drift.json"),
            json!({
                "_id": "drift", "name": "Drift", "type": "npc", "system": {},
                "items": [
                    {
                        "_id": "action", "name": "Drifting Damage", "type": "action",
                        "system": {
                            "damage": {"legacy": {
                                "formula": "1d6", "type": "fire", "applyMod": {"legacy": 0}
                            }}
                        }
                    },
                    {
                        "_id": "entry", "name": "Drifting Slots", "type": "spellcastingEntry",
                        "system": {"slots": {"slot1": {"max": ["4"], "value": true}}}
                    },
                    {
                        "_id": "lore", "name": "Valid Lore", "type": "lore",
                        "system": {"mod": {"value": 12}}
                    }
                ]
            }),
        )
        .expect("field-local drift does not reject the source envelope");
        let candidates = super::collect_npc_embedded_candidates(&source);
        let converted =
            convert_npc_embedded_entities(key("bestiary", "drift"), &candidates, |_| None);
        let FactValue::Value(embedded) = &converted.embedded else {
            panic!("embedded entities")
        };
        assert_eq!(embedded.occurrences.len(), 3);

        let action = embedded
            .occurrences_of(CreatureEntityFamily::Action)
            .next()
            .expect("action sibling survives");
        let CreatureCapability::Action(action) = &action.capability else {
            panic!("action capability")
        };
        assert!(matches!(
            action
                .damage
                .as_value()
                .and_then(|damage| damage[0].apply_modifier.as_value()),
            Some(CreatureSourceScalar::Unsupported(value))
                if value.shape == UnsupportedSourceShape::Object
                    && value.value == "{\"legacy\":0}"
        ));

        let entry = embedded
            .occurrences_of(CreatureEntityFamily::SpellcastingEntry)
            .next()
            .expect("entry sibling survives");
        let CreatureCapability::SpellcastingEntry(entry) = &entry.capability else {
            panic!("entry capability")
        };
        let slot = &entry.slots.as_value().expect("slots")[0];
        assert!(matches!(
            slot.maximum.as_value(),
            Some(CreatureSourceScalar::Unsupported(value))
                if value.shape == UnsupportedSourceShape::Array && value.value == "[\"4\"]"
        ));
        assert!(matches!(
            slot.serialized_value.as_value(),
            Some(CreatureSourceScalar::Unsupported(value))
                if value.shape == UnsupportedSourceShape::Boolean && value.value == "true"
        ));

        let lore = embedded
            .occurrences_of(CreatureEntityFamily::Lore)
            .next()
            .expect("valid lore sibling survives");
        let CreatureCapability::Lore(lore) = &lore.capability else {
            panic!("lore capability")
        };
        assert_eq!(lore.modifier, FactValue::Value(12));

        let drift = converted
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == NpcEmbeddedDiagnosticKind::SourceTypeDrift)
            .collect::<Vec<_>>();
        assert_eq!(drift.len(), 3);
        assert!(drift.iter().all(|diagnostic| {
            diagnostic.code == "atlas.npc_embedded.source_type_drift.v1"
                && diagnostic.record_key == "bestiary:drift"
                && diagnostic.source_path.is_empty()
                && diagnostic.disposition == NpcEmbeddedDiagnosticDisposition::TypedUnsupported
                && diagnostic.owner == NpcEmbeddedDiagnosticOwner::CreatureEmbeddedEntities
        }));
    }

    #[test]
    fn pinned_fourteen_type_drift_records_preserve_all_supported_occurrences_and_diagnostics() {
        let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let source_root = Path::new(&source_root);
        let mut loaded = super::super::loader::load_foundry_source_records(source_root, None)
            .expect("exact pinned source loads");
        assert_eq!(loaded.source_record_count, 25_641);
        assert!(loaded.skipped_records.is_empty());
        assert_eq!(
            loaded.source_signature,
            "foundry-pf2e:sha256:dd78d67f5b6d25bf65e30ca4da66af76e7a31e1e7d990562f139154b1752603a"
        );
        let index = crate::records::references::build_record_reference_index(&loaded.records);
        super::finalize_npc_embedded_entities(&mut loaded.records, &index);

        let expected = [
            (
                "age-of-ashes-bestiary:kLX36WXp6rjTt71z",
                "packs/age-of-ashes-bestiary/book-6-broken-promises/rinnarv-bontimar.json",
                1usize,
                "boolean",
            ),
            (
                "blood-lords-bestiary:IeAy6hvDcnU1pCZr",
                "packs/blood-lords-bestiary/book-3-field-of-maidens/guloval.json",
                1,
                "boolean",
            ),
            (
                "blood-lords-bestiary:ZDrt7yX9seyY3lCC",
                "packs/blood-lords-bestiary/book-6-ghost-kings-rage/shabti-votary.json",
                1,
                "boolean",
            ),
            (
                "fists-of-the-ruby-phoenix-bestiary:6z4xzi3yZuGr1Iv7",
                "packs/fists-of-the-ruby-phoenix-bestiary/book-3-king-of-the-mountain/spirit-turtle.json",
                1,
                "boolean",
            ),
            (
                "pathfinder-bestiary:NgCKGq28Qvt6ZOSr",
                "packs/pathfinder-bestiary/gold-dragon-adult-spellcaster.json",
                1,
                "boolean",
            ),
            (
                "pathfinder-bestiary:VEnjCj4geJBzTgBJ",
                "packs/pathfinder-bestiary/gold-dragon-ancient-spellcaster.json",
                1,
                "boolean",
            ),
            (
                "pathfinder-bestiary:aq2H1lRALUNMEGRG",
                "packs/pathfinder-bestiary/guardian-naga.json",
                1,
                "boolean",
            ),
            (
                "pathfinder-bestiary-3:wLG0f6J8cgyCA0w4",
                "packs/pathfinder-bestiary-3/zuishin.json",
                1,
                "boolean",
            ),
            (
                "pfs-season-5-bestiary:vXTBGridjDSxFRlU",
                "packs/pfs-season-5-bestiary/5-18/jamimpi.json",
                1,
                "boolean",
            ),
            (
                "pfs-season-5-bestiary:05Vy7mpQdzAs55rh",
                "packs/pfs-season-5-bestiary/5-18/resolved-jamimpi.json",
                1,
                "boolean",
            ),
            (
                "pathfinder-bestiary-2:DEo5AEY1i9s0kWYZ",
                "packs/pathfinder-bestiary-2/lunar-naga.json",
                6,
                "integer",
            ),
            (
                "pathfinder-bestiary-2:3dlupMwDA9qv1xWS",
                "packs/pathfinder-bestiary-2/suli-dune-dancer.json",
                2,
                "integer",
            ),
            (
                "pathfinder-bestiary-2:WjZCNuHNKfeeHCQS",
                "packs/pathfinder-bestiary-2/worm-that-walks-cultist.json",
                14,
                "integer",
            ),
            (
                "pathfinder-monster-core:SMLMW81mKN5VlcVV",
                "packs/pathfinder-monster-core/bone-prophet.json",
                8,
                "integer",
            ),
        ];

        for (record_key, source_path, diagnostic_count, expected_shape) in expected {
            let parent = loaded
                .records
                .iter()
                .find(|record| record.record.identity.key.to_string() == record_key)
                .unwrap_or_else(|| panic!("{record_key} parent survives"));
            assert_eq!(parent.record.provenance.source_path, source_path);
            assert!(
                parent.facts.npc_source.is_some(),
                "{record_key} source envelope"
            );
            let supported_candidates = parent
                .facts
                .npc_embedded_candidates
                .as_ref()
                .and_then(|candidates| candidates.items.as_value())
                .expect("embedded candidates")
                .iter()
                .count();
            let Some(RecordBody::Creature(creature)) = &parent.facts.canonical_body else {
                panic!("{record_key} canonical parent")
            };
            let embedded = creature
                .embedded_entities
                .value
                .as_value()
                .expect("embedded entities");
            assert_eq!(
                embedded.occurrences.len(),
                supported_candidates,
                "{record_key} keeps every supported sibling occurrence"
            );
            let diagnostics = parent
                .facts
                .npc_embedded_diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.kind == NpcEmbeddedDiagnosticKind::SourceTypeDrift)
                .collect::<Vec<_>>();
            assert_eq!(diagnostics.len(), diagnostic_count, "{record_key}");
            assert!(diagnostics.iter().all(|diagnostic| {
                diagnostic.code == "atlas.npc_embedded.source_type_drift.v1"
                    && diagnostic.record_key == record_key
                    && diagnostic.source_path == source_path
                    && diagnostic.source_field.starts_with("$.items[")
                    && diagnostic.expected_shape == expected_shape
                    && diagnostic.observed_shape == "string"
                    && diagnostic.source_value.starts_with('"')
                    && diagnostic.source_value.ends_with('"')
                    && diagnostic.disposition == NpcEmbeddedDiagnosticDisposition::TypedUnsupported
                    && diagnostic.owner == NpcEmbeddedDiagnosticOwner::CreatureEmbeddedEntities
            }));
        }

        let drift = loaded
            .records
            .iter()
            .flat_map(|record| &record.facts.npc_embedded_diagnostics)
            .filter(|diagnostic| diagnostic.kind == NpcEmbeddedDiagnosticKind::SourceTypeDrift)
            .collect::<Vec<_>>();
        assert_eq!(drift.len(), 40);
        assert_eq!(
            drift
                .iter()
                .filter(|diagnostic| diagnostic.expected_shape == "boolean")
                .count(),
            10
        );
        assert_eq!(
            drift
                .iter()
                .filter(|diagnostic| diagnostic.expected_shape == "integer")
                .count(),
            30
        );
        assert_eq!(
            loaded
                .records
                .iter()
                .flat_map(|record| &record.facts.npc_core_diagnostics)
                .filter(|diagnostic| diagnostic.code
                    == "atlas.npc_core.component_id_source_fallback.v1")
                .count(),
            8
        );

        let report = crate::report::analyze_source_load(source_root.to_path_buf(), loaded);
        let type_drift = report
            .diagnostics
            .pointer("/source_preservation/npc_embedded_entities/type_drift")
            .expect("portable type-drift report");
        assert_eq!(
            type_drift.pointer("/count").and_then(Value::as_u64),
            Some(40)
        );
        assert_eq!(
            type_drift
                .pointer("/by_disposition/preserved_as_typed_unsupported")
                .and_then(Value::as_u64),
            Some(40)
        );
        let entries = type_drift
            .pointer("/entries")
            .and_then(Value::as_array)
            .expect("diagnostic entries");
        assert!(entries.iter().all(|entry| {
            entry.pointer("/code").and_then(Value::as_str)
                == Some("atlas.npc_embedded.source_type_drift.v1")
                && entry
                    .pointer("/source_path")
                    .and_then(Value::as_str)
                    .is_some_and(|path| path.starts_with("packs/") && !path.starts_with('/'))
                && entry
                    .pointer("/record_key")
                    .and_then(Value::as_str)
                    .is_some()
                && entry
                    .pointer("/source_field")
                    .and_then(Value::as_str)
                    .is_some()
                && entry
                    .pointer("/expected_shape")
                    .and_then(Value::as_str)
                    .is_some()
                && entry
                    .pointer("/observed_shape")
                    .and_then(Value::as_str)
                    .is_some()
                && entry
                    .pointer("/source_value")
                    .and_then(Value::as_str)
                    .is_some()
                && entry.pointer("/disposition").and_then(Value::as_str)
                    == Some("preserved_as_typed_unsupported")
                && entry.pointer("/owner").and_then(Value::as_str)
                    == Some("creature_embedded_entities")
        }));
    }

    #[test]
    fn absent_nested_id_uses_diagnosed_unstable_ordinal_fallback() {
        let source = parse_npc_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("bestiary:actor", "packs/bestiary/actor.json"),
            json!({"_id":"actor","name":"Actor","type":"npc","system":{},"items":[spell("valid","Spell",None::<&'static str>,"entry",1)]}),
        ).expect("source");
        let mut candidates = super::collect_npc_embedded_candidates(&source);
        let SourcePresence::Value(items) = &mut candidates.items else {
            panic!("items")
        };
        items[0].nested_source_id.clear();
        let converted =
            convert_npc_embedded_entities(key("bestiary", "actor"), &candidates, |_| None);
        assert!(
            converted
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind
                    == NpcEmbeddedDiagnosticKind::UnstableOwnerFamilyOrdinalIdentity)
        );
    }

    #[test]
    fn source_sort_controls_authored_order_without_controlling_occurrence_identity() {
        let source = parse_npc_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("bestiary:sort", "packs/bestiary/sort.json"),
            json!({
                "_id": "sort", "name": "Sort", "type": "npc", "system": {},
                "items": [
                    {"_id":"b","name":"B","type":"action","sort":20,"system":{}},
                    {"_id":"c","name":"C","type":"action","sort":10,"system":{}},
                    {"_id":"a","name":"A","type":"action","sort":20,"system":{}}
                ]
            }),
        )
        .expect("source");
        let owner = key("bestiary", "sort");
        let candidates = super::collect_npc_embedded_candidates(&source);
        let first = convert_npc_embedded_entities(owner.clone(), &candidates, |_| None);
        let FactValue::Value(first) = first.embedded else {
            panic!("embedded")
        };
        let labels = first
            .occurrences
            .iter()
            .map(|occurrence| {
                occurrence
                    .context
                    .contextual_label
                    .as_value()
                    .unwrap()
                    .as_str()
            })
            .collect::<Vec<_>>();
        assert_eq!(labels, ["C", "A", "B"]);
        assert_eq!(first.occurrences[0].source_sort, FactValue::Value(10));

        let mut reordered = candidates.clone();
        let SourcePresence::Value(items) = &mut reordered.items else {
            panic!("items")
        };
        items.reverse();
        let second = convert_npc_embedded_entities(owner, &reordered, |_| None);
        let FactValue::Value(second) = second.embedded else {
            panic!("embedded")
        };
        assert_eq!(
            first
                .occurrences
                .iter()
                .map(|occurrence| occurrence.id.clone())
                .collect::<Vec<_>>(),
            second
                .occurrences
                .iter()
                .map(|occurrence| occurrence.id.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn prepared_slots_actor_context_locators_and_nonexecuting_relationships_are_typed() {
        let source = parse_npc_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("bestiary:relationships", "packs/bestiary/relationships.json"),
            json!({
                "_id": "relationships", "name": "Relationships", "type": "npc",
                "system": {"spellcasting": {"rituals": {"dc": 31}}},
                "items": [
                    {
                        "_id":"entry","name":"Prepared Spells","type":"spellcastingEntry","sort":10,
                        "flags":{"core":{"sourceId":"Compendium.pf2e.bestiary-ability-glossary-srd.Item.entry"}},
                        "system":{"slots":{"slot4":{"max":2,"value":2,"prepared":[{"id":"spell","expended":true}]}}}
                    },
                    {
                        "_id":"spell","name":"Fireball","type":"spell","sort":20,
                        "system":{"level":{"value":3},"location":{"value":"entry"}}
                    },
                    {
                        "_id":"grantor","name":"Grantor","type":"action","sort":30,
                        "flags":{"pf2e":{"itemGrants":{"child":{"id":"child","onDelete":"detach"}}}},
                        "system":{}
                    },
                    {
                        "_id":"child","name":"Child","type":"effect","sort":40,
                        "flags":{"pf2e":{"grantedBy":{"id":"grantor","onDelete":"cascade"}}},
                        "system":{"futureFact":{"value":7}}
                    },
                    {
                        "_id":"strike","name":"Strike","type":"melee","sort":50,
                        "flags":{"pf2e":{"linkedWeapon":"weapon"}},"system":{}
                    },
                    {"_id":"weapon","name":"Weapon","type":"weapon","sort":60,"system":{}}
                ]
            }),
        )
        .expect("source");
        let candidates = super::collect_npc_embedded_candidates(&source);
        let converted = convert_npc_embedded_entities(
            key("bestiary", "relationships"),
            &candidates,
            |locator| {
                locator
                    .ends_with(".entry")
                    .then(|| key("bestiary-ability-glossary-srd", "entry"))
            },
        );
        let FactValue::Value(embedded) = converted.embedded else {
            panic!("embedded")
        };
        assert!(matches!(
            embedded.actor_spellcasting,
            FactValue::Value(ref context)
                if context.rituals_dc == FactValue::Value(CreatureSourceScalar::Value(31))
        ));
        let entry = embedded
            .occurrences_of(CreatureEntityFamily::SpellcastingEntry)
            .next()
            .expect("entry");
        assert_eq!(entry.source_identity.source_locators.len(), 1);
        assert!(matches!(
            entry.target,
            CreatureEntityTarget::CanonicalRecord(_)
        ));
        let CreatureCapability::SpellcastingEntry(entry_capability) = &entry.capability else {
            panic!("entry capability")
        };
        assert!(matches!(
            &entry_capability.slots.as_value().unwrap()[0].prepared,
            FactValue::Value(prepared)
                if matches!(&prepared[0], CreaturePreparedSpellSlot::Spell { id: FactValue::Value(id), expended: FactValue::Value(true), .. } if id.as_str() == "spell")
        ));
        let spell = embedded
            .occurrences_of(CreatureEntityFamily::Spell)
            .next()
            .expect("spell");
        assert_eq!(spell.context.rank, FactValue::Value(4));
        assert_eq!(spell.context.slot, FactValue::Value("slot4:0".to_string()));
        assert!(matches!(
            spell.parent,
            super::CreatureOccurrenceParent::SpellcastingEntry(_)
        ));
        assert_eq!(embedded.relationships.len(), 4);
        assert!(embedded.relationships.iter().all(|relationship| {
            relationship.execution == CreatureRelationshipExecution::ProvenanceOnly
                && matches!(
                    relationship.target,
                    CreatureRelationshipTarget::Occurrence(_)
                )
        }));
        assert_eq!(
            embedded
                .relationships
                .iter()
                .filter(|relationship| relationship.kind
                    == CreatureEntityRelationshipKind::PreparedSpell)
                .count(),
            1
        );
        let shell = embedded
            .occurrences_of(CreatureEntityFamily::Effect)
            .next()
            .expect("typed effect shell");
        assert!(matches!(
            &shell.capability,
            CreatureCapability::Unsupported(capability)
                if capability.unsupported_notes.iter().any(|note| note.source_path.ends_with("futureFact.value"))
        ));
    }

    #[test]
    fn pinned_corpus_accounts_for_every_b4_order_slot_context_locator_and_relationship() {
        let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let mut loaded =
            super::super::loader::load_foundry_source_records(Path::new(&source_root), None)
                .expect("exact pinned source loads");
        assert_eq!(loaded.source_record_count, 25_641);
        assert!(loaded.skipped_records.is_empty());
        assert_eq!(
            loaded.source_signature,
            "foundry-pf2e:sha256:dd78d67f5b6d25bf65e30ca4da66af76e7a31e1e7d990562f139154b1752603a"
        );
        let index = crate::records::references::build_record_reference_index(&loaded.records);
        super::finalize_npc_embedded_entities(&mut loaded.records, &index);

        let embedded = loaded
            .records
            .iter()
            .filter_map(|record| {
                let RecordBody::Creature(creature) = record.facts.canonical_body.as_ref()?;
                creature.embedded_entities.value.as_value()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            embedded
                .iter()
                .map(|value| value.occurrences.len())
                .sum::<usize>(),
            72_354
        );
        assert_eq!(
            embedded
                .iter()
                .flat_map(|value| &value.occurrences)
                .filter(|occurrence| occurrence.source_sort.as_value().is_some())
                .count(),
            72_354
        );
        assert_eq!(
            embedded
                .iter()
                .filter(|value| value.actor_spellcasting.as_value().is_some())
                .count(),
            262
        );
        assert_eq!(
            embedded
                .iter()
                .flat_map(|value| &value.occurrences)
                .flat_map(|occurrence| &occurrence.source_identity.source_locators)
                .filter(|locator| locator.source_path == "$.flags.core.sourceId")
                .count(),
            15
        );

        let relationships = embedded
            .iter()
            .flat_map(|value| &value.relationships)
            .collect::<Vec<_>>();
        assert_eq!(
            relationships
                .iter()
                .filter(
                    |relationship| relationship.kind == CreatureEntityRelationshipKind::GrantedBy
                )
                .count(),
            3
        );
        assert_eq!(
            relationships
                .iter()
                .filter(
                    |relationship| relationship.kind == CreatureEntityRelationshipKind::ItemGrant
                )
                .count(),
            8
        );
        assert_eq!(
            relationships
                .iter()
                .filter(|relationship| relationship.kind
                    == CreatureEntityRelationshipKind::LinkedWeapon)
                .count(),
            3_725
        );
        assert_eq!(
            relationships
                .iter()
                .filter(|relationship| relationship.kind
                    == CreatureEntityRelationshipKind::PreparedSpell)
                .count(),
            8_818
        );
        assert!(
            relationships
                .iter()
                .all(|relationship| relationship.execution
                    == CreatureRelationshipExecution::ProvenanceOnly)
        );

        let expected_capability_counts = BTreeMap::from([
            ("$.items[].system.attackEffects.custom", 6_usize),
            ("$.items[].system.area.details", 197),
            ("$.items[].system.damage.*.materials[]", 7),
            ("$.items[].system.defense.passive.statistic", 45),
            ("$.items[].system.location.autoHeightenLevel", 1),
            ("$.items[].system.prepared.flexible", 1_535),
            ("$.items[].system.prepared.label", 32),
            ("$.items[].system.prepared.type", 32),
            ("$.items[].system.prepared.validItems", 2),
            ("$.items[].system.spelldc.item", 245),
            ("$.items[].system.spelldc.label", 32),
            ("$.items[].system.spelldc.mod", 1_675),
            ("$.items[].system.spelldc.type", 32),
        ]);
        let mut retained_capability_counts = BTreeMap::new();
        for occurrence in embedded.iter().flat_map(|value| &value.occurrences) {
            for note in capability_notes(&occurrence.capability) {
                if let Some(family) = retained_occurrence_capability_family(occurrence.family, note)
                {
                    *retained_capability_counts.entry(family).or_insert(0) += 1;
                }
            }
        }
        assert_eq!(retained_capability_counts, expected_capability_counts);
        assert_eq!(retained_capability_counts.values().sum::<usize>(), 3_841);

        let retained_diagnostics = loaded
            .records
            .iter()
            .flat_map(|record| &record.facts.npc_embedded_diagnostics)
            .filter(|diagnostic| {
                diagnostic.kind == NpcEmbeddedDiagnosticKind::UnsupportedMechanic
                    && retained_capability_family(&diagnostic.source_field).is_some()
            })
            .collect::<Vec<_>>();
        assert_eq!(retained_diagnostics.len(), 3_841);
        assert!(retained_diagnostics.iter().all(|diagnostic| {
            !diagnostic.record_key.is_empty()
                && diagnostic.source_path.starts_with("packs/")
                && diagnostic.source_item_id.is_some()
                && !diagnostic.source_value.is_empty()
                && diagnostic.disposition == NpcEmbeddedDiagnosticDisposition::TypedUnsupported
                && diagnostic.owner == NpcEmbeddedDiagnosticOwner::CreatureEmbeddedEntities
        }));
        let mut diagnostic_counts = BTreeMap::new();
        for diagnostic in retained_diagnostics {
            let family = retained_capability_family(&diagnostic.source_field)
                .expect("retained diagnostic family");
            *diagnostic_counts.entry(family).or_insert(0) += 1;
        }
        assert_eq!(diagnostic_counts, expected_capability_counts);
    }

    #[test]
    fn pinned_night_hag_retains_all_embedded_entity_families() {
        let Some(root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let relative = "packs/pathfinder-bestiary/night-hag.json";
        let raw: Value = serde_json::from_str(
            &std::fs::read_to_string(Path::new(&root).join(relative)).expect("Night Hag source"),
        )
        .expect("Night Hag JSON");
        let source = parse_npc_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("pathfinder-bestiary:WQy7HBUcgDLsfVJd", relative),
            raw,
        )
        .expect("Night Hag DTO");
        let candidates = super::collect_npc_embedded_candidates(&source);
        let converted = convert_npc_embedded_entities(
            key("pathfinder-bestiary", "WQy7HBUcgDLsfVJd"),
            &candidates,
            |locator| {
                let parts = locator.split('.').collect::<Vec<_>>();
                (parts.len() >= 5).then(|| key(parts[2], parts[4]))
            },
        );
        let FactValue::Value(embedded) = converted.embedded else {
            panic!("embedded")
        };
        assert_eq!(
            embedded
                .occurrences_of(CreatureEntityFamily::SpellcastingEntry)
                .count(),
            2
        );
        assert_eq!(
            embedded.occurrences_of(CreatureEntityFamily::Spell).count(),
            27
        );
        assert_eq!(
            embedded
                .occurrences_of(CreatureEntityFamily::Strike)
                .count(),
            2
        );
        assert_eq!(
            embedded
                .occurrences_of(CreatureEntityFamily::Action)
                .count(),
            9
        );
        assert_eq!(
            embedded
                .occurrences_of(CreatureEntityFamily::Equipment)
                .count(),
            1
        );
        assert_eq!(embedded.occurrences.len(), 41);
        assert!(
            embedded
                .occurrences
                .iter()
                .enumerate()
                .all(|(index, occurrence)| {
                    occurrence.authored_order == index as u32
                        && occurrence.identity_stability
                            == atlas_record::OccurrenceIdentityStability::StableNestedSourceId
                })
        );
        assert_eq!(
            embedded
                .occurrences
                .iter()
                .map(|occurrence| occurrence.id.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            41
        );
        assert!(embedded.occurrences.iter().any(|occurrence| {
            occurrence
                .context
                .contextual_label
                .as_value()
                .map(String::as_str)
                == Some("Heartstone")
        }));
        let nightmares = embedded
            .occurrences_of(CreatureEntityFamily::Spell)
            .filter(|occurrence| {
                occurrence
                    .context
                    .contextual_label
                    .as_value()
                    .map(String::as_str)
                    == Some("Nightmare")
            })
            .collect::<Vec<_>>();
        assert_eq!(nightmares.len(), 2);
        assert_eq!(nightmares[0].target, nightmares[1].target);
        assert_ne!(nightmares[0].id, nightmares[1].id);
        assert_eq!(
            embedded
                .occurrences_of(CreatureEntityFamily::Spell)
                .filter(|occurrence| matches!(
                    occurrence.parent,
                    atlas_record::CreatureOccurrenceParent::SpellcastingEntry(_)
                ))
                .count(),
            26
        );
        assert!(
            embedded
                .occurrences_of(CreatureEntityFamily::Spell)
                .any(|occurrence| {
                    let atlas_record::CreatureCapability::Spell(spell) = &occurrence.capability
                    else {
                        return false;
                    };
                    spell.damage.as_value().is_some_and(Vec::is_empty)
                })
        );
    }

    fn capability_notes(capability: &CreatureCapability) -> &[UnsupportedMechanicNote] {
        capability_unsupported_notes_ref(capability)
    }

    fn spell(
        id: &str,
        name: &str,
        locator: impl Into<Option<&'static str>>,
        location: &str,
        rank: i64,
    ) -> Value {
        let locator = locator.into();
        json!({
            "_id": id, "name": name, "type": "spell",
            "_stats": {"compendiumSource": locator},
            "system": {
                "level": {"value": rank}, "location": {"value": location, "heightenedLevel": rank},
                "traits": {"value": []}, "damage": {}, "target": {"value": ""},
                "duration": {"value": "", "sustained": false}, "time": {"value": "2"}
            }
        })
    }

    fn key(pack: &str, id: &str) -> RecordKey {
        RecordKey::new(
            PackName::new(pack.to_string()).expect("pack"),
            RecordId::new(id.to_string()).expect("id"),
        )
    }
}
