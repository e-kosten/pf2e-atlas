use std::collections::BTreeMap;

use atlas_app_model::{
    ConsumableDamageView, ConsumableDefinitionView, ConsumableEquippedView, ConsumableFactView,
    ConsumableMaterialView, ConsumableOccurrenceIdentityStabilityView,
    ConsumableOccurrenceTargetView, ConsumableOccurrenceView, ConsumablePriceDenominationView,
    ConsumablePriceView, ConsumablePublicationView, ConsumableSourceStateView,
    ConsumableSpellChildLinkView, ConsumableSurfaceView, ConsumableTargetReasonView,
};
use atlas_record::{
    ConsumableDefinition, ConsumableEntityTarget, ConsumableFact,
    ConsumableOccurrenceIdentityStability, ConsumableOccurrenceSet, ConsumableSourceState,
    ConsumableSourceValue, ConsumableSpellReuse, ConsumableTargetResolution, FactValue,
    SpellStandaloneTarget,
};

pub(crate) fn standalone_consumable_surface(
    record: &atlas_record::ConsumableRecord,
    spell_children: &[atlas_record::ConsumableSpellChild],
) -> ConsumableSurfaceView {
    consumable_surface(
        &record.definition,
        &record.source_state,
        Some(&record.content),
        spell_children.first().map(spell_child_link),
    )
}

