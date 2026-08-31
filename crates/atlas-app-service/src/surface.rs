use atlas_app_model::{
    CreatureSurfaceAbilitiesView, CreatureSurfaceActionCostView, CreatureSurfaceActivityTypeView,
    CreatureSurfaceActivityView, CreatureSurfaceAwarenessView, CreatureSurfaceContentOwnerView,
    CreatureSurfaceContentProvenanceView, CreatureSurfaceContentRoleView,
    CreatureSurfaceContentView, CreatureSurfaceDamageView, CreatureSurfaceDefensesView,
    CreatureSurfaceFactOwnerView, CreatureSurfaceFactProvenanceView, CreatureSurfaceIwrView,
    CreatureSurfaceMovementView, CreatureSurfaceProvenanceView,
    CreatureSurfaceRelationshipKindView, CreatureSurfaceRelationshipTargetView,
    CreatureSurfaceRelationshipView, CreatureSurfaceResourceView, CreatureSurfaceRollView,
    CreatureSurfaceSaveView, CreatureSurfaceSavesView, CreatureSurfaceSenseView,
    CreatureSurfaceSkillView, CreatureSurfaceSourceFieldView, CreatureSurfaceSpellView,
    CreatureSurfaceSpellcastingView, CreatureSurfaceView, CreatureSurfaceVitalsView,
    EncounterRuntimeView, RecordSurfaceMetadataView, RecordSurfacePresentationView,
    RecordSurfaceProfileView, RecordSurfaceSourceView, RecordSurfaceView,
    SurfaceUnavailableReasonView, SurfaceUnavailableView,
};
use atlas_record::{
    ContentOwner, ContentRole, CreatureActionCost, CreatureCapability, CreatureDamage,
    CreatureDefenses, CreatureEmbeddedEntities, CreatureEntityOccurrence,
    CreatureEntityRelationshipKind, CreatureEntityTarget, CreatureIwr, CreatureMovementMode,
    CreatureNumber, CreatureOccurrenceParent, CreatureRecord, CreatureRelationshipTarget,
    CreatureResourceAmount, CreatureRoll, CreatureRollKind, CreatureSpellPreparation, FactValue,
    RecordBody, RetrievedRecord, SenseAcuity,
};

use crate::projection::kind_label;

pub(crate) fn record_surface(
    retrieved: &RetrievedRecord,
    profile: RecordSurfaceProfileView,
    encounter: Option<EncounterRuntimeView>,
) -> RecordSurfaceView {
    let metadata = record_metadata(retrieved);
    let presentation = match (&retrieved.record.classification.kind, &retrieved.body) {
        (atlas_domain::RecordKind::Creature, Some(RecordBody::Creature(creature))) => {
            RecordSurfacePresentationView::Creature {
                body: Box::new(creature_surface(creature, profile)),
            }
        }
        (atlas_domain::RecordKind::Creature, _) => unavailable_presentation(
            &metadata.kind,
            SurfaceUnavailableReasonView::RecordUnavailable,
            "Canonical creature data is unavailable, so the surface fails closed.",
        ),
        _ => unavailable_presentation(
            &metadata.kind,
            SurfaceUnavailableReasonView::RecordFamilyNotMigrated,
            "This record family does not yet have an approved typed app surface.",
        ),
    };
    RecordSurfaceView {
        metadata,
        profile,
        presentation,
        encounter,
    }
}

pub(crate) fn unavailable_participant_surface(
    record_key: Option<String>,
    title: String,
    kind: String,
    profile: RecordSurfaceProfileView,
    reason: SurfaceUnavailableReasonView,
    encounter: EncounterRuntimeView,
) -> RecordSurfaceView {
    let metadata = RecordSurfaceMetadataView {
        record_key,
        title,
        kind_label: kind_label(&kind),
        kind: kind.clone(),
        level: None,
        rarity: None,
        traits: Vec::new(),
        source: None,
    };
    RecordSurfaceView {
        metadata,
        profile,
        presentation: unavailable_presentation(
            &kind,
            reason,
            "No canonical record body is available for this participant.",
        ),
        encounter: Some(encounter),
    }
}

