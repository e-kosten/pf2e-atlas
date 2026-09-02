use std::collections::BTreeSet;

use atlas_domain::MetricDomain;

use crate::{
    ActorMechanics, CreatureDefenses, CreatureMovementMode, CreatureNumber, CreatureRecord,
    CreatureSave, CreatureSkill, CreatureSkillKind, MetricRow, MetricValue, metrics,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureFactProjection {
    pub metrics: Vec<MetricRow>,
    pub actor_side_facts: ActorMechanics,
}

pub fn project_creature_facts(creature: &CreatureRecord) -> CreatureFactProjection {
    CreatureFactProjection {
        metrics: project_metrics(creature),
        actor_side_facts: project_actor_side_facts(creature),
    }
}

fn project_metrics(creature: &CreatureRecord) -> Vec<MetricRow> {
    let mut projected = Vec::new();

    if let Some(perception) = creature.perception.value.as_value() {
        push_defined_number(
            &mut projected,
            metrics::actor::PERCEPTION_MOD,
            perception.modifier.as_value().copied(),
        );
        if let Some(senses) = perception.senses.as_value() {
            for sense in senses {
                let segment = metrics::normalize_metric_key_segment(sense.sense_type.as_str());
                if !segment.is_empty() {
                    push_number(
                        &mut projected,
                        metrics::actor::sense::range_key(&segment),
                        sense.range.as_value().copied(),
                    );
                }
            }
        }
    }

    if let Some(defenses) = creature.defenses.value.as_value() {
        project_defense_metrics(&mut projected, defenses);
    }

    if let Some(abilities) = creature.legacy_abilities.value.as_value() {
        for (slug, modifier) in [
            ("str", &abilities.strength),
            ("dex", &abilities.dexterity),
            ("con", &abilities.constitution),
            ("int", &abilities.intelligence),
            ("wis", &abilities.wisdom),
            ("cha", &abilities.charisma),
        ] {
            push_number(
                &mut projected,
                metrics::actor::ability::mod_key(slug),
                modifier.as_value().copied(),
            );
        }
    }

    if let Some(skills) = creature.skills.value.as_value() {
        for skill in skills {
            if let Some(segment) = skill_metric_segment(skill)
                && !segment.is_empty()
            {
                push_number(
                    &mut projected,
                    metrics::actor::skill::mod_key(&segment),
                    skill.modifier.as_value().copied(),
                );
            }
        }
    }

    if let Some(speeds) = creature.movement.value.as_value() {
        for speed in speeds {
            let segment = metrics::normalize_metric_key_segment(&movement_slug(&speed.mode));
            if !segment.is_empty() {
                push_number(
                    &mut projected,
                    metrics::actor::speed::value_key(&segment),
                    speed.value.as_value().copied(),
                );
            }
        }
    }

    dedupe_metrics(projected)
}

fn project_defense_metrics(projected: &mut Vec<MetricRow>, defenses: &CreatureDefenses) {
    if let Some(armor_class) = defenses.armor_class.as_value() {
        push_defined_number(
            projected,
            metrics::actor::ARMOR_CLASS,
            armor_class.value.as_value().copied(),
        );
    }
    if let Some(hit_points) = defenses.hit_points.as_value() {
        let value = hit_points.value.as_value().and_then(|value| match value {
            CreatureNumber::Integer(value) => Some(*value),
            CreatureNumber::Unsupported(_) => None,
        });
        push_defined_number(projected, metrics::actor::HP_VALUE, value);
        push_defined_number(
            projected,
            metrics::actor::HP_MAX,
            hit_points.maximum.as_value().copied(),
        );
    }
    if let Some(saves) = defenses.saves.as_value() {
        let saves = [
            ("fort", saves.fortitude.as_value()),
            ("ref", saves.reflex.as_value()),
            ("will", saves.will.as_value()),
        ];
        let mut values = Vec::new();
        for (slug, save) in saves {
            let value = save.and_then(save_value);
            push_number(projected, metrics::actor::save::mod_key(slug), value);
            if let Some(value) = value {
                values.push((slug, value));
            }
        }
        if let Some((best, _)) = values.iter().max_by(|left, right| left.1.cmp(&right.1)) {
            push_defined_text(projected, metrics::actor::save::BEST, best);
        }
        if let Some((worst, _)) = values.iter().min_by(|left, right| left.1.cmp(&right.1)) {
            push_defined_text(projected, metrics::actor::save::WORST, worst);
        }
    }
}

fn save_value(save: &CreatureSave) -> Option<i64> {
    save.value.as_value().copied()
}

fn project_actor_side_facts(creature: &CreatureRecord) -> ActorMechanics {
    let size = creature
        .size
        .value
        .as_value()
        .map(|size| size.as_source().to_string());
    let languages = creature
        .languages
        .value
        .as_value()
        .and_then(|languages| languages.values.as_value())
        .map(|values| {
            values
                .iter()
                .map(|value| value.as_str().to_string())
                .collect()
        })
        .unwrap_or_default();
    let mut speed_types = creature
        .movement
        .value
        .as_value()
        .map(|speeds| {
            speeds
                .iter()
                .map(|speed| movement_slug(&speed.mode))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    speed_types.sort();
    speed_types.dedup();
    let senses = creature
        .perception
        .value
        .as_value()
        .and_then(|perception| perception.senses.as_value())
        .map(|senses| {
            senses
                .iter()
                .map(|sense| sense.sense_type.as_str().to_string())
                .collect()
        })
        .unwrap_or_default();
    let defenses = creature.defenses.value.as_value();
    ActorMechanics {
        size,
        languages,
        speed_types,
        senses,
        immunities: project_iwr(defenses.and_then(|value| value.immunities.as_value())),
        resistances: project_iwr(defenses.and_then(|value| value.resistances.as_value())),
        weaknesses: project_iwr(defenses.and_then(|value| value.weaknesses.as_value())),
        disable_text: None,
        disable_skills: Vec::new(),
        is_complex: false,
    }
}

fn project_iwr(entries: Option<&Vec<crate::CreatureIwr>>) -> Vec<String> {
    entries
        .into_iter()
        .flatten()
        .map(|entry| entry.iwr_type.as_str().to_string())
        .collect()
}

fn skill_metric_segment(skill: &CreatureSkill) -> Option<String> {
    match skill.kind {
        CreatureSkillKind::Lore => Some(metrics::normalize_metric_key_segment(&skill.label)),
        CreatureSkillKind::Unmodeled => None,
        kind => Some(kind.source_slug().to_string()),
    }
}

fn movement_slug(mode: &CreatureMovementMode) -> String {
    match mode {
        CreatureMovementMode::Land => "land".to_string(),
        CreatureMovementMode::Burrow => "burrow".to_string(),
        CreatureMovementMode::Climb => "climb".to_string(),
        CreatureMovementMode::Fly => "fly".to_string(),
        CreatureMovementMode::Swim => "swim".to_string(),
        CreatureMovementMode::Unsupported(value) => value.value.clone(),
    }
}

fn push_defined_number(
    projected: &mut Vec<MetricRow>,
    definition: metrics::MetricDefinition,
    value: Option<i64>,
) {
    if let Some(key) = definition.exact_key() {
        push_number(projected, key.to_string(), value);
    }
}

fn push_defined_text(
    projected: &mut Vec<MetricRow>,
    definition: metrics::MetricDefinition,
    value: &str,
) {
    if let Some(key) = definition.exact_key() {
        projected.push(MetricRow {
            domain: MetricDomain::Actor,
            key: key.to_string(),
            value: MetricValue::Text(value.to_string()),
        });
    }
}

fn push_number(projected: &mut Vec<MetricRow>, key: String, value: Option<i64>) {
    if let Some(value) = value {
        projected.push(MetricRow {
            domain: MetricDomain::Actor,
            key,
            value: MetricValue::Number(value as f64),
        });
    }
}

fn dedupe_metrics(metrics: Vec<MetricRow>) -> Vec<MetricRow> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for metric in metrics.into_iter().rev() {
        if seen.insert((metric.domain, metric.key.clone())) {
            deduped.push(metric);
        }
    }
    deduped.reverse();
    deduped
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, PublicationCategory, Rarity, RecordId, RecordKey, RecordKind};

    use super::*;
    use crate::{
        CreatureArmorClass, CreatureComponentId, CreatureFact, CreatureHitPoints, CreatureIdentity,
        CreatureIwr, CreatureIwrKind, CreatureLanguages, CreatureNumber, CreaturePerception,
        CreatureProvenance, CreaturePublication, CreatureSave, CreatureSaveKind, CreatureSaves,
        CreatureSense, CreatureSize, CreatureSkill, CreatureSourceField, CreatureSourceId,
        CreatureSpeed, FactValue, FoundryDocumentMechanics, FoundryDocumentType, FoundryRecordInfo,
        FoundryRecordType, IwrType, Language, PresentationBlock, RecordClassification,
        RecordContent, RecordIdentity, RecordMechanics, RecordProvenance, RecordPublication,
        RecordRequirements, RecordTaxonomy, RecordTiming, RecordVisibility, SenseType,
        UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
        build_record_fts_projection, build_record_presentation_document,
    };

    #[test]
    fn canonical_metrics_side_display_and_fts_are_consistent() {
        let creature = dense_creature();
        let projection = project_creature_facts(&creature);

        assert_number(&projection.metrics, "perception.mod", 18.0);
        assert_number(&projection.metrics, "ac.value", 28.0);
        assert_number(&projection.metrics, "hp.value", 170.0);
        assert_number(&projection.metrics, "hp.max", 170.0);
        assert_number(&projection.metrics, "save.fort.mod", 19.0);
        assert_number(&projection.metrics, "skill.arcana.mod", 18.0);
        assert_number(&projection.metrics, "skill.theater_lore.mod", 25.0);
        assert_number(&projection.metrics, "speed.land.value", 25.0);
        assert_number(&projection.metrics, "speed.fly.value", 40.0);
        assert_number(&projection.metrics, "sense.scent.range", 60.0);
        assert_eq!(projection.actor_side_facts.size.as_deref(), Some("med"));
        assert_eq!(projection.actor_side_facts.languages, ["aklo", "common"]);
        assert_eq!(projection.actor_side_facts.speed_types, ["fly", "land"]);
        assert_eq!(projection.actor_side_facts.senses, ["darkvision", "scent"]);
        assert_eq!(projection.actor_side_facts.resistances, ["mental"]);

        let record = projected_record(&creature, projection);
        let presentation = build_record_presentation_document(&record);
        assert!(
            presentation.sections.iter().any(|section| {
                section.blocks.iter().any(|block| {
                    matches!(
                        block,
                        PresentationBlock::FactList(facts)
                            if facts.iter().any(|fact| {
                            fact.label == "Skills"
                                && fact.value.contains("Arcana +18")
                                && fact.value.contains("Theater lore +25")
                            })
                    )
                })
            }),
            "{presentation:#?}"
        );
        let fts = build_record_fts_projection(&record, &[]);
        assert!(fts.metric_terms.contains("Arcana"));
        assert!(fts.metric_terms.contains("Theater lore"));
        assert!(fts.metric_terms.contains("Fly Speed"));
        assert!(!fts.metric_terms.contains("40"));
        assert!(fts.mechanic_terms.contains("darkvision"));
        assert!(fts.mechanic_terms.contains("mental"));
    }

    #[test]
    fn missing_null_and_zero_remain_distinct_before_sparse_projection() {
        let mut creature = dense_creature();
        creature.perception.value = FactValue::Missing;
        creature.skills.value = FactValue::Null;
        creature.defenses.value = FactValue::Null;
        creature.movement.value = FactValue::Value(vec![speed(
            "speed:land:0",
            0,
            CreatureMovementMode::Land,
            FactValue::Value(0),
        )]);

        let projection = project_creature_facts(&creature);

        assert_eq!(creature.perception.value, FactValue::Missing);
        assert_eq!(creature.skills.value, FactValue::Null);
        assert_eq!(creature.defenses.value, FactValue::Null);
        assert!(metric(&projection.metrics, "perception.mod").is_none());
        assert!(metric(&projection.metrics, "ac.value").is_none());
        assert_number(&projection.metrics, "speed.land.value", 0.0);
        assert!(projection.actor_side_facts.languages.len() == 2);
        assert_eq!(projection.actor_side_facts.speed_types, ["land"]);
    }

    #[test]
    fn unsupported_numeric_values_do_not_become_metrics() {
        let mut creature = dense_creature();
        let defenses = creature
            .defenses
            .value
            .as_value()
            .expect("defenses")
            .clone();
        let mut hit_points = defenses.hit_points.as_value().expect("hit points").clone();
        hit_points.value = FactValue::Value(CreatureNumber::Unsupported(UnsupportedSourceValue {
            shape: UnsupportedSourceShape::String,
            value: "many".to_string(),
            reason: UnsupportedSourceReason::AmbiguousLegacyShape,
        }));
        let mut defenses = defenses;
        defenses.hit_points = FactValue::Value(hit_points);
        creature.defenses.value = FactValue::Value(defenses);

        let projection = project_creature_facts(&creature);

        assert!(metric(&projection.metrics, "hp.value").is_none());
        assert_number(&projection.metrics, "hp.max", 170.0);
    }

    fn dense_creature() -> CreatureRecord {
        let record_key = key();
        CreatureRecord {
            identity: CreatureIdentity {
                record_key,
                source_id: CreatureSourceId::new("night-hag").expect("source id"),
                name: "Night Hag".to_string(),
                family: crate::CreatureFamily::Npc,
            },
            level: source(FactValue::Value(9), CreatureSourceField::Level),
            rarity: source(
                FactValue::Value(Rarity::Common),
                CreatureSourceField::Rarity,
            ),
            traits: source(FactValue::Value(Vec::new()), CreatureSourceField::Traits),
            size: source(
                FactValue::Value(CreatureSize::Medium),
                CreatureSourceField::Size,
            ),
            publication: source(
                FactValue::Value(CreaturePublication {
                    title: FactValue::Value("Pathfinder Bestiary".to_string()),
                    remaster: FactValue::Value(false),
                    license: FactValue::Missing,
                }),
                CreatureSourceField::Publication,
            ),
            adjustment: source(FactValue::Missing, CreatureSourceField::Adjustment),
            source_alliance: source(FactValue::Missing, CreatureSourceField::SourceAlliance),
            perception: source(
                FactValue::Value(CreaturePerception {
                    modifier: FactValue::Value(18),
                    details: FactValue::Missing,
                    has_vision: FactValue::Missing,
                    senses: FactValue::Value(vec![
                        sense("sense:darkvision:0", 0, "darkvision", FactValue::Missing),
                        sense("sense:scent:0", 1, "scent", FactValue::Value(60)),
                    ]),
                }),
                CreatureSourceField::Perception,
            ),
            initiative: source(FactValue::Missing, CreatureSourceField::Initiative),
            languages: source(
                FactValue::Value(CreatureLanguages {
                    values: FactValue::Value(vec![language("aklo"), language("common")]),
                    details: FactValue::Missing,
                }),
                CreatureSourceField::Languages,
            ),
            skills: source(
                FactValue::Value(vec![
                    skill("skill:arcana", CreatureSkillKind::Arcana, "Arcana", 18),
                    skill(
                        "skill:lore:theater",
                        CreatureSkillKind::Lore,
                        "Theater Lore",
                        25,
                    ),
                ]),
                CreatureSourceField::Skills,
            ),
            legacy_abilities: source(FactValue::Missing, CreatureSourceField::LegacyAbilities),
            defenses: source(
                FactValue::Value(CreatureDefenses {
                    armor_class: FactValue::Value(CreatureArmorClass {
                        value: FactValue::Value(28),
                        details: FactValue::Missing,
                    }),
                    hit_points: FactValue::Value(CreatureHitPoints {
                        value: FactValue::Value(CreatureNumber::Integer(170)),
                        maximum: FactValue::Value(170),
                        temporary: FactValue::Value(0),
                        temporary_maximum: FactValue::Missing,
                        details: FactValue::Missing,
                    }),
                    hardness: FactValue::Missing,
                    shield: FactValue::Missing,
                    saves: FactValue::Value(CreatureSaves {
                        fortitude: FactValue::Value(save(
                            "save:fortitude",
                            CreatureSaveKind::Fortitude,
                            19,
                        )),
                        reflex: FactValue::Value(save("save:reflex", CreatureSaveKind::Reflex, 17)),
                        will: FactValue::Value(save("save:will", CreatureSaveKind::Will, 18)),
                    }),
                    all_saves_note: FactValue::Missing,
                    immunities: FactValue::Value(Vec::new()),
                    resistances: FactValue::Value(vec![iwr("mental")]),
                    weaknesses: FactValue::Value(Vec::new()),
                }),
                CreatureSourceField::Defenses,
            ),
            movement: source(
                FactValue::Value(vec![
                    speed(
                        "speed:land:0",
                        0,
                        CreatureMovementMode::Land,
                        FactValue::Value(25),
                    ),
                    speed(
                        "speed:fly:0",
                        1,
                        CreatureMovementMode::Fly,
                        FactValue::Value(40),
                    ),
                ]),
                CreatureSourceField::Movement,
            ),
            resources: source(FactValue::Missing, CreatureSourceField::Resources),
            embedded_entities: source(FactValue::Missing, CreatureSourceField::EmbeddedEntities),
            content: crate::OwnedRichContent::default(),
            provenance: CreatureProvenance {
                source_path: "packs/bestiary/night-hag.json".to_string(),
                source_contract_version: "pf2e-serialized-source/v1".to_string(),
                source_system_version: "6.12.4".to_string(),
                source_upstream_commit: "pinned".to_string(),
            },
        }
    }

    fn projected_record(
        creature: &CreatureRecord,
        projection: CreatureFactProjection,
    ) -> crate::AtlasRecord {
        crate::AtlasRecord {
            identity: RecordIdentity {
                key: creature.identity.record_key.clone(),
                name: creature.identity.name.clone(),
            },
            classification: RecordClassification {
                kind: RecordKind::Creature,
                level: creature.level.value.as_value().copied(),
                rarity: creature.rarity.value.as_value().copied(),
                traits: Vec::new(),
                taxonomy: RecordTaxonomy::default(),
            },
            foundry: FoundryRecordInfo {
                pack_label: "Bestiary".to_string(),
                document_type: FoundryDocumentType::Actor,
                record_type: FoundryRecordType::Npc,
                folder_id: None,
            },
            provenance: RecordProvenance {
                source_path: creature.provenance.source_path.clone(),
                raw_json: None,
            },
            publication: RecordPublication {
                title: Some("Pathfinder Bestiary".to_string()),
                remaster: false,
                category: PublicationCategory::Unknown,
            },
            requirements: RecordRequirements::default(),
            timing: RecordTiming::default(),
            mechanics: RecordMechanics {
                metrics: projection.metrics,
                document: FoundryDocumentMechanics::Actor(projection.actor_side_facts),
                spellcasting_entries: Vec::new(),
                activities: Vec::new(),
            },
            content: RecordContent::default(),
            variant: None,
            visibility: RecordVisibility::default(),
        }
    }

    fn source<T>(value: FactValue<T>, field: CreatureSourceField) -> CreatureFact<T> {
        CreatureFact::source(value, field)
    }

    fn sense(id: &str, order: u32, kind: &str, range: FactValue<i64>) -> CreatureSense {
        CreatureSense {
            id: CreatureComponentId::new(id).expect("sense id"),
            authored_order: order,
            sense_type: SenseType::new(kind).expect("sense type"),
            acuity: FactValue::Missing,
            range,
        }
    }

    fn skill(id: &str, kind: CreatureSkillKind, label: &str, modifier: i64) -> CreatureSkill {
        CreatureSkill {
            id: CreatureComponentId::new(id).expect("skill id"),
            authored_order: 0,
            source_entries: vec![crate::CreatureSkillSourceEntry {
                authored_key: kind.source_slug().to_string(),
                modifier: FactValue::Value(modifier),
            }],
            kind,
            label: label.to_string(),
            modifier: FactValue::Value(modifier),
            note: FactValue::Missing,
            variants: FactValue::Missing,
            source_item_id: FactValue::Missing,
            unmodeled: FactValue::Missing,
        }
    }

    fn save(id: &str, kind: CreatureSaveKind, value: i64) -> CreatureSave {
        CreatureSave {
            id: CreatureComponentId::new(id).expect("save id"),
            kind,
            value: FactValue::Value(value),
            details: FactValue::Missing,
        }
    }

    fn speed(
        id: &str,
        authored_order: u32,
        mode: CreatureMovementMode,
        value: FactValue<i64>,
    ) -> CreatureSpeed {
        CreatureSpeed {
            id: CreatureComponentId::new(id).expect("speed id"),
            authored_order,
            mode,
            value,
            label: FactValue::Missing,
            details: FactValue::Missing,
        }
    }

    fn iwr(kind: &str) -> CreatureIwr {
        CreatureIwr {
            id: CreatureComponentId::new(format!("iwr:resistance:{kind}:0")).expect("iwr id"),
            authored_order: 0,
            kind: CreatureIwrKind::Resistance,
            iwr_type: IwrType::new(kind).expect("iwr type"),
            value: FactValue::Value(10),
            exceptions: FactValue::Missing,
            double_vs: FactValue::Missing,
            apply_once: FactValue::Missing,
        }
    }

    fn language(value: &str) -> Language {
        Language::new(value).expect("language")
    }

    fn key() -> RecordKey {
        RecordKey::new(
            PackName::new("bestiary").expect("pack"),
            RecordId::new("night-hag").expect("id"),
        )
    }

    fn metric<'a>(metrics: &'a [MetricRow], key: &str) -> Option<&'a MetricRow> {
        metrics
            .iter()
            .find(|metric| metric.domain == MetricDomain::Actor && metric.key == key)
    }

    fn assert_number(metrics: &[MetricRow], key: &str, expected: f64) {
        assert_eq!(
            metric(metrics, key).map(|metric| &metric.value),
            Some(&MetricValue::Number(expected)),
            "metric {key}"
        );
    }
}
