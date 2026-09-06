use atlas_record::*;
use serde_json::{Map, Value};

use crate::source::dto::{SpellDocumentSource, SpellItemSource, VersionedItemSource};

pub(crate) fn source_view(document: &SpellDocumentSource, common: &VersionedItemSource) -> Value {
    match document {
        SpellDocumentSource::Standalone(spell) => {
            let item = common.source.source();
            let mut root = Map::new();
            root.insert("_id".into(), Value::String(spell.id.clone()));
            root.insert("name".into(), Value::String(spell.name.clone()));
            root.insert("type".into(), Value::String(item.item_type.as_str().into()));
            insert_source_presence(&mut root, "folder", &item.folder, |value| {
                Value::String(value.clone())
            });
            insert_spell_fact(&mut root, "img", &spell.image, |value| {
                Value::String(value.clone())
            });
            root.insert("system".into(), source_system_view(spell));
            Value::Object(root)
        }
        SpellDocumentSource::ConsumableChild(child) => child_source_view(child),
    }
}

pub(crate) fn canonical_spell_view(record: &AtlasRecord, spell: &SpellRecord) -> Value {
    let mut root = Map::new();
    root.insert(
        "_id".into(),
        Value::String(spell.identity.source_id.as_str().to_string()),
    );
    root.insert("name".into(), Value::String(spell.identity.name.clone()));
    root.insert("type".into(), Value::String("spell".into()));
    if let Some(folder) = &record.foundry.folder_id {
        root.insert("folder".into(), Value::String(folder.clone()));
    }
    insert_spell_fact(
        &mut root,
        "img",
        &spell.definition.source_context.image,
        |value| Value::String(value.clone()),
    );
    let mut system = definition_system_view(&spell.definition);
    if let Some(title) = &record.publication.title {
        object_member(&mut system, "publication")
            .insert("title".into(), Value::String(title.clone()));
    }
    object_member(&mut system, "publication")
        .insert("remaster".into(), Value::Bool(record.publication.remaster));
    if let Some(rarity) = &record.classification.rarity {
        object_member(&mut system, "traits")
            .insert("rarity".into(), Value::String(rarity.as_str().to_string()));
    }
    root.insert("system".into(), Value::Object(system));
    Value::Object(root)
}

pub(crate) fn canonical_child_view(child: &ConsumableSpellChild) -> Value {
    let mut root = Map::new();
    root.insert(
        "_id".into(),
        Value::String(child.child_id.as_str().to_string()),
    );
    insert_spell_fact(&mut root, "name", &child.name, |value| {
        Value::String(value.clone())
    });
    root.insert("type".into(), Value::String("spell".into()));
    insert_spell_fact(
        object_member(object_member(&mut root, "flags"), "core"),
        "sourceId",
        &child.standalone_locator,
        |value| Value::String(value.as_str().to_string()),
    );
    insert_spell_fact(
        &mut root,
        "img",
        &child.definition.source_context.image,
        |value| Value::String(value.clone()),
    );
    let mut system = definition_system_view(&child.definition);
    if let FactValue::Value(source) = &child.definition.source_context.consumable_child {
        insert_spell_fact(&mut system, "slug", &source.slug, |value| {
            Value::String(value.clone())
        });
        let publication = object_member(&mut system, "publication");
        insert_spell_fact(publication, "title", &source.publication_title, |value| {
            Value::String(value.clone())
        });
        insert_spell_fact(
            publication,
            "remaster",
            &source.publication_remaster,
            |value| Value::Bool(*value),
        );
        let traits = object_member(&mut system, "traits");
        insert_spell_fact(traits, "rarity", &source.rarity, |value| {
            Value::String(value.as_str().to_string())
        });
    }
    insert_spell_fact(&mut system, "location", &child.location, location_value);
    root.insert("system".into(), Value::Object(system));
    Value::Object(root)
}

fn child_source_view(child: &crate::source::dto::ConsumableSpellChildSource) -> Value {
    let spell = &child.spell;
    let mut root = Map::new();
    root.insert("_id".into(), Value::String(spell.id.clone()));
    root.insert("name".into(), Value::String(spell.name.clone()));
    root.insert("type".into(), Value::String("spell".into()));
    insert_spell_fact(
        object_member(object_member(&mut root, "flags"), "core"),
        "sourceId",
        &child.standalone_locator,
        |value| Value::String(value.as_str().to_string()),
    );
    insert_spell_fact(&mut root, "img", &spell.image, |value| {
        Value::String(value.clone())
    });
    root.insert("system".into(), source_system_view(spell));
    Value::Object(root)
}

