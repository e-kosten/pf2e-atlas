use atlas_domain::Rarity;
use atlas_record::{
    ConsumableSpellLocation, FactValue, PublicationLicense, SpellAreaPatch, SpellAreaType,
    SpellAreaValue, SpellCasting, SpellCastingPatch, SpellClassification, SpellClassificationPatch,
    SpellDamage, SpellDamageAlterationRule, SpellDamageDiceRule, SpellDamagePatch,
    SpellDefensePatch, SpellDefenseValue, SpellDuration, SpellDurationPatch,
    SpellEphemeralEffectRule, SpellFact, SpellFixedHeighteningLayer, SpellHeightening,
    SpellHeighteningPatch, SpellHeighteningType, SpellIntervalHeightening, SpellItemAlterationRule,
    SpellKeyedPatch, SpellKeyedPatchMember, SpellKeyedPatchOperation, SpellLegacyAreaType,
    SpellOrderedMember, SpellOverlay, SpellOverlayId, SpellOverlayType, SpellPatch,
    SpellRangeValue, SpellRitual, SpellRollOptionRule, SpellRule, SpellRuleElement,
    SpellRulePredicate, SpellRuleSuboption, SpellSave, SpellSavePatch, SpellSourceValue,
    SpellStatistic, SpellTargeting, SpellTargetingPatch, SpellTextPatch, SpellTradition,
    SpellTrait, SpellUnsupportedPatchField, SpellUnsupportedRule, SpellUnsupportedRulePredicate,
    SpellUnsupportedSourceFact, SpellUnsupportedSourceField, StableSourceLocator,
    UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
};

