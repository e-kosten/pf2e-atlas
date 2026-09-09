use atlas_record::{
    AtlasRecord, ConsumableEntityTarget, ConsumableOccurrenceSet, ConsumableSpellChild,
    ContentIdentityStability, ContentOwner, ContentRole, CreatureEntityRelationshipKind,
    CreatureEntityTarget, CreatureOccurrenceParent, CreatureRelationshipExecution,
    CreatureRelationshipTarget, FactValue, HazardOccurrenceIdentityStability, HazardRecord,
    HazardRelationshipKind, HazardRelationshipTarget, OccurrenceIdentityStability, RecordBody,
    RichLinkTarget, SpellStandaloneTarget,
};
use std::collections::{BTreeMap, BTreeSet};

use diesel::SqliteConnection;
use diesel::prelude::*;

use crate::IndexWriteError;
use crate::artifact::canonical_json::encode;

use super::models::{
    CanonicalConsumableEntityRow, CanonicalConsumableOccurrenceRow, CanonicalConsumableRecordRow,
    CanonicalConsumableSpellChildRow, CanonicalCreatureEntityRow, CanonicalCreatureOccurrenceRow,
    CanonicalCreatureRecordRow, CanonicalCreatureRelationshipRow, CanonicalCreatureResourceRow,
    CanonicalHazardEntityRow, CanonicalHazardOccurrenceRow, CanonicalHazardRecordRow,
    CanonicalHazardRelationshipRow, CanonicalSpellRecordRow, ConsumableQueryRecordRow,
    RecordContentExclusionRow, RecordContentRow, ReferenceOccurrenceRow, SpellDamageTypeRow,
    SpellRecordRow, SpellTraditionRow,
};

