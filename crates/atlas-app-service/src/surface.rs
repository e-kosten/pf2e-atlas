use std::collections::BTreeMap;

use atlas_app_model::{
    CreatureSurfaceAbilitiesView, CreatureSurfaceActionCostView, CreatureSurfaceActivityTypeView,
    CreatureSurfaceActivityView, CreatureSurfaceAwarenessView, CreatureSurfaceContentOwnerView,
    CreatureSurfaceContentProvenanceView, CreatureSurfaceContentRoleView,
    CreatureSurfaceContentView, CreatureSurfaceDamageView, CreatureSurfaceDefensesView,
    CreatureSurfaceDomainUnavailableView, CreatureSurfaceFactOwnerView,
    CreatureSurfaceFactProvenanceView, CreatureSurfaceIwrView, CreatureSurfaceMovementView,
    CreatureSurfaceProvenanceView, CreatureSurfaceRelationshipKindView,
    CreatureSurfaceRelationshipTargetView, CreatureSurfaceRelationshipView,
    CreatureSurfaceResourceView, CreatureSurfaceRollView, CreatureSurfaceSaveView,
    CreatureSurfaceSavesView, CreatureSurfaceSenseView, CreatureSurfaceSkillView,
    CreatureSurfaceSourceFieldView, CreatureSurfaceSpellView, CreatureSurfaceSpellcastingView,
    CreatureSurfaceUnavailableCauseView, CreatureSurfaceUnavailableDomainsView,
    CreatureSurfaceUnavailableFieldView, CreatureSurfaceUnavailableStateView, CreatureSurfaceView,
    CreatureSurfaceVitalsView, EncounterRuntimeView, RecordSurfaceMetadataView,
    RecordSurfacePresentationView, RecordSurfaceProfileView, RecordSurfaceSourceView,
    RecordSurfaceView, SurfaceUnavailableReasonView, SurfaceUnavailableView,
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
    let mut unavailable = SurfaceUnavailableDomains::default();
    let defenses = match &creature.defenses.value {
        FactValue::Value(defenses) => Some(defenses),
        FactValue::Missing => {
            unavailable
                .add_selected_defenses(profile, CreatureSurfaceUnavailableStateView::Missing);
            None
        }
        FactValue::Null => {
            unavailable.add_selected_defenses(profile, CreatureSurfaceUnavailableStateView::Null);
            None
        }
    };

    let vitals = if encounter {
        None
    } else {
        defenses.and_then(|defenses| vitals(defenses, &mut unavailable))
    };
    let defenses_view = if encounter {
        None
    } else {
        defenses.map(|defenses| defenses_view(defenses, &mut unavailable))
    };
    let saves = if detail {
        defenses.and_then(|defenses| saves_view(defenses, &mut unavailable))
    } else {
        None
    };
    let awareness = (!encounter)
        .then(|| awareness(creature, &mut unavailable))
        .flatten();
    let abilities = detail
        .then(|| abilities(creature, &mut unavailable))
        .flatten();
    let skills = detail.then(|| skills(creature, &mut unavailable)).flatten();
    let movement = detail
        .then(|| movement(creature, &mut unavailable))
        .flatten();
    let resources = detail
        .then(|| resources(creature, &mut unavailable))
        .flatten();
    let spellcasting = detail
        .then(|| spellcasting(creature, &mut unavailable))
        .flatten();
    let activities = detail
        .then(|| activities(creature, &mut unavailable))
        .flatten();
    let relationships = (detail || encounter)
        .then(|| relationships(creature, &mut unavailable))
        .flatten();

    CreatureSurfaceView {
        vitals,
        defenses: defenses_view,
        saves,
        awareness,
        abilities,
        skills,
        movement,
        resources,
        spellcasting,
        activities,
        content: (detail || encounter).then(|| content(creature)).flatten(),
        relationships,
        unavailable_domains: unavailable.into_view(),
        provenance: Some(CreatureSurfaceProvenanceView {
            source_path: creature.provenance.source_path.clone(),
            source_contract_version: creature.provenance.source_contract_version.clone(),
            source_system_version: creature.provenance.source_system_version.clone(),
            source_upstream_commit: creature.provenance.source_upstream_commit.clone(),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SurfaceDomain {
    Vitals,
    Defenses,
    Saves,
    Awareness,
    Abilities,
    Skills,
    Movement,
    Resources,
    Spellcasting,
    Activities,
    Relationships,
}

#[derive(Default)]
struct SurfaceUnavailableDomains {
    causes: BTreeMap<SurfaceDomain, Vec<CreatureSurfaceUnavailableCauseView>>,
}

impl SurfaceUnavailableDomains {
    fn add(
        &mut self,
        domain: SurfaceDomain,
        state: CreatureSurfaceUnavailableStateView,
        field: CreatureSurfaceUnavailableFieldView,
        source_field: CreatureSurfaceSourceFieldView,
        component_id: Option<String>,
    ) {
        let cause = CreatureSurfaceUnavailableCauseView {
            state,
            field,
            component_id,
            provenance: fact_provenance(source_field),
            message: "This canonical field could not be projected safely; state and field are authoritative."
                .to_string(),
        };
        let causes = self.causes.entry(domain).or_default();
        if !causes.contains(&cause) {
            causes.push(cause);
            causes.sort();
        }
    }

    fn add_selected_defenses(
        &mut self,
        profile: RecordSurfaceProfileView,
        state: CreatureSurfaceUnavailableStateView,
    ) {
        if profile != RecordSurfaceProfileView::EncounterParticipant {
            self.add(
                SurfaceDomain::Vitals,
                state,
                CreatureSurfaceUnavailableFieldView::Defenses,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
            self.add(
                SurfaceDomain::Defenses,
                state,
                CreatureSurfaceUnavailableFieldView::Defenses,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
        }
        if profile == RecordSurfaceProfileView::RecordDetail {
            self.add(
                SurfaceDomain::Saves,
                state,
                CreatureSurfaceUnavailableFieldView::Defenses,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
        }
    }

    fn into_view(mut self) -> Option<CreatureSurfaceUnavailableDomainsView> {
        if self.causes.is_empty() {
            return None;
        }
        let mut take = |domain| {
            self.causes
                .remove(&domain)
                .map(|causes| CreatureSurfaceDomainUnavailableView { causes })
        };
        Some(CreatureSurfaceUnavailableDomainsView {
            vitals: take(SurfaceDomain::Vitals),
            defenses: take(SurfaceDomain::Defenses),
            saves: take(SurfaceDomain::Saves),
            awareness: take(SurfaceDomain::Awareness),
            abilities: take(SurfaceDomain::Abilities),
            skills: take(SurfaceDomain::Skills),
            movement: take(SurfaceDomain::Movement),
            resources: take(SurfaceDomain::Resources),
            spellcasting: take(SurfaceDomain::Spellcasting),
            activities: take(SurfaceDomain::Activities),
            relationships: take(SurfaceDomain::Relationships),
        })
    }
}

fn fact_provenance(field: CreatureSurfaceSourceFieldView) -> CreatureSurfaceFactProvenanceView {
    CreatureSurfaceFactProvenanceView {
        owner: CreatureSurfaceFactOwnerView::CanonicalCreature,
        field,
    }
}

fn required_fact<'a, T>(
    value: &'a FactValue<T>,
    unavailable: &mut SurfaceUnavailableDomains,
    domain: SurfaceDomain,
    field: CreatureSurfaceUnavailableFieldView,
    source_field: CreatureSurfaceSourceFieldView,
    component_id: Option<String>,
) -> Option<&'a T> {
    match value {
        FactValue::Value(value) => Some(value),
        FactValue::Missing => {
            unavailable.add(
                domain,
                CreatureSurfaceUnavailableStateView::Missing,
                field,
                source_field,
                component_id,
            );
            None
        }
        FactValue::Null => {
            unavailable.add(
                domain,
                CreatureSurfaceUnavailableStateView::Null,
                field,
                source_field,
                component_id,
            );
            None
        }
    }
}

fn unsupported(
    unavailable: &mut SurfaceUnavailableDomains,
    domain: SurfaceDomain,
    field: CreatureSurfaceUnavailableFieldView,
    source_field: CreatureSurfaceSourceFieldView,
    component_id: Option<String>,
) {
    unavailable.add(
        domain,
        CreatureSurfaceUnavailableStateView::Unsupported,
        field,
        source_field,
        component_id,
    );
}

fn vitals(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceVitalsView> {
    let hp = required_fact(
        &defenses.hit_points,
        unavailable,
        SurfaceDomain::Vitals,
        CreatureSurfaceUnavailableFieldView::HitPoints,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )?;
    let hit_points = match required_fact(
        &hp.value,
        unavailable,
        SurfaceDomain::Vitals,
        CreatureSurfaceUnavailableFieldView::HitPoints,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    ) {
        Some(CreatureNumber::Integer(value)) => Some(*value),
        Some(CreatureNumber::Unsupported(_)) => {
            unsupported(
                unavailable,
                SurfaceDomain::Vitals,
                CreatureSurfaceUnavailableFieldView::HitPoints,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            );
            None
        }
        None => None,
    };
    Some(CreatureSurfaceVitalsView {
        hit_points,
        details: note(&hp.details),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    })
}

fn defenses_view(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> CreatureSurfaceDefensesView {
    let armor_class = required_fact(
        &defenses.armor_class,
        unavailable,
        SurfaceDomain::Defenses,
        CreatureSurfaceUnavailableFieldView::ArmorClass,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    );
    CreatureSurfaceDefensesView {
        armor_class: armor_class.and_then(|value| {
            required_fact(
                &value.value,
                unavailable,
                SurfaceDomain::Defenses,
                CreatureSurfaceUnavailableFieldView::ArmorClass,
                CreatureSurfaceSourceFieldView::Defenses,
                None,
            )
            .copied()
        }),
        armor_class_details: armor_class.and_then(|value| note(&value.details)),
        hardness: integer(&defenses.hardness),
        immunities: iwr(
            &defenses.immunities,
            unavailable,
            CreatureSurfaceUnavailableFieldView::Immunities,
        ),
        resistances: iwr(
            &defenses.resistances,
            unavailable,
            CreatureSurfaceUnavailableFieldView::Resistances,
        ),
        weaknesses: iwr(
            &defenses.weaknesses,
            unavailable,
            CreatureSurfaceUnavailableFieldView::Weaknesses,
        ),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    }
}

fn iwr(
    values: &FactValue<Vec<CreatureIwr>>,
    unavailable: &mut SurfaceUnavailableDomains,
    field: CreatureSurfaceUnavailableFieldView,
) -> Vec<CreatureSurfaceIwrView> {
    let Some(values) = required_fact(
        values,
        unavailable,
        SurfaceDomain::Defenses,
        field,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    ) else {
        return Vec::new();
    };
    let mut projected = values
        .iter()
        .map(|value| {
            let component_id = value.id.as_str().to_string();
            let amount = match value.kind {
                atlas_record::CreatureIwrKind::Immunity => integer(&value.value),
                atlas_record::CreatureIwrKind::Resistance
                | atlas_record::CreatureIwrKind::Weakness => required_fact(
                    &value.value,
                    unavailable,
                    SurfaceDomain::Defenses,
                    CreatureSurfaceUnavailableFieldView::IwrAmount,
                    CreatureSurfaceSourceFieldView::Defenses,
                    Some(component_id.clone()),
                )
                .copied(),
            };
            let exceptions = required_fact(
                &value.exceptions,
                unavailable,
                SurfaceDomain::Defenses,
                CreatureSurfaceUnavailableFieldView::IwrExceptions,
                CreatureSurfaceSourceFieldView::Defenses,
                Some(component_id.clone()),
            )
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            })
            .unwrap_or_else(Vec::new);
            let double_vs = required_fact(
                &value.double_vs,
                unavailable,
                SurfaceDomain::Defenses,
                CreatureSurfaceUnavailableFieldView::IwrDoubleVs,
                CreatureSurfaceSourceFieldView::Defenses,
                Some(component_id.clone()),
            )
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            })
            .unwrap_or_else(Vec::new);
            CreatureSurfaceIwrView {
                component_id,
                authored_order: value.authored_order,
                kind: value.iwr_type.as_str().to_string(),
                amount,
                exceptions,
                double_vs,
            }
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    projected
}

fn saves_view(
    defenses: &CreatureDefenses,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceSavesView> {
    let saves = required_fact(
        &defenses.saves,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        None,
    )?;
    Some(CreatureSurfaceSavesView {
        fortitude: save_fact(&saves.fortitude, unavailable, "fortitude"),
        reflex: save_fact(&saves.reflex, unavailable, "reflex"),
        will: save_fact(&saves.will, unavailable, "will"),
        all_saves_note: note(&defenses.all_saves_note),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Defenses),
    })
}

fn save_fact(
    value: &FactValue<atlas_record::CreatureSave>,
    unavailable: &mut SurfaceUnavailableDomains,
    destination: &str,
) -> Option<CreatureSurfaceSaveView> {
    let value = required_fact(
        value,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        Some(destination.to_string()),
    )?;
    let component_id = value.id.as_str().to_string();
    let modifier = required_fact(
        &value.value,
        unavailable,
        SurfaceDomain::Saves,
        CreatureSurfaceUnavailableFieldView::Saves,
        CreatureSurfaceSourceFieldView::Defenses,
        Some(component_id.clone()),
    )
    .copied();
    Some(CreatureSurfaceSaveView {
        component_id,
        modifier,
        details: note(&value.details),
    })
}

fn awareness(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceAwarenessView> {
    let languages = required_fact(
        &creature.languages.value,
        unavailable,
        SurfaceDomain::Awareness,
        CreatureSurfaceUnavailableFieldView::Languages,
        CreatureSurfaceSourceFieldView::Languages,
        None,
    );
    let language_values = languages.and_then(|value| {
        required_fact(
            &value.values,
            unavailable,
            SurfaceDomain::Awareness,
            CreatureSurfaceUnavailableFieldView::Languages,
            CreatureSurfaceSourceFieldView::Languages,
            None,
        )
    });
    let perception = required_fact(
        &creature.perception.value,
        unavailable,
        SurfaceDomain::Awareness,
        CreatureSurfaceUnavailableFieldView::Perception,
        CreatureSurfaceSourceFieldView::Perception,
        None,
    );
    let mut senses = perception
        .and_then(|value| {
            required_fact(
                &value.senses,
                unavailable,
                SurfaceDomain::Awareness,
                CreatureSurfaceUnavailableFieldView::Senses,
                CreatureSurfaceSourceFieldView::Perception,
                None,
            )
        })
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    let component_id = value.id.as_str().to_string();
                    let acuity = match required_fact(
                        &value.acuity,
                        unavailable,
                        SurfaceDomain::Awareness,
                        CreatureSurfaceUnavailableFieldView::SenseAcuity,
                        CreatureSurfaceSourceFieldView::Perception,
                        Some(component_id.clone()),
                    ) {
                        Some(SenseAcuity::Precise) => Some("precise".to_string()),
                        Some(SenseAcuity::Imprecise) => Some("imprecise".to_string()),
                        Some(SenseAcuity::Vague) => Some("vague".to_string()),
                        Some(SenseAcuity::Unsupported(_)) => {
                            unsupported(
                                unavailable,
                                SurfaceDomain::Awareness,
                                CreatureSurfaceUnavailableFieldView::SenseAcuity,
                                CreatureSurfaceSourceFieldView::Perception,
                                Some(component_id.clone()),
                            );
                            None
                        }
                        None => None,
                    };
                    CreatureSurfaceSenseView {
                        component_id,
                        authored_order: value.authored_order,
                        kind: value.sense_type.as_str().to_string(),
                        acuity,
                        range_feet: integer(&value.range),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(Vec::new);
    senses.sort_by_key(|value| value.authored_order);
    Some(CreatureSurfaceAwarenessView {
        perception: perception.and_then(|value| {
            required_fact(
                &value.modifier,
                unavailable,
                SurfaceDomain::Awareness,
                CreatureSurfaceUnavailableFieldView::Perception,
                CreatureSurfaceSourceFieldView::Perception,
                None,
            )
            .copied()
        }),
        details: perception.and_then(|value| note(&value.details)),
        has_vision: perception.and_then(|value| boolean(&value.has_vision)),
        senses,
        languages: language_values
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            })
            .unwrap_or_else(Vec::new),
        language_details: languages.and_then(|value| note(&value.details)),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::Perception),
    })
}

fn abilities(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<CreatureSurfaceAbilitiesView> {
    let abilities = required_fact(
        &creature.legacy_abilities.value,
        unavailable,
        SurfaceDomain::Abilities,
        CreatureSurfaceUnavailableFieldView::LegacyAbilities,
        CreatureSurfaceSourceFieldView::LegacyAbilities,
        None,
    )?;
    Some(CreatureSurfaceAbilitiesView {
        strength: ability_value(&abilities.strength, unavailable, "strength"),
        dexterity: ability_value(&abilities.dexterity, unavailable, "dexterity"),
        constitution: ability_value(&abilities.constitution, unavailable, "constitution"),
        intelligence: ability_value(&abilities.intelligence, unavailable, "intelligence"),
        wisdom: ability_value(&abilities.wisdom, unavailable, "wisdom"),
        charisma: ability_value(&abilities.charisma, unavailable, "charisma"),
        provenance: fact_provenance(CreatureSurfaceSourceFieldView::LegacyAbilities),
    })
}

fn ability_value(
    value: &FactValue<i64>,
    unavailable: &mut SurfaceUnavailableDomains,
    component_id: &str,
) -> Option<i64> {
    required_fact(
        value,
        unavailable,
        SurfaceDomain::Abilities,
        CreatureSurfaceUnavailableFieldView::LegacyAbilities,
        CreatureSurfaceSourceFieldView::LegacyAbilities,
        Some(component_id.to_string()),
    )
    .copied()
}

fn skills(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceSkillView>> {
    let values = required_fact(
        &creature.skills.value,
        unavailable,
        SurfaceDomain::Skills,
        CreatureSurfaceUnavailableFieldView::Skills,
        CreatureSurfaceSourceFieldView::Skills,
        None,
    )?;
    let mut projected = values
        .iter()
        .map(|value| {
            let component_id = value.id.as_str().to_string();
            CreatureSurfaceSkillView {
                component_id: component_id.clone(),
                authored_order: value.authored_order,
                kind: value.kind.source_slug().to_string(),
                label: value.label.clone(),
                modifier: required_fact(
                    &value.modifier,
                    unavailable,
                    SurfaceDomain::Skills,
                    CreatureSurfaceUnavailableFieldView::SkillModifier,
                    CreatureSurfaceSourceFieldView::Skills,
                    Some(component_id),
                )
                .copied(),
                note: note(&value.note),
            }
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn movement(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceMovementView>> {
    let values = required_fact(
        &creature.movement.value,
        unavailable,
        SurfaceDomain::Movement,
        CreatureSurfaceUnavailableFieldView::Movement,
        CreatureSurfaceSourceFieldView::Movement,
        None,
    )?;
    let mut projected = values
        .iter()
        .filter_map(|value| {
            let component_id = value.id.as_str().to_string();
            let speed_feet = required_fact(
                &value.value,
                unavailable,
                SurfaceDomain::Movement,
                CreatureSurfaceUnavailableFieldView::MovementSpeed,
                CreatureSurfaceSourceFieldView::Movement,
                Some(component_id.clone()),
            )
            .copied();
            let mode = match &value.mode {
                CreatureMovementMode::Land => "land",
                CreatureMovementMode::Burrow => "burrow",
                CreatureMovementMode::Climb => "climb",
                CreatureMovementMode::Fly => "fly",
                CreatureMovementMode::Swim => "swim",
                CreatureMovementMode::Unsupported(_) => {
                    unsupported(
                        unavailable,
                        SurfaceDomain::Movement,
                        CreatureSurfaceUnavailableFieldView::MovementMode,
                        CreatureSurfaceSourceFieldView::Movement,
                        Some(component_id.clone()),
                    );
                    return None;
                }
            };
            Some(CreatureSurfaceMovementView {
                component_id: component_id.clone(),
                authored_order: value.authored_order,
                mode: mode.to_string(),
                label: text(&value.label),
                speed_feet,
                details: note(&value.details),
            })
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn resources(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceResourceView>> {
    let values = required_fact(
        &creature.resources.value,
        unavailable,
        SurfaceDomain::Resources,
        CreatureSurfaceUnavailableFieldView::Resources,
        CreatureSurfaceSourceFieldView::Resources,
        None,
    )?;
    let mut projected = values
        .iter()
        .map(|value| {
            let component_id = value.id.as_str().to_string();
            let maximum = match required_fact(
                &value.maximum,
                unavailable,
                SurfaceDomain::Resources,
                CreatureSurfaceUnavailableFieldView::ResourceMaximum,
                CreatureSurfaceSourceFieldView::Resources,
                Some(component_id.clone()),
            ) {
                Some(CreatureResourceAmount::Integer(value)) => Some(*value),
                Some(CreatureResourceAmount::Unsupported(_)) => {
                    unsupported(
                        unavailable,
                        SurfaceDomain::Resources,
                        CreatureSurfaceUnavailableFieldView::ResourceMaximum,
                        CreatureSurfaceSourceFieldView::Resources,
                        Some(component_id.clone()),
                    );
                    None
                }
                None => None,
            };
            CreatureSurfaceResourceView {
                component_id,
                authored_order: value.authored_order,
                kind: value.kind.as_str().to_string(),
                label: value.label.clone(),
                maximum,
            }
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

fn activities(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceActivityView>> {
    let embedded = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Activities,
        CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    )?;
    let mut projected = embedded
        .occurrences
        .iter()
        .filter_map(|occurrence| match &occurrence.capability {
            CreatureCapability::Strike(capability) => Some(activity(
                occurrence,
                embedded,
                ActivityProjection {
                    activity_type: CreatureSurfaceActivityTypeView::Strike,
                    traits: &capability.traits,
                    action_cost: &capability.action_cost,
                    rolls: &capability.rolls,
                    damage: &capability.damage,
                },
                unavailable,
            )),
            CreatureCapability::Action(capability) => Some(activity(
                occurrence,
                embedded,
                ActivityProjection {
                    activity_type: CreatureSurfaceActivityTypeView::Action,
                    traits: &capability.traits,
                    action_cost: &capability.action_cost,
                    rolls: &capability.rolls,
                    damage: &capability.damage,
                },
                unavailable,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    projected.sort_by_key(|value| value.authored_order);
    non_empty(projected)
}

struct ActivityProjection<'a> {
    activity_type: CreatureSurfaceActivityTypeView,
    traits: &'a FactValue<Vec<String>>,
    action_cost: &'a CreatureActionCost,
    rolls: &'a [CreatureRoll],
    damage: &'a FactValue<Vec<CreatureDamage>>,
}

fn activity(
    occurrence: &CreatureEntityOccurrence,
    embedded: &CreatureEmbeddedEntities,
    projection: ActivityProjection<'_>,
    unavailable: &mut SurfaceUnavailableDomains,
) -> CreatureSurfaceActivityView {
    let component_id = occurrence.id.as_str().to_string();
    let traits = required_fact(
        projection.traits,
        unavailable,
        SurfaceDomain::Activities,
        CreatureSurfaceUnavailableFieldView::ActivityTraits,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        Some(component_id.clone()),
    )
    .cloned()
    .unwrap_or_default();
    CreatureSurfaceActivityView {
        occurrence_id: component_id.clone(),
        authored_order: occurrence.authored_order,
        activity_type: projection.activity_type,
        label: occurrence_label(occurrence, embedded),
        traits,
        action_cost: action_cost(projection.action_cost, unavailable, &component_id),
        rolls: rolls(projection.rolls, unavailable, &component_id),
        damage: damage(projection.damage, unavailable, &component_id),
    }
}

fn spellcasting(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceSpellcastingView>> {
    let embedded = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Spellcasting,
        CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    )?;
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
                    let component_id = spell.id.as_str().to_string();
                    let rank = required_fact(
                        &capability.base_rank,
                        unavailable,
                        SurfaceDomain::Spellcasting,
                        CreatureSurfaceUnavailableFieldView::SpellRank,
                        CreatureSurfaceSourceFieldView::EmbeddedEntities,
                        Some(component_id.clone()),
                    )
                    .copied();
                    let traits = required_fact(
                        &capability.traits,
                        unavailable,
                        SurfaceDomain::Spellcasting,
                        CreatureSurfaceUnavailableFieldView::SpellTraits,
                        CreatureSurfaceSourceFieldView::EmbeddedEntities,
                        Some(component_id.clone()),
                    )
                    .cloned()
                    .unwrap_or_default();
                    Some(CreatureSurfaceSpellView {
                        occurrence_id: component_id,
                        authored_order: spell.authored_order,
                        label: occurrence_label(spell, embedded),
                        target_record_key: match &spell.target {
                            CreatureEntityTarget::CanonicalRecord(key) => Some(key.to_string()),
                            CreatureEntityTarget::ActorOwned(_) => None,
                        },
                        rank,
                        traits,
                    })
                })
                .collect::<Vec<_>>();
            spells.sort_by_key(|value| value.authored_order);
            let component_id = entry.id.as_str().to_string();
            let preparation = required_fact(
                &capability.preparation,
                unavailable,
                SurfaceDomain::Spellcasting,
                CreatureSurfaceUnavailableFieldView::SpellPreparation,
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
                Some(component_id.clone()),
            )
            .and_then(|value| preparation(value, unavailable, &component_id));
            Some(CreatureSurfaceSpellcastingView {
                occurrence_id: component_id,
                authored_order: entry.authored_order,
                label: occurrence_label(entry, embedded),
                preparation,
                tradition: required_fact(
                    &capability.tradition,
                    unavailable,
                    SurfaceDomain::Spellcasting,
                    CreatureSurfaceUnavailableFieldView::SpellTradition,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(entry.id.as_str().to_string()),
                )
                .cloned(),
                attack_modifier: required_fact(
                    &capability.attack,
                    unavailable,
                    SurfaceDomain::Spellcasting,
                    CreatureSurfaceUnavailableFieldView::SpellAttack,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(entry.id.as_str().to_string()),
                )
                .copied(),
                difficulty_class: required_fact(
                    &capability.dc,
                    unavailable,
                    SurfaceDomain::Spellcasting,
                    CreatureSurfaceUnavailableFieldView::SpellDifficultyClass,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(entry.id.as_str().to_string()),
                )
                .copied(),
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

fn relationships(
    creature: &CreatureRecord,
    unavailable: &mut SurfaceUnavailableDomains,
) -> Option<Vec<CreatureSurfaceRelationshipView>> {
    let embedded = required_fact(
        &creature.embedded_entities.value,
        unavailable,
        SurfaceDomain::Relationships,
        CreatureSurfaceUnavailableFieldView::Relationships,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        None,
    )?;
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

fn action_cost(
    value: &CreatureActionCost,
    unavailable: &mut SurfaceUnavailableDomains,
    component_id: &str,
) -> Option<CreatureSurfaceActionCostView> {
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
        CreatureActionCost::Unsupported(_) => {
            unsupported(
                unavailable,
                SurfaceDomain::Activities,
                CreatureSurfaceUnavailableFieldView::ActivityActionCost,
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
                Some(component_id.to_string()),
            );
            None
        }
    }
}

fn rolls(
    values: &[CreatureRoll],
    unavailable: &mut SurfaceUnavailableDomains,
    activity_id: &str,
) -> Vec<CreatureSurfaceRollView> {
    values
        .iter()
        .map(|value| {
            let component_id = format!("{activity_id}/{}", value.id);
            CreatureSurfaceRollView {
                roll_id: value.id.clone(),
                label: value.label.clone(),
                kind: match value.kind {
                    CreatureRollKind::Attack => "attack",
                    CreatureRollKind::DifficultyClass => "difficulty_class",
                    CreatureRollKind::Check => "check",
                }
                .to_string(),
                modifier: required_fact(
                    &value.value,
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::ActivityRoll,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id),
                )
                .copied(),
            }
        })
        .collect()
}

fn damage(
    values: &FactValue<Vec<CreatureDamage>>,
    unavailable: &mut SurfaceUnavailableDomains,
    activity_id: &str,
) -> Vec<CreatureSurfaceDamageView> {
    let Some(values) = required_fact(
        values,
        unavailable,
        SurfaceDomain::Activities,
        CreatureSurfaceUnavailableFieldView::ActivityDamage,
        CreatureSurfaceSourceFieldView::EmbeddedEntities,
        Some(activity_id.to_string()),
    ) else {
        return Vec::new();
    };
    values
        .iter()
        .map(|value| {
            let component_id = format!("{activity_id}/{}", value.id);
            CreatureSurfaceDamageView {
                damage_id: value.id.clone(),
                formula: required_fact(
                    &value.formula,
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::DamageFormula,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id.clone()),
                )
                .cloned(),
                damage_type: required_fact(
                    &value.damage_type,
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::DamageType,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id),
                )
                .cloned(),
                category: text(&value.category),
            }
        })
        .collect()
}

fn preparation(
    value: &CreatureSpellPreparation,
    unavailable: &mut SurfaceUnavailableDomains,
    component_id: &str,
) -> Option<String> {
    match value {
        CreatureSpellPreparation::Prepared => Some("prepared".to_string()),
        CreatureSpellPreparation::Spontaneous => Some("spontaneous".to_string()),
        CreatureSpellPreparation::Focus => Some("focus".to_string()),
        CreatureSpellPreparation::Innate => Some("innate".to_string()),
        CreatureSpellPreparation::Ritual => Some("ritual".to_string()),
        CreatureSpellPreparation::Unsupported(_) => {
            unsupported(
                unavailable,
                SurfaceDomain::Spellcasting,
                CreatureSurfaceUnavailableFieldView::SpellPreparation,
                CreatureSurfaceSourceFieldView::EmbeddedEntities,
                Some(component_id.to_string()),
            );
            None
        }
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

fn non_empty<T>(values: Vec<T>) -> Option<Vec<T>> {
    (!values.is_empty()).then_some(values)
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, RecordId, RecordKey};
    use atlas_record::{
        CreatureArmorClass, CreatureDefenses, CreatureEmbeddedEntities, CreatureFact,
        CreatureFamily, CreatureHitPoints, CreatureIdentity, CreatureLanguages,
        CreatureLegacyAbilities, CreatureNumber, CreaturePerception, CreatureProvenance,
        CreatureSave, CreatureSaveKind, CreatureSaves, CreatureSourceField, FactValue,
        OwnedRichContent, UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
    };

    use super::{creature_surface, non_empty};
    use atlas_app_model::{
        CreatureSurfaceDomainUnavailableView, CreatureSurfaceFactOwnerView,
        CreatureSurfaceSourceFieldView, CreatureSurfaceUnavailableFieldView,
        CreatureSurfaceUnavailableStateView, RecordSurfaceProfileView,
    };

    type CauseTuple = (
        CreatureSurfaceUnavailableStateView,
        CreatureSurfaceUnavailableFieldView,
        Option<String>,
        CreatureSurfaceFactOwnerView,
        CreatureSurfaceSourceFieldView,
    );

    #[test]
    fn collection_projection_distinguishes_populated_from_known_empty() {
        assert_eq!(non_empty::<u8>(Vec::new()), None);
        assert_eq!(non_empty(vec![7_u8]), Some(vec![7_u8]));
    }

    #[test]
    fn known_empty_domains_omit_collections_without_reporting_failure() {
        let surface = creature_surface(
            &known_empty_creature(),
            RecordSurfaceProfileView::RecordDetail,
        );

        assert!(surface.skills.is_none());
        assert!(surface.movement.is_none());
        assert!(surface.resources.is_none());
        assert!(surface.spellcasting.is_none());
        assert!(surface.activities.is_none());
        assert!(surface.relationships.is_none());
        assert!(surface.unavailable_domains.is_none());
    }

    #[test]
    fn missing_and_null_roots_report_each_affected_named_domain() {
        for state in [
            CreatureSurfaceUnavailableStateView::Missing,
            CreatureSurfaceUnavailableStateView::Null,
        ] {
            let mut creature = known_empty_creature();
            creature.defenses.value = non_value(state);
            creature.perception.value = non_value(state);
            creature.languages.value = non_value(state);
            creature.skills.value = non_value(state);
            creature.legacy_abilities.value = non_value(state);
            creature.movement.value = non_value(state);
            creature.resources.value = non_value(state);
            creature.embedded_entities.value = non_value(state);

            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            let unavailable = surface
                .unavailable_domains
                .expect("non-value roots must be public typed failures");
            assert_causes(
                &unavailable.vitals,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Defenses,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                )],
            );
            assert_causes(
                &unavailable.defenses,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Defenses,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                )],
            );
            assert_causes(
                &unavailable.saves,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Defenses,
                    None,
                    CreatureSurfaceSourceFieldView::Defenses,
                )],
            );
            assert_causes(
                &unavailable.awareness,
                vec![
                    cause(
                        state,
                        CreatureSurfaceUnavailableFieldView::Perception,
                        None,
                        CreatureSurfaceSourceFieldView::Perception,
                    ),
                    cause(
                        state,
                        CreatureSurfaceUnavailableFieldView::Languages,
                        None,
                        CreatureSurfaceSourceFieldView::Languages,
                    ),
                ],
            );
            assert_causes(
                &unavailable.abilities,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::LegacyAbilities,
                    None,
                    CreatureSurfaceSourceFieldView::LegacyAbilities,
                )],
            );
            assert_causes(
                &unavailable.skills,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Skills,
                    None,
                    CreatureSurfaceSourceFieldView::Skills,
                )],
            );
            assert_causes(
                &unavailable.movement,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Movement,
                    None,
                    CreatureSurfaceSourceFieldView::Movement,
                )],
            );
            assert_causes(
                &unavailable.resources,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Resources,
                    None,
                    CreatureSurfaceSourceFieldView::Resources,
                )],
            );
            assert_causes(
                &unavailable.spellcasting,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
                    None,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            assert_causes(
                &unavailable.activities,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::EmbeddedEntities,
                    None,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
            assert_causes(
                &unavailable.relationships,
                vec![cause(
                    state,
                    CreatureSurfaceUnavailableFieldView::Relationships,
                    None,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
        }
    }

    #[test]
    fn unavailable_perception_still_projects_or_reports_independent_languages() {
        let cases = [
            (
                CreatureSurfaceUnavailableStateView::Missing,
                FactValue::Value(CreatureLanguages {
                    values: FactValue::Value(vec![
                        atlas_record::Language::new("common").expect("language"),
                    ]),
                    details: FactValue::Missing,
                }),
                vec![cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::Perception,
                    None,
                    CreatureSurfaceSourceFieldView::Perception,
                )],
                vec!["common".to_string()],
            ),
            (
                CreatureSurfaceUnavailableStateView::Null,
                FactValue::Missing,
                vec![
                    cause(
                        CreatureSurfaceUnavailableStateView::Null,
                        CreatureSurfaceUnavailableFieldView::Perception,
                        None,
                        CreatureSurfaceSourceFieldView::Perception,
                    ),
                    cause(
                        CreatureSurfaceUnavailableStateView::Missing,
                        CreatureSurfaceUnavailableFieldView::Languages,
                        None,
                        CreatureSurfaceSourceFieldView::Languages,
                    ),
                ],
                Vec::new(),
            ),
            (
                CreatureSurfaceUnavailableStateView::Missing,
                FactValue::Null,
                vec![
                    cause(
                        CreatureSurfaceUnavailableStateView::Missing,
                        CreatureSurfaceUnavailableFieldView::Perception,
                        None,
                        CreatureSurfaceSourceFieldView::Perception,
                    ),
                    cause(
                        CreatureSurfaceUnavailableStateView::Null,
                        CreatureSurfaceUnavailableFieldView::Languages,
                        None,
                        CreatureSurfaceSourceFieldView::Languages,
                    ),
                ],
                Vec::new(),
            ),
        ];

        for (perception_state, languages, expected_causes, expected_languages) in cases {
            let mut creature = known_empty_creature();
            creature.perception.value = non_value(perception_state);
            creature.languages.value = languages;
            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            assert_eq!(
                surface
                    .awareness
                    .as_ref()
                    .expect("awareness should preserve independent language payload")
                    .languages,
                expected_languages
            );
            assert_causes(
                &surface
                    .unavailable_domains
                    .expect("perception failure should remain typed")
                    .awareness,
                expected_causes,
            );
        }
    }

    #[test]
    fn populated_and_nested_unsupported_values_are_distinct_and_order_independent() {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        let defenses = creature.defenses.value.as_value().expect("defenses");
        let mut defenses = defenses.clone();
        defenses.hit_points.as_value().expect("hp");
        defenses.hit_points = FactValue::Value(CreatureHitPoints {
            value: FactValue::Value(CreatureNumber::Unsupported(unsupported_value())),
            maximum: FactValue::Value(50),
            temporary: FactValue::Missing,
            temporary_maximum: FactValue::Missing,
            details: FactValue::Missing,
        });
        defenses.saves.as_value().expect("saves");
        let mut saves = defenses.saves.as_value().expect("saves").clone();
        saves.fortitude.as_value().expect("fortitude");
        let mut fortitude = saves.fortitude.as_value().expect("fortitude").clone();
        fortitude.value = FactValue::Null;
        saves.fortitude = FactValue::Value(fortitude);
        defenses.saves = FactValue::Value(saves);
        defenses.resistances = FactValue::Value(vec![atlas_record::CreatureIwr {
            id: atlas_record::CreatureComponentId::new("fire").expect("iwr id"),
            authored_order: 0,
            kind: atlas_record::CreatureIwrKind::Resistance,
            iwr_type: atlas_record::IwrType::new("fire").expect("iwr type"),
            value: FactValue::Missing,
            exceptions: FactValue::Null,
            double_vs: FactValue::Missing,
            apply_once: FactValue::Missing,
        }]);
        creature.defenses.value = FactValue::Value(defenses);

        let perception = creature.perception.value.as_value().expect("perception");
        creature.perception.value = FactValue::Value(CreaturePerception {
            modifier: FactValue::Value(10),
            details: FactValue::Missing,
            has_vision: FactValue::Value(true),
            senses: FactValue::Value(vec![atlas_record::CreatureSense {
                id: atlas_record::CreatureComponentId::new("odd-sense").expect("sense id"),
                authored_order: 0,
                sense_type: atlas_record::SenseType::new("odd-sense").expect("sense type"),
                acuity: FactValue::Value(atlas_record::SenseAcuity::Unsupported(
                    unsupported_value(),
                )),
                range: perception
                    .senses
                    .as_value()
                    .and(Some(30))
                    .map_or(FactValue::Missing, FactValue::Value),
            }]),
        });
        creature.languages.value = FactValue::Value(CreatureLanguages {
            values: FactValue::Value(vec![
                atlas_record::Language::new("common").expect("language"),
            ]),
            details: FactValue::Missing,
        });
        creature.skills.value = FactValue::Value(vec![atlas_record::CreatureSkill {
            id: atlas_record::CreatureComponentId::new("athletics").expect("skill id"),
            authored_order: 0,
            kind: atlas_record::CreatureSkillKind::Athletics,
            label: "Athletics".to_string(),
            modifier: FactValue::Missing,
            note: FactValue::Missing,
            variants: FactValue::Value(Vec::new()),
            source_item_id: FactValue::Missing,
        }]);
        let mut abilities = creature
            .legacy_abilities
            .value
            .as_value()
            .expect("abilities")
            .clone();
        abilities.strength = FactValue::Null;
        creature.legacy_abilities.value = FactValue::Value(abilities);

        let movement = |id: &str, order, value| atlas_record::CreatureSpeed {
            id: atlas_record::CreatureComponentId::new(id).expect("speed id"),
            authored_order: order,
            mode: atlas_record::CreatureMovementMode::Unsupported(unsupported_value()),
            value,
            label: FactValue::Value(id.to_string()),
            details: FactValue::Missing,
        };
        creature.movement.value = FactValue::Value(vec![
            movement("zeta", 1, FactValue::Missing),
            movement("alpha", 0, FactValue::Null),
        ]);
        creature.resources.value = FactValue::Value(vec![atlas_record::CreatureResource {
            id: atlas_record::CreatureComponentId::new("focus").expect("resource id"),
            authored_order: 0,
            kind: atlas_record::CreatureResourceKind::new("focus").expect("resource kind"),
            label: "Focus".to_string(),
            maximum: FactValue::Value(atlas_record::CreatureResourceAmount::Unsupported(
                unsupported_value(),
            )),
            serialized_value: FactValue::Missing,
            source_drift: FactValue::Value(Vec::new()),
            current_policy: atlas_record::ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
        }]);

        let entry_id = atlas_record::CreatureOccurrenceId::new("entry").expect("entry id");
        let strike = occurrence(
            &owner,
            "strike",
            0,
            atlas_record::CreatureEntityFamily::Strike,
            atlas_record::CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::Strike(atlas_record::CreatureStrikeCapability {
                traits: FactValue::Missing,
                attack_effects: FactValue::Value(Vec::new()),
                rolls: vec![atlas_record::CreatureRoll {
                    id: "attack".to_string(),
                    label: "Attack".to_string(),
                    kind: atlas_record::CreatureRollKind::Attack,
                    value: FactValue::Null,
                    ability: FactValue::Missing,
                }],
                damage: FactValue::Value(vec![atlas_record::CreatureDamage {
                    id: "main".to_string(),
                    formula: FactValue::Missing,
                    damage_type: FactValue::Null,
                    category: FactValue::Missing,
                    kinds: FactValue::Value(Vec::new()),
                    apply_modifier: FactValue::Missing,
                }]),
                action_cost: atlas_record::CreatureActionCost::Unsupported(unsupported_value()),
                unsupported_notes: Vec::new(),
            }),
        );
        let entry = occurrence(
            &owner,
            "entry",
            1,
            atlas_record::CreatureEntityFamily::SpellcastingEntry,
            atlas_record::CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::SpellcastingEntry(
                atlas_record::CreatureSpellcastingEntryCapability {
                    preparation: FactValue::Value(
                        atlas_record::CreatureSpellPreparation::Unsupported(unsupported_value()),
                    ),
                    tradition: FactValue::Missing,
                    attack: FactValue::Null,
                    dc: FactValue::Missing,
                    slots: FactValue::Value(Vec::new()),
                    unsupported_notes: Vec::new(),
                },
            ),
        );
        let spell = occurrence(
            &owner,
            "spell",
            2,
            atlas_record::CreatureEntityFamily::Spell,
            atlas_record::CreatureOccurrenceParent::SpellcastingEntry(entry_id),
            atlas_record::CreatureCapability::Spell(atlas_record::CreatureSpellCapability {
                traits: FactValue::Missing,
                base_rank: FactValue::Null,
                signature: FactValue::Missing,
                traditions: FactValue::Missing,
                requirements: FactValue::Missing,
                cost: FactValue::Missing,
                counteraction: FactValue::Missing,
                ritual: FactValue::Missing,
                target: FactValue::Missing,
                area: FactValue::Missing,
                range: FactValue::Missing,
                time: FactValue::Missing,
                duration: FactValue::Missing,
                defense: FactValue::Missing,
                damage: FactValue::Value(Vec::new()),
                action_cost: atlas_record::CreatureActionCost::Actions(1),
                unsupported_notes: Vec::new(),
            }),
        );
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: Vec::new(),
            occurrences: vec![strike, entry, spell],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });

        let forward = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let unavailable = forward
            .unavailable_domains
            .as_ref()
            .expect("nested failures");
        assert!(
            forward
                .defenses
                .as_ref()
                .is_some_and(|value| value.resistances.len() == 1)
        );
        assert!(
            forward
                .awareness
                .as_ref()
                .is_some_and(|value| value.languages == ["common"])
        );
        assert_causes(
            &unavailable.vitals,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Unsupported,
                CreatureSurfaceUnavailableFieldView::HitPoints,
                None,
                CreatureSurfaceSourceFieldView::Defenses,
            )],
        );
        assert_causes(
            &unavailable.defenses,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::IwrAmount,
                    Some("fire"),
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::IwrExceptions,
                    Some("fire"),
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::IwrDoubleVs,
                    Some("fire"),
                    CreatureSurfaceSourceFieldView::Defenses,
                ),
            ],
        );
        assert_causes(
            &unavailable.saves,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Null,
                CreatureSurfaceUnavailableFieldView::Saves,
                Some("fortitude"),
                CreatureSurfaceSourceFieldView::Defenses,
            )],
        );
        assert_causes(
            &unavailable.awareness,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Unsupported,
                CreatureSurfaceUnavailableFieldView::SenseAcuity,
                Some("odd-sense"),
                CreatureSurfaceSourceFieldView::Perception,
            )],
        );
        assert_causes(
            &unavailable.abilities,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Null,
                CreatureSurfaceUnavailableFieldView::LegacyAbilities,
                Some("strength"),
                CreatureSurfaceSourceFieldView::LegacyAbilities,
            )],
        );
        assert_causes(
            &unavailable.skills,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Missing,
                CreatureSurfaceUnavailableFieldView::SkillModifier,
                Some("athletics"),
                CreatureSurfaceSourceFieldView::Skills,
            )],
        );
        assert_causes(
            &unavailable.movement,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::MovementSpeed,
                    Some("alpha"),
                    CreatureSurfaceSourceFieldView::Movement,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::MovementMode,
                    Some("alpha"),
                    CreatureSurfaceSourceFieldView::Movement,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::MovementSpeed,
                    Some("zeta"),
                    CreatureSurfaceSourceFieldView::Movement,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::MovementMode,
                    Some("zeta"),
                    CreatureSurfaceSourceFieldView::Movement,
                ),
            ],
        );
        assert_causes(
            &unavailable.resources,
            vec![cause(
                CreatureSurfaceUnavailableStateView::Unsupported,
                CreatureSurfaceUnavailableFieldView::ResourceMaximum,
                Some("focus"),
                CreatureSurfaceSourceFieldView::Resources,
            )],
        );
        assert_causes(
            &unavailable.spellcasting,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::SpellPreparation,
                    Some("entry"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::SpellTradition,
                    Some("entry"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::SpellAttack,
                    Some("entry"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::SpellDifficultyClass,
                    Some("entry"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::SpellTraits,
                    Some("spell"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::SpellRank,
                    Some("spell"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
            ],
        );
        assert_causes(
            &unavailable.activities,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::ActivityTraits,
                    Some("strike"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::ActivityActionCost,
                    Some("strike"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::ActivityRoll,
                    Some("strike/attack"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Missing,
                    CreatureSurfaceUnavailableFieldView::DamageFormula,
                    Some("strike/main"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Null,
                    CreatureSurfaceUnavailableFieldView::DamageType,
                    Some("strike/main"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
            ],
        );

        creature.movement.value.as_value().expect("movement");
        if let FactValue::Value(values) = &mut creature.movement.value {
            values.reverse();
        }
        if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
            embedded.occurrences.reverse();
        }
        let reverse = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            forward.unavailable_domains, reverse.unavailable_domains,
            "complete typed cause tuples must be order independent"
        );
    }

    fn known_empty_creature() -> atlas_record::CreatureRecord {
        let record_key = RecordKey::new(
            PackName::new("creatures".to_string()).expect("pack"),
            RecordId::new("surface-fixture".to_string()).expect("id"),
        );
        let save = |id: &str, kind| CreatureSave {
            id: atlas_record::CreatureComponentId::new(id).expect("save id"),
            kind,
            value: FactValue::Value(10),
            details: FactValue::Missing,
        };
        atlas_record::CreatureRecord {
            identity: CreatureIdentity {
                record_key,
                source_id: atlas_record::CreatureSourceId::new("surface-fixture")
                    .expect("source id"),
                name: "Surface Fixture".to_string(),
                family: CreatureFamily::Npc,
            },
            level: missing(CreatureSourceField::Level),
            rarity: missing(CreatureSourceField::Rarity),
            traits: missing(CreatureSourceField::Traits),
            size: missing(CreatureSourceField::Size),
            publication: missing(CreatureSourceField::Publication),
            adjustment: missing(CreatureSourceField::Adjustment),
            source_alliance: missing(CreatureSourceField::SourceAlliance),
            perception: CreatureFact::source(
                FactValue::Value(CreaturePerception {
                    modifier: FactValue::Value(10),
                    details: FactValue::Missing,
                    has_vision: FactValue::Value(true),
                    senses: FactValue::Value(Vec::new()),
                }),
                CreatureSourceField::Perception,
            ),
            initiative: missing(CreatureSourceField::Initiative),
            languages: CreatureFact::source(
                FactValue::Value(CreatureLanguages {
                    values: FactValue::Value(Vec::new()),
                    details: FactValue::Missing,
                }),
                CreatureSourceField::Languages,
            ),
            skills: CreatureFact::source(FactValue::Value(Vec::new()), CreatureSourceField::Skills),
            legacy_abilities: CreatureFact::source(
                FactValue::Value(CreatureLegacyAbilities {
                    strength: FactValue::Value(0),
                    dexterity: FactValue::Value(0),
                    constitution: FactValue::Value(0),
                    intelligence: FactValue::Value(0),
                    wisdom: FactValue::Value(0),
                    charisma: FactValue::Value(0),
                }),
                CreatureSourceField::LegacyAbilities,
            ),
            defenses: CreatureFact::source(
                FactValue::Value(CreatureDefenses {
                    armor_class: FactValue::Value(CreatureArmorClass {
                        value: FactValue::Value(20),
                        details: FactValue::Missing,
                    }),
                    hit_points: FactValue::Value(CreatureHitPoints {
                        value: FactValue::Value(CreatureNumber::Integer(50)),
                        maximum: FactValue::Value(50),
                        temporary: FactValue::Missing,
                        temporary_maximum: FactValue::Missing,
                        details: FactValue::Missing,
                    }),
                    hardness: FactValue::Missing,
                    shield: FactValue::Missing,
                    saves: FactValue::Value(CreatureSaves {
                        fortitude: FactValue::Value(save("fortitude", CreatureSaveKind::Fortitude)),
                        reflex: FactValue::Value(save("reflex", CreatureSaveKind::Reflex)),
                        will: FactValue::Value(save("will", CreatureSaveKind::Will)),
                    }),
                    all_saves_note: FactValue::Missing,
                    immunities: FactValue::Value(Vec::new()),
                    resistances: FactValue::Value(Vec::new()),
                    weaknesses: FactValue::Value(Vec::new()),
                }),
                CreatureSourceField::Defenses,
            ),
            movement: CreatureFact::source(
                FactValue::Value(Vec::new()),
                CreatureSourceField::Movement,
            ),
            resources: CreatureFact::source(
                FactValue::Value(Vec::new()),
                CreatureSourceField::Resources,
            ),
            embedded_entities: CreatureFact::source(
                FactValue::Value(CreatureEmbeddedEntities::default()),
                CreatureSourceField::EmbeddedEntities,
            ),
            content: OwnedRichContent::default(),
            provenance: CreatureProvenance {
                source_path: "packs/creatures/surface-fixture.json".to_string(),
                source_contract_version: "test".to_string(),
                source_system_version: "test".to_string(),
                source_upstream_commit: "test".to_string(),
            },
        }
    }

    fn missing<T>(field: CreatureSourceField) -> CreatureFact<T> {
        CreatureFact::source(FactValue::Missing, field)
    }

    fn non_value<T>(state: CreatureSurfaceUnavailableStateView) -> FactValue<T> {
        match state {
            CreatureSurfaceUnavailableStateView::Missing => FactValue::Missing,
            CreatureSurfaceUnavailableStateView::Null => FactValue::Null,
            CreatureSurfaceUnavailableStateView::Unsupported => unreachable!(),
        }
    }

    fn unsupported_value() -> UnsupportedSourceValue {
        UnsupportedSourceValue {
            shape: UnsupportedSourceShape::Object,
            value: "{}".to_string(),
            reason: UnsupportedSourceReason::SourceFieldDrift,
        }
    }

    fn occurrence(
        owner: &RecordKey,
        id: &str,
        authored_order: u32,
        family: atlas_record::CreatureEntityFamily,
        parent: atlas_record::CreatureOccurrenceParent,
        capability: atlas_record::CreatureCapability,
    ) -> atlas_record::CreatureEntityOccurrence {
        atlas_record::CreatureEntityOccurrence {
            id: atlas_record::CreatureOccurrenceId::new(id).expect("occurrence id"),
            identity_stability: atlas_record::OccurrenceIdentityStability::StableNestedSourceId,
            owner: owner.clone(),
            target: atlas_record::CreatureEntityTarget::ActorOwned(
                atlas_record::CreatureEntityId::new(id).expect("entity id"),
            ),
            family,
            authored_order,
            source_sort: FactValue::Value(authored_order.into()),
            source_folder: FactValue::Missing,
            source_identity: atlas_record::CreatureEntitySourceIdentity {
                nested_source_id: FactValue::Missing,
                stable_source_locator: FactValue::Missing,
                source_locators: Vec::new(),
            },
            parent,
            context: atlas_record::CreatureOccurrenceContext {
                contextual_label: FactValue::Value(id.to_string()),
                ..atlas_record::CreatureOccurrenceContext::default()
            },
            capability,
            deltas: Vec::new(),
        }
    }

    fn cause(
        state: CreatureSurfaceUnavailableStateView,
        field: CreatureSurfaceUnavailableFieldView,
        component_id: Option<&str>,
        source_field: CreatureSurfaceSourceFieldView,
    ) -> CauseTuple {
        (
            state,
            field,
            component_id.map(str::to_string),
            CreatureSurfaceFactOwnerView::CanonicalCreature,
            source_field,
        )
    }

    fn assert_causes(
        domain: &Option<CreatureSurfaceDomainUnavailableView>,
        mut expected: Vec<CauseTuple>,
    ) {
        let actual = domain
            .as_ref()
            .expect("expected unavailable domain")
            .causes
            .iter()
            .map(|cause| {
                (
                    cause.state,
                    cause.field,
                    cause.component_id.clone(),
                    cause.provenance.owner,
                    cause.provenance.field,
                )
            })
            .collect::<Vec<_>>();
        let mut sorted_actual = actual.clone();
        sorted_actual.sort();
        assert_eq!(
            actual, sorted_actual,
            "typed causes must use canonical order"
        );
        expected.sort();
        assert_eq!(actual, expected);
    }
}
