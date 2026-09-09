use std::collections::BTreeSet;

use atlas_app_model::{
    CreatureSurfaceDomainUnavailableView, CreatureSurfaceUnavailableDomainsView,
    CreatureSurfaceUnavailableFieldView, CreatureSurfaceUnavailableStateView,
    HazardSurfaceUnavailableStateView, HazardSurfaceView, RecordSurfaceIssueCodeView,
    RecordSurfaceIssuePlacementView, RecordSurfaceIssueSubjectView, RecordSurfaceIssueTargetView,
    RecordSurfaceIssueView, RecordSurfacePresentationView,
};
use atlas_record::{
    ConsumableDefinition, ConsumableFact, ConsumableOccurrenceSet, ConsumableSourceState,
    ConsumableSourceValue, FactIssueKind, FactPresentationDisposition, FactPresentationRole,
    FactPresentationState, FactRequirement, FactValue, HazardSourceMetadataField,
    HazardSourceMetadataIssueKind, RecordBody, RetrievedRecord, SpellPresentationIssue,
    SpellPresentationIssuePlacement, UnsupportedSourceValue, classify_fact_presentation,
    merge_spell_presentation_issues, project_hazard_source_metadata,
    project_spell_presentation_issues,
};

pub(crate) fn record_surface_issues(
    retrieved: &RetrievedRecord,
    presentation: &RecordSurfacePresentationView,
    selected_spell_issues: &[SpellPresentationIssue],
) -> Option<Vec<RecordSurfaceIssueView>> {
    let mut issues = Vec::new();
    match (presentation, &retrieved.body) {
        (RecordSurfacePresentationView::Creature { body }, Some(RecordBody::Creature(_))) => {
            creature_issues(body.unavailable_domains.as_ref(), &mut issues);
            consumable_occurrence_issues(&retrieved.consumable_occurrences, &mut issues);
        }
        (RecordSurfacePresentationView::Hazard { body }, Some(RecordBody::Hazard(hazard))) => {
            hazard_issues(hazard, body, &mut issues);
            consumable_occurrence_issues(&retrieved.consumable_occurrences, &mut issues);
        }
        (RecordSurfacePresentationView::Spell { .. }, Some(RecordBody::Spell(spell))) => {
            spell_issues(spell, selected_spell_issues, &mut issues);
        }
        (
            RecordSurfacePresentationView::Consumable { .. },
            Some(RecordBody::Consumable(consumable)),
        ) => consumable_issues(
            &consumable.definition,
            &consumable.source_state,
            &consumable.unsupported_content,
            None,
            &mut issues,
        ),
        (RecordSurfacePresentationView::Unavailable { .. }, _) => {
            issues.push(RecordSurfaceIssueView {
                consequence: None,
                fact_id: None,
                code: RecordSurfaceIssueCodeView::Unavailable,
                placement: RecordSurfaceIssuePlacementView::Record,
                subject: None,
                fact_label: None,
                message: "This record does not have an available typed presentation.".to_string(),
            })
        }
        _ => {}
    }
    (!issues.is_empty()).then_some(issues)
}

fn consumable_occurrence_issues(
    set: &ConsumableOccurrenceSet,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    for occurrence in &set.occurrences {
        let definition = set
            .entities
            .iter()
            .find(|entity| entity.id == occurrence.entity_id)
            .and_then(|entity| match &entity.target {
                atlas_record::ConsumableEntityTarget::Resolved { .. } => None,
                atlas_record::ConsumableEntityTarget::ParentOwned { definition, .. } => {
                    Some(definition.as_ref())
                }
            });
        if let Some(definition) = definition {
            consumable_issues(
                definition,
                &occurrence.state,
                &occurrence.unsupported_content,
                Some((occurrence.id.as_str(), occurrence.contextual_name.as_str())),
                issues,
            );
        } else {
            consumable_state_issues(
                &occurrence.state,
                Some((occurrence.id.as_str(), occurrence.contextual_name.as_str())),
                issues,
            );
            push_consumable_content_issues(
                &occurrence.unsupported_content,
                Some((occurrence.id.as_str(), occurrence.contextual_name.as_str())),
                issues,
            );
        }
        if let atlas_record::ConsumableSpellReuse::Mismatch { reason, .. } = &occurrence.spell_reuse
        {
            issues.push(RecordSurfaceIssueView {
                consequence: Some(
                    "The authored embedded spell is retained as source evidence but is not used as a spell presentation."
                        .to_string(),
                ),
                fact_id: Some(format!(
                    "consumable:{}:spell_child",
                    occurrence.id.as_str()
                )),
                code: RecordSurfaceIssueCodeView::Unavailable,
                placement: RecordSurfaceIssuePlacementView::Record,
                subject: Some(RecordSurfaceIssueSubjectView {
                    label: occurrence.contextual_name.clone(),
                    target: None,
                }),
                fact_label: Some("Embedded spell".to_string()),
                message: consumable_spell_mismatch_message(*reason).to_string(),
            });
        }
    }
}