type HazardContentOwnerColumns = (&'static str, Option<String>, Option<String>, Option<String>);

pub(super) fn write_canonical_records(
    connection: &mut SqliteConnection,
    atlas_records: &[AtlasRecord],
    bodies: &[RecordBody],
    spell_children: &[ConsumableSpellChild],
    consumable_occurrence_sets: &[ConsumableOccurrenceSet],
) -> Result<(), IndexWriteError> {
    let mut records = Vec::new();
    let mut resources = Vec::new();
    let mut entities = Vec::new();
    let mut occurrences = Vec::new();
    let mut relationships = Vec::new();
    let mut hazard_records = Vec::new();
    let mut hazard_entities = Vec::new();
    let mut hazard_occurrences = Vec::new();
    let mut hazard_relationships = Vec::new();
    let mut content = Vec::new();
    let mut exclusions = Vec::new();
    let mut references = Vec::new();
    let mut spell_records = Vec::new();
    let mut spell_children_rows = Vec::new();
    let mut spell_query_rows = Vec::new();
    let mut spell_traditions = Vec::new();
    let mut spell_damage_types = Vec::new();
    let mut consumable_records = Vec::new();
    let mut consumable_entities = Vec::new();
    let mut consumable_occurrences = Vec::new();
    let mut consumable_query_records = Vec::new();

    for body in bodies {
        match body {
            RecordBody::Creature(creature) => {
                let record_key = creature.identity.record_key.to_string();
                records.push(CanonicalCreatureRecordRow {
                    record_key: record_key.clone(),
                    source_id: creature.identity.source_id.as_str().to_string(),
                    name: creature.identity.name.clone(),
                    family: "npc".to_string(),
                    canonical_json: encode(body).map_err(IndexWriteError::WriteFailed)?,
                });
                let occurrence_orders = creature
                    .embedded_entities
                    .value
                    .as_value()
                    .map(|embedded| {
                        let mut by_id = BTreeMap::<String, Vec<i64>>::new();
                        for occurrence in &embedded.occurrences {
                            by_id
                                .entry(occurrence.id.as_str().to_string())
                                .or_default()
                                .push(i64::from(occurrence.authored_order));
                        }
                        by_id
                    })
                    .unwrap_or_default();
                if let FactValue::Value(values) = &creature.resources.value {
                    for resource in values {
                        resources.push(CanonicalCreatureResourceRow {
                            record_key: record_key.clone(),
                            resource_id: resource.id.as_str().to_string(),
                            authored_order: i64::from(resource.authored_order),
                            resource_kind: resource.kind.as_str().to_string(),
                            resource_json: encode(resource)
                                .map_err(IndexWriteError::WriteFailed)?,
                        });
                    }
                }
                if let FactValue::Value(embedded) = &creature.embedded_entities.value {
                    for entity in &embedded.entities {
                        entities.push(CanonicalCreatureEntityRow {
                            record_key: record_key.clone(),
                            entity_id: entity.id.as_str().to_string(),
                            family: entity.family.as_str().to_string(),
                            label: entity.label.clone(),
                            source_identity_json: encode(&entity.source_identity)
                                .map_err(IndexWriteError::WriteFailed)?,
                        });
                    }
                    for occurrence in &embedded.occurrences {
                        let (parent_kind, parent_occurrence_id, parent_occurrence_authored_order) =
                            match &occurrence.parent {
                                CreatureOccurrenceParent::Creature => ("creature", None, None),
                                CreatureOccurrenceParent::SpellcastingEntry(parent) => (
                                    "spellcasting_entry",
                                    Some(parent.as_str().to_string()),
                                    occurrence_order(&occurrence_orders, parent.as_str(), None),
                                ),
                            };
                        let (target_kind, target_record_key, target_entity_id) =
                            match &occurrence.target {
                                CreatureEntityTarget::CanonicalRecord(key) => {
                                    ("canonical_record", Some(key.to_string()), None)
                                }
                                CreatureEntityTarget::ActorOwned(id) => {
                                    ("actor_owned", None, Some(id.as_str().to_string()))
                                }
                            };
                        occurrences.push(CanonicalCreatureOccurrenceRow {
                            record_key: record_key.clone(),
                            occurrence_id: occurrence.id.as_str().to_string(),
                            identity_stability: occurrence_stability(occurrence.identity_stability)
                                .to_string(),
                            family: occurrence.family.as_str().to_string(),
                            authored_order: i64::from(occurrence.authored_order),
                            source_sort_json: encode(&occurrence.source_sort)
                                .map_err(IndexWriteError::WriteFailed)?,
                            source_folder_json: encode(&occurrence.source_folder)
                                .map_err(IndexWriteError::WriteFailed)?,
                            source_identity_json: encode(&occurrence.source_identity)
                                .map_err(IndexWriteError::WriteFailed)?,
                            parent_kind: parent_kind.to_string(),
                            parent_occurrence_id,
                            parent_occurrence_authored_order,
                            target_kind: target_kind.to_string(),
                            target_record_key,
                            target_entity_id,
                            context_json: encode(&occurrence.context)
                                .map_err(IndexWriteError::WriteFailed)?,
                            capability_json: encode(&occurrence.capability)
                                .map_err(IndexWriteError::WriteFailed)?,
                            deltas_json: encode(&occurrence.deltas)
                                .map_err(IndexWriteError::WriteFailed)?,
                        });
                    }
                    for (order, relationship) in embedded.relationships.iter().enumerate() {
                        let source_occurrence_authored_order = required_occurrence_order(
                            &occurrence_orders,
                            relationship.source.as_str(),
                            &record_key,
                            "relationship source",
                        )?;
                        let (
                            target_kind,
                            target_occurrence_id,
                            target_occurrence_authored_order,
                            target_source_id,
                        ) = match &relationship.target {
                            CreatureRelationshipTarget::Occurrence(id) => (
                                "occurrence",
                                Some(id.as_str().to_string()),
                                Some(required_occurrence_order(
                                    &occurrence_orders,
                                    id.as_str(),
                                    &record_key,
                                    "relationship target",
                                )?),
                                None,
                            ),
                            CreatureRelationshipTarget::UnresolvedNestedSourceId(id) => (
                                "unresolved_nested_source_id",
                                None,
                                None,
                                Some(id.as_str().to_string()),
                            ),
                        };
                        relationships.push(CanonicalCreatureRelationshipRow {
                    record_key: record_key.clone(),
                    relationship_order: i64::try_from(order).map_err(|_| {
                        IndexWriteError::WriteFailed(format!(
                            "canonical relationship order is out of range for `{record_key}`"
                        ))
                    })?,
                    source_occurrence_id: relationship.source.as_str().to_string(),
                    source_occurrence_authored_order,
                    relationship_kind: relationship_kind(relationship.kind).to_string(),
                    target_kind: target_kind.to_string(),
                    target_occurrence_id,
                    target_occurrence_authored_order,
                    target_source_id,
                    source_path: relationship.source_path.clone(),
                    contextual_label_json: encode(&relationship.contextual_label)
                        .map_err(IndexWriteError::WriteFailed)?,
                    lifecycle_json: encode(&relationship.lifecycle)
                        .map_err(IndexWriteError::WriteFailed)?,
                    execution: relationship_execution(relationship.execution).to_string(),
                });
                    }
                }
                for document in &creature.content.documents {
                    let (owner_kind, owner_record_key, owner_entity_id, owner_occurrence_id) =
                        content_owner(&document.owner);
                    let owner_occurrence_authored_order =
                        owner_occurrence_id.as_deref().and_then(|id| {
                            occurrence_order(
                                &occurrence_orders,
                                id,
                                Some(i64::from(document.authored_order)),
                            )
                        });
                    let target_record =
                        |target: &RichLinkTarget| target.record_key().map(ToString::to_string);
                    content.push(RecordContentRow {
                        record_key: record_key.clone(),
                        content_key: document.id.content_key.as_str().to_string(),
                        authored_order: i64::from(document.authored_order),
                        identity_stability: content_stability(document.identity_stability)
                            .to_string(),
                        owner_kind: owner_kind.to_string(),
                        owner_record_key: owner_record_key.clone(),
                        owner_entity_id: owner_entity_id.clone(),
                        owner_occurrence_id: owner_occurrence_id.clone(),
                        owner_occurrence_authored_order,
                        owner_hazard_entity_id: None,
                        owner_hazard_occurrence_id: None,
                        owner_hazard_occurrence_authored_order: None,
                        owner_consumable_occurrence_id: None,
                        owner_consumable_occurrence_authored_order: None,
                        role: content_role(document.role).to_string(),
                        origin_json: encode(&document.origin)
                            .map_err(IndexWriteError::WriteFailed)?,
                        visibility: document.visibility.as_str().to_string(),
                        provenance_json: encode(&document.provenance)
                            .map_err(IndexWriteError::WriteFailed)?,
                        source_kind: document.source_kind.as_str().to_string(),
                        contributes_to_search: document.source_kind.default_contributes_to_search(),
                        contributes_to_references: document
                            .source_kind
                            .default_contributes_to_reference_occurrences(),
                        label: document.label.clone(),
                        content_json: encode(&document.document)
                            .map_err(IndexWriteError::WriteFailed)?,
                        content_hash: document.content_hash.as_str().to_string(),
                        duplicate_status_json: encode(&document.duplicate_status)
                            .map_err(IndexWriteError::WriteFailed)?,
                        diagnostics_json: encode(&document.diagnostics)
                            .map_err(IndexWriteError::WriteFailed)?,
                    });
                    for occurrence in &document.reference_occurrences {
                        references.push(ReferenceOccurrenceRow {
                            record_key: record_key.clone(),
                            content_key: document.id.content_key.as_str().to_string(),
                            content_authored_order: i64::from(document.authored_order),
                            occurrence_ordinal: i64::from(occurrence.ordinal),
                            owner_kind: owner_kind.to_string(),
                            owner_record_key: owner_record_key.clone(),
                            owner_entity_id: owner_entity_id.clone(),
                            owner_occurrence_id: owner_occurrence_id.clone(),
                            owner_occurrence_authored_order,
                            owner_hazard_entity_id: None,
                            owner_hazard_occurrence_id: None,
                            owner_hazard_occurrence_authored_order: None,
                            owner_consumable_occurrence_id: None,
                            owner_consumable_occurrence_authored_order: None,
                            role: content_role(occurrence.role).to_string(),
                            origin_json: encode(&occurrence.origin)
                                .map_err(IndexWriteError::WriteFailed)?,
                            visibility: occurrence.visibility.as_str().to_string(),
                            provenance_json: encode(&occurrence.provenance)
                                .map_err(IndexWriteError::WriteFailed)?,
                            target_kind: target_kind(&occurrence.target).to_string(),
                            target_record_key: target_record(&occurrence.target),
                            target_json: serde_json::to_string(&occurrence.target)
                                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                            label: occurrence.label.clone(),
                            relation_kind: occurrence.relation_kind.as_str().to_string(),
                        });
                    }
                }
                for exclusion in &creature.content.exclusions {
                    exclusions.push(RecordContentExclusionRow {
                        record_key: record_key.clone(),
                        content_key: exclusion.content_key.as_str().to_string(),
                        relative_source_path: exclusion.relative_source_path.clone(),
                        label: exclusion.label.clone(),
                        reason: match exclusion.reason {
                            atlas_record::ContentExclusionReason::DeferredEntityFamily => {
                                "deferred_entity_family"
                            }
                            atlas_record::ContentExclusionReason::MissingTypedOwner => {
                                "missing_typed_owner"
                            }
                        }
                        .to_string(),
                    });
                }
            }
            RecordBody::Hazard(hazard) => {
                let record_key = hazard.identity.record_key.to_string();
                hazard_records.push(CanonicalHazardRecordRow {
                    record_key: record_key.clone(),
                    source_id: hazard.identity.source_id.as_str().to_string(),
                    name: hazard.identity.name.clone(),
                    family: "hazard".to_string(),
                    canonical_json: encode(body).map_err(IndexWriteError::WriteFailed)?,
                });
                let mut occurrence_orders = BTreeMap::new();
                if let Some(embedded) = hazard.embedded_entities.typed() {
                    for occurrence in &embedded.occurrences {
                        if occurrence_orders
                            .insert(
                                occurrence.id.as_str().to_string(),
                                i64::from(occurrence.authored_order),
                            )
                            .is_some()
                        {
                            return Err(IndexWriteError::WriteFailed(format!(
                                "duplicate canonical hazard occurrence ID `{}` for `{record_key}`",
                                occurrence.id.as_str()
                            )));
                        }
                    }
                }
                if let Some(embedded) = hazard.embedded_entities.typed() {
                    for entity in &embedded.entities {
                        hazard_entities.push(CanonicalHazardEntityRow {
                            record_key: record_key.clone(),
                            entity_id: entity.id.as_str().to_string(),
                            family: entity.family.as_str().to_string(),
                            label: entity.label.clone(),
                            image_json: encode(&entity.image)
                                .map_err(IndexWriteError::WriteFailed)?,
                            source_identity_json: encode(&entity.source_identity)
                                .map_err(IndexWriteError::WriteFailed)?,
                            capability_json: encode(&entity.capability)
                                .map_err(IndexWriteError::WriteFailed)?,
                        });
                    }
                    for occurrence in &embedded.occurrences {
                        hazard_occurrences.push(CanonicalHazardOccurrenceRow {
                            record_key: record_key.clone(),
                            occurrence_id: occurrence.id.as_str().to_string(),
                            entity_id: occurrence.entity_id.as_str().to_string(),
                            identity_stability: hazard_occurrence_stability(
                                occurrence.identity_stability,
                            )
                            .to_string(),
                            family: occurrence.family.as_str().to_string(),
                            authored_order: i64::from(occurrence.authored_order),
                            source_sort_json: encode(&occurrence.source_sort)
                                .map_err(IndexWriteError::WriteFailed)?,
                            source_folder_json: encode(&occurrence.source_folder)
                                .map_err(IndexWriteError::WriteFailed)?,
                            source_ordinal: i64::from(occurrence.source_ordinal),
                            contextual_label_json: encode(&occurrence.contextual_label)
                                .map_err(IndexWriteError::WriteFailed)?,
                        });
                    }
                }
                for relationship in &hazard.relationships {
                    let source_occurrence_id = relationship
                        .source_occurrence_id
                        .as_ref()
                        .map(|id| id.as_str().to_string());
                    let source_occurrence_authored_order = source_occurrence_id
                        .as_deref()
                        .map(|id| {
                            required_hazard_occurrence_order(&occurrence_orders, id, &record_key)
                        })
                        .transpose()?;
                    let (target_kind, target_entity_id, target_occurrence_id, target_order) =
                        match &relationship.target {
                            HazardRelationshipTarget::Entity(id) => {
                                ("entity", Some(id.as_str().to_string()), None, None)
                            }
                            HazardRelationshipTarget::Occurrence(id) => (
                                "occurrence",
                                None,
                                Some(id.as_str().to_string()),
                                Some(required_hazard_occurrence_order(
                                    &occurrence_orders,
                                    id.as_str(),
                                    &record_key,
                                )?),
                            ),
                        };
                    hazard_relationships.push(CanonicalHazardRelationshipRow {
                        record_key: record_key.clone(),
                        relationship_id: relationship.id.as_str().to_string(),
                        authored_order: i64::from(relationship.authored_order),
                        source_occurrence_id,
                        source_occurrence_authored_order,
                        relationship_kind: hazard_relationship_kind(relationship.kind).to_string(),
                        target_kind: target_kind.to_string(),
                        target_entity_id,
                        target_occurrence_id,
                        target_occurrence_authored_order: target_order,
                    });
                }
                write_hazard_content(
                    hazard,
                    &record_key,
                    &occurrence_orders,
                    &mut content,
                    &mut exclusions,
                    &mut references,
                )?;
            }
            RecordBody::Spell(spell) => {
                let record_key = spell.identity.record_key.to_string();
                spell_records.push(CanonicalSpellRecordRow {
                    record_key: record_key.clone(),
                    source_id: spell.identity.source_id.as_str().to_string(),
                    name: spell.identity.name.clone(),
                    canonical_json: encode(body).map_err(IndexWriteError::WriteFailed)?,
                });
                append_spell_content(
                    &record_key,
                    &spell.definition.content,
                    &mut content,
                    &mut references,
                )?;
                let projection = crate::spell_query::SpellQueryProjection::from_spell(spell)
                    .map_err(IndexWriteError::WriteFailed)?;
                let traditions = projection.tradition_values();
                let damage_types = projection.damage_type_values();
                spell_query_rows.push(SpellRecordRow {
                    record_key: record_key.clone(),
                    traditions_json: serde_json::to_string(&traditions)
                        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                    spell_kinds_json: serde_json::to_string(&projection.spell_kinds)
                        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                    range_text: projection.range_text,
                    range_value: projection.range_value,
                    target_text: projection.target_text,
                    area_type: projection.area_type,
                    area_value: projection.area_value,
                    save_type: projection.save_type,
                    sustained: projection.sustained,
                    basic_save: projection.basic_save,
                    damage_types_json: serde_json::to_string(&damage_types)
                        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                    rank: projection.rank,
                    range_kind: projection.range_kind,
                    range_rule: projection.range_rule,
                });
                spell_traditions.extend(projection.traditions.into_iter().map(|value| {
                    SpellTraditionRow {
                        record_key: record_key.clone(),
                        authored_order: value.authored_order,
                        tradition: value.tradition,
                    }
                }));
                spell_damage_types.extend(projection.damage_types.into_iter().map(|value| {
                    SpellDamageTypeRow {
                        record_key: record_key.clone(),
                        damage_key: value.damage_key,
                        damage_authored_order: value.damage_authored_order,
                        type_authored_order: value.type_authored_order,
                        damage_type: value.damage_type,
                    }
                }));
            }
            RecordBody::Consumable(consumable) => {
                let record_key = consumable.identity.record_key.to_string();
                consumable_records.push(CanonicalConsumableRecordRow {
                    record_key: record_key.clone(),
                    source_id: consumable.identity.source_id.as_str().to_string(),
                    name: consumable.identity.name.clone(),
                    canonical_json: encode(body).map_err(IndexWriteError::WriteFailed)?,
                });
                let projection =
                    crate::consumable_query::project_consumable_query(&consumable.definition);
                consumable_query_records.push(ConsumableQueryRecordRow {
                    record_key: record_key.clone(),
                    category: projection.category,
                    usage: projection.usage,
                    base_item: projection.base_item,
                    bulk_value: projection.bulk_value,
                    hands_requirement: projection.hands_requirement,
                    price_cp: projection.price_cp,
                    damage_types_json: serde_json::to_string(&projection.damage_types)
                        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                });
                append_spell_content(
                    &record_key,
                    &consumable.content,
                    &mut content,
                    &mut references,
                )?;
            }
        }
    }

    for set in consumable_occurrence_sets {
        for entity in &set.entities {
            let target_record_key = match &entity.target {
                ConsumableEntityTarget::Resolved { record_key, .. } => Some(record_key.to_string()),
                ConsumableEntityTarget::ParentOwned { .. } => None,
            };
            consumable_entities.push(CanonicalConsumableEntityRow {
                owner_record_key: entity.owner_record_key.to_string(),
                entity_id: entity.id.as_str().to_string(),
                target_record_key,
                canonical_json: encode(entity).map_err(IndexWriteError::WriteFailed)?,
            });
        }
        for occurrence in &set.occurrences {
            let occurrence_order = i64::from(occurrence.authored_order);
            consumable_occurrences.push(CanonicalConsumableOccurrenceRow {
                owner_record_key: occurrence.owner_record_key.to_string(),
                occurrence_id: occurrence.id.as_str().to_string(),
                entity_id: occurrence.entity_id.as_str().to_string(),
                authored_order: occurrence_order,
                canonical_json: encode(occurrence).map_err(IndexWriteError::WriteFailed)?,
            });
            append_consumable_occurrence_content(
                occurrence,
                occurrence_order,
                &mut content,
                &mut references,
            )?;
        }
    }

    let records_by_key = atlas_records
        .iter()
        .map(|record| (record.identity.key.to_string(), record))
        .collect::<BTreeMap<_, _>>();
    let canonical_spell_keys = bodies
        .iter()
        .filter_map(|body| {
            body.as_spell()
                .map(|spell| spell.identity.record_key.to_string())
        })
        .collect::<BTreeSet<_>>();
    for child in spell_children {
        let parent_key = child.parent_record_key.to_string();
        let parent = records_by_key.get(&parent_key).ok_or_else(|| {
            IndexWriteError::WriteFailed(format!(
                "consumable spell child `{}` has no parent record `{parent_key}`",
                child.child_id.as_str()
            ))
        })?;
        if parent.foundry.record_type != atlas_record::FoundryRecordType::Consumable {
            return Err(IndexWriteError::WriteFailed(format!(
                "spell child `{}` parent `{parent_key}` is not a consumable record",
                child.child_id.as_str()
            )));
        }
        let standalone_target_record_key = match &child.standalone_target {
            FactValue::Value(SpellStandaloneTarget::Resolved(key)) => {
                if !canonical_spell_keys.contains(&key.to_string()) {
                    return Err(IndexWriteError::WriteFailed(format!(
                        "resolved spell child `{}` target `{key}` is not a canonical spell record",
                        child.child_id.as_str()
                    )));
                }
                Some(key.to_string())
            }
            FactValue::Missing
            | FactValue::Null
            | FactValue::Value(SpellStandaloneTarget::Unresolved(_)) => None,
        };
        spell_children_rows.push(CanonicalConsumableSpellChildRow {
            parent_record_key: parent_key.clone(),
            child_id: child.child_id.as_str().to_string(),
            authored_order: i64::from(child.authored_order),
            standalone_target_record_key,
            canonical_json: encode(child).map_err(IndexWriteError::WriteFailed)?,
        });
        append_spell_content(
            &parent_key,
            &child.definition.content,
            &mut content,
            &mut references,
        )?;
    }

    macro_rules! insert_rows {
        ($table:path, $rows:expr) => {
            for chunk in $rows.chunks(super::INSERT_BATCH_ROWS) {
                diesel::insert_into($table)
                    .values(chunk)
                    .execute(connection)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
            }
        };
    }
    let mut content_orders = BTreeMap::<(&str, i64), (&str, &str)>::new();
    for row in &content {
        if let Some((existing_key, existing_owner)) = content_orders.insert(
            (&row.record_key, row.authored_order),
            (&row.content_key, &row.owner_kind),
        ) {
            return Err(IndexWriteError::WriteFailed(format!(
                "record content authored order collision for `{}` at {}: `{}` ({}) and `{}` ({})",
                row.record_key,
                row.authored_order,
                existing_key,
                existing_owner,
                row.content_key,
                row.owner_kind
            )));
        }
    }
    insert_rows!(crate::schema::canonical_creature_records::table, records);
    insert_rows!(
        crate::schema::canonical_creature_resources::table,
        resources
    );
    insert_rows!(crate::schema::canonical_creature_entities::table, entities);
    insert_rows!(
        crate::schema::canonical_creature_occurrences::table,
        occurrences
    );
    insert_rows!(
        crate::schema::canonical_creature_relationships::table,
        relationships
    );
    insert_rows!(
        crate::schema::canonical_hazard_records::table,
        hazard_records
    );
    insert_rows!(
        crate::schema::canonical_hazard_entities::table,
        hazard_entities
    );
    insert_rows!(
        crate::schema::canonical_hazard_occurrences::table,
        hazard_occurrences
    );
    insert_rows!(
        crate::schema::canonical_hazard_relationships::table,
        hazard_relationships
    );
    insert_rows!(crate::schema::canonical_spell_records::table, spell_records);
    insert_rows!(
        crate::schema::canonical_consumable_records::table,
        consumable_records
    );
    insert_rows!(
        crate::schema::canonical_consumable_entities::table,
        consumable_entities
    );
    insert_rows!(
        crate::schema::canonical_consumable_occurrences::table,
        consumable_occurrences
    );
    insert_rows!(
        crate::schema::consumable_query_records::table,
        consumable_query_records
    );
    insert_rows!(crate::schema::spell_records::table, spell_query_rows);
    insert_rows!(crate::schema::spell_traditions::table, spell_traditions);
    insert_rows!(crate::schema::spell_damage_types::table, spell_damage_types);
    insert_rows!(
        crate::schema::canonical_consumable_spell_children::table,
        spell_children_rows
    );
    insert_rows!(crate::schema::record_content::table, content);
    insert_rows!(crate::schema::record_content_exclusions::table, exclusions);
    insert_rows!(crate::schema::reference_occurrences::table, references);
    Ok(())
}

fn append_spell_content(
    record_key: &str,
    owned: &atlas_record::OwnedRichContent,
    content: &mut Vec<RecordContentRow>,
    references: &mut Vec<ReferenceOccurrenceRow>,
) -> Result<(), IndexWriteError> {
    for document in &owned.documents {
        if document.id.parent_record_key.to_string() != record_key
            || document.owner != ContentOwner::Record(document.id.parent_record_key.clone())
        {
            return Err(IndexWriteError::WriteFailed(format!(
                "spell content `{}` is not owned by record `{record_key}`",
                document.id.content_key.as_str()
            )));
        }
        let target_record = |target: &RichLinkTarget| target.record_key().map(ToString::to_string);
        content.push(RecordContentRow {
            record_key: record_key.to_string(),
            content_key: document.id.content_key.as_str().to_string(),
            authored_order: i64::from(document.authored_order),
            identity_stability: content_stability(document.identity_stability).to_string(),
            owner_kind: "record".to_string(),
            owner_record_key: Some(record_key.to_string()),
            owner_entity_id: None,
            owner_occurrence_id: None,
            owner_occurrence_authored_order: None,
            owner_hazard_entity_id: None,
            owner_hazard_occurrence_id: None,
            owner_hazard_occurrence_authored_order: None,
            owner_consumable_occurrence_id: None,
            owner_consumable_occurrence_authored_order: None,
            role: content_role(document.role).to_string(),
            origin_json: encode(&document.origin).map_err(IndexWriteError::WriteFailed)?,
            visibility: document.visibility.as_str().to_string(),
            provenance_json: encode(&document.provenance).map_err(IndexWriteError::WriteFailed)?,
            source_kind: document.source_kind.as_str().to_string(),
            contributes_to_search: document.source_kind.default_contributes_to_search(),
            contributes_to_references: document
                .source_kind
                .default_contributes_to_reference_occurrences(),
            label: document.label.clone(),
            content_json: encode(&document.document).map_err(IndexWriteError::WriteFailed)?,
            content_hash: document.content_hash.as_str().to_string(),
            duplicate_status_json: encode(&document.duplicate_status)
                .map_err(IndexWriteError::WriteFailed)?,
            diagnostics_json: encode(&document.diagnostics)
                .map_err(IndexWriteError::WriteFailed)?,
        });
        for occurrence in &document.reference_occurrences {
            references.push(ReferenceOccurrenceRow {
                record_key: record_key.to_string(),
                content_key: document.id.content_key.as_str().to_string(),
                content_authored_order: i64::from(document.authored_order),
                occurrence_ordinal: i64::from(occurrence.ordinal),
                owner_kind: "record".to_string(),
                owner_record_key: Some(record_key.to_string()),
                owner_entity_id: None,
                owner_occurrence_id: None,
                owner_occurrence_authored_order: None,
                owner_hazard_entity_id: None,
                owner_hazard_occurrence_id: None,
                owner_hazard_occurrence_authored_order: None,
                owner_consumable_occurrence_id: None,
                owner_consumable_occurrence_authored_order: None,
                role: content_role(occurrence.role).to_string(),
                origin_json: encode(&occurrence.origin).map_err(IndexWriteError::WriteFailed)?,
                visibility: occurrence.visibility.as_str().to_string(),
                provenance_json: encode(&occurrence.provenance)
                    .map_err(IndexWriteError::WriteFailed)?,
                target_kind: target_kind(&occurrence.target).to_string(),
                target_record_key: target_record(&occurrence.target),
                target_json: serde_json::to_string(&occurrence.target)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                label: occurrence.label.clone(),
                relation_kind: occurrence.relation_kind.as_str().to_string(),
            });
        }
    }
    Ok(())
}

fn append_consumable_occurrence_content(
    occurrence: &atlas_record::ConsumableOccurrence,
    occurrence_order: i64,
    content: &mut Vec<RecordContentRow>,
    references: &mut Vec<ReferenceOccurrenceRow>,
) -> Result<(), IndexWriteError> {
    let record_key = occurrence.owner_record_key.to_string();
    for document in occurrence
        .owned_content()
        .flat_map(|content| &content.documents)
    {
        if document.id.parent_record_key != occurrence.owner_record_key
            || document.owner != ContentOwner::ConsumableOccurrence(occurrence.id.clone())
        {
            return Err(IndexWriteError::WriteFailed(format!(
                "consumable content `{}` does not match occurrence `{}` on `{record_key}`",
                document.id.content_key.as_str(),
                occurrence.id.as_str()
            )));
        }
        content.push(RecordContentRow {
            record_key: record_key.clone(),
            content_key: document.id.content_key.as_str().to_string(),
            authored_order: i64::from(document.authored_order),
            identity_stability: content_stability(document.identity_stability).to_string(),
            owner_kind: "consumable_occurrence".to_string(),
            owner_record_key: None,
            owner_entity_id: None,
            owner_occurrence_id: None,
            owner_occurrence_authored_order: None,
            owner_hazard_entity_id: None,
            owner_hazard_occurrence_id: None,
            owner_hazard_occurrence_authored_order: None,
            owner_consumable_occurrence_id: Some(occurrence.id.as_str().to_string()),
            owner_consumable_occurrence_authored_order: Some(occurrence_order),
            role: content_role(document.role).to_string(),
            origin_json: encode(&document.origin).map_err(IndexWriteError::WriteFailed)?,
            visibility: document.visibility.as_str().to_string(),
            provenance_json: encode(&document.provenance).map_err(IndexWriteError::WriteFailed)?,
            source_kind: document.source_kind.as_str().to_string(),
            contributes_to_search: document.visibility == atlas_record::ContentVisibility::Public,
            contributes_to_references: document
                .source_kind
                .default_contributes_to_reference_occurrences(),
            label: document.label.clone(),
            content_json: encode(&document.document).map_err(IndexWriteError::WriteFailed)?,
            content_hash: document.content_hash.as_str().to_string(),
            duplicate_status_json: encode(&document.duplicate_status)
                .map_err(IndexWriteError::WriteFailed)?,
            diagnostics_json: encode(&document.diagnostics)
                .map_err(IndexWriteError::WriteFailed)?,
        });
        for reference in &document.reference_occurrences {
            references.push(ReferenceOccurrenceRow {
                record_key: record_key.clone(),
                content_key: document.id.content_key.as_str().to_string(),
                content_authored_order: i64::from(document.authored_order),
                occurrence_ordinal: i64::from(reference.ordinal),
                owner_kind: "consumable_occurrence".to_string(),
                owner_record_key: None,
                owner_entity_id: None,
                owner_occurrence_id: None,
                owner_occurrence_authored_order: None,
                owner_hazard_entity_id: None,
                owner_hazard_occurrence_id: None,
                owner_hazard_occurrence_authored_order: None,
                owner_consumable_occurrence_id: Some(occurrence.id.as_str().to_string()),
                owner_consumable_occurrence_authored_order: Some(occurrence_order),
                role: content_role(reference.role).to_string(),
                origin_json: encode(&reference.origin).map_err(IndexWriteError::WriteFailed)?,
                visibility: reference.visibility.as_str().to_string(),
                provenance_json: encode(&reference.provenance)
                    .map_err(IndexWriteError::WriteFailed)?,
                target_kind: target_kind(&reference.target).to_string(),
                target_record_key: reference.target.record_key().map(ToString::to_string),
                target_json: serde_json::to_string(&reference.target)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                label: reference.label.clone(),
                relation_kind: reference.relation_kind.as_str().to_string(),
            });
        }
    }
    if occurrence
        .owned_content()
        .any(|content| !content.exclusions.is_empty())
    {
        return Err(IndexWriteError::WriteFailed(format!(
            "consumable occurrence `{}` has unpersisted content exclusions",
            occurrence.id.as_str()
        )));
    }
    Ok(())
}

fn occurrence_order(
    orders: &BTreeMap<String, Vec<i64>>,
    id: &str,
    preferred: Option<i64>,
) -> Option<i64> {
    let values = orders.get(id)?;
    preferred
        .filter(|preferred| values.contains(preferred))
        .or_else(|| (values.len() == 1).then_some(values[0]))
}

fn required_hazard_occurrence_order(
    orders: &BTreeMap<String, i64>,
    id: &str,
    record_key: &str,
) -> Result<i64, IndexWriteError> {
    orders.get(id).copied().ok_or_else(|| {
        IndexWriteError::WriteFailed(format!(
            "canonical hazard occurrence `{id}` for `{record_key}` does not resolve"
        ))
    })
}

fn write_hazard_content(
    hazard: &HazardRecord,
    record_key: &str,
    occurrence_orders: &BTreeMap<String, i64>,
    content: &mut Vec<RecordContentRow>,
    exclusions: &mut Vec<RecordContentExclusionRow>,
    references: &mut Vec<ReferenceOccurrenceRow>,
) -> Result<(), IndexWriteError> {
    for document in &hazard.content.documents {
        let (owner_kind, owner_record_key, owner_hazard_entity_id, owner_hazard_occurrence_id) =
            hazard_content_owner(&document.owner, record_key)?;
        let owner_hazard_occurrence_authored_order = owner_hazard_occurrence_id
            .as_deref()
            .map(|id| required_hazard_occurrence_order(occurrence_orders, id, record_key))
            .transpose()?;
        content.push(RecordContentRow {
            record_key: record_key.to_string(),
            content_key: document.id.content_key.as_str().to_string(),
            authored_order: i64::from(document.authored_order),
            identity_stability: content_stability(document.identity_stability).to_string(),
            owner_kind: owner_kind.to_string(),
            owner_record_key: owner_record_key.clone(),
            owner_entity_id: None,
            owner_occurrence_id: None,
            owner_occurrence_authored_order: None,
            owner_hazard_entity_id: owner_hazard_entity_id.clone(),
            owner_hazard_occurrence_id: owner_hazard_occurrence_id.clone(),
            owner_hazard_occurrence_authored_order,
            owner_consumable_occurrence_id: None,
            owner_consumable_occurrence_authored_order: None,
            role: content_role(document.role).to_string(),
            origin_json: encode(&document.origin).map_err(IndexWriteError::WriteFailed)?,
            visibility: document.visibility.as_str().to_string(),
            provenance_json: encode(&document.provenance).map_err(IndexWriteError::WriteFailed)?,
            source_kind: document.source_kind.as_str().to_string(),
            contributes_to_search: document.source_kind.default_contributes_to_search(),
            contributes_to_references: document
                .source_kind
                .default_contributes_to_reference_occurrences(),
            label: document.label.clone(),
            content_json: encode(&document.document).map_err(IndexWriteError::WriteFailed)?,
            content_hash: document.content_hash.as_str().to_string(),
            duplicate_status_json: encode(&document.duplicate_status)
                .map_err(IndexWriteError::WriteFailed)?,
            diagnostics_json: encode(&document.diagnostics)
                .map_err(IndexWriteError::WriteFailed)?,
        });
        for occurrence in &document.reference_occurrences {
            references.push(ReferenceOccurrenceRow {
                record_key: record_key.to_string(),
                content_key: document.id.content_key.as_str().to_string(),
                content_authored_order: i64::from(document.authored_order),
                occurrence_ordinal: i64::from(occurrence.ordinal),
                owner_kind: owner_kind.to_string(),
                owner_record_key: owner_record_key.clone(),
                owner_entity_id: None,
                owner_occurrence_id: None,
                owner_occurrence_authored_order: None,
                owner_hazard_entity_id: owner_hazard_entity_id.clone(),
                owner_hazard_occurrence_id: owner_hazard_occurrence_id.clone(),
                owner_hazard_occurrence_authored_order,
                owner_consumable_occurrence_id: None,
                owner_consumable_occurrence_authored_order: None,
                role: content_role(occurrence.role).to_string(),
                origin_json: encode(&occurrence.origin).map_err(IndexWriteError::WriteFailed)?,
                visibility: occurrence.visibility.as_str().to_string(),
                provenance_json: encode(&occurrence.provenance)
                    .map_err(IndexWriteError::WriteFailed)?,
                target_kind: target_kind(&occurrence.target).to_string(),
                target_record_key: occurrence.target.record_key().map(ToString::to_string),
                target_json: serde_json::to_string(&occurrence.target)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                label: occurrence.label.clone(),
                relation_kind: occurrence.relation_kind.as_str().to_string(),
            });
        }
    }
    for exclusion in &hazard.content.exclusions {
        exclusions.push(RecordContentExclusionRow {
            record_key: record_key.to_string(),
            content_key: exclusion.content_key.as_str().to_string(),
            relative_source_path: exclusion.relative_source_path.clone(),
            label: exclusion.label.clone(),
            reason: exclusion_reason(exclusion.reason).to_string(),
        });
    }
    Ok(())
}

fn hazard_content_owner(
    owner: &ContentOwner,
    record_key: &str,
) -> Result<HazardContentOwnerColumns, IndexWriteError> {
    match owner {
        ContentOwner::Record(key) => Ok(("record", Some(key.to_string()), None, None)),
        ContentOwner::HazardEntity(id) => {
            Ok(("hazard_entity", None, Some(id.as_str().to_string()), None))
        }
        ContentOwner::HazardOccurrence(id) => Ok((
            "hazard_occurrence",
            None,
            None,
            Some(id.as_str().to_string()),
        )),
        ContentOwner::CreatureEntity(_) | ContentOwner::CreatureOccurrence(_) => {
            Err(IndexWriteError::WriteFailed(format!(
                "hazard record `{record_key}` has a cross-family creature content owner"
            )))
        }
        ContentOwner::ConsumableOccurrence(_) => Err(IndexWriteError::WriteFailed(format!(
            "hazard record `{record_key}` has a cross-family consumable content owner"
        ))),
    }
}

fn exclusion_reason(value: atlas_record::ContentExclusionReason) -> &'static str {
    match value {
        atlas_record::ContentExclusionReason::DeferredEntityFamily => "deferred_entity_family",
        atlas_record::ContentExclusionReason::MissingTypedOwner => "missing_typed_owner",
    }
}

