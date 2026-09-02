use std::collections::BTreeMap;

use serde_json::{Map, Value};

use super::{SourceDiagnostic, SourceDiagnosticKind, SourceIdentity, SourcePresence, actual_shape};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcCoreSource {
    pub details: SourcePresence<NpcDetailsSource>,
    pub traits: SourcePresence<NpcTraitsSource>,
    pub perception: SourcePresence<NpcPerceptionSource>,
    pub initiative: SourcePresence<NpcInitiativeSource>,
    pub abilities: SourcePresence<NpcAbilitiesSource>,
    pub attributes: SourcePresence<NpcAttributesSource>,
    pub saves: SourcePresence<NpcSavesSource>,
    pub skills: SourcePresence<BTreeMap<String, NpcSkillSource>>,
    pub resources: SourcePresence<BTreeMap<String, NpcResourceSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcDetailsSource {
    pub level: SourcePresence<NpcLevelSource>,
    pub languages: SourcePresence<NpcLanguagesSource>,
    pub publication: SourcePresence<NpcPublicationSource>,
    pub alliance: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcLevelSource {
    pub value: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcLanguagesSource {
    pub values: SourcePresence<Vec<String>>,
    pub details: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcPublicationSource {
    pub title: SourcePresence<String>,
    pub remaster: SourcePresence<bool>,
    pub license: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcTraitsSource {
    pub values: SourcePresence<Vec<String>>,
    pub rarity: SourcePresence<String>,
    pub size: SourcePresence<NpcSizeSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSizeSource {
    pub value: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcPerceptionSource {
    pub modifier: SourcePresence<i64>,
    pub legacy_value: SourcePresence<i64>,
    pub details: SourcePresence<String>,
    pub vision: SourcePresence<bool>,
    pub senses: SourcePresence<Vec<NpcSenseSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcInitiativeSource {
    pub statistic: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcAbilitiesSource {
    pub strength: SourcePresence<NpcLegacyAbilitySource>,
    pub dexterity: SourcePresence<NpcLegacyAbilitySource>,
    pub constitution: SourcePresence<NpcLegacyAbilitySource>,
    pub intelligence: SourcePresence<NpcLegacyAbilitySource>,
    pub wisdom: SourcePresence<NpcLegacyAbilitySource>,
    pub charisma: SourcePresence<NpcLegacyAbilitySource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcLegacyAbilitySource {
    pub r#mod: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSenseSource {
    pub sense_type: SourcePresence<String>,
    pub acuity: SourcePresence<String>,
    pub range: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcAttributesSource {
    pub adjustment: SourcePresence<String>,
    pub armor_class: SourcePresence<NpcArmorClassSource>,
    pub all_saves: SourcePresence<NpcAllSavesSource>,
    pub hit_points: SourcePresence<NpcHitPointsSource>,
    pub hardness: SourcePresence<NpcHardnessSource>,
    pub shield: SourcePresence<NpcShieldSource>,
    pub immunities: SourcePresence<Vec<NpcIwrSource>>,
    pub resistances: SourcePresence<Vec<NpcIwrSource>>,
    pub weaknesses: SourcePresence<Vec<NpcIwrSource>>,
    pub speed: SourcePresence<NpcSpeedSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcHardnessSource {
    pub value: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcShieldSource {
    pub armor_class_bonus: SourcePresence<i64>,
    pub broken_threshold: SourcePresence<i64>,
    pub hardness: SourcePresence<i64>,
    pub maximum_hit_points: SourcePresence<i64>,
    pub serialized_hit_points: SourcePresence<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcArmorClassSource {
    pub value: SourcePresence<i64>,
    pub details: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcAllSavesSource {
    pub value: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcHitPointsSource {
    pub value: SourcePresence<SourceInteger>,
    pub maximum: SourcePresence<i64>,
    pub temporary: SourcePresence<i64>,
    pub temporary_maximum: SourcePresence<i64>,
    pub details: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceInteger {
    Integer(i64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcIwrSource {
    pub iwr_type: SourcePresence<String>,
    pub value: SourcePresence<i64>,
    pub exceptions: SourcePresence<Vec<String>>,
    pub double_vs: SourcePresence<Vec<String>>,
    pub apply_once: SourcePresence<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSpeedSource {
    pub value: SourcePresence<i64>,
    pub details: SourcePresence<String>,
    pub other_speeds: SourcePresence<Vec<NpcOtherSpeedSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcOtherSpeedSource {
    pub speed_type: SourcePresence<String>,
    pub value: SourcePresence<i64>,
    pub label: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSavesSource {
    pub fortitude: SourcePresence<NpcSaveSource>,
    pub reflex: SourcePresence<NpcSaveSource>,
    pub will: SourcePresence<NpcSaveSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSaveSource {
    pub value: SourcePresence<i64>,
    pub details: SourcePresence<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSkillSource {
    pub base: SourcePresence<i64>,
    pub note: SourcePresence<String>,
    pub special: SourcePresence<Vec<NpcSkillVariantSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSkillVariantSource {
    pub base: SourcePresence<i64>,
    pub label: SourcePresence<String>,
    pub predicate: SourcePresence<Vec<NpcPredicateSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcPredicateSource {
    Term(String),
    Not(String),
    Any(Vec<String>),
    AtLeast { term: String, minimum: i64 },
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcResourceSource {
    pub maximum: SourcePresence<NpcResourceAmountSource>,
    pub maximum_drift: SourcePresence<i64>,
    pub value: SourcePresence<NpcResourceAmountSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcResourceAmountSource {
    Integer(i64),
    Text(String),
    Nested {
        maximum: SourcePresence<i64>,
        value: SourcePresence<i64>,
    },
    UnsupportedObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoreSource {
    pub modifier: SourcePresence<LoreModifierSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoreModifierSource {
    pub value: SourcePresence<i64>,
}

pub(super) fn parse_npc_core(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
) -> Result<NpcCoreSource, SourceDiagnostic> {
    Ok(NpcCoreSource {
        details: object_member(
            system,
            "details",
            identity,
            "$.system.details",
            parse_details,
        )?,
        traits: object_member(system, "traits", identity, "$.system.traits", parse_traits)?,
        perception: object_member(
            system,
            "perception",
            identity,
            "$.system.perception",
            parse_perception,
        )?,
        initiative: object_member(
            system,
            "initiative",
            identity,
            "$.system.initiative",
            parse_initiative,
        )?,
        abilities: object_member(
            system,
            "abilities",
            identity,
            "$.system.abilities",
            parse_abilities,
        )?,
        attributes: object_member(
            system,
            "attributes",
            identity,
            "$.system.attributes",
            parse_attributes,
        )?,
        saves: object_member(system, "saves", identity, "$.system.saves", parse_saves)?,
        skills: object_member(system, "skills", identity, "$.system.skills", parse_skills)?,
        resources: object_member(
            system,
            "resources",
            identity,
            "$.system.resources",
            parse_resources,
        )?,
    })
}

pub(super) fn parse_lore_source(
    system: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<LoreSource, SourceDiagnostic> {
    Ok(LoreSource {
        modifier: object_member(
            system,
            "mod",
            identity,
            &format!("{path}.mod"),
            |map, identity, path| {
                Ok(LoreModifierSource {
                    value: integer_member(map, "value", identity, &format!("{path}.value"))?,
                })
            },
        )?,
    })
}

fn parse_details(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcDetailsSource, SourceDiagnostic> {
    Ok(NpcDetailsSource {
        level: object_member(
            map,
            "level",
            identity,
            &format!("{path}.level"),
            |map, identity, path| {
                Ok(NpcLevelSource {
                    value: integer_member(map, "value", identity, &format!("{path}.value"))?,
                })
            },
        )?,
        languages: object_member(
            map,
            "languages",
            identity,
            &format!("{path}.languages"),
            |map, identity, path| {
                Ok(NpcLanguagesSource {
                    values: string_array_member(map, "value", identity, &format!("{path}.value"))?,
                    details: string_member(map, "details", identity, &format!("{path}.details"))?,
                })
            },
        )?,
        publication: object_member(
            map,
            "publication",
            identity,
            &format!("{path}.publication"),
            |map, identity, path| {
                Ok(NpcPublicationSource {
                    title: string_member(map, "title", identity, &format!("{path}.title"))?,
                    remaster: bool_member(map, "remaster", identity, &format!("{path}.remaster"))?,
                    license: string_member(map, "license", identity, &format!("{path}.license"))?,
                })
            },
        )?,
        alliance: string_member(map, "alliance", identity, &format!("{path}.alliance"))?,
    })
}

fn parse_traits(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcTraitsSource, SourceDiagnostic> {
    Ok(NpcTraitsSource {
        values: string_array_member(map, "value", identity, &format!("{path}.value"))?,
        rarity: string_member(map, "rarity", identity, &format!("{path}.rarity"))?,
        size: object_member(
            map,
            "size",
            identity,
            &format!("{path}.size"),
            |map, identity, path| {
                Ok(NpcSizeSource {
                    value: string_member(map, "value", identity, &format!("{path}.value"))?,
                })
            },
        )?,
    })
}

fn parse_perception(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcPerceptionSource, SourceDiagnostic> {
    Ok(NpcPerceptionSource {
        modifier: integer_member(map, "mod", identity, &format!("{path}.mod"))?,
        legacy_value: integer_member(map, "value", identity, &format!("{path}.value"))?,
        details: string_member(map, "details", identity, &format!("{path}.details"))?,
        vision: bool_member(map, "vision", identity, &format!("{path}.vision"))?,
        senses: object_array_member(
            map,
            "senses",
            identity,
            &format!("{path}.senses"),
            |map, identity, path| {
                Ok(NpcSenseSource {
                    sense_type: string_member(map, "type", identity, &format!("{path}.type"))?,
                    acuity: string_member(map, "acuity", identity, &format!("{path}.acuity"))?,
                    range: integer_member(map, "range", identity, &format!("{path}.range"))?,
                })
            },
        )?,
    })
}

fn parse_initiative(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcInitiativeSource, SourceDiagnostic> {
    Ok(NpcInitiativeSource {
        statistic: string_member(map, "statistic", identity, &format!("{path}.statistic"))?,
    })
}

fn parse_abilities(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcAbilitiesSource, SourceDiagnostic> {
    let parse = |map: &Map<String, Value>, identity: &SourceIdentity, path: &str| {
        Ok(NpcLegacyAbilitySource {
            r#mod: integer_member(map, "mod", identity, &format!("{path}.mod"))?,
        })
    };
    Ok(NpcAbilitiesSource {
        strength: object_member(map, "str", identity, &format!("{path}.str"), parse)?,
        dexterity: object_member(map, "dex", identity, &format!("{path}.dex"), parse)?,
        constitution: object_member(map, "con", identity, &format!("{path}.con"), parse)?,
        intelligence: object_member(map, "int", identity, &format!("{path}.int"), parse)?,
        wisdom: object_member(map, "wis", identity, &format!("{path}.wis"), parse)?,
        charisma: object_member(map, "cha", identity, &format!("{path}.cha"), parse)?,
    })
}

fn parse_attributes(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcAttributesSource, SourceDiagnostic> {
    Ok(NpcAttributesSource {
        adjustment: string_member(map, "adjustment", identity, &format!("{path}.adjustment"))?,
        armor_class: object_member(
            map,
            "ac",
            identity,
            &format!("{path}.ac"),
            |map, identity, path| {
                Ok(NpcArmorClassSource {
                    value: integer_member(map, "value", identity, &format!("{path}.value"))?,
                    details: string_member(map, "details", identity, &format!("{path}.details"))?,
                })
            },
        )?,
        all_saves: object_member(
            map,
            "allSaves",
            identity,
            &format!("{path}.allSaves"),
            |map, identity, path| {
                Ok(NpcAllSavesSource {
                    value: string_member(map, "value", identity, &format!("{path}.value"))?,
                })
            },
        )?,
        hit_points: object_member(map, "hp", identity, &format!("{path}.hp"), parse_hp)?,
        hardness: object_member(
            map,
            "hardness",
            identity,
            &format!("{path}.hardness"),
            |map, identity, path| {
                Ok(NpcHardnessSource {
                    value: integer_member(map, "value", identity, &format!("{path}.value"))?,
                })
            },
        )?,
        shield: object_member(
            map,
            "shield",
            identity,
            &format!("{path}.shield"),
            |map, identity, path| {
                Ok(NpcShieldSource {
                    armor_class_bonus: integer_member(map, "ac", identity, &format!("{path}.ac"))?,
                    broken_threshold: integer_member(
                        map,
                        "brokenThreshold",
                        identity,
                        &format!("{path}.brokenThreshold"),
                    )?,
                    hardness: integer_member(
                        map,
                        "hardness",
                        identity,
                        &format!("{path}.hardness"),
                    )?,
                    maximum_hit_points: integer_member(
                        map,
                        "max",
                        identity,
                        &format!("{path}.max"),
                    )?,
                    serialized_hit_points: integer_member(
                        map,
                        "value",
                        identity,
                        &format!("{path}.value"),
                    )?,
                })
            },
        )?,
        immunities: object_array_member(
            map,
            "immunities",
            identity,
            &format!("{path}.immunities"),
            parse_iwr,
        )?,
        resistances: object_array_member(
            map,
            "resistances",
            identity,
            &format!("{path}.resistances"),
            parse_iwr,
        )?,
        weaknesses: object_array_member(
            map,
            "weaknesses",
            identity,
            &format!("{path}.weaknesses"),
            parse_iwr,
        )?,
        speed: object_member(
            map,
            "speed",
            identity,
            &format!("{path}.speed"),
            parse_speed,
        )?,
    })
}

fn parse_hp(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcHitPointsSource, SourceDiagnostic> {
    Ok(NpcHitPointsSource {
        value: integer_or_text_member(map, "value", identity, &format!("{path}.value"))?,
        maximum: integer_member(map, "max", identity, &format!("{path}.max"))?,
        temporary: integer_member(map, "temp", identity, &format!("{path}.temp"))?,
        temporary_maximum: integer_member(map, "tempmax", identity, &format!("{path}.tempmax"))?,
        details: string_member(map, "details", identity, &format!("{path}.details"))?,
    })
}

fn parse_iwr(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcIwrSource, SourceDiagnostic> {
    Ok(NpcIwrSource {
        iwr_type: string_member(map, "type", identity, &format!("{path}.type"))?,
        value: integer_member(map, "value", identity, &format!("{path}.value"))?,
        exceptions: string_array_member(
            map,
            "exceptions",
            identity,
            &format!("{path}.exceptions"),
        )?,
        double_vs: string_array_member(map, "doubleVs", identity, &format!("{path}.doubleVs"))?,
        apply_once: bool_member(map, "applyOnce", identity, &format!("{path}.applyOnce"))?,
    })
}

fn parse_speed(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcSpeedSource, SourceDiagnostic> {
    Ok(NpcSpeedSource {
        value: integer_member(map, "value", identity, &format!("{path}.value"))?,
        details: string_member(map, "details", identity, &format!("{path}.details"))?,
        other_speeds: object_array_member(
            map,
            "otherSpeeds",
            identity,
            &format!("{path}.otherSpeeds"),
            |map, identity, path| {
                Ok(NpcOtherSpeedSource {
                    speed_type: string_member(map, "type", identity, &format!("{path}.type"))?,
                    value: integer_member(map, "value", identity, &format!("{path}.value"))?,
                    label: string_member(map, "label", identity, &format!("{path}.label"))?,
                })
            },
        )?,
    })
}

fn parse_saves(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcSavesSource, SourceDiagnostic> {
    let parse = |map: &Map<String, Value>, identity: &SourceIdentity, path: &str| {
        Ok(NpcSaveSource {
            value: integer_member(map, "value", identity, &format!("{path}.value"))?,
            details: string_member(map, "saveDetail", identity, &format!("{path}.saveDetail"))?,
        })
    };
    Ok(NpcSavesSource {
        fortitude: object_member(
            map,
            "fortitude",
            identity,
            &format!("{path}.fortitude"),
            parse,
        )?,
        reflex: object_member(map, "reflex", identity, &format!("{path}.reflex"), parse)?,
        will: object_member(map, "will", identity, &format!("{path}.will"), parse)?,
    })
}

fn parse_skills(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<BTreeMap<String, NpcSkillSource>, SourceDiagnostic> {
    map.iter()
        .map(|(slug, value)| {
            let skill_path = format!("{path}.{slug}");
            let skill = value.as_object().ok_or_else(|| {
                malformed(identity, &skill_path, "skill object", actual_shape(value))
            })?;
            Ok((
                slug.clone(),
                NpcSkillSource {
                    base: integer_member(skill, "base", identity, &format!("{skill_path}.base"))?,
                    note: string_member(skill, "note", identity, &format!("{skill_path}.note"))?,
                    special: object_array_member(
                        skill,
                        "special",
                        identity,
                        &format!("{skill_path}.special"),
                        parse_skill_variant,
                    )?,
                },
            ))
        })
        .collect()
}

fn parse_skill_variant(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<NpcSkillVariantSource, SourceDiagnostic> {
    Ok(NpcSkillVariantSource {
        base: integer_member(map, "base", identity, &format!("{path}.base"))?,
        label: string_member(map, "label", identity, &format!("{path}.label"))?,
        predicate: predicate_array_member(
            map,
            "predicate",
            identity,
            &format!("{path}.predicate"),
        )?,
    })
}

fn parse_resources(
    map: &Map<String, Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<BTreeMap<String, NpcResourceSource>, SourceDiagnostic> {
    map.iter()
        .map(|(key, value)| {
            let resource_path = format!("{path}.{key}");
            let resource = value.as_object().ok_or_else(|| {
                malformed(
                    identity,
                    &resource_path,
                    "resource object",
                    actual_shape(value),
                )
            })?;
            Ok((
                key.clone(),
                NpcResourceSource {
                    maximum: resource_amount_member(
                        resource,
                        "max",
                        identity,
                        &format!("{resource_path}.max"),
                    )?,
                    maximum_drift: integer_member(
                        resource,
                        "maxx",
                        identity,
                        &format!("{resource_path}.maxx"),
                    )?,
                    value: resource_amount_member(
                        resource,
                        "value",
                        identity,
                        &format!("{resource_path}.value"),
                    )?,
                },
            ))
        })
        .collect()
}

fn predicate_array_member(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<Vec<NpcPredicateSource>>, SourceDiagnostic> {
    array_member(map, key, identity, path, |value, identity, path| {
        if let Some(term) = value.as_str() {
            return Ok(NpcPredicateSource::Term(term.to_string()));
        }
        let Some(object) = value.as_object() else {
            return Ok(NpcPredicateSource::Unsupported(value.to_string()));
        };
        if let Some(term) = object.get("not").and_then(Value::as_str) {
            return Ok(NpcPredicateSource::Not(term.to_string()));
        }
        if let Some(terms) = object.get("or").and_then(Value::as_array)
            && let Some(terms) = terms
                .iter()
                .map(Value::as_str)
                .map(|term| term.map(str::to_string))
                .collect::<Option<Vec<_>>>()
        {
            return Ok(NpcPredicateSource::Any(terms));
        }
        if let Some(parts) = object.get("gte").and_then(Value::as_array)
            && parts.len() == 2
            && let (Some(term), Some(minimum)) = (parts[0].as_str(), parts[1].as_i64())
        {
            return Ok(NpcPredicateSource::AtLeast {
                term: term.to_string(),
                minimum,
            });
        }
        let _ = (identity, path);
        Ok(NpcPredicateSource::Unsupported(value.to_string()))
    })
}

fn resource_amount_member(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<NpcResourceAmountSource>, SourceDiagnostic> {
    presence_member(
        map,
        key,
        identity,
        path,
        "integer, string, or resource amount object",
        |value| {
            if let Some(value) = value.as_i64() {
                return Some(NpcResourceAmountSource::Integer(value));
            }
            if let Some(value) = value.as_str() {
                return Some(NpcResourceAmountSource::Text(value.to_string()));
            }
            value.as_object().map(|object| {
                let maximum = loose_integer_presence(object.get("max"));
                let value = loose_integer_presence(object.get("value"));
                if matches!(maximum, SourcePresence::Missing | SourcePresence::Null)
                    && matches!(value, SourcePresence::Missing | SourcePresence::Null)
                {
                    NpcResourceAmountSource::UnsupportedObject
                } else {
                    NpcResourceAmountSource::Nested { maximum, value }
                }
            })
        },
    )
}

fn loose_integer_presence(value: Option<&Value>) -> SourcePresence<i64> {
    match value {
        None => SourcePresence::Missing,
        Some(Value::Null) => SourcePresence::Null,
        Some(value) => value
            .as_i64()
            .map(SourcePresence::Value)
            .unwrap_or(SourcePresence::Missing),
    }
}

fn object_member<T>(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
    parse: impl FnOnce(&Map<String, Value>, &SourceIdentity, &str) -> Result<T, SourceDiagnostic>,
) -> Result<SourcePresence<T>, SourceDiagnostic> {
    match map.get(key) {
        None => Ok(SourcePresence::Missing),
        Some(Value::Null) => Ok(SourcePresence::Null),
        Some(Value::Object(value)) => parse(value, identity, path).map(SourcePresence::Value),
        Some(value) => Err(malformed(identity, path, "object", actual_shape(value))),
    }
}

fn object_array_member<T>(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
    parse: impl Fn(&Map<String, Value>, &SourceIdentity, &str) -> Result<T, SourceDiagnostic>,
) -> Result<SourcePresence<Vec<T>>, SourceDiagnostic> {
    array_member(map, key, identity, path, |value, identity, item_path| {
        let object = value
            .as_object()
            .ok_or_else(|| malformed(identity, item_path, "object", actual_shape(value)))?;
        parse(object, identity, item_path)
    })
}

fn array_member<T>(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
    parse: impl Fn(&Value, &SourceIdentity, &str) -> Result<T, SourceDiagnostic>,
) -> Result<SourcePresence<Vec<T>>, SourceDiagnostic> {
    match map.get(key) {
        None => Ok(SourcePresence::Missing),
        Some(Value::Null) => Ok(SourcePresence::Null),
        Some(Value::Array(values)) => values
            .iter()
            .enumerate()
            .map(|(index, value)| parse(value, identity, &format!("{path}[{index}]")))
            .collect::<Result<Vec<_>, _>>()
            .map(SourcePresence::Value),
        Some(value) => Err(malformed(identity, path, "array", actual_shape(value))),
    }
}

fn string_member(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<String>, SourceDiagnostic> {
    presence_member(map, key, identity, path, "string", |value| {
        value.as_str().map(str::to_string)
    })
}

fn bool_member(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<bool>, SourceDiagnostic> {
    presence_member(map, key, identity, path, "boolean", Value::as_bool)
}

fn integer_member(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<i64>, SourceDiagnostic> {
    presence_member(map, key, identity, path, "integer", Value::as_i64)
}

fn integer_or_text_member(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<SourceInteger>, SourceDiagnostic> {
    presence_member(map, key, identity, path, "integer or string", |value| {
        value.as_i64().map(SourceInteger::Integer).or_else(|| {
            value
                .as_str()
                .map(|value| SourceInteger::Text(value.to_string()))
        })
    })
}

fn string_array_member(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
) -> Result<SourcePresence<Vec<String>>, SourceDiagnostic> {
    array_member(map, key, identity, path, |value, identity, item_path| {
        value
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| malformed(identity, item_path, "string", actual_shape(value)))
    })
}

fn presence_member<T>(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    path: &str,
    expected: &str,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> Result<SourcePresence<T>, SourceDiagnostic> {
    match map.get(key) {
        None => Ok(SourcePresence::Missing),
        Some(Value::Null) => Ok(SourcePresence::Null),
        Some(value) => parse(value)
            .map(SourcePresence::Value)
            .ok_or_else(|| malformed(identity, path, expected, actual_shape(value))),
    }
}

fn malformed(
    identity: &SourceIdentity,
    path: &str,
    expected: &str,
    actual: &str,
) -> SourceDiagnostic {
    SourceDiagnostic::new(
        SourceDiagnosticKind::MalformedShape,
        identity,
        path,
        expected,
        actual,
    )
}
