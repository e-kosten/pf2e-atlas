use atlas_app_model::{
    CreatureSurfaceActionCostView, CreatureSurfaceDamageView, HazardSurfaceActivityTypeView,
    HazardSurfaceActivityView, HazardSurfaceAttackModeView, HazardSurfaceComplexityView,
    HazardSurfaceDefensesView, HazardSurfaceDetectionView, HazardSurfaceEmitsSoundView,
    HazardSurfaceFrequencyView, HazardSurfaceHitPointsView, HazardSurfaceIwrView,
    HazardSurfaceLifecycleView, HazardSurfaceOccurrenceIdentityStabilityView,
    HazardSurfaceProvenanceTextView, HazardSurfaceProvenanceView, HazardSurfaceRuleView,
    HazardSurfaceSaveKindView, HazardSurfaceSavesView, HazardSurfaceSelfEffectView,
    HazardSurfaceSizeView, HazardSurfaceSourceFactView, HazardSurfaceSourceMetadataFactView,
    HazardSurfaceUnavailableStateView, HazardSurfaceUnavailableView, HazardSurfaceView,
    RecordSurfaceProfileView,
};
use atlas_record::{
    ContentOwner, ContentSourceKind, FactValue, HazardActionType, HazardAttackMode,
    HazardCapability, HazardComplexity, HazardEmitsSound, HazardEntity, HazardEntityOccurrence,
    HazardFact, HazardFrequencyInterval, HazardItemCommon, HazardIwr,
    HazardOccurrenceIdentityStability, HazardRecord, HazardRuleElement, HazardRuleMode,
    HazardRuleType, HazardSaveKind, HazardSize, HazardSourceMetadataFact, HazardSourceValue,
    HazardUnsupportedField, PublicationLicense, RichDocument, project_hazard_attack_mode,
    project_hazard_conveniences, project_hazard_source_metadata, project_hazard_strike_action_cost,
    project_presentation_content,
};

pub(crate) fn hazard_surface(
    hazard: &HazardRecord,
    profile: RecordSurfaceProfileView,
    teaser: Option<String>,
) -> HazardSurfaceView {
    let detail = profile == RecordSurfaceProfileView::RecordDetail;
    let compact = profile == RecordSurfaceProfileView::SearchCompact;
    let encounter = profile == RecordSurfaceProfileView::EncounterParticipant;
    let conveniences = project_hazard_conveniences(hazard);
    let source_metadata = project_hazard_source_metadata(hazard);
    let mut unavailable = Vec::new();

    let complexity = typed_fact(&hazard.complexity, "complexity", None, &mut unavailable)
        .copied()
        .map(|value| match value {
            HazardComplexity::Simple => HazardSurfaceComplexityView::Simple,
            HazardComplexity::Complex => HazardSurfaceComplexityView::Complex,
        });
    let size = detail
        .then(|| typed_fact(&hazard.size, "size", None, &mut unavailable))
        .flatten()
        .copied()
        .map(|value| match value {
            HazardSize::Tiny => HazardSurfaceSizeView::Tiny,
            HazardSize::Small => HazardSurfaceSizeView::Small,
            HazardSize::Medium => HazardSurfaceSizeView::Medium,
            HazardSize::Large => HazardSurfaceSizeView::Large,
            HazardSize::Huge => HazardSurfaceSizeView::Huge,
            HazardSize::Gargantuan => HazardSurfaceSizeView::Gargantuan,
        });
    let emits_sound = detail
        .then(|| typed_fact(&hazard.emits_sound, "emits_sound", None, &mut unavailable))
        .flatten()
        .map(|value| match value {
            HazardEmitsSound::Boolean(value) => {
                HazardSurfaceEmitsSoundView::Boolean { value: *value }
            }
            HazardEmitsSound::Named(value) => HazardSurfaceEmitsSoundView::Named {
                value: value.clone(),
            },
        });

    let detection =
        typed_fact(&hazard.detection, "detection", None, &mut unavailable).map(|detection| {
            HazardSurfaceDetectionView {
                stealth_modifier: (!encounter)
                    .then(|| {
                        typed_fact(
                            &detection.stealth_modifier,
                            "detection.stealth_modifier",
                            None,
                            &mut unavailable,
                        )
                        .copied()
                    })
                    .flatten(),
                difficulty_class: (!encounter).then_some(conveniences.detection_dc).flatten(),
                details: (detail || encounter)
                    .then(|| {
                        rich_document(
                            &detection.details,
                            "detection.details",
                            None,
                            &mut unavailable,
                        )
                    })
                    .flatten(),
            }
        });

    let defenses =
        typed_fact(&hazard.defenses, "defenses", None, &mut unavailable).map(|defenses| {
            let hit_points = typed_fact(
                &defenses.hit_points,
                "defenses.hit_points",
                None,
                &mut unavailable,
            )
            .map(|hit_points| HazardSurfaceHitPointsView {
                current: (!encounter)
                    .then(|| {
                        typed_fact(
                            &hit_points.current,
                            "defenses.hit_points.current",
                            None,
                            &mut unavailable,
                        )
                        .copied()
                    })
                    .flatten(),
                maximum: (!encounter)
                    .then(|| {
                        typed_fact(
                            &hit_points.maximum,
                            "defenses.hit_points.maximum",
                            None,
                            &mut unavailable,
                        )
                        .copied()
                    })
                    .flatten(),
                temporary: detail
                    .then(|| {
                        typed_fact(
                            &hit_points.temporary,
                            "defenses.hit_points.temporary",
                            None,
                            &mut unavailable,
                        )
                        .copied()
                    })
                    .flatten(),
                broken_threshold: (!encounter)
                    .then_some(conveniences.broken_threshold)
                    .flatten(),
                details: (detail || encounter)
                    .then(|| {
                        rich_document(
                            &hit_points.details,
                            "defenses.hit_points.details",
                            None,
                            &mut unavailable,
                        )
                    })
                    .flatten(),
            });
            let saves = typed_fact(&defenses.saves, "defenses.saves", None, &mut unavailable).map(
                |saves| HazardSurfaceSavesView {
                    fortitude: (!encounter)
                        .then(|| {
                            typed_fact(
                                &saves.fortitude,
                                "defenses.saves.fortitude",
                                None,
                                &mut unavailable,
                            )
                            .copied()
                        })
                        .flatten(),
                    reflex: (!encounter)
                        .then(|| {
                            typed_fact(
                                &saves.reflex,
                                "defenses.saves.reflex",
                                None,
                                &mut unavailable,
                            )
                            .copied()
                        })
                        .flatten(),
                    will: (!encounter)
                        .then(|| {
                            typed_fact(&saves.will, "defenses.saves.will", None, &mut unavailable)
                                .copied()
                        })
                        .flatten(),
                },
            );
            HazardSurfaceDefensesView {
                armor_class: (!encounter)
                    .then(|| {
                        typed_fact(
                            &defenses.armor_class,
                            "defenses.armor_class",
                            None,
                            &mut unavailable,
                        )
                        .copied()
                    })
                    .flatten(),
                hardness: typed_fact(
                    &defenses.hardness,
                    "defenses.hardness",
                    None,
                    &mut unavailable,
                )
                .copied(),
                hit_points,
                saves,
                immunities: iwr(
                    &defenses.immunities,
                    "defenses.immunities",
                    &mut unavailable,
                ),
                weaknesses: iwr(
                    &defenses.weaknesses,
                    "defenses.weaknesses",
                    &mut unavailable,
                ),
                resistances: iwr(
                    &defenses.resistances,
                    "defenses.resistances",
                    &mut unavailable,
                ),
            }
        });

    let lifecycle = (detail || encounter)
        .then(|| {
            typed_fact(&hazard.lifecycle, "lifecycle", None, &mut unavailable).map(|lifecycle| {
                HazardSurfaceLifecycleView {
                    description: rich_document(
                        &lifecycle.description,
                        "lifecycle.description",
                        None,
                        &mut unavailable,
                    ),
                    disable: rich_document(
                        &lifecycle.disable,
                        "lifecycle.disable",
                        None,
                        &mut unavailable,
                    ),
                    routine: rich_document(
                        &lifecycle.routine,
                        "lifecycle.routine",
                        None,
                        &mut unavailable,
                    ),
                    reset: rich_document(
                        &lifecycle.reset,
                        "lifecycle.reset",
                        None,
                        &mut unavailable,
                    ),
                }
            })
        })
        .flatten();

    let activities = (detail || encounter)
        .then(|| activities(hazard, &mut unavailable))
        .flatten();
    let content = (detail || encounter)
        .then(|| general_content(hazard))
        .flatten();

    // Search compact intentionally records only absence that affects compact fields.
    if compact {
        typed_fact(&hazard.level, "level", None, &mut unavailable);
    } else {
        typed_fact(&hazard.level, "level", None, &mut unavailable);
        typed_fact(&hazard.rarity, "rarity", None, &mut unavailable);
        typed_fact(&hazard.traits, "traits", None, &mut unavailable);
        typed_fact(&hazard.publication, "publication", None, &mut unavailable);
        for unsupported in &hazard.unsupported_fields {
            push_unavailable(
                &mut unavailable,
                HazardSurfaceUnavailableStateView::Unsupported,
                unsupported_field_name(&unsupported.field),
                None,
            );
        }
        for issue in &source_metadata.issues {
            push_unavailable_message(
                &mut unavailable,
                HazardSurfaceUnavailableStateView::Unsupported,
                issue.field_key(),
                issue.component_id(),
                issue.message(),
            );
        }
    }

    HazardSurfaceView {
        teaser,
        complexity,
        size,
        emits_sound,
        detection,
        defenses,
        lifecycle,
        activities,
        content,
        relationships: None,
        unavailable_fields: non_empty(unavailable),
        provenance: HazardSurfaceProvenanceView {
            source_path: hazard.provenance.source_path.clone(),
            source_contract_version: hazard.provenance.source_contract_version.clone(),
            source_system_version: hazard.provenance.source_system_version.clone(),
            source_upstream_commit: hazard.provenance.source_upstream_commit.clone(),
            convenience_rule_id: conveniences.rule_id.to_string(),
            convenience_rule_version: conveniences.rule_version,
            image: provenance_text(&hazard.provenance.image),
            publication_license: publication_license(hazard),
            source_metadata: source_metadata
                .facts
                .into_iter()
                .map(source_metadata_fact)
                .collect(),
        },
    }
}