fn hazard_relationship_kind(value: HazardRelationshipKind) -> &'static str {
    match value {
        HazardRelationshipKind::Contains => "contains",
    }
}

fn hazard_occurrence_stability(value: HazardOccurrenceIdentityStability) -> &'static str {
    match value {
        HazardOccurrenceIdentityStability::StableSourceIdentity => "stable_source_identity",
        HazardOccurrenceIdentityStability::UnstableAuthoredOrdinal => "unstable_authored_ordinal",
    }
}

fn required_occurrence_order(
    orders: &BTreeMap<String, Vec<i64>>,
    id: &str,
    record_key: &str,
    role: &str,
) -> Result<i64, IndexWriteError> {
    occurrence_order(orders, id, None).ok_or_else(|| {
        IndexWriteError::WriteFailed(format!(
            "canonical {role} `{id}` for `{record_key}` does not resolve to exactly one occurrence"
        ))
    })
}

fn relationship_kind(value: CreatureEntityRelationshipKind) -> &'static str {
    match value {
        CreatureEntityRelationshipKind::GrantedBy => "granted_by",
        CreatureEntityRelationshipKind::ItemGrant => "item_grant",
        CreatureEntityRelationshipKind::LinkedWeapon => "linked_weapon",
        CreatureEntityRelationshipKind::PreparedSpell => "prepared_spell",
    }
}