fn source_system_view(spell: &SpellItemSource) -> Value {
    let mut system = definition_fields_view(DefinitionFields {
        classification: &spell.classification,
        casting: &spell.casting,
        targeting: &spell.targeting,
        defense: &spell.defense,
        damage: &spell.damage,
        duration: &spell.duration,
        heightening: &spell.heightening,
        overlays: &spell.overlays,
        ritual: &spell.ritual,
        rules: &spell.rules,
        unsupported_notes: &spell.unsupported_notes,
    });
    insert_spell_fact(&mut system, "slug", &spell.slug, |value| {
        Value::String(value.clone())
    });
    let publication = object_member(&mut system, "publication");
    insert_spell_fact(
        publication,
        "license",
        &spell.publication_license,
        |value| Value::String(value.as_str().to_string()),
    );
    insert_spell_fact(publication, "title", &spell.publication_title, |value| {
        Value::String(value.clone())
    });
    insert_spell_fact(
        publication,
        "remaster",
        &spell.publication_remaster,
        |value| Value::Bool(*value),
    );
    let traits = object_member(&mut system, "traits");
    insert_spell_fact(traits, "rarity", &spell.rarity, |value| {
        Value::String(value.as_str().to_string())
    });
    insert_spell_fact(&mut system, "location", &spell.location, location_value);
    insert_spell_fact(
        object_member(&mut system, "description"),
        "value",
        &spell.description_markup,
        |value| Value::String(value.clone()),
    );
    Value::Object(system)
}

fn definition_system_view(definition: &SpellDefinition) -> Map<String, Value> {
    let mut system = definition_fields_view(DefinitionFields {
        classification: &definition.classification,
        casting: &definition.casting,
        targeting: &definition.targeting,
        defense: &definition.defense,
        damage: &definition.damage,
        duration: &definition.duration,
        heightening: &definition.heightening,
        overlays: &definition.overlays,
        ritual: &definition.ritual,
        rules: &definition.rules,
        unsupported_notes: &definition.unsupported_notes,
    });
    insert_spell_fact(
        object_member(&mut system, "publication"),
        "license",
        &definition.source_context.publication_license,
        |value| Value::String(value.as_str().to_string()),
    );
    if !matches!(
        definition.provenance.standalone_location,
        FactValue::Missing
    ) {
        system.insert(
            "location".into(),
            unsupported_fact_value(&definition.provenance.standalone_location),
        );
    }
    system
}

struct DefinitionFields<'a> {
    classification: &'a SpellFact<SpellClassification>,
    casting: &'a SpellFact<SpellCasting>,
    targeting: &'a SpellFact<SpellTargeting>,
    defense: &'a SpellFact<SpellDefenseValue>,
    damage: &'a SpellFact<Vec<SpellOrderedMember<SpellDamage>>>,
    duration: &'a SpellFact<SpellDuration>,
    heightening: &'a SpellFact<SpellHeightening>,
    overlays: &'a SpellFact<Vec<SpellOverlay>>,
    ritual: &'a SpellFact<SpellRitual>,
    rules: &'a SpellFact<Vec<SpellRuleElement>>,
    unsupported_notes: &'a [SpellUnsupportedSourceFact],
}

