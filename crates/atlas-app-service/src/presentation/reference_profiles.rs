//! Typed, read-only profiles. Source presence and choices survive projection.
use super::*;
use atlas_foundry_model::generated::*;
use serde::Serialize;

fn label(value: &str) -> String {
    let text = value.replace(['-', '_'], " ");
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_uppercase().collect::<String>() + chars.as_str()
    })
}
fn labels<T: Serialize>(value: SourceFieldView<'_, &Vec<T>>) -> FactView<Vec<String>> {
    let mut result = identifiers(value);
    if let Some(values) = &mut result.value {
        for value in values {
            *value = label(value);
        }
    }
    result
}
fn attribute_names(
    value: SourceFieldView<'_, &Vec<PhysicalSystemSourceApexAttribute>>,
) -> FactView<Vec<String>> {
    fact(value.map(|values| {
        values
            .iter()
            .map(|v| {
                match v {
                    PhysicalSystemSourceApexAttribute::Str => "Strength",
                    PhysicalSystemSourceApexAttribute::Dex => "Dexterity",
                    PhysicalSystemSourceApexAttribute::Con => "Constitution",
                    PhysicalSystemSourceApexAttribute::Int => "Intelligence",
                    PhysicalSystemSourceApexAttribute::Wis => "Wisdom",
                    PhysicalSystemSourceApexAttribute::Cha => "Charisma",
                }
                .to_owned()
            })
            .collect()
    }))
}
fn attribute_choices(
    value: SourceFieldView<'_, &AncestrySystemSourceBoosts>,
) -> FactView<Vec<AttributeChoiceView>> {
    fact(value.map(|slots| {
        slots
            .entries
            .iter()
            .map(|(_, slot)| AttributeChoiceView {
                free_choice: slot.value.as_value().is_some_and(|choices| {
                    use PhysicalSystemSourceApexAttribute::*;
                    [Str, Dex, Con, Int, Wis, Cha]
                        .iter()
                        .all(|attribute| choices.contains(attribute))
                }),
                choices: attribute_names((&slot.value).into()),
            })
            .collect()
    }))
}
fn proficiency(value: SourceFieldView<'_, &serde_json::Number>) -> FactView<String> {
    fact(value.and_then(|value| match value.as_u64() {
        Some(rank @ 0..=4) => SourceFieldView::Value(
            ["Untrained", "Trained", "Expert", "Master", "Legendary"][rank as usize].into(),
        ),
        _ => SourceFieldView::ProjectionInvalid {
            source_path: "$",
            reason: "Proficiency rank must be an integer from 0 through 4",
        },
    }))
}
fn levels(
    value: SourceFieldView<'_, &ClassSystemSourceAncestryFeatLevelsSourceFromSchema>,
) -> FactView<Vec<u8>> {
    fact(value.and_then(|v| (&v.value).into()).and_then(|v| {
        v.iter()
            .map(|v| v.as_u64().and_then(|v| u8::try_from(v).ok()))
            .collect::<Option<Vec<_>>>()
            .map(SourceFieldView::Value)
            .unwrap_or(SourceFieldView::ProjectionInvalid {
                source_path: "$",
                reason: "Advancement levels cannot be represented",
            })
    }))
}
fn grants(
    value: SourceFieldView<'_, &AncestrySystemSourceItems>,
) -> FactView<Vec<ReferenceGrantView>> {
    fact(value.map(|items| {
        items
            .entries
            .iter()
            .map(|(_, item)| ReferenceGrantView {
                name: text((&item.name).into()),
                target: text((&item.uuid).into()),
                level: number((&item.level).into()),
                navigation: None,
            })
            .collect()
    }))
}
pub(super) fn item(item: ItemSourceView<'_>) -> Option<RecordBodyView> {
    Some(match item {
        ItemSourceView::HeritageSource(source) => {
            RecordBodyView::Heritage(Box::new(HeritageReferenceView {
                ancestry: text(
                    SourceFieldView::from(&source.system)
                        .and_then(|s| (&s.ancestry).into())
                        .and_then(|a| (&a.name).into()),
                ),
            }))
        }
        ItemSourceView::AncestrySource(source) => {
            let s = SourceFieldView::from(&source.system);
            let languages = s.and_then(|s| (&s.languages).into());
            let additional = s.and_then(|s| (&s.additional_languages).into());
            RecordBodyView::Ancestry(Box::new(AncestryReferenceView {
                hit_points: number(s.and_then(|s| (&s.hp).into())),
                speed: number(s.and_then(|s| (&s.speed).into())),
                size: fact(
                    s.and_then(|s| (&s.size).into())
                        .map(super::facts::size_label),
                ),
                vision: fact(s.and_then(|s| (&s.vision).into()).map(|v| {
                    match v {
                        AncestryVision::Normal => "Normal",
                        AncestryVision::Darkvision => "Darkvision",
                        AncestryVision::LowLightVision => "Low-light vision",
                    }
                    .into()
                })),
                hands: number(s.and_then(|s| (&s.hands).into())),
                reach: number(s.and_then(|s| (&s.reach).into())),
                boosts: attribute_choices(s.and_then(|s| (&s.boosts).into())),
                flaws: attribute_choices(s.and_then(|s| (&s.flaws).into())),
                languages: labels(languages.and_then(|l| (&l.value).into())),
                language_details: text(languages.and_then(|l| (&l.custom).into())),
                additional_languages: labels(additional.and_then(|l| (&l.value).into())),
                additional_language_count: number(additional.and_then(|l| (&l.count).into())),
                additional_language_details: text(additional.and_then(|l| (&l.custom).into())),
                grants: grants(s.and_then(|s| (&s.items).into())),
            }))
        }
        ItemSourceView::ClassSource(source) => {
            let s = SourceFieldView::from(&source.system);
            let attacks = s.and_then(|s| (&s.attacks).into());
            let defenses = s.and_then(|s| (&s.defenses).into());
            let saves = s.and_then(|s| (&s.saving_throws).into());
            let skills = s.and_then(|s| (&s.trained_skills).into());
            let other = attacks.and_then(|a| (&a.other).into());
            RecordBodyView::Class(Box::new(ClassReferenceView {
                hit_points: number(s.and_then(|s| (&s.hp).into())),
                key_attributes: attribute_names(
                    s.and_then(|s| (&s.key_ability).into())
                        .and_then(|a| (&a.value).into()),
                ),
                perception: proficiency(s.and_then(|s| (&s.perception).into())),
                fortitude: proficiency(saves.and_then(|s| (&s.fortitude).into())),
                reflex: proficiency(saves.and_then(|s| (&s.reflex).into())),
                will: proficiency(saves.and_then(|s| (&s.will).into())),
                simple_weapons: proficiency(attacks.and_then(|a| (&a.simple).into())),
                martial_weapons: proficiency(attacks.and_then(|a| (&a.martial).into())),
                advanced_weapons: proficiency(attacks.and_then(|a| (&a.advanced).into())),
                unarmed: proficiency(attacks.and_then(|a| (&a.unarmed).into())),
                other_weapons: text(other.and_then(|a| (&a.name).into())),
                other_weapon_rank: proficiency(other.and_then(|a| (&a.rank).into())),
                unarmored: proficiency(defenses.and_then(|d| (&d.unarmored).into())),
                light_armor: proficiency(defenses.and_then(|d| (&d.light).into())),
                medium_armor: proficiency(defenses.and_then(|d| (&d.medium).into())),
                heavy_armor: proficiency(defenses.and_then(|d| (&d.heavy).into())),
                spellcasting: proficiency(s.and_then(|s| (&s.spellcasting).into())),
                trained_skills: labels(skills.and_then(|s| (&s.value).into())),
                additional_skills: number(skills.and_then(|s| (&s.additional).into())),
                grants: grants(s.and_then(|s| (&s.items).into())),
                ancestry_feat_levels: levels(s.and_then(|s| (&s.ancestry_feat_levels).into())),
                class_feat_levels: levels(s.and_then(|s| (&s.class_feat_levels).into())),
                general_feat_levels: levels(s.and_then(|s| (&s.general_feat_levels).into())),
                skill_feat_levels: levels(s.and_then(|s| (&s.skill_feat_levels).into())),
                skill_increase_levels: levels(s.and_then(|s| (&s.skill_increase_levels).into())),
            }))
        }
        ItemSourceView::DeitySource(source) => {
            let s = SourceFieldView::from(&source.system);
            let domains = s.and_then(|s| (&s.domains).into());
            RecordBodyView::Deity(Box::new(DeityReferenceView {
                attributes: attribute_names(s.and_then(|s| (&s.attribute).into())),
                fonts: fact(
                    item.deity_fonts()
                        .map(|values| values.iter().map(|v| label(v)).collect()),
                ),
                sanctification: fact(s.and_then(|s| (&s.sanctification).into()).map(|v| {
                    DeitySanctificationView {
                        requirement: identifier((&v.modal).into()),
                        choices: labels((&v.what).into()),
                    }
                })),
                skills: labels(s.and_then(|s| (&s.skill).into())),
                primary_domains: labels(domains.and_then(|d| (&d.primary).into())),
                alternate_domains: labels(domains.and_then(|d| (&d.alternate).into())),
                favored_weapons: labels(s.and_then(|s| (&s.weapons).into())),
                spells: fact(s.and_then(|s| (&s.spells).into()).map(|spells| {
                    spells
                        .indexed_fields
                        .entries
                        .iter()
                        .map(|(rank, target)| ReferenceGrantView {
                            name: unavailable(),
                            target: FactView {
                                state: QueryFieldState::Value,
                                value: Some(target.clone()),
                            },
                            level: number(
                                rank.parse::<serde_json::Number>()
                                    .as_ref()
                                    .map(SourceFieldView::Value)
                                    .unwrap_or(SourceFieldView::ProjectionInvalid {
                                        source_path: "/system/spells",
                                        reason: "Spell rank cannot be represented",
                                    }),
                            ),
                            navigation: None,
                        })
                        .collect()
                })),
            }))
        }
        _ => return None,
    })
}
pub(super) fn actor(
    record: &SourceBackedRecord,
    node: SourceNodeView<'_>,
) -> Option<RecordBodyView> {
    let query = SourceQueryView {
        source: node,
        pack_id: record.key().pack().as_str(),
        pack_label: "",
    }
    .actor();
    Some(match node {
        SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::VehicleSource(source)) => {
            let s = SourceFieldView::from(&source.system);
            let d = s.and_then(|s| (&s.details).into());
            let a = s.and_then(|s| (&s.attributes).into());
            let space = d.and_then(|d| (&d.space).into());
            RecordBodyView::Vehicle(Box::new(VehicleReferenceView {
                crew: text(d.and_then(|d| (&d.crew).into())),
                passengers: text(d.and_then(|d| (&d.passengers).into())),
                speed: text(d.and_then(|d| (&d.speed).into())),
                piloting: text(d.and_then(|d| (&d.piloting_check).into())),
                price_gp: number(d.and_then(|d| (&d.price).into())),
                length: number(space.and_then(|s| (&s.long).into())),
                width: number(space.and_then(|s| (&s.wide).into())),
                height: number(space.and_then(|s| (&s.high).into())),
                size: fact(query.size().map(super::facts::size_label)),
                collision_damage: text(
                    a.and_then(|a| (&a.collision_damage).into())
                        .and_then(|d| (&d.value).into()),
                ),
                collision_dc: number(
                    a.and_then(|a| (&a.collision_dc).into())
                        .and_then(|d| (&d.value).into()),
                ),
                fortitude: number(
                    s.and_then(|s| (&s.saves).into())
                        .and_then(|s| (&s.fortitude).into())
                        .and_then(|s| (&s.value).into()),
                ),
                defenses: ObjectDefensesReferenceView {
                    armor_class: number(
                        a.and_then(|a| (&a.ac).into())
                            .and_then(|ac| (&ac.value).into()),
                    ),
                    hit_points: number(
                        a.and_then(|a| (&a.hp).into())
                            .and_then(|hp| (&hp.max).into()),
                    ),
                    hardness: number(a.and_then(|a| (&a.hardness).into())),
                    immunities: super::actor::iwr(
                        query.iwr(atlas_record::source_record::ActorIwrKind::Immunity),
                    ),
                    weaknesses: super::actor::iwr(
                        query.iwr(atlas_record::source_record::ActorIwrKind::Weakness),
                    ),
                    resistances: super::actor::iwr(
                        query.iwr(atlas_record::source_record::ActorIwrKind::Resistance),
                    ),
                },
            }))
        }
        SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::ArmySource(source)) => {
            let s = SourceFieldView::from(&source.system);
            let a = s.and_then(|s| (&s.attributes).into());
            let hp = a.and_then(|a| (&a.hp).into());
            let saves = s.and_then(|s| (&s.saves).into());
            let resources = s.and_then(|s| (&s.resources).into());
            let weapons = s.and_then(|s| (&s.weapons).into());
            RecordBodyView::Army(Box::new(ArmyReferenceView {
                army_type: identifier(
                    s.and_then(|s| (&s.traits).into())
                        .and_then(|t| (&t.r#type).into()),
                ),
                armor_class: number(
                    s.and_then(|s| (&s.ac).into())
                        .and_then(|a| (&a.value).into()),
                ),
                hit_points: number(hp.and_then(|hp| (&hp.max).into())),
                rout_threshold: number(hp.and_then(|hp| (&hp.rout_threshold).into())),
                maneuver: number(saves.and_then(|s| (&s.maneuver).into())),
                morale: number(saves.and_then(|s| (&s.morale).into())),
                scouting: number(s.and_then(|s| (&s.scouting).into())),
                consumption: number(s.and_then(|s| (&s.consumption).into())),
                recruitment_dc: number(s.and_then(|s| (&s.recruitment_dc).into())),
                ammunition: number(
                    resources
                        .and_then(|r| (&r.ammunition).into())
                        .and_then(|r| (&r.value).into()),
                ),
                potions: number(
                    resources
                        .and_then(|r| (&r.potions).into())
                        .and_then(|r| (&r.value).into()),
                ),
                melee_weapon: text(
                    weapons
                        .and_then(|w| (&w.melee).into())
                        .and_then(|w| (&w.name).into()),
                ),
                ranged_weapon: text(
                    weapons
                        .and_then(|w| (&w.ranged).into())
                        .and_then(|w| (&w.name).into()),
                ),
                immunities: super::actor::iwr(
                    query.iwr(atlas_record::source_record::ActorIwrKind::Immunity),
                ),
                weaknesses: super::actor::iwr(
                    query.iwr(atlas_record::source_record::ActorIwrKind::Weakness),
                ),
                resistances: super::actor::iwr(
                    query.iwr(atlas_record::source_record::ActorIwrKind::Resistance),
                ),
            }))
        }
        _ => return None,
    })
}