fn consumable_spell_mismatch_message(
    reason: atlas_record::ConsumableSpellMismatchReason,
) -> &'static str {
    match reason {
        atlas_record::ConsumableSpellMismatchReason::UnresolvedParent => {
            "The embedded spell cannot be compared because the consumable target is unresolved."
        }
        atlas_record::ConsumableSpellMismatchReason::TargetWithoutChild => {
            "The target consumable has no matching embedded spell."
        }
        atlas_record::ConsumableSpellMismatchReason::LocalChildMissing => {
            "The local consumable has no embedded spell that matches its target."
        }
        atlas_record::ConsumableSpellMismatchReason::LocalChildMalformed => {
            "The local embedded spell has an unsupported source shape."
        }
        atlas_record::ConsumableSpellMismatchReason::ChildIdentity => {
            "The local and target embedded spell identities differ."
        }
        atlas_record::ConsumableSpellMismatchReason::SourceContext => {
            "The local and target embedded spell source contexts differ."
        }
        atlas_record::ConsumableSpellMismatchReason::Definition => {
            "The local and target embedded spell definitions differ."
        }
        atlas_record::ConsumableSpellMismatchReason::ContentOrReferences => {
            "The local and target embedded spell content or references differ."
        }
        atlas_record::ConsumableSpellMismatchReason::OverlayOrFormOrder => {
            "The local and target embedded spell forms or authored order differ."
        }
    }
}

fn consumable_issues(
    definition: &ConsumableDefinition,
    state: &ConsumableSourceState,
    unsupported_content: &[UnsupportedSourceValue],
    occurrence: Option<(&str, &str)>,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    macro_rules! fact {
        ($field:ident, $label:literal) => {
            push_consumable_fact_issue(
                &definition.$field,
                concat!("definition.", stringify!($field)),
                $label,
                occurrence,
                issues,
            );
        };
    }
    fact!(slug, "Slug");
    fact!(level, "Level");
    fact!(category, "Category");
    fact!(rarity, "Rarity");
    fact!(traits, "Traits");
    fact!(other_tags, "Other tags");
    fact!(base_item, "Base item");
    fact!(bulk, "Bulk");
    fact!(size, "Size");
    fact!(stack_group, "Stack group");
    fact!(material, "Material");
    fact!(price, "Price");
    fact!(usage, "Usage");
    fact!(maximum_uses, "Maximum uses");
    fact!(auto_destroy, "Auto-destroy");
    fact!(maximum_hp, "Maximum HP");
    fact!(hardness, "Hardness");
    fact!(damage, "Damage");
    fact!(publication, "Publication");
    fact!(rules, "Rules");
    if let FactValue::Value(ConsumableSourceValue::Known(material)) = &definition.material {
        push_consumable_fact_issue(
            &material.grade,
            "definition.material.grade",
            "Material grade",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &material.material_type,
            "definition.material.type",
            "Material type",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &material.effects,
            "definition.material.effects",
            "Material effects",
            occurrence,
            issues,
        );
    }
    if let FactValue::Value(ConsumableSourceValue::Known(price)) = &definition.price {
        push_consumable_fact_issue(
            &price.denominations,
            "definition.price.denominations",
            "Price denominations",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &price.per,
            "definition.price.per",
            "Price quantity",
            occurrence,
            issues,
        );
    }
    if let FactValue::Value(ConsumableSourceValue::Known(publication)) = &definition.publication {
        push_consumable_fact_issue(
            &publication.title,
            "definition.publication.title",
            "Publication title",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &publication.license,
            "definition.publication.license",
            "Publication license",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &publication.remaster,
            "definition.publication.remaster",
            "Remaster status",
            occurrence,
            issues,
        );
    }
    if let FactValue::Value(ConsumableSourceValue::Known(damage)) = &definition.damage {
        push_consumable_fact_issue(
            &damage.formula,
            "definition.damage.formula",
            "Damage formula",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &damage.category,
            "definition.damage.kind",
            "Damage kind",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &damage.damage_type,
            "definition.damage.type",
            "Damage type",
            occurrence,
            issues,
        );
    }
    consumable_state_issues(state, occurrence, issues);
    push_consumable_content_issues(unsupported_content, occurrence, issues);
}

fn push_consumable_content_issues(
    unsupported_content: &[UnsupportedSourceValue],
    occurrence: Option<(&str, &str)>,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    for (ordinal, _) in unsupported_content.iter().enumerate() {
        let (fact_id, subject) = occurrence.map_or_else(
            || (format!("consumable:content.unsupported.{ordinal}"), None),
            |(occurrence_id, occurrence_label)| {
                (
                    format!("consumable:{occurrence_id}:content.unsupported.{ordinal}"),
                    Some(RecordSurfaceIssueSubjectView {
                        label: occurrence_label.to_string(),
                        target: None,
                    }),
                )
            },
        );
        issues.push(RecordSurfaceIssueView {
            fact_id: Some(fact_id),
            code: RecordSurfaceIssueCodeView::Unsupported,
            placement: RecordSurfaceIssuePlacementView::Record,
            subject,
            fact_label: Some("Description".to_string()),
            message: "Description could not be presented from the authored source value."
                .to_string(),
            consequence: Some("The affected consumable description is unavailable.".to_string()),
        });
    }
}

