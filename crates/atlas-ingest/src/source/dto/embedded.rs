use serde_json::{Map, Value};

use super::{
    ItemType, SourceDiagnostic, SourceDiagnosticKind, SourceIdentity, SourcePresence, actual_shape,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NpcEmbeddedItemSource {
    Action(ActionSource),
    Strike(StrikeSource),
    SpellcastingEntry(SpellcastingEntrySource),
    Spell(Box<SpellSource>),
    Equipment(EquipmentSource),
    Lore(LoreEntitySource),
    Deferred(DeferredEmbeddedSource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmbeddedCommonSource {
    pub(crate) slug: SourcePresence<String>,
    pub(crate) traits: SourcePresence<Vec<String>>,
    pub(crate) rules: SourcePresence<Vec<ValueSummary>>,
    pub(crate) local_unsupported: Vec<ValueSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActionSource {
    pub(crate) common: EmbeddedCommonSource,
    pub(crate) category: SourcePresence<String>,
    pub(crate) action_type: SourcePresence<String>,
    pub(crate) actions: SourcePresence<i64>,
    pub(crate) frequency: SourcePresence<FrequencySource>,
    pub(crate) self_effect: SourcePresence<String>,
    pub(crate) self_effect_label: SourcePresence<String>,
    pub(crate) requirements: SourcePresence<String>,
    pub(crate) cost: SourcePresence<String>,
    pub(crate) bonus: SourcePresence<i64>,
    pub(crate) dc: SourcePresence<i64>,
    pub(crate) damage: SourcePresence<Vec<DamageSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StrikeSource {
    pub(crate) common: EmbeddedCommonSource,
    pub(crate) bonus: SourcePresence<i64>,
    pub(crate) attack_effects: SourcePresence<Vec<String>>,
    pub(crate) damage: SourcePresence<Vec<DamageSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpellcastingEntrySource {
    pub(crate) common: EmbeddedCommonSource,
    pub(crate) preparation: SourcePresence<String>,
    pub(crate) tradition: SourcePresence<String>,
    pub(crate) attack: SourcePresence<i64>,
    pub(crate) dc: SourcePresence<i64>,
    pub(crate) slots: SourcePresence<Vec<SpellSlotSource>>,
    pub(crate) auto_heighten_level: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpellSource {
    pub(crate) common: EmbeddedCommonSource,
    pub(crate) level: SourcePresence<i64>,
    pub(crate) location: SourcePresence<String>,
    pub(crate) heightened_level: SourcePresence<i64>,
    pub(crate) signature: SourcePresence<bool>,
    pub(crate) traditions: SourcePresence<Vec<String>>,
    pub(crate) requirements: SourcePresence<String>,
    pub(crate) cost: SourcePresence<String>,
    pub(crate) counteraction: SourcePresence<bool>,
    pub(crate) ritual: SourcePresence<RitualSource>,
    pub(crate) uses: SourcePresence<UseLimitSource>,
    pub(crate) target: SourcePresence<String>,
    pub(crate) area: SourcePresence<SpellAreaSource>,
    pub(crate) range: SourcePresence<String>,
    pub(crate) time: SourcePresence<String>,
    pub(crate) duration: SourcePresence<SpellDurationSource>,
    pub(crate) defense: SourcePresence<SpellDefenseSource>,
    pub(crate) damage: SourcePresence<Vec<DamageSource>>,
    pub(crate) overlays: SourcePresence<Vec<ValueSummary>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EquipmentSource {
    pub(crate) common: EmbeddedCommonSource,
    pub(crate) level: SourcePresence<i64>,
    pub(crate) usage: SourcePresence<String>,
    pub(crate) quantity: SourcePresence<i64>,
    pub(crate) uses: SourcePresence<UseLimitSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoreEntitySource {
    pub(crate) common: EmbeddedCommonSource,
    pub(crate) modifier: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeferredEmbeddedSource {
    pub(crate) item_type: ItemType,
    pub(crate) common: EmbeddedCommonSource,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrequencySource {
    pub(crate) maximum: SourcePresence<i64>,
    pub(crate) period: SourcePresence<String>,
    pub(crate) value: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RitualSource {
    pub(crate) primary_check: SourcePresence<String>,
    pub(crate) secondary_casters: SourcePresence<EmbeddedSourceScalar<i64>>,
    pub(crate) secondary_checks: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UseLimitSource {
    pub(crate) maximum: SourcePresence<i64>,
    pub(crate) value: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DamageSource {
    pub(crate) id: String,
    pub(crate) formula: SourcePresence<String>,
    pub(crate) damage_type: SourcePresence<String>,
    pub(crate) category: SourcePresence<String>,
    pub(crate) kinds: SourcePresence<Vec<String>>,
    pub(crate) apply_modifier: SourcePresence<EmbeddedSourceScalar<bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpellSlotSource {
    pub(crate) rank: i64,
    pub(crate) maximum: SourcePresence<EmbeddedSourceScalar<i64>>,
    pub(crate) value: SourcePresence<EmbeddedSourceScalar<i64>>,
    pub(crate) prepared: SourcePresence<Vec<PreparedSlotSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreparedSlotSource {
    Spell {
        id: SourcePresence<String>,
        name: SourcePresence<String>,
        expended: SourcePresence<bool>,
        prepared: SourcePresence<bool>,
    },
    Unsupported(SourceTypeDrift),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActorSpellcastingSource {
    pub(crate) rituals_dc: SourcePresence<EmbeddedSourceScalar<i64>>,
    pub(crate) unsupported: Vec<ValueSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EmbeddedSourceScalar<T> {
    Value(T),
    Unsupported(SourceTypeDrift),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceTypeDrift {
    pub(crate) source_path: String,
    pub(crate) expected_shape: String,
    pub(crate) observed_shape: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpellAreaSource {
    pub(crate) area_type: SourcePresence<String>,
    pub(crate) value: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpellDurationSource {
    pub(crate) value: SourcePresence<String>,
    pub(crate) sustained: SourcePresence<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpellDefenseSource {
    pub(crate) statistic: SourcePresence<String>,
    pub(crate) basic: SourcePresence<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueSummary {
    pub(crate) source_path: String,
    pub(crate) shape: String,
    pub(crate) value: String,
}

pub(crate) fn parse_npc_embedded_item(
    item_type: ItemType,
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    json_path: &str,
) -> Result<NpcEmbeddedItemSource, SourceDiagnostic> {
    let common = common(system, identity, json_path)?;
    Ok(match item_type {
        ItemType::Action => NpcEmbeddedItemSource::Action(ActionSource {
            common,
            category: string_at(system, &["category"], identity, json_path)?,
            action_type: string_at(system, &["actionType", "value"], identity, json_path)?,
            actions: integer_at(system, &["actions", "value"], identity, json_path)?,
            frequency: frequency_at(system, identity, json_path)?,
            self_effect: string_at(system, &["selfEffect", "uuid"], identity, json_path)?,
            self_effect_label: string_at(system, &["selfEffect", "name"], identity, json_path)?,
            requirements: string_at(system, &["requirements"], identity, json_path)?,
            cost: string_at(system, &["cost", "value"], identity, json_path)?,
            bonus: integer_at(system, &["bonus", "value"], identity, json_path)?,
            dc: integer_at(system, &["dc", "value"], identity, json_path)?,
            damage: damage_at(system, &["damageRolls"], true, identity, json_path)
                .and_then(|damage| merge_damage(damage, system, identity, json_path))?,
        }),
        ItemType::Melee => NpcEmbeddedItemSource::Strike(StrikeSource {
            common,
            bonus: integer_at(system, &["bonus", "value"], identity, json_path)?,
            attack_effects: string_array_at(
                system,
                &["attackEffects", "value"],
                identity,
                json_path,
            )?,
            damage: damage_at(system, &["damageRolls"], true, identity, json_path)?,
        }),
        ItemType::SpellcastingEntry => {
            NpcEmbeddedItemSource::SpellcastingEntry(SpellcastingEntrySource {
                common,
                preparation: string_at(system, &["prepared", "value"], identity, json_path)?,
                tradition: string_at(system, &["tradition", "value"], identity, json_path)?,
                attack: integer_at(system, &["spelldc", "value"], identity, json_path)?,
                dc: integer_at(system, &["spelldc", "dc"], identity, json_path)?,
                slots: slots_at(system, identity, json_path)?,
                auto_heighten_level: integer_at(
                    system,
                    &["autoHeightenLevel", "value"],
                    identity,
                    json_path,
                )?,
            })
        }
        ItemType::Spell => NpcEmbeddedItemSource::Spell(Box::new(SpellSource {
            common,
            level: integer_at(system, &["level", "value"], identity, json_path)?,
            location: string_at(system, &["location", "value"], identity, json_path)?,
            heightened_level: integer_at(
                system,
                &["location", "heightenedLevel"],
                identity,
                json_path,
            )?,
            signature: boolean_at(system, &["location", "signature"], identity, json_path)?,
            traditions: first_present(
                string_array_at(system, &["traits", "traditions"], identity, json_path)?,
                string_array_at(
                    system,
                    &["spell", "system", "traits", "traditions"],
                    identity,
                    json_path,
                )?,
            ),
            requirements: string_at(system, &["requirements"], identity, json_path)?,
            cost: string_at(system, &["cost", "value"], identity, json_path)?,
            counteraction: first_present(
                boolean_at(system, &["counteraction"], identity, json_path)?,
                boolean_at(
                    system,
                    &["spell", "system", "counteraction"],
                    identity,
                    json_path,
                )?,
            ),
            ritual: ritual_at(system, identity, json_path)?,
            uses: uses_at(system, &["location", "uses"], identity, json_path)?,
            target: string_at(system, &["target", "value"], identity, json_path)?,
            area: area_at(system, identity, json_path)?,
            range: string_at(system, &["range", "value"], identity, json_path)?,
            time: string_at(system, &["time", "value"], identity, json_path)?,
            duration: duration_at(system, identity, json_path)?,
            defense: defense_at(system, identity, json_path)?,
            damage: damage_at(system, &["damage"], false, identity, json_path)?,
            overlays: summaries_at(system, &["overlays"], identity, json_path)?,
        })),
        ItemType::Equipment => NpcEmbeddedItemSource::Equipment(EquipmentSource {
            common,
            level: integer_at(system, &["level", "value"], identity, json_path)?,
            usage: string_at(system, &["usage", "value"], identity, json_path)?,
            quantity: integer_at(system, &["quantity"], identity, json_path)?,
            uses: uses_at(system, &["uses"], identity, json_path)?,
        }),
        ItemType::Lore => NpcEmbeddedItemSource::Lore(LoreEntitySource {
            common,
            modifier: integer_at(system, &["mod", "value"], identity, json_path)?,
        }),
        other => NpcEmbeddedItemSource::Deferred(DeferredEmbeddedSource {
            item_type: other,
            common,
            reason: "exact non-creature item family remains assigned to its reviewed H-family plan",
        }),
    })
}

fn common(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<EmbeddedCommonSource, SourceDiagnostic> {
    Ok(EmbeddedCommonSource {
        slug: string_at(system, &["slug"], identity, path)?,
        traits: string_array_at(system, &["traits", "value"], identity, path)?,
        rules: summaries_at(system, &["rules"], identity, path)?,
        local_unsupported: unsupported_local_facts(system, path),
    })
}

fn frequency_at(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<FrequencySource>, SourceDiagnostic> {
    object_at(
        system,
        &["frequency"],
        identity,
        path,
        |map, nested_path| {
            Ok(FrequencySource {
                maximum: integer_at(map, &["max"], identity, nested_path)?,
                period: string_at(map, &["per"], identity, nested_path)?,
                value: integer_at(map, &["value"], identity, nested_path)?,
            })
        },
    )
}

fn uses_at(
    system: &Map<String, Value>,
    segments: &[&str],
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<UseLimitSource>, SourceDiagnostic> {
    object_at(system, segments, identity, path, |map, nested_path| {
        Ok(UseLimitSource {
            maximum: integer_at(map, &["max"], identity, nested_path)?,
            value: integer_at(map, &["value"], identity, nested_path)?,
        })
    })
}

fn area_at(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<SpellAreaSource>, SourceDiagnostic> {
    object_at(system, &["area"], identity, path, |map, nested_path| {
        Ok(SpellAreaSource {
            area_type: string_at(map, &["type"], identity, nested_path)?,
            value: integer_at(map, &["value"], identity, nested_path)?,
        })
    })
}

fn duration_at(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<SpellDurationSource>, SourceDiagnostic> {
    object_at(system, &["duration"], identity, path, |map, nested_path| {
        Ok(SpellDurationSource {
            value: string_at(map, &["value"], identity, nested_path)?,
            sustained: boolean_at(map, &["sustained"], identity, nested_path)?,
        })
    })
}

fn defense_at(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<SpellDefenseSource>, SourceDiagnostic> {
    object_at(
        system,
        &["defense", "save"],
        identity,
        path,
        |map, nested_path| {
            Ok(SpellDefenseSource {
                statistic: string_at(map, &["statistic"], identity, nested_path)?,
                basic: boolean_at(map, &["basic"], identity, nested_path)?,
            })
        },
    )
}

fn slots_at(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<Vec<SpellSlotSource>>, SourceDiagnostic> {
    object_at(system, &["slots"], identity, path, |map, nested_path| {
        let mut slots = Vec::new();
        for (key, value) in map {
            let Some(rank) = key
                .strip_prefix("slot")
                .and_then(|value| value.parse().ok())
            else {
                continue;
            };
            let slot = value.as_object().ok_or_else(|| {
                malformed(identity, &format!("{nested_path}.{key}"), "object", value)
            })?;
            slots.push(SpellSlotSource {
                rank,
                maximum: preserving_scalar_at(
                    slot,
                    &["max"],
                    &format!("{nested_path}.{key}"),
                    "integer",
                    Value::as_i64,
                ),
                value: preserving_scalar_at(
                    slot,
                    &["value"],
                    &format!("{nested_path}.{key}"),
                    "integer",
                    Value::as_i64,
                ),
                prepared: prepared_slots_at(slot, &format!("{nested_path}.{key}"), identity)?,
            });
        }
        slots.sort_by_key(|slot| slot.rank);
        Ok(slots)
    })
}

fn prepared_slots_at(
    slot: &Map<String, Value>,
    path: &str,
    identity: &SourceIdentity,
) -> Result<SourcePresence<Vec<PreparedSlotSource>>, SourceDiagnostic> {
    let (presence, full_path) = value_at(slot, &["prepared"], path);
    match presence {
        SourcePresence::Missing => Ok(SourcePresence::Missing),
        SourcePresence::Null => Ok(SourcePresence::Null),
        SourcePresence::Value(value @ (Value::Array(_) | Value::Object(_))) => {
            let values = match value {
                Value::Array(values) => values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (index.to_string(), value))
                    .collect::<Vec<_>>(),
                Value::Object(values) => values
                    .iter()
                    .map(|(key, value)| (key.clone(), value))
                    .collect(),
                _ => Vec::new(),
            };
            Ok(SourcePresence::Value(
                values
                    .into_iter()
                    .map(|(index, value)| {
                        let entry_path = format!("{full_path}[{index}]");
                        let Some(map) = value.as_object() else {
                            return PreparedSlotSource::Unsupported(SourceTypeDrift {
                                source_path: entry_path,
                                expected_shape: "prepared slot object".to_string(),
                                observed_shape: actual_shape(value).to_string(),
                                value: value.to_string(),
                            });
                        };
                        PreparedSlotSource::Spell {
                            id: string_at(map, &["id"], identity, &entry_path)
                                .unwrap_or(SourcePresence::Missing),
                            name: string_at(map, &["name"], identity, &entry_path)
                                .unwrap_or(SourcePresence::Missing),
                            expended: boolean_at(map, &["expended"], identity, &entry_path)
                                .unwrap_or(SourcePresence::Missing),
                            prepared: boolean_at(map, &["prepared"], identity, &entry_path)
                                .unwrap_or(SourcePresence::Missing),
                        }
                    })
                    .collect(),
            ))
        }
        SourcePresence::Value(value) => Err(malformed(
            identity,
            &full_path,
            "array of prepared spell slots",
            value,
        )),
    }
}

fn ritual_at(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<RitualSource>, SourceDiagnostic> {
    object_at(
        system,
        &["ritual"],
        identity,
        path,
        |ritual, ritual_path| {
            Ok(RitualSource {
                primary_check: string_at(ritual, &["primary", "check"], identity, ritual_path)?,
                secondary_casters: preserving_scalar_at(
                    ritual,
                    &["secondary", "casters"],
                    ritual_path,
                    "integer",
                    Value::as_i64,
                ),
                secondary_checks: string_at(
                    ritual,
                    &["secondary", "checks"],
                    identity,
                    ritual_path,
                )?,
            })
        },
    )
}

pub(crate) fn parse_actor_spellcasting(
    system: &Map<String, Value>,
) -> SourcePresence<ActorSpellcastingSource> {
    let (presence, path) = value_at(system, &["spellcasting"], "$.system");
    match presence {
        SourcePresence::Missing => SourcePresence::Missing,
        SourcePresence::Null => SourcePresence::Null,
        SourcePresence::Value(value) => {
            let Some(map) = value.as_object() else {
                return SourcePresence::Value(ActorSpellcastingSource {
                    rituals_dc: SourcePresence::Value(EmbeddedSourceScalar::Unsupported(
                        SourceTypeDrift {
                            source_path: path,
                            expected_shape: "spellcasting object".to_string(),
                            observed_shape: actual_shape(value).to_string(),
                            value: value.to_string(),
                        },
                    )),
                    unsupported: Vec::new(),
                });
            };
            SourcePresence::Value(ActorSpellcastingSource {
                rituals_dc: preserving_scalar_at(
                    map,
                    &["rituals", "dc"],
                    &path,
                    "integer",
                    Value::as_i64,
                ),
                unsupported: unsupported_local_facts(map, &path)
                    .into_iter()
                    .filter(|fact| fact.source_path != "$.system.spellcasting.rituals.dc")
                    .collect(),
            })
        }
    }
}

fn unsupported_local_facts(system: &Map<String, Value>, path: &str) -> Vec<ValueSummary> {
    let mut facts = Vec::new();
    for (key, value) in system {
        if matches!(key.as_str(), "description" | "slug" | "rules") {
            continue;
        }
        flatten_fact(value, format!("{path}.{key}"), &mut facts);
    }
    facts
}

fn flatten_fact(value: &Value, source_path: String, facts: &mut Vec<ValueSummary>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, value) in map {
                flatten_fact(value, format!("{source_path}.{key}"), facts);
            }
        }
        Value::Array(values) if !values.is_empty() => {
            for value in values {
                flatten_fact(value, format!("{source_path}[]"), facts);
            }
        }
        _ => facts.push(ValueSummary {
            source_path,
            shape: actual_shape(value).to_string(),
            value: value.to_string(),
        }),
    }
}

fn damage_at(
    system: &Map<String, Value>,
    segments: &[&str],
    strike_shape: bool,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<Vec<DamageSource>>, SourceDiagnostic> {
    object_at(system, segments, identity, path, |map, nested_path| {
        let mut damage = Vec::new();
        for (id, value) in map {
            let entry = value.as_object().ok_or_else(|| {
                malformed(identity, &format!("{nested_path}.{id}"), "object", value)
            })?;
            damage.push(DamageSource {
                id: id.clone(),
                formula: string_at(
                    entry,
                    &[if strike_shape { "damage" } else { "formula" }],
                    identity,
                    &format!("{nested_path}.{id}"),
                )?,
                damage_type: string_at(
                    entry,
                    &[if strike_shape { "damageType" } else { "type" }],
                    identity,
                    &format!("{nested_path}.{id}"),
                )?,
                category: string_at(
                    entry,
                    &["category"],
                    identity,
                    &format!("{nested_path}.{id}"),
                )?,
                kinds: string_array_at(
                    entry,
                    &["kinds"],
                    identity,
                    &format!("{nested_path}.{id}"),
                )?,
                apply_modifier: preserving_scalar_at(
                    entry,
                    &["applyMod"],
                    &format!("{nested_path}.{id}"),
                    "boolean",
                    Value::as_bool,
                ),
            });
        }
        damage.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(damage)
    })
}

fn merge_damage(
    first: SourcePresence<Vec<DamageSource>>,
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<Vec<DamageSource>>, SourceDiagnostic> {
    let second = damage_at(system, &["damage"], false, identity, path)?;
    Ok(match (first, second) {
        (SourcePresence::Value(mut left), SourcePresence::Value(right)) => {
            left.extend(right);
            SourcePresence::Value(left)
        }
        (SourcePresence::Value(left), _) => SourcePresence::Value(left),
        (_, SourcePresence::Value(right)) => SourcePresence::Value(right),
        (SourcePresence::Null, _) | (_, SourcePresence::Null) => SourcePresence::Null,
        _ => SourcePresence::Missing,
    })
}

fn first_present<T>(first: SourcePresence<T>, second: SourcePresence<T>) -> SourcePresence<T> {
    match first {
        SourcePresence::Missing => second,
        value => value,
    }
}

fn summaries_at(
    root: &Map<String, Value>,
    segments: &[&str],
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<Vec<ValueSummary>>, SourceDiagnostic> {
    presence_at(
        root,
        segments,
        identity,
        path,
        "array or object",
        |value, full_path| {
            let values = match value {
                Value::Array(values) => values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("{full_path}[{index}]"), value))
                    .collect::<Vec<_>>(),
                Value::Object(values) => values
                    .iter()
                    .map(|(key, value)| (format!("{full_path}.{key}"), value))
                    .collect::<Vec<_>>(),
                _ => return None,
            };
            Some(
                values
                    .into_iter()
                    .map(|(source_path, value)| ValueSummary {
                        source_path,
                        shape: actual_shape(value).to_string(),
                        value: value.to_string(),
                    })
                    .collect(),
            )
        },
    )
}

fn string_at(
    root: &Map<String, Value>,
    segments: &[&str],
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<String>, SourceDiagnostic> {
    presence_at(root, segments, identity, path, "string", |value, _| {
        value.as_str().map(str::to_string)
    })
}

fn integer_at(
    root: &Map<String, Value>,
    segments: &[&str],
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<i64>, SourceDiagnostic> {
    presence_at(root, segments, identity, path, "integer", |value, _| {
        value.as_i64()
    })
}

fn boolean_at(
    root: &Map<String, Value>,
    segments: &[&str],
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<bool>, SourceDiagnostic> {
    presence_at(root, segments, identity, path, "boolean", |value, _| {
        value.as_bool()
    })
}

fn preserving_scalar_at<T>(
    root: &Map<String, Value>,
    segments: &[&str],
    path: &str,
    expected: &str,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> SourcePresence<EmbeddedSourceScalar<T>> {
    let (presence, full_path) = value_at(root, segments, path);
    match presence {
        SourcePresence::Missing => SourcePresence::Missing,
        SourcePresence::Null => SourcePresence::Null,
        SourcePresence::Value(value) => SourcePresence::Value(match parse(value) {
            Some(value) => EmbeddedSourceScalar::Value(value),
            None => EmbeddedSourceScalar::Unsupported(SourceTypeDrift {
                source_path: full_path,
                expected_shape: expected.to_string(),
                observed_shape: actual_shape(value).to_string(),
                value: value.to_string(),
            }),
        }),
    }
}

fn string_array_at(
    root: &Map<String, Value>,
    segments: &[&str],
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<Vec<String>>, SourceDiagnostic> {
    presence_at(
        root,
        segments,
        identity,
        path,
        "array of strings",
        |value, _| {
            value
                .as_array()?
                .iter()
                .map(|value| value.as_str().map(str::to_string))
                .collect()
        },
    )
}

fn object_at<T>(
    root: &Map<String, Value>,
    segments: &[&str],
    identity: &SourceIdentity,
    path: &str,
    parse: impl FnOnce(&Map<String, Value>, &str) -> Result<T, SourceDiagnostic>,
) -> Result<SourcePresence<T>, SourceDiagnostic> {
    let (presence, full_path) = value_at(root, segments, path);
    match presence {
        SourcePresence::Missing => Ok(SourcePresence::Missing),
        SourcePresence::Null => Ok(SourcePresence::Null),
        SourcePresence::Value(value) => value
            .as_object()
            .ok_or_else(|| malformed(identity, &full_path, "object", value))
            .and_then(|value| parse(value, &full_path))
            .map(SourcePresence::Value),
    }
}

fn presence_at<T>(
    root: &Map<String, Value>,
    segments: &[&str],
    identity: &SourceIdentity,
    path: &str,
    expected: &str,
    parse: impl FnOnce(&Value, &str) -> Option<T>,
) -> Result<SourcePresence<T>, SourceDiagnostic> {
    let (presence, full_path) = value_at(root, segments, path);
    match presence {
        SourcePresence::Missing => Ok(SourcePresence::Missing),
        SourcePresence::Null => Ok(SourcePresence::Null),
        SourcePresence::Value(value) => parse(value, &full_path)
            .map(SourcePresence::Value)
            .ok_or_else(|| malformed(identity, &full_path, expected, value)),
    }
}

fn value_at<'a>(
    root: &'a Map<String, Value>,
    segments: &[&str],
    path: &str,
) -> (SourcePresence<&'a Value>, String) {
    let mut current = root;
    let mut full_path = path.to_string();
    for (index, segment) in segments.iter().enumerate() {
        full_path.push('.');
        full_path.push_str(segment);
        let Some(value) = current.get(*segment) else {
            return (SourcePresence::Missing, full_path);
        };
        if value.is_null() {
            return (SourcePresence::Null, full_path);
        }
        if index == segments.len() - 1 {
            return (SourcePresence::Value(value), full_path);
        }
        let Some(map) = value.as_object() else {
            return (SourcePresence::Value(value), full_path);
        };
        current = map;
    }
    (SourcePresence::Missing, full_path)
}

fn malformed(
    identity: &SourceIdentity,
    path: &str,
    expected: &str,
    value: &Value,
) -> SourceDiagnostic {
    SourceDiagnostic::new(
        SourceDiagnosticKind::MalformedShape,
        identity,
        path,
        expected,
        actual_shape(value),
    )
}