fn source_metadata_fact(fact: HazardSourceMetadataFact) -> HazardSurfaceSourceMetadataFactView {
    match fact {
        HazardSourceMetadataFact::TokenName { value } => {
            HazardSurfaceSourceMetadataFactView::TokenName {
                value: source_fact(value, |value| value),
            }
        }
        HazardSourceMetadataFact::HasHealth { value } => {
            HazardSurfaceSourceMetadataFactView::HasHealth {
                value: source_fact(value, |value| value),
            }
        }
        HazardSourceMetadataFact::TemporaryMaximum { value } => {
            HazardSurfaceSourceMetadataFactView::TemporaryMaximum {
                value: source_fact(value, |value| value),
            }
        }
        HazardSourceMetadataFact::SaveDetail { save, value } => {
            HazardSurfaceSourceMetadataFactView::SaveDetail {
                save: match save {
                    HazardSaveKind::Fortitude => HazardSurfaceSaveKindView::Fortitude,
                    HazardSaveKind::Reflex => HazardSurfaceSaveKindView::Reflex,
                    HazardSaveKind::Will => HazardSurfaceSaveKindView::Will,
                },
                value: source_fact(value, |value| value),
            }
        }
        HazardSourceMetadataFact::ItemRarity { entity_id, value } => {
            HazardSurfaceSourceMetadataFactView::ItemRarity {
                entity_id: entity_id.as_str().to_string(),
                value: source_fact(value, |value| value.as_str().to_string()),
            }
        }
        HazardSourceMetadataFact::ItemLineage { entity_id, value } => {
            HazardSurfaceSourceMetadataFactView::ItemLineage {
                entity_id: entity_id.as_str().to_string(),
                value: source_fact(value, |value| {
                    atlas_app_model::HazardSurfaceItemLineageView {
                        compendium_source: Box::new(source_fact(
                            value.compendium_source,
                            |value| value,
                        )),
                    }
                }),
            }
        }
        HazardSourceMetadataFact::StrikeAttack { entity_id, value } => {
            HazardSurfaceSourceMetadataFactView::StrikeAttack {
                entity_id: entity_id.as_str().to_string(),
                value: source_fact(value, |value| value),
            }
        }
        HazardSourceMetadataFact::StrikeWeaponType { entity_id, value } => {
            HazardSurfaceSourceMetadataFactView::StrikeWeaponType {
                entity_id: entity_id.as_str().to_string(),
                value: source_fact(value, |value| match value {
                    atlas_record::HazardSourceAttackMode::Melee => {
                        atlas_app_model::HazardSurfaceAttackModeView::Melee
                    }
                    atlas_record::HazardSourceAttackMode::Ranged => {
                        atlas_app_model::HazardSurfaceAttackModeView::Ranged
                    }
                }),
            }
        }
        HazardSourceMetadataFact::StrikeAttackEffectsCustom { entity_id, value } => {
            HazardSurfaceSourceMetadataFactView::StrikeAttackEffectsCustom {
                entity_id: entity_id.as_str().to_string(),
                value: source_fact(value, |value| value),
            }
        }
    }
}

fn source_fact<T, U>(
    fact: HazardFact<T>,
    map: impl FnOnce(T) -> U,
) -> HazardSurfaceSourceFactView<U> {
    let source_path = fact.provenance.relative_source_path;
    match fact.value {
        FactValue::Missing => HazardSurfaceSourceFactView::Missing { source_path },
        FactValue::Null => HazardSurfaceSourceFactView::Null { source_path },
        FactValue::Value(HazardSourceValue::Typed(value)) => HazardSurfaceSourceFactView::Typed {
            source_path,
            value: map(value),
        },
        FactValue::Value(HazardSourceValue::Unsupported(value)) => {
            HazardSurfaceSourceFactView::Unsupported {
                source_path,
                exact_json: value.exact_json,
                expected_shape: expected_shape(value.expected_shape),
                actual_shape: source_shape(value.actual_shape),
            }
        }
    }
}