pub(super) fn resolve_links(
    body: &mut RecordBodyView,
    retrieval: &atlas_search::AtlasRetrievalService,
) -> crate::AppServiceResult<()> {
    let grants = match body {
        RecordBodyView::Ancestry(view) => &mut view.grants,
        RecordBodyView::Class(view) => &mut view.grants,
        RecordBodyView::Deity(view) => &mut view.spells,
        _ => return Ok(()),
    };
    for grant in grants.value.iter_mut().flatten() {
        let Some(target) = &grant.target.value else {
            continue;
        };
        let Some(resolved) = retrieval.resolve_source_root_reference(target)? else {
            continue;
        };
        if let atlas_record::source_content::ContentReferenceTarget::Record { key } =
            resolved.target
        {
            if grant.name.value.is_none() {
                grant.name = FactView {
                    state: QueryFieldState::Value,
                    value: resolved.display_name,
                };
            }
            let spell_rank = (resolved.item_family.as_deref() == Some("spell"))
                .then(|| {
                    grant
                        .level
                        .value
                        .as_ref()
                        .and_then(|rank| rank.as_u64())
                        .and_then(|rank| u8::try_from(rank).ok())
                        .filter(|rank| (1..=10).contains(rank))
                })
                .flatten();
            grant.navigation = Some(RecordNavigationView {
                record_key: key.to_string(),
                owners: vec![],
                spell_rank,
                field: None,
                passage: None,
                source_fingerprint: None,
            });
        }
    }
    Ok(())
}