fn relationship_execution(value: CreatureRelationshipExecution) -> &'static str {
    match value {
        CreatureRelationshipExecution::ProvenanceOnly => "provenance_only",
    }
}

fn occurrence_stability(value: OccurrenceIdentityStability) -> &'static str {
    match value {
        OccurrenceIdentityStability::StableNestedSourceId => "stable_nested_source_id",
        OccurrenceIdentityStability::UnstableOwnerFamilyOrdinal => "unstable_owner_family_ordinal",
    }
}

fn content_stability(value: ContentIdentityStability) -> &'static str {
    match value {
        ContentIdentityStability::StableSourceIdentity => "stable_source_identity",
        ContentIdentityStability::UnstableAuthoredOrdinal => "unstable_authored_ordinal",
    }
}

fn content_role(value: ContentRole) -> &'static str {
    match value {
        ContentRole::PrimaryDescription => "primary_description",
        ContentRole::Summary => "summary",
        ContentRole::SupplementalRules => "supplemental_rules",
        ContentRole::EmbeddedCapability => "embedded_capability",
        ContentRole::JournalPage => "journal_page",
        ContentRole::TableResult => "table_result",
        ContentRole::GeneratedNarrative => "generated_narrative",
        ContentRole::Provenance => "provenance",
    }
}

