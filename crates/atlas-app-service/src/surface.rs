use std::collections::{BTreeMap, BTreeSet};

use atlas_app_model::{
    CreatureSurfaceAbilitiesView, CreatureSurfaceActionCostView, CreatureSurfaceActivityTypeView,
    CreatureSurfaceActivityView, CreatureSurfaceAwarenessView, CreatureSurfaceContentBlockView,
    CreatureSurfaceContentInlineView, CreatureSurfaceContentListItemView,
    CreatureSurfaceContentProvenanceView, CreatureSurfaceContentRoleView,
    CreatureSurfaceContentTableRowView, CreatureSurfaceContentView, CreatureSurfaceDamageView,
    CreatureSurfaceDefensesView, CreatureSurfaceDomainUnavailableView,
    CreatureSurfaceFactOwnerView, CreatureSurfaceFactProvenanceView, CreatureSurfaceIwrView,
    CreatureSurfaceMovementView, CreatureSurfaceProvenanceView,
    CreatureSurfaceRelationshipKindView, CreatureSurfaceRelationshipTargetView,
    CreatureSurfaceRelationshipView, CreatureSurfaceResourceView, CreatureSurfaceRollView,
    CreatureSurfaceSaveView, CreatureSurfaceSavesView, CreatureSurfaceSenseView,
    CreatureSurfaceSkillView, CreatureSurfaceSourceFieldView, CreatureSurfaceSpellView,
    CreatureSurfaceSpellcastingView, CreatureSurfaceUnavailableCauseView,
    CreatureSurfaceUnavailableDomainsView, CreatureSurfaceUnavailableFieldView,
    CreatureSurfaceUnavailableStateView, CreatureSurfaceView, CreatureSurfaceVitalsView,
    EncounterRuntimeActivityKindView, EncounterRuntimeActivityView,
    EncounterRuntimeAutomationLimitationCodeView, EncounterRuntimeAutomationLimitationTargetView,
    EncounterRuntimeAutomationLimitationView, EncounterRuntimeSpellView,
    EncounterRuntimeSpellcastingView, EncounterRuntimeView, RecordSurfaceMetadataView,
    RecordSurfacePresentationView, RecordSurfaceProfileView, RecordSurfaceSourceView,
    RecordSurfaceView, RuntimeFactProvenanceView, RuntimeFactSourceView,
    SurfaceUnavailableReasonView, SurfaceUnavailableView,
};
use atlas_record::{
    ContentOwner, ContentRole, CreatureActionCost, CreatureCapability, CreatureDamage,
    CreatureDefenses, CreatureEmbeddedEntities, CreatureEntityOccurrence,
    CreatureEntityRelationshipKind, CreatureEntityTarget, CreatureIwr, CreatureMovementMode,
    CreatureNumber, CreatureOccurrenceParent, CreatureRecord, CreatureRelationshipTarget,
    CreatureResourceAmount, CreatureRoll, CreatureRollKind, CreatureSpellPreparation, FactValue,
    PresentationContent, PresentationContentBlock, PresentationInline, RecordBody, RetrievedRecord,
    SenseAcuity, project_presentation_content,
};

use crate::projection::kind_label;