fn definition_fields_view(fields: DefinitionFields<'_>) -> Map<String, Value> {
    let mut system = Map::new();
    insert_spell_fact(
        &mut system,
        "level",
        fields.classification,
        |classification| {
            let mut level = Map::new();
            insert_spell_fact(&mut level, "value", &classification.rank, |value| {
                Value::from(*value)
            });
            Value::Object(level)
        },
    );
    if let Some(SpellSourceValue::Known(classification)) = fields.classification.as_value() {
        let traits = object_member(&mut system, "traits");
        insert_spell_fact(traits, "value", &classification.traits, |values| {
            Value::Array(
                values
                    .iter()
                    .map(|value| Value::String(value.as_str().to_string()))
                    .collect(),
            )
        });
        insert_spell_fact(traits, "traditions", &classification.traditions, |values| {
            Value::Array(
                values
                    .iter()
                    .map(|value| Value::String(value.as_str().to_string()))
                    .collect(),
            )
        });
    }
    if let Some(SpellSourceValue::Known(casting)) = fields.casting.as_value() {
        insert_nested_string(&mut system, "time", "value", &casting.time);
        insert_nested_string(&mut system, "cost", "value", &casting.cost);
        insert_spell_fact(
            &mut system,
            "requirements",
            &casting.requirements,
            |value| Value::String(value.clone()),
        );
        insert_spell_fact(
            &mut system,
            "counteraction",
            &casting.counteraction,
            |value| Value::Bool(*value),
        );
    } else {
        insert_spell_fact(&mut system, "casting", fields.casting, |_| Value::Null);
    }
    if let Some(SpellSourceValue::Known(targeting)) = fields.targeting.as_value() {
        insert_nested_string(&mut system, "target", "value", &targeting.target);
        let range = object_member(&mut system, "range");
        insert_spell_fact(range, "value", &targeting.range, |value| {
            Value::String(value.authored_text.clone())
        });
        insert_spell_fact(&mut system, "area", &targeting.area, area_value);
    }
    insert_spell_fact(&mut system, "defense", fields.defense, defense_value);
    insert_spell_fact(&mut system, "damage", fields.damage, |values| {
        damage_map(values)
    });
    insert_spell_fact(&mut system, "duration", fields.duration, duration_value);
    insert_spell_fact(
        &mut system,
        "heightening",
        fields.heightening,
        heightening_value,
    );
    insert_spell_fact(&mut system, "overlays", fields.overlays, |values| {
        overlay_map(values)
    });
    insert_spell_fact(&mut system, "ritual", fields.ritual, ritual_value);
    insert_spell_fact(&mut system, "rules", fields.rules, |values| {
        rules_value(values)
    });
    let mut selected = Map::new();
    for note in fields
        .unsupported_notes
        .iter()
        .filter(|note| note.field == SpellUnsupportedSourceField::LegacyTraitSelection)
    {
        selected.insert(note.authored_key.clone(), unsupported_value(&note.value));
    }
    if !selected.is_empty() {
        object_member(&mut system, "traits").insert("selected".into(), Value::Object(selected));
    }
    system
}

fn insert_nested_string(
    system: &mut Map<String, Value>,
    root: &str,
    leaf: &str,
    fact: &SpellFact<String>,
) {
    insert_spell_fact(object_member(system, root), leaf, fact, |value| {
        Value::String(value.clone())
    });
}

fn area_value(area: &SpellAreaValue) -> Value {
    let mut object = Map::new();
    insert_spell_fact(&mut object, "value", &area.value, |value| {
        Value::from(*value)
    });
    insert_spell_fact(&mut object, "type", &area.area_type, |value| {
        Value::String(value.as_str().to_string())
    });
    insert_spell_fact(&mut object, "areaType", &area.legacy_area_type, |value| {
        Value::String(value.value.as_str().to_string())
    });
    insert_spell_fact(&mut object, "details", &area.details, |value| {
        Value::String(value.clone())
    });
    Value::Object(object)
}

fn area_patch_value(area: &SpellAreaPatch) -> Value {
    let mut object = Map::new();
    insert_spell_fact(&mut object, "value", &area.value, |value| {
        Value::from(*value)
    });
    insert_spell_fact(&mut object, "type", &area.area_type, |value| {
        Value::String(value.as_str().to_string())
    });
    insert_spell_fact(&mut object, "areaType", &area.legacy_area_type, |value| {
        Value::String(value.value.as_str().to_string())
    });
    insert_spell_fact(&mut object, "details", &area.details, |value| {
        Value::String(value.clone())
    });
    Value::Object(object)
}

fn defense_value(defense: &SpellDefenseValue) -> Value {
    let mut object = Map::new();
    insert_spell_fact(&mut object, "passive", &defense.passive, |value| {
        let mut passive = Map::new();
        passive.insert(
            "statistic".into(),
            Value::String(value.as_str().to_string()),
        );
        Value::Object(passive)
    });
    insert_spell_fact(&mut object, "save", &defense.save, save_value);
    Value::Object(object)
}