fn unavailable_presentation(
    requested_kind: &str,
    reason: SurfaceUnavailableReasonView,
    message: &str,
) -> RecordSurfacePresentationView {
    RecordSurfacePresentationView::Unavailable {
        unavailable: SurfaceUnavailableView {
            reason,
            requested_kind: requested_kind.to_string(),
            message: message.to_string(),
        },
    }
}

fn record_metadata(retrieved: &RetrievedRecord) -> RecordSurfaceMetadataView {
    let record = &retrieved.record;
    let creature_provenance = retrieved
        .body
        .as_ref()
        .map(|RecordBody::Creature(creature)| &creature.provenance);
    RecordSurfaceMetadataView {
        record_key: Some(record.identity.key.to_string()),
        title: record.identity.name.clone(),
        kind: record.classification.kind.as_str().to_string(),
        kind_label: kind_label(record.classification.kind.as_str()),
        level: record.classification.level,
        rarity: record
            .classification
            .rarity
            .map(|rarity| rarity.as_str().to_string()),
        traits: record.classification.traits.clone(),
        source: Some(RecordSurfaceSourceView {
            publication_title: record.publication.title.clone(),
            pack_label: record.foundry.pack_label.clone(),
            document_type: record.foundry.document_type.as_str().to_string(),
            record_type: record.foundry.record_type.as_str().to_string(),
            source_path: Some(record.provenance.source_path.clone()),
            source_contract_version: creature_provenance
                .map(|value| value.source_contract_version.clone()),
            source_system_version: creature_provenance
                .map(|value| value.source_system_version.clone()),
            source_upstream_commit: creature_provenance
                .map(|value| value.source_upstream_commit.clone()),
        }),
    }
}

fn creature_surface(
    creature: &CreatureRecord,
    profile: RecordSurfaceProfileView,
) -> CreatureSurfaceView {
    let detail = profile == RecordSurfaceProfileView::RecordDetail;
    let encounter = profile == RecordSurfaceProfileView::EncounterParticipant;
    let defenses = creature.defenses.value.as_value();

    CreatureSurfaceView {
        vitals: (!encounter).then(|| defenses.and_then(vitals)).flatten(),
        defenses: (!encounter).then(|| defenses.map(defenses_view)).flatten(),
        saves: detail.then(|| defenses.and_then(saves_view)).flatten(),
        awareness: (!encounter).then(|| awareness(creature)).flatten(),
        abilities: detail.then(|| abilities(creature)).flatten(),
        skills: detail.then(|| skills(creature)).flatten(),
        movement: detail.then(|| movement(creature)).flatten(),
        resources: detail.then(|| resources(creature)).flatten(),
        spellcasting: detail.then(|| spellcasting(creature)).flatten(),
        activities: detail.then(|| activities(creature)).flatten(),
        content: (detail || encounter).then(|| content(creature)).flatten(),
        relationships: (detail || encounter)
            .then(|| relationships(creature))
            .flatten(),
        provenance: Some(CreatureSurfaceProvenanceView {
            source_path: creature.provenance.source_path.clone(),
            source_contract_version: creature.provenance.source_contract_version.clone(),
            source_system_version: creature.provenance.source_system_version.clone(),
            source_upstream_commit: creature.provenance.source_upstream_commit.clone(),
        }),
    }
}

fn fact_provenance(field: CreatureSurfaceSourceFieldView) -> CreatureSurfaceFactProvenanceView {
    CreatureSurfaceFactProvenanceView {
        owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
        field,
    }
}

fn vitals(defenses: &CreatureDefenses) -> Option<CreatureSurfaceVitalsView> {
    let hp = defenses.hit_points.as_value()?;
    let hit_points = match hp.value.as_value() {
        Some(CreatureNumber::Integer(value)) => Some(*value),
        Some(CreatureNumber::Unsupported(_)) | None => hp.maximum.as_value().copied(),
    };
    Some(CreatureSurfaceVitalsView {
        hit_points,
        details: note(&hp.details),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    })
}

fn defenses_view(defenses: &CreatureDefenses) -> CreatureSurfaceDefensesView {
    let armor_class = defenses.armor_class.as_value();
    CreatureSurfaceDefensesView {
        armor_class: armor_class.and_then(|value| integer(&value.value)),
        armor_class_details: armor_class.and_then(|value| note(&value.details)),
        hardness: integer(&defenses.hardness),
        immunities: iwr(&defenses.immunities),
        resistances: iwr(&defenses.resistances),
        weaknesses: iwr(&defenses.weaknesses),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    }
}