fn expected_shape(
    value: atlas_record::HazardExpectedShape,
) -> atlas_app_model::HazardSurfaceExpectedShapeView {
    use atlas_app_model::HazardSurfaceExpectedShapeView as View;
    match value {
        atlas_record::HazardExpectedShape::Any => View::Any,
        atlas_record::HazardExpectedShape::Boolean => View::Boolean,
        atlas_record::HazardExpectedShape::Integer => View::Integer,
        atlas_record::HazardExpectedShape::String => View::String,
        atlas_record::HazardExpectedShape::StringOrBoolean => View::StringOrBoolean,
        atlas_record::HazardExpectedShape::StringOrArray => View::StringOrArray,
        atlas_record::HazardExpectedShape::Array => View::Array,
        atlas_record::HazardExpectedShape::Object => View::Object,
        atlas_record::HazardExpectedShape::ClosedVocabulary => View::ClosedVocabulary,
        atlas_record::HazardExpectedShape::RichDocument => View::RichDocument,
    }
}

fn source_shape(
    value: atlas_record::HazardSourceShape,
) -> atlas_app_model::HazardSurfaceSourceShapeView {
    use atlas_app_model::HazardSurfaceSourceShapeView as View;
    match value {
        atlas_record::HazardSourceShape::Missing => View::Missing,
        atlas_record::HazardSourceShape::Null => View::Null,
        atlas_record::HazardSourceShape::Boolean => View::Boolean,
        atlas_record::HazardSourceShape::Number => View::Number,
        atlas_record::HazardSourceShape::String => View::String,
        atlas_record::HazardSourceShape::Array => View::Array,
        atlas_record::HazardSourceShape::Object => View::Object,
    }
}

fn activities(
    hazard: &HazardRecord,
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
) -> Option<Vec<HazardSurfaceActivityView>> {
    let embedded = typed_fact(
        &hazard.embedded_entities,
        "embedded_entities",
        None,
        unavailable,
    )?;
    let mut occurrences = embedded.occurrences.iter().collect::<Vec<_>>();
    occurrences.sort_by_key(|occurrence| (occurrence.authored_order, occurrence.source_ordinal));
    non_empty(
        occurrences
            .into_iter()
            .filter_map(|occurrence| {
                let entity = embedded
                    .entities
                    .iter()
                    .find(|entity| entity.id == occurrence.entity_id)?;
                Some(activity(hazard, occurrence, entity, unavailable))
            })
            .collect(),
    )
}