fn save_value(save: &SpellSave) -> Value {
    let mut object = Map::new();
    insert_spell_fact(&mut object, "statistic", &save.statistic, |value| {
        Value::String(value.as_str().to_string())
    });
    insert_spell_fact(&mut object, "basic", &save.basic, |value| {
        Value::Bool(*value)
    });
    Value::Object(object)
}

fn damage_map(values: &[SpellOrderedMember<SpellDamage>]) -> Value {
    Value::Object(
        values
            .iter()
            .map(|member| (member.key.clone(), damage_value(&member.value)))
            .collect(),
    )
}

fn damage_value(damage: &SpellDamage) -> Value {
    let mut object = Map::new();
    insert_spell_fact(&mut object, "applyMod", &damage.apply_mod, |value| {
        Value::Bool(*value)
    });
    insert_spell_fact(&mut object, "category", &damage.category, |value| {
        Value::String(value.clone())
    });
    insert_spell_fact(&mut object, "formula", &damage.formula, |value| {
        Value::String(value.clone())
    });
    insert_spell_fact(&mut object, "kinds", &damage.kinds, |values| {
        string_array(values)
    });
    insert_spell_fact(&mut object, "materials", &damage.materials, |values| {
        string_array(values)
    });
    insert_spell_fact(&mut object, "type", &damage.damage_type, |value| {
        Value::String(value.clone())
    });
    Value::Object(object)
}

fn duration_value(duration: &SpellDuration) -> Value {
    let mut object = Map::new();
    insert_spell_fact(&mut object, "value", &duration.value, |value| {
        Value::String(value.clone())
    });
    insert_spell_fact(&mut object, "sustained", &duration.sustained, |value| {
        Value::Bool(*value)
    });
    Value::Object(object)
}

fn heightening_value(heightening: &SpellHeightening) -> Value {
    let mut object = Map::new();
    match heightening {
        SpellHeightening::Interval(interval) => {
            object.insert("type".into(), Value::String("interval".into()));
            insert_spell_fact(&mut object, "interval", &interval.interval, |value| {
                Value::from(*value)
            });
            insert_spell_fact(&mut object, "area", &interval.area, |value| {
                Value::from(*value)
            });
            insert_spell_fact(&mut object, "damage", &interval.damage, |members| {
                Value::Object(
                    members
                        .iter()
                        .map(|member| (member.key.clone(), Value::String(member.value.clone())))
                        .collect(),
                )
            });
        }
        SpellHeightening::Fixed(layers) => {
            object.insert("type".into(), Value::String("fixed".into()));
            object.insert(
                "levels".into(),
                Value::Object(
                    layers
                        .iter()
                        .map(|layer| (layer.key.clone(), patch_value(&layer.patch)))
                        .collect(),
                ),
            );
        }
    }
    Value::Object(object)
}

fn overlay_map(overlays: &[SpellOverlay]) -> Value {
    Value::Object(
        overlays
            .iter()
            .map(|overlay| {
                let mut object = Map::new();
                insert_spell_fact(&mut object, "_id", &overlay.source_id, |value| {
                    Value::String(value.as_str().to_string())
                });
                insert_spell_fact(&mut object, "name", &overlay.name, |value| {
                    Value::String(value.clone())
                });
                insert_spell_fact(&mut object, "overlayType", &overlay.overlay_type, |_| {
                    Value::String("override".into())
                });
                insert_spell_fact(&mut object, "sort", &overlay.sort, |value| {
                    Value::from(*value)
                });
                object.insert("system".into(), patch_value(&overlay.patch));
                (overlay.key.clone(), Value::Object(object))
            })
            .collect(),
    )
}