use super::value::{
    SerializedSourceObject, SerializedSourceValue, required_serialized_object,
    serialized_member as member, serialized_member_order as member_order,
    serialized_members as members, serialized_unique_object as unique_object,
    serialized_unique_string as unique_string,
};
use super::{SourceDiagnostic, SourceDiagnosticKind, SourceIdentity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpellDocumentSource {
    Standalone(SpellItemSource),
    ConsumableChild(ConsumableSpellChildSource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpellItemSource {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) image: SpellFact<String>,
    pub(crate) publication_license: SpellFact<PublicationLicense>,
    pub(crate) slug: SpellFact<String>,
    pub(crate) publication_title: SpellFact<String>,
    pub(crate) publication_remaster: SpellFact<bool>,
    pub(crate) rarity: SpellFact<Rarity>,
    pub(crate) classification: SpellFact<SpellClassification>,
    pub(crate) casting: SpellFact<SpellCasting>,
    pub(crate) targeting: SpellFact<SpellTargeting>,
    pub(crate) defense: SpellFact<SpellDefenseValue>,
    pub(crate) damage: SpellFact<Vec<SpellOrderedMember<SpellDamage>>>,
    pub(crate) duration: SpellFact<SpellDuration>,
    pub(crate) heightening: SpellFact<SpellHeightening>,
    pub(crate) overlays: SpellFact<Vec<SpellOverlay>>,
    pub(crate) ritual: SpellFact<SpellRitual>,
    pub(crate) rules: SpellFact<Vec<SpellRuleElement>>,
    pub(crate) location: SpellFact<ConsumableSpellLocation>,
    pub(crate) location_provenance: FactValue<UnsupportedSourceValue>,
    pub(crate) description_markup: SpellFact<String>,
    pub(crate) unsupported_notes: Vec<SpellUnsupportedSourceFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumableSpellChildSource {
    pub(crate) authored_order: u32,
    pub(crate) spell: SpellItemSource,
    pub(crate) standalone_locator: SpellFact<StableSourceLocator>,
}

pub(crate) fn parse_spell_document_source(
    source: &SerializedSourceObject,
    identity: &SourceIdentity,
) -> Result<Option<SpellDocumentSource>, SourceDiagnostic> {
    let root = source.fields();
    let item_type = match members(root, "type").as_slice() {
        [] => return Ok(None),
        [SerializedSourceValue::String(value)] => value.as_str(),
        values => {
            return Err(SourceDiagnostic::new(
                SourceDiagnosticKind::MalformedShape,
                identity,
                "$.type",
                "one string document type",
                duplicate_value("type", values),
            ));
        }
    };
    match item_type {
        "spell" => parse_spell_item(root, "$", identity)
            .map(|source| Some(SpellDocumentSource::Standalone(source))),
        "consumable" => {
            let Some(system) = unique_object(root, "system") else {
                return Ok(None);
            };
            let matches = members(system, "spell");
            if matches.is_empty()
                || matches
                    .iter()
                    .all(|value| matches!(value, SerializedSourceValue::Null))
            {
                return Ok(None);
            }
            if matches.len() != 1 {
                return Err(SourceDiagnostic::new(
                    SourceDiagnosticKind::MalformedShape,
                    identity,
                    "$.system.spell",
                    "one consumable spell child",
                    duplicate_value("spell", &matches),
                ));
            }
            let child = required_serialized_object(matches[0], identity, "$.system.spell")?;
            if unique_string(child, "type") != Some("spell") {
                return Err(SourceDiagnostic::new(
                    SourceDiagnosticKind::MalformedShape,
                    identity,
                    "$.system.spell.type",
                    "spell",
                    member(child, "type")
                        .map(SerializedSourceValue::compact_json)
                        .unwrap_or_else(|| "<missing>".to_string()),
                ));
            }
            let spell = parse_spell_item(child, "$.system.spell", identity)?;
            let standalone_locator = nested_fact(
                child,
                &["flags", "core", "sourceId"],
                "system.spell.flags.core.sourceId",
                |value| match value {
                    SerializedSourceValue::String(value) => {
                        StableSourceLocator::new(value.clone()).ok()
                    }
                    _ => None,
                },
            );
            Ok(Some(SpellDocumentSource::ConsumableChild(
                ConsumableSpellChildSource {
                    authored_order: 0,
                    spell,
                    standalone_locator,
                },
            )))
        }
        _ => Ok(None),
    }
}

fn parse_spell_item(
    item: &[(String, SerializedSourceValue)],
    item_path: &str,
    identity: &SourceIdentity,
) -> Result<SpellItemSource, SourceDiagnostic> {
    let id = unique_string(item, "_id")
        .ok_or_else(|| malformed(identity, &format!("{item_path}._id"), "one string", item))?
        .to_string();
    let name = unique_string(item, "name")
        .ok_or_else(|| malformed(identity, &format!("{item_path}.name"), "one string", item))?
        .to_string();
    let system = unique_object(item, "system")
        .ok_or_else(|| malformed(identity, &format!("{item_path}.system"), "one object", item))?;
    let system_path = if item_path == "$" {
        "system".to_string()
    } else {
        format!("{}.system", item_path.trim_start_matches("$."))
    };
    let image = member_fact(item, "img", &format!("{item_path}.img"), parse_string);
    let publication_license = nested_fact(
        system,
        &["publication", "license"],
        &format!("{system_path}.publication.license"),
        |value| parse_string(value).and_then(|value| PublicationLicense::new(value).ok()),
    );
    let slug = member_fact(system, "slug", &format!("{system_path}.slug"), parse_string);
    let publication_title = nested_fact(
        system,
        &["publication", "title"],
        &format!("{system_path}.publication.title"),
        parse_string,
    );
    let publication_remaster = nested_fact(
        system,
        &["publication", "remaster"],
        &format!("{system_path}.publication.remaster"),
        parse_bool,
    );
    let rarity = nested_fact(
        system,
        &["traits", "rarity"],
        &format!("{system_path}.traits.rarity"),
        |value| parse_string(value).and_then(|value| Rarity::from_canonical(&value)),
    );
    let mut unsupported_notes = unknown_members(
        system,
        &[
            "area",
            "cost",
            "counteraction",
            "damage",
            "defense",
            "description",
            "duration",
            "heightening",
            "level",
            "location",
            "overlays",
            "publication",
            "range",
            "requirements",
            "ritual",
            "rules",
            "slug",
            "target",
            "time",
            "traits",
        ],
        &system_path,
        SpellUnsupportedSourceField::SystemMember,
    );
    retain_nested_source_context(system, &system_path, &mut unsupported_notes);
    let classification = known(SpellClassification {
        rank: nested_fact(
            system,
            &["level", "value"],
            &format!("{system_path}.level.value"),
            parse_u8,
        ),
        traits: nested_fact(
            system,
            &["traits", "value"],
            &format!("{system_path}.traits.value"),
            |value| parse_string_array(value, SpellTrait::new),
        ),
        traditions: nested_fact(
            system,
            &["traits", "traditions"],
            &format!("{system_path}.traits.traditions"),
            |value| parse_string_array(value, SpellTradition::new),
        ),
    });
    let casting = known(SpellCasting {
        time: nested_fact(
            system,
            &["time", "value"],
            &format!("{system_path}.time.value"),
            parse_string,
        ),
        cost: nested_fact(
            system,
            &["cost", "value"],
            &format!("{system_path}.cost.value"),
            parse_string,
        ),
        requirements: member_fact(
            system,
            "requirements",
            &format!("{system_path}.requirements"),
            parse_string,
        ),
        counteraction: member_fact(
            system,
            "counteraction",
            &format!("{system_path}.counteraction"),
            parse_bool,
        ),
    });
    let targeting = known(SpellTargeting {
        target: nested_fact(
            system,
            &["target", "value"],
            &format!("{system_path}.target.value"),
            parse_string,
        ),
        range: nested_fact(
            system,
            &["range", "value"],
            &format!("{system_path}.range.value"),
            |value| parse_string(value).map(SpellRangeValue::from_authored_text),
        ),
        area: parse_area_member(
            system,
            "area",
            &format!("{system_path}.area"),
            &mut unsupported_notes,
        ),
    });
    let defense = parse_defense(system, &system_path, &mut unsupported_notes);
    let damage = parse_damage_map(
        system,
        "damage",
        &format!("{system_path}.damage"),
        &mut unsupported_notes,
    );
    let duration = known(SpellDuration {
        value: nested_fact(
            system,
            &["duration", "value"],
            &format!("{system_path}.duration.value"),
            parse_string,
        ),
        sustained: nested_fact(
            system,
            &["duration", "sustained"],
            &format!("{system_path}.duration.sustained"),
            parse_bool,
        ),
    });
    let heightening = parse_heightening(system, &system_path, &mut unsupported_notes);
    let overlays = parse_overlays(system, &system_path, &mut unsupported_notes);
    let ritual = parse_ritual(system, &system_path, &mut unsupported_notes);
    let rules = parse_rules(system, &system_path);
    let location = parse_location(system, &system_path, &mut unsupported_notes);
    let location_provenance = provenance_fact(system, "location");
    let description_markup = nested_fact(
        system,
        &["description", "value"],
        &format!("{system_path}.description.value"),
        parse_string,
    );
    if matches!(
        description_markup,
        FactValue::Value(SpellSourceValue::Unsupported(_))
    ) {
        if let Some(value) = unique_object(system, "description")
            .and_then(|description| member(description, "value"))
        {
            unsupported_notes.push(unsupported_note(
                SpellUnsupportedSourceField::ProvenanceMember,
                &format!("{system_path}.description.value"),
                "value",
                None,
                value,
            ));
        } else {
            let values = members(system, "description");
            unsupported_notes.push(SpellUnsupportedSourceFact {
                field: SpellUnsupportedSourceField::ProvenanceMember,
                source_path: format!("{system_path}.description"),
                authored_key: "description".to_string(),
                authored_order: member_order(system, "description"),
                value: UnsupportedSourceValue {
                    shape: UnsupportedSourceShape::Array,
                    value: duplicate_value("description", &values),
                    reason: UnsupportedSourceReason::SourceFieldDrift,
                },
            });
        }
    }
    Ok(SpellItemSource {
        id,
        name,
        image,
        publication_license,
        slug,
        publication_title,
        publication_remaster,
        rarity,
        classification,
        casting,
        targeting,
        defense,
        damage,
        duration,
        heightening,
        overlays,
        ritual,
        rules,
        location,
        location_provenance,
        description_markup,
        unsupported_notes,
    })
}

fn parse_area_member(
    object: &[(String, SerializedSourceValue)],
    key: &str,
    path: &str,
    parent_notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<SpellAreaValue> {
    member_fact(object, key, path, |value| {
        let area = value.object()?;
        let unsupported_notes = unknown_members(
            area,
            &["type", "value", "details", "areaType"],
            path,
            SpellUnsupportedSourceField::AreaMember,
        );
        parent_notes.extend(unsupported_notes.iter().cloned());
        Some(SpellAreaValue {
            value: member_fact(area, "value", &format!("{path}.value"), parse_u32),
            area_type: member_fact(area, "type", &format!("{path}.type"), |value| {
                parse_string(value).and_then(|value| SpellAreaType::new(value).ok())
            }),
            legacy_area_type: parse_legacy_area_type(area, path),
            details: member_fact(area, "details", &format!("{path}.details"), parse_string),
            unsupported_notes,
        })
    })
}

fn retain_nested_source_context(
    system: &[(String, SerializedSourceValue)],
    system_path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) {
    for (root, known, field) in [
        (
            "cost",
            &["value"][..],
            SpellUnsupportedSourceField::CastingMember,
        ),
        (
            "level",
            &["value"][..],
            SpellUnsupportedSourceField::ClassificationMember,
        ),
        (
            "range",
            &["value"][..],
            SpellUnsupportedSourceField::TargetingMember,
        ),
        (
            "target",
            &["value"][..],
            SpellUnsupportedSourceField::TargetingMember,
        ),
        (
            "time",
            &["value"][..],
            SpellUnsupportedSourceField::CastingMember,
        ),
        (
            "duration",
            &["sustained", "value"][..],
            SpellUnsupportedSourceField::DurationMember,
        ),
        (
            "description",
            &["gm", "value"][..],
            SpellUnsupportedSourceField::ProvenanceMember,
        ),
        (
            "publication",
            &["license", "remaster", "title"][..],
            SpellUnsupportedSourceField::ProvenanceMember,
        ),
    ] {
        if let Some(object) = unique_object(system, root) {
            notes.extend(unknown_members(
                object,
                known,
                &format!("{system_path}.{root}"),
                field,
            ));
        }
    }
    let Some(traits) = unique_object(system, "traits") else {
        return;
    };
    notes.extend(unknown_members(
        traits,
        &["rarity", "selected", "traditions", "value"],
        &format!("{system_path}.traits"),
        SpellUnsupportedSourceField::ClassificationMember,
    ));
    let Some(selected) = unique_object(traits, "selected") else {
        return;
    };
    notes.extend(selected.iter().enumerate().map(|(order, (key, value))| {
        unsupported_note(
            SpellUnsupportedSourceField::LegacyTraitSelection,
            &format!("{system_path}.traits.selected.{key}"),
            key,
            Some(order as u32),
            value,
        )
    }));
}

fn parse_legacy_area_type(
    area: &[(String, SerializedSourceValue)],
    path: &str,
) -> SpellFact<SpellLegacyAreaType> {
    let values = members(area, "areaType");
    match values.as_slice() {
        [] => FactValue::Missing,
        [SerializedSourceValue::Null] => FactValue::Null,
        [SerializedSourceValue::String(value)] => match SpellAreaType::new(value.clone()) {
            Ok(value) => known(SpellLegacyAreaType {
                source_path: format!("{path}.areaType"),
                authored_key: "areaType".to_string(),
                authored_order: member_order(area, "areaType").unwrap_or_default(),
                value,
            }),
            Err(_) => unsupported(values[0], UnsupportedSourceReason::SourceFieldDrift),
        },
        [value] => unsupported(value, UnsupportedSourceReason::SourceFieldDrift),
        _ => unsupported_duplicate("areaType", &values),
    }
}

fn parse_defense(
    system: &[(String, SerializedSourceValue)],
    system_path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<SpellDefenseValue> {
    member_fact(
        system,
        "defense",
        &format!("{system_path}.defense"),
        |value| {
            let defense = value.object()?;
            if has_unknown(defense, &["passive", "save"]) {
                notes.extend(unknown_members(
                    defense,
                    &["passive", "save"],
                    &format!("{system_path}.defense"),
                    SpellUnsupportedSourceField::DefenseMember,
                ));
                return None;
            }
            if let Some(passive) = unique_object(defense, "passive")
                && has_unknown(passive, &["statistic"])
            {
                notes.extend(unknown_members(
                    passive,
                    &["statistic"],
                    &format!("{system_path}.defense.passive"),
                    SpellUnsupportedSourceField::DefenseMember,
                ));
                return None;
            }
            if let Some(save) = unique_object(defense, "save")
                && has_unknown(save, &["statistic", "basic"])
            {
                notes.extend(unknown_members(
                    save,
                    &["statistic", "basic"],
                    &format!("{system_path}.defense.save"),
                    SpellUnsupportedSourceField::DefenseMember,
                ));
                return None;
            }
            Some(SpellDefenseValue {
                passive: nested_fact(
                    defense,
                    &["passive", "statistic"],
                    &format!("{system_path}.defense.passive.statistic"),
                    |value| parse_string(value).and_then(|value| SpellStatistic::new(value).ok()),
                ),
                save: member_fact(
                    defense,
                    "save",
                    &format!("{system_path}.defense.save"),
                    |value| {
                        let save = value.object()?;
                        if has_unknown(save, &["statistic", "basic"]) {
                            return None;
                        }
                        Some(SpellSave {
                            statistic: member_fact(
                                save,
                                "statistic",
                                &format!("{system_path}.defense.save.statistic"),
                                |value| {
                                    parse_string(value)
                                        .and_then(|value| SpellStatistic::new(value).ok())
                                },
                            ),
                            basic: member_fact(
                                save,
                                "basic",
                                &format!("{system_path}.defense.save.basic"),
                                parse_bool,
                            ),
                        })
                    },
                ),
            })
        },
    )
}

fn parse_damage_map(
    object: &[(String, SerializedSourceValue)],
    key: &str,
    path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<Vec<SpellOrderedMember<SpellDamage>>> {
    member_fact(object, key, path, |value| {
        let damage = value.object()?;
        Some(
            damage
                .iter()
                .enumerate()
                .map(|(order, (member_key, value))| {
                    let entry_path = format!("{path}.{member_key}");
                    let parsed = parse_damage(value, &entry_path, notes).unwrap_or_else(|| {
                        notes.push(unsupported_note(
                            SpellUnsupportedSourceField::DamageMember,
                            &entry_path,
                            member_key,
                            Some(order as u32),
                            value,
                        ));
                        SpellDamage {
                            formula: unsupported(value, UnsupportedSourceReason::SourceFieldDrift),
                            ..SpellDamage::default()
                        }
                    });
                    SpellOrderedMember {
                        key: member_key.clone(),
                        authored_order: order as u32,
                        value: parsed,
                    }
                })
                .collect(),
        )
    })
}

fn parse_damage(
    value: &SerializedSourceValue,
    path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> Option<SpellDamage> {
    let damage = value.object()?;
    notes.extend(unknown_members(
        damage,
        &[
            "applyMod",
            "category",
            "formula",
            "kinds",
            "materials",
            "type",
        ],
        path,
        SpellUnsupportedSourceField::DamageMember,
    ));
    Some(SpellDamage {
        apply_mod: member_fact(damage, "applyMod", &format!("{path}.applyMod"), parse_bool),
        category: member_fact(
            damage,
            "category",
            &format!("{path}.category"),
            parse_string,
        ),
        formula: member_fact(damage, "formula", &format!("{path}.formula"), parse_string),
        kinds: member_fact(damage, "kinds", &format!("{path}.kinds"), |value| {
            parse_string_array(value, Ok::<_, ()>)
        }),
        materials: member_fact(damage, "materials", &format!("{path}.materials"), |value| {
            parse_string_array(value, Ok::<_, ()>)
        }),
        damage_type: member_fact(damage, "type", &format!("{path}.type"), parse_string),
    })
}

fn parse_heightening(
    system: &[(String, SerializedSourceValue)],
    system_path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<SpellHeightening> {
    member_fact(
        system,
        "heightening",
        &format!("{system_path}.heightening"),
        |value| {
            let heightening = value.object()?;
            match unique_string(heightening, "type") {
                Some("interval") => {
                    let unknown = unknown_members(
                        heightening,
                        &["area", "damage", "interval", "type"],
                        &format!("{system_path}.heightening"),
                        SpellUnsupportedSourceField::HeighteningMember,
                    );
                    if !unknown.is_empty() {
                        notes.extend(unknown);
                        return None;
                    }
                    Some(SpellHeightening::Interval(SpellIntervalHeightening {
                        interval: member_fact(
                            heightening,
                            "interval",
                            &format!("{system_path}.heightening.interval"),
                            parse_u8,
                        ),
                        area: member_fact(
                            heightening,
                            "area",
                            &format!("{system_path}.heightening.area"),
                            parse_u32,
                        ),
                        damage: parse_text_map(
                            heightening,
                            "damage",
                            &format!("{system_path}.heightening.damage"),
                            notes,
                        ),
                    }))
                }
                Some("fixed") => {
                    let unknown = unknown_members(
                        heightening,
                        &["levels", "type"],
                        &format!("{system_path}.heightening"),
                        SpellUnsupportedSourceField::HeighteningMember,
                    );
                    if !unknown.is_empty() {
                        notes.extend(unknown);
                        return None;
                    }
                    let levels = unique_object(heightening, "levels")?;
                    Some(SpellHeightening::Fixed(
                        levels
                            .iter()
                            .enumerate()
                            .map(|(order, (key, value))| SpellFixedHeighteningLayer {
                                key: key.clone(),
                                authored_order: order as u32,
                                rank: key
                                    .parse::<u8>()
                                    .map(SpellSourceValue::Known)
                                    .unwrap_or_else(|_| {
                                        SpellSourceValue::Unsupported(unsupported_value(
                                            value,
                                            UnsupportedSourceReason::SourceFieldDrift,
                                        ))
                                    }),
                                patch: parse_patch(
                                    value,
                                    &format!("{system_path}.heightening.levels.{key}"),
                                ),
                            })
                            .collect(),
                    ))
                }
                _ => {
                    notes.push(unsupported_note(
                        SpellUnsupportedSourceField::HeighteningMember,
                        &format!("{system_path}.heightening.type"),
                        "type",
                        member_order(heightening, "type"),
                        member(heightening, "type").unwrap_or(value),
                    ));
                    None
                }
            }
        },
    )
}

fn parse_text_map(
    object: &[(String, SerializedSourceValue)],
    key: &str,
    path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<Vec<SpellOrderedMember<String>>> {
    member_fact(object, key, path, |value| {
        let values = value.object()?;
        let mut parsed = Vec::with_capacity(values.len());
        for (order, (key, value)) in values.iter().enumerate() {
            let Some(value_text) = parse_string(value) else {
                notes.push(unsupported_note(
                    SpellUnsupportedSourceField::HeighteningMember,
                    &format!("{path}.{key}"),
                    key,
                    Some(order as u32),
                    value,
                ));
                return None;
            };
            parsed.push(SpellOrderedMember {
                key: key.clone(),
                authored_order: order as u32,
                value: value_text,
            });
        }
        Some(parsed)
    })
}

fn parse_overlays(
    system: &[(String, SerializedSourceValue)],
    system_path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<Vec<SpellOverlay>> {
    member_fact(
        system,
        "overlays",
        &format!("{system_path}.overlays"),
        |value| {
            let overlays = value.object()?;
            let mut parsed = Vec::with_capacity(overlays.len());
            for (order, (key, value)) in overlays.iter().enumerate() {
                let path = format!("{system_path}.overlays.{key}");
                let Some(overlay) = value.object() else {
                    notes.push(unsupported_note(
                        SpellUnsupportedSourceField::OverlayMember,
                        &path,
                        key,
                        Some(order as u32),
                        value,
                    ));
                    return None;
                };
                let unknown_overlay_members = unknown_members(
                    overlay,
                    &["_id", "name", "overlayType", "sort", "system"],
                    &path,
                    SpellUnsupportedSourceField::OverlayMember,
                );
                notes.extend(unknown_overlay_members.iter().cloned());
                let overlay_id = match SpellOverlayId::new(key.clone()) {
                    Ok(value) => value,
                    Err(_) => {
                        notes.push(unsupported_note(
                            SpellUnsupportedSourceField::OverlayMember,
                            &path,
                            key,
                            Some(order as u32),
                            value,
                        ));
                        return None;
                    }
                };
                let mut patch = member(overlay, "system")
                    .map(|value| parse_patch(value, &format!("{path}.system")))
                    .unwrap_or_default();
                for note in unknown_overlay_members {
                    patch.unsupported.extend(
                        [
                            atlas_record::SpellFormField::Classification,
                            atlas_record::SpellFormField::Casting,
                            atlas_record::SpellFormField::Targeting,
                            atlas_record::SpellFormField::Defense,
                            atlas_record::SpellFormField::Damage,
                            atlas_record::SpellFormField::Duration,
                            atlas_record::SpellFormField::Heightening,
                            atlas_record::SpellFormField::Rules,
                        ]
                        .into_iter()
                        .map(|field| SpellUnsupportedPatchField {
                            field,
                            source_path: note.source_path.clone(),
                            authored_key: note.authored_key.clone(),
                            authored_order: note.authored_order,
                            value: note.value.clone(),
                        }),
                    );
                }
                parsed.push(SpellOverlay {
                    key: key.clone(),
                    authored_order: order as u32,
                    overlay_id,
                    source_id: member_fact(overlay, "_id", &format!("{path}._id"), |value| {
                        parse_string(value).and_then(|value| SpellOverlayId::new(value).ok())
                    }),
                    sort: member_fact(overlay, "sort", &format!("{path}.sort"), parse_i64),
                    name: member_fact(overlay, "name", &format!("{path}.name"), parse_string),
                    overlay_type: member_fact(
                        overlay,
                        "overlayType",
                        &format!("{path}.overlayType"),
                        |value| match parse_string(value).as_deref() {
                            Some("override") => Some(SpellOverlayType::Override),
                            _ => None,
                        },
                    ),
                    patch,
                });
            }
            Some(parsed)
        },
    )
}

fn parse_patch(value: &SerializedSourceValue, path: &str) -> SpellPatch {
    let Some(system) = value.object() else {
        return SpellPatch {
            unsupported: unsupported_patch_for_all_fields(path, "system", None, value),
            ..SpellPatch::default()
        };
    };
    let mut unsupported_fields = Vec::new();
    for (order, (key, value)) in system.iter().enumerate() {
        if !matches!(
            key.as_str(),
            "area"
                | "cost"
                | "counteraction"
                | "damage"
                | "defense"
                | "duration"
                | "heightening"
                | "level"
                | "range"
                | "requirements"
                | "rules"
                | "target"
                | "time"
                | "traits"
        ) {
            unsupported_fields.extend(unsupported_patch_for_all_fields(
                &format!("{path}.{key}"),
                key,
                Some(order as u32),
                value,
            ));
        }
    }
    retain_patch_nested_unknown(system, path, &mut unsupported_fields);
    let classification = if has_any_member(system, &["level", "traits"]) {
        known(SpellClassificationPatch {
            rank: nested_fact(
                system,
                &["level", "value"],
                &format!("{path}.level.value"),
                parse_u8,
            ),
            traits: nested_fact(
                system,
                &["traits", "value"],
                &format!("{path}.traits.value"),
                |value| parse_string_array(value, SpellTrait::new),
            ),
            traditions: nested_fact(
                system,
                &["traits", "traditions"],
                &format!("{path}.traits.traditions"),
                |value| parse_string_array(value, SpellTradition::new),
            ),
        })
    } else {
        FactValue::Missing
    };
    let casting = if has_any_member(system, &["time", "cost", "requirements", "counteraction"]) {
        known(SpellCastingPatch {
            time: nested_fact(
                system,
                &["time", "value"],
                &format!("{path}.time.value"),
                parse_string,
            ),
            cost: nested_fact(
                system,
                &["cost", "value"],
                &format!("{path}.cost.value"),
                parse_string,
            ),
            requirements: member_fact(
                system,
                "requirements",
                &format!("{path}.requirements"),
                parse_string,
            ),
            counteraction: member_fact(
                system,
                "counteraction",
                &format!("{path}.counteraction"),
                parse_bool,
            ),
        })
    } else {
        FactValue::Missing
    };
    let mut area_notes = Vec::new();
    let targeting = if has_any_member(system, &["target", "range", "area"]) {
        known(SpellTargetingPatch {
            target: nested_fact(
                system,
                &["target", "value"],
                &format!("{path}.target.value"),
                parse_string,
            ),
            range: nested_fact(
                system,
                &["range", "value"],
                &format!("{path}.range.value"),
                |value| parse_string(value).map(SpellRangeValue::from_authored_text),
            ),
            area: parse_area_patch(system, "area", &format!("{path}.area"), &mut area_notes),
        })
    } else {
        FactValue::Missing
    };
    unsupported_fields.extend(
        area_notes
            .into_iter()
            .map(|note| SpellUnsupportedPatchField {
                field: atlas_record::SpellFormField::Targeting,
                source_path: note.source_path,
                authored_key: note.authored_key,
                authored_order: note.authored_order,
                value: note.value,
            }),
    );
    let defense = parse_defense_patch(system, path, &mut unsupported_fields);
    let damage = parse_damage_patch_map(system, path, &mut unsupported_fields);
    let duration = if has_any_member(system, &["duration"]) {
        known(SpellDurationPatch {
            value: nested_fact(
                system,
                &["duration", "value"],
                &format!("{path}.duration.value"),
                parse_string,
            ),
            sustained: nested_fact(
                system,
                &["duration", "sustained"],
                &format!("{path}.duration.sustained"),
                parse_bool,
            ),
        })
    } else {
        FactValue::Missing
    };
    let heightening = member_fact(
        system,
        "heightening",
        &format!("{path}.heightening"),
        |value| {
            let heightening = value.object()?;
            if has_unknown(heightening, &["type", "interval", "area", "damage"]) {
                return None;
            }
            Some(SpellHeighteningPatch {
                kind: member_fact(
                    heightening,
                    "type",
                    &format!("{path}.heightening.type"),
                    |value| {
                        matches!(parse_string(value).as_deref(), Some("interval"))
                            .then_some(SpellHeighteningType::Interval)
                    },
                ),
                interval: member_fact(
                    heightening,
                    "interval",
                    &format!("{path}.heightening.interval"),
                    parse_u8,
                ),
                area: member_fact(
                    heightening,
                    "area",
                    &format!("{path}.heightening.area"),
                    parse_u32,
                ),
                interval_damage: parse_text_patch_map(
                    heightening,
                    "damage",
                    &format!("{path}.heightening.damage"),
                ),
            })
        },
    );
    let rules = parse_rules_at(system, "rules", &format!("{path}.rules"));
    SpellPatch {
        classification,
        casting,
        targeting,
        defense,
        damage,
        duration,
        heightening,
        rules,
        unsupported: unsupported_fields,
    }
}

fn parse_area_patch(
    object: &[(String, SerializedSourceValue)],
    key: &str,
    path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<SpellAreaPatch> {
    member_fact(object, key, path, |value| {
        let area = value.object()?;
        let unsupported_notes = unknown_members(
            area,
            &["type", "value", "details", "areaType"],
            path,
            SpellUnsupportedSourceField::AreaMember,
        );
        notes.extend(unsupported_notes.iter().cloned());
        Some(SpellAreaPatch {
            value: member_fact(area, "value", &format!("{path}.value"), parse_u32),
            area_type: member_fact(area, "type", &format!("{path}.type"), |value| {
                parse_string(value).and_then(|value| SpellAreaType::new(value).ok())
            }),
            legacy_area_type: parse_legacy_area_type(area, path),
            details: member_fact(area, "details", &format!("{path}.details"), parse_string),
            unsupported_notes,
        })
    })
}

fn parse_defense_patch(
    system: &[(String, SerializedSourceValue)],
    path: &str,
    unsupported_fields: &mut Vec<SpellUnsupportedPatchField>,
) -> SpellFact<SpellDefensePatch> {
    member_fact(system, "defense", &format!("{path}.defense"), |value| {
        let defense = value.object()?;
        for (order, (key, value)) in defense.iter().enumerate() {
            if !matches!(key.as_str(), "passive" | "save") {
                unsupported_fields.push(SpellUnsupportedPatchField {
                    field: atlas_record::SpellFormField::Defense,
                    source_path: format!("{path}.defense.{key}"),
                    authored_key: key.clone(),
                    authored_order: Some(order as u32),
                    value: unsupported_value(value, UnsupportedSourceReason::SourceFieldDrift),
                });
            }
        }
        if let Some(save) = unique_object(defense, "save") {
            for (order, (key, value)) in save.iter().enumerate() {
                if !matches!(key.as_str(), "statistic" | "basic") {
                    unsupported_fields.push(SpellUnsupportedPatchField {
                        field: atlas_record::SpellFormField::Defense,
                        source_path: format!("{path}.defense.save.{key}"),
                        authored_key: key.clone(),
                        authored_order: Some(order as u32),
                        value: unsupported_value(value, UnsupportedSourceReason::SourceFieldDrift),
                    });
                }
            }
        }
        Some(SpellDefensePatch {
            passive: nested_fact(
                defense,
                &["passive", "statistic"],
                &format!("{path}.defense.passive.statistic"),
                |value| parse_string(value).and_then(|value| SpellStatistic::new(value).ok()),
            ),
            save: member_fact(defense, "save", &format!("{path}.defense.save"), |value| {
                let save = value.object()?;
                Some(SpellSavePatch {
                    statistic: member_fact(
                        save,
                        "statistic",
                        &format!("{path}.defense.save.statistic"),
                        |value| {
                            parse_string(value).and_then(|value| SpellStatistic::new(value).ok())
                        },
                    ),
                    basic: member_fact(
                        save,
                        "basic",
                        &format!("{path}.defense.save.basic"),
                        parse_bool,
                    ),
                })
            }),
        })
    })
}

fn parse_damage_patch_map(
    system: &[(String, SerializedSourceValue)],
    path: &str,
    unsupported_fields: &mut Vec<SpellUnsupportedPatchField>,
) -> SpellFact<SpellKeyedPatch<SpellDamagePatch>> {
    member_fact(system, "damage", &format!("{path}.damage"), |value| {
        let damage = value.object()?;
        Some(SpellKeyedPatch {
            members: damage
                .iter()
                .enumerate()
                .map(|(order, (key, value))| SpellKeyedPatchMember {
                    key: key.clone(),
                    authored_order: order as u32,
                    operation: match value {
                        SerializedSourceValue::Null => SpellKeyedPatchOperation::Delete,
                        _ => parse_damage_patch(
                            value,
                            &format!("{path}.damage.{key}"),
                            unsupported_fields,
                        )
                        .map(SpellKeyedPatchOperation::Merge)
                        .unwrap_or_else(|| {
                            SpellKeyedPatchOperation::Unsupported(unsupported_value(
                                value,
                                UnsupportedSourceReason::SourceFieldDrift,
                            ))
                        }),
                    },
                })
                .collect(),
        })
    })
}

fn parse_damage_patch(
    value: &SerializedSourceValue,
    path: &str,
    unsupported_fields: &mut Vec<SpellUnsupportedPatchField>,
) -> Option<SpellDamagePatch> {
    let damage = value.object()?;
    for (order, (key, value)) in damage.iter().enumerate() {
        if !matches!(
            key.as_str(),
            "applyMod" | "category" | "formula" | "kinds" | "materials" | "type"
        ) {
            unsupported_fields.push(SpellUnsupportedPatchField {
                field: atlas_record::SpellFormField::Damage,
                source_path: format!("{path}.{key}"),
                authored_key: key.clone(),
                authored_order: Some(order as u32),
                value: unsupported_value(value, UnsupportedSourceReason::SourceFieldDrift),
            });
        }
    }
    Some(SpellDamagePatch {
        apply_mod: member_fact(damage, "applyMod", &format!("{path}.applyMod"), parse_bool),
        category: member_fact(
            damage,
            "category",
            &format!("{path}.category"),
            parse_string,
        ),
        formula: member_fact(damage, "formula", &format!("{path}.formula"), parse_string),
        kinds: member_fact(damage, "kinds", &format!("{path}.kinds"), |value| {
            parse_string_array(value, Ok::<_, ()>)
        }),
        materials: member_fact(damage, "materials", &format!("{path}.materials"), |value| {
            parse_string_array(value, Ok::<_, ()>)
        }),
        damage_type: member_fact(damage, "type", &format!("{path}.type"), parse_string),
    })
}

fn parse_text_patch_map(
    object: &[(String, SerializedSourceValue)],
    key: &str,
    path: &str,
) -> SpellFact<SpellKeyedPatch<SpellTextPatch>> {
    member_fact(object, key, path, |value| {
        let values = value.object()?;
        Some(SpellKeyedPatch {
            members: values
                .iter()
                .enumerate()
                .map(|(order, (key, value))| SpellKeyedPatchMember {
                    key: key.clone(),
                    authored_order: order as u32,
                    operation: match value {
                        SerializedSourceValue::Null => SpellKeyedPatchOperation::Delete,
                        SerializedSourceValue::String(value) => {
                            SpellKeyedPatchOperation::Merge(SpellTextPatch {
                                value: known(value.clone()),
                            })
                        }
                        _ => SpellKeyedPatchOperation::Unsupported(unsupported_value(
                            value,
                            UnsupportedSourceReason::SourceFieldDrift,
                        )),
                    },
                })
                .collect(),
        })
    })
}

fn parse_ritual(
    system: &[(String, SerializedSourceValue)],
    system_path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<SpellRitual> {
    member_fact(
        system,
        "ritual",
        &format!("{system_path}.ritual"),
        |value| {
            let ritual = value.object()?;
            let mut unknown = unknown_members(
                ritual,
                &["primary", "secondary"],
                &format!("{system_path}.ritual"),
                SpellUnsupportedSourceField::RitualMember,
            );
            if let Some(primary) = unique_object(ritual, "primary") {
                unknown.extend(unknown_members(
                    primary,
                    &["check"],
                    &format!("{system_path}.ritual.primary"),
                    SpellUnsupportedSourceField::RitualMember,
                ));
            }
            if let Some(secondary) = unique_object(ritual, "secondary") {
                unknown.extend(unknown_members(
                    secondary,
                    &["casters", "checks"],
                    &format!("{system_path}.ritual.secondary"),
                    SpellUnsupportedSourceField::RitualMember,
                ));
            }
            if !unknown.is_empty() {
                notes.extend(unknown);
                return None;
            }
            Some(SpellRitual {
                primary_check: nested_fact(
                    ritual,
                    &["primary", "check"],
                    &format!("{system_path}.ritual.primary.check"),
                    parse_string,
                ),
                secondary_casters: nested_fact(
                    ritual,
                    &["secondary", "casters"],
                    &format!("{system_path}.ritual.secondary.casters"),
                    parse_u32,
                ),
                secondary_checks: nested_fact(
                    ritual,
                    &["secondary", "checks"],
                    &format!("{system_path}.ritual.secondary.checks"),
                    parse_string,
                ),
            })
        },
    )
}

fn parse_location(
    system: &[(String, SerializedSourceValue)],
    system_path: &str,
    notes: &mut Vec<SpellUnsupportedSourceFact>,
) -> SpellFact<ConsumableSpellLocation> {
    member_fact(
        system,
        "location",
        &format!("{system_path}.location"),
        |value| {
            let location = value.object()?;
            if has_unknown(location, &["value", "heightenedLevel"]) {
                notes.extend(unknown_members(
                    location,
                    &["value", "heightenedLevel"],
                    &format!("{system_path}.location"),
                    SpellUnsupportedSourceField::LocationMember,
                ));
                return None;
            }
            Some(ConsumableSpellLocation {
                value: member_fact(
                    location,
                    "value",
                    &format!("{system_path}.location.value"),
                    parse_string,
                ),
                heightened_rank: member_fact(
                    location,
                    "heightenedLevel",
                    &format!("{system_path}.location.heightenedLevel"),
                    parse_u8,
                ),
            })
        },
    )
}

fn parse_rules(
    system: &[(String, SerializedSourceValue)],
    system_path: &str,
) -> SpellFact<Vec<SpellRuleElement>> {
    parse_rules_at(system, "rules", &format!("{system_path}.rules"))
}

fn parse_rules_at(
    object: &[(String, SerializedSourceValue)],
    key: &str,
    path: &str,
) -> SpellFact<Vec<SpellRuleElement>> {
    member_fact(object, key, path, |value| {
        let SerializedSourceValue::Array(rules) = value else {
            return None;
        };
        Some(
            rules
                .iter()
                .enumerate()
                .map(|(order, value)| parse_rule(value, order as u32, &format!("{path}.{order}")))
                .collect(),
        )
    })
}

fn parse_rule(value: &SerializedSourceValue, authored_order: u32, path: &str) -> SpellRuleElement {
    let raw = value.compact_json();
    let Some(rule) = value.object() else {
        return unsupported_rule_element(value, authored_order, path, "<non-object>");
    };
    let key = unique_string(rule, "key")
        .unwrap_or("<missing>")
        .to_string();
    let parsed = match key.as_str() {
        "DamageDice"
            if !has_unknown(
                rule,
                &[
                    "key",
                    "selector",
                    "predicate",
                    "diceNumber",
                    "dieSize",
                    "damageType",
                    "hideIfDisabled",
                ],
            ) =>
        {
            SpellRule::DamageDice(SpellDamageDiceRule {
                selector: member_fact(rule, "selector", &format!("{path}.selector"), parse_string),
                predicate: parse_predicate(rule, path),
                dice_number: member_fact(
                    rule,
                    "diceNumber",
                    &format!("{path}.diceNumber"),
                    parse_string,
                ),
                die_size: member_fact(rule, "dieSize", &format!("{path}.dieSize"), parse_string),
                damage_type: member_fact(
                    rule,
                    "damageType",
                    &format!("{path}.damageType"),
                    parse_string,
                ),
                hide_if_disabled: member_fact(
                    rule,
                    "hideIfDisabled",
                    &format!("{path}.hideIfDisabled"),
                    parse_bool,
                ),
            })
        }
        "EphemeralEffect" if !has_unknown(rule, &["key", "predicate", "selectors", "uuid"]) => {
            SpellRule::EphemeralEffect(SpellEphemeralEffectRule {
                predicate: parse_predicate(rule, path),
                selectors: member_fact(rule, "selectors", &format!("{path}.selectors"), |value| {
                    parse_string_array(value, Ok::<_, ()>)
                }),
                uuid: member_fact(rule, "uuid", &format!("{path}.uuid"), parse_string),
            })
        }
        "DamageAlteration"
            if !has_unknown(
                rule,
                &[
                    "key",
                    "mode",
                    "predicate",
                    "property",
                    "selectors",
                    "slug",
                    "value",
                ],
            ) =>
        {
            SpellRule::DamageAlteration(SpellDamageAlterationRule {
                mode: member_fact(rule, "mode", &format!("{path}.mode"), parse_string),
                predicate: parse_predicate(rule, path),
                property: member_fact(rule, "property", &format!("{path}.property"), parse_string),
                selectors: member_fact(rule, "selectors", &format!("{path}.selectors"), |value| {
                    parse_string_array(value, Ok::<_, ()>)
                }),
                slug: member_fact(rule, "slug", &format!("{path}.slug"), parse_string),
                value: member_fact(rule, "value", &format!("{path}.value"), parse_string),
            })
        }
        "RollOption"
            if !has_unknown(
                rule,
                &[
                    "key",
                    "domain",
                    "label",
                    "option",
                    "placement",
                    "predicate",
                    "suboptions",
                    "toggleable",
                ],
            ) =>
        {
            SpellRule::RollOption(SpellRollOptionRule {
                domain: member_fact(rule, "domain", &format!("{path}.domain"), parse_string),
                label: member_fact(rule, "label", &format!("{path}.label"), parse_string),
                option: member_fact(rule, "option", &format!("{path}.option"), parse_string),
                placement: member_fact(
                    rule,
                    "placement",
                    &format!("{path}.placement"),
                    parse_string,
                ),
                predicate: parse_predicate(rule, path),
                suboptions: member_fact(
                    rule,
                    "suboptions",
                    &format!("{path}.suboptions"),
                    parse_suboptions,
                ),
                toggleable: member_fact(
                    rule,
                    "toggleable",
                    &format!("{path}.toggleable"),
                    parse_bool,
                ),
            })
        }
        "ItemAlteration"
            if !has_unknown(
                rule,
                &["key", "itemId", "mode", "predicate", "property", "value"],
            ) =>
        {
            SpellRule::ItemAlteration(SpellItemAlterationRule {
                item_id: member_fact(rule, "itemId", &format!("{path}.itemId"), parse_string),
                mode: member_fact(rule, "mode", &format!("{path}.mode"), parse_string),
                predicate: parse_predicate(rule, path),
                property: member_fact(rule, "property", &format!("{path}.property"), parse_string),
                value: member_fact(rule, "value", &format!("{path}.value"), parse_string),
            })
        }
        _ => SpellRule::Unsupported(SpellUnsupportedRule {
            authored_key: key.clone(),
            source_path: path.to_string(),
            value: unsupported_value(value, UnsupportedSourceReason::SourceFieldDrift),
        }),
    };
    SpellRuleElement {
        authored_order,
        source_path: path.to_string(),
        authored_key: key,
        authored_object_json: raw,
        rule: parsed,
    }
}

fn unsupported_rule_element(
    value: &SerializedSourceValue,
    authored_order: u32,
    path: &str,
    key: &str,
) -> SpellRuleElement {
    SpellRuleElement {
        authored_order,
        source_path: path.to_string(),
        authored_key: key.to_string(),
        authored_object_json: value.compact_json(),
        rule: SpellRule::Unsupported(SpellUnsupportedRule {
            authored_key: key.to_string(),
            source_path: path.to_string(),
            value: unsupported_value(value, UnsupportedSourceReason::SourceFieldDrift),
        }),
    }
}

fn parse_predicate(
    rule: &[(String, SerializedSourceValue)],
    path: &str,
) -> SpellFact<Vec<SpellRulePredicate>> {
    member_fact(rule, "predicate", &format!("{path}.predicate"), |value| {
        let SerializedSourceValue::Array(terms) = value else {
            return None;
        };
        Some(
            terms
                .iter()
                .enumerate()
                .map(|(order, term)| match term {
                    SerializedSourceValue::String(term) => SpellRulePredicate::Term(term.clone()),
                    SerializedSourceValue::Object(object)
                        if object.len() == 1 && object[0].0 == "or" =>
                    {
                        match parse_string_array(&object[0].1, Ok::<_, ()>) {
                            Some(terms) => SpellRulePredicate::Or(terms),
                            None => unsupported_predicate(term, path, order as u32),
                        }
                    }
                    _ => unsupported_predicate(term, path, order as u32),
                })
                .collect(),
        )
    })
}

fn unsupported_predicate(
    value: &SerializedSourceValue,
    path: &str,
    authored_order: u32,
) -> SpellRulePredicate {
    SpellRulePredicate::Unsupported(SpellUnsupportedRulePredicate {
        source_path: format!("{path}.predicate.{authored_order}"),
        authored_key: value
            .object()
            .and_then(|object| object.first())
            .map(|(key, _)| key.clone()),
        authored_order,
        value: unsupported_value(value, UnsupportedSourceReason::InvalidPredicate),
    })
}

fn parse_suboptions(value: &SerializedSourceValue) -> Option<Vec<SpellRuleSuboption>> {
    let SerializedSourceValue::Array(values) = value else {
        return None;
    };
    values
        .iter()
        .map(|value| {
            let object = value.object()?;
            (!has_unknown(object, &["label", "value"])).then(|| SpellRuleSuboption {
                label: member_fact(
                    object,
                    "label",
                    "system.rules.suboptions.label",
                    parse_string,
                ),
                value: member_fact(
                    object,
                    "value",
                    "system.rules.suboptions.value",
                    parse_string,
                ),
            })
        })
        .collect()
}

fn member_fact<T>(
    object: &[(String, SerializedSourceValue)],
    key: &str,
    _path: &str,
    parse: impl FnOnce(&SerializedSourceValue) -> Option<T>,
) -> SpellFact<T> {
    let values = members(object, key);
    match values.as_slice() {
        [] => FactValue::Missing,
        [SerializedSourceValue::Null] => FactValue::Null,
        [value] => parse(value)
            .map(known)
            .unwrap_or_else(|| unsupported(value, UnsupportedSourceReason::SourceFieldDrift)),
        _ => unsupported_duplicate(key, &values),
    }
}

fn provenance_fact(
    object: &[(String, SerializedSourceValue)],
    key: &str,
) -> FactValue<UnsupportedSourceValue> {
    let values = members(object, key);
    match values.as_slice() {
        [] => FactValue::Missing,
        [SerializedSourceValue::Null] => FactValue::Null,
        [value] => FactValue::Value(unsupported_value(
            value,
            UnsupportedSourceReason::SourceFieldDrift,
        )),
        _ => FactValue::Value(UnsupportedSourceValue {
            shape: UnsupportedSourceShape::Array,
            value: duplicate_value(key, &values),
            reason: UnsupportedSourceReason::SourceFieldDrift,
        }),
    }
}

fn nested_fact<T>(
    object: &[(String, SerializedSourceValue)],
    segments: &[&str],
    path: &str,
    parse: impl FnOnce(&SerializedSourceValue) -> Option<T>,
) -> SpellFact<T> {
    let Some((last, parents)) = segments.split_last() else {
        return FactValue::Missing;
    };
    let mut current = object;
    for parent in parents {
        let values = members(current, parent);
        match values.as_slice() {
            [] => return FactValue::Missing,
            [SerializedSourceValue::Null] => return FactValue::Null,
            [SerializedSourceValue::Object(value)] => current = value,
            [value] => return unsupported(value, UnsupportedSourceReason::SourceFieldDrift),
            _ => return unsupported_duplicate(parent, &values),
        }
    }
    member_fact(current, last, path, parse)
}

fn malformed(
    identity: &SourceIdentity,
    path: &str,
    expected: &str,
    object: &[(String, SerializedSourceValue)],
) -> SourceDiagnostic {
    SourceDiagnostic::new(
        SourceDiagnosticKind::MalformedShape,
        identity,
        path,
        expected,
        SerializedSourceValue::Object(SerializedSourceObject::from_fields(object.to_vec()))
            .compact_json(),
    )
}

fn parse_string(value: &SerializedSourceValue) -> Option<String> {
    match value {
        SerializedSourceValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn parse_bool(value: &SerializedSourceValue) -> Option<bool> {
    match value {
        SerializedSourceValue::Boolean(value) => Some(*value),
        _ => None,
    }
}

fn parse_i64(value: &SerializedSourceValue) -> Option<i64> {
    match value {
        SerializedSourceValue::Number(value) => value.as_i64(),
        _ => None,
    }
}

fn parse_u32(value: &SerializedSourceValue) -> Option<u32> {
    match value {
        SerializedSourceValue::Number(value) => value.as_u64()?.try_into().ok(),
        _ => None,
    }
}

fn parse_u8(value: &SerializedSourceValue) -> Option<u8> {
    parse_u32(value)?.try_into().ok()
}

fn parse_string_array<T, E>(
    value: &SerializedSourceValue,
    convert: impl Fn(String) -> Result<T, E>,
) -> Option<Vec<T>> {
    let SerializedSourceValue::Array(values) = value else {
        return None;
    };
    values
        .iter()
        .map(|value| parse_string(value).and_then(|value| convert(value).ok()))
        .collect()
}

fn known<T>(value: T) -> SpellFact<T> {
    FactValue::Value(SpellSourceValue::Known(value))
}

fn unsupported<T>(value: &SerializedSourceValue, reason: UnsupportedSourceReason) -> SpellFact<T> {
    FactValue::Value(SpellSourceValue::Unsupported(unsupported_value(
        value, reason,
    )))
}

fn unsupported_duplicate<T>(key: &str, values: &[&SerializedSourceValue]) -> SpellFact<T> {
    FactValue::Value(SpellSourceValue::Unsupported(UnsupportedSourceValue {
        shape: UnsupportedSourceShape::Array,
        value: duplicate_value(key, values),
        reason: UnsupportedSourceReason::SourceFieldDrift,
    }))
}

fn duplicate_value(key: &str, values: &[&SerializedSourceValue]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .enumerate()
            .map(|(order, value)| format!(
                "{{\"key\":{},\"authored_order\":{order},\"value\":{}}}",
                serde_json::Value::String(key.to_string()),
                value.compact_json()
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn unsupported_value(
    value: &SerializedSourceValue,
    reason: UnsupportedSourceReason,
) -> UnsupportedSourceValue {
    UnsupportedSourceValue {
        shape: value.shape(),
        value: value.compact_json(),
        reason,
    }
}

fn unsupported_note(
    field: SpellUnsupportedSourceField,
    path: &str,
    key: &str,
    order: Option<u32>,
    value: &SerializedSourceValue,
) -> SpellUnsupportedSourceFact {
    SpellUnsupportedSourceFact {
        field,
        source_path: path.to_string(),
        authored_key: key.to_string(),
        authored_order: order,
        value: unsupported_value(value, UnsupportedSourceReason::SourceFieldDrift),
    }
}

fn unknown_members(
    object: &[(String, SerializedSourceValue)],
    known: &[&str],
    path: &str,
    field: SpellUnsupportedSourceField,
) -> Vec<SpellUnsupportedSourceFact> {
    object
        .iter()
        .enumerate()
        .filter(|(_, (key, _))| !known.contains(&key.as_str()))
        .map(|(order, (key, value))| {
            unsupported_note(
                field,
                &format!("{path}.{key}"),
                key,
                Some(order as u32),
                value,
            )
        })
        .collect()
}

fn has_unknown(object: &[(String, SerializedSourceValue)], known: &[&str]) -> bool {
    object.iter().any(|(key, _)| !known.contains(&key.as_str()))
}

fn has_any_member(object: &[(String, SerializedSourceValue)], keys: &[&str]) -> bool {
    object.iter().any(|(key, _)| keys.contains(&key.as_str()))
}

fn unsupported_patch_for_all_fields(
    source_path: &str,
    authored_key: &str,
    authored_order: Option<u32>,
    value: &SerializedSourceValue,
) -> Vec<SpellUnsupportedPatchField> {
    [
        atlas_record::SpellFormField::Classification,
        atlas_record::SpellFormField::Casting,
        atlas_record::SpellFormField::Targeting,
        atlas_record::SpellFormField::Defense,
        atlas_record::SpellFormField::Damage,
        atlas_record::SpellFormField::Duration,
        atlas_record::SpellFormField::Heightening,
        atlas_record::SpellFormField::Rules,
    ]
    .into_iter()
    .map(|field| SpellUnsupportedPatchField {
        field,
        source_path: source_path.to_string(),
        authored_key: authored_key.to_string(),
        authored_order,
        value: unsupported_value(value, UnsupportedSourceReason::SourceFieldDrift),
    })
    .collect()
}

fn retain_patch_nested_unknown(
    system: &[(String, SerializedSourceValue)],
    path: &str,
    unsupported: &mut Vec<SpellUnsupportedPatchField>,
) {
    for (root, known, field) in [
        (
            "level",
            &["value"][..],
            atlas_record::SpellFormField::Classification,
        ),
        (
            "traits",
            &["traditions", "value"][..],
            atlas_record::SpellFormField::Classification,
        ),
        (
            "time",
            &["value"][..],
            atlas_record::SpellFormField::Casting,
        ),
        (
            "cost",
            &["value"][..],
            atlas_record::SpellFormField::Casting,
        ),
        (
            "target",
            &["value"][..],
            atlas_record::SpellFormField::Targeting,
        ),
        (
            "range",
            &["value"][..],
            atlas_record::SpellFormField::Targeting,
        ),
        (
            "duration",
            &["sustained", "value"][..],
            atlas_record::SpellFormField::Duration,
        ),
        (
            "heightening",
            &["area", "damage", "interval", "type"][..],
            atlas_record::SpellFormField::Heightening,
        ),
    ] {
        let Some(object) = unique_object(system, root) else {
            continue;
        };
        unsupported.extend(
            object
                .iter()
                .enumerate()
                .filter(|(_, (key, _))| !known.contains(&key.as_str()))
                .map(|(order, (key, value))| SpellUnsupportedPatchField {
                    field,
                    source_path: format!("{path}.{root}.{key}"),
                    authored_key: key.clone(),
                    authored_order: Some(order as u32),
                    value: unsupported_value(value, UnsupportedSourceReason::SourceFieldDrift),
                }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standalone(source: &[u8], key: &str, path: &str) -> SpellItemSource {
        let source =
            super::super::parse_serialized_source_object(source).expect("lossless source object");
        let parsed = parse_spell_document_source(&source, &SourceIdentity::new(key, path))
            .expect("ordered spell source")
            .expect("spell source");
        let SpellDocumentSource::Standalone(spell) = parsed else {
            panic!("standalone spell")
        };
        spell
    }

    #[test]
    fn ordered_maps_preserve_duplicate_keys_before_serde_json_map_conversion() {
        let source = br#"{"_id":"s","name":"S","type":"spell","system":{"damage":{"a":{"formula":"1d6","type":"fire"},"a":{"formula":"2d6","type":"cold"}},"rules":[]}}"#;
        let spell = standalone(source, "spells:s", "packs/spells/s.json");
        let damage = spell
            .damage
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("damage members");
        assert_eq!(
            damage
                .iter()
                .map(|member| (member.key.as_str(), member.authored_order))
                .collect::<Vec<_>>(),
            vec![("a", 0), ("a", 1)]
        );
        assert_eq!(
            damage[0]
                .value
                .formula
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("1d6")
        );
        assert_eq!(
            damage[1]
                .value
                .formula
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("2d6")
        );
    }

    #[test]
    fn duplicate_scalar_member_remains_exact_typed_unsupported() {
        let spell = standalone(
            br#"{"_id":"s","name":"S","type":"spell","system":{"duration":{"sustained":false,"value":"1 round"},"duration":{"sustained":false,"value":"2 rounds"},"rules":[]}}"#,
            "spells:s",
            "packs/spells/s.json",
        );
        let duration = spell
            .duration
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("typed duration owner");
        let FactValue::Value(SpellSourceValue::Unsupported(value)) = &duration.value else {
            panic!("duplicate duration value must remain typed unsupported")
        };
        assert_eq!(
            value.value,
            r#"[{"key":"duration","authored_order":0,"value":{"sustained":false,"value":"1 round"}},{"key":"duration","authored_order":1,"value":{"sustained":false,"value":"2 rounds"}}]"#
        );
    }

    #[test]
    fn raw_source_parser_preserves_nonlexical_key_order() {
        let spell = standalone(
            include_bytes!(
                "../../../tests/fixtures/foundry-source/spell-source-contract/packs/spells/nonlexical-duplicate.json"
            ),
            "spells-srd:nonlexicalSpellMaps",
            "fixtures/nonlexical-spell-maps.json",
        );

        let damage = spell
            .damage
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("ordered damage map");
        assert_eq!(
            damage
                .iter()
                .map(|member| (member.key.as_str(), member.authored_order))
                .collect::<Vec<_>>(),
            vec![("zeta-damage", 0), ("alpha-damage", 1), ("zeta-damage", 2),]
        );
        assert_eq!(
            damage
                .iter()
                .map(|member| {
                    member
                        .value
                        .formula
                        .as_value()
                        .and_then(SpellSourceValue::as_known)
                        .map(String::as_str)
                })
                .collect::<Vec<_>>(),
            [Some("1d6"), Some("1d4"), Some("2d6")]
        );

        let heightening = spell
            .heightening
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("fixed heightening");
        let SpellHeightening::Fixed(layers) = heightening else {
            panic!("fixed heightening");
        };
        assert_eq!(
            layers
                .iter()
                .map(|layer| (layer.key.as_str(), layer.authored_order))
                .collect::<Vec<_>>(),
            vec![("8", 0), ("5", 1)]
        );

        let overlays = spell
            .overlays
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("ordered overlays");
        assert_eq!(
            overlays
                .iter()
                .map(|overlay| (overlay.key.as_str(), overlay.authored_order))
                .collect::<Vec<_>>(),
            vec![("zeta-overlay", 0), ("alpha-overlay", 1)]
        );
    }

    #[test]
    fn heal_and_rime_preserve_authored_map_identity_and_patch_semantics() {
        let heal = standalone(
            br#"{"_id":"rfZpqmj0AIIdkVIs","name":"Heal","type":"spell","system":{"area":null,"cost":{"value":""},"counteraction":false,"damage":{"0":{"applyMod":false,"category":null,"formula":"1d8","kinds":["damage","healing"],"materials":[],"type":"vitality"}},"defense":{"save":{"basic":true,"statistic":"fortitude"}},"duration":{"sustained":false,"value":""},"heightening":{"damage":{"0":"1d8"},"interval":1,"type":"interval"},"level":{"value":1},"overlays":{"7qdtetowq348s9oc":{"overlayType":"override","sort":1,"system":{"range":{"value":"touch"},"time":{"value":"1"}}}},"range":{"value":"varies"},"requirements":"","ritual":null,"rules":[],"target":{"value":"1 willing living creature or 1 undead"},"time":{"value":"1 to 3"},"traits":{"rarity":"common","traditions":["divine","primal"],"value":["healing","manipulate","vitality"]}}}"#,
            "spells-srd:rfZpqmj0AIIdkVIs",
            "packs/spells/1st-rank/heal.json",
        );
        let range = heal
            .targeting
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .and_then(|targeting| targeting.range.as_value())
            .and_then(SpellSourceValue::as_known)
            .expect("authored range");
        assert_eq!(range.authored_text, "varies");
        assert_eq!(range.numeric, None);
        let overlays = heal
            .overlays
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("overlay map");
        assert_eq!(overlays[0].key, "7qdtetowq348s9oc");
        assert_eq!(overlays[0].authored_order, 0);
        assert_eq!(overlays[0].source_id, FactValue::Missing);

        let rime = standalone(
            br#"{"_id":"Popa5umI3H33levx","name":"Rime Slick","type":"spell","system":{"area":{"type":"burst","value":15},"cost":{"value":""},"counteraction":false,"damage":{"0":{"applyMod":false,"category":null,"formula":"2d4","kinds":["damage"],"materials":[],"type":"cold"}},"defense":{"save":{"basic":true,"statistic":"reflex"}},"duration":{"sustained":false,"value":"1 minute"},"heightening":{"levels":{"5":{"area":{"areaType":"burst","type":"burst","value":30},"damage":{"0":{"applyMod":false,"category":null,"formula":"8d4","materials":[],"type":"cold"}}},"8":{"area":{"areaType":"burst","type":"burst","value":60}}},"type":"fixed"},"level":{"value":2},"range":{"value":"60 feet"},"requirements":"","rules":[],"target":{"value":""},"time":{"value":"2"},"traits":{"rarity":"uncommon","traditions":["arcane","primal"],"value":["cold","concentrate","manipulate"]}}}"#,
            "spells-srd:Popa5umI3H33levx",
            "packs/spells/2nd-rank/rime-slick.json",
        );
        let SpellHeightening::Fixed(layers) = rime
            .heightening
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("fixed heightening")
        else {
            panic!("fixed heightening")
        };
        assert_eq!(
            layers
                .iter()
                .map(|layer| layer.key.as_str())
                .collect::<Vec<_>>(),
            vec!["5", "8"]
        );
        let area = layers[0]
            .patch
            .targeting
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .and_then(|targeting| targeting.area.as_value())
            .and_then(SpellSourceValue::as_known)
            .expect("area patch");
        let legacy = area
            .legacy_area_type
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("legacy area evidence");
        assert_eq!(
            legacy.source_path,
            "system.heightening.levels.5.area.areaType"
        );
        assert_eq!(legacy.authored_key, "areaType");
        assert_eq!(legacy.value.as_str(), "burst");
    }

    #[test]
    fn heal_overlay_map_order_and_explicit_sort_remain_independent() {
        let heal = standalone(
            br#"{"_id":"rfZpqmj0AIIdkVIs","name":"Heal","type":"spell","system":{"level":{"value":1},"overlays":{"37gy7l19tik74o4s":{"_id":"37gy7l19tik74o4s","name":"Heal (vs. Living)","overlayType":"override","sort":2,"system":{"range":{"value":"30 feet"}}},"7qdtetowq348s9oc":{"_id":"7qdtetowq348s9oc","overlayType":"override","sort":1,"system":{"range":{"value":"touch"}}},"7vbvdrv2cl87sqta":{"_id":"7vbvdrv2cl87sqta","overlayType":"override","sort":4,"system":{"area":{"type":"emanation","value":30}}},"lfxcoz2d3f8j2zq1":{"_id":"lfxcoz2d3f8j2zq1","name":"Heal (vs. Undead)","overlayType":"override","sort":3,"system":{"target":{"value":"1 undead"}}}},"rules":[],"traits":{"rarity":"common","traditions":["divine","primal"],"value":["healing","manipulate","vitality"]}}}"#,
            "spells-srd:rfZpqmj0AIIdkVIs",
            "packs/spells/1st-rank/heal.json",
        );
        let overlays = heal
            .overlays
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("Heal overlays");
        assert_eq!(
            overlays
                .iter()
                .map(|overlay| (overlay.key.as_str(), overlay.authored_order))
                .collect::<Vec<_>>(),
            vec![
                ("37gy7l19tik74o4s", 0),
                ("7qdtetowq348s9oc", 1),
                ("7vbvdrv2cl87sqta", 2),
                ("lfxcoz2d3f8j2zq1", 3),
            ]
        );
        assert_eq!(
            overlays
                .iter()
                .map(|overlay| {
                    overlay
                        .sort
                        .as_value()
                        .and_then(SpellSourceValue::as_known)
                        .copied()
                })
                .collect::<Vec<_>>(),
            vec![Some(2), Some(1), Some(4), Some(3)]
        );
    }

    #[test]
    fn explicit_ritual_and_dual_defense_are_typed_without_prose_inference() {
        let ritual = standalone(
            br#"{"_id":"HmKajQS0DP23bipp","name":"Planar Displacement","type":"spell","system":{"area":{"type":"burst","value":20},"cost":{"value":"rare incense, precious metals, and purified chalk worth 500 gp"},"counteraction":false,"damage":{},"defense":null,"description":{"value":"<p>Heightened prose is content only.</p>"},"duration":{"sustained":false,"value":""},"level":{"value":7},"range":{"value":"20 feet"},"requirements":"planar key for the destination plane used as a locus","ritual":{"primary":{"check":"Arcana (master), Nature (master), Occultism (master), or Religion (master)"},"secondary":{"casters":2,"checks":"Lore (related to the destination plane), Survival"}},"rules":[],"target":{"value":""},"time":{"value":"1 day"},"traits":{"rarity":"uncommon","traditions":[],"value":["teleportation"]}}}"#,
            "spells-srd:HmKajQS0DP23bipp",
            "packs/spells/ritual/planar-displacement.json",
        );
        assert!(matches!(ritual.heightening, FactValue::Missing));
        let ritual = ritual
            .ritual
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("explicit ritual root");
        assert_eq!(
            ritual
                .secondary_casters
                .as_value()
                .and_then(SpellSourceValue::as_known),
            Some(&2)
        );

        let strike = standalone(
            br#"{"_id":"x9RIFhquazom4p02","name":"Deity's Strike","type":"spell","system":{"area":null,"cost":{"value":""},"counteraction":false,"damage":{"0":{"applyMod":false,"category":null,"formula":"7d12","kinds":["damage"],"type":"force"}},"defense":{"passive":{"statistic":"ac"},"save":{"basic":true,"statistic":"reflex"}},"duration":{"sustained":false,"value":""},"heightening":{"levels":{"9":{"damage":{"0":{"applyMod":false,"category":null,"formula":"8d12","type":"force"}}}},"type":"fixed"},"level":{"value":7},"range":{"value":"500 feet"},"requirements":"You have a deity.","rules":[],"target":{"value":"1 creature"},"time":{"value":"2"},"traits":{"rarity":"common","traditions":["divine"],"value":["concentrate","force","manipulate"]}}}"#,
            "spells-srd:x9RIFhquazom4p02",
            "packs/spells/7th-rank/deitys-strike.json",
        );
        let defense = strike
            .defense
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("dual defense");
        assert_eq!(
            defense
                .passive
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(SpellStatistic::as_str),
            Some("ac")
        );
        assert_eq!(
            defense
                .save
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .and_then(|save| save.statistic.as_value())
                .and_then(SpellSourceValue::as_known)
                .map(SpellStatistic::as_str),
            Some("reflex")
        );
    }

    #[test]
    fn rule_order_overlay_heightening_and_legacy_trait_labels_remain_exact() {
        let qi = standalone(
            br#"{"_id":"oo7YcRC2gcez81PV","name":"Qi Blast","type":"spell","system":{"area":{"type":"cone","value":15},"cost":{"value":""},"counteraction":false,"damage":{},"defense":null,"duration":{"sustained":false,"value":""},"level":{"value":3},"overlays":{"6wf5l1kiwcdtnx6s":{"overlayType":"override","sort":3,"system":{"heightening":{"area":0,"damage":{"0":"2d6"},"interval":1,"type":"interval"}}}},"range":{"value":""},"requirements":"","rules":[{"domain":"all","key":"RollOption","label":"PF2E.SpecificRule.Monk.QiSpells.HeavensThunder.RollOptionLabel","option":"heavens-thunder","placement":"spellcasting","predicate":["self:effect:heavens-thunder"],"suboptions":[{"label":"PF2E.TraitElectricity","value":"electricity"}],"toggleable":true},{"key":"DamageAlteration","mode":"override","predicate":["heavens-thunder"],"property":"damage-type","selectors":["{item|id}-damage"],"value":"{item|flags.pf2e.rulesSelections.heavensThunder}"},{"itemId":"{item|id}","key":"ItemAlteration","mode":"remove","predicate":["heavens-thunder"],"property":"traits","value":"force"}],"target":{"value":""},"time":{"value":"1 to 3"},"traits":{"rarity":"uncommon","traditions":[],"value":["focus","force"]}}}"#,
            "spells-srd:oo7YcRC2gcez81PV",
            "packs/spells/focus/qi-blast.json",
        );
        let rules = qi
            .rules
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("rules");
        assert_eq!(
            rules
                .iter()
                .map(|rule| rule.authored_key.as_str())
                .collect::<Vec<_>>(),
            vec!["RollOption", "DamageAlteration", "ItemAlteration"]
        );
        assert_eq!(rules[0].authored_order, 0);
        assert!(matches!(rules[0].rule, SpellRule::RollOption(_)));
        let overlays = qi
            .overlays
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("overlays");
        let heightening = overlays[0]
            .patch
            .heightening
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("overlay heightening");
        assert_eq!(
            heightening
                .kind
                .as_value()
                .and_then(SpellSourceValue::as_known),
            Some(&SpellHeighteningType::Interval)
        );
        assert_eq!(
            heightening
                .interval
                .as_value()
                .and_then(SpellSourceValue::as_known),
            Some(&1)
        );
        assert_eq!(
            heightening
                .area
                .as_value()
                .and_then(SpellSourceValue::as_known),
            Some(&0)
        );

        let phase = standalone(
            br#"{"_id":"5gophZ4AOKW4VW27","name":"Phase Bolt","type":"spell","system":{"area":null,"cost":{"value":""},"counteraction":false,"damage":{"nqp6byhfcwbvtstj":{"applyMod":false,"category":null,"formula":"3d4","kinds":["damage"],"materials":[],"type":"piercing"}},"defense":null,"duration":{"sustained":false,"value":""},"heightening":{"damage":{"nqp6byhfcwbvtstj":"1d4"},"interval":1,"type":"interval"},"level":{"value":1},"range":{"value":"30 feet"},"requirements":"","rules":[{"key":"EphemeralEffect","predicate":["item:slug:phase-bolt"],"selectors":["spell-attack-roll"],"uuid":"Compendium.pf2e.spell-effects.Item.Spell Effect: Phase Bolt"}],"target":{"value":"1 creature"},"time":{"value":"2"},"traits":{"rarity":"common","traditions":["arcane","occult"],"value":["attack","cantrip","concentrate","manipulate"]}}}"#,
            "spells-srd:5gophZ4AOKW4VW27",
            "packs/spells/cantrip/phase-bolt.json",
        );
        let phase_rules = phase
            .rules
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("phase bolt rule");
        assert!(matches!(phase_rules[0].rule, SpellRule::EphemeralEffect(_)));
        assert_eq!(
            phase_rules[0].authored_object_json,
            r#"{"key":"EphemeralEffect","predicate":["item:slug:phase-bolt"],"selectors":["spell-attack-roll"],"uuid":"Compendium.pf2e.spell-effects.Item.Spell Effect: Phase Bolt"}"#
        );

        let legacy = standalone(
            br#"{"_id":"uToa7ksKAzmEpkKC","name":"Admonishing Ray","type":"spell","system":{"counteraction":false,"duration":{"sustained":false,"value":""},"level":{"value":1},"rules":[],"traits":{"rarity":"common","selected":{"attack":"Attack","necromancy":"Necromancy","nonlethal":"Nonlethal"},"traditions":["arcane","divine"],"value":["attack","concentrate","manipulate","nonlethal"]}}}"#,
            "spells-srd:uToa7ksKAzmEpkKC",
            "packs/spells/1st-rank/admonishing-ray.json",
        );
        let selected = legacy
            .unsupported_notes
            .iter()
            .filter(|note| note.field == SpellUnsupportedSourceField::LegacyTraitSelection)
            .collect::<Vec<_>>();
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[1].source_path, "system.traits.selected.necromancy");
        assert_eq!(selected[1].value.value, "\"Necromancy\"");
    }

    #[test]
    fn invalid_publication_license_and_rarity_remain_exact_typed_unsupported() {
        let source = super::super::parse_serialized_source_object(
            br#"{"_id":"metadata-parent","name":"Metadata Parent","type":"consumable","system":{"spell":{"_id":"metadata-child","name":"Metadata Drift","type":"spell","system":{"publication":{"license":" "},"rules":[],"traits":{"rarity":"mythic","traditions":[],"value":[]}}}}}"#,
        )
        .expect("lossless source object");
        let parsed = parse_spell_document_source(
            &source,
            &SourceIdentity::new(
                "equipment-srd:metadata-parent",
                "fixtures/metadata-drift.json",
            ),
        )
        .expect("ordered consumable source")
        .expect("consumable spell child");
        let SpellDocumentSource::ConsumableChild(child) = parsed else {
            panic!("consumable spell child")
        };
        let drift = child.spell;

        let FactValue::Value(SpellSourceValue::Unsupported(license)) = &drift.publication_license
        else {
            panic!("invalid publication license must be typed unsupported");
        };
        assert_eq!(license.value, "\" \"");

        let FactValue::Value(SpellSourceValue::Unsupported(rarity)) = &drift.rarity else {
            panic!("unknown rarity must be typed unsupported");
        };
        assert_eq!(rarity.value, "\"mythic\"");
    }
}
