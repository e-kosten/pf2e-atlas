use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::{Rarity, RecordKey};
use atlas_record::{
    ConsumableContentIdentity, ConsumableDamage, ConsumableDefinition, ConsumableEntity,
    ConsumableEntityId, ConsumableEntityTarget, ConsumableEquippedState, ConsumableExactDecimal,
    ConsumableFact, ConsumableIdentity, ConsumableLocalSpellEvidence, ConsumableLocatorState,
    ConsumableMaterial, ConsumableMismatch, ConsumableMismatchValue, ConsumableOccurrence,
    ConsumableOccurrenceId, ConsumableOccurrenceIdentityStability, ConsumablePrice,
    ConsumablePriceDenomination, ConsumableProvenance, ConsumablePublication, ConsumableRecord,
    ConsumableSourceId, ConsumableSourceState, ConsumableSourceValue,
    ConsumableSpellMismatchReason, ConsumableSpellReuse, ConsumableTargetResolution, ContentId,
    ContentKey, ContentOrigin, ContentOwner, ContentProvenance, ContentRole,
    DuplicateContentStatus, FactValue, OwnedRichContent, OwnedRichContentDocument,
    PublicationLicense, RecordBody, StableSourceLocator, UnsupportedSourceReason,
    UnsupportedSourceShape, UnsupportedSourceValue,
};

use crate::records::{LoadedSourceRecord, RecordReferenceIndex};