fn consumable_state_issues(
    state: &ConsumableSourceState,
    occurrence: Option<(&str, &str)>,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    push_consumable_fact_issue(
        &state.quantity,
        "state.quantity",
        "Quantity",
        occurrence,
        issues,
    );
    push_consumable_fact_issue(
        &state.current_uses,
        "state.current_uses",
        "Uses remaining",
        occurrence,
        issues,
    );
    push_consumable_fact_issue(
        &state.current_hp,
        "state.current_hp",
        "Current HP",
        occurrence,
        issues,
    );
    push_consumable_fact_issue(
        &state.container_id,
        "state.container_id",
        "Container",
        occurrence,
        issues,
    );
    push_consumable_fact_issue(
        &state.equipped,
        "state.equipped",
        "Equipped state",
        occurrence,
        issues,
    );
    if let FactValue::Value(ConsumableSourceValue::Known(equipped)) = &state.equipped {
        push_consumable_fact_issue(
            &equipped.carry_type,
            "state.equipped.carry_type",
            "Carry type",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &equipped.hands_held,
            "state.equipped.hands_held",
            "Hands held",
            occurrence,
            issues,
        );
        push_consumable_fact_issue(
            &equipped.in_slot,
            "state.equipped.in_slot",
            "In slot",
            occurrence,
            issues,
        );
    }
}

fn push_consumable_fact_issue<T>(
    fact: &ConsumableFact<T>,
    fact_path: &str,
    label: &str,
    occurrence: Option<(&str, &str)>,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    if !matches!(
        fact,
        FactValue::Value(ConsumableSourceValue::Unsupported(_))
    ) {
        return;
    }
    let (fact_id, subject) = occurrence.map_or_else(
        || (format!("consumable:{fact_path}"), None),
        |(occurrence_id, occurrence_label)| {
            (
                format!("consumable:{occurrence_id}:{fact_path}"),
                Some(RecordSurfaceIssueSubjectView {
                    label: occurrence_label.to_string(),
                    target: None,
                }),
            )
        },
    );
    issues.push(RecordSurfaceIssueView {
        fact_id: Some(fact_id),
        code: RecordSurfaceIssueCodeView::Unsupported,
        placement: RecordSurfaceIssuePlacementView::Record,
        subject,
        fact_label: Some(label.to_string()),
        message: format!("{label} could not be presented from the authored source value."),
        consequence: Some("The affected consumable fact is unavailable.".to_string()),
    });
}

fn creature_issues(
    unavailable: Option<&CreatureSurfaceUnavailableDomainsView>,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    let Some(unavailable) = unavailable else {
        return;
    };
    let mut seen = BTreeSet::new();
    for (placement, domain) in [
        (
            RecordSurfaceIssuePlacementView::Classification,
            unavailable.classification.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Classification,
            unavailable.initiative.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Defenses,
            unavailable.vitals.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Defenses,
            unavailable.defenses.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Defenses,
            unavailable.saves.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.awareness.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.abilities.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.skills.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.movement.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.resources.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.equipment.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.lore.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Casting,
            unavailable.spellcasting.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Activity,
            unavailable.activities.as_ref(),
        ),
        (
            RecordSurfaceIssuePlacementView::Record,
            unavailable.relationships.as_ref(),
        ),
    ] {
        extend_creature_domain(issues, &mut seen, placement, domain);
    }
}

fn extend_creature_domain(
    issues: &mut Vec<RecordSurfaceIssueView>,
    seen: &mut BTreeSet<(
        CreatureSurfaceUnavailableStateView,
        CreatureSurfaceUnavailableFieldView,
        Option<String>,
    )>,
    placement: RecordSurfaceIssuePlacementView,
    domain: Option<&CreatureSurfaceDomainUnavailableView>,
) {
    let Some(domain) = domain else {
        return;
    };
    for cause in &domain.causes {
        if !seen.insert((cause.state, cause.field, cause.component_id.clone())) {
            continue;
        }
        let (role, state) = if cause.field == CreatureSurfaceUnavailableFieldView::UnmodeledSkill {
            (
                FactPresentationRole::Unmodeled,
                FactPresentationState::Known,
            )
        } else {
            (
                FactPresentationRole::Gameplay,
                match cause.state {
                    CreatureSurfaceUnavailableStateView::Missing => FactPresentationState::Missing,
                    CreatureSurfaceUnavailableStateView::Null => FactPresentationState::Null,
                    CreatureSurfaceUnavailableStateView::Unsupported => {
                        FactPresentationState::Unsupported
                    }
                },
            )
        };
        push_policy_issue(
            issues,
            role,
            state,
            FactRequirement::Required,
            placement,
            &cause.message,
        );
    }
}