fn patch_value(patch: &SpellPatch) -> Value {
    let mut system = Map::new();
    if let Some(SpellSourceValue::Known(classification)) = patch.classification.as_value() {
        insert_spell_fact(
            object_member(&mut system, "level"),
            "value",
            &classification.rank,
            |value| Value::from(*value),
        );
        let traits = object_member(&mut system, "traits");
        insert_spell_fact(traits, "value", &classification.traits, |values| {
            Value::Array(
                values
                    .iter()
                    .map(|value| Value::String(value.as_str().to_string()))
                    .collect(),
            )
        });
        insert_spell_fact(traits, "traditions", &classification.traditions, |values| {
            Value::Array(
                values
                    .iter()
                    .map(|value| Value::String(value.as_str().to_string()))
                    .collect(),
            )
        });
    }
    if let Some(SpellSourceValue::Known(casting)) = patch.casting.as_value() {
        insert_nested_string(&mut system, "time", "value", &casting.time);
        insert_nested_string(&mut system, "cost", "value", &casting.cost);
        insert_spell_fact(
            &mut system,
            "requirements",
            &casting.requirements,
            |value| Value::String(value.clone()),
        );
        insert_spell_fact(
            &mut system,
            "counteraction",
            &casting.counteraction,
            |value| Value::Bool(*value),
        );
    }
    if let Some(SpellSourceValue::Known(targeting)) = patch.targeting.as_value() {
        insert_nested_string(&mut system, "target", "value", &targeting.target);
        insert_spell_fact(
            object_member(&mut system, "range"),
            "value",
            &targeting.range,
            |value| Value::String(value.authored_text.clone()),
        );
        insert_spell_fact(&mut system, "area", &targeting.area, area_patch_value);
    }
    if let Some(SpellSourceValue::Known(defense)) = patch.defense.as_value() {
        let defense_object = object_member(&mut system, "defense");
        insert_spell_fact(defense_object, "passive", &defense.passive, |value| {
            let mut object = Map::new();
            object.insert(
                "statistic".into(),
                Value::String(value.as_str().to_string()),
            );
            Value::Object(object)
        });
        insert_spell_fact(defense_object, "save", &defense.save, |save| {
            let mut object = Map::new();
            insert_spell_fact(&mut object, "statistic", &save.statistic, |value| {
                Value::String(value.as_str().to_string())
            });
            insert_spell_fact(&mut object, "basic", &save.basic, |value| {
                Value::Bool(*value)
            });
            Value::Object(object)
        });
    }
    if let Some(SpellSourceValue::Known(damage)) = patch.damage.as_value() {
        system.insert("damage".into(), keyed_damage_patch(damage));
    }
    if let Some(SpellSourceValue::Known(duration)) = patch.duration.as_value() {
        let object = object_member(&mut system, "duration");
        insert_spell_fact(object, "value", &duration.value, |value| {
            Value::String(value.clone())
        });
        insert_spell_fact(object, "sustained", &duration.sustained, |value| {
            Value::Bool(*value)
        });
    }
    if let Some(SpellSourceValue::Known(heightening)) = patch.heightening.as_value() {
        let mut value = Map::new();
        insert_spell_fact(&mut value, "type", &heightening.kind, |_| {
            Value::String("interval".into())
        });
        insert_spell_fact(&mut value, "interval", &heightening.interval, |number| {
            Value::from(*number)
        });
        insert_spell_fact(&mut value, "area", &heightening.area, |number| {
            Value::from(*number)
        });
        if let Some(SpellSourceValue::Known(damage)) = heightening.interval_damage.as_value() {
            value.insert("damage".into(), keyed_text_patch(damage));
        }
        system.insert("heightening".into(), Value::Object(value));
    }
    if let Some(SpellSourceValue::Known(rules)) = patch.rules.as_value() {
        system.insert("rules".into(), rules_value(rules));
    }
    Value::Object(system)
}

fn keyed_damage_patch(patch: &SpellKeyedPatch<SpellDamagePatch>) -> Value {
    Value::Object(
        patch
            .members
            .iter()
            .map(|member| {
                let value = match &member.operation {
                    SpellKeyedPatchOperation::Merge(value) => {
                        let mut object = Map::new();
                        insert_spell_fact(&mut object, "applyMod", &value.apply_mod, |value| {
                            Value::Bool(*value)
                        });
                        insert_spell_fact(&mut object, "category", &value.category, |value| {
                            Value::String(value.clone())
                        });
                        insert_spell_fact(&mut object, "formula", &value.formula, |value| {
                            Value::String(value.clone())
                        });
                        insert_spell_fact(&mut object, "kinds", &value.kinds, |values| {
                            string_array(values)
                        });
                        insert_spell_fact(&mut object, "materials", &value.materials, |values| {
                            string_array(values)
                        });
                        insert_spell_fact(&mut object, "type", &value.damage_type, |value| {
                            Value::String(value.clone())
                        });
                        Value::Object(object)
                    }
                    SpellKeyedPatchOperation::Delete => Value::Null,
                    SpellKeyedPatchOperation::Unsupported(value) => unsupported_value(value),
                };
                (member.key.clone(), value)
            })
            .collect(),
    )
}

