use serde_json::Value;

use super::{
    RawSourceJson, SerializedSourceMember, SerializedSourceObject, SerializedSourceValue,
    SourceDiagnostic, SourceDiagnosticKind, SourceIdentity, SourcePresence, SourceVersionMetadata,
    ValueSummary, actual_shape,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HazardSourceValue<T> {
    Typed(T),
    Unsupported(ValueSummary),
}

pub(crate) type HazardSourceField<T> = SourcePresence<HazardSourceValue<T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardSource {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) actor_type: String,
    pub(crate) image: HazardSourceField<String>,
    pub(crate) folder: HazardSourceField<String>,
    pub(crate) traits: HazardSourceField<HazardTraitsSource>,
    pub(crate) level: HazardSourceField<i64>,
    pub(crate) publication: HazardSourceField<HazardPublicationSource>,
    pub(crate) complexity: HazardSourceField<bool>,
    pub(crate) detection: HazardSourceField<HazardDetectionSource>,
    pub(crate) defenses: HazardSourceField<HazardDefensesSource>,
    pub(crate) lifecycle: HazardSourceField<HazardLifecycleSource>,
    pub(crate) emits_sound: HazardSourceField<HazardEmitsSoundSource>,
    pub(crate) creature_type: HazardSourceField<String>,
    pub(crate) status_effects: HazardSourceField<Vec<String>>,
    pub(crate) items: HazardSourceField<Vec<HazardItemSource>>,
    pub(crate) effects: HazardSourceField<Vec<ValueSummary>>,
    pub(crate) prototype_token: HazardSourceField<HazardTokenSourceMetadata>,
    pub(crate) unclaimed: Vec<ValueSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardTokenSourceMetadata {
    pub(crate) name: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardTraitsSource {
    pub(crate) values: HazardSourceField<Vec<String>>,
    pub(crate) rarity: HazardSourceField<String>,
    pub(crate) size: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardPublicationSource {
    pub(crate) title: HazardSourceField<String>,
    pub(crate) remaster: HazardSourceField<bool>,
    pub(crate) license: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardDetectionSource {
    pub(crate) value: HazardSourceField<i64>,
    pub(crate) details: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardDefensesSource {
    pub(crate) armor_class: HazardSourceField<i64>,
    pub(crate) hardness: HazardSourceField<i64>,
    pub(crate) hit_points: HazardSourceField<HazardHitPointsSource>,
    pub(crate) saves: HazardSourceField<HazardSavesSource>,
    pub(crate) immunities: HazardSourceField<Vec<HazardIwrSource>>,
    pub(crate) weaknesses: HazardSourceField<Vec<HazardIwrSource>>,
    pub(crate) resistances: HazardSourceField<Vec<HazardIwrSource>>,
    pub(crate) has_health: HazardSourceField<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardHitPointsSource {
    pub(crate) current: HazardSourceField<i64>,
    pub(crate) maximum: HazardSourceField<i64>,
    pub(crate) temporary: HazardSourceField<i64>,
    pub(crate) details: HazardSourceField<String>,
    pub(crate) temporary_maximum: HazardSourceField<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardSavesSource {
    pub(crate) fortitude: HazardSaveSource,
    pub(crate) reflex: HazardSaveSource,
    pub(crate) will: HazardSaveSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardSaveSource {
    pub(crate) value: HazardSourceField<i64>,
    pub(crate) detail: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardIwrSource {
    pub(crate) source_ordinal: u32,
    pub(crate) iwr_type: HazardSourceField<String>,
    pub(crate) value: HazardSourceField<i64>,
    pub(crate) exceptions: HazardSourceField<Vec<String>>,
    pub(crate) double_vs: HazardSourceField<Vec<String>>,
    pub(crate) unclaimed: Vec<ValueSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardLifecycleSource {
    pub(crate) description: HazardSourceField<String>,
    pub(crate) disable: HazardSourceField<String>,
    pub(crate) routine: HazardSourceField<String>,
    pub(crate) reset: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HazardEmitsSoundSource {
    Boolean(bool),
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardItemSource {
    pub(crate) source_ordinal: u32,
    pub(crate) id: HazardSourceField<String>,
    pub(crate) name: HazardSourceField<String>,
    pub(crate) item_type: HazardSourceField<String>,
    pub(crate) image: HazardSourceField<String>,
    pub(crate) folder: HazardSourceField<String>,
    pub(crate) sort: HazardSourceField<i64>,
    pub(crate) common: HazardItemCommonSource,
    pub(crate) action: HazardActionSource,
    pub(crate) strike: HazardStrikeSource,
    pub(crate) unclaimed: Vec<ValueSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardItemCommonSource {
    pub(crate) description: HazardSourceField<String>,
    pub(crate) publication: HazardSourceField<HazardPublicationSource>,
    pub(crate) rules: HazardSourceField<Vec<ValueSummary>>,
    pub(crate) slug: HazardSourceField<String>,
    pub(crate) traits: HazardSourceField<Vec<String>>,
    pub(crate) rarity: HazardSourceField<String>,
    pub(crate) lineage: HazardSourceField<HazardItemLineageSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardItemLineageSource {
    pub(crate) compendium_source: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardActionSource {
    pub(crate) action_type: HazardSourceField<String>,
    pub(crate) actions: HazardSourceField<i64>,
    pub(crate) category: HazardSourceField<String>,
    pub(crate) death_note: HazardSourceField<bool>,
    pub(crate) frequency: HazardSourceField<HazardFrequencySource>,
    pub(crate) self_effect: HazardSourceField<HazardSelfEffectSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardFrequencySource {
    pub(crate) value: HazardSourceField<i64>,
    pub(crate) maximum: HazardSourceField<i64>,
    pub(crate) per: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardSelfEffectSource {
    pub(crate) uuid: HazardSourceField<String>,
    pub(crate) name: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardStrikeSource {
    pub(crate) bonus: HazardSourceField<i64>,
    pub(crate) attack_effects: HazardSourceField<Vec<String>>,
    pub(crate) damage_rolls: HazardSourceField<Vec<HazardDamageSource>>,
    pub(crate) attack: HazardSourceField<i64>,
    pub(crate) weapon_type: HazardSourceField<String>,
    pub(crate) attack_effects_custom: HazardSourceField<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardDamageSource {
    pub(crate) source_path: String,
    pub(crate) source_key: String,
    pub(crate) authored_order: u32,
    pub(crate) damage: HazardSourceField<String>,
    pub(crate) damage_type: HazardSourceField<String>,
    pub(crate) category: HazardSourceField<String>,
    pub(crate) unclaimed: Vec<ValueSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionedHazardSource {
    pub(crate) version: SourceVersionMetadata,
    pub(crate) source: HazardSource,
    provenance: RawSourceJson,
}

impl VersionedHazardSource {
    #[cfg(test)]
    pub(crate) fn raw_json_for_audit(&self) -> &Value {
        self.provenance.for_audit()
    }
}

pub(crate) fn parse_hazard_source(
    version: SourceVersionMetadata,
    identity: SourceIdentity,
    raw: Value,
    serialized: &SerializedSourceObject,
) -> Result<VersionedHazardSource, SourceDiagnostic> {
    let map = raw.as_object().ok_or_else(|| {
        SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            &identity,
            "$",
            "Actor source object",
            actual_shape(&raw),
        )
    })?;
    let id = required_root_string(map.get("_id"), &identity, "$._id")?;
    let name = required_root_string(map.get("name"), &identity, "$.name")?;
    let actor_type = required_root_string(map.get("type"), &identity, "$.type")?;
    if actor_type != "hazard" {
        return Err(SourceDiagnostic::new(
            SourceDiagnosticKind::InvalidParentContext,
            &identity,
            "$.type",
            "Actor discriminator hazard",
            format!("Actor discriminator {actor_type:?}"),
        ));
    }

    let source = HazardSource {
        id,
        name,
        actor_type,
        image: string_field(&raw, "/img"),
        folder: string_field(&raw, "/folder"),
        traits: object_field(&raw, "/system/traits", |value| HazardTraitsSource {
            values: string_array_field(value, "/value", "/system/traits/value"),
            rarity: string_field_with_path(value, "/rarity", "/system/traits/rarity"),
            size: string_field_with_path(value, "/size/value", "/system/traits/size/value"),
        }),
        level: integer_field(&raw, "/system/details/level/value"),
        publication: publication_field(&raw, "/system/details/publication"),
        complexity: boolean_field(&raw, "/system/details/isComplex"),
        detection: object_field(&raw, "/system/attributes/stealth", |value| {
            HazardDetectionSource {
                value: integer_field_with_path(value, "/value", "/system/attributes/stealth/value"),
                details: string_field_with_path(
                    value,
                    "/details",
                    "/system/attributes/stealth/details",
                ),
            }
        }),
        defenses: defenses_field(&raw),
        lifecycle: object_field(&raw, "/system/details", |value| HazardLifecycleSource {
            description: string_field_with_path(
                value,
                "/description",
                "/system/details/description",
            ),
            disable: string_field_with_path(value, "/disable", "/system/details/disable"),
            routine: string_field_with_path(value, "/routine", "/system/details/routine"),
            reset: string_field_with_path(value, "/reset", "/system/details/reset"),
        }),
        emits_sound: field(
            &raw,
            "/system/attributes/emitsSound",
            "string or boolean",
            |value| {
                value
                    .as_bool()
                    .map(HazardEmitsSoundSource::Boolean)
                    .or_else(|| {
                        value
                            .as_str()
                            .map(|value| HazardEmitsSoundSource::Named(value.to_string()))
                    })
            },
        ),
        creature_type: string_field(&raw, "/system/creatureType"),
        status_effects: string_array_field(&raw, "/system/statusEffects", "/system/statusEffects"),
        items: items_field(&raw, serialized, &identity)?,
        effects: summaries_field(&raw, "/effects"),
        prototype_token: object_field(&raw, "/prototypeToken", |token| HazardTokenSourceMetadata {
            name: string_field_with_path(token, "/name", "/prototypeToken/name"),
        }),
        unclaimed: collect_unclaimed_root(&raw),
    };

    Ok(VersionedHazardSource {
        version,
        source,
        provenance: RawSourceJson::new(raw),
    })
}

fn defenses_field(raw: &Value) -> HazardSourceField<HazardDefensesSource> {
    object_field(raw, "/system/attributes", |attributes| {
        HazardDefensesSource {
            armor_class: integer_field_with_path(
                attributes,
                "/ac/value",
                "/system/attributes/ac/value",
            ),
            hardness: integer_field_with_path(
                attributes,
                "/hardness",
                "/system/attributes/hardness",
            ),
            hit_points: object_field_with_path(attributes, "/hp", "/system/attributes/hp", |hp| {
                HazardHitPointsSource {
                    current: integer_field_with_path(hp, "/value", "/system/attributes/hp/value"),
                    maximum: integer_field_with_path(hp, "/max", "/system/attributes/hp/max"),
                    temporary: integer_field_with_path(hp, "/temp", "/system/attributes/hp/temp"),
                    details: string_field_with_path(
                        hp,
                        "/details",
                        "/system/attributes/hp/details",
                    ),
                    temporary_maximum: integer_field_with_path(
                        hp,
                        "/tempmax",
                        "/system/attributes/hp/tempmax",
                    ),
                }
            }),
            saves: object_field_with_path(raw, "/system/saves", "/system/saves", |saves| {
                HazardSavesSource {
                    fortitude: save_field(saves, "fortitude"),
                    reflex: save_field(saves, "reflex"),
                    will: save_field(saves, "will"),
                }
            }),
            immunities: iwr_field(attributes, "/immunities", "/system/attributes/immunities"),
            weaknesses: iwr_field(attributes, "/weaknesses", "/system/attributes/weaknesses"),
            resistances: iwr_field(attributes, "/resistances", "/system/attributes/resistances"),
            has_health: boolean_field_with_path(
                attributes,
                "/hasHealth",
                "/system/attributes/hasHealth",
            ),
        }
    })
}

fn save_field(saves: &Value, kind: &str) -> HazardSaveSource {
    let base = format!("/system/saves/{kind}");
    HazardSaveSource {
        value: integer_field_with_path(saves, &format!("/{kind}/value"), &format!("{base}/value")),
        detail: string_field_with_path(
            saves,
            &format!("/{kind}/saveDetail"),
            &format!("{base}/saveDetail"),
        ),
    }
}

fn iwr_field(
    value: &Value,
    local_path: &str,
    full_path: &str,
) -> HazardSourceField<Vec<HazardIwrSource>> {
    field_with_path(value, local_path, full_path, "array", |value| {
        let entries = value.as_array()?;
        Some(
            entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let base = format!("{full_path}/{index}");
                    HazardIwrSource {
                        source_ordinal: index as u32,
                        iwr_type: string_field_with_path(entry, "/type", &format!("{base}/type")),
                        value: integer_field_with_path(entry, "/value", &format!("{base}/value")),
                        exceptions: string_array_field(
                            entry,
                            "/exceptions",
                            &format!("{base}/exceptions"),
                        ),
                        double_vs: string_array_field(
                            entry,
                            "/doubleVs",
                            &format!("{base}/doubleVs"),
                        ),
                        unclaimed: collect_unclaimed(
                            entry,
                            &base,
                            &["/type", "/value", "/exceptions", "/doubleVs"],
                        ),
                    }
                })
                .collect(),
        )
    })
}

fn publication_field(value: &Value, path: &str) -> HazardSourceField<HazardPublicationSource> {
    object_field(value, path, |publication| HazardPublicationSource {
        title: string_field_with_path(publication, "/title", &format!("{path}/title")),
        remaster: boolean_field_with_path(publication, "/remaster", &format!("{path}/remaster")),
        license: string_field_with_path(publication, "/license", &format!("{path}/license")),
    })
}

fn items_field(
    raw: &Value,
    serialized: &SerializedSourceObject,
    identity: &SourceIdentity,
) -> Result<HazardSourceField<Vec<HazardItemSource>>, SourceDiagnostic> {
    let serialized_items = match serialized.member("items") {
        SerializedSourceMember::Value(SerializedSourceValue::Array(items)) => {
            Some(items.as_slice())
        }
        _ => None,
    };
    let Some(value) = raw.pointer("/items") else {
        return Ok(SourcePresence::Missing);
    };
    if value.is_null() {
        return Ok(SourcePresence::Null);
    }
    let Some(items) = value.as_array() else {
        return Ok(SourcePresence::Value(HazardSourceValue::Unsupported(
            ValueSummary {
                source_path: "/items".to_string(),
                shape: format!("expected array; observed {}", actual_shape(value)),
                value: exact_json(value),
            },
        )));
    };
    let items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            parse_item(
                index,
                item,
                serialized_items.and_then(|items| items.get(index)),
                identity,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SourcePresence::Value(HazardSourceValue::Typed(items)))
}

fn parse_item(
    source_ordinal: usize,
    item: &Value,
    serialized: Option<&SerializedSourceValue>,
    identity: &SourceIdentity,
) -> Result<HazardItemSource, SourceDiagnostic> {
    let base = format!("/items/{source_ordinal}");
    let item_type = item.pointer("/type").and_then(Value::as_str);
    let common = HazardItemCommonSource {
        description: string_field_with_path(
            item,
            "/system/description/value",
            &format!("{base}/system/description/value"),
        ),
        publication: publication_field(item, "/system/publication"),
        rules: summaries_field_with_path(item, "/system/rules", &format!("{base}/system/rules")),
        slug: string_field_with_path(item, "/system/slug", &format!("{base}/system/slug")),
        traits: string_array_field(
            item,
            "/system/traits/value",
            &format!("{base}/system/traits/value"),
        ),
        rarity: string_field_with_path(
            item,
            "/system/traits/rarity",
            &format!("{base}/system/traits/rarity"),
        ),
        lineage: object_field_with_path(item, "/_stats", &format!("{base}/_stats"), |stats| {
            HazardItemLineageSource {
                compendium_source: string_field_with_path(
                    stats,
                    "/compendiumSource",
                    &format!("{base}/_stats/compendiumSource"),
                ),
            }
        }),
    };
    let action = HazardActionSource {
        action_type: string_field_with_path(
            item,
            "/system/actionType/value",
            &format!("{base}/system/actionType/value"),
        ),
        actions: integer_field_with_path(
            item,
            "/system/actions/value",
            &format!("{base}/system/actions/value"),
        ),
        category: string_field_with_path(
            item,
            "/system/category",
            &format!("{base}/system/category"),
        ),
        death_note: boolean_field_with_path(
            item,
            "/system/deathNote",
            &format!("{base}/system/deathNote"),
        ),
        frequency: object_field_with_path(
            item,
            "/system/frequency",
            &format!("{base}/system/frequency"),
            |frequency| HazardFrequencySource {
                value: integer_field_with_path(
                    frequency,
                    "/value",
                    &format!("{base}/system/frequency/value"),
                ),
                maximum: integer_field_with_path(
                    frequency,
                    "/max",
                    &format!("{base}/system/frequency/max"),
                ),
                per: string_field_with_path(
                    frequency,
                    "/per",
                    &format!("{base}/system/frequency/per"),
                ),
            },
        ),
        self_effect: object_field_with_path(
            item,
            "/system/selfEffect",
            &format!("{base}/system/selfEffect"),
            |self_effect| HazardSelfEffectSource {
                uuid: string_field_with_path(
                    self_effect,
                    "/uuid",
                    &format!("{base}/system/selfEffect/uuid"),
                ),
                name: string_field_with_path(
                    self_effect,
                    "/name",
                    &format!("{base}/system/selfEffect/name"),
                ),
            },
        ),
    };
    let strike = HazardStrikeSource {
        bonus: integer_field_with_path(
            item,
            "/system/bonus/value",
            &format!("{base}/system/bonus/value"),
        ),
        attack_effects: string_array_field(
            item,
            "/system/attackEffects/value",
            &format!("{base}/system/attackEffects/value"),
        ),
        damage_rolls: damage_field(item, source_ordinal, serialized, identity)?,
        attack: integer_field_with_path(
            item,
            "/system/attack/value",
            &format!("{base}/system/attack/value"),
        ),
        weapon_type: string_field_with_path(
            item,
            "/system/weaponType/value",
            &format!("{base}/system/weaponType/value"),
        ),
        attack_effects_custom: string_field_with_path(
            item,
            "/system/attackEffects/custom",
            &format!("{base}/system/attackEffects/custom"),
        ),
    };
    Ok(HazardItemSource {
        source_ordinal: source_ordinal as u32,
        id: string_field_with_path(item, "/_id", &format!("{base}/_id")),
        name: string_field_with_path(item, "/name", &format!("{base}/name")),
        item_type: string_field_with_path(item, "/type", &format!("{base}/type")),
        image: string_field_with_path(item, "/img", &format!("{base}/img")),
        folder: string_field_with_path(item, "/folder", &format!("{base}/folder")),
        sort: integer_field_with_path(item, "/sort", &format!("{base}/sort")),
        common,
        action,
        strike,
        unclaimed: collect_unclaimed(item, &base, &item_claimed_paths(item_type)),
    })
}

fn damage_field(
    item: &Value,
    source_ordinal: usize,
    serialized: Option<&SerializedSourceValue>,
    identity: &SourceIdentity,
) -> Result<HazardSourceField<Vec<HazardDamageSource>>, SourceDiagnostic> {
    let base = format!("/items/{source_ordinal}/system/damageRolls");
    let serialized_entries = serialized
        .and_then(SerializedSourceValue::object)
        .and_then(|item| match item.member("system") {
            SerializedSourceMember::Value(SerializedSourceValue::Object(system)) => Some(system),
            _ => None,
        })
        .and_then(|system| match system.member("damageRolls") {
            SerializedSourceMember::Value(SerializedSourceValue::Object(damage)) => Some(damage),
            _ => None,
        });
    let Some(value) = item.pointer("/system/damageRolls") else {
        return Ok(SourcePresence::Missing);
    };
    if value.is_null() {
        return Ok(SourcePresence::Null);
    }
    let (true, Some(entries)) = (value.is_object(), serialized_entries) else {
        return Ok(SourcePresence::Value(HazardSourceValue::Unsupported(
            ValueSummary {
                source_path: base,
                shape: format!("expected object; observed {}", actual_shape(value)),
                value: exact_json(value),
            },
        )));
    };
    let values = entries
        .fields()
        .iter()
        .enumerate()
        .map(|(order, (key, serialized_value))| {
            let entry_base = format!("{base}/{key}");
            let value = serialized_value
                .to_legacy_json_rejecting_duplicates()
                .map_err(|error| {
                    SourceDiagnostic::new(
                        SourceDiagnosticKind::MalformedShape,
                        identity,
                        &entry_base,
                        "damage entry without duplicate members",
                        error.to_string(),
                    )
                })?;
            Ok(HazardDamageSource {
                source_path: entry_base.clone(),
                source_key: key.clone(),
                authored_order: order as u32,
                damage: string_field_with_path(&value, "/damage", &format!("{entry_base}/damage")),
                damage_type: string_field_with_path(
                    &value,
                    "/damageType",
                    &format!("{entry_base}/damageType"),
                ),
                category: string_field_with_path(
                    &value,
                    "/category",
                    &format!("{entry_base}/category"),
                ),
                unclaimed: collect_unclaimed(
                    &value,
                    &entry_base,
                    &["/damage", "/damageType", "/category"],
                ),
            })
        })
        .collect::<Result<Vec<_>, SourceDiagnostic>>()?;
    Ok(SourcePresence::Value(HazardSourceValue::Typed(values)))
}

fn summaries_field(value: &Value, path: &str) -> HazardSourceField<Vec<ValueSummary>> {
    summaries_field_with_path(value, path, path)
}

fn summaries_field_with_path(
    value: &Value,
    local_path: &str,
    full_path: &str,
) -> HazardSourceField<Vec<ValueSummary>> {
    field_with_path(value, local_path, full_path, "array", |value| {
        value.as_array().map(|values| {
            values
                .iter()
                .enumerate()
                .map(|(index, value)| summary(&format!("{full_path}/{index}"), value))
                .collect()
        })
    })
}

fn string_field(value: &Value, path: &str) -> HazardSourceField<String> {
    string_field_with_path(value, path, path)
}

fn string_field_with_path(
    value: &Value,
    local_path: &str,
    full_path: &str,
) -> HazardSourceField<String> {
    field_with_path(value, local_path, full_path, "string", |value| {
        value.as_str().map(str::to_string)
    })
}

fn integer_field(value: &Value, path: &str) -> HazardSourceField<i64> {
    integer_field_with_path(value, path, path)
}

fn integer_field_with_path(
    value: &Value,
    local_path: &str,
    full_path: &str,
) -> HazardSourceField<i64> {
    field_with_path(value, local_path, full_path, "integer", Value::as_i64)
}

fn boolean_field(value: &Value, path: &str) -> HazardSourceField<bool> {
    boolean_field_with_path(value, path, path)
}

fn boolean_field_with_path(
    value: &Value,
    local_path: &str,
    full_path: &str,
) -> HazardSourceField<bool> {
    field_with_path(value, local_path, full_path, "boolean", Value::as_bool)
}

fn string_array_field(
    value: &Value,
    local_path: &str,
    full_path: &str,
) -> HazardSourceField<Vec<String>> {
    field_with_path(value, local_path, full_path, "array of strings", |value| {
        value
            .as_array()?
            .iter()
            .map(|value| value.as_str().map(str::to_string))
            .collect()
    })
}

fn object_field<T>(
    value: &Value,
    path: &str,
    parse: impl FnOnce(&Value) -> T,
) -> HazardSourceField<T> {
    object_field_with_path(value, path, path, parse)
}

fn object_field_with_path<T>(
    value: &Value,
    local_path: &str,
    full_path: &str,
    parse: impl FnOnce(&Value) -> T,
) -> HazardSourceField<T> {
    field_with_path(value, local_path, full_path, "object", |value| {
        value.is_object().then(|| parse(value))
    })
}

fn field<T>(
    value: &Value,
    path: &str,
    expected: &str,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> HazardSourceField<T> {
    field_with_path(value, path, path, expected, parse)
}

fn field_with_path<T>(
    value: &Value,
    local_path: &str,
    full_path: &str,
    expected: &str,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> HazardSourceField<T> {
    let Some(value) = value.pointer(local_path) else {
        return SourcePresence::Missing;
    };
    if value.is_null() {
        return SourcePresence::Null;
    }
    SourcePresence::Value(match parse(value) {
        Some(value) => HazardSourceValue::Typed(value),
        None => HazardSourceValue::Unsupported(ValueSummary {
            source_path: full_path.to_string(),
            shape: format!("expected {expected}; observed {}", actual_shape(value)),
            value: exact_json(value),
        }),
    })
}

fn collect_unclaimed_root(value: &Value) -> Vec<ValueSummary> {
    collect_unclaimed(value, "", &root_claimed_paths())
}

fn collect_unclaimed(value: &Value, base: &str, claimed: &[&str]) -> Vec<ValueSummary> {
    let mut output = Vec::new();
    collect_leaves(value, base, "", claimed, &mut output);
    output
}

fn collect_leaves(
    value: &Value,
    base: &str,
    local: &str,
    claimed: &[&str],
    output: &mut Vec<ValueSummary>,
) {
    let full_local = if local.is_empty() { "/" } else { local };
    if claimed.iter().any(|path| {
        *path == full_local
            || path.ends_with("/**") && full_local.starts_with(path.trim_end_matches("**"))
    }) {
        return;
    }
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                output.push(summary(&format!("{base}{local}"), value));
            } else {
                for (key, value) in map {
                    let next = format!("{local}/{}", escape_pointer(key));
                    collect_leaves(value, base, &next, claimed, output);
                }
            }
        }
        Value::Array(values) => {
            if values.is_empty() {
                output.push(summary(&format!("{base}{local}"), value));
            } else {
                for (index, value) in values.iter().enumerate() {
                    let next = format!("{local}/{index}");
                    collect_leaves(value, base, &next, claimed, output);
                }
            }
        }
        _ => output.push(summary(&format!("{base}{local}"), value)),
    }
}

fn root_claimed_paths() -> Vec<&'static str> {
    vec![
        "/_id",
        "/name",
        "/type",
        "/img",
        "/folder",
        "/items",
        "/items/**",
        "/effects",
        "/effects/**",
        "/prototypeToken/name",
        "/system/traits/value",
        "/system/traits/value/**",
        "/system/traits/rarity",
        "/system/traits/size/value",
        "/system/details/level/value",
        "/system/details/publication/title",
        "/system/details/publication/remaster",
        "/system/details/publication/license",
        "/system/details/isComplex",
        "/system/details/description",
        "/system/details/disable",
        "/system/details/routine",
        "/system/details/reset",
        "/system/attributes/stealth/value",
        "/system/attributes/stealth/details",
        "/system/attributes/ac/value",
        "/system/attributes/hardness",
        "/system/attributes/hp/value",
        "/system/attributes/hp/max",
        "/system/attributes/hp/temp",
        "/system/attributes/hp/details",
        "/system/attributes/hp/tempmax",
        "/system/attributes/immunities",
        "/system/attributes/immunities/**",
        "/system/attributes/weaknesses",
        "/system/attributes/weaknesses/**",
        "/system/attributes/resistances",
        "/system/attributes/resistances/**",
        "/system/attributes/hasHealth",
        "/system/attributes/emitsSound",
        "/system/saves/fortitude/value",
        "/system/saves/fortitude/saveDetail",
        "/system/saves/reflex/value",
        "/system/saves/reflex/saveDetail",
        "/system/saves/will/value",
        "/system/saves/will/saveDetail",
        "/system/creatureType",
        "/system/statusEffects",
        "/system/statusEffects/**",
    ]
}

fn item_claimed_paths(item_type: Option<&str>) -> Vec<&'static str> {
    let mut paths = vec![
        "/_id",
        "/name",
        "/type",
        "/img",
        "/folder",
        "/sort",
        "/system/description/value",
        "/system/publication/title",
        "/system/publication/remaster",
        "/system/publication/license",
        "/system/rules",
        "/system/rules/**",
        "/system/slug",
        "/system/traits/value",
        "/system/traits/value/**",
        "/system/traits/rarity",
        "/_stats/compendiumSource",
    ];
    match item_type {
        Some("action") => paths.extend([
            "/system/actionType/value",
            "/system/actions/value",
            "/system/category",
            "/system/deathNote",
            "/system/frequency/value",
            "/system/frequency/max",
            "/system/frequency/per",
            "/system/selfEffect/uuid",
            "/system/selfEffect/name",
        ]),
        Some("melee") => paths.extend([
            "/system/bonus/value",
            "/system/attack/value",
            "/system/weaponType/value",
            "/system/attackEffects/value",
            "/system/attackEffects/value/**",
            "/system/attackEffects/custom",
            "/system/damageRolls",
            "/system/damageRolls/**",
        ]),
        _ => {}
    }
    paths
}

fn summary(path: &str, value: &Value) -> ValueSummary {
    ValueSummary {
        source_path: path.to_string(),
        shape: actual_shape(value).to_string(),
        value: exact_json(value),
    }
}

fn exact_json(value: &Value) -> String {
    value.to_string()
}

fn escape_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn required_root_string(
    value: Option<&Value>,
    identity: &SourceIdentity,
    path: &str,
) -> Result<String, SourceDiagnostic> {
    value
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            SourceDiagnostic::new(
                SourceDiagnosticKind::MalformedShape,
                identity,
                path,
                "string",
                value.map(actual_shape).unwrap_or("missing"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::dto::{parse_serialized_source_object, pinned_source_version_metadata};

    #[test]
    fn nested_duplicate_damage_member_is_a_typed_source_error() {
        let bytes = br#"{"_id":"hazard-id","name":"Duplicate Damage","type":"hazard","items":[{"_id":"strike-id","name":"Strike","type":"melee","system":{"damageRolls":{"first":{"damage":"1d6","damage":"2d6","damageType":"piercing"}}}}],"system":{}}"#;
        let raw: Value = serde_json::from_slice(bytes).expect("collapsed fixture JSON");
        let serialized = parse_serialized_source_object(bytes).expect("lossless fixture JSON");

        let error = parse_hazard_source(
            pinned_source_version_metadata(),
            SourceIdentity::new("hazards:hazard-id", "packs/hazards/hazard-id.json"),
            raw,
            &serialized,
        )
        .expect_err("nested duplicate damage must fail instead of panicking");

        assert_eq!(error.kind, SourceDiagnosticKind::MalformedShape);
        assert_eq!(error.json_path(), "/items/0/system/damageRolls/first");
        assert!(error.actual_shape().contains("/damage"));
    }
}