fn hazard_issues(
    hazard: &atlas_record::HazardRecord,
    body: &HazardSurfaceView,
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    let metadata = project_hazard_source_metadata(hazard);
    let mut metadata_issue_keys = BTreeSet::new();
    for issue in metadata.issues {
        metadata_issue_keys.insert((issue.field_key(), issue.component_id().map(str::to_string)));
        issues.push(RecordSurfaceIssueView {
            consequence: None,
            fact_id: None,
            code: match issue.kind {
                HazardSourceMetadataIssueKind::Malformed => RecordSurfaceIssueCodeView::Malformed,
                HazardSourceMetadataIssueKind::HasHealthConflict
                | HazardSourceMetadataIssueKind::WeaponTypeConflict => {
                    RecordSurfaceIssueCodeView::SourceConsistencyConflict
                }
                HazardSourceMetadataIssueKind::NonZeroTemporaryMaximum
                | HazardSourceMetadataIssueKind::NonEmptySaveDetail
                | HazardSourceMetadataIssueKind::NonEmptyAttackEffectsCustom => {
                    RecordSurfaceIssueCodeView::Unmodeled
                }
            },
            placement: match issue.field {
                HazardSourceMetadataField::HasHealth
                | HazardSourceMetadataField::TemporaryMaximum
                | HazardSourceMetadataField::SaveDetail(_) => {
                    RecordSurfaceIssuePlacementView::Defenses
                }
                HazardSourceMetadataField::ItemRarity
                | HazardSourceMetadataField::ItemLineage
                | HazardSourceMetadataField::StrikeAttack
                | HazardSourceMetadataField::StrikeWeaponType
                | HazardSourceMetadataField::StrikeAttackEffectsCustom => {
                    RecordSurfaceIssuePlacementView::Activity
                }
                HazardSourceMetadataField::TokenName => RecordSurfaceIssuePlacementView::Record,
            },
            subject: issue
                .entity_id
                .as_ref()
                .and_then(|entity_id| hazard_issue_subject(hazard, entity_id.as_str())),
            fact_label: Some(hazard_metadata_fact_label(issue.field).to_string()),
            message: hazard_metadata_issue_message(hazard, &issue),
        });
    }

    for unavailable in body.unavailable_fields.iter().flatten() {
        if metadata_issue_keys
            .contains(&(unavailable.field.as_str(), unavailable.component_id.clone()))
        {
            continue;
        }
        let state = match unavailable.state {
            HazardSurfaceUnavailableStateView::Missing => FactPresentationState::Missing,
            HazardSurfaceUnavailableStateView::Null => FactPresentationState::Null,
            HazardSurfaceUnavailableStateView::Unsupported => FactPresentationState::Unsupported,
        };
        let FactPresentationDisposition::Issue(kind) = classify_fact_presentation(
            FactPresentationRole::Gameplay,
            state,
            FactRequirement::Optional,
        ) else {
            continue;
        };
        issues.push(RecordSurfaceIssueView {
            fact_id: unavailable.fact_id.clone(),
            code: issue_code(kind),
            placement: if unavailable.component_id.is_some() {
                RecordSurfaceIssuePlacementView::Activity
            } else {
                RecordSurfaceIssuePlacementView::Record
            },
            subject: unavailable
                .component_id
                .as_deref()
                .and_then(|component_id| hazard_issue_subject(hazard, component_id)),
            fact_label: hazard_unavailable_fact_label(&unavailable.field).map(str::to_string),
            message: unavailable.message.clone(),
            consequence: unavailable.consequence.clone(),
        });
    }
}

fn spell_issues(
    spell: &atlas_record::SpellRecord,
    selected: &[SpellPresentationIssue],
    issues: &mut Vec<RecordSurfaceIssueView>,
) {
    let projected = project_spell_presentation_issues(spell);
    issues.extend(
        merge_spell_presentation_issues(projected, selected.iter().copied())
            .into_iter()
            .map(|issue| RecordSurfaceIssueView {
                consequence: None,
                fact_id: None,
                code: issue_code(issue.kind),
                placement: spell_issue_placement(issue.placement()),
                subject: None,
                fact_label: Some(issue.field.label().to_string()),
                message: issue.message(),
            }),
    );
}

fn spell_issue_placement(
    placement: SpellPresentationIssuePlacement,
) -> RecordSurfaceIssuePlacementView {
    match placement {
        SpellPresentationIssuePlacement::Record => RecordSurfaceIssuePlacementView::Record,
        SpellPresentationIssuePlacement::Classification => {
            RecordSurfaceIssuePlacementView::Classification
        }
        SpellPresentationIssuePlacement::Casting => RecordSurfaceIssuePlacementView::Casting,
        SpellPresentationIssuePlacement::Targeting => RecordSurfaceIssuePlacementView::Targeting,
        SpellPresentationIssuePlacement::Defenses => RecordSurfaceIssuePlacementView::Defenses,
        SpellPresentationIssuePlacement::Damage => RecordSurfaceIssuePlacementView::Damage,
        SpellPresentationIssuePlacement::Duration => RecordSurfaceIssuePlacementView::Duration,
        SpellPresentationIssuePlacement::Heightening => {
            RecordSurfaceIssuePlacementView::Heightening
        }
        SpellPresentationIssuePlacement::Ritual => RecordSurfaceIssuePlacementView::Ritual,
        SpellPresentationIssuePlacement::Rules => RecordSurfaceIssuePlacementView::Rules,
        SpellPresentationIssuePlacement::Forms => RecordSurfaceIssuePlacementView::Forms,
    }
}