fn keyed_text_patch(patch: &SpellKeyedPatch<SpellTextPatch>) -> Value {
    Value::Object(
        patch
            .members
            .iter()
            .map(|member| {
                let value = match &member.operation {
                    SpellKeyedPatchOperation::Merge(value) => {
                        spell_fact_value(&value.value, |value| Value::String(value.clone()))
                            .unwrap_or(Value::Null)
                    }
                    SpellKeyedPatchOperation::Delete => Value::Null,
                    SpellKeyedPatchOperation::Unsupported(value) => unsupported_value(value),
                };
                (member.key.clone(), value)
            })
            .collect(),
    )
}

fn ritual_value(ritual: &SpellRitual) -> Value {
    let mut root = Map::new();
    insert_spell_fact(
        object_member(&mut root, "primary"),
        "check",
        &ritual.primary_check,
        |value| Value::String(value.clone()),
    );
    let secondary = object_member(&mut root, "secondary");
    insert_spell_fact(secondary, "casters", &ritual.secondary_casters, |value| {
        Value::from(*value)
    });
    insert_spell_fact(secondary, "checks", &ritual.secondary_checks, |value| {
        Value::String(value.clone())
    });
    Value::Object(root)
}

fn rules_value(rules: &[SpellRuleElement]) -> Value {
    Value::Array(
        rules
            .iter()
            .map(|rule| {
                serde_json::from_str(&rule.authored_object_json)
                    .unwrap_or_else(|_| Value::String(rule.authored_object_json.clone()))
            })
            .collect(),
    )
}

fn location_value(location: &ConsumableSpellLocation) -> Value {
    let mut object = Map::new();
    insert_spell_fact(&mut object, "value", &location.value, |value| {
        Value::String(value.clone())
    });
    insert_spell_fact(
        &mut object,
        "heightenedLevel",
        &location.heightened_rank,
        |value| Value::from(*value),
    );
    Value::Object(object)
}

fn insert_source_presence<T>(
    object: &mut Map<String, Value>,
    key: &str,
    fact: &crate::SourcePresence<T>,
    known: impl Fn(&T) -> Value,
) {
    match fact {
        crate::SourcePresence::Missing => {}
        crate::SourcePresence::Null => {
            object.insert(key.into(), Value::Null);
        }
        crate::SourcePresence::Value(value) => {
            object.insert(key.into(), known(value));
        }
    }
}

fn insert_spell_fact<T>(
    object: &mut Map<String, Value>,
    key: &str,
    fact: &SpellFact<T>,
    known: impl Fn(&T) -> Value,
) {
    if let Some(value) = spell_fact_value(fact, known) {
        object.insert(key.into(), value);
    }
}

fn spell_fact_value<T>(fact: &SpellFact<T>, known: impl Fn(&T) -> Value) -> Option<Value> {
    match fact {
        FactValue::Missing => None,
        FactValue::Null => Some(Value::Null),
        FactValue::Value(SpellSourceValue::Known(value)) => Some(known(value)),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => Some(unsupported_value(value)),
    }
}

fn unsupported_fact_value(fact: &FactValue<UnsupportedSourceValue>) -> Value {
    match fact {
        FactValue::Missing => Value::Null,
        FactValue::Null => Value::Null,
        FactValue::Value(value) => unsupported_value(value),
    }
}

fn unsupported_value(value: &UnsupportedSourceValue) -> Value {
    serde_json::from_str(&value.value).unwrap_or_else(|_| Value::String(value.value.clone()))
}

fn object_member<'a>(object: &'a mut Map<String, Value>, key: &str) -> &'a mut Map<String, Value> {
    object
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("spell receipt view owns object member")
}

fn string_array(values: &[String]) -> Value {
    Value::Array(values.iter().cloned().map(Value::String).collect())
}