fn activity(
    hazard: &HazardRecord,
    occurrence: &HazardEntityOccurrence,
    entity: &HazardEntity,
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
) -> HazardSurfaceActivityView {
    let component_id = Some(occurrence.id.as_str());
    let (
        activity_type,
        child_type,
        common,
        action_cost,
        frequency,
        category,
        death_note,
        self_effect,
        attack_bonus,
        attack_effects,
        damage,
    ) = match &entity.capability {
        HazardCapability::Action(action) => {
            let action_type = typed_fact(
                &action.action_type,
                "activity.action_type",
                component_id,
                unavailable,
            )
            .copied();
            let action_cost = match action_type {
                Some(HazardActionType::Action) => typed_fact(
                    &action.actions,
                    "activity.action_count",
                    component_id,
                    unavailable,
                )
                .map(|count| CreatureSurfaceActionCostView::Actions {
                    count: count.value(),
                }),
                Some(HazardActionType::Reaction) => Some(CreatureSurfaceActionCostView::Reaction),
                Some(HazardActionType::Free) => Some(CreatureSurfaceActionCostView::FreeAction),
                Some(HazardActionType::Passive) => Some(CreatureSurfaceActionCostView::Passive),
                None => None,
            };
            let frequency = typed_fact(
                &action.frequency,
                "activity.frequency",
                component_id,
                unavailable,
            )
            .map(|frequency| HazardSurfaceFrequencyView {
                maximum: typed_fact(
                    &frequency.maximum,
                    "activity.frequency.maximum",
                    component_id,
                    unavailable,
                )
                .copied(),
                period: typed_fact(
                    &frequency.per,
                    "activity.frequency.period",
                    component_id,
                    unavailable,
                )
                .map(|period| frequency_period(*period)),
                value: typed_fact(
                    &frequency.value,
                    "activity.frequency.value",
                    component_id,
                    unavailable,
                )
                .copied(),
            });
            let self_effect = typed_fact(
                &action.self_effect,
                "activity.self_effect",
                component_id,
                unavailable,
            )
            .map(|effect| HazardSurfaceSelfEffectView {
                target_uuid: typed_fact(
                    &effect.target_uuid,
                    "activity.self_effect.target_uuid",
                    component_id,
                    unavailable,
                )
                .cloned(),
                label: typed_fact(
                    &effect.label,
                    "activity.self_effect.label",
                    component_id,
                    unavailable,
                )
                .cloned(),
            });
            (
                HazardSurfaceActivityTypeView::Action,
                None,
                &action.common,
                action_cost,
                frequency,
                typed_fact(
                    &action.category,
                    "activity.category",
                    component_id,
                    unavailable,
                )
                .map(|category| format!("{category:?}").to_ascii_lowercase()),
                typed_fact(
                    &action.death_note,
                    "activity.death_note",
                    component_id,
                    unavailable,
                )
                .copied(),
                self_effect,
                None,
                None,
                None,
            )
        }
        HazardCapability::Strike(strike) => {
            let damage = typed_fact(
                &strike.damage_rolls,
                "activity.damage",
                component_id,
                unavailable,
            )
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| {
                        let damage_id = value.source_key.clone();
                        let formula = typed_fact(
                            &value.damage,
                            "activity.damage.formula",
                            Some(&damage_id),
                            unavailable,
                        )?
                        .clone();
                        Some(CreatureSurfaceDamageView {
                            damage_id,
                            formula: Some(formula),
                            damage_type: typed_fact(
                                &value.damage_type,
                                "activity.damage.type",
                                component_id,
                                unavailable,
                            )
                            .cloned(),
                            category: typed_fact(
                                &value.category,
                                "activity.damage.category",
                                component_id,
                                unavailable,
                            )
                            .map(|category| format!("{category:?}").to_ascii_lowercase()),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .and_then(non_empty);
            (
                HazardSurfaceActivityTypeView::Strike,
                None,
                &strike.common,
                None,
                None,
                None,
                None,
                None,
                typed_fact(
                    &strike.bonus,
                    "activity.attack_bonus",
                    component_id,
                    unavailable,
                )
                .copied(),
                typed_fact(
                    &strike.attack_effects,
                    "activity.attack_effects",
                    component_id,
                    unavailable,
                )
                .cloned()
                .and_then(non_empty),
                damage,
            )
        }
        HazardCapability::Condition(condition) => (
            HazardSurfaceActivityTypeView::Condition,
            None,
            &condition.common,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        HazardCapability::Effect(effect) => (
            HazardSurfaceActivityTypeView::Effect,
            None,
            &effect.common,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        HazardCapability::UnsupportedChild(child) => {
            push_unavailable(
                unavailable,
                HazardSurfaceUnavailableStateView::Unsupported,
                "activity.child_type",
                component_id,
            );
            (
                HazardSurfaceActivityTypeView::UnsupportedChild,
                Some(child.child_type.clone()),
                &child.common,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }
    };
    let traits = typed_fact(&common.traits, "activity.traits", component_id, unavailable)
        .map(|traits| {
            traits
                .iter()
                .map(|value| value.as_str().to_string())
                .collect::<Vec<_>>()
        })
        .and_then(non_empty);
    typed_fact(
        &common.description,
        "activity.description",
        component_id,
        unavailable,
    );
    push_unsupported_fields(entity_unsupported_fields(entity), component_id, unavailable);
    let strike_action_cost = project_hazard_strike_action_cost(entity);
    let attack_mode = strike_action_cost
        .as_ref()
        .and_then(|_| project_hazard_attack_mode(entity))
        .map(|projection| match projection.mode {
            HazardAttackMode::Melee => HazardSurfaceAttackModeView::Melee,
            HazardAttackMode::Ranged => HazardSurfaceAttackModeView::Ranged,
        });
    let action_cost = action_cost.or_else(|| {
        strike_action_cost.map(|projection| CreatureSurfaceActionCostView::Actions {
            count: projection.cost.value(),
        })
    });
    HazardSurfaceActivityView {
        occurrence_id: occurrence.id.as_str().to_string(),
        entity_id: occurrence.entity_id.as_str().to_string(),
        authored_order: occurrence.authored_order,
        source_ordinal: occurrence.source_ordinal,
        identity_stability: match occurrence.identity_stability {
            HazardOccurrenceIdentityStability::StableSourceIdentity => {
                HazardSurfaceOccurrenceIdentityStabilityView::StableSourceIdentity
            }
            HazardOccurrenceIdentityStability::UnstableAuthoredOrdinal => {
                HazardSurfaceOccurrenceIdentityStabilityView::UnstableAuthoredOrdinal
            }
        },
        label: typed_fact(
            &occurrence.contextual_label,
            "activity.contextual_label",
            component_id,
            unavailable,
        )
        .cloned()
        .unwrap_or_else(|| entity.label.clone()),
        activity_type,
        child_type,
        traits,
        attack_mode,
        action_cost,
        frequency,
        category,
        death_note,
        self_effect,
        attack_bonus,
        attack_effects,
        damage,
        rules: rules(common, component_id, unavailable),
        content: activity_content(hazard, occurrence, entity),
    }
}

fn entity_unsupported_fields(entity: &HazardEntity) -> &[atlas_record::HazardUnsupportedFact] {
    match &entity.capability {
        HazardCapability::Action(value) => &value.unsupported_fields,
        HazardCapability::Strike(value) => &value.unsupported_fields,
        HazardCapability::Condition(value) => &value.unsupported_fields,
        HazardCapability::Effect(value) => &value.unsupported_fields,
        HazardCapability::UnsupportedChild(value) => &value.unsupported_fields,
    }
}

fn rules(
    common: &HazardItemCommon,
    component_id: Option<&str>,
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
) -> Option<Vec<HazardSurfaceRuleView>> {
    typed_fact(&common.rules, "activity.rules", component_id, unavailable)
        .map(|rules| {
            rules
                .iter()
                .map(|rule| match rule {
                    HazardRuleElement::Immunity(rule) => HazardSurfaceRuleView::Immunity {
                        authored_order: rule.authored_order,
                        mode: typed_fact(
                            &rule.mode,
                            "activity.rule.immunity.mode",
                            component_id,
                            unavailable,
                        )
                        .map(|mode| rule_mode(*mode)),
                        immunity_types: typed_fact(
                            &rule.immunity_types,
                            "activity.rule.immunity.types",
                            component_id,
                            unavailable,
                        )
                        .map(|types| match types {
                            HazardRuleType::Single(value) => vec![value.clone()],
                            HazardRuleType::Multiple(values) => values.clone(),
                        })
                        .unwrap_or_default(),
                    },
                    HazardRuleElement::ActiveEffectLike(rule) => {
                        HazardSurfaceRuleView::ActiveEffectLike {
                            authored_order: rule.authored_order,
                            mode: typed_fact(
                                &rule.mode,
                                "activity.rule.active_effect_like.mode",
                                component_id,
                                unavailable,
                            )
                            .map(|mode| rule_mode(*mode)),
                            path: typed_fact(
                                &rule.path,
                                "activity.rule.active_effect_like.path",
                                component_id,
                                unavailable,
                            )
                            .cloned(),
                            value: typed_fact(
                                &rule.value,
                                "activity.rule.active_effect_like.value",
                                component_id,
                                unavailable,
                            )
                            .copied(),
                        }
                    }
                    HazardRuleElement::Aura(rule) => HazardSurfaceRuleView::Aura {
                        authored_order: rule.authored_order,
                        radius: typed_fact(
                            &rule.radius,
                            "activity.rule.aura.radius",
                            component_id,
                            unavailable,
                        )
                        .copied(),
                        slug: typed_fact(
                            &rule.slug,
                            "activity.rule.aura.slug",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                        traits: typed_fact(
                            &rule.traits,
                            "activity.rule.aura.traits",
                            component_id,
                            unavailable,
                        )
                        .map(|traits| {
                            traits
                                .iter()
                                .map(|value| value.as_str().to_string())
                                .collect()
                        })
                        .unwrap_or_default(),
                    },
                    HazardRuleElement::DamageDice(rule) => HazardSurfaceRuleView::DamageDice {
                        authored_order: rule.authored_order,
                        critical: typed_fact(
                            &rule.critical,
                            "activity.rule.damage_dice.critical",
                            component_id,
                            unavailable,
                        )
                        .copied(),
                        dice_number: typed_fact(
                            &rule.dice_number,
                            "activity.rule.damage_dice.dice_number",
                            component_id,
                            unavailable,
                        )
                        .copied(),
                        die_size: typed_fact(
                            &rule.die_size,
                            "activity.rule.damage_dice.die_size",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                        damage_type: typed_fact(
                            &rule.damage_type,
                            "activity.rule.damage_dice.damage_type",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                        selector: typed_fact(
                            &rule.selector,
                            "activity.rule.damage_dice.selector",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                    },
                    HazardRuleElement::FlatModifier(rule) => HazardSurfaceRuleView::FlatModifier {
                        authored_order: rule.authored_order,
                        critical: typed_fact(
                            &rule.critical,
                            "activity.rule.flat_modifier.critical",
                            component_id,
                            unavailable,
                        )
                        .copied(),
                        damage_type: typed_fact(
                            &rule.damage_type,
                            "activity.rule.flat_modifier.damage_type",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                        selector: typed_fact(
                            &rule.selector,
                            "activity.rule.flat_modifier.selector",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                        value: typed_fact(
                            &rule.value,
                            "activity.rule.flat_modifier.value",
                            component_id,
                            unavailable,
                        )
                        .copied(),
                    },
                    HazardRuleElement::Note(rule) => HazardSurfaceRuleView::Note {
                        authored_order: rule.authored_order,
                        outcomes: typed_fact(
                            &rule.outcomes,
                            "activity.rule.note.outcomes",
                            component_id,
                            unavailable,
                        )
                        .cloned()
                        .unwrap_or_default(),
                        selector: typed_fact(
                            &rule.selector,
                            "activity.rule.note.selector",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                        text: rich_document(
                            &rule.text,
                            "activity.rule.note.text",
                            component_id,
                            unavailable,
                        ),
                        title: typed_fact(
                            &rule.title,
                            "activity.rule.note.title",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                        visibility: typed_fact(
                            &rule.visibility,
                            "activity.rule.note.visibility",
                            component_id,
                            unavailable,
                        )
                        .cloned(),
                    },
                    HazardRuleElement::Unsupported(rule) => {
                        push_unavailable(
                            unavailable,
                            HazardSurfaceUnavailableStateView::Unsupported,
                            "activity.rule",
                            component_id,
                        );
                        HazardSurfaceRuleView::Unsupported {
                            authored_order: rule.authored_order,
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .and_then(non_empty)
}

fn rule_mode(mode: HazardRuleMode) -> String {
    match mode {
        HazardRuleMode::Add => "add",
        HazardRuleMode::Remove => "remove",
        HazardRuleMode::Override => "override",
    }
    .to_string()
}

fn activity_content(
    hazard: &HazardRecord,
    occurrence: &HazardEntityOccurrence,
    entity: &HazardEntity,
) -> Option<Vec<atlas_app_model::CreatureSurfaceContentView>> {
    let mut documents = hazard
        .content
        .documents
        .iter()
        .filter(|document| {
            matches!(&document.owner, ContentOwner::HazardOccurrence(id) if id == &occurrence.id)
                || matches!(&document.owner, ContentOwner::HazardEntity(id) if id == &entity.id)
        })
        .collect::<Vec<_>>();
    documents.sort_by_key(|document| (document.authored_order, document.id.content_key.as_str()));
    non_empty(
        documents
            .into_iter()
            .filter_map(crate::surface::content_view)
            .collect(),
    )
}

fn general_content(
    hazard: &HazardRecord,
) -> Option<Vec<atlas_app_model::CreatureSurfaceContentView>> {
    let mut documents = hazard
        .content
        .documents
        .iter()
        .filter(|document| {
            matches!(&document.owner, ContentOwner::Record(key) if key == &hazard.identity.record_key)
                && !matches!(
                    document.source_kind,
                    ContentSourceKind::Description
                        | ContentSourceKind::Disable
                        | ContentSourceKind::Routine
                        | ContentSourceKind::Reset
                        | ContentSourceKind::StealthDetails
                )
        })
        .collect::<Vec<_>>();
    documents.sort_by_key(|document| (document.authored_order, document.id.content_key.as_str()));
    non_empty(
        documents
            .into_iter()
            .filter_map(crate::surface::content_view)
            .collect(),
    )
}

fn iwr(
    fact: &HazardFact<Vec<HazardIwr>>,
    field: &str,
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
) -> Option<Vec<HazardSurfaceIwrView>> {
    typed_fact(fact, field, None, unavailable)
        .map(|values| {
            let mut values = values.iter().collect::<Vec<_>>();
            values.sort_by_key(|value| value.authored_order);
            values
                .into_iter()
                .filter_map(|value| {
                    let component_id = value.id.as_str();
                    let kind = typed_fact(
                        &value.iwr_type,
                        &format!("{field}.type"),
                        Some(component_id),
                        unavailable,
                    )?
                    .clone();
                    Some(HazardSurfaceIwrView {
                        component_id: component_id.to_string(),
                        authored_order: value.authored_order,
                        kind,
                        amount: typed_fact(
                            &value.value,
                            &format!("{field}.value"),
                            Some(component_id),
                            unavailable,
                        )
                        .copied(),
                        exceptions: typed_fact(
                            &value.exceptions,
                            &format!("{field}.exceptions"),
                            Some(component_id),
                            unavailable,
                        )
                        .cloned()
                        .unwrap_or_default(),
                        double_vs: typed_fact(
                            &value.double_vs,
                            &format!("{field}.double_vs"),
                            Some(component_id),
                            unavailable,
                        )
                        .cloned()
                        .unwrap_or_default(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .and_then(non_empty)
}

fn rich_document(
    fact: &HazardFact<RichDocument>,
    field: &str,
    component_id: Option<&str>,
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
) -> Option<Vec<atlas_app_model::CreatureSurfaceContentBlockView>> {
    typed_fact(fact, field, component_id, unavailable)
        .map(project_presentation_content)
        .map(crate::surface::project_content)
        .and_then(non_empty)
}

fn typed_fact<'a, T>(
    fact: &'a HazardFact<T>,
    field: &str,
    component_id: Option<&str>,
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
) -> Option<&'a T> {
    match &fact.value {
        FactValue::Missing => {
            push_unavailable(
                unavailable,
                HazardSurfaceUnavailableStateView::Missing,
                field,
                component_id,
            );
            None
        }
        FactValue::Null => {
            push_unavailable(
                unavailable,
                HazardSurfaceUnavailableStateView::Null,
                field,
                component_id,
            );
            None
        }
        FactValue::Value(HazardSourceValue::Typed(value)) => Some(value),
        FactValue::Value(HazardSourceValue::Unsupported(_)) => {
            push_unavailable(
                unavailable,
                HazardSurfaceUnavailableStateView::Unsupported,
                field,
                component_id,
            );
            None
        }
    }
}

fn push_unavailable(
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
    state: HazardSurfaceUnavailableStateView,
    field: &str,
    component_id: Option<&str>,
) {
    let message = match state {
        HazardSurfaceUnavailableStateView::Missing => {
            "This canonical hazard field was not authored."
        }
        HazardSurfaceUnavailableStateView::Null => {
            "This canonical hazard field was explicitly null."
        }
        HazardSurfaceUnavailableStateView::Unsupported => {
            "This authored hazard field is retained but not supported on this surface."
        }
    };
    push_unavailable_message(unavailable, state, field, component_id, message);
}

fn push_unavailable_message(
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
    state: HazardSurfaceUnavailableStateView,
    field: &str,
    component_id: Option<&str>,
    message: &str,
) {
    let value = HazardSurfaceUnavailableView {
        state,
        field: field.to_string(),
        component_id: component_id.map(str::to_string),
        message: message.to_string(),
    };
    if !unavailable.contains(&value) {
        unavailable.push(value);
    }
}

fn push_unsupported_fields(
    fields: &[atlas_record::HazardUnsupportedFact],
    component_id: Option<&str>,
    unavailable: &mut Vec<HazardSurfaceUnavailableView>,
) {
    for unsupported in fields {
        push_unavailable(
            unavailable,
            HazardSurfaceUnavailableStateView::Unsupported,
            unsupported_field_name(&unsupported.field),
            component_id,
        );
    }
}

fn publication_license(hazard: &HazardRecord) -> HazardSurfaceProvenanceTextView {
    match &hazard.publication.value {
        FactValue::Missing => HazardSurfaceProvenanceTextView::Missing,
        FactValue::Null => HazardSurfaceProvenanceTextView::Null,
        FactValue::Value(HazardSourceValue::Unsupported(_)) => {
            HazardSurfaceProvenanceTextView::Unsupported
        }
        FactValue::Value(HazardSourceValue::Typed(publication)) => {
            provenance_text_with(&publication.license, PublicationLicense::as_str)
        }
    }
}

fn provenance_text(fact: &HazardFact<String>) -> HazardSurfaceProvenanceTextView {
    provenance_text_with(fact, String::as_str)
}

fn provenance_text_with<T>(
    fact: &HazardFact<T>,
    value: impl FnOnce(&T) -> &str,
) -> HazardSurfaceProvenanceTextView {
    match &fact.value {
        FactValue::Missing => HazardSurfaceProvenanceTextView::Missing,
        FactValue::Null => HazardSurfaceProvenanceTextView::Null,
        FactValue::Value(HazardSourceValue::Unsupported(_)) => {
            HazardSurfaceProvenanceTextView::Unsupported
        }
        FactValue::Value(HazardSourceValue::Typed(fact)) => {
            HazardSurfaceProvenanceTextView::Value {
                value: value(fact).to_string(),
            }
        }
    }
}

fn frequency_period(value: HazardFrequencyInterval) -> String {
    match value {
        HazardFrequencyInterval::Turn => "turn",
        HazardFrequencyInterval::Round => "round",
        HazardFrequencyInterval::OneMinute => "PT1M",
        HazardFrequencyInterval::TenMinutes => "PT10M",
        HazardFrequencyInterval::OneHour => "PT1H",
        HazardFrequencyInterval::TwentyFourHours => "PT24H",
        HazardFrequencyInterval::Day => "day",
        HazardFrequencyInterval::Week => "P1W",
        HazardFrequencyInterval::Month => "P1M",
        HazardFrequencyInterval::Year => "P1Y",
    }
    .to_string()
}

fn non_empty<T>(values: Vec<T>) -> Option<Vec<T>> {
    (!values.is_empty()).then_some(values)
}

fn unsupported_field_name(field: &HazardUnsupportedField) -> &'static str {
    match field {
        HazardUnsupportedField::HazardUnexpected(_) => "hazard.unexpected",
        HazardUnsupportedField::ActionUnexpected(_) => "activity.action.unexpected",
        HazardUnsupportedField::StrikeUnexpected(_) => "activity.strike.unexpected",
        HazardUnsupportedField::ConditionUnexpected(_) => "activity.condition.unexpected",
        HazardUnsupportedField::EffectUnexpected(_) => "activity.effect.unexpected",
        HazardUnsupportedField::UnsupportedChildField(_) => "activity.unsupported_child.field",
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::{Rarity, RecordKey, RecordKind};
    use atlas_record::{
        FactValue, HazardActionCapability, HazardActionCount, HazardActionType, HazardCapability,
        HazardDefenseSourceMetadata, HazardDefenses, HazardDiagnosticCode, HazardEmbeddedEntities,
        HazardEntity, HazardEntityFamily, HazardEntityId, HazardEntityOccurrence,
        HazardEntitySourceIdentity, HazardExpectedShape, HazardFact, HazardHitPointSourceMetadata,
        HazardHitPoints, HazardIdentity, HazardItemCommon, HazardItemLineage, HazardOccurrenceId,
        HazardOccurrenceIdentityStability, HazardProvenance, HazardRecord,
        HazardSaveSourceMetadata, HazardSaves, HazardSourceAttackMode, HazardSourceId,
        HazardSourceShape, HazardSourceValue, HazardStrikeCapability, HazardStrikeSourceMetadata,
        HazardTokenSourceMetadata, HazardTrait, HazardUnsupportedOwner, HazardUnsupportedValue,
        OwnedRichContent, RecordBody, RetrievedRecord,
    };
    use atlas_search::RemasterLinksResult;

    use super::{RecordSurfaceProfileView, hazard_surface};

    #[test]
    fn app_hazard_diagnostics_consume_only_actionable_canonical_source_metadata_issues() {
        let benign = hazard_surface(
            &source_metadata_fixture(false),
            RecordSurfaceProfileView::RecordDetail,
            None,
        );
        assert!(source_metadata_fields(&benign).is_empty());
        let benign_provenance = serde_json::to_value(&benign.provenance.source_metadata)
            .expect("typed hazard source provenance");
        let benign_facts = benign_provenance.as_array().expect("metadata fact array");
        assert_eq!(benign_facts.len(), 11);
        assert!(benign_facts.iter().any(|fact| {
            fact["field"] == "has_health"
                && fact["value"]["state"] == "typed"
                && fact["value"]["value"] == true
                && fact["value"]["source_path"] == "/system/attributes/hasHealth"
        }));
        assert!(benign_facts.iter().any(|fact| {
            fact["field"] == "item_rarity"
                && fact["entity_id"] == "metadata-strike"
                && fact["value"]["value"] == "common"
        }));
        assert!(benign_facts.iter().any(|fact| {
            fact["field"] == "item_lineage"
                && fact["value"]["value"]["compendium_source"]["value"]
                    == "Compendium.pf2e.hazards.Item.fixture"
        }));

        let actionable = hazard_surface(
            &source_metadata_fixture(true),
            RecordSurfaceProfileView::RecordDetail,
            None,
        );
        let fields = source_metadata_fields(&actionable);
        assert_eq!(fields.len(), 7);
        for expected in [
            "provenance.token.name",
            "defenses.source_metadata.has_health",
            "defenses.hit_points.source_metadata.temporary_maximum",
            "defenses.saves.source_metadata.fortitude_detail",
            "activity.source_metadata.rarity",
            "activity.strike.source_metadata.weapon_type",
            "activity.strike.source_metadata.attack_effects_custom",
        ] {
            assert_eq!(
                fields
                    .iter()
                    .filter(|field| field.as_str() == expected)
                    .count(),
                1,
                "{expected} should reach the app once"
            );
        }
        let unavailable = actionable
            .unavailable_fields
            .as_ref()
            .expect("actionable diagnostics");
        assert!(
            unavailable
                .iter()
                .filter(|entry| {
                    entry.field.contains("source_metadata")
                        || entry.field == "provenance.token.name"
                })
                .all(|entry| {
                    !entry.message.contains("/system/")
                        && !entry.message.contains("/items/")
                        && entry.state
                            == atlas_app_model::HazardSurfaceUnavailableStateView::Unsupported
                })
        );
        let actionable_provenance = serde_json::to_value(&actionable.provenance.source_metadata)
            .expect("actionable source provenance");
        assert!(
            actionable_provenance
                .as_array()
                .expect("metadata fact array")
                .iter()
                .any(|fact| {
                    fact["field"] == "token_name"
                        && fact["value"]["state"] == "unsupported"
                        && fact["value"]["exact_json"] == "17"
                        && fact["value"]["actual_shape"] == "number"
                })
        );

        let benign = record_surface_for(source_metadata_fixture(false));
        assert!(benign.issues.is_none());
        let actionable = record_surface_for(source_metadata_fixture(true));
        let issues = actionable.issues.expect("actionable outer issues");
        assert_eq!(issues.len(), 7);
        assert!(issues.iter().all(|issue| {
            !issue.message.contains("/system/")
                && !issue.message.contains("/items/")
                && !issue.message.contains("metadata-strike")
        }));
    }

    #[test]
    fn hazard_strike_activity_uses_only_canonical_mode_and_action_cost_projections() {
        let acid = hazard_activity_fixture(
            "Acid Spray Fountain",
            "Acid Spray",
            typed(Vec::new(), "/items/0/system/traits/value"),
            HazardSourceAttackMode::Melee,
        );
        let acid = only_activity(hazard_surface(
            &acid,
            RecordSurfaceProfileView::RecordDetail,
            None,
        ));
        assert_eq!(
            acid.attack_mode,
            Some(atlas_app_model::HazardSurfaceAttackModeView::Melee)
        );
        assert_eq!(
            acid.action_cost,
            Some(atlas_app_model::CreatureSurfaceActionCostView::Actions { count: 1 })
        );

        let dragon = hazard_activity_fixture(
            "Dragon Pillar",
            "Eye Beam",
            typed(
                vec![HazardTrait::new("range-120").expect("trait")],
                "/items/0/system/traits/value",
            ),
            HazardSourceAttackMode::Ranged,
        );
        let dragon = only_activity(hazard_surface(
            &dragon,
            RecordSurfaceProfileView::RecordDetail,
            None,
        ));
        assert_eq!(
            dragon.attack_mode,
            Some(atlas_app_model::HazardSurfaceAttackModeView::Ranged)
        );
        assert_eq!(
            dragon.action_cost,
            Some(atlas_app_model::CreatureSurfaceActionCostView::Actions { count: 1 })
        );

        let mut unsupported = source_metadata_fixture(false);
        let unsupported_owner = embedded_mut(&mut unsupported).entities[0].id.clone();
        let HazardCapability::Strike(strike) =
            &mut embedded_mut(&mut unsupported).entities[0].capability
        else {
            panic!("strike fixture")
        };
        strike.common.traits = malformed(
            "/items/0/system/traits/value",
            "17",
            HazardUnsupportedOwner::Entity(unsupported_owner),
        );
        let unsupported =
            hazard_surface(&unsupported, RecordSurfaceProfileView::RecordDetail, None);
        let unsupported_activity = unsupported
            .activities
            .as_ref()
            .and_then(|activities| activities.first())
            .expect("activity");
        assert_eq!(unsupported_activity.attack_mode, None);
        assert_eq!(
            unsupported_activity.action_cost,
            Some(atlas_app_model::CreatureSurfaceActionCostView::Actions { count: 1 })
        );
        assert_eq!(
            unsupported
                .unavailable_fields
                .iter()
                .flatten()
                .filter(|issue| issue.field == "activity.traits")
                .count(),
            1
        );

        let mut missing_traits = source_metadata_fixture(false);
        let HazardCapability::Strike(strike) =
            &mut embedded_mut(&mut missing_traits).entities[0].capability
        else {
            panic!("strike fixture")
        };
        strike.common.traits = missing("/items/0/system/traits/value");
        let missing_traits = only_activity(hazard_surface(
            &missing_traits,
            RecordSurfaceProfileView::RecordDetail,
            None,
        ));
        assert_eq!(missing_traits.attack_mode, None);
        assert_eq!(
            missing_traits.action_cost,
            Some(atlas_app_model::CreatureSurfaceActionCostView::Actions { count: 1 })
        );

        let mut authored_action = source_metadata_fixture(false);
        let embedded = embedded_mut(&mut authored_action);
        let common = match &embedded.entities[0].capability {
            HazardCapability::Strike(strike) => strike.common.clone(),
            _ => panic!("strike fixture"),
        };
        embedded.entities[0].family = HazardEntityFamily::Action;
        embedded.entities[0].capability =
            HazardCapability::Action(Box::new(HazardActionCapability {
                common,
                action_type: typed(HazardActionType::Action, "/items/0/system/actionType/value"),
                actions: typed(HazardActionCount::Two, "/items/0/system/actions/value"),
                category: missing("/items/0/system/category"),
                death_note: missing("/items/0/system/deathNote"),
                frequency: missing("/items/0/system/frequency"),
                self_effect: missing("/items/0/system/selfEffect"),
                unsupported_fields: Vec::new(),
            }));
        embedded.occurrences[0].family = HazardEntityFamily::Action;
        let authored_action = only_activity(hazard_surface(
            &authored_action,
            RecordSurfaceProfileView::RecordDetail,
            None,
        ));
        assert_eq!(authored_action.attack_mode, None);
        assert_eq!(
            authored_action.action_cost,
            Some(atlas_app_model::CreatureSurfaceActionCostView::Actions { count: 2 })
        );

        let mut mismatched_family = source_metadata_fixture(false);
        let embedded = embedded_mut(&mut mismatched_family);
        embedded.entities[0].family = HazardEntityFamily::Action;
        embedded.occurrences[0].family = HazardEntityFamily::Action;
        let mismatched_family = only_activity(hazard_surface(
            &mismatched_family,
            RecordSurfaceProfileView::RecordDetail,
            None,
        ));
        assert_eq!(mismatched_family.attack_mode, None);
        assert_eq!(mismatched_family.action_cost, None);
    }

    fn only_activity(
        view: atlas_app_model::HazardSurfaceView,
    ) -> atlas_app_model::HazardSurfaceActivityView {
        view.activities
            .expect("activities")
            .into_iter()
            .next()
            .expect("activity")
    }

    fn embedded_mut(hazard: &mut HazardRecord) -> &mut HazardEmbeddedEntities {
        let FactValue::Value(HazardSourceValue::Typed(embedded)) =
            &mut hazard.embedded_entities.value
        else {
            panic!("typed embedded fixture")
        };
        embedded
    }

    fn hazard_activity_fixture(
        record_name: &str,
        activity_name: &str,
        traits: HazardFact<Vec<HazardTrait>>,
        source_mode: HazardSourceAttackMode,
    ) -> HazardRecord {
        let mut hazard = source_metadata_fixture(false);
        hazard.identity.name = record_name.to_string();
        let embedded = embedded_mut(&mut hazard);
        embedded.occurrences[0].contextual_label =
            typed(activity_name.to_string(), "/items/0/name");
        let HazardCapability::Strike(strike) = &mut embedded.entities[0].capability else {
            panic!("strike fixture")
        };
        strike.common.traits = traits;
        strike.source_metadata.weapon_type = typed(source_mode, "/items/0/system/weaponType/value");
        hazard
    }

    fn record_surface_for(hazard: HazardRecord) -> atlas_app_model::RecordSurfaceView {
        let fixture = crate::test_support::encounter_fixture_worker();
        let mut record = fixture
            .worker
            .get_records(vec![
                RecordKey::parse("actions:testAction1").expect("fixture key"),
            ])
            .expect("fixture record should load")
            .pop()
            .expect("fixture record should exist");
        record.record.identity.key = hazard.identity.record_key.clone();
        record.record.identity.name = hazard.identity.name.clone();
        record.record.classification.kind = RecordKind::Hazard;
        record.body = Some(RecordBody::Hazard(hazard));
        record_surface(&record)
    }

    fn record_surface(record: &RetrievedRecord) -> atlas_app_model::RecordSurfaceView {
        let remaster_lookup =
            crate::retrieval::VerifiedRemasterLookup::from_test_result(RemasterLinksResult {
                seed: record.clone(),
                links: Vec::new(),
            });
        crate::surface::record_surface(
            record,
            RecordSurfaceProfileView::RecordDetail,
            None,
            None,
            &remaster_lookup,
        )
    }

    fn source_metadata_fields(view: &atlas_app_model::HazardSurfaceView) -> Vec<String> {
        view.unavailable_fields
            .iter()
            .flatten()
            .filter(|entry| {
                entry.field.contains("source_metadata") || entry.field == "provenance.token.name"
            })
            .map(|entry| entry.field.clone())
            .collect()
    }

    fn source_metadata_fixture(actionable: bool) -> HazardRecord {
        let record_key = RecordKey::parse("hazards:metadata-fixture").expect("record key");
        let entity_id = HazardEntityId::new("metadata-strike").expect("entity id");
        let rarity = if actionable {
            malformed(
                "/items/0/system/traits/rarity",
                "42",
                HazardUnsupportedOwner::Entity(entity_id.clone()),
            )
        } else {
            typed(Rarity::Common, "/items/0/system/traits/rarity")
        };
        let common = HazardItemCommon {
            description: missing("/items/0/system/description/value"),
            publication: missing("/items/0/system/publication"),
            rules: typed(Vec::new(), "/items/0/system/rules"),
            slug: missing("/items/0/system/slug"),
            traits: typed(
                vec![HazardTrait::new("range-120").expect("trait")],
                "/items/0/system/traits/value",
            ),
            rarity,
            lineage: typed(
                HazardItemLineage {
                    compendium_source: typed(
                        "Compendium.pf2e.hazards.Item.fixture".to_string(),
                        "/items/0/_stats/compendiumSource",
                    ),
                },
                "/items/0/_stats",
            ),
        };
        let strike = HazardEntity {
            id: entity_id.clone(),
            family: HazardEntityFamily::Strike,
            label: "Metadata Strike".to_string(),
            image: missing("/items/0/img"),
            source_identity: HazardEntitySourceIdentity::Stable {
                source_id: HazardSourceId::new("metadata-strike").expect("source id"),
            },
            capability: HazardCapability::Strike(Box::new(HazardStrikeCapability {
                common,
                bonus: typed(10, "/items/0/system/bonus/value"),
                attack_effects: typed(Vec::new(), "/items/0/system/attackEffects/value"),
                damage_rolls: typed(Vec::new(), "/items/0/system/damageRolls"),
                source_metadata: HazardStrikeSourceMetadata {
                    attack: missing("/items/0/system/attack/value"),
                    weapon_type: typed(
                        if actionable {
                            HazardSourceAttackMode::Melee
                        } else {
                            HazardSourceAttackMode::Ranged
                        },
                        "/items/0/system/weaponType/value",
                    ),
                    attack_effects_custom: typed(
                        if actionable {
                            "corrosive mist".to_string()
                        } else {
                            String::new()
                        },
                        "/items/0/system/attackEffects/custom",
                    ),
                },
                unsupported_fields: Vec::new(),
            })),
        };
        let token_name = if actionable {
            malformed(
                "/prototypeToken/name",
                "17",
                HazardUnsupportedOwner::Record(record_key.clone()),
            )
        } else {
            typed("Metadata Fixture".to_string(), "/prototypeToken/name")
        };
        HazardRecord {
            identity: HazardIdentity {
                record_key: record_key.clone(),
                source_id: HazardSourceId::new("metadata-fixture").expect("source id"),
                name: "Metadata Fixture".to_string(),
            },
            level: missing("/system/details/level/value"),
            rarity: missing("/system/traits/rarity"),
            traits: missing("/system/traits/value"),
            size: missing("/system/traits/size/value"),
            publication: missing("/system/details/publication"),
            complexity: missing("/system/details/isComplex"),
            detection: missing("/system/attributes/stealth"),
            defenses: typed(
                HazardDefenses {
                    armor_class: missing("/system/attributes/ac/value"),
                    hardness: missing("/system/attributes/hardness"),
                    hit_points: typed(
                        HazardHitPoints {
                            current: typed(12, "/system/attributes/hp/value"),
                            maximum: typed(12, "/system/attributes/hp/max"),
                            temporary: typed(0, "/system/attributes/hp/temp"),
                            details: missing("/system/attributes/hp/details"),
                            source_metadata: HazardHitPointSourceMetadata {
                                temporary_maximum: typed(
                                    if actionable { 9 } else { 0 },
                                    "/system/attributes/hp/tempmax",
                                ),
                            },
                        },
                        "/system/attributes/hp",
                    ),
                    saves: typed(
                        HazardSaves {
                            fortitude: missing("/system/saves/fortitude/value"),
                            reflex: missing("/system/saves/reflex/value"),
                            will: missing("/system/saves/will/value"),
                            source_metadata: HazardSaveSourceMetadata {
                                fortitude_detail: typed(
                                    if actionable {
                                        "against forced movement".to_string()
                                    } else {
                                        String::new()
                                    },
                                    "/system/saves/fortitude/saveDetail",
                                ),
                                reflex_detail: typed(
                                    String::new(),
                                    "/system/saves/reflex/saveDetail",
                                ),
                                will_detail: typed(String::new(), "/system/saves/will/saveDetail"),
                            },
                        },
                        "/system/saves",
                    ),
                    immunities: missing("/system/attributes/immunities"),
                    weaknesses: missing("/system/attributes/weaknesses"),
                    resistances: missing("/system/attributes/resistances"),
                    source_metadata: HazardDefenseSourceMetadata {
                        has_health: typed(!actionable, "/system/attributes/hasHealth"),
                    },
                },
                "/system/attributes",
            ),
            lifecycle: missing("/system/details"),
            emits_sound: missing("/system/attributes/emitsSound"),
            embedded_entities: typed(
                HazardEmbeddedEntities {
                    entities: vec![strike],
                    occurrences: vec![HazardEntityOccurrence {
                        id: HazardOccurrenceId::new("metadata-strike-occurrence")
                            .expect("occurrence id"),
                        owner_record_key: record_key.clone(),
                        entity_id,
                        family: HazardEntityFamily::Strike,
                        authored_order: 0,
                        source_sort: typed(0, "/items/0/sort"),
                        source_folder: missing("/items/0/folder"),
                        source_ordinal: 0,
                        contextual_label: typed("Metadata Strike".to_string(), "/items/0/name"),
                        identity_stability: HazardOccurrenceIdentityStability::StableSourceIdentity,
                    }],
                },
                "/items",
            ),
            content: OwnedRichContent::default(),
            relationships: Vec::new(),
            unsupported_fields: Vec::new(),
            provenance: HazardProvenance {
                source_path: "packs/hazards/metadata-fixture.json".to_string(),
                source_contract_version: "fixture".to_string(),
                source_system_version: "fixture".to_string(),
                source_upstream_commit: "fixture".to_string(),
                source_folder: missing("/folder"),
                image: missing("/img"),
                source_creature_type: missing("/system/creatureType"),
                source_status_effects: missing("/system/statusEffects"),
                actor_effects: missing("/effects"),
                token: typed(
                    HazardTokenSourceMetadata { name: token_name },
                    "/prototypeToken",
                ),
            },
        }
    }

    fn typed<T>(value: T, path: impl Into<String>) -> HazardFact<T> {
        HazardFact::source(FactValue::Value(HazardSourceValue::Typed(value)), path)
    }

    fn missing<T>(path: impl Into<String>) -> HazardFact<T> {
        HazardFact::source(FactValue::Missing, path)
    }

    fn malformed<T>(path: &str, exact_json: &str, owner: HazardUnsupportedOwner) -> HazardFact<T> {
        HazardFact::source(
            FactValue::Value(HazardSourceValue::Unsupported(HazardUnsupportedValue {
                exact_json: exact_json.to_string(),
                expected_shape: HazardExpectedShape::String,
                actual_shape: HazardSourceShape::Number,
                relative_source_path: path.to_string(),
                owner,
                diagnostic_code: HazardDiagnosticCode::UnexpectedShape,
            })),
            path,
        )
    }
}