fn push_policy_issue(
    issues: &mut Vec<RecordSurfaceIssueView>,
    role: FactPresentationRole,
    state: FactPresentationState,
    requirement: FactRequirement,
    placement: RecordSurfaceIssuePlacementView,
    message: &str,
) {
    let FactPresentationDisposition::Issue(kind) =
        classify_fact_presentation(role, state, requirement)
    else {
        return;
    };
    issues.push(RecordSurfaceIssueView {
        consequence: None,
        fact_id: None,
        code: issue_code(kind),
        placement,
        subject: None,
        fact_label: None,
        message: message.to_string(),
    });
}

fn hazard_issue_subject(
    hazard: &atlas_record::HazardRecord,
    component_id: &str,
) -> Option<RecordSurfaceIssueSubjectView> {
    let embedded = hazard.embedded_entities.typed()?;
    let occurrence = embedded.occurrences.iter().find(|occurrence| {
        occurrence.id.as_str() == component_id || occurrence.entity_id.as_str() == component_id
    })?;
    let entity = embedded
        .entities
        .iter()
        .find(|entity| entity.id == occurrence.entity_id)?;
    let label = occurrence
        .contextual_label
        .typed()
        .cloned()
        .unwrap_or_else(|| entity.label.clone());
    Some(RecordSurfaceIssueSubjectView {
        label,
        target: Some(RecordSurfaceIssueTargetView::Activity {
            occurrence_id: occurrence.id.as_str().to_string(),
        }),
    })
}

fn hazard_metadata_issue_message(
    hazard: &atlas_record::HazardRecord,
    issue: &atlas_record::HazardSourceMetadataIssue,
) -> String {
    if issue.kind != HazardSourceMetadataIssueKind::Malformed {
        return issue.message().to_string();
    }
    if issue.field == HazardSourceMetadataField::StrikeAttack {
        let value = hazard
            .embedded_entities
            .typed()
            .and_then(|embedded| {
                embedded
                    .entities
                    .iter()
                    .find(|entity| Some(&entity.id) == issue.entity_id.as_ref())
            })
            .and_then(|entity| match &entity.capability {
                atlas_record::HazardCapability::Strike(strike) => {
                    match &strike.source_metadata.attack.value {
                        atlas_record::FactValue::Value(
                            atlas_record::HazardSourceValue::Unsupported(value),
                        ) => Some(value),
                        _ => None,
                    }
                }
                _ => None,
            });
        let shape = value
            .map(|value| hazard_source_shape_label(value.actual_shape))
            .unwrap_or("an unsupported value");
        return format!(
            "The extra source attack field contains {shape} where an integer is expected. It does not supply the displayed attack bonus; that bonus comes from the separate gameplay attack field. The original value is retained in Source & provenance."
        );
    }
    format!(
        "{} has an unsupported source value. It is retained in Source & provenance and is not used as a gameplay fallback.",
        hazard_metadata_fact_label(issue.field)
    )
}

pub(crate) fn hazard_source_shape_label(shape: atlas_record::HazardSourceShape) -> &'static str {
    use atlas_record::HazardSourceShape;
    match shape {
        HazardSourceShape::Missing => "a missing value",
        HazardSourceShape::Null => "an explicit null",
        HazardSourceShape::Boolean => "a yes/no value",
        HazardSourceShape::Number => "a number",
        HazardSourceShape::String => "text",
        HazardSourceShape::Array => "a list",
        HazardSourceShape::Object => "a structured value",
    }
}

fn hazard_metadata_fact_label(field: HazardSourceMetadataField) -> &'static str {
    match field {
        HazardSourceMetadataField::TokenName => "Token name",
        HazardSourceMetadataField::HasHealth => "Health compatibility",
        HazardSourceMetadataField::TemporaryMaximum => "Temporary maximum HP",
        HazardSourceMetadataField::SaveDetail(atlas_record::HazardSaveKind::Fortitude) => {
            "Fortitude source note"
        }
        HazardSourceMetadataField::SaveDetail(atlas_record::HazardSaveKind::Reflex) => {
            "Reflex source note"
        }
        HazardSourceMetadataField::SaveDetail(atlas_record::HazardSaveKind::Will) => {
            "Will source note"
        }
        HazardSourceMetadataField::ItemRarity => "Component rarity",
        HazardSourceMetadataField::ItemLineage => "Component lineage",
        HazardSourceMetadataField::StrikeAttack => "Strike source attack",
        HazardSourceMetadataField::StrikeWeaponType => "Strike source mode",
        HazardSourceMetadataField::StrikeAttackEffectsCustom => "Custom attack effect",
    }
}

fn hazard_unavailable_fact_label(field: &str) -> Option<&'static str> {
    match field {
        "activity.attack_effects" => Some("Attack effect"),
        "activity.action.unexpected" => Some("Authored action field"),
        "activity.strike.unexpected" => Some("Authored strike field"),
        "activity.condition.unexpected" => Some("Authored condition field"),
        "activity.effect.unexpected" => Some("Authored effect field"),
        "activity.unsupported_child.field" => Some("Content-only component field"),
        "activity.child_type" => Some("Content-only component type"),
        "activity.rules" => Some("Activity rule"),
        _ => None,
    }
}

