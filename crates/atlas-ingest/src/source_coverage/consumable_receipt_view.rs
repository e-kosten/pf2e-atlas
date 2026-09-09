//! Explicit H5 typed observers for the existing sealed receipt machinery.
//! These audit views never serve product parsing, projection, or hydration.
use super::{CoverageContractError, CoverageFailureCode};
use crate::source::dto::{ConsumableItemSource, ConsumableSourceFact};
use atlas_record::{
    ConsumableDefinition, ConsumableFact, ConsumableSourceState, ConsumableSourceValue, FactValue,
};
use serde_json::{Map, Value};

type Result<T> = std::result::Result<T, CoverageContractError>;

fn retained(value: &str) -> Result<Value> {
    serde_json::from_str(value).or_else(|_| Ok(Value::String(value.to_string())))
}

fn source_fact<T>(
    fact: &ConsumableSourceFact<T>,
    value: impl FnOnce(&T) -> Result<Value>,
) -> Result<Option<Value>> {
    match fact {
        ConsumableSourceFact::Missing => Ok(None),
        ConsumableSourceFact::Null => Ok(Some(Value::Null)),
        ConsumableSourceFact::Value(fact) => value(fact).map(Some),
        ConsumableSourceFact::Unsupported(fact) => retained(&fact.value).map(Some),
    }
}

fn canonical_fact<T>(
    fact: &ConsumableFact<T>,
    value: impl FnOnce(&T) -> Result<Value>,
) -> Result<Option<Value>> {
    match fact {
        FactValue::Missing => Ok(None),
        FactValue::Null => Ok(Some(Value::Null)),
        FactValue::Value(ConsumableSourceValue::Known(fact)) => value(fact).map(Some),
        FactValue::Value(ConsumableSourceValue::Unsupported(fact)) => {
            retained(&fact.value).map(Some)
        }
    }
}