pub(crate) fn record_surface(
    retrieved: &RetrievedRecord,
    profile: RecordSurfaceProfileView,
    mut encounter: Option<EncounterRuntimeView>,
) -> RecordSurfaceView {
    let metadata = record_metadata(retrieved);
    let presentation = match (&retrieved.record.classification.kind, &retrieved.body) {
        (atlas_domain::RecordKind::Creature, Some(RecordBody::Creature(creature))) => {
            let content_placement = activity_content_placement(creature);
            if let Some(runtime) = encounter.as_mut() {
                compose_encounter_payload(creature, &content_placement, runtime);
            }
            RecordSurfacePresentationView::Creature {
                body: Box::new(creature_surface_with_placement(
                    creature,
                    profile,
                    &content_placement,
                )),
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

#[cfg(test)]
fn creature_surface(
    creature: &CreatureRecord,
    profile: RecordSurfaceProfileView,
) -> CreatureSurfaceView {
    let activity_content = activity_content_placement(creature);
    creature_surface_with_placement(creature, profile, &activity_content)
}

fn creature_surface_with_placement(
    creature: &CreatureRecord,
    profile: RecordSurfaceProfileView,
    activity_content: &ActivityContentPlacement,
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
        .then(|| activities(creature, activity_content, &mut unavailable))
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
        content: (detail || encounter)
            .then(|| content(creature, activity_content, encounter))
            .flatten(),
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
    activity_content: &ActivityContentPlacement,
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
        .filter_map(|occurrence| {
            let component_id = occurrence.id.as_str();
            if activity_content.failed_occurrences.contains(component_id) {
                unsupported(
                    unavailable,
                    SurfaceDomain::Activities,
                    CreatureSurfaceUnavailableFieldView::ActivityContent,
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                    Some(component_id.to_string()),
                );
                return None;
            }
            let content = activity_content
                .by_occurrence
                .get(component_id)
                .and_then(|indices| {
                    non_empty(
                        indices
                            .iter()
                            .filter_map(|index| content_view(&creature.content.documents[*index]))
                            .collect(),
                    )
                });
            match &occurrence.capability {
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
                    content,
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
                    content,
                    unavailable,
                )),
                _ => None,
            }
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
    content: Option<Vec<CreatureSurfaceContentView>>,
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
        content,
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

#[derive(Default)]
struct ActivityContentPlacement {
    by_occurrence: BTreeMap<String, Vec<usize>>,
    spell_by_occurrence: BTreeMap<String, Vec<usize>>,
    withheld_activity_documents: BTreeSet<usize>,
    withheld_spell_documents: BTreeSet<usize>,
    failed_occurrences: BTreeSet<String>,
    failed_spells: BTreeSet<String>,
}

fn activity_content_placement(creature: &CreatureRecord) -> ActivityContentPlacement {
    let Some(embedded) = creature.embedded_entities.value.as_value() else {
        return ActivityContentPlacement {
            withheld_activity_documents: creature
                .content
                .documents
                .iter()
                .enumerate()
                .filter_map(|(index, document)| {
                    (!matches!(document.owner, ContentOwner::Record(_))).then_some(index)
                })
                .collect(),
            withheld_spell_documents: creature
                .content
                .documents
                .iter()
                .enumerate()
                .filter_map(|(index, document)| {
                    (!matches!(document.owner, ContentOwner::Record(_))).then_some(index)
                })
                .collect(),
            ..ActivityContentPlacement::default()
        };
    };
    let capabilities = embedded
        .occurrences
        .iter()
        .filter(|occurrence| {
            matches!(
                occurrence.capability,
                CreatureCapability::Action(_)
                    | CreatureCapability::Strike(_)
                    | CreatureCapability::Spell(_)
            )
        })
        .collect::<Vec<_>>();
    let mut placement = ActivityContentPlacement::default();
    let occurrence_counts = embedded.occurrences.iter().fold(
        BTreeMap::<&str, usize>::new(),
        |mut counts, occurrence| {
            *counts.entry(occurrence.id.as_str()).or_default() += 1;
            counts
        },
    );
    let entity_counts =
        embedded
            .entities
            .iter()
            .fold(BTreeMap::<&str, usize>::new(), |mut counts, entity| {
                *counts.entry(entity.id.as_str()).or_default() += 1;
                counts
            });
    let content_counts = creature.content.documents.iter().fold(
        BTreeMap::<(String, String), usize>::new(),
        |mut counts, document| {
            *counts
                .entry((
                    document.id.parent_record_key.to_string(),
                    document.id.content_key.as_str().to_string(),
                ))
                .or_default() += 1;
            counts
        },
    );

    for occurrence in &capabilities {
        let occurrence_id = occurrence.id.as_str();
        let failed = match occurrence.capability {
            CreatureCapability::Spell(_) => &mut placement.failed_spells,
            CreatureCapability::Action(_) | CreatureCapability::Strike(_) => {
                &mut placement.failed_occurrences
            }
            _ => continue,
        };
        if occurrence_counts.get(occurrence_id) != Some(&1) {
            failed.insert(occurrence_id.to_string());
        }
        if let CreatureEntityTarget::ActorOwned(entity_id) = &occurrence.target
            && entity_counts.get(entity_id.as_str()) != Some(&1)
        {
            failed.insert(occurrence_id.to_string());
        }
    }

    for (document_index, document) in creature.content.documents.iter().enumerate() {
        let matches = capabilities
            .iter()
            .filter(|occurrence| match &document.owner {
                ContentOwner::Record(_) => false,
                ContentOwner::CreatureOccurrence(owner) => owner == &occurrence.id,
                ContentOwner::CreatureEntity(owner) => {
                    matches!(&occurrence.target, CreatureEntityTarget::ActorOwned(target) if target == owner)
                }
            })
            .collect::<Vec<_>>();
        if matches.is_empty() {
            continue;
        }
        for occurrence in &matches {
            match occurrence.capability {
                CreatureCapability::Spell(_) => {
                    placement.withheld_spell_documents.insert(document_index);
                }
                CreatureCapability::Action(_) | CreatureCapability::Strike(_) => {
                    placement.withheld_activity_documents.insert(document_index);
                }
                _ => {}
            }
        }
        if matches.len() != 1
            || content_counts.get(&(
                document.id.parent_record_key.to_string(),
                document.id.content_key.as_str().to_string(),
            )) != Some(&1)
        {
            for occurrence in matches {
                match occurrence.capability {
                    CreatureCapability::Spell(_) => {
                        placement
                            .failed_spells
                            .insert(occurrence.id.as_str().to_string());
                    }
                    CreatureCapability::Action(_) | CreatureCapability::Strike(_) => {
                        placement
                            .failed_occurrences
                            .insert(occurrence.id.as_str().to_string());
                    }
                    _ => {}
                }
            }
            continue;
        }
        let occurrence = matches[0];
        let target = match occurrence.capability {
            CreatureCapability::Spell(_) => &mut placement.spell_by_occurrence,
            CreatureCapability::Action(_) | CreatureCapability::Strike(_) => {
                &mut placement.by_occurrence
            }
            _ => continue,
        };
        target
            .entry(occurrence.id.as_str().to_string())
            .or_default()
            .push(document_index);
    }

    for indices in placement
        .by_occurrence
        .values_mut()
        .chain(placement.spell_by_occurrence.values_mut())
    {
        indices.sort_by_key(|index| {
            let document = &creature.content.documents[*index];
            (document.authored_order, document.id.content_key.as_str())
        });
    }
    for failed in &placement.failed_occurrences {
        placement.by_occurrence.remove(failed);
    }
    for failed in &placement.failed_spells {
        placement.spell_by_occurrence.remove(failed);
    }
    placement
}

fn compose_encounter_payload(
    creature: &CreatureRecord,
    placement: &ActivityContentPlacement,
    runtime: &mut EncounterRuntimeView,
) {
    let Some(embedded) = creature.embedded_entities.value.as_value() else {
        for activity in std::mem::take(&mut runtime.activities) {
            push_payload_limitation(
                runtime,
                EncounterRuntimeAutomationLimitationCodeView::EncounterPayloadContentUnavailable,
                EncounterRuntimeAutomationLimitationTargetView::Activity {
                    activity_id: activity.activity_id,
                },
                "Canonical occurrence ownership is unavailable for this activity.",
            );
        }
        runtime.spellcasting.clear();
        runtime.standalone_spells.clear();
        return;
    };

    let occurrence_counts = embedded.occurrences.iter().fold(
        BTreeMap::<&str, usize>::new(),
        |mut counts, occurrence| {
            *counts.entry(occurrence.id.as_str()).or_default() += 1;
            counts
        },
    );
    let entity_counts =
        embedded
            .entities
            .iter()
            .fold(BTreeMap::<&str, usize>::new(), |mut counts, entity| {
                *counts.entry(entity.id.as_str()).or_default() += 1;
                counts
            });

    let mut runtime_activities = BTreeMap::<String, Vec<EncounterRuntimeActivityView>>::new();
    for activity in std::mem::take(&mut runtime.activities) {
        runtime_activities
            .entry(activity.activity_id.clone())
            .or_default()
            .push(activity);
    }

    let mut retained_activities = Vec::new();
    let mut activity_occurrences = embedded
        .occurrences
        .iter()
        .filter(|occurrence| {
            matches!(
                occurrence.capability,
                CreatureCapability::Action(_) | CreatureCapability::Strike(_)
            )
        })
        .collect::<Vec<_>>();
    activity_occurrences.sort_by_key(|occurrence| occurrence.authored_order);
    for occurrence in activity_occurrences {
        let occurrence_id = occurrence.id.as_str();
        let mut matches = runtime_activities.remove(occurrence_id).unwrap_or_default();
        let safe = occurrence_counts.get(occurrence_id) == Some(&1)
            && actor_owned_target_is_unique(occurrence, &entity_counts)
            && !placement.failed_occurrences.contains(occurrence_id)
            && matches.len() == 1;
        if !safe {
            push_payload_limitation(
                runtime,
                EncounterRuntimeAutomationLimitationCodeView::EncounterPayloadContentUnavailable,
                EncounterRuntimeAutomationLimitationTargetView::Activity {
                    activity_id: occurrence_id.to_string(),
                },
                "This activity was omitted because its canonical occurrence/content association was not unique.",
            );
            continue;
        }
        let mut activity = matches.remove(0);
        activity.content =
            content_for_occurrence(creature, &placement.by_occurrence, occurrence_id);
        retained_activities.push(activity);
    }
    runtime.activities = retained_activities;

    let mut adjusted_entries = BTreeMap::<String, Vec<EncounterRuntimeSpellcastingView>>::new();
    for entry in std::mem::take(&mut runtime.spellcasting) {
        adjusted_entries
            .entry(entry.entry_id.clone())
            .or_default()
            .push(entry);
    }

    let mut entries = embedded
        .occurrences
        .iter()
        .filter(|occurrence| {
            matches!(
                occurrence.capability,
                CreatureCapability::SpellcastingEntry(_)
            )
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.authored_order);
    let entry_counts = entries
        .iter()
        .fold(BTreeMap::<&str, usize>::new(), |mut counts, entry| {
            *counts.entry(entry.id.as_str()).or_default() += 1;
            counts
        });

    let mut grouped_spells = BTreeMap::<String, Vec<&CreatureEntityOccurrence>>::new();
    let mut standalone_spells = Vec::new();
    for occurrence in embedded
        .occurrences
        .iter()
        .filter(|occurrence| matches!(occurrence.capability, CreatureCapability::Spell(_)))
    {
        match &occurrence.parent {
            CreatureOccurrenceParent::SpellcastingEntry(parent) => grouped_spells
                .entry(parent.as_str().to_string())
                .or_default()
                .push(occurrence),
            CreatureOccurrenceParent::Creature => standalone_spells.push(occurrence),
        }
    }

    let mut projected_entries = Vec::new();
    for entry in entries {
        let entry_id = entry.id.as_str();
        let mut adjusted = adjusted_entries.remove(entry_id).unwrap_or_default();
        let safe = entry_counts.get(entry_id) == Some(&1)
            && occurrence_counts.get(entry_id) == Some(&1)
            && matches!(entry.parent, CreatureOccurrenceParent::Creature)
            && actor_owned_target_is_unique(entry, &entity_counts)
            && adjusted.len() <= 1;
        if !safe {
            push_payload_limitation(
                runtime,
                EncounterRuntimeAutomationLimitationCodeView::EncounterSpellGroupingUnavailable,
                EncounterRuntimeAutomationLimitationTargetView::Spellcasting {
                    entry_id: entry_id.to_string(),
                },
                "This spellcasting entry was omitted because its canonical identity or grouping was not unique.",
            );
            grouped_spells.remove(entry_id);
            continue;
        }
        let CreatureCapability::SpellcastingEntry(capability) = &entry.capability else {
            continue;
        };
        let mut view = adjusted
            .pop()
            .unwrap_or_else(|| EncounterRuntimeSpellcastingView {
                entry_id: entry_id.to_string(),
                authored_order: entry.authored_order,
                label: occurrence_label(entry, embedded),
                preparation: None,
                tradition: None,
                attack: None,
                dc: None,
                slots: Vec::new(),
                spells: Vec::new(),
            });
        view.authored_order = entry.authored_order;
        view.label = occurrence_label(entry, embedded);
        view.preparation = capability.preparation.as_value().and_then(preparation_name);
        view.tradition = capability.tradition.as_value().cloned();
        view.slots.sort_by_key(|slot| slot.rank);
        view.spells = grouped_spells
            .remove(entry_id)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|spell| {
                encounter_spell_view(
                    spell,
                    creature,
                    embedded,
                    placement,
                    &occurrence_counts,
                    &entity_counts,
                    &mut runtime_activities,
                    runtime,
                )
            })
            .collect();
        view.spells.sort_by_key(|spell| spell.authored_order);
        projected_entries.push(view);
    }
    projected_entries.sort_by_key(|entry| entry.authored_order);
    runtime.spellcasting = projected_entries;

    standalone_spells.sort_by_key(|spell| spell.authored_order);
    runtime.standalone_spells = standalone_spells
        .into_iter()
        .filter_map(|spell| {
            encounter_spell_view(
                spell,
                creature,
                embedded,
                placement,
                &occurrence_counts,
                &entity_counts,
                &mut runtime_activities,
                runtime,
            )
        })
        .collect();

    for spells in grouped_spells.into_values() {
        for spell in spells {
            push_payload_limitation(
                runtime,
                EncounterRuntimeAutomationLimitationCodeView::EncounterSpellGroupingUnavailable,
                EncounterRuntimeAutomationLimitationTargetView::Spell {
                    occurrence_id: spell.id.as_str().to_string(),
                },
                "This spell was omitted because its spellcasting parent is dangling.",
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn encounter_spell_view(
    spell: &CreatureEntityOccurrence,
    creature: &CreatureRecord,
    embedded: &CreatureEmbeddedEntities,
    placement: &ActivityContentPlacement,
    occurrence_counts: &BTreeMap<&str, usize>,
    entity_counts: &BTreeMap<&str, usize>,
    runtime_activities: &mut BTreeMap<String, Vec<EncounterRuntimeActivityView>>,
    runtime: &mut EncounterRuntimeView,
) -> Option<EncounterRuntimeSpellView> {
    let occurrence_id = spell.id.as_str();
    let mut activities = runtime_activities.remove(occurrence_id).unwrap_or_default();
    let safe = occurrence_counts.get(occurrence_id) == Some(&1)
        && actor_owned_target_is_unique(spell, entity_counts)
        && !placement.failed_spells.contains(occurrence_id)
        && activities.len() <= 1;
    if !safe {
        push_payload_limitation(
            runtime,
            EncounterRuntimeAutomationLimitationCodeView::EncounterSpellGroupingUnavailable,
            EncounterRuntimeAutomationLimitationTargetView::Spell {
                occurrence_id: occurrence_id.to_string(),
            },
            "This spell was omitted because its canonical occurrence, content, or runtime activity association was not unique.",
        );
        return None;
    }
    let CreatureCapability::Spell(capability) = &spell.capability else {
        return None;
    };
    let activity = activities
        .pop()
        .filter(|activity| activity.kind == EncounterRuntimeActivityKindView::Spell);
    let provenance = activity
        .as_ref()
        .map_or_else(canonical_runtime_provenance, |activity| {
            activity.provenance.clone()
        });
    Some(EncounterRuntimeSpellView {
        occurrence_id: occurrence_id.to_string(),
        authored_order: spell.authored_order,
        label: occurrence_label(spell, embedded),
        target_record_key: match &spell.target {
            CreatureEntityTarget::CanonicalRecord(key) => Some(key.to_string()),
            CreatureEntityTarget::ActorOwned(_) => None,
        },
        rank: capability.base_rank.as_value().copied(),
        traits: capability.traits.as_value().cloned().unwrap_or_default(),
        content: content_for_occurrence(creature, &placement.spell_by_occurrence, occurrence_id),
        activity,
        provenance,
    })
}

fn actor_owned_target_is_unique(
    occurrence: &CreatureEntityOccurrence,
    entity_counts: &BTreeMap<&str, usize>,
) -> bool {
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(_) => true,
        CreatureEntityTarget::ActorOwned(entity_id) => {
            entity_counts.get(entity_id.as_str()) == Some(&1)
        }
    }
}

fn preparation_name(value: &CreatureSpellPreparation) -> Option<String> {
    match value {
        CreatureSpellPreparation::Prepared => Some("prepared".to_string()),
        CreatureSpellPreparation::Spontaneous => Some("spontaneous".to_string()),
        CreatureSpellPreparation::Focus => Some("focus".to_string()),
        CreatureSpellPreparation::Innate => Some("innate".to_string()),
        CreatureSpellPreparation::Ritual => Some("ritual".to_string()),
        CreatureSpellPreparation::Unsupported(_) => None,
    }
}

fn canonical_runtime_provenance() -> RuntimeFactProvenanceView {
    RuntimeFactProvenanceView {
        source: RuntimeFactSourceView::CanonicalRecord,
        canonical_target: None,
    }
}

fn push_payload_limitation(
    runtime: &mut EncounterRuntimeView,
    code: EncounterRuntimeAutomationLimitationCodeView,
    target: EncounterRuntimeAutomationLimitationTargetView,
    message: &str,
) {
    runtime
        .automation_limitations
        .push(EncounterRuntimeAutomationLimitationView {
            code,
            target,
            message: message.to_string(),
        });
}

fn content_for_occurrence(
    creature: &CreatureRecord,
    placement: &BTreeMap<String, Vec<usize>>,
    occurrence_id: &str,
) -> Option<Vec<CreatureSurfaceContentView>> {
    placement.get(occurrence_id).and_then(|indices| {
        non_empty(
            indices
                .iter()
                .filter_map(|index| content_view(&creature.content.documents[*index]))
                .collect(),
        )
    })
}

fn content(
    creature: &CreatureRecord,
    activity_content: &ActivityContentPlacement,
    encounter: bool,
) -> Option<Vec<CreatureSurfaceContentView>> {
    let mut documents = creature
        .content
        .documents
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            !activity_content.withheld_activity_documents.contains(index)
                && (!encounter || !activity_content.withheld_spell_documents.contains(index))
        })
        .collect::<Vec<_>>();
    documents
        .sort_by_key(|(_, document)| (document.authored_order, document.id.content_key.as_str()));
    non_empty(
        documents
            .into_iter()
            .filter_map(|(_, document)| content_view(document))
            .collect(),
    )
}

fn content_view(
    document: &atlas_record::OwnedRichContentDocument,
) -> Option<CreatureSurfaceContentView> {
    let blocks = project_content(project_presentation_content(&document.document));
    (!blocks.is_empty()).then(|| CreatureSurfaceContentView {
        content_key: document.id.content_key.as_str().to_string(),
        role: content_role(document.role),
        authored_order: document.authored_order,
        label: document.label.clone(),
        blocks,
        content_hash: document.content_hash.as_str().to_string(),
        visibility: document.visibility.as_str().to_string(),
        provenance: CreatureSurfaceContentProvenanceView {
            source_record_key: document.provenance.source_record_key.to_string(),
            relative_source_path: document.provenance.relative_source_path.clone(),
            field_family: document.provenance.field_or_pointer_family.clone(),
            nested_source_id: document.provenance.nested_source_id.clone(),
        },
    })
}

fn project_content(content: PresentationContent) -> Vec<CreatureSurfaceContentBlockView> {
    content
        .blocks
        .into_iter()
        .map(project_content_block)
        .collect()
}

fn project_content_block(block: PresentationContentBlock) -> CreatureSurfaceContentBlockView {
    match block {
        PresentationContentBlock::Heading { level, text } => {
            CreatureSurfaceContentBlockView::Heading { level, text }
        }
        PresentationContentBlock::Paragraph { spans } => {
            CreatureSurfaceContentBlockView::Paragraph {
                spans: spans.into_iter().map(project_content_inline).collect(),
            }
        }
        PresentationContentBlock::List { ordered, items } => {
            CreatureSurfaceContentBlockView::List {
                ordered,
                items: items
                    .into_iter()
                    .map(|item| CreatureSurfaceContentListItemView {
                        blocks: item.blocks.into_iter().map(project_content_block).collect(),
                    })
                    .collect(),
            }
        }
        PresentationContentBlock::Table { caption, rows } => {
            CreatureSurfaceContentBlockView::Table {
                caption,
                rows: rows
                    .into_iter()
                    .map(|row| CreatureSurfaceContentTableRowView {
                        cells: row.cells.into_iter().map(project_content).collect(),
                    })
                    .collect(),
            }
        }
        PresentationContentBlock::Rule => CreatureSurfaceContentBlockView::Divider,
    }
}

fn project_content_inline(span: PresentationInline) -> CreatureSurfaceContentInlineView {
    match span {
        PresentationInline::Text { text } => CreatureSurfaceContentInlineView::Text { text },
        PresentationInline::Strong { spans } => CreatureSurfaceContentInlineView::Strong {
            spans: spans.into_iter().map(project_content_inline).collect(),
        },
        PresentationInline::Emphasis { spans } => CreatureSurfaceContentInlineView::Emphasis {
            spans: spans.into_iter().map(project_content_inline).collect(),
        },
        PresentationInline::Code { text } => CreatureSurfaceContentInlineView::Code { text },
        PresentationInline::Reference {
            label,
            record_key,
            embedded,
        } => CreatureSurfaceContentInlineView::Reference {
            label,
            record_key: record_key.map(|key| key.to_string()),
            embedded,
        },
        PresentationInline::LineBreak => CreatureSurfaceContentInlineView::LineBreak,
    }
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
    use std::collections::BTreeMap;

    use atlas_domain::{PackName, RecordId, RecordKey};
    use atlas_record::{
        ContentId, ContentIdentityStability, ContentKey, ContentOrigin, ContentOwner,
        ContentProvenance, ContentRole, ContentSourceKind, ContentVisibility,
        CreatureActionCapability, CreatureArmorClass, CreatureDefenses, CreatureEmbeddedEntities,
        CreatureEntity, CreatureEntityFamily, CreatureEntityTarget, CreatureFact, CreatureFamily,
        CreatureHitPoints, CreatureIdentity, CreatureLanguages, CreatureLegacyAbilities,
        CreatureNumber, CreatureOccurrenceParent, CreaturePerception, CreatureProvenance,
        CreatureSave, CreatureSaveKind, CreatureSaves, CreatureSourceField, DuplicateContentStatus,
        FactValue, FoundryNode, OwnedRichContent, OwnedRichContentDocument, RichDocument, RichNode,
        UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
    };

    use super::{
        activity_content_placement, compose_encounter_payload, creature_surface, non_empty,
    };
    use atlas_app_model::{
        CreatureSurfaceContentBlockView, CreatureSurfaceContentInlineView,
        CreatureSurfaceDomainUnavailableView, CreatureSurfaceFactOwnerView,
        CreatureSurfaceSourceFieldView, CreatureSurfaceUnavailableFieldView,
        CreatureSurfaceUnavailableStateView, EncounterRuntimeActivityKindView,
        EncounterRuntimeActivityUsageView, EncounterRuntimeActivityView,
        EncounterRuntimeAutomationLimitationCodeView, EncounterRuntimeSpellcastingView,
        EncounterRuntimeView, RecordSurfaceProfileView, RuntimeFactProvenanceView,
        RuntimeFactSourceView,
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
            entities: vec![entity(&owner, "strike", CreatureEntityFamily::Strike)],
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

    #[test]
    fn activity_content_uses_typed_targets_once_and_preserves_blocks_order_and_provenance() {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        let entity_id = "entity-plague";
        let occurrence_id = "occurrence-plague";
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![entity(&owner, entity_id, CreatureEntityFamily::Action)],
            occurrences: vec![action_occurrence(&owner, occurrence_id, entity_id, 3)],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });
        creature.content.documents = vec![
            content_document(
                &owner,
                "public-notes",
                ContentOwner::Record(owner.clone()),
                0,
                "Public Notes",
                vec![
                    paragraph(vec![RichNode::Text {
                        text: "First paragraph.".to_string(),
                    }]),
                    RichNode::HtmlElement {
                        tag: "hr".to_string(),
                        attributes: BTreeMap::new(),
                        children: Vec::new(),
                    },
                    paragraph(vec![RichNode::Text {
                        text: "After divider.".to_string(),
                    }]),
                ],
            ),
            content_document(
                &owner,
                "item:plague:occurrence-note",
                ContentOwner::CreatureOccurrence(
                    atlas_record::CreatureOccurrenceId::new(occurrence_id).expect("occurrence id"),
                ),
                1,
                "Occurrence Note",
                vec![paragraph(vec![RichNode::Text {
                    text: "Occurrence-owned.".to_string(),
                }])],
            ),
            content_document(
                &owner,
                "item:plague:description",
                ContentOwner::CreatureEntity(
                    atlas_record::CreatureEntityId::new(entity_id).expect("entity id"),
                ),
                2,
                "Abyssal Plague",
                vec![paragraph(vec![
                    RichNode::Text {
                        text: "Saving Throw ".to_string(),
                    },
                    RichNode::Foundry {
                        node: FoundryNode::Check {
                            statistic: Some("fortitude".to_string()),
                            options: BTreeMap::from([("dc".to_string(), "28".to_string())]),
                            label: None,
                        },
                    },
                ])],
            ),
            content_document(
                &owner,
                "item:equipment:description",
                ContentOwner::CreatureEntity(
                    atlas_record::CreatureEntityId::new("unrelated-equipment").expect("entity id"),
                ),
                4,
                "Unrelated Equipment",
                vec![paragraph(vec![RichNode::Text {
                    text: "Stays general.".to_string(),
                }])],
            ),
        ];

        let forward = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        let activity = &forward.activities.as_ref().expect("activity")[0];
        let attached = activity.content.as_ref().expect("attached content");
        assert_eq!(
            attached
                .iter()
                .map(|content| content.content_key.as_str())
                .collect::<Vec<_>>(),
            ["item:plague:occurrence-note", "item:plague:description"]
        );
        assert_eq!(attached[1].authored_order, 2);
        assert_eq!(attached[1].provenance.source_record_key, owner.to_string());
        assert!(content_contains_text(
            &attached[1].blocks,
            "Fortitude DC 28"
        ));
        let general = forward.content.as_ref().expect("general content");
        assert_eq!(
            general
                .iter()
                .map(|content| content.content_key.as_str())
                .collect::<Vec<_>>(),
            ["public-notes", "item:equipment:description"]
        );
        assert!(matches!(
            general[0].blocks.get(1),
            Some(CreatureSurfaceContentBlockView::Divider)
        ));

        creature.content.documents.reverse();
        let reverse = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        assert_eq!(
            forward, reverse,
            "authored order must not depend on input order"
        );
    }

    #[test]
    fn activity_content_association_failures_omit_activity_and_never_fall_back_to_general() {
        let base = activity_content_fixture();
        let cases = [
            ("missing-target", {
                let mut creature = base.clone();
                creature
                    .embedded_entities
                    .value
                    .as_value()
                    .expect("embedded");
                if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
                    embedded.entities.clear();
                }
                creature
            }),
            ("duplicate-entity", {
                let mut creature = base.clone();
                if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
                    embedded.entities.push(embedded.entities[0].clone());
                }
                creature
            }),
            ("duplicate-occurrence", {
                let mut creature = base.clone();
                if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
                    embedded.occurrences.push(embedded.occurrences[0].clone());
                }
                creature
            }),
            ("duplicate-content", {
                let mut creature = base.clone();
                creature
                    .content
                    .documents
                    .push(creature.content.documents[0].clone());
                creature
            }),
        ];

        for (case, creature) in cases {
            let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
            assert!(surface.activities.is_none(), "{case} must fail closed");
            assert!(
                surface.content.is_none(),
                "{case} must not fall back to lore"
            );
            assert_causes(
                &surface
                    .unavailable_domains
                    .expect("failure should remain typed")
                    .activities,
                vec![cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::ActivityContent,
                    Some("activity"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                )],
            );
        }
    }

    #[test]
    fn one_entity_document_matching_multiple_activities_fails_both_activities_closed() {
        let mut creature = activity_content_fixture();
        let owner = creature.identity.record_key.clone();
        if let FactValue::Value(embedded) = &mut creature.embedded_entities.value {
            embedded
                .occurrences
                .push(action_occurrence(&owner, "activity-two", "entity", 1));
        }

        let surface = creature_surface(&creature, RecordSurfaceProfileView::RecordDetail);
        assert!(surface.activities.is_none());
        assert!(surface.content.is_none());
        assert_causes(
            &surface
                .unavailable_domains
                .expect("multi-match should remain typed")
                .activities,
            vec![
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::ActivityContent,
                    Some("activity"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
                cause(
                    CreatureSurfaceUnavailableStateView::Unsupported,
                    CreatureSurfaceUnavailableFieldView::ActivityContent,
                    Some("activity-two"),
                    CreatureSurfaceSourceFieldView::EmbeddedEntities,
                ),
            ],
        );
    }

    #[test]
    fn encounter_activity_content_is_nested_once_and_removed_from_general_content() {
        let creature = activity_content_fixture();
        let placement = activity_content_placement(&creature);
        let mut runtime = empty_runtime();
        runtime.activities.push(runtime_activity(
            "activity",
            EncounterRuntimeActivityKindView::Other,
        ));

        compose_encounter_payload(&creature, &placement, &mut runtime);

        assert_eq!(runtime.activities.len(), 1);
        assert!(
            runtime.activities[0]
                .content
                .as_ref()
                .is_some_and(|content| {
                    content.len() == 1
                        && content_contains_text(&content[0].blocks, "Activity content.")
                })
        );
        assert!(super::content(&creature, &placement, true).is_none());
        assert!(runtime.automation_limitations.is_empty());
    }

    #[test]
    fn encounter_spells_group_by_typed_parent_preserve_standalone_and_ignore_input_order() {
        let creature = spell_payload_fixture();
        let placement = activity_content_placement(&creature);
        let runtime = spell_payload_runtime();

        let mut forward = runtime.clone();
        compose_encounter_payload(&creature, &placement, &mut forward);

        let mut reversed_creature = creature.clone();
        if let FactValue::Value(embedded) = &mut reversed_creature.embedded_entities.value {
            embedded.occurrences.reverse();
        }
        let reversed_placement = activity_content_placement(&reversed_creature);
        let mut reversed = runtime;
        reversed.activities.reverse();
        reversed.spellcasting.reverse();
        compose_encounter_payload(&reversed_creature, &reversed_placement, &mut reversed);

        assert_eq!(forward, reversed);
        assert_eq!(forward.spellcasting.len(), 2);
        assert_eq!(forward.spellcasting[0].label, "Arcane Innate Spells");
        assert_eq!(
            forward.spellcasting[0].preparation.as_deref(),
            Some("innate")
        );
        assert_eq!(forward.spellcasting[0].tradition.as_deref(), Some("arcane"));
        assert_eq!(forward.spellcasting[0].spells.len(), 2);
        assert_eq!(
            forward.spellcasting[0].spells[0].target_record_key,
            forward.spellcasting[0].spells[1].target_record_key,
            "repeated canonical targets remain distinct occurrence rows"
        );
        assert!(forward.spellcasting[0].spells[0].content.is_some());
        assert_eq!(forward.standalone_spells.len(), 1);
        assert_eq!(forward.standalone_spells[0].label, "Control Weather");
        assert!(
            forward
                .activities
                .iter()
                .all(|activity| activity.kind != EncounterRuntimeActivityKindView::Spell)
        );
        assert!(super::content(&creature, &placement, true).is_none());
        assert!(forward.automation_limitations.is_empty());
    }

    #[test]
    fn encounter_spell_grouping_fails_only_affected_rows_closed() {
        let mut creature = spell_payload_fixture();
        let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
            panic!("embedded entities should be present");
        };
        let duplicate = embedded
            .occurrences
            .iter()
            .find(|occurrence| occurrence.id.as_str() == "spell-a")
            .expect("spell-a")
            .clone();
        embedded.occurrences.push(duplicate);
        let dangling = spell_occurrence(
            &creature.identity.record_key,
            "dangling-spell",
            "Dangling Spell",
            9,
            CreatureOccurrenceParent::SpellcastingEntry(
                atlas_record::CreatureOccurrenceId::new("missing-entry").expect("entry id"),
            ),
            "spells:dangling-spell",
        );
        embedded.occurrences.push(dangling);
        let placement = activity_content_placement(&creature);
        let mut runtime = spell_payload_runtime();
        runtime.activities.push(runtime_activity(
            "spell-a",
            EncounterRuntimeActivityKindView::Spell,
        ));

        compose_encounter_payload(&creature, &placement, &mut runtime);

        assert_eq!(runtime.spellcasting[0].spells.len(), 1);
        assert_eq!(runtime.spellcasting[0].spells[0].occurrence_id, "spell-b");
        assert_eq!(runtime.standalone_spells.len(), 1);
        assert!(runtime.automation_limitations.iter().any(|limitation| {
            limitation.code
                == EncounterRuntimeAutomationLimitationCodeView::EncounterSpellGroupingUnavailable
        }));
        assert!(
            runtime
                .activities
                .iter()
                .all(|activity| activity.kind != EncounterRuntimeActivityKindView::Spell)
        );
    }

    fn empty_runtime() -> EncounterRuntimeView {
        EncounterRuntimeView {
            level: None,
            vitals: None,
            defenses: None,
            saves: None,
            awareness: None,
            abilities: None,
            skills: Vec::new(),
            movement: None,
            resources: Vec::new(),
            spellcasting: Vec::new(),
            standalone_spells: Vec::new(),
            activities: Vec::new(),
            action_budget: None,
            conditions: Vec::new(),
            automation_limitations: Vec::new(),
        }
    }

    fn runtime_activity(
        activity_id: &str,
        kind: EncounterRuntimeActivityKindView,
    ) -> EncounterRuntimeActivityView {
        EncounterRuntimeActivityView {
            activity_id: activity_id.to_string(),
            label: activity_id.to_string(),
            kind,
            usage: EncounterRuntimeActivityUsageView::Unlimited,
            action_cost: None,
            frequency: None,
            uses: None,
            rolls: Vec::new(),
            damage: Vec::new(),
            modes: Vec::new(),
            content: None,
            provenance: RuntimeFactProvenanceView {
                source: RuntimeFactSourceView::CanonicalRecord,
                canonical_target: None,
            },
        }
    }

    fn spell_payload_runtime() -> EncounterRuntimeView {
        let mut runtime = empty_runtime();
        for entry_id in ["entry-arcane", "entry-divine"] {
            runtime.spellcasting.push(EncounterRuntimeSpellcastingView {
                entry_id: entry_id.to_string(),
                authored_order: 0,
                label: "Spell Attack".to_string(),
                preparation: None,
                tradition: None,
                attack: None,
                dc: None,
                slots: Vec::new(),
                spells: Vec::new(),
            });
            runtime.activities.push(runtime_activity(
                entry_id,
                EncounterRuntimeActivityKindView::Spell,
            ));
        }
        for spell_id in ["spell-a", "spell-b", "control-weather"] {
            runtime.activities.push(runtime_activity(
                spell_id,
                EncounterRuntimeActivityKindView::Spell,
            ));
        }
        runtime
    }

    fn spell_payload_fixture() -> atlas_record::CreatureRecord {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        let arcane = spellcasting_entry_occurrence(
            &owner,
            "entry-arcane",
            "Arcane Innate Spells",
            1,
            "arcane",
        );
        let divine = spellcasting_entry_occurrence(
            &owner,
            "entry-divine",
            "Divine Innate Spells",
            2,
            "divine",
        );
        let parent = atlas_record::CreatureOccurrenceId::new("entry-arcane").expect("entry id");
        let spell_a = spell_occurrence(
            &owner,
            "spell-a",
            "Repeated Spell",
            3,
            CreatureOccurrenceParent::SpellcastingEntry(parent.clone()),
            "spells:repeated-spell",
        );
        let spell_b = spell_occurrence(
            &owner,
            "spell-b",
            "Repeated Spell",
            4,
            CreatureOccurrenceParent::SpellcastingEntry(parent),
            "spells:repeated-spell",
        );
        let standalone = spell_occurrence(
            &owner,
            "control-weather",
            "Control Weather",
            5,
            CreatureOccurrenceParent::Creature,
            "spells:control-weather",
        );
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![
                entity(
                    &owner,
                    "entry-arcane",
                    CreatureEntityFamily::SpellcastingEntry,
                ),
                entity(
                    &owner,
                    "entry-divine",
                    CreatureEntityFamily::SpellcastingEntry,
                ),
            ],
            occurrences: vec![divine, spell_b, standalone, arcane, spell_a],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });
        creature.content.documents = vec![content_document(
            &owner,
            "spell-a-rules",
            ContentOwner::CreatureOccurrence(
                atlas_record::CreatureOccurrenceId::new("spell-a").expect("spell id"),
            ),
            0,
            "Repeated Spell Rules",
            vec![paragraph(vec![RichNode::Text {
                text: "Spell occurrence rules.".to_string(),
            }])],
        )];
        creature
    }

    fn spellcasting_entry_occurrence(
        owner: &RecordKey,
        id: &str,
        label: &str,
        authored_order: u32,
        tradition: &str,
    ) -> atlas_record::CreatureEntityOccurrence {
        let mut occurrence = occurrence(
            owner,
            id,
            authored_order,
            CreatureEntityFamily::SpellcastingEntry,
            CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::SpellcastingEntry(
                atlas_record::CreatureSpellcastingEntryCapability {
                    preparation: FactValue::Value(atlas_record::CreatureSpellPreparation::Innate),
                    tradition: FactValue::Value(tradition.to_string()),
                    attack: FactValue::Value(20),
                    dc: FactValue::Value(30),
                    slots: FactValue::Value(Vec::new()),
                    unsupported_notes: Vec::new(),
                },
            ),
        );
        occurrence.context.contextual_label = FactValue::Value(label.to_string());
        occurrence
    }

    fn spell_occurrence(
        owner: &RecordKey,
        id: &str,
        label: &str,
        authored_order: u32,
        parent: CreatureOccurrenceParent,
        target: &str,
    ) -> atlas_record::CreatureEntityOccurrence {
        let mut occurrence = occurrence(
            owner,
            id,
            authored_order,
            CreatureEntityFamily::Spell,
            parent,
            atlas_record::CreatureCapability::Spell(atlas_record::CreatureSpellCapability {
                traits: FactValue::Value(vec!["spell".to_string()]),
                base_rank: FactValue::Value(6),
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
                action_cost: atlas_record::CreatureActionCost::Actions(2),
                unsupported_notes: Vec::new(),
            }),
        );
        occurrence.context.contextual_label = FactValue::Value(label.to_string());
        occurrence.target = CreatureEntityTarget::CanonicalRecord(
            RecordKey::parse(target).expect("target key should parse"),
        );
        occurrence
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

    fn activity_content_fixture() -> atlas_record::CreatureRecord {
        let mut creature = known_empty_creature();
        let owner = creature.identity.record_key.clone();
        creature.embedded_entities.value = FactValue::Value(CreatureEmbeddedEntities {
            entities: vec![entity(&owner, "entity", CreatureEntityFamily::Action)],
            occurrences: vec![action_occurrence(&owner, "activity", "entity", 0)],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        });
        creature.content.documents = vec![content_document(
            &owner,
            "item:activity:description",
            ContentOwner::CreatureEntity(
                atlas_record::CreatureEntityId::new("entity").expect("entity id"),
            ),
            0,
            "Activity",
            vec![paragraph(vec![RichNode::Text {
                text: "Activity content.".to_string(),
            }])],
        )];
        creature
    }

    fn entity(owner: &RecordKey, id: &str, family: CreatureEntityFamily) -> CreatureEntity {
        CreatureEntity {
            id: atlas_record::CreatureEntityId::new(id).expect("entity id"),
            family,
            label: id.to_string(),
            source_identity: atlas_record::CreatureEntitySourceIdentity {
                nested_source_id: FactValue::Missing,
                stable_source_locator: FactValue::Missing,
                source_locators: vec![atlas_record::CreatureSourceLocator {
                    source_path: format!("{owner}/items/{id}"),
                    locator: atlas_record::StableSourceLocator::new(format!("items/{id}"))
                        .expect("source locator"),
                    precedence: 0,
                }],
            },
        }
    }

    fn action_occurrence(
        owner: &RecordKey,
        occurrence_id: &str,
        entity_id: &str,
        authored_order: u32,
    ) -> atlas_record::CreatureEntityOccurrence {
        let mut occurrence = occurrence(
            owner,
            occurrence_id,
            authored_order,
            CreatureEntityFamily::Action,
            atlas_record::CreatureOccurrenceParent::Creature,
            atlas_record::CreatureCapability::Action(CreatureActionCapability {
                category: FactValue::Missing,
                traits: FactValue::Value(Vec::new()),
                action_cost: atlas_record::CreatureActionCost::Actions(1),
                frequency: FactValue::Missing,
                self_effect: FactValue::Missing,
                self_effect_label: FactValue::Missing,
                requirements: FactValue::Missing,
                cost: FactValue::Missing,
                rolls: Vec::new(),
                damage: FactValue::Value(Vec::new()),
                unsupported_notes: Vec::new(),
            }),
        );
        occurrence.target = atlas_record::CreatureEntityTarget::ActorOwned(
            atlas_record::CreatureEntityId::new(entity_id).expect("entity id"),
        );
        occurrence
    }

    fn content_document(
        record_key: &RecordKey,
        content_key: &str,
        owner: ContentOwner,
        authored_order: u32,
        label: &str,
        nodes: Vec<RichNode>,
    ) -> OwnedRichContentDocument {
        let record_owned = matches!(&owner, ContentOwner::Record(_));
        let relative_source_path = if record_owned {
            "system.details.publicNotes".to_string()
        } else {
            format!("items[{label}].system.description.value")
        };
        let source_kind = if record_owned {
            ContentSourceKind::PublicNotes
        } else {
            ContentSourceKind::EmbeddedItemDescription
        };
        let origin = if record_owned {
            ContentOrigin::RecordField {
                source_kind,
                relative_source_path: relative_source_path.clone(),
            }
        } else {
            ContentOrigin::EmbeddedEntityField {
                family: CreatureEntityFamily::Action,
                nested_source_id: Some(label.to_string()),
                relative_source_path: relative_source_path.clone(),
            }
        };
        OwnedRichContentDocument::new(
            ContentId::new(
                record_key.clone(),
                ContentKey::new(content_key).expect("content key"),
            ),
            ContentIdentityStability::StableSourceIdentity,
            owner,
            if record_owned {
                ContentRole::SupplementalRules
            } else {
                ContentRole::EmbeddedCapability
            },
            origin,
            ContentVisibility::Public,
            ContentProvenance {
                source_record_key: record_key.clone(),
                relative_source_path,
                field_or_pointer_family: source_kind.as_str().to_string(),
                nested_source_id: (!record_owned).then(|| label.to_string()),
                authored_ordinal_or_range: Some(authored_order.to_string()),
                authored_label: Some(label.to_string()),
            },
            source_kind,
            authored_order,
            Some(label.to_string()),
            RichDocument::new(nodes),
            DuplicateContentStatus::Unique,
            Vec::new(),
        )
    }

    fn paragraph(children: Vec<RichNode>) -> RichNode {
        RichNode::HtmlElement {
            tag: "p".to_string(),
            attributes: BTreeMap::new(),
            children,
        }
    }

    fn content_contains_text(blocks: &[CreatureSurfaceContentBlockView], needle: &str) -> bool {
        blocks.iter().any(|block| match block {
            CreatureSurfaceContentBlockView::Heading { text, .. } => text.contains(needle),
            CreatureSurfaceContentBlockView::Paragraph { spans } => {
                spans.iter().any(|span| inline_contains_text(span, needle))
            }
            CreatureSurfaceContentBlockView::List { items, .. } => items
                .iter()
                .any(|item| content_contains_text(&item.blocks, needle)),
            CreatureSurfaceContentBlockView::Table { caption, rows } => {
                caption.as_ref().is_some_and(|value| value.contains(needle))
                    || rows.iter().any(|row| {
                        row.cells
                            .iter()
                            .any(|cell| content_contains_text(cell, needle))
                    })
            }
            CreatureSurfaceContentBlockView::Divider => false,
        })
    }

    fn inline_contains_text(span: &CreatureSurfaceContentInlineView, needle: &str) -> bool {
        match span {
            CreatureSurfaceContentInlineView::Text { text }
            | CreatureSurfaceContentInlineView::Code { text } => text.contains(needle),
            CreatureSurfaceContentInlineView::Strong { spans }
            | CreatureSurfaceContentInlineView::Emphasis { spans } => {
                spans.iter().any(|span| inline_contains_text(span, needle))
            }
            CreatureSurfaceContentInlineView::Reference { label, .. } => label.contains(needle),
            CreatureSurfaceContentInlineView::LineBreak => false,
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