fn issue_code(kind: FactIssueKind) -> RecordSurfaceIssueCodeView {
    match kind {
        FactIssueKind::RequiredMissing => RecordSurfaceIssueCodeView::RequiredMissing,
        FactIssueKind::RequiredNull => RecordSurfaceIssueCodeView::RequiredNull,
        FactIssueKind::RequiredEmpty => RecordSurfaceIssueCodeView::RequiredEmpty,
        FactIssueKind::Unsupported => RecordSurfaceIssueCodeView::Unsupported,
        FactIssueKind::Malformed => RecordSurfaceIssueCodeView::Malformed,
        FactIssueKind::Ambiguous => RecordSurfaceIssueCodeView::Ambiguous,
        FactIssueKind::Unavailable => RecordSurfaceIssueCodeView::Unavailable,
        FactIssueKind::Unmodeled => RecordSurfaceIssueCodeView::Unmodeled,
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        RecordSurfaceIssueCodeView, RecordSurfaceIssuePlacementView, RecordSurfacePresentationView,
        RecordSurfaceProfileView, SpellFormResultView, SpellResolvedFieldView,
    };
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::{
        FactValue, RecordBody, SpellClassification, SpellFixedHeighteningLayer, SpellFormId,
        SpellHeightening, SpellIdentity, SpellKeyedPatch, SpellKeyedPatchMember,
        SpellKeyedPatchOperation, SpellOverlay, SpellOverlayId, SpellOverlayType, SpellPatch,
        SpellProvenance, SpellRecord, SpellRollOptionRule, SpellRule, SpellRuleElement,
        SpellRulePredicate, SpellRuleSuboption, SpellSourceId, SpellSourceValue,
        SpellUnsupportedRule, SpellUnsupportedRulePredicate, SpellUnsupportedSourceFact,
        SpellUnsupportedSourceField, UnsupportedSourceReason, UnsupportedSourceShape,
        UnsupportedSourceValue,
    };
    use atlas_search::RemasterLinksResult;

    use crate::retrieval::VerifiedRemasterLookup;
    use crate::test_support::encounter_fixture_worker;

    #[test]
    fn unsupported_consumable_descriptions_report_each_authored_failure_once() {
        let unsupported = UnsupportedSourceValue {
            shape: UnsupportedSourceShape::Object,
            value: r#"{"value":"first","value":"second"}"#.to_string(),
            reason: UnsupportedSourceReason::SourceFieldDrift,
        };
        let mut issues = Vec::new();
        super::push_consumable_content_issues(
            &[unsupported],
            Some(("occurrence-1", "Malformed Description")),
            &mut issues,
        );

        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].fact_id.as_deref(),
            Some("consumable:occurrence-1:content.unsupported.0")
        );
        assert_eq!(issues[0].fact_label.as_deref(), Some("Description"));
        assert_eq!(
            issues[0]
                .subject
                .as_ref()
                .map(|subject| subject.label.as_str()),
            Some("Malformed Description")
        );
        assert!(!issues[0].message.contains("first"));
    }

    #[test]
    fn spell_expectedness_keeps_optional_absence_quiet_and_unsupported_actionable() {
        for classification in [FactValue::Missing, FactValue::Null] {
            let record = spell_fixture(classification);
            let surface = surface(&record, None);
            assert_spell_metadata_has_no_classification(&surface);
            assert!(surface.issues.iter().flatten().all(|issue| {
                issue.placement != RecordSurfaceIssuePlacementView::Classification
            }));
        }

        let known_empty = SpellClassification {
            rank: known(2),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        };
        let record = spell_fixture(FactValue::Value(SpellSourceValue::Known(known_empty)));
        let known_empty_surface = surface(&record, None);
        assert_spell_metadata_has_no_classification(&known_empty_surface);
        assert!(known_empty_surface.issues.is_none());

        let record = spell_fixture(FactValue::Value(SpellSourceValue::Unsupported(
            unsupported_value(),
        )));
        let surface = surface(&record, None);
        assert_spell_metadata_has_no_classification(&surface);
        let issues = surface
            .issues
            .as_ref()
            .expect("unsupported base fact issue");
        assert_eq!(
            issues
                .iter()
                .filter(|issue| {
                    issue.code == RecordSurfaceIssueCodeView::Unsupported
                        && issue.placement == RecordSurfaceIssuePlacementView::Classification
                })
                .count(),
            1
        );
        assert!(issues.iter().all(|issue| {
            !issue.message.contains("/system/")
                && !issue.message.contains("spell-form:")
                && !issue.message.contains("unsupported-fixture")
        }));
        let RecordSurfacePresentationView::Spell { body } = &surface.presentation else {
            panic!("spell surface");
        };
        let SpellFormResultView::Available { definition } = &body.effective_form.result else {
            panic!("base Unsupported remains a localized field failure");
        };
        assert!(matches!(
            definition.classification,
            SpellResolvedFieldView::Unavailable { .. }
        ));
    }

    #[test]
    fn populated_spell_rule_and_resolved_unavailability_each_project_once() {
        let classification = SpellClassification {
            rank: known(2),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        };
        let mut record = spell_fixture(FactValue::Value(SpellSourceValue::Known(classification)));
        let Some(RecordBody::Spell(spell)) = record.body.as_mut() else {
            panic!("spell fixture body");
        };
        spell.definition.rules = known(vec![SpellRuleElement {
            authored_order: 0,
            source_path: "/system/rules/0".to_string(),
            authored_key: "FutureRule".to_string(),
            authored_object_json: "{}".to_string(),
            rule: SpellRule::Unsupported(SpellUnsupportedRule {
                authored_key: "FutureRule".to_string(),
                source_path: "/system/rules/0".to_string(),
                value: unsupported_value(),
            }),
        }]);
        let base_form = SpellFormId::base(&record.record.identity.key);
        let issues = surface(&record, Some((base_form, 1)))
            .issues
            .expect("rule and selection issues");
        assert_eq!(
            issues
                .iter()
                .filter(|issue| issue.placement == RecordSurfaceIssuePlacementView::Rules)
                .count(),
            1
        );
        assert_eq!(
            issues
                .iter()
                .filter(|issue| {
                    issue.code == RecordSurfaceIssueCodeView::Unavailable
                        && issue.placement == RecordSurfaceIssuePlacementView::Forms
                })
                .count(),
            1
        );
        assert!(issues.iter().all(|issue| !issue.message.contains('/')));
    }

    #[test]
    fn nested_spell_rank_predicate_and_suboption_issues_are_localized_once() {
        let classification = SpellClassification {
            rank: unsupported_fact(),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        };
        let mut record = spell_fixture(FactValue::Value(SpellSourceValue::Known(classification)));
        let Some(RecordBody::Spell(spell)) = record.body.as_mut() else {
            panic!("spell fixture body");
        };
        spell.definition.rules = known(vec![SpellRuleElement {
            authored_order: 0,
            source_path: "/system/rules/0".to_string(),
            authored_key: "RollOption".to_string(),
            authored_object_json: "{}".to_string(),
            rule: SpellRule::RollOption(SpellRollOptionRule {
                domain: known("all".to_string()),
                label: FactValue::Missing,
                option: known("fixture-option".to_string()),
                placement: known("spellcasting".to_string()),
                predicate: known(vec![SpellRulePredicate::Unsupported(
                    SpellUnsupportedRulePredicate {
                        source_path: "/system/rules/0/predicate/0".to_string(),
                        authored_key: None,
                        authored_order: 0,
                        value: unsupported_value(),
                    },
                )]),
                suboptions: known(vec![SpellRuleSuboption {
                    label: unsupported_fact(),
                    value: known("fire".to_string()),
                }]),
                toggleable: known(false),
            }),
        }]);

        let base_form = SpellFormId::base(&record.record.identity.key);
        let issues = surface(&record, Some((base_form, 1)))
            .issues
            .expect("nested unsupported spell facts should remain actionable");
        for (placement, message) in [
            (RecordSurfaceIssuePlacementView::Classification, "Rank"),
            (
                RecordSurfaceIssuePlacementView::Rules,
                "Roll option predicate",
            ),
            (
                RecordSurfaceIssuePlacementView::Rules,
                "Roll option suboption label",
            ),
        ] {
            assert_eq!(
                issues
                    .iter()
                    .filter(|issue| {
                        issue.code == RecordSurfaceIssueCodeView::Unsupported
                            && issue.placement == placement
                            && issue.message.contains(message)
                    })
                    .count(),
                1,
                "{message} should have one naturally placed issue"
            );
        }
        assert!(issues.iter().all(|issue| {
            !issue.message.contains("/system/")
                && !issue.message.contains("fixture-option")
                && !issue.message.contains("issue-source")
                && issue.message != "A resolved spell field is unavailable."
        }));
    }

    #[test]
    fn selected_overlay_uses_actual_resolved_field_issues_and_shared_source_notes() {
        let classification = SpellClassification {
            rank: known(2),
            traits: known(Vec::new()),
            traditions: known(Vec::new()),
        };
        let mut record = spell_fixture(known(classification));
        let Some(RecordBody::Spell(spell)) = record.body.as_mut() else {
            panic!("spell fixture body");
        };
        spell
            .definition
            .unsupported_notes
            .push(SpellUnsupportedSourceFact {
                field: SpellUnsupportedSourceField::RitualMember,
                source_path: "/system/ritual/private".to_string(),
                authored_key: "private".to_string(),
                authored_order: Some(0),
                value: unsupported_value(),
            });
        spell.definition.heightening =
            known(SpellHeightening::Fixed(vec![SpellFixedHeighteningLayer {
                key: "5".to_string(),
                authored_order: 0,
                rank: SpellSourceValue::Known(5),
                patch: SpellPatch {
                    damage: known(SpellKeyedPatch {
                        members: vec![
                            SpellKeyedPatchMember {
                                key: "duplicate".to_string(),
                                authored_order: 0,
                                operation: SpellKeyedPatchOperation::Delete,
                            },
                            SpellKeyedPatchMember {
                                key: "duplicate".to_string(),
                                authored_order: 1,
                                operation: SpellKeyedPatchOperation::Delete,
                            },
                        ],
                    }),
                    ..SpellPatch::default()
                },
            }]));
        let overlay_id = SpellOverlayId::new("alternate").expect("overlay id");
        let overlay = SpellOverlay {
            key: overlay_id.as_str().to_string(),
            authored_order: 0,
            overlay_id: overlay_id.clone(),
            source_id: known(overlay_id.clone()),
            sort: known(0),
            name: known("Alternate".to_string()),
            overlay_type: known(SpellOverlayType::Override),
            patch: SpellPatch {
                classification: unsupported_fact(),
                ..SpellPatch::default()
            },
        };
        let form_id = overlay.form_id(&record.record.identity.key);
        let selected_form_id = form_id.as_str().to_string();
        spell.definition.overlays = known(vec![overlay]);

        let surface = surface(&record, Some((form_id, 5)));
        assert_spell_metadata_has_no_classification(&surface);
        let issues = surface
            .issues
            .as_ref()
            .expect("selected field and source note issues");
        for (code, placement, label) in [
            (
                RecordSurfaceIssueCodeView::Unavailable,
                RecordSurfaceIssuePlacementView::Damage,
                "Damage",
            ),
            (
                RecordSurfaceIssueCodeView::Unmodeled,
                RecordSurfaceIssuePlacementView::Ritual,
                "Ritual source fact",
            ),
        ] {
            assert_eq!(
                issues
                    .iter()
                    .filter(|issue| {
                        issue.code == code
                            && issue.placement == placement
                            && issue.message.contains(label)
                    })
                    .count(),
                1,
                "{label} should be reported exactly once"
            );
        }
        assert_eq!(
            issues
                .iter()
                .filter(|issue| {
                    issue.placement == RecordSurfaceIssuePlacementView::Classification
                })
                .count(),
            1,
            "selected classification failure should have one localized issue"
        );
        assert!(issues.iter().all(|issue| {
            !issue.message.contains("/system/")
                && !issue.message.contains("private")
                && !issue.message.contains("alternate")
        }));

        let RecordSurfacePresentationView::Spell { body } = &surface.presentation else {
            panic!("spell surface");
        };
        let SpellFormResultView::Available { definition } = &body.effective_form.result else {
            panic!("overall selected form remains available");
        };
        assert_eq!(body.effective_form.id, selected_form_id);
        assert!(matches!(
            definition.classification,
            SpellResolvedFieldView::Unavailable { .. }
        ));
        assert!(matches!(
            definition.casting,
            SpellResolvedFieldView::Available { .. }
        ));
        assert!(matches!(
            definition.damage,
            SpellResolvedFieldView::Unavailable { .. }
        ));
    }

    fn surface(
        record: &atlas_record::RetrievedRecord,
        selection: Option<(SpellFormId, u8)>,
    ) -> atlas_app_model::RecordSurfaceView {
        let remaster_lookup = VerifiedRemasterLookup::from_test_result(RemasterLinksResult {
            seed: record.clone(),
            links: Vec::new(),
        });
        crate::surface::record_surface(
            record,
            RecordSurfaceProfileView::RecordDetail,
            None,
            selection,
            &remaster_lookup,
        )
    }

    fn spell_fixture(
        classification: atlas_record::SpellFact<SpellClassification>,
    ) -> atlas_record::RetrievedRecord {
        let fixture = encounter_fixture_worker();
        let key = RecordKey::parse("spells:expectednessFixture").expect("spell key");
        let mut record = fixture
            .worker
            .get_records(vec![
                RecordKey::parse("actions:testAction1").expect("fixture key"),
            ])
            .expect("fixture record should load")
            .pop()
            .expect("fixture record should exist");
        record.record.identity.key = key.clone();
        record.record.identity.name = "Expectedness Fixture".to_string();
        record.record.classification.kind = RecordKind::Spell;
        record.record.classification.level = Some(99);
        record.record.classification.traits = vec!["stale-query-trait".to_string()];
        let mut spell = SpellRecord::new(
            SpellIdentity {
                record_key: key,
                source_id: SpellSourceId::new("expectedness-source").expect("source id"),
                name: "Expectedness Fixture".to_string(),
            },
            SpellProvenance {
                source_path: "fixture.json".to_string(),
                source_contract_version: "v1".to_string(),
                source_system_version: "7".to_string(),
                source_upstream_commit: "fixture".to_string(),
                standalone_location: FactValue::Missing,
            },
        );
        spell.definition.classification = classification;
        record.body = Some(RecordBody::Spell(spell));
        record
    }

    fn assert_spell_metadata_has_no_classification(surface: &atlas_app_model::RecordSurfaceView) {
        assert_eq!(surface.metadata.level, None);
        assert!(surface.metadata.traits.is_empty());
    }

    fn known<T>(value: T) -> atlas_record::SpellFact<T> {
        FactValue::Value(SpellSourceValue::Known(value))
    }

    fn unsupported_fact<T>() -> atlas_record::SpellFact<T> {
        FactValue::Value(SpellSourceValue::Unsupported(unsupported_value()))
    }

    fn unsupported_value() -> UnsupportedSourceValue {
        UnsupportedSourceValue {
            shape: UnsupportedSourceShape::Object,
            value: "{}".to_string(),
            reason: UnsupportedSourceReason::SourceFieldDrift,
        }
    }
}