fn insert(root: &mut Map<String, Value>, path: &[&str], value: Option<Value>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let (last, parents) = path.split_last().ok_or_else(|| CoverageContractError {
        code: CoverageFailureCode::InvalidContract,
        message: "empty H5 observer path".to_string(),
    })?;
    let mut current = root;
    for parent in parents {
        current = current
            .entry((*parent).to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or_else(|| CoverageContractError {
                code: CoverageFailureCode::InvalidContract,
                message: "conflicting H5 observer owner".to_string(),
            })?;
    }
    current.insert((*last).to_string(), value);
    Ok(())
}

fn json<T: serde::Serialize>(value: &T) -> Result<Value> {
    serde_json::to_value(value).map_err(|error| CoverageContractError {
        code: CoverageFailureCode::ReaderNotObserved,
        message: error.to_string(),
    })
}

pub(super) fn source_system(source: &ConsumableItemSource) -> Result<Value> {
    let mut root = Map::new();
    insert(&mut root, &["slug"], source_fact(&source.slug, json)?)?;
    insert(
        &mut root,
        &["level", "value"],
        source_fact(&source.level, json)?,
    )?;
    insert(
        &mut root,
        &["category"],
        source_fact(&source.category, json)?,
    )?;
    insert(
        &mut root,
        &["traits", "rarity"],
        source_fact(&source.rarity, json)?,
    )?;
    insert(
        &mut root,
        &["traits", "value"],
        source_fact(&source.traits, json)?,
    )?;
    insert(
        &mut root,
        &["traits", "otherTags"],
        source_fact(&source.other_tags, json)?,
    )?;
    insert(
        &mut root,
        &["baseItem"],
        source_fact(&source.base_item, json)?,
    )?;
    insert(
        &mut root,
        &["bulk", "value"],
        source_fact(&source.bulk, json)?,
    )?;
    insert(&mut root, &["size"], source_fact(&source.size, json)?)?;
    insert(
        &mut root,
        &["stackGroup"],
        source_fact(&source.stack_group, json)?,
    )?;
    insert(
        &mut root,
        &["quantity"],
        source_fact(&source.quantity, json)?,
    )?;
    insert(
        &mut root,
        &["usage", "value"],
        source_fact(&source.usage, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "max"],
        source_fact(&source.maximum_uses, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "value"],
        source_fact(&source.current_uses, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "autoDestroy"],
        source_fact(&source.auto_destroy, json)?,
    )?;
    insert(
        &mut root,
        &["containerId"],
        source_fact(&source.container_id, json)?,
    )?;
    insert(
        &mut root,
        &["hp", "max"],
        source_fact(&source.maximum_hp, json)?,
    )?;
    insert(
        &mut root,
        &["hp", "value"],
        source_fact(&source.current_hp, json)?,
    )?;
    insert(
        &mut root,
        &["hardness"],
        source_fact(&source.hardness, json)?,
    )?;
    insert(
        &mut root,
        &["equipped"],
        source_fact(&source.equipped, |owner| {
            let mut nested = Map::new();
            insert(
                &mut nested,
                &["carryType"],
                source_fact(&owner.carry_type, json)?,
            )?;
            insert(
                &mut nested,
                &["handsHeld"],
                source_fact(&owner.hands_held, json)?,
            )?;
            insert(&mut nested, &["inSlot"], source_fact(&owner.in_slot, json)?)?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["material"],
        source_fact(&source.material, |owner| {
            let mut nested = Map::new();
            insert(&mut nested, &["grade"], source_fact(&owner.grade, json)?)?;
            insert(
                &mut nested,
                &["type"],
                source_fact(&owner.material_type, json)?,
            )?;
            insert(
                &mut nested,
                &["effects"],
                source_fact(&owner.effects, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["damage"],
        source_fact(&source.damage, |owner| {
            let mut nested = Map::new();
            insert(
                &mut nested,
                &["formula"],
                source_fact(&owner.formula, json)?,
            )?;
            insert(&mut nested, &["kind"], source_fact(&owner.category, json)?)?;
            insert(
                &mut nested,
                &["type"],
                source_fact(&owner.damage_type, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["publication"],
        source_fact(&source.publication, |owner| {
            let mut nested = Map::new();
            insert(&mut nested, &["title"], source_fact(&owner.title, json)?)?;
            insert(
                &mut nested,
                &["license"],
                source_fact(&owner.license, json)?,
            )?;
            insert(
                &mut nested,
                &["remaster"],
                source_fact(&owner.remaster, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["price"],
        source_fact(&source.price, |price| {
            let mut nested = Map::new();
            insert(&mut nested, &["per"], source_fact(&price.per, json)?)?;
            insert(
                &mut nested,
                &["value"],
                source_fact(&price.denominations, |values| {
                    Ok(Value::Object(
                        values
                            .iter()
                            .map(|(key, value)| (key.clone(), Value::from(*value)))
                            .collect(),
                    ))
                })?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["rules"],
        source_fact(&source.rules, |rules| {
            rules
                .iter()
                .map(|rule| retained(&rule.value))
                .collect::<Result<Vec<_>>>()
                .map(Value::Array)
        })?,
    )?;
    insert(
        &mut root,
        &["__h5_rule_evidence"],
        source_fact(&source.rules, |rules| {
            rules
                .iter()
                .map(|rule| {
                    if rule.reason != "rule execution is outside H5" {
                        return Err(CoverageContractError {
                            code: CoverageFailureCode::ReaderNotObserved,
                            message: "unexpected source rule retention reason".to_string(),
                        });
                    }
                    Ok(
                        serde_json::json!({"shape":format!("{:?}", rule.shape).to_ascii_lowercase(),
                    "value":rule.value,"reason":"source_field_drift"}),
                    )
                })
                .collect::<Result<Vec<_>>>()
                .map(Value::Array)
        })?,
    )?;
    Ok(Value::Object(root))
}

pub(super) fn canonical_system(
    definition: &ConsumableDefinition,
    state: &ConsumableSourceState,
) -> Result<Value> {
    let mut root = Map::new();
    insert(
        &mut root,
        &["slug"],
        canonical_fact(&definition.slug, json)?,
    )?;
    insert(
        &mut root,
        &["level", "value"],
        canonical_fact(&definition.level, json)?,
    )?;
    insert(
        &mut root,
        &["category"],
        canonical_fact(&definition.category, json)?,
    )?;
    insert(
        &mut root,
        &["traits", "rarity"],
        canonical_fact(&definition.rarity, |value| json(&value.as_str()))?,
    )?;
    insert(
        &mut root,
        &["traits", "value"],
        canonical_fact(&definition.traits, json)?,
    )?;
    insert(
        &mut root,
        &["traits", "otherTags"],
        canonical_fact(&definition.other_tags, json)?,
    )?;
    insert(
        &mut root,
        &["baseItem"],
        canonical_fact(&definition.base_item, json)?,
    )?;
    insert(
        &mut root,
        &["bulk", "value"],
        canonical_fact(&definition.bulk, |value| json(&value.as_str()))?,
    )?;
    insert(
        &mut root,
        &["size"],
        canonical_fact(&definition.size, json)?,
    )?;
    insert(
        &mut root,
        &["stackGroup"],
        canonical_fact(&definition.stack_group, json)?,
    )?;
    insert(
        &mut root,
        &["quantity"],
        canonical_fact(&state.quantity, json)?,
    )?;
    insert(
        &mut root,
        &["usage", "value"],
        canonical_fact(&definition.usage, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "max"],
        canonical_fact(&definition.maximum_uses, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "value"],
        canonical_fact(&state.current_uses, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "autoDestroy"],
        canonical_fact(&definition.auto_destroy, json)?,
    )?;
    insert(
        &mut root,
        &["containerId"],
        canonical_fact(&state.container_id, json)?,
    )?;
    insert(
        &mut root,
        &["hp", "max"],
        canonical_fact(&definition.maximum_hp, json)?,
    )?;
    insert(
        &mut root,
        &["hp", "value"],
        canonical_fact(&state.current_hp, json)?,
    )?;
    insert(
        &mut root,
        &["hardness"],
        canonical_fact(&definition.hardness, json)?,
    )?;
    insert(
        &mut root,
        &["equipped"],
        canonical_fact(&state.equipped, |owner| {
            let mut nested = Map::new();
            insert(
                &mut nested,
                &["carryType"],
                canonical_fact(&owner.carry_type, json)?,
            )?;
            insert(
                &mut nested,
                &["handsHeld"],
                canonical_fact(&owner.hands_held, json)?,
            )?;
            insert(
                &mut nested,
                &["inSlot"],
                canonical_fact(&owner.in_slot, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["material"],
        canonical_fact(&definition.material, |owner| {
            let mut nested = Map::new();
            insert(&mut nested, &["grade"], canonical_fact(&owner.grade, json)?)?;
            insert(
                &mut nested,
                &["type"],
                canonical_fact(&owner.material_type, json)?,
            )?;
            insert(
                &mut nested,
                &["effects"],
                canonical_fact(&owner.effects, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["damage"],
        canonical_fact(&definition.damage, |owner| {
            let mut nested = Map::new();
            insert(
                &mut nested,
                &["formula"],
                canonical_fact(&owner.formula, json)?,
            )?;
            insert(
                &mut nested,
                &["kind"],
                canonical_fact(&owner.category, json)?,
            )?;
            insert(
                &mut nested,
                &["type"],
                canonical_fact(&owner.damage_type, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["publication"],
        canonical_fact(&definition.publication, |owner| {
            let mut nested = Map::new();
            insert(&mut nested, &["title"], canonical_fact(&owner.title, json)?)?;
            insert(
                &mut nested,
                &["license"],
                canonical_fact(&owner.license, |value| json(&value.as_str()))?,
            )?;
            insert(
                &mut nested,
                &["remaster"],
                canonical_fact(&owner.remaster, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["price"],
        canonical_fact(&definition.price, |price| {
            let mut nested = Map::new();
            insert(&mut nested, &["per"], canonical_fact(&price.per, json)?)?;
            insert(
                &mut nested,
                &["value"],
                canonical_fact(&price.denominations, |values| {
                    Ok(Value::Object(
                        values
                            .iter()
                            .map(|value| (value.denomination.clone(), Value::from(value.amount)))
                            .collect(),
                    ))
                })?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["rules"],
        canonical_fact(&definition.rules, |rules| {
            rules
                .iter()
                .map(|rule| retained(&rule.value))
                .collect::<Result<Vec<_>>>()
                .map(Value::Array)
        })?,
    )?;
    insert(
        &mut root,
        &["__h5_rule_evidence"],
        canonical_fact(&definition.rules, |rules| {
            rules
                .iter()
                .map(|rule| {
                    let reason = match rule.reason {
                        atlas_record::UnsupportedSourceReason::SourceFieldDrift => {
                            "source_field_drift"
                        }
                        _ => {
                            return Err(CoverageContractError {
                                code: CoverageFailureCode::CanonicalMismatch,
                                message: "canonical rule retention reason changed".to_string(),
                            });
                        }
                    };
                    Ok(
                        serde_json::json!({"shape":format!("{:?}", rule.shape).to_ascii_lowercase(),
                    "value":rule.value,"reason":reason}),
                    )
                })
                .collect::<Result<Vec<_>>>()
                .map(Value::Array)
        })?,
    )?;
    Ok(Value::Object(root))
}

pub(super) fn source_item(item: &crate::source::dto::FullItemSource) -> Result<Value> {
    let definition = item
        .consumable
        .as_ref()
        .ok_or_else(|| CoverageContractError {
            code: CoverageFailureCode::ReaderNotObserved,
            message: "missing H5 source DTO".to_string(),
        })?;
    let mut root = Map::new();
    root.insert("_id".to_string(), Value::String(item.id.clone()));
    root.insert("name".to_string(), Value::String(item.name.clone()));
    root.insert(
        "type".to_string(),
        Value::String(item.item_type.as_str().to_string()),
    );
    for (key, fact) in [("img", &item.image), ("folder", &item.folder)] {
        insert(
            &mut root,
            &[key],
            match fact {
                crate::SourcePresence::Missing => None,
                crate::SourcePresence::Null => Some(Value::Null),
                crate::SourcePresence::Value(value) => Some(Value::String(value.clone())),
            },
        )?;
    }
    insert(
        &mut root,
        &["sort"],
        match &item.sort {
            crate::SourcePresence::Missing => None,
            crate::SourcePresence::Null => Some(Value::Null),
            crate::SourcePresence::Value(value) => Some(Value::from(*value)),
        },
    )?;
    insert(
        &mut root,
        &["_stats", "compendiumSource"],
        source_fact(&definition.target_locator, json)?,
    )?;
    root.insert("system".to_string(), source_system(definition)?);
    Ok(Value::Object(root))
}

pub(super) fn canonical_item(
    record: &atlas_record::RetrievedRecord,
    ordinal: Option<u32>,
) -> Result<Value> {
    let mut root = Map::new();
    let (definition, state, locator) = if let Some(ordinal) = ordinal {
        let occurrence = record
            .consumable_occurrences
            .occurrences
            .iter()
            .find(|occurrence| occurrence.authored_order == ordinal)
            .ok_or_else(|| CoverageContractError {
                code: CoverageFailureCode::CanonicalMismatch,
                message: "missing exact H5 occurrence".to_string(),
            })?;
        let entity = record
            .consumable_occurrences
            .entities
            .iter()
            .find(|entity| entity.id == occurrence.entity_id)
            .ok_or_else(|| CoverageContractError {
                code: CoverageFailureCode::CanonicalMismatch,
                message: "missing exact H5 entity".to_string(),
            })?;
        let atlas_record::ConsumableEntityTarget::ParentOwned { definition, .. } = &entity.target
        else {
            return Err(CoverageContractError {
                code: CoverageFailureCode::CanonicalMismatch,
                message: "isolated receipt unexpectedly resolved an external target".to_string(),
            });
        };
        insert(
            &mut root,
            &["_id"],
            canonical_fact(&occurrence.source_id, |id| json(&id.as_str()))?,
        )?;
        root.insert(
            "name".to_string(),
            Value::String(occurrence.contextual_name.clone()),
        );
        insert(
            &mut root,
            &["img"],
            canonical_fact(&occurrence.source_image, json)?,
        )?;
        insert(
            &mut root,
            &["folder"],
            canonical_fact(&occurrence.source_folder, json)?,
        )?;
        insert(
            &mut root,
            &["sort"],
            canonical_fact(&occurrence.source_sort, json)?,
        )?;
        (definition.as_ref(), &occurrence.state, &occurrence.locator)
    } else {
        let body = record
            .body
            .as_ref()
            .and_then(atlas_record::RecordBody::as_consumable)
            .ok_or_else(|| CoverageContractError {
                code: CoverageFailureCode::CanonicalMismatch,
                message: "missing H5 canonical body".to_string(),
            })?;
        root.insert(
            "_id".to_string(),
            Value::String(body.identity.source_id.as_str().to_string()),
        );
        root.insert(
            "name".to_string(),
            Value::String(body.identity.name.clone()),
        );
        insert(
            &mut root,
            &["img"],
            canonical_fact(&body.provenance.image, json)?,
        )?;
        insert(
            &mut root,
            &["folder"],
            canonical_fact(&body.provenance.folder, json)?,
        )?;
        insert(
            &mut root,
            &["sort"],
            canonical_fact(&body.provenance.source_sort, json)?,
        )?;
        (
            &body.definition,
            &body.source_state,
            &body.provenance.target_locator,
        )
    };
    root.insert("type".to_string(), Value::String("consumable".to_string()));
    insert(
        &mut root,
        &["_stats", "compendiumSource"],
        match locator {
            atlas_record::ConsumableLocatorState::Missing => None,
            atlas_record::ConsumableLocatorState::Null => Some(Value::Null),
            atlas_record::ConsumableLocatorState::Known(value) => {
                Some(Value::String(value.as_str().to_string()))
            }
            atlas_record::ConsumableLocatorState::Unsupported(value) => {
                Some(retained(&value.value)?)
            }
        },
    )?;
    root.insert("system".to_string(), canonical_system(definition, state)?);
    Ok(Value::Object(root))
}

fn public_fact<T>(
    fact: &atlas_record::ConsumableFactJson<T>,
    value: impl FnOnce(&T) -> Result<Value>,
) -> Result<Option<Value>> {
    use atlas_record::ConsumableFactJson as Fact;
    match fact {
        Fact::Missing => Ok(None),
        Fact::Null => Ok(Some(Value::Null)),
        Fact::Known(fact) => value(fact).map(Some),
        Fact::Unsupported(fact) => retained(&fact.value).map(Some),
    }
}

pub(super) fn public_system(
    definition: &atlas_record::ConsumableDefinitionJson,
    state: &atlas_record::ConsumableSourceStateJson,
) -> Result<Value> {
    let mut root = Map::new();
    insert(&mut root, &["slug"], public_fact(&definition.slug, json)?)?;
    insert(
        &mut root,
        &["level", "value"],
        public_fact(&definition.level, json)?,
    )?;
    insert(
        &mut root,
        &["category"],
        public_fact(&definition.category, json)?,
    )?;
    insert(
        &mut root,
        &["traits", "rarity"],
        public_fact(&definition.rarity, |value| json(&value.as_str()))?,
    )?;
    insert(
        &mut root,
        &["traits", "value"],
        public_fact(&definition.traits, json)?,
    )?;
    insert(
        &mut root,
        &["traits", "otherTags"],
        public_fact(&definition.other_tags, json)?,
    )?;
    insert(
        &mut root,
        &["baseItem"],
        public_fact(&definition.base_item, json)?,
    )?;
    insert(
        &mut root,
        &["bulk", "value"],
        public_fact(&definition.bulk, |value| json(&value.as_str()))?,
    )?;
    insert(&mut root, &["size"], public_fact(&definition.size, json)?)?;
    insert(
        &mut root,
        &["stackGroup"],
        public_fact(&definition.stack_group, json)?,
    )?;
    insert(
        &mut root,
        &["quantity"],
        public_fact(&state.quantity, json)?,
    )?;
    insert(
        &mut root,
        &["usage", "value"],
        public_fact(&definition.usage, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "max"],
        public_fact(&definition.maximum_uses, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "value"],
        public_fact(&state.current_uses, json)?,
    )?;
    insert(
        &mut root,
        &["uses", "autoDestroy"],
        public_fact(&definition.auto_destroy, json)?,
    )?;
    insert(
        &mut root,
        &["containerId"],
        public_fact(&state.container_id, json)?,
    )?;
    insert(
        &mut root,
        &["hp", "max"],
        public_fact(&definition.maximum_hp, json)?,
    )?;
    insert(
        &mut root,
        &["hp", "value"],
        public_fact(&state.current_hp, json)?,
    )?;
    insert(
        &mut root,
        &["hardness"],
        public_fact(&definition.hardness, json)?,
    )?;
    insert(
        &mut root,
        &["equipped"],
        public_fact(&state.equipped, |owner| {
            let mut nested = Map::new();
            insert(
                &mut nested,
                &["carryType"],
                public_fact(&owner.carry_type, json)?,
            )?;
            insert(
                &mut nested,
                &["handsHeld"],
                public_fact(&owner.hands_held, json)?,
            )?;
            insert(&mut nested, &["inSlot"], public_fact(&owner.in_slot, json)?)?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["material"],
        public_fact(&definition.material, |owner| {
            let mut nested = Map::new();
            insert(&mut nested, &["grade"], public_fact(&owner.grade, json)?)?;
            insert(
                &mut nested,
                &["type"],
                public_fact(&owner.material_type, json)?,
            )?;
            insert(
                &mut nested,
                &["effects"],
                public_fact(&owner.effects, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["damage"],
        public_fact(&definition.damage, |owner| {
            let mut nested = Map::new();
            insert(
                &mut nested,
                &["formula"],
                public_fact(&owner.formula, json)?,
            )?;
            insert(&mut nested, &["kind"], public_fact(&owner.category, json)?)?;
            insert(
                &mut nested,
                &["type"],
                public_fact(&owner.damage_type, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["publication"],
        public_fact(&definition.publication, |owner| {
            let mut nested = Map::new();
            insert(&mut nested, &["title"], public_fact(&owner.title, json)?)?;
            insert(
                &mut nested,
                &["license"],
                public_fact(&owner.license, |value| json(&value.as_str()))?,
            )?;
            insert(
                &mut nested,
                &["remaster"],
                public_fact(&owner.remaster, json)?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["price"],
        public_fact(&definition.price, |price| {
            let mut nested = Map::new();
            insert(&mut nested, &["per"], public_fact(&price.per, json)?)?;
            insert(
                &mut nested,
                &["value"],
                public_fact(&price.denominations, |values| {
                    Ok(Value::Object(
                        values
                            .iter()
                            .map(|value| (value.denomination.clone(), Value::from(value.amount)))
                            .collect(),
                    ))
                })?,
            )?;
            Ok(Value::Object(nested))
        })?,
    )?;
    insert(
        &mut root,
        &["rules"],
        public_fact(&definition.rules, |rules| {
            rules
                .iter()
                .map(|rule| retained(&rule.value))
                .collect::<Result<Vec<_>>>()
                .map(Value::Array)
        })?,
    )?;
    insert(
        &mut root,
        &["__h5_rule_evidence"],
        public_fact(&definition.rules, |rules| {
            rules.iter().map(|rule| { Ok(serde_json::json!({"shape":rule.shape,"value":rule.value,"reason":rule.reason})) }).collect::<Result<Vec<_>>>().map(Value::Array)
        })?,
    )?;
    Ok(Value::Object(root))
}