pub(crate) fn consumable_occurrence_views(
    set: &ConsumableOccurrenceSet,
) -> Option<Vec<ConsumableOccurrenceView>> {
    let entities = set
        .entities
        .iter()
        .map(|entity| (entity.id.as_str(), &entity.target))
        .collect::<BTreeMap<_, _>>();
    let values = set
        .occurrences
        .iter()
        .filter_map(|occurrence| {
            let target = *entities.get(occurrence.entity_id.as_str())?;
            let (target_view, definition) = match target {
                ConsumableEntityTarget::Resolved {
                    record_key,
                    immutable_mismatches,
                } => (
                    ConsumableOccurrenceTargetView::Resolved {
                        record_key: record_key.to_string(),
                        mismatch_fields: immutable_mismatches
                            .iter()
                            .map(|mismatch| mismatch.field_path.clone())
                            .collect(),
                    },
                    None,
                ),
                ConsumableEntityTarget::ParentOwned {
                    definition,
                    resolution,
                    ..
                } => (
                    ConsumableOccurrenceTargetView::ParentOwned {
                        reason: match resolution {
                            ConsumableTargetResolution::NoLocator => {
                                ConsumableTargetReasonView::NoLocator
                            }
                            ConsumableTargetResolution::MalformedOrDuplicateLocator(_) => {
                                ConsumableTargetReasonView::MalformedOrDuplicateLocator
                            }
                            ConsumableTargetResolution::TargetMissing(_) => {
                                ConsumableTargetReasonView::TargetMissing
                            }
                            ConsumableTargetResolution::WrongDocumentOrFamily(_) => {
                                ConsumableTargetReasonView::WrongDocumentOrFamily
                            }
                        },
                    },
                    Some(Box::new(consumable_definition_surface(definition))),
                ),
            };
            let target_record_key = match target {
                ConsumableEntityTarget::Resolved { record_key, .. } => Some(record_key),
                ConsumableEntityTarget::ParentOwned { .. } => None,
            };
            Some(ConsumableOccurrenceView {
                occurrence_id: occurrence.id.as_str().to_string(),
                authored_order: occurrence.authored_order,
                name: occurrence.contextual_name.clone(),
                identity_stability: match occurrence.identity_stability {
                    ConsumableOccurrenceIdentityStability::StableSourceIdentity => {
                        ConsumableOccurrenceIdentityStabilityView::StableSourceIdentity
                    }
                    ConsumableOccurrenceIdentityStability::UnstableOwnerOrdinal => {
                        ConsumableOccurrenceIdentityStabilityView::UnstableOwnerOrdinal
                    }
                },
                target: target_view,
                definition,
                source_state: source_state(&occurrence.state),
                spell_child: occurrence_spell_link(&occurrence.spell_reuse, target_record_key),
                content: occurrence
                    .authored_content
                    .documents
                    .iter()
                    .filter_map(crate::surface::content_view)
                    .collect(),
            })
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then_some(values)
}

fn consumable_surface(
    definition: &ConsumableDefinition,
    state: &ConsumableSourceState,
    content: Option<&atlas_record::OwnedRichContent>,
    spell_child: Option<ConsumableSpellChildLinkView>,
) -> ConsumableSurfaceView {
    let ConsumableDefinitionView {
        slug,
        level,
        category,
        rarity,
        traits,
        other_tags,
        usage,
        base_item,
        bulk,
        size,
        stack_group,
        material,
        price,
        maximum_uses,
        auto_destroy,
        maximum_hp,
        hardness,
        publication,
        damage,
    } = consumable_definition_surface(definition);
    ConsumableSurfaceView {
        slug,
        level,
        category,
        rarity,
        traits,
        other_tags,
        usage,
        base_item,
        bulk,
        size,
        stack_group,
        material,
        price,
        maximum_uses,
        auto_destroy,
        maximum_hp,
        hardness,
        publication,
        source_state: source_state(state),
        damage,
        spell_child,
        content: content
            .into_iter()
            .flat_map(|content| &content.documents)
            .filter_map(crate::surface::content_view)
            .collect(),
    }
}

fn consumable_definition_surface(definition: &ConsumableDefinition) -> ConsumableDefinitionView {
    ConsumableDefinitionView {
        slug: fact(&definition.slug, Clone::clone),
        level: integer_fact(&definition.level),
        category: fact(&definition.category, Clone::clone),
        rarity: fact(&definition.rarity, |value| value.as_str().to_string()),
        traits: fact(&definition.traits, Clone::clone),
        other_tags: fact(&definition.other_tags, Clone::clone),
        usage: fact(&definition.usage, Clone::clone),
        base_item: fact(&definition.base_item, Clone::clone),
        bulk: fact(&definition.bulk, |value| value.as_str().to_string()),
        size: fact(&definition.size, Clone::clone),
        stack_group: fact(&definition.stack_group, Clone::clone),
        material: fact(&definition.material, |value| ConsumableMaterialView {
            grade: fact(&value.grade, Clone::clone),
            material_type: fact(&value.material_type, Clone::clone),
            effects: fact(&value.effects, Clone::clone),
        }),
        price: fact(&definition.price, |value| ConsumablePriceView {
            denominations: fact_try(&value.denominations, |values| {
                values
                    .iter()
                    .map(|value| {
                        Ok(ConsumablePriceDenominationView {
                            denomination: value.denomination.clone(),
                            amount: i32::try_from(value.amount)?,
                        })
                    })
                    .collect::<Result<Vec<_>, std::num::TryFromIntError>>()
            }),
            per: integer_fact(&value.per),
        }),
        maximum_uses: integer_fact(&definition.maximum_uses),
        auto_destroy: fact(&definition.auto_destroy, |value| *value),
        maximum_hp: integer_fact(&definition.maximum_hp),
        hardness: integer_fact(&definition.hardness),
        publication: fact(&definition.publication, |value| ConsumablePublicationView {
            title: fact(&value.title, Clone::clone),
            license: fact(&value.license, |value| value.as_str().to_string()),
            remaster: fact(&value.remaster, |value| *value),
        }),
        damage: fact(&definition.damage, |value| ConsumableDamageView {
            formula: fact(&value.formula, Clone::clone),
            category: fact(&value.category, Clone::clone),
            damage_type: fact(&value.damage_type, Clone::clone),
        }),
    }
}

fn source_state(state: &ConsumableSourceState) -> ConsumableSourceStateView {
    ConsumableSourceStateView {
        quantity: integer_fact(&state.quantity),
        current_uses: integer_fact(&state.current_uses),
        current_hp: integer_fact(&state.current_hp),
        container_id: fact(&state.container_id, Clone::clone),
        equipped: fact(&state.equipped, |value| ConsumableEquippedView {
            carry_type: fact(&value.carry_type, Clone::clone),
            hands_held: integer_fact(&value.hands_held),
            in_slot: fact(&value.in_slot, |value| *value),
        }),
    }
}

fn integer_fact(fact: &ConsumableFact<i64>) -> ConsumableFactView<i32> {
    match fact {
        FactValue::Missing => ConsumableFactView::Missing,
        FactValue::Null => ConsumableFactView::Null,
        FactValue::Value(ConsumableSourceValue::Known(value)) => i32::try_from(*value)
            .map(ConsumableFactView::Known)
            .unwrap_or(ConsumableFactView::Unsupported),
        FactValue::Value(ConsumableSourceValue::Unsupported(_)) => ConsumableFactView::Unsupported,
    }
}

fn fact_try<T, U, E>(
    fact: &ConsumableFact<T>,
    map: impl FnOnce(&T) -> Result<U, E>,
) -> ConsumableFactView<U> {
    match fact {
        FactValue::Missing => ConsumableFactView::Missing,
        FactValue::Null => ConsumableFactView::Null,
        FactValue::Value(ConsumableSourceValue::Known(value)) => map(value)
            .map(ConsumableFactView::Known)
            .unwrap_or(ConsumableFactView::Unsupported),
        FactValue::Value(ConsumableSourceValue::Unsupported(_)) => ConsumableFactView::Unsupported,
    }
}

fn fact<T, U>(fact: &ConsumableFact<T>, map: impl FnOnce(&T) -> U) -> ConsumableFactView<U> {
    match fact {
        FactValue::Missing => ConsumableFactView::Missing,
        FactValue::Null => ConsumableFactView::Null,
        FactValue::Value(ConsumableSourceValue::Known(value)) => {
            ConsumableFactView::Known(map(value))
        }
        FactValue::Value(ConsumableSourceValue::Unsupported(_)) => ConsumableFactView::Unsupported,
    }
}

fn spell_child_link(child: &atlas_record::ConsumableSpellChild) -> ConsumableSpellChildLinkView {
    ConsumableSpellChildLinkView {
        child_id: child.child_id.as_str().to_string(),
        target_record_key: match &child.standalone_target {
            FactValue::Value(SpellStandaloneTarget::Resolved(record_key)) => {
                Some(record_key.to_string())
            }
            _ => None,
        },
    }
}

fn occurrence_spell_link(
    reuse: &ConsumableSpellReuse,
    target_record_key: Option<&atlas_domain::RecordKey>,
) -> Option<ConsumableSpellChildLinkView> {
    match reuse {
        ConsumableSpellReuse::Reused { target_child_id } => Some(ConsumableSpellChildLinkView {
            child_id: target_child_id.as_str().to_string(),
            target_record_key: target_record_key.map(ToString::to_string),
        }),
        ConsumableSpellReuse::NotPresent | ConsumableSpellReuse::Mismatch { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::RecordKey;
    use atlas_record::{
        ConsumableDefinition, ConsumableEntity, ConsumableEntityId, ConsumableEntityTarget,
        ConsumableOccurrence, ConsumableOccurrenceId, ConsumableOccurrenceIdentityStability,
        ConsumableOccurrenceSet, ConsumableSourceState, ConsumableSpellReuse,
        ConsumableTargetResolution, FactValue, OwnedRichContent,
    };

    #[test]
    fn parent_owned_definition_does_not_repeat_occurrence_state_or_spell_child() {
        let owner_record_key = RecordKey::parse("actors:torchbearer").expect("owner key");
        let entity_id = ConsumableEntityId::new("entity-torch").expect("entity ID");
        let set = ConsumableOccurrenceSet {
            entities: vec![ConsumableEntity {
                id: entity_id.clone(),
                owner_record_key: owner_record_key.clone(),
                target: ConsumableEntityTarget::ParentOwned {
                    definition: Box::new(missing_definition()),
                    resolution: ConsumableTargetResolution::NoLocator,
                    content_identity: FactValue::Missing,
                },
            }],
            occurrences: vec![ConsumableOccurrence {
                id: ConsumableOccurrenceId::new("torch").expect("occurrence ID"),
                source_id: FactValue::Missing,
                identity_stability: ConsumableOccurrenceIdentityStability::StableSourceIdentity,
                owner_record_key,
                entity_id,
                authored_order: 0,
                source_path: "items[0]".to_string(),
                source_sort: FactValue::Missing,
                source_image: FactValue::Missing,
                source_folder: FactValue::Missing,
                contextual_name: "Torch".to_string(),
                locator: atlas_record::ConsumableLocatorState::Missing,
                state: ConsumableSourceState {
                    quantity: FactValue::Value(atlas_record::ConsumableSourceValue::Known(2)),
                    current_uses: FactValue::Missing,
                    current_hp: FactValue::Missing,
                    container_id: FactValue::Null,
                    equipped: FactValue::Missing,
                },
                spell_reuse: ConsumableSpellReuse::NotPresent,
                authored_content: OwnedRichContent::default(),
                unsupported_content: Vec::new(),
            }],
        };

        let views = super::consumable_occurrence_views(&set).expect("one occurrence");
        let json = serde_json::to_value(&views[0]).expect("serialize occurrence");
        assert_eq!(
            json.pointer("/source_state/quantity/value"),
            Some(&serde_json::json!(2))
        );
        let definition = json
            .pointer("/definition")
            .expect("parent-owned definition");
        assert!(definition.get("source_state").is_none());
        assert!(definition.get("spell_child").is_none());
        assert!(definition.get("content").is_none());
    }

    fn missing_definition() -> ConsumableDefinition {
        ConsumableDefinition {
            slug: FactValue::Missing,
            level: FactValue::Missing,
            category: FactValue::Missing,
            rarity: FactValue::Missing,
            traits: FactValue::Missing,
            other_tags: FactValue::Missing,
            base_item: FactValue::Missing,
            bulk: FactValue::Missing,
            size: FactValue::Missing,
            stack_group: FactValue::Missing,
            material: FactValue::Missing,
            price: FactValue::Missing,
            usage: FactValue::Missing,
            maximum_uses: FactValue::Missing,
            auto_destroy: FactValue::Missing,
            maximum_hp: FactValue::Missing,
            hardness: FactValue::Missing,
            damage: FactValue::Missing,
            publication: FactValue::Missing,
            rules: FactValue::Missing,
            spell_child_id: FactValue::Missing,
        }
    }
}