fn content_owner(
    owner: &ContentOwner,
) -> (&'static str, Option<String>, Option<String>, Option<String>) {
    match owner {
        ContentOwner::Record(key) => ("record", Some(key.to_string()), None, None),
        ContentOwner::CreatureEntity(id) => {
            ("creature_entity", None, Some(id.as_str().to_string()), None)
        }
        ContentOwner::CreatureOccurrence(id) => (
            "creature_occurrence",
            None,
            None,
            Some(id.as_str().to_string()),
        ),
        ContentOwner::HazardEntity(id) => {
            ("hazard_entity", None, Some(id.as_str().to_string()), None)
        }
        ContentOwner::HazardOccurrence(id) => (
            "hazard_occurrence",
            None,
            None,
            Some(id.as_str().to_string()),
        ),
        ContentOwner::ConsumableOccurrence(id) => (
            "consumable_occurrence",
            None,
            None,
            Some(id.as_str().to_string()),
        ),
    }
}

fn target_kind(target: &RichLinkTarget) -> &'static str {
    match target {
        RichLinkTarget::Record { .. } => "record",
        RichLinkTarget::LocalContent { .. } => "local_content",
        RichLinkTarget::External { .. } => "external",
        RichLinkTarget::Unresolved { .. } => "unresolved",
    }
}
