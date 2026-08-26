use atlas_record::{
    ContentIdentityStability, ContentOwner, ContentRole, CreatureEntityRelationshipKind,
    CreatureEntityTarget, CreatureOccurrenceParent, CreatureRelationshipExecution,
    CreatureRelationshipTarget, FactValue, OccurrenceIdentityStability, RecordBody, RichLinkTarget,
};
use std::collections::BTreeMap;

use diesel::SqliteConnection;
use diesel::prelude::*;

use crate::IndexWriteError;
use crate::artifact::canonical_json::encode;

use super::models::{
    CanonicalCreatureEntityRow, CanonicalCreatureOccurrenceRow, CanonicalCreatureRecordRow,
    CanonicalCreatureRelationshipRow, CanonicalCreatureResourceRow, RecordContentExclusionRow,
    RecordContentRow, ReferenceOccurrenceRow,
};

pub(super) fn write_canonical_records(
    connection: &mut SqliteConnection,
    bodies: &[RecordBody],
) -> Result<(), IndexWriteError> {
    let mut records = Vec::new();
    let mut resources = Vec::new();
    let mut entities = Vec::new();
    let mut occurrences = Vec::new();
    let mut relationships = Vec::new();
    let mut content = Vec::new();
    let mut exclusions = Vec::new();
    let mut references = Vec::new();

    for body in bodies {
        let RecordBody::Creature(creature) = body;
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
                    resource_json: encode(resource).map_err(IndexWriteError::WriteFailed)?,
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
                let (target_kind, target_record_key, target_entity_id) = match &occurrence.target {
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
            let owner_occurrence_authored_order = owner_occurrence_id.as_deref().and_then(|id| {
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
                identity_stability: content_stability(document.identity_stability).to_string(),
                owner_kind: owner_kind.to_string(),
                owner_record_key: owner_record_key.clone(),
                owner_entity_id: owner_entity_id.clone(),
                owner_occurrence_id: owner_occurrence_id.clone(),
                owner_occurrence_authored_order,
                role: content_role(document.role).to_string(),
                origin_json: encode(&document.origin).map_err(IndexWriteError::WriteFailed)?,
                visibility: document.visibility.as_str().to_string(),
                provenance_json: encode(&document.provenance)
                    .map_err(IndexWriteError::WriteFailed)?,
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
                    record_key: record_key.clone(),
                    content_key: document.id.content_key.as_str().to_string(),
                    content_authored_order: i64::from(document.authored_order),
                    occurrence_ordinal: i64::from(occurrence.ordinal),
                    owner_kind: owner_kind.to_string(),
                    owner_record_key: owner_record_key.clone(),
                    owner_entity_id: owner_entity_id.clone(),
                    owner_occurrence_id: owner_occurrence_id.clone(),
                    owner_occurrence_authored_order,
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
    insert_rows!(crate::schema::record_content::table, content);
    insert_rows!(crate::schema::record_content_exclusions::table, exclusions);
    insert_rows!(crate::schema::reference_occurrences::table, references);
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