fn iwr(values: &FactValue<Vec<CreatureIwr>>) -> Vec<CreatureSurfaceIwrView> {
    let mut projected = values
        .as_value()
        .map(|values| {
            values
                .iter()
                .map(|value| CreatureSurfaceIwrView {
                    component_id: value.id.as_str().to_string(),
                    authored_order: value.authored_order,
                    kind: value.iwr_type.as_str().to_string(),
                    amount: integer(&value.value),
                    exceptions: value
                        .exceptions
                        .as_value()
                        .map(|values| {
                            values
                                .iter()
                                .map(|value| value.as_str().to_string())
                                .collect()
                        })
                        .unwrap_or_default(),
                    double_vs: value
                        .double_vs
                        .as_value()
                        .map(|values| {
                            values
                                .iter()
                                .map(|value| value.as_str().to_string())
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    projected.sort_by_key(|value| value.authored_order);
    projected
}

fn saves_view(defenses: &CreatureDefenses) -> Option<CreatureSurfaceSavesView> {
    let saves = defenses.saves.as_value()?;
    Some(CreatureSurfaceSavesView {
        fortitude: saves.fortitude.as_value().map(save),
        reflex: saves.reflex.as_value().map(save),
        will: saves.will.as_value().map(save),
        all_saves_note: note(&defenses.all_saves_note),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    })
}

fn save(value: &atlas_record::CreatureSave) -> CreatureSurfaceSaveView {
    CreatureSurfaceSaveView {
        component_id: value.id.as_str().to_string(),
        modifier: integer(&value.value),
        details: note(&value.details),
    }
}

fn awareness(creature: &CreatureRecord) -> Option<CreatureSurfaceAwarenessView> {
    let perception = creature.perception.value.as_value()?;
    let languages = creature.languages.value.as_value();
    let mut senses = perception
        .senses
        .as_value()
        .map(|values| {
            values
                .iter()
                .map(|value| CreatureSurfaceSenseView {
                    component_id: value.id.as_str().to_string(),
                    authored_order: value.authored_order,
                    kind: value.sense_type.as_str().to_string(),
                    acuity: value.acuity.as_value().and_then(|value| match value {
                        SenseAcuity::Precise => Some("precise".to_string()),
                        SenseAcuity::Imprecise => Some("imprecise".to_string()),
                        SenseAcuity::Vague => Some("vague".to_string()),
                        SenseAcuity::Unsupported(_) => None,
                    }),
                    range_feet: integer(&value.range),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    senses.sort_by_key(|value| value.authored_order);
    Some(CreatureSurfaceAwarenessView {
        perception: integer(&perception.modifier),
        details: note(&perception.details),
        has_vision: boolean(&perception.has_vision),
        senses,
        languages: languages
            .and_then(|value| value.values.as_value())
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            })
            .unwrap_or_default(),
        language_details: languages.and_then(|value| note(&value.details)),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Perception),
    })
}

fn abilities(creature: &CreatureRecord) -> Option<CreatureSurfaceAbilitiesView> {
    let abilities = creature.legacy_abilities.value.as_value()?;
    Some(CreatureSurfaceAbilitiesView {
        strength: integer(&abilities.strength),
        dexterity: integer(&abilities.dexterity),
        constitution: integer(&abilities.constitution),
        intelligence: integer(&abilities.intelligence),
        wisdom: integer(&abilities.wisdom),
        charisma: integer(&abilities.charisma),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::LegacyAbilities),
    })
}

fn skills(creature: &CreatureRecord) -> Option<Vec<CreatureSurfaceSkillView>> {
    let mut projected = creature
        .skills
        .value
        .as_value()?
        .iter()
        .map(|value| CreatureSurfaceSkillView {
            component_id: value.id.as_str().to_string(),
            authored_order: value.authored_order,
            kind: value.kind.source_slug().to_string(),
            label: value.label.clone(),
            modifier: integer(&value.modifier),
            note: note(&value.note),
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn movement(creature: &CreatureRecord) -> Option<Vec<CreatureSurfaceMovementView>> {
    let mut projected = creature
        .movement
        .value
        .as_value()?
        .iter()
        .map(|value| CreatureSurfaceMovementView {
            component_id: value.id.as_str().to_string(),
            authored_order: value.authored_order,
            mode: match &value.mode {
                CreatureMovementMode::Land => "land",
                CreatureMovementMode::Burrow => "burrow",
                CreatureMovementMode::Climb => "climb",
                CreatureMovementMode::Fly => "fly",
                CreatureMovementMode::Swim => "swim",
                CreatureMovementMode::Unsupported(_) => "unavailable",
            }
            .to_string(),
            label: text(&value.label),
            speed_feet: integer(&value.value),
            details: note(&value.details),
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn resources(creature: &CreatureRecord) -> Option<Vec<CreatureSurfaceResourceView>> {
    let mut projected = creature
        .resources
        .value
        .as_value()?
        .iter()
        .map(|value| CreatureSurfaceResourceView {
            component_id: value.id.as_str().to_string(),
            authored_order: value.authored_order,
            kind: value.kind.as_str().to_string(),
            label: value.label.clone(),
            maximum: value.maximum.as_value().and_then(|value| match value {
                CreatureResourceAmount::Integer(value) => Some(*value),
                CreatureResourceAmount::Unsupported(_) => None,
            }),
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn activities(creature: &CreatureRecord) -> Option<Vec<CreatureSurfaceActivityView>> {
    let embedded = creature.embedded_entities.value.as_value()?;
    let mut projected = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| match &occurrence.capability {
            CreatureCapability::Strike(capability) => Some(activity(
                occurrence,
                embedded,
                CreatureSurfaceActivityTypeView::Strike,
                strings(&capability.traits),
                &capability.action_cost,
                &capability.rolls,
                &capability.damage,
            )),
            CreatureCapability::Action(capability) => Some(activity(
                occurrence,
                embedded,
                CreatureSurfaceActivityTypeView::Action,
                strings(&capability.traits),
                &capability.action_cost,
                &capability.rolls,
                &capability.damage,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn activity(
    occurrence: &CreatureEntityOccurrence,
    embedded: &CreatureEmbeddedEntities,
    activity_type: CreatureSurfaceActivityTypeView,
    traits: Vec<String>,
    action_cost_value: &CreatureActionCost,
    roll_values: &[CreatureRoll],
    damage_values: &FactValue<Vec<CreatureDamage>>,
) -> CreatureSurfaceActivityView {
    CreatureSurfaceActivityView {
        occurrence_id: occurrence.id.as_str().to_string(),
        authored_order: occurrence.authored_order,
        activity_type,
        label: occurrence_label(occurrence, embedded),
        traits,
        action_cost: action_cost(action_cost_value),
        rolls: rolls(roll_values),
        damage: damage(damage_values),
    }
}

fn spellcasting(creature: &CreatureRecord) -> Option<Vec<CreatureSurfaceSpellcastingView>> {
    let embedded = creature.embedded_entities.value.as_value()?;
    let mut projected = embedded
        .occurrences
        .iter()
        .filter_map(|entry| {
            let CreatureCapability::SpellcastingEntry(capability) = &entry.capability else {
                return None;
            };
            let mut spells = embedded
                .occurrences
                .iter()
                .filter_map(|spell| {
                    let CreatureCapability::Spell(capability) = &spell.capability else {
                        return None;
                    };
                    let CreatureOccurrenceParent::SpellcastingEntry(parent) = &spell.parent else {
                        return None;
                    };
                    if parent != &entry.id {
                        return None;
                    }
                    Some(CreatureSurfaceSpellView {
                        occurrence_id: spell.id.as_str().to_string(),
                        authored_order: spell.authored_order,
                        label: occurrence_label(spell, embedded),
                        target_record_key: match &spell.target {
                            CreatureEntityTarget::CanonicalRecord(key) => Some(key.to_string()),
                            CreatureEntityTarget::ActorOwned(_) => None,
                        },
                        rank: integer(&capability.base_rank),
                        traits: strings(&capability.traits),
                    })
                })
                .collect::<Vec<_>>();
            spells.sort_by_key(|value| value.authored_order);
            Some(CreatureSurfaceSpellcastingView {
                occurrence_id: entry.id.as_str().to_string(),
                authored_order: entry.authored_order,
                label: occurrence_label(entry, embedded),
                preparation: capability.preparation.as_value().and_then(preparation),
                tradition: text(&capability.tradition),
                attack_modifier: integer(&capability.attack),
                difficulty_class: integer(&capability.dc),
                spells,
            })
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn content(creature: &CreatureRecord) -> Option<Vec<CreatureSurfaceContentView>> {
    let mut documents = creature.content.documents.iter().collect::<Vec<_>>();
    documents.sort_by_key(|document| document.authored_order);
    non_empty(
        documents
            .into_iter()
            .map(|document| CreatureSurfaceContentView {
                content_key: document.id.content_key.as_str().to_string(),
                owner: match &document.owner {
                    ContentOwner::Record(key) => CreatureSurfaceContentOwnerView::Record {
                        record_key: key.to_string(),
                    },
                    ContentOwner::CreatureEntity(id) => CreatureSurfaceContentOwnerView::Entity {
                        entity_id: id.as_str().to_string(),
                    },
                    ContentOwner::CreatureOccurrence(id) => {
                        CreatureSurfaceContentOwnerView::Occurrence {
                            occurrence_id: id.as_str().to_string(),
                        }
                    }
                },
                role: content_role(document.role),
                authored_order: document.authored_order,
                label: document.label.clone(),
                text: atlas_record::render_plain_text(&document.document),
                content_hash: document.content_hash.as_str().to_string(),
                visibility: document.visibility.as_str().to_string(),
                provenance: CreatureSurfaceContentProvenanceView {
                    source_record_key: document.provenance.source_record_key.to_string(),
                    relative_source_path: document.provenance.relative_source_path.clone(),
                    field_family: document.provenance.field_or_pointer_family.clone(),
                    nested_source_id: document.provenance.nested_source_id.clone(),
                },
            })
            .collect(),
    )
}

fn content_role(role: ContentRole) -> CreatureSurfaceContentRoleView {
    match role {
        ContentRole::PrimaryDescription => CreatureSurfaceContentRoleView::PrimaryDescription,
        ContentRole::Summary => CreatureSurfaceContentRoleView::Summary,
        ContentRole::SupplementalRules => CreatureSurfaceContentRoleView::SupplementalRules,
        ContentRole::EmbeddedCapability => CreatureSurfaceContentRoleView::EmbeddedCapability,
        ContentRole::JournalPage => CreatureSurfaceContentRoleView::JournalPage,
        ContentRole::TableResult => CreatureSurfaceContentRoleView::TableResult,
        ContentRole::GeneratedNarrative => CreatureSurfaceContentRoleView::GeneratedNarrative,
        ContentRole::Provenance => CreatureSurfaceContentRoleView::Provenance,
    }
}

fn relationships(creature: &CreatureRecord) -> Option<Vec<CreatureSurfaceRelationshipView>> {
    let embedded = creature.embedded_entities.value.as_value()?;
    non_empty(
        embedded
            .relationships
            .iter()
            .map(|relationship| CreatureSurfaceRelationshipView {
                source_occurrence_id: relationship.source.as_str().to_string(),
                kind: match relationship.kind {
                    CreatureEntityRelationshipKind::GrantedBy => {
                        CreatureSurfaceRelationshipKindView::GrantedBy
                    }
                    CreatureEntityRelationshipKind::ItemGrant => {
                        CreatureSurfaceRelationshipKindView::ItemGrant
                    }
                    CreatureEntityRelationshipKind::LinkedWeapon => {
                        CreatureSurfaceRelationshipKindView::LinkedWeapon
                    }
                    CreatureEntityRelationshipKind::PreparedSpell => {
                        CreatureSurfaceRelationshipKindView::PreparedSpell
                    }
                },
                target: match &relationship.target {
                    CreatureRelationshipTarget::Occurrence(id) => {
                        CreatureSurfaceRelationshipTargetView::Occurrence {
                            occurrence_id: id.as_str().to_string(),
                        }
                    }
                    CreatureRelationshipTarget::UnresolvedNestedSourceId(id) => {
                        CreatureSurfaceRelationshipTargetView::UnresolvedSource {
                            source_id: id.as_str().to_string(),
                        }
                    }
                },
                contextual_label: text(&relationship.contextual_label),
                provenance_only: true,
            })
            .collect(),
    )
}

fn occurrence_label(
    occurrence: &CreatureEntityOccurrence,
    embedded: &CreatureEmbeddedEntities,
) -> String {
    if let Some(label) = occurrence.context.contextual_label.as_value() {
        return label.clone();
    }
    if let CreatureEntityTarget::ActorOwned(id) = &occurrence.target
        && let Some(entity) = embedded.entities.iter().find(|entity| &entity.id == id)
    {
        return entity.label.clone();
    }
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(key) => key.to_string(),
        CreatureEntityTarget::ActorOwned(id) => id.as_str().to_string(),
    }
}

fn action_cost(value: &CreatureActionCost) -> Option<CreatureSurfaceActionCostView> {
    match value {
        CreatureActionCost::Passive => Some(CreatureSurfaceActionCostView::Passive),
        CreatureActionCost::Reaction => Some(CreatureSurfaceActionCostView::Reaction),
        CreatureActionCost::FreeAction => Some(CreatureSurfaceActionCostView::FreeAction),
        CreatureActionCost::Actions(count) => {
            Some(CreatureSurfaceActionCostView::Actions { count: *count })
        }
        CreatureActionCost::Time(value) => Some(CreatureSurfaceActionCostView::Time {
            value: value.clone(),
        }),
        CreatureActionCost::Unsupported(_) => None,
    }
}

fn rolls(values: &[CreatureRoll]) -> Vec<CreatureSurfaceRollView> {
    values
        .iter()
        .map(|value| CreatureSurfaceRollView {
            roll_id: value.id.clone(),
            label: value.label.clone(),
            kind: match value.kind {
                CreatureRollKind::Attack => "attack",
                CreatureRollKind::DifficultyClass => "difficulty_class",
                CreatureRollKind::Check => "check",
            }
            .to_string(),
            modifier: integer(&value.value),
        })
        .collect()
}

fn damage(values: &FactValue<Vec<CreatureDamage>>) -> Vec<CreatureSurfaceDamageView> {
    values
        .as_value()
        .map(|values| {
            values
                .iter()
                .map(|value| CreatureSurfaceDamageView {
                    damage_id: value.id.clone(),
                    formula: text(&value.formula),
                    damage_type: text(&value.damage_type),
                    category: text(&value.category),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn preparation(value: &CreatureSpellPreparation) -> Option<String> {
    match value {
        CreatureSpellPreparation::Prepared => Some("prepared".to_string()),
        CreatureSpellPreparation::Spontaneous => Some("spontaneous".to_string()),
        CreatureSpellPreparation::Focus => Some("focus".to_string()),
        CreatureSpellPreparation::Innate => Some("innate".to_string()),
        CreatureSpellPreparation::Ritual => Some("ritual".to_string()),
        CreatureSpellPreparation::Unsupported(_) => None,
    }
}

fn integer(value: &FactValue<i64>) -> Option<i64> {
    value.as_value().copied()
}

fn boolean(value: &FactValue<bool>) -> Option<bool> {
    value.as_value().copied()
}

fn text(value: &FactValue<String>) -> Option<String> {
    value.as_value().cloned()
}

fn note(value: &FactValue<atlas_record::CreatureNote>) -> Option<String> {
    value.as_value().map(|value| value.as_str().to_string())
}

fn strings(value: &FactValue<Vec<String>>) -> Vec<String> {
    value.as_value().cloned().unwrap_or_default()
}

fn non_empty<T>(values: Vec<T>) -> Option<Vec<T>> {
    (!values.is_empty()).then_some(values)
}

#[cfg(test)]
mod tests {
    use super::non_empty;

    #[test]
    fn collection_projection_distinguishes_populated_from_known_empty() {
        assert_eq!(non_empty::<u8>(Vec::new()), None);
        assert_eq!(non_empty(vec![7_u8]), Some(vec![7_u8]));
    }
}