use super::dto::{
    ConsumableDamageSource, ConsumableEquippedSource, ConsumableItemSource,
    ConsumableMaterialSource, ConsumablePriceSource, ConsumablePublicationSource,
    ConsumableSourceFact, ConsumableSpellChildSource, ConsumableUnsupportedSource, ItemSource,
    LegacyDuplicateDisposition, SerializedSourceMember, SerializedSourceObject,
    SerializedSourceValue, SourceIdentity, SourceParentContext, SourcePresence,
    SpellDocumentSource, VersionedItemSource, parse_item_source_from_serialized,
    parse_spell_document_source, pinned_source_version_metadata,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumableOccurrenceCandidate {
    pub(crate) authored_order: u32,
    pub(crate) source_path: String,
    pub(crate) source_id: ConsumableSourceFact<String>,
    pub(crate) content_source_id: String,
    pub(crate) source: VersionedItemSource,
    spell_source: LocalSpellSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalSpellSource {
    Missing,
    Null,
    Malformed(UnsupportedSourceValue),
    Child(Box<ConsumableSpellChildSource>),
}

pub(crate) fn collect_actor_consumable_candidates(
    root: &SerializedSourceObject,
    actor_type: &str,
    owner_key: &RecordKey,
    source_path: &str,
) -> Result<Vec<ConsumableOccurrenceCandidate>, String> {
    let parent_context = match actor_type {
        "npc" => SourceParentContext::NpcItems,
        "character" => SourceParentContext::CharacterItems,
        "hazard" => SourceParentContext::HazardItems,
        _ => return Ok(Vec::new()),
    };
    let items = match root.member("items") {
        SerializedSourceMember::Missing | SerializedSourceMember::Null => return Ok(Vec::new()),
        SerializedSourceMember::Value(SerializedSourceValue::Array(items)) => items,
        SerializedSourceMember::Value(value) => {
            return Err(format!(
                "Actor[{actor_type}].items must be an array, found {}",
                value.compact_json()
            ));
        }
        SerializedSourceMember::Duplicate(values) => {
            return Err(format!(
                "Actor[{actor_type}].items has duplicate members: [{}]",
                values
                    .iter()
                    .map(|value| value.compact_json())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    };
    let mut candidates = Vec::new();
    for (ordinal, value) in items.iter().enumerate() {
        let SerializedSourceValue::Object(item) = value else {
            continue;
        };
        let is_consumable = matches!(
            item.member("type"),
            SerializedSourceMember::Value(SerializedSourceValue::String(value))
                if value == "consumable"
        );
        if !is_consumable {
            continue;
        }
        let authored_order = u32::try_from(ordinal)
            .map_err(|_| "Actor items exceed supported authored-order range".to_string())?;
        let source_id = consumable_source_id(item);
        let parse_source = consumable_parse_source(item, owner_key, authored_order, &source_id);
        // The typed consumable DTO below is the owner of duplicate evidence. The
        // legacy envelope is needed only by the still-shared Item parser, so omit
        // duplicate members there after the lossless SerializedSourceObject has
        // retained them. This prevents a nested duplicate from dropping the
        // complete consumable occurrence before its field can become Unsupported.
        let raw = parse_source
            .to_legacy_json(&mut |_, _| LegacyDuplicateDisposition::OmitAfterFamilyRetention)
            .map_err(|error| error.to_string())?;
        let identity = SourceIdentity::new(
            format!("{}#items[{ordinal}]", owner_key),
            format!("{source_path}#items[{ordinal}]"),
        );
        let source = parse_item_source_from_serialized(
            pinned_source_version_metadata(),
            identity,
            Some(parent_context.clone()),
            raw,
            &parse_source,
        )
        .map_err(|error| error.to_string())?;
        let spell_source = local_spell_source(
            item,
            &SourceIdentity::new(
                format!("{}#items[{ordinal}]", owner_key),
                format!("{source_path}#items[{ordinal}]"),
            ),
        );
        candidates.push(ConsumableOccurrenceCandidate {
            authored_order,
            source_path: format!("{source_path}#items[{ordinal}]"),
            content_source_id: match &source_id {
                ConsumableSourceFact::Value(value) => value.clone(),
                _ => format!("item-{ordinal}"),
            },
            source_id,
            source,
            spell_source,
        });
    }
    Ok(candidates)
}

fn local_spell_source(
    item: &SerializedSourceObject,
    identity: &SourceIdentity,
) -> LocalSpellSource {
    let system = match item.member("system") {
        SerializedSourceMember::Value(SerializedSourceValue::Object(system)) => system,
        member => return LocalSpellSource::Malformed(unsupported_spell_member(member)),
    };
    match system.member("spell") {
        SerializedSourceMember::Missing => LocalSpellSource::Missing,
        SerializedSourceMember::Null => LocalSpellSource::Null,
        member => match parse_spell_document_source(item, identity) {
            Ok(Some(SpellDocumentSource::ConsumableChild(child))) => {
                LocalSpellSource::Child(Box::new(child))
            }
            Ok(Some(SpellDocumentSource::Standalone(_))) | Ok(None) | Err(_) => {
                LocalSpellSource::Malformed(unsupported_spell_member(member))
            }
        },
    }
}

fn unsupported_spell_member(member: SerializedSourceMember<'_>) -> UnsupportedSourceValue {
    let (shape, value) = match member {
        SerializedSourceMember::Missing => {
            (UnsupportedSourceShape::Missing, "<missing>".to_string())
        }
        SerializedSourceMember::Null => (UnsupportedSourceShape::Null, "null".to_string()),
        SerializedSourceMember::Value(value) => (value.shape(), value.compact_json()),
        SerializedSourceMember::Duplicate(values) => (
            UnsupportedSourceShape::Array,
            format!(
                "[{}]",
                values
                    .iter()
                    .map(|value| value.compact_json())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ),
    };
    UnsupportedSourceValue {
        shape,
        value,
        reason: UnsupportedSourceReason::SourceFieldDrift,
    }
}

pub(crate) fn stabilize_actor_consumable_legacy_ids(
    source: &SerializedSourceObject,
    raw: &mut serde_json::Value,
    owner_key: &RecordKey,
) {
    let SerializedSourceMember::Value(SerializedSourceValue::Array(source_items)) =
        source.member("items")
    else {
        return;
    };
    let Some(raw_items) = raw
        .get_mut("items")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };
    let source_ids = source_items
        .iter()
        .filter_map(SerializedSourceValue::object)
        .filter(|item| {
            matches!(
                item.member("type"),
                SerializedSourceMember::Value(value) if value.string() == Some("consumable")
            )
        })
        .map(consumable_source_id)
        .collect::<Vec<_>>();
    let counts = source_ids
        .iter()
        .fold(BTreeMap::new(), |mut counts, source_id| {
            if let Some(source_id) = valid_source_id(source_id) {
                *counts.entry(source_id.to_string()).or_insert(0_usize) += 1;
            }
            counts
        });

    for (ordinal, (source_item, raw_item)) in source_items.iter().zip(raw_items).enumerate() {
        let (Some(source_item), Some(raw_item)) = (source_item.object(), raw_item.as_object_mut())
        else {
            continue;
        };
        if !matches!(
            source_item.member("type"),
            SerializedSourceMember::Value(value) if value.string() == Some("consumable")
        ) {
            continue;
        }
        let source_id = consumable_source_id(source_item);
        let stable =
            valid_source_id(&source_id).is_some_and(|source_id| counts.get(source_id) == Some(&1));
        if !stable {
            raw_item.insert(
                "_id".to_string(),
                serde_json::Value::String(format!(
                    "atlas-consumable-envelope-{}-{ordinal}",
                    hex(owner_key.to_string().as_bytes())
                )),
            );
        }
    }
}

fn consumable_source_id(item: &SerializedSourceObject) -> ConsumableSourceFact<String> {
    match item.member("_id") {
        SerializedSourceMember::Missing => ConsumableSourceFact::Missing,
        SerializedSourceMember::Null => ConsumableSourceFact::Null,
        SerializedSourceMember::Value(SerializedSourceValue::String(value)) => {
            ConsumableSourceFact::Value(value.clone())
        }
        SerializedSourceMember::Value(value) => {
            ConsumableSourceFact::Unsupported(ConsumableUnsupportedSource {
                shape: value.shape(),
                value: value.compact_json(),
                reason: "consumable source ID must be a string",
            })
        }
        SerializedSourceMember::Duplicate(values) => {
            ConsumableSourceFact::Unsupported(ConsumableUnsupportedSource {
                shape: UnsupportedSourceShape::Array,
                value: format!(
                    "[{}]",
                    values
                        .iter()
                        .map(|value| value.compact_json())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                reason: "duplicate consumable source ID",
            })
        }
    }
}

fn valid_source_id(source_id: &ConsumableSourceFact<String>) -> Option<&str> {
    let ConsumableSourceFact::Value(value) = source_id else {
        return None;
    };
    ConsumableSourceId::new(value.clone())
        .ok()
        .map(|_| value.as_str())
}

fn valid_candidate_source_id(candidate: &ConsumableOccurrenceCandidate) -> Option<&str> {
    valid_source_id(&candidate.source_id)
}

fn consumable_parse_source(
    item: &SerializedSourceObject,
    owner_key: &RecordKey,
    authored_order: u32,
    source_id: &ConsumableSourceFact<String>,
) -> SerializedSourceObject {
    if valid_source_id(source_id).is_some() {
        return item.clone();
    }
    let parser_id = format!(
        "atlas-consumable-envelope-{}-{authored_order}",
        hex(owner_key.to_string().as_bytes())
    );
    let mut fields = vec![("_id".to_string(), SerializedSourceValue::String(parser_id))];
    fields.extend(
        item.fields()
            .iter()
            .filter(|(member, _)| member != "_id")
            .cloned(),
    );
    SerializedSourceObject::from_fields(fields)
}

pub(crate) fn convert_standalone_consumable(
    record_key: RecordKey,
    source_path: &str,
    source: &VersionedItemSource,
) -> Result<RecordBody, String> {
    let ItemSource::Consumable(item) = &source.source else {
        return Err("standalone consumable conversion received a non-consumable item".to_string());
    };
    let definition = item
        .consumable
        .as_ref()
        .ok_or_else(|| "consumable source is missing its typed definition".to_string())?;
    let source_id = ConsumableSourceId::new(item.id.clone())
        .map_err(|_| format!("invalid consumable source ID {:?}", item.id))?;

    Ok(RecordBody::Consumable(ConsumableRecord {
        identity: ConsumableIdentity {
            record_key,
            source_id,
            name: item.name.clone(),
        },
        definition: convert_definition(definition)?,
        source_state: convert_state(definition),
        content: OwnedRichContent::default(),
        unsupported_content: definition
            .unsupported_content
            .iter()
            .map(convert_unsupported)
            .collect(),
        provenance: ConsumableProvenance {
            image: source_presence_fact(&item.image),
            folder: source_presence_fact(&item.folder),
            source_sort: source_presence_fact(&item.sort),
            target_locator: convert_locator(&definition.target_locator),
            source_path: source_path.to_string(),
            source_contract_version: source.version.contract_version().to_string(),
            source_system_version: source.version.system_version().to_string(),
            source_upstream_commit: source.version.upstream_commit().to_string(),
        },
    }))
}

pub(crate) fn finalize_standalone_consumable_spell_bindings(
    records: &mut [LoadedSourceRecord],
) -> Result<(), String> {
    for loaded in records {
        let Some(RecordBody::Consumable(consumable)) = &mut loaded.facts.canonical_body else {
            continue;
        };
        consumable.definition.spell_child_id =
            match loaded.facts.canonical_spell_children.as_slice() {
                [] => FactValue::Missing,
                [child] => FactValue::Value(child.child_id.clone()),
                children => {
                    return Err(format!(
                        "consumable `{}` has {} canonical spell children; expected at most one",
                        loaded.record.identity.key,
                        children.len()
                    ));
                }
            };
    }
    Ok(())
}

pub(crate) fn finalize_consumable_occurrences(
    records: &mut [LoadedSourceRecord],
    index: &RecordReferenceIndex,
) -> Result<(), String> {
    let targets = records
        .iter()
        .filter_map(|loaded| {
            let RecordBody::Consumable(consumable) = loaded.facts.canonical_body.as_ref()? else {
                return None;
            };
            Some((
                consumable.identity.record_key.clone(),
                (
                    consumable.clone(),
                    loaded.facts.canonical_spell_children.clone(),
                ),
            ))
        })
        .collect::<BTreeMap<_, _>>();

    for loaded in records {
        if loaded.facts.consumable_occurrence_candidates.is_empty() {
            continue;
        }
        let owner_key = loaded.record.identity.key.clone();
        let candidates = loaded.facts.consumable_occurrence_candidates.clone();
        let authored_counts = candidates
            .iter()
            .fold(BTreeMap::new(), |mut counts, candidate| {
                if let Some(id) = valid_candidate_source_id(candidate) {
                    *counts.entry(id.to_string()).or_insert(0_usize) += 1;
                }
                counts
            });
        let mut allocated = candidates
            .iter()
            .filter_map(|candidate| {
                let id = valid_candidate_source_id(candidate)?;
                (authored_counts.get(id) == Some(&1)).then(|| stable_occurrence_id(id))
            })
            .collect::<Result<BTreeSet<_>, _>>()?;

        for candidate in candidates {
            let item = candidate.source.source.source();
            let stable_source_id = valid_candidate_source_id(&candidate);
            let has_stable_identity =
                stable_source_id.is_some_and(|id| authored_counts.get(id) == Some(&1));
            let occurrence_id = match (stable_source_id, has_stable_identity) {
                (Some(source_id), true) => stable_occurrence_id(source_id)?,
                _ => fallback_occurrence_id(&owner_key, candidate.authored_order, &mut allocated)?,
            };
            allocated.insert(occurrence_id.clone());
            let identity_stability = if has_stable_identity {
                ConsumableOccurrenceIdentityStability::StableSourceIdentity
            } else {
                ConsumableOccurrenceIdentityStability::UnstableOwnerOrdinal
            };
            let definition_source = item
                .consumable
                .as_ref()
                .ok_or_else(|| "embedded consumable lacks typed definition".to_string())?;
            let local_definition = convert_definition(definition_source)?;
            let local_state = convert_state(definition_source);
            let locator = convert_locator(&definition_source.target_locator);
            let resolved_key = match &locator {
                ConsumableLocatorState::Known(locator) => strict_pack_id(locator.as_str(), index),
                ConsumableLocatorState::Missing
                | ConsumableLocatorState::Null
                | ConsumableLocatorState::Unsupported(_) => None,
            };
            let target = match (&locator, resolved_key.as_ref()) {
                (ConsumableLocatorState::Known(locator), Some(record_key)) => {
                    if let Some((target_record, _)) = targets.get(record_key) {
                        ConsumableEntityTarget::Resolved {
                            record_key: record_key.clone(),
                            immutable_mismatches: definition_mismatches(
                                &local_definition,
                                &target_record.definition,
                            ),
                        }
                    } else {
                        ConsumableEntityTarget::ParentOwned {
                            definition: Box::new(local_definition.clone()),
                            resolution: ConsumableTargetResolution::WrongDocumentOrFamily(
                                locator.clone(),
                            ),
                            content_identity: FactValue::Missing,
                        }
                    }
                }
                (ConsumableLocatorState::Known(locator), None) => {
                    ConsumableEntityTarget::ParentOwned {
                        definition: Box::new(local_definition.clone()),
                        resolution: ConsumableTargetResolution::TargetMissing(locator.clone()),
                        content_identity: FactValue::Missing,
                    }
                }
                (ConsumableLocatorState::Unsupported(value), _) => {
                    ConsumableEntityTarget::ParentOwned {
                        definition: Box::new(local_definition.clone()),
                        resolution: ConsumableTargetResolution::MalformedOrDuplicateLocator(
                            value.clone(),
                        ),
                        content_identity: FactValue::Missing,
                    }
                }
                (ConsumableLocatorState::Missing | ConsumableLocatorState::Null, _) => {
                    ConsumableEntityTarget::ParentOwned {
                        definition: Box::new(local_definition.clone()),
                        resolution: ConsumableTargetResolution::NoLocator,
                        content_identity: FactValue::Missing,
                    }
                }
            };
            let entity_id = ConsumableEntityId::new(format!("entity-{}", occurrence_id.as_str()))
                .map_err(|_| "invalid consumable entity ID".to_string())?;
            let resolved_target = resolved_key
                .as_ref()
                .and_then(|record_key| targets.get(record_key))
                .map(|(record, _)| record);
            let authored_content = occurrence_content(
                loaded,
                &occurrence_id,
                candidate.authored_order,
                match &candidate.source_id {
                    ConsumableSourceFact::Value(value) => Some(value.as_str()),
                    ConsumableSourceFact::Missing
                    | ConsumableSourceFact::Null
                    | ConsumableSourceFact::Unsupported(_) => None,
                },
                resolved_target,
            )?;
            let content_identity =
                authored_content
                    .documents
                    .first()
                    .map(|document| ConsumableContentIdentity {
                        content_key: document.id.content_key.as_str().to_string(),
                        content_hash: document.content_hash.as_str().to_string(),
                    });
            let mut target = target;
            if let ConsumableEntityTarget::ParentOwned {
                content_identity: identity,
                ..
            } = &mut target
            {
                *identity = content_identity.map_or(FactValue::Missing, FactValue::Value);
            }
            let spell_reuse = compare_spell_child(
                &candidate,
                &owner_key,
                loaded,
                resolved_key.as_ref(),
                &targets,
                index,
            )?;
            loaded
                .facts
                .consumable_occurrences
                .entities
                .push(ConsumableEntity {
                    id: entity_id.clone(),
                    owner_record_key: owner_key.clone(),
                    target,
                });
            loaded
                .facts
                .consumable_occurrences
                .occurrences
                .push(ConsumableOccurrence {
                    id: occurrence_id,
                    source_id: map_fact(&candidate.source_id, |value| {
                        ConsumableSourceId::new(value.clone())
                            .map_err(|_| unsupported_string(value))
                    }),
                    identity_stability,
                    owner_record_key: owner_key.clone(),
                    entity_id,
                    authored_order: candidate.authored_order,
                    source_path: candidate.source_path.clone(),
                    source_sort: source_presence_fact(&item.sort),
                    source_image: source_presence_fact(&item.image),
                    source_folder: source_presence_fact(&item.folder),
                    contextual_name: item.name.clone(),
                    locator,
                    state: local_state,
                    spell_reuse,
                    authored_content,
                    unsupported_content: definition_source
                        .unsupported_content
                        .iter()
                        .map(convert_unsupported)
                        .collect(),
                });
        }
    }
    Ok(())
}

fn convert_definition(source: &ConsumableItemSource) -> Result<ConsumableDefinition, String> {
    Ok(ConsumableDefinition {
        slug: map_fact(&source.slug, |value| Ok(value.clone())),
        level: map_fact(&source.level, |value| Ok(*value)),
        category: map_fact(&source.category, |value| Ok(value.clone())),
        rarity: map_fact(&source.rarity, |value| {
            Rarity::from_canonical(value).ok_or_else(|| unsupported_string(value))
        }),
        traits: map_fact(&source.traits, |value| Ok(value.clone())),
        other_tags: map_fact(&source.other_tags, |value| Ok(value.clone())),
        base_item: map_fact(&source.base_item, |value| Ok(value.clone())),
        bulk: map_fact(&source.bulk, |value| {
            ConsumableExactDecimal::new(value.clone()).map_err(|_| unsupported_string(value))
        }),
        size: map_fact(&source.size, |value| Ok(value.clone())),
        stack_group: map_fact(&source.stack_group, |value| Ok(value.clone())),
        material: map_fact(&source.material, |value| Ok(convert_material(value))),
        price: map_fact(&source.price, |value| Ok(convert_price(value))),
        usage: map_fact(&source.usage, |value| Ok(value.clone())),
        maximum_uses: map_fact(&source.maximum_uses, |value| Ok(*value)),
        auto_destroy: map_fact(&source.auto_destroy, |value| Ok(*value)),
        maximum_hp: map_fact(&source.maximum_hp, |value| Ok(*value)),
        hardness: map_fact(&source.hardness, |value| Ok(*value)),
        damage: map_fact(&source.damage, |value| Ok(convert_damage(value))),
        publication: map_fact(&source.publication, |value| Ok(convert_publication(value))),
        rules: map_fact(&source.rules, |values| {
            Ok(values.iter().map(convert_unsupported).collect())
        }),
        spell_child_id: FactValue::Missing,
    })
}

fn convert_state(source: &ConsumableItemSource) -> ConsumableSourceState {
    ConsumableSourceState {
        quantity: map_fact(&source.quantity, |value| Ok(*value)),
        current_uses: map_fact(&source.current_uses, |value| Ok(*value)),
        current_hp: map_fact(&source.current_hp, |value| Ok(*value)),
        container_id: map_fact(&source.container_id, |value| Ok(value.clone())),
        equipped: map_fact(&source.equipped, |value| Ok(convert_equipped(value))),
    }
}

fn convert_equipped(source: &ConsumableEquippedSource) -> ConsumableEquippedState {
    ConsumableEquippedState {
        carry_type: map_fact(&source.carry_type, |value| Ok(value.clone())),
        hands_held: map_fact(&source.hands_held, |value| Ok(*value)),
        in_slot: map_fact(&source.in_slot, |value| Ok(*value)),
    }
}

fn convert_material(source: &ConsumableMaterialSource) -> ConsumableMaterial {
    ConsumableMaterial {
        grade: map_fact(&source.grade, |value| Ok(value.clone())),
        material_type: map_fact(&source.material_type, |value| Ok(value.clone())),
        effects: map_fact(&source.effects, |value| Ok(value.clone())),
    }
}

fn convert_price(source: &ConsumablePriceSource) -> ConsumablePrice {
    ConsumablePrice {
        denominations: map_fact(&source.denominations, |values| {
            Ok(values
                .iter()
                .map(|(denomination, amount)| ConsumablePriceDenomination {
                    denomination: denomination.clone(),
                    amount: *amount,
                })
                .collect())
        }),
        per: map_fact(&source.per, |value| Ok(*value)),
    }
}

fn convert_damage(source: &ConsumableDamageSource) -> ConsumableDamage {
    ConsumableDamage {
        formula: map_fact(&source.formula, |value| Ok(value.clone())),
        category: map_fact(&source.category, |value| Ok(value.clone())),
        damage_type: map_fact(&source.damage_type, |value| Ok(value.clone())),
    }
}

fn convert_publication(source: &ConsumablePublicationSource) -> ConsumablePublication {
    ConsumablePublication {
        title: map_fact(&source.title, |value| Ok(value.clone())),
        license: map_fact(&source.license, |value| {
            PublicationLicense::new(value.clone()).map_err(|_| unsupported_string(value))
        }),
        remaster: map_fact(&source.remaster, |value| Ok(*value)),
    }
}

fn map_fact<T: Clone, U>(
    source: &ConsumableSourceFact<T>,
    convert: impl FnOnce(&T) -> Result<U, ConsumableUnsupportedSource>,
) -> ConsumableFact<U> {
    match source {
        ConsumableSourceFact::Missing => FactValue::Missing,
        ConsumableSourceFact::Null => FactValue::Null,
        ConsumableSourceFact::Value(value) => match convert(value) {
            Ok(value) => FactValue::Value(ConsumableSourceValue::Known(value)),
            Err(value) => FactValue::Value(ConsumableSourceValue::Unsupported(
                convert_unsupported(&value),
            )),
        },
        ConsumableSourceFact::Unsupported(value) => FactValue::Value(
            ConsumableSourceValue::Unsupported(convert_unsupported(value)),
        ),
    }
}

fn unsupported_string(value: &str) -> ConsumableUnsupportedSource {
    ConsumableUnsupportedSource {
        shape: atlas_record::UnsupportedSourceShape::String,
        value: value.to_string(),
        reason: "unsupported canonical consumable value",
    }
}

fn convert_unsupported(value: &ConsumableUnsupportedSource) -> UnsupportedSourceValue {
    UnsupportedSourceValue {
        shape: value.shape,
        value: value.value.clone(),
        reason: UnsupportedSourceReason::SourceFieldDrift,
    }
}

fn stable_occurrence_id(source_id: &str) -> Result<ConsumableOccurrenceId, String> {
    ConsumableOccurrenceId::new(format!("consumable-{}", hex(source_id.as_bytes())))
        .map_err(|_| "invalid stable consumable occurrence ID".to_string())
}

fn fallback_occurrence_id(
    owner_key: &RecordKey,
    authored_order: u32,
    allocated: &mut BTreeSet<ConsumableOccurrenceId>,
) -> Result<ConsumableOccurrenceId, String> {
    for suffix in 0_u32..=u32::MAX {
        let candidate = ConsumableOccurrenceId::new(format!(
            "consumable-fallback-{}-{authored_order}-{suffix}",
            hex(owner_key.to_string().as_bytes())
        ))
        .map_err(|_| "invalid fallback consumable occurrence ID".to_string())?;
        if !allocated.contains(&candidate) {
            return Ok(candidate);
        }
    }
    Err("consumable fallback occurrence ID space exhausted".to_string())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn convert_locator(source: &ConsumableSourceFact<String>) -> ConsumableLocatorState {
    match source {
        ConsumableSourceFact::Missing => ConsumableLocatorState::Missing,
        ConsumableSourceFact::Null => ConsumableLocatorState::Null,
        ConsumableSourceFact::Unsupported(value) => {
            ConsumableLocatorState::Unsupported(convert_unsupported(value))
        }
        ConsumableSourceFact::Value(value) => StableSourceLocator::new(value.clone()).map_or_else(
            |_| {
                ConsumableLocatorState::Unsupported(UnsupportedSourceValue {
                    shape: UnsupportedSourceShape::String,
                    value: value.clone(),
                    reason: UnsupportedSourceReason::SourceFieldDrift,
                })
            },
            ConsumableLocatorState::Known,
        ),
    }
}

fn strict_pack_id(locator: &str, index: &RecordReferenceIndex) -> Option<RecordKey> {
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

fn definition_mismatches(
    local: &ConsumableDefinition,
    target: &ConsumableDefinition,
) -> Vec<ConsumableMismatch> {
    let mut mismatches = Vec::new();
    macro_rules! compare {
        ($field:ident, $variant:ident) => {
            if local.$field != target.$field {
                mismatches.push(ConsumableMismatch {
                    field_path: concat!("definition.", stringify!($field)).to_string(),
                    local_value: ConsumableMismatchValue::$variant(local.$field.clone()),
                    target_value: ConsumableMismatchValue::$variant(target.$field.clone()),
                });
            }
        };
    }
    compare!(slug, String);
    compare!(level, Integer);
    compare!(category, String);
    compare!(rarity, Rarity);
    compare!(traits, Strings);
    compare!(other_tags, Strings);
    compare!(base_item, String);
    compare!(bulk, ExactDecimal);
    compare!(size, String);
    compare!(stack_group, String);
    compare!(material, Material);
    compare!(price, Price);
    compare!(usage, String);
    compare!(maximum_uses, Integer);
    compare!(auto_destroy, Boolean);
    compare!(maximum_hp, Integer);
    compare!(hardness, Integer);
    compare!(damage, Damage);
    compare!(publication, Publication);
    compare!(rules, Rules);
    compare!(spell_child_id, SpellChildId);
    mismatches
}

fn occurrence_content(
    loaded: &LoadedSourceRecord,
    occurrence_id: &ConsumableOccurrenceId,
    authored_order: u32,
    nested_source_id: Option<&str>,
    resolved_target: Option<&ConsumableRecord>,
) -> Result<OwnedRichContent, String> {
    let mut content = OwnedRichContent::default();
    let authored_ordinal = authored_order.to_string();
    for source in loaded
        .facts
        .source_facts
        .content_sources
        .iter()
        .filter(|source| {
            source.authored_ordinal_or_range.as_deref() == Some(authored_ordinal.as_str())
                && matches!(
                    source.source_kind,
                    atlas_record::ContentSourceKind::EmbeddedItemDescription
                        | atlas_record::ContentSourceKind::EmbeddedGmDescription
                )
        })
        .cloned()
    {
        let content_key = ContentKey::new(source.content_key.clone())
            .map_err(|_| format!("invalid consumable content key {:?}", source.content_key))?;
        let diagnostics = super::owned_content::source_diagnostics(&source);
        let duplicate_status = resolved_target
            .and_then(|target| {
                let hash = atlas_record::ContentHash::for_document(&source.document);
                target
                    .content
                    .documents
                    .iter()
                    .find(|target| {
                        target.content_hash == hash
                            && content_reference_semantics_match(&source.document, &target.document)
                    })
                    .map(
                        |target| DuplicateContentStatus::CopiedFromConsumableTarget {
                            target_record_key: target.id.parent_record_key.clone(),
                            target_content_key: target.id.content_key.clone(),
                            target_content_hash: target.content_hash.as_str().to_string(),
                        },
                    )
            })
            .unwrap_or(DuplicateContentStatus::Unique);
        content.documents.push(OwnedRichContentDocument::new(
            ContentId::new(loaded.record.identity.key.clone(), content_key),
            source.identity_stability,
            ContentOwner::ConsumableOccurrence(occurrence_id.clone()),
            ContentRole::EmbeddedCapability,
            ContentOrigin::ConsumableEmbeddedField {
                nested_source_id: nested_source_id.map(ToOwned::to_owned),
                relative_source_path: source.relative_source_path.clone(),
            },
            source.source_kind.default_visibility(),
            ContentProvenance {
                source_record_key: loaded.record.identity.key.clone(),
                relative_source_path: loaded.record.provenance.source_path.clone(),
                field_or_pointer_family: source.relative_source_path.clone(),
                nested_source_id: source.nested_source_id.clone(),
                authored_ordinal_or_range: source.authored_ordinal_or_range.clone(),
                authored_label: source.label.clone(),
            },
            source.source_kind,
            source.authored_order,
            source.label,
            source.document,
            duplicate_status,
            diagnostics,
        ));
    }
    content
        .documents
        .sort_by_key(|document| document.authored_order);
    Ok(content)
}

fn content_reference_semantics_match(
    local: &atlas_record::RichDocument,
    target: &atlas_record::RichDocument,
) -> bool {
    let local = atlas_record::iter_foundry_links(local)
        .map(|link| (&link.target, &link.label, &link.behavior))
        .collect::<Vec<_>>();
    let target = atlas_record::iter_foundry_links(target)
        .map(|link| (&link.target, &link.label, &link.behavior))
        .collect::<Vec<_>>();
    local == target
}

fn compare_spell_child(
    candidate: &ConsumableOccurrenceCandidate,
    owner_key: &RecordKey,
    loaded: &LoadedSourceRecord,
    resolved_key: Option<&RecordKey>,
    targets: &BTreeMap<RecordKey, (ConsumableRecord, Vec<atlas_record::ConsumableSpellChild>)>,
    index: &RecordReferenceIndex,
) -> Result<ConsumableSpellReuse, String> {
    let target_has_child = resolved_key
        .and_then(|record_key| targets.get(record_key))
        .is_some_and(|(_, children)| !children.is_empty());
    if let Some(reuse) = unavailable_local_spell_reuse(&candidate.spell_source, target_has_child) {
        return Ok(reuse);
    }
    let source = match &candidate.spell_source {
        LocalSpellSource::Child(source) => source.clone(),
        LocalSpellSource::Missing | LocalSpellSource::Null | LocalSpellSource::Malformed(_) => {
            return Ok(ConsumableSpellReuse::NotPresent);
        }
    };
    let child_content = actor_spell_content(
        &loaded.facts.source_facts.content_sources,
        &candidate.content_source_id,
        candidate.authored_order,
    )?
    .map(|mut content| {
        // The lookup key may be a parser-only fallback for duplicate/missing IDs.
        // Canonical provenance retains the authored ID and exact source position.
        content.nested_source_id = match &candidate.source_id {
            ConsumableSourceFact::Value(id) => Some(id.clone()),
            _ => None,
        };
        content.relative_source_path = format!(
            "$.items[{}].system.spell.system.description.value",
            candidate.authored_order
        );
        content
    });
    let local = super::spells::convert_consumable_spell_child(
        owner_key.clone(),
        &loaded.record.provenance.source_path,
        source.as_ref().clone(),
        index,
        child_content,
    )?;
    let Some(resolved_key) = resolved_key else {
        return Ok(spell_mismatch(
            ConsumableSpellMismatchReason::UnresolvedParent,
            local,
        ));
    };
    let Some((_, target_children)) = targets.get(resolved_key) else {
        return Ok(spell_mismatch(
            ConsumableSpellMismatchReason::UnresolvedParent,
            local,
        ));
    };
    let Some(target) = target_children.first() else {
        return Ok(spell_mismatch(
            ConsumableSpellMismatchReason::TargetWithoutChild,
            local,
        ));
    };
    if local.child_id != target.child_id {
        return Ok(spell_mismatch(
            ConsumableSpellMismatchReason::ChildIdentity,
            local,
        ));
    }
    if let Some(reason) = spell_definition_mismatch(&local.definition, &target.definition) {
        return Ok(spell_mismatch(reason, local));
    }
    Ok(ConsumableSpellReuse::Reused {
        target_child_id: target.child_id.clone(),
    })
}

// Source facts are scoped to one containing record. Bind both its normalized
// item identity and authored position; duplicate IDs cannot redirect the lookup.
fn actor_spell_content(
    content_sources: &[crate::records::SourceContentFact],
    item_id: &str,
    item_order: u32,
) -> Result<Option<crate::records::SourceContentFact>, String> {
    let ordinal = item_order.to_string();
    let mut matching = content_sources.iter().filter(|content| {
        content.source_kind == atlas_record::ContentSourceKind::EmbeddedSpellDescription
            && content.authored_ordinal_or_range.as_deref() == Some(ordinal.as_str())
    });
    let Some(content) = matching.next() else {
        return Ok(None);
    };
    if content.nested_source_id.as_deref() != Some(item_id) || matching.next().is_some() {
        return Err(
            "actor Spell content does not match unique containing item identity".to_string(),
        );
    }
    Ok(Some(content.clone()))
}

fn unavailable_local_spell_reuse(
    source: &LocalSpellSource,
    target_has_child: bool,
) -> Option<ConsumableSpellReuse> {
    match source {
        LocalSpellSource::Missing if target_has_child => Some(ConsumableSpellReuse::Mismatch {
            reason: ConsumableSpellMismatchReason::LocalChildMissing,
            local_evidence: ConsumableLocalSpellEvidence::Missing,
        }),
        LocalSpellSource::Null if target_has_child => Some(ConsumableSpellReuse::Mismatch {
            reason: ConsumableSpellMismatchReason::LocalChildMissing,
            local_evidence: ConsumableLocalSpellEvidence::Null,
        }),
        LocalSpellSource::Malformed(evidence) => Some(ConsumableSpellReuse::Mismatch {
            reason: ConsumableSpellMismatchReason::LocalChildMalformed,
            local_evidence: ConsumableLocalSpellEvidence::Malformed(evidence.clone()),
        }),
        LocalSpellSource::Missing | LocalSpellSource::Null | LocalSpellSource::Child(_) => None,
    }
}

fn spell_definition_mismatch(
    local: &atlas_record::SpellDefinition,
    target: &atlas_record::SpellDefinition,
) -> Option<ConsumableSpellMismatchReason> {
    if local.source_context != target.source_context {
        return Some(ConsumableSpellMismatchReason::SourceContext);
    }
    if !owned_spell_content_matches(&local.content, &target.content) {
        return Some(ConsumableSpellMismatchReason::ContentOrReferences);
    }
    if local.overlays != target.overlays {
        return Some(ConsumableSpellMismatchReason::OverlayOrFormOrder);
    }

    let mut local = local.clone();
    let mut target = target.clone();
    local.source_context = Default::default();
    target.source_context = Default::default();
    local.content = OwnedRichContent::default();
    target.content = OwnedRichContent::default();
    local.overlays = FactValue::Missing;
    target.overlays = FactValue::Missing;
    // The two snapshots originate in different containing records. Compare the
    // authenticated source contract, system version, and upstream commit while
    // deliberately excluding only that container-local path.
    target.provenance.source_path = local.provenance.source_path.clone();
    target.provenance.standalone_location = local.provenance.standalone_location.clone();

    (local != target).then_some(ConsumableSpellMismatchReason::Definition)
}

fn owned_spell_content_matches(local: &OwnedRichContent, target: &OwnedRichContent) -> bool {
    local.documents.len() == target.documents.len()
        && local
            .documents
            .iter()
            .zip(&target.documents)
            .all(|(local, target)| {
                local.id.content_key == target.id.content_key
                    && local.identity_stability == target.identity_stability
                    && local.role == target.role
                    && local.visibility == target.visibility
                    && local.source_kind == target.source_kind
                    && local.authored_order == target.authored_order
                    && local.label == target.label
                    && local.document == target.document
                    && local.content_hash == target.content_hash
                    && local.diagnostics == target.diagnostics
                    && local.reference_occurrences.len() == target.reference_occurrences.len()
                    && local
                        .reference_occurrences
                        .iter()
                        .zip(&target.reference_occurrences)
                        .all(|(local, target)| {
                            local.source_content_id.content_key
                                == target.source_content_id.content_key
                                && local.ordinal == target.ordinal
                                && local.role == target.role
                                && local.visibility == target.visibility
                                && local.target == target.target
                                && local.label == target.label
                                && local.relation_kind == target.relation_kind
                        })
            })
        && local.exclusions.len() == target.exclusions.len()
        && local
            .exclusions
            .iter()
            .zip(&target.exclusions)
            .all(|(local, target)| {
                local.content_key == target.content_key
                    && local.label == target.label
                    && local.reason == target.reason
            })
}

fn spell_mismatch(
    reason: ConsumableSpellMismatchReason,
    local: atlas_record::ConsumableSpellChild,
) -> ConsumableSpellReuse {
    ConsumableSpellReuse::Mismatch {
        reason,
        local_evidence: ConsumableLocalSpellEvidence::Child(Box::new(local)),
    }
}

fn source_presence_fact<T: Clone>(source: &SourcePresence<T>) -> ConsumableFact<T> {
    match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(value) => {
            FactValue::Value(ConsumableSourceValue::Known(value.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use atlas_domain::RecordKey;
    use atlas_record::{
        ConsumableLocalSpellEvidence, ConsumableSpellReuse, ContentIdentityStability,
        ContentSourceKind, ContentVisibility, FactValue, RichDocument, RichNode, SpellDefinition,
        SpellProvenance,
    };

    use super::{
        ConsumableSpellMismatchReason, ContentId, ContentKey, ContentOrigin, ContentOwner,
        ContentProvenance, ContentRole, DuplicateContentStatus, LocalSpellSource,
        OwnedRichContentDocument, collect_actor_consumable_candidates, fallback_occurrence_id,
        spell_definition_mismatch, stable_occurrence_id, unavailable_local_spell_reuse,
    };
    use crate::source::dto::{ConsumableSourceFact, ItemSource, parse_serialized_source_object};

    #[test]
    fn production_actor_spell_hashes_match_independent_raw_identity()
    -> Result<(), Box<dyn std::error::Error>> {
        use serde_json::{Value, json};
        use std::{collections::BTreeMap, fs};
        struct Fixture(std::path::PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let root = Fixture(std::env::temp_dir().join(format!(
            "atlas-h5-raw-child-hash-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
        )));
        fs::create_dir_all(root.0.join("packs/actors"))?;
        fs::write(
            root.0.join("module.json"),
            r#"{"packs":[{"name":"actors","label":"Actors","type":"Actor","path":"packs/actors"}]}"#,
        )?;
        let seed: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/foundry-source/spell-source-contract/packs/equipment/arboreal-wand-rank-4.json"
        ))?;
        let mut expected = BTreeMap::new();
        for kind in ["npc", "character", "hazard"] {
            let mut items = Vec::new();
            for ordinal in 0..2_u32 {
                let mut item = seed.clone();
                // NPC/Character duplicate item IDs and identical nested Spell IDs
                // must still preserve the two different authored descriptions.
                let item_id = if kind == "hazard" {
                    format!("dose{ordinal}")
                } else {
                    "sameDose".to_string()
                };
                item["_id"] = json!(item_id);
                let prose =
                    format!("<p>{kind} child <strong>{ordinal}</strong> retained narrative.</p>");
                item["system"]["spell"]["system"]["description"] = json!({"value":prose});
                let raw_child = &item["system"]["spell"];
                let parsed = crate::source::normalize::parse_foundry_content(
                    raw_child["system"]["description"]["value"]
                        .as_str()
                        .unwrap(),
                );
                let hash = atlas_record::ContentHash::for_document(&parsed.document);
                expected.insert(
                    (
                        format!("actors:{kind}"),
                        ordinal,
                        item_id,
                        raw_child["_id"].as_str().unwrap().to_string(),
                    ),
                    hash,
                );
                items.push(item);
            }
            fs::write(
                root.0.join(format!("packs/actors/{kind}.json")),
                serde_json::to_vec(&json!({
                    "_id":kind,"name":kind,"type":kind,"system":{},"items":items
                }))?,
            )?;
        }
        let loaded = crate::source_pipeline::load_foundry_source(&root.0, None)?;
        let mut actual = BTreeMap::new();
        for record in loaded.records {
            for occurrence in record.facts.consumable_occurrences.occurrences {
                let ConsumableSpellReuse::Mismatch {
                    local_evidence: ConsumableLocalSpellEvidence::Child(child),
                    ..
                } = occurrence.spell_reuse
                else {
                    panic!("unresolved authored child must retain its typed evidence");
                };
                let item_id = occurrence
                    .source_id
                    .as_value()
                    .and_then(atlas_record::ConsumableSourceValue::known)
                    .unwrap()
                    .as_str()
                    .to_string();
                let [document] = child.definition.content.documents.as_slice() else {
                    panic!("one exact authored child document");
                };
                assert_eq!(
                    document.provenance.nested_source_id.as_deref(),
                    Some(item_id.as_str())
                );
                assert_eq!(
                    document.provenance.authored_ordinal_or_range.as_deref(),
                    Some(occurrence.authored_order.to_string().as_str())
                );
                assert_eq!(
                    document.content_hash,
                    atlas_record::ContentHash::for_document(&document.document)
                );
                assert!(
                    actual
                        .insert(
                            (
                                record.record.identity.key.to_string(),
                                occurrence.authored_order,
                                item_id,
                                child.child_id.as_str().to_string()
                            ),
                            document.content_hash.clone()
                        )
                        .is_none()
                );
            }
        }
        assert_eq!(actual.len(), 6);
        assert_eq!(
            actual, expected,
            "canonical child hashes must match independently parsed raw descriptions by full containing occurrence identity"
        );
        Ok(())
    }

    #[test]
    fn actor_spell_content_requires_identity_position_and_unique_document() {
        let parsed =
            crate::source::normalize::parse_foundry_content("<p>Exact local narrative</p>");
        let document = crate::records::SourceContentFact {
            content_key: "local-spell".to_string(),
            identity_stability: atlas_record::ContentIdentityStability::StableSourceIdentity,
            source_kind: atlas_record::ContentSourceKind::EmbeddedSpellDescription,
            relative_source_path: "$.items[_id=local].system.spell.system.description.value"
                .to_string(),
            nested_source_id: Some("local".to_string()),
            authored_ordinal_or_range: Some("2".to_string()),
            authored_order: 3,
            label: Some("Local".to_string()),
            document: parsed.document,
            diagnostics: parsed.diagnostics,
        };
        let facts = [document.clone()];
        assert!(
            super::actor_spell_content(&facts, "local", 2)
                .unwrap()
                .is_some()
        );
        assert!(super::actor_spell_content(&facts, "nested-spell-id", 2).is_err());
        assert!(
            super::actor_spell_content(&facts, "local", 1)
                .unwrap()
                .is_none()
        );
        assert!(
            super::actor_spell_content(&[document.clone(), document.clone()], "local", 2).is_err()
        );
        let mut sibling = document.clone();
        sibling.authored_ordinal_or_range = Some("1".to_string());
        assert_eq!(
            super::actor_spell_content(&[sibling, document.clone()], "local", 2).unwrap(),
            Some(document)
        );
    }

    #[test]
    fn actor_collector_retains_nested_duplicate_as_typed_unsupported() {
        let root = parse_serialized_source_object(
            br#"{
              "_id":"actor",
              "name":"Actor",
              "type":"npc",
              "system":{},
              "items":[{
                "_id":"dose",
                "name":"Dose",
                "type":"consumable",
                "system":{"uses":{"max":2,"value":1,"value":2}}
              }]
            }"#,
        )
        .expect("lossless actor object");
        let candidates = collect_actor_consumable_candidates(
            &root,
            "npc",
            &RecordKey::parse("actors:dose-owner").expect("owner key"),
            "packs/actors/dose-owner.json",
        )
        .expect("nested duplicate must not discard occurrence");
        assert_eq!(candidates.len(), 1);
        let ItemSource::Consumable(item) = &candidates[0].source.source else {
            panic!("typed consumable candidate");
        };
        let definition = item.consumable.as_ref().expect("consumable definition");
        let ConsumableSourceFact::Unsupported(uses) = &definition.maximum_uses else {
            panic!("duplicate uses resource must remain unsupported once");
        };
        assert_eq!(uses.value, r#"{"max":2,"value":1,"value":2}"#);
        let ConsumableSourceFact::Unsupported(current_uses) = &definition.current_uses else {
            panic!("duplicate uses evidence must reach every affected fact");
        };
        assert_eq!(current_uses.value, r#"{"max":2,"value":1,"value":2}"#);
    }

    #[test]
    fn local_spell_absence_and_malformed_evidence_fail_closed_without_dropping_parent() {
        let root = parse_serialized_source_object(
            br#"{
              "_id":"actor","name":"Actor","type":"npc","system":{},"items":[
                {"_id":"missing","name":"Missing","type":"consumable","system":{}},
                {"_id":"null","name":"Null","type":"consumable","system":{"spell":null}},
                {"_id":"bad","name":"Bad","type":"consumable","system":{"spell":"raw"}}
              ]
            }"#,
        )
        .expect("lossless actor object");
        let candidates = collect_actor_consumable_candidates(
            &root,
            "npc",
            &RecordKey::parse("actors:spell-evidence").expect("owner key"),
            "packs/actors/spell-evidence.json",
        )
        .expect("malformed local spell remains occurrence-owned evidence");
        assert!(matches!(
            candidates[0].spell_source,
            LocalSpellSource::Missing
        ));
        assert!(matches!(candidates[1].spell_source, LocalSpellSource::Null));
        let LocalSpellSource::Malformed(ref malformed) = candidates[2].spell_source else {
            panic!("malformed local spell evidence");
        };
        assert_eq!(malformed.value, "\"raw\"");

        assert!(unavailable_local_spell_reuse(&candidates[0].spell_source, false).is_none());
        assert!(matches!(
            unavailable_local_spell_reuse(&candidates[0].spell_source, true),
            Some(ConsumableSpellReuse::Mismatch {
                reason: ConsumableSpellMismatchReason::LocalChildMissing,
                local_evidence: ConsumableLocalSpellEvidence::Missing,
            })
        ));
        assert!(matches!(
            unavailable_local_spell_reuse(&candidates[1].spell_source, true),
            Some(ConsumableSpellReuse::Mismatch {
                reason: ConsumableSpellMismatchReason::LocalChildMissing,
                local_evidence: ConsumableLocalSpellEvidence::Null,
            })
        ));
        assert!(matches!(
            unavailable_local_spell_reuse(&candidates[2].spell_source, false),
            Some(ConsumableSpellReuse::Mismatch {
                reason: ConsumableSpellMismatchReason::LocalChildMalformed,
                local_evidence: ConsumableLocalSpellEvidence::Malformed(_),
            })
        ));
    }

    #[test]
    fn fallback_occurrence_ids_do_not_collide_with_stable_or_generated_ids() {
        let owner_key = RecordKey::parse("actors:dose-owner").expect("owner key");
        let stable = stable_occurrence_id("fallback-1").expect("stable ID");
        let mut allocated = BTreeSet::from([
            stable,
            atlas_record::ConsumableOccurrenceId::new(format!(
                "consumable-fallback-{}-1-0",
                super::hex(owner_key.to_string().as_bytes())
            ))
            .expect("reserved fallback ID"),
        ]);
        let first = fallback_occurrence_id(&owner_key, 1, &mut allocated).expect("fallback ID");
        allocated.insert(first.clone());
        let second =
            fallback_occurrence_id(&owner_key, 1, &mut allocated).expect("next fallback ID");
        let prefix = format!(
            "consumable-fallback-{}-1-",
            super::hex(owner_key.to_string().as_bytes())
        );
        assert_eq!(first.as_str(), format!("{prefix}1"));
        assert_eq!(second.as_str(), format!("{prefix}2"));
        assert_ne!(first, second);
    }

    #[test]
    fn actor_collector_retains_missing_invalid_and_duplicate_source_ids() {
        let root = parse_serialized_source_object(
            br#"{
              "_id":"actor","name":"Actor","type":"npc","system":{},"items":[
                {"name":"Missing","type":"consumable","system":{}},
                {"_id":"bad id","name":"Invalid","type":"consumable","system":{}},
                {"_id":"same","name":"First","type":"consumable","system":{}},
                {"_id":"same","name":"Second","type":"consumable","system":{}}
              ]
            }"#,
        )
        .expect("lossless actor object");
        let candidates = collect_actor_consumable_candidates(
            &root,
            "npc",
            &RecordKey::parse("actors:id-owner").expect("owner key"),
            "packs/actors/id-owner.json",
        )
        .expect("malformed identities remain family-owned evidence");
        assert!(matches!(
            candidates[0].source_id,
            ConsumableSourceFact::Missing
        ));
        assert_eq!(
            candidates[1].source_id,
            ConsumableSourceFact::Value("bad id".to_string())
        );
        assert_eq!(
            candidates[2].source_id,
            ConsumableSourceFact::Value("same".to_string())
        );
        assert_eq!(candidates[2].source_id, candidates[3].source_id);
        let parser_ids = candidates
            .iter()
            .map(|candidate| candidate.source.source.source().id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(parser_ids[2..], ["same", "same"]);
        assert_ne!(parser_ids[0], parser_ids[1]);
    }

    #[test]
    fn occurrence_state_does_not_convert_missing_or_null_to_known_values() {
        let root = parse_serialized_source_object(
            br#"{"items":[{"_id":"dose","name":"Dose","type":"consumable","system":{"quantity":null}}]}"#,
        )
        .expect("lossless actor object");
        let candidates = collect_actor_consumable_candidates(
            &root,
            "character",
            &RecordKey::parse("actors:character").expect("owner key"),
            "packs/actors/character.json",
        )
        .expect("character attachment collection");
        let ItemSource::Consumable(item) = &candidates[0].source.source else {
            panic!("typed consumable candidate");
        };
        let definition = item.consumable.as_ref().expect("consumable definition");
        assert!(matches!(definition.quantity, ConsumableSourceFact::Null));
        assert!(matches!(
            definition.current_uses,
            ConsumableSourceFact::Missing
        ));
        assert!(matches!(
            super::convert_state(definition).quantity,
            FactValue::Null
        ));
        assert!(matches!(
            super::convert_state(definition).current_uses,
            FactValue::Missing
        ));
    }

    #[test]
    fn spell_snapshot_comparison_keeps_content_and_reference_evidence() {
        let local_key = RecordKey::parse("actors:local").expect("local key");
        let target_key = RecordKey::parse("equipment:target").expect("target key");
        let mut local = spell_definition("packs/actors/local.json#system.spell");
        let mut target = spell_definition("packs/equipment/target.json#system.spell");
        local.content.documents.push(spell_content_document(
            local_key.clone(),
            ContentOwner::Record(local_key),
            "Shared wording",
        ));
        target.content.documents.push(spell_content_document(
            target_key.clone(),
            ContentOwner::Record(target_key),
            "Shared wording",
        ));

        assert_eq!(spell_definition_mismatch(&local, &target), None);
        target.content.documents[0].document = RichDocument::new(vec![RichNode::Text {
            text: "Changed wording".to_string(),
        }]);
        target.content.documents[0].refresh_derived_state();
        assert_eq!(
            spell_definition_mismatch(&local, &target),
            Some(ConsumableSpellMismatchReason::ContentOrReferences)
        );
    }

    #[test]
    fn spell_snapshot_comparison_classifies_context_forms_and_definition_drift() {
        let local = spell_definition("packs/actors/local.json#system.spell");

        let mut target = spell_definition("packs/equipment/target.json#system.spell");
        target.source_context.image = FactValue::Null;
        assert_eq!(
            spell_definition_mismatch(&local, &target),
            Some(ConsumableSpellMismatchReason::SourceContext)
        );

        let mut target = spell_definition("packs/equipment/target.json#system.spell");
        target.overlays = FactValue::Null;
        assert_eq!(
            spell_definition_mismatch(&local, &target),
            Some(ConsumableSpellMismatchReason::OverlayOrFormOrder)
        );

        let mut target = spell_definition("packs/equipment/target.json#system.spell");
        target.classification = FactValue::Null;
        assert_eq!(
            spell_definition_mismatch(&local, &target),
            Some(ConsumableSpellMismatchReason::Definition)
        );
    }

    fn spell_definition(source_path: &str) -> SpellDefinition {
        SpellDefinition::new(SpellProvenance {
            source_path: source_path.to_string(),
            source_contract_version: "pf2e-serialized-source/v1".to_string(),
            source_system_version: "6.12.4".to_string(),
            source_upstream_commit: "4cbdaa37".to_string(),
            standalone_location: FactValue::Missing,
        })
    }

    fn spell_content_document(
        parent_record_key: RecordKey,
        owner: ContentOwner,
        text: &str,
    ) -> OwnedRichContentDocument {
        OwnedRichContentDocument::new(
            ContentId::new(
                parent_record_key.clone(),
                ContentKey::new("embedded-spell-description").expect("content key"),
            ),
            ContentIdentityStability::StableSourceIdentity,
            owner,
            ContentRole::EmbeddedCapability,
            ContentOrigin::RecordField {
                source_kind: ContentSourceKind::EmbeddedSpellDescription,
                relative_source_path: "system.spell.system.description.value".to_string(),
            },
            ContentVisibility::Public,
            ContentProvenance {
                source_record_key: parent_record_key,
                relative_source_path: "fixture.json".to_string(),
                field_or_pointer_family: "system.spell.system.description.value".to_string(),
                nested_source_id: Some("spell-child".to_string()),
                authored_ordinal_or_range: None,
                authored_label: None,
            },
            ContentSourceKind::EmbeddedSpellDescription,
            0,
            None,
            RichDocument::new(vec![RichNode::Text {
                text: text.to_string(),
            }]),
            DuplicateContentStatus::Unique,
            Vec::new(),
        )
    }
}
