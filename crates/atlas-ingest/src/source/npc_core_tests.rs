use atlas_domain::{MetricDomain, PackName, Rarity, RecordId, RecordKey};
use atlas_record::{
    CreatureAdjustment, CreatureInitiativeStatistic, CreatureIwrKind, CreatureMovementMode,
    CreatureNumber, CreatureResourceAmount, CreatureSkillKind, CreatureSourceAlliance,
    CreatureUnmodeledSkillReason, CreatureUnsupportedSourceField, FactValue, MetricValue,
    RecordBody, ResourceCurrentPolicy, SenseAcuity, ShieldCurrentPolicy, UnsupportedSourceReason,
    UnsupportedSourceShape, project_creature_facts,
};
use serde_json::{Value, json};
use std::path::Path;

use super::dto::{SourceIdentity, parse_npc_source, pinned_source_version_metadata};
use super::npc_core::{
    NpcCoreDiagnosticDisposition, NpcCoreDiagnosticKind, NpcCoreDiagnosticOwner, convert_npc_core,
};

#[test]
fn night_hag_matches_approved_canonical_core_facts() {
    let source = parse(night_hag_source(), "pathfinder-bestiary:WQy7HBUcgDLsfVJd");
    let converted = convert_npc_core(
        record_key("pathfinder-bestiary", "WQy7HBUcgDLsfVJd"),
        "packs/pathfinder-bestiary/night-hag.json",
        &source,
    )
    .expect("Night Hag core conversion");
    assert!(converted.diagnostics.is_empty());

    let RecordBody::Creature(creature) = &converted.body;
    assert_eq!(creature.identity.source_id.as_str(), "WQy7HBUcgDLsfVJd");
    assert_eq!(creature.identity.name, "Night Hag");
    assert_eq!(creature.level.value, FactValue::Value(9));
    assert_eq!(creature.rarity.value, FactValue::Value(Rarity::Common));
    assert_eq!(
        creature
            .traits
            .value
            .as_value()
            .expect("traits")
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["evil", "fiend", "hag", "humanoid", "unholy"]
    );
    assert_eq!(
        creature.size.value.as_value().expect("size").as_source(),
        "med"
    );

    let perception = creature.perception.value.as_value().expect("perception");
    assert_eq!(perception.modifier, FactValue::Value(18));
    let senses = perception.senses.as_value().expect("senses");
    assert_eq!(senses.len(), 1);
    assert_eq!(senses[0].id.as_str(), "sense:darkvision:0");
    assert_eq!(senses[0].authored_order, 0);
    assert_eq!(senses[0].sense_type.as_str(), "darkvision");
    assert_eq!(senses[0].acuity, FactValue::Missing);
    assert_eq!(senses[0].range, FactValue::Missing);

    let languages = creature.languages.value.as_value().expect("languages");
    assert_eq!(
        languages
            .values
            .as_value()
            .expect("language values")
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["aklo", "chthonian", "common", "diabolic", "empyrean"]
    );

    let skills = creature.skills.value.as_value().expect("skills");
    assert_eq!(skills.len(), 6);
    assert_eq!(
        skills
            .iter()
            .map(|skill| (
                skill.id.as_str(),
                skill.kind,
                skill.modifier.as_value().copied()
            ))
            .collect::<Vec<_>>(),
        [
            ("skill:arcana", CreatureSkillKind::Arcana, Some(18)),
            ("skill:deception", CreatureSkillKind::Deception, Some(18)),
            ("skill:diplomacy", CreatureSkillKind::Diplomacy, Some(18)),
            (
                "skill:intimidation",
                CreatureSkillKind::Intimidation,
                Some(14)
            ),
            ("skill:occultism", CreatureSkillKind::Occultism, Some(20)),
            ("skill:religion", CreatureSkillKind::Religion, Some(20)),
        ]
    );

    let defenses = creature.defenses.value.as_value().expect("defenses");
    assert_eq!(
        defenses.armor_class.as_value().expect("armor class").value,
        FactValue::Value(28)
    );
    let hp = defenses.hit_points.as_value().expect("hit points");
    assert_eq!(hp.value, FactValue::Value(CreatureNumber::Integer(170)));
    assert_eq!(hp.maximum, FactValue::Value(170));
    assert_eq!(
        defenses
            .all_saves_note
            .as_value()
            .expect("all saves note")
            .as_str(),
        "+2 status to all saves vs. magic, -2 to all saves (if heartstone is lost)"
    );
    assert_iwr(
        defenses.immunities.as_value().expect("immunities"),
        CreatureIwrKind::Immunity,
        "sleep",
        None,
    );
    assert_iwr(
        defenses.resistances.as_value().expect("resistances"),
        CreatureIwrKind::Resistance,
        "mental",
        Some(10),
    );
    assert_iwr(
        defenses.weaknesses.as_value().expect("weaknesses"),
        CreatureIwrKind::Weakness,
        "cold-iron",
        Some(10),
    );

    let movement = creature.movement.value.as_value().expect("movement");
    assert_eq!(movement.len(), 1);
    assert_eq!(movement[0].id.as_str(), "speed:land:0");
    assert_eq!(movement[0].mode, CreatureMovementMode::Land);
    assert_eq!(movement[0].value, FactValue::Value(25));

    let resources = creature.resources.value.as_value().expect("resources");
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].id.as_str(), "resource:focus");
    assert_eq!(resources[0].kind.as_str(), "focus");
    assert_eq!(
        resources[0].maximum,
        FactValue::Value(CreatureResourceAmount::Integer(1))
    );
    assert_eq!(
        resources[0].serialized_value,
        FactValue::Value(CreatureResourceAmount::Integer(1))
    );
    assert_eq!(
        resources[0].current_policy,
        ResourceCurrentPolicy::SerializedValueIsProvenanceOnly
    );
    assert_eq!(
        creature.provenance.source_path,
        "packs/pathfinder-bestiary/night-hag.json"
    );

    let projected = project_creature_facts(creature);
    assert_eq!(
        projected.metrics.iter().find_map(|metric| {
            (metric.domain == MetricDomain::Actor && metric.key == "skill.occultism.mod")
                .then_some(&metric.value)
        }),
        Some(&MetricValue::Number(20.0))
    );
    assert_eq!(
        projected.metrics.iter().find_map(|metric| {
            (metric.domain == MetricDomain::Actor && metric.key == "speed.land.value")
                .then_some(&metric.value)
        }),
        Some(&MetricValue::Number(25.0))
    );
}

#[test]
fn dense_core_preserves_skill_variants_lore_iwr_exceptions_and_movement_order() {
    let source = parse(dense_source(), "fixture-pack:dense");
    let converted = convert_npc_core(
        record_key("fixture-pack", "dense"),
        "packs/fixture-pack/dense.json",
        &source,
    )
    .expect("dense conversion");
    let RecordBody::Creature(creature) = &converted.body;

    let skills = creature.skills.value.as_value().expect("skills");
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0].kind, CreatureSkillKind::Performance);
    let variants = skills[0].variants.as_value().expect("variants");
    assert_eq!(variants[0].id.as_str(), "skill:performance:variant:0");
    assert_eq!(variants[0].modifier, FactValue::Value(31));
    assert_eq!(variants[0].authored_order, 0);
    assert_eq!(skills[1].kind, CreatureSkillKind::Lore);
    assert_eq!(skills[1].id.as_str(), "skill:lore:lore-source-id");
    assert_eq!(skills[1].modifier, FactValue::Value(25));
    assert_eq!(
        skills[1]
            .source_item_id
            .as_value()
            .expect("source id")
            .as_str(),
        "lore-source-id"
    );

    let senses = creature
        .perception
        .value
        .as_value()
        .expect("perception")
        .senses
        .as_value()
        .expect("senses");
    assert_eq!(senses[1].acuity, FactValue::Value(SenseAcuity::Imprecise));
    assert_eq!(senses[1].range, FactValue::Value(60));

    let defenses = creature.defenses.value.as_value().expect("defenses");
    let resistance = &defenses.resistances.as_value().expect("resistance")[0];
    assert_eq!(resistance.id.as_str(), "iwr:resistance:all-damage:0");
    assert_eq!(
        resistance
            .exceptions
            .as_value()
            .expect("exceptions")
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["force", "ghost-touch", "vitality"]
    );
    assert_eq!(
        resistance
            .double_vs
            .as_value()
            .expect("double vs")
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["non-magical"]
    );
    assert_eq!(resistance.apply_once, FactValue::Value(true));

    let movement = creature.movement.value.as_value().expect("movement");
    assert_eq!(
        movement
            .iter()
            .map(|speed| (
                &speed.mode,
                speed.value.as_value().copied(),
                speed.authored_order
            ))
            .collect::<Vec<_>>(),
        [
            (&CreatureMovementMode::Land, Some(0), 0),
            (&CreatureMovementMode::Fly, Some(30), 1),
        ]
    );
    assert!(converted.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == NpcCoreDiagnosticKind::UnstableSourceOrderIdentity
    }));
}

#[test]
fn malformed_component_ids_use_portable_source_scoped_fallbacks_without_losing_siblings() {
    let raw = json!({
        "_id": "movement-fallback",
        "name": "Movement Fallback Fixture",
        "type": "npc",
        "system": {"attributes": {"speed": {
            "details": "all sibling movement fields survive",
            "otherSpeeds": [
                {"label": "spell-granted", "type": "Air Walk", "value": 25},
                {"label": "explicit null type", "type": null, "value": 15},
                {"label": "missing type", "value": 10}
            ]
        }}}
    });
    let source = parse(raw, "fixture-pack:movement-fallback");
    let converted = convert_npc_core(
        record_key("fixture-pack", "movement-fallback"),
        "packs/fixture-pack/movement-fallback.json",
        &source,
    )
    .expect("component-id problems preserve the enclosing creature");
    let RecordBody::Creature(creature) = &converted.body;
    let movement = creature.movement.value.as_value().expect("movement");

    assert_eq!(movement.len(), 3);
    assert_eq!(movement[0].value, FactValue::Value(25));
    assert_eq!(
        movement[0].label,
        FactValue::Value("spell-granted".to_string())
    );
    assert_eq!(movement[1].value, FactValue::Value(15));
    assert_eq!(movement[2].value, FactValue::Value(10));
    assert_eq!(
        movement[0].id.as_str(),
        "speed:source:fixture-pack:movement-fallback:Air%20Walk:0"
    );
    assert_eq!(
        movement[1].id.as_str(),
        "speed:scoped:fixture-pack:movement-fallback:other-speed:1:0"
    );
    assert_eq!(
        movement[2].id.as_str(),
        "speed:scoped:fixture-pack:movement-fallback:other-speed:2:0"
    );
    assert!(matches!(
        movement[1].mode,
        CreatureMovementMode::Unsupported(atlas_record::UnsupportedSourceValue {
            shape: atlas_record::UnsupportedSourceShape::Null,
            ..
        })
    ));
    assert!(matches!(
        movement[2].mode,
        CreatureMovementMode::Unsupported(atlas_record::UnsupportedSourceValue {
            shape: atlas_record::UnsupportedSourceShape::Missing,
            ..
        })
    ));

    let fallback = converted
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.kind == NpcCoreDiagnosticKind::ComponentIdSourceFallback)
        .expect("source-derived fallback diagnostic");
    assert_eq!(
        fallback.code,
        "atlas.npc_core.component_id_source_fallback.v1"
    );
    assert_eq!(fallback.record_key, "fixture-pack:movement-fallback");
    assert_eq!(
        fallback.source_path,
        "packs/fixture-pack/movement-fallback.json"
    );
    assert_eq!(
        fallback.source_field,
        "$.system.attributes.speed.otherSpeeds[0].type"
    );
    assert_eq!(fallback.source_value, "Air Walk");
    assert_eq!(
        fallback.disposition,
        NpcCoreDiagnosticDisposition::StableSourceFallback
    );
    assert_eq!(fallback.owner, NpcCoreDiagnosticOwner::CanonicalNpcCore);
    assert!(!fallback.source_path.starts_with('/'));
    let portable = serde_json::to_value(fallback).expect("portable diagnostic JSON");
    assert_eq!(
        portable["observed_issue"],
        "source-derived component identity violates canonical id syntax"
    );
    assert_eq!(portable["owner"], "canonical_npc_core");

    let reordered = parse(
        json!({
            "_id": "movement-fallback",
            "name": "Movement Fallback Fixture",
            "type": "npc",
            "system": {"attributes": {"speed": {"otherSpeeds": [
                {"type": "Spider Climb", "value": 20},
                {"type": "Air Walk", "value": 25}
            ]}}}
        }),
        "fixture-pack:movement-fallback",
    );
    let reordered = convert_npc_core(
        record_key("fixture-pack", "movement-fallback"),
        "packs/fixture-pack/movement-fallback.json",
        &reordered,
    )
    .expect("reordered source-derived fallbacks");
    let RecordBody::Creature(reordered) = &reordered.body;
    let air_walk = reordered
        .movement
        .value
        .as_value()
        .expect("movement")
        .iter()
        .find(|speed| matches!(&speed.mode, CreatureMovementMode::Unsupported(value) if value.value == "Air Walk"))
        .expect("Air Walk");
    assert_eq!(air_walk.id, movement[0].id);
}

#[test]
fn conversion_preserves_missing_null_and_zero_without_prepared_defaults() {
    let source = parse(
        json!({
            "_id": "presence",
            "name": "Presence Fixture",
            "type": "npc",
            "system": {
                "details": null,
                "traits": {"rarity": null, "size": null, "value": null},
                "perception": null,
                "attributes": {"speed": {"value": 0, "otherSpeeds": null}},
                "saves": null,
                "skills": null,
                "resources": null
            },
            "items": null
        }),
        "fixture-pack:presence",
    );
    let converted = convert_npc_core(
        record_key("fixture-pack", "presence"),
        "packs/fixture-pack/presence.json",
        &source,
    )
    .expect("presence conversion");
    let RecordBody::Creature(creature) = &converted.body;
    assert_eq!(creature.level.value, FactValue::Null);
    assert_eq!(creature.rarity.value, FactValue::Null);
    assert_eq!(creature.traits.value, FactValue::Null);
    assert_eq!(creature.size.value, FactValue::Null);
    assert_eq!(creature.perception.value, FactValue::Null);
    assert_eq!(creature.skills.value, FactValue::Null);
    assert_eq!(creature.resources.value, FactValue::Null);
    let movement = creature.movement.value.as_value().expect("movement");
    assert_eq!(movement[0].value, FactValue::Value(0));
}

#[test]
fn actor_root_facts_preserve_missing_null_value_and_serialized_state_boundaries() {
    let value = parse(
        json!({
            "_id": "actor-root-value",
            "name": "Actor Root Value",
            "type": "npc",
            "system": {
                "abilities": {
                    "str": {"mod": 0, "value": 10}, "dex": {"mod": 2, "value": 12},
                    "con": {"mod": 3, "value": 13}, "int": {"mod": 4, "value": 14},
                    "wis": {"mod": 5, "value": 15}, "cha": {"mod": 6, "value": 16}
                },
                "attributes": {
                    "adjustment": "elite",
                    "hardness": {"value": 0},
                    "shield": {"ac": 2, "brokenThreshold": 0, "hardness": 0, "max": 0, "value": 0}
                },
                "details": {"alliance": "party"},
                "initiative": {"statistic": "perception"}
            }
        }),
        "fixture-pack:actor-root-value",
    );
    let converted = convert_npc_core(
        record_key("fixture-pack", "actor-root-value"),
        "packs/fixture-pack/actor-root-value.json",
        &value,
    )
    .expect("actor-root value conversion");
    assert!(converted.diagnostics.is_empty());
    let RecordBody::Creature(creature) = &converted.body;
    assert_eq!(
        creature.adjustment.value,
        FactValue::Value(CreatureAdjustment::Elite)
    );
    let CreatureSourceAlliance::Named(alliance) = creature
        .source_alliance
        .value
        .as_value()
        .expect("source alliance")
    else {
        panic!("party alliance should remain a named source fact");
    };
    assert_eq!(alliance.as_str(), "party");
    let initiative = creature.initiative.value.as_value().expect("initiative");
    let CreatureInitiativeStatistic::Named(statistic) = initiative
        .statistic
        .as_value()
        .expect("initiative statistic")
    else {
        panic!("perception should remain a named statistic");
    };
    assert_eq!(statistic.as_str(), "perception");
    let abilities = creature
        .legacy_abilities
        .value
        .as_value()
        .expect("legacy abilities");
    assert_eq!(abilities.strength, FactValue::Value(0));
    assert_eq!(abilities.dexterity, FactValue::Value(2));
    assert_eq!(abilities.charisma, FactValue::Value(6));
    let defenses = creature.defenses.value.as_value().expect("defenses");
    assert_eq!(defenses.hardness, FactValue::Value(0));
    let shield = defenses.shield.as_value().expect("shield");
    assert_eq!(shield.armor_class_bonus, FactValue::Value(2));
    assert_eq!(shield.broken_threshold, FactValue::Value(0));
    assert_eq!(shield.hardness, FactValue::Value(0));
    assert_eq!(shield.maximum_hit_points, FactValue::Value(0));
    assert_eq!(shield.serialized_hit_points, FactValue::Value(0));
    assert_eq!(
        shield.current_policy,
        ShieldCurrentPolicy::SerializedHitPointsAreProvenanceOnly
    );

    let null = parse(
        json!({
            "_id": "actor-root-null", "name": "Actor Root Null", "type": "npc",
            "system": {
                "abilities": null,
                "attributes": {"adjustment": null, "hardness": null, "shield": null},
                "details": {"alliance": null},
                "initiative": null
            }
        }),
        "fixture-pack:actor-root-null",
    );
    let converted = convert_npc_core(
        record_key("fixture-pack", "actor-root-null"),
        "packs/fixture-pack/actor-root-null.json",
        &null,
    )
    .expect("actor-root null conversion");
    let RecordBody::Creature(creature) = &converted.body;
    assert_eq!(creature.adjustment.value, FactValue::Null);
    assert_eq!(creature.source_alliance.value, FactValue::Null);
    assert_eq!(creature.initiative.value, FactValue::Null);
    assert_eq!(creature.legacy_abilities.value, FactValue::Null);
    let defenses = creature.defenses.value.as_value().expect("defenses");
    assert_eq!(defenses.hardness, FactValue::Null);
    assert_eq!(defenses.shield, FactValue::Null);

    let missing = parse(
        json!({"_id": "actor-root-missing", "name": "Actor Root Missing", "type": "npc", "system": {}}),
        "fixture-pack:actor-root-missing",
    );
    let converted = convert_npc_core(
        record_key("fixture-pack", "actor-root-missing"),
        "packs/fixture-pack/actor-root-missing.json",
        &missing,
    )
    .expect("actor-root missing conversion");
    let RecordBody::Creature(creature) = &converted.body;
    assert_eq!(creature.adjustment.value, FactValue::Missing);
    assert_eq!(creature.source_alliance.value, FactValue::Missing);
    assert_eq!(creature.initiative.value, FactValue::Missing);
    assert_eq!(creature.legacy_abilities.value, FactValue::Missing);
    assert_eq!(creature.defenses.value, FactValue::Missing);
}

#[test]
fn maxx_drift_is_preserved_as_typed_unsupported_without_coercing_maximum() {
    let source = parse(
        json!({
            "_id": "maxx-drift", "name": "Maxx Drift", "type": "npc",
            "system": {"resources": {"focus": {"max": 1, "maxx": 7, "value": 1}}}
        }),
        "fixture-pack:maxx-drift",
    );
    let converted = convert_npc_core(
        record_key("fixture-pack", "maxx-drift"),
        "packs/fixture-pack/maxx-drift.json",
        &source,
    )
    .expect("maxx drift conversion");
    let RecordBody::Creature(creature) = &converted.body;
    let focus = &creature.resources.value.as_value().expect("resources")[0];
    assert_eq!(
        focus.maximum,
        FactValue::Value(CreatureResourceAmount::Integer(1))
    );
    let drift = &focus.source_drift.as_value().expect("typed source drift")[0];
    assert_eq!(
        drift.field,
        CreatureUnsupportedSourceField::ResourceMaximumDrift
    );
    assert_eq!(drift.value.shape, UnsupportedSourceShape::Number);
    assert_eq!(drift.value.value, "7");
    assert_eq!(
        drift.value.reason,
        UnsupportedSourceReason::SourceFieldDrift
    );
    let diagnostic = converted
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.kind == NpcCoreDiagnosticKind::ResourceMaximumDrift)
        .expect("maxx diagnostic");
    assert_eq!(
        diagnostic.code,
        "atlas.npc_core.resource_maxx_source_drift.v1"
    );
    assert_eq!(diagnostic.record_key, "fixture-pack:maxx-drift");
    assert_eq!(diagnostic.source_path, "packs/fixture-pack/maxx-drift.json");
    assert_eq!(diagnostic.source_field, "$.system.resources.focus.maxx");
    assert_eq!(diagnostic.source_value, "7");
    assert_eq!(
        diagnostic.disposition,
        NpcCoreDiagnosticDisposition::TypedUnsupported
    );
    assert!(!diagnostic.source_path.starts_with('/'));
}

#[test]
fn malformed_and_unsupported_source_are_diagnosed_explicitly() {
    let malformed = parse_npc_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(
            "fixture-pack:malformed",
            "packs/fixture-pack/malformed.json",
        ),
        json!({
            "_id": "malformed",
            "name": "Malformed",
            "type": "npc",
            "system": {"attributes": {"ac": "28"}}
        }),
    )
    .expect_err("malformed AC must fail source DTO parsing");
    assert_eq!(malformed.json_path(), "$.system.attributes.ac");
    assert_eq!(malformed.expected_shape(), "object");

    let unsupported_size = parse(
        json!({
            "_id": "unsupported-size",
            "name": "Unsupported Size",
            "type": "npc",
            "system": {"traits": {"rarity": "common", "size": {"value": "colossal"}, "value": []}}
        }),
        "fixture-pack:unsupported-size",
    );
    let error = convert_npc_core(
        record_key("fixture-pack", "unsupported-size"),
        "packs/fixture-pack/unsupported-size.json",
        &unsupported_size,
    )
    .expect_err("closed size vocabulary must reject unsupported value");
    assert_eq!(error.source_field, "$.system.traits.size.value");

    let unsupported_open = parse(
        json!({
            "_id": "unsupported-open",
            "name": "Unsupported Open Values",
            "type": "npc",
            "system": {
                "attributes": {"adjustment": "mythic"},
                "details": {"alliance": ""},
                "initiative": {"statistic": "Perception"},
                "traits": {"rarity": "common", "size": {"value": "med"}, "value": []},
                "skills": {"or singing)": {"base": null}},
                "resources": {"focus": {"max": "one", "value": 0}}
            }
        }),
        "fixture-pack:unsupported-open",
    );
    let converted = convert_npc_core(
        record_key("fixture-pack", "unsupported-open"),
        "packs/fixture-pack/unsupported-open.json",
        &unsupported_open,
    )
    .expect("open values are preserved with diagnostics");
    assert!(
        !converted
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.source_field == "$.system.skills.or singing)")
    );
    assert!(converted.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == NpcCoreDiagnosticKind::UnsupportedLegacyShape
            && diagnostic.source_field == "$.system.resources.focus.max"
    }));
    for field in [
        "$.system.attributes.adjustment",
        "$.system.details.alliance",
        "$.system.initiative.statistic",
    ] {
        assert!(converted.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == NpcCoreDiagnosticKind::UnsupportedOpenValue
                && diagnostic.source_field == field
                && diagnostic.disposition == NpcCoreDiagnosticDisposition::TypedUnsupported
        }));
    }
    let RecordBody::Creature(creature) = &converted.body;
    let unmodeled = creature
        .skills
        .value
        .as_value()
        .expect("skills")
        .iter()
        .find(|skill| skill.kind == CreatureSkillKind::Unmodeled)
        .expect("unknown authored skill key is canonical evidence");
    assert_eq!(unmodeled.modifier, FactValue::Null);
    assert_eq!(unmodeled.source_entries[0].authored_key, "or singing)");
    assert_eq!(unmodeled.source_entries[0].modifier, FactValue::Null);
    let unmodeled_fact = unmodeled
        .unmodeled
        .as_value()
        .expect("named unmodeled skill fact");
    assert_eq!(unmodeled_fact.authored_key, "or singing)");
    assert_eq!(unmodeled_fact.base, FactValue::Null);
    assert_eq!(
        unmodeled_fact.reason,
        CreatureUnmodeledSkillReason::UnknownAuthoredKey
    );
    assert!(matches!(
        creature.adjustment.value,
        FactValue::Value(CreatureAdjustment::Unsupported(_))
    ));
    assert!(matches!(
        creature.source_alliance.value,
        FactValue::Value(CreatureSourceAlliance::Unsupported(_))
    ));
    assert!(matches!(
        creature
            .initiative
            .value
            .as_value()
            .expect("initiative")
            .statistic,
        FactValue::Value(CreatureInitiativeStatistic::Unsupported(_))
    ));
    let focus = &creature.resources.value.as_value().expect("resources")[0];
    assert!(matches!(
        focus.maximum,
        FactValue::Value(CreatureResourceAmount::Unsupported(_))
    ));
    assert_eq!(
        focus.serialized_value,
        FactValue::Value(CreatureResourceAmount::Integer(0))
    );
}

#[test]
fn pinned_approved_fixture_cores_match_the_source_contract() {
    let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
        return;
    };
    let source_root = Path::new(&source_root);

    let night_hag = convert_pinned(
        source_root,
        "pathfinder-bestiary",
        "WQy7HBUcgDLsfVJd",
        "packs/pathfinder-bestiary/night-hag.json",
    );
    let RecordBody::Creature(night_hag) = &night_hag.body;
    assert_eq!(night_hag.level.value, FactValue::Value(9));
    assert_eq!(night_hag.skills.value.as_value().expect("skills").len(), 6);

    let goblin = convert_pinned(
        source_root,
        "pathfinder-monster-core",
        "fLLKuOXwPq1Iq0U4",
        "packs/pathfinder-monster-core/goblin-warrior.json",
    );
    let RecordBody::Creature(goblin) = &goblin.body;
    assert_eq!(goblin.level.value, FactValue::Value(-1));
    assert_eq!(goblin.skills.value.as_value().expect("skills").len(), 4);

    let merchant = convert_pinned(
        source_root,
        "pathfinder-npc-core",
        "qCKNT6U8O0su578A",
        "packs/pathfinder-npc-core/artisan/merchant.json",
    );
    let RecordBody::Creature(merchant) = &merchant.body;
    let merchant_lore = merchant
        .skills
        .value
        .as_value()
        .expect("skills")
        .iter()
        .find(|skill| skill.kind == CreatureSkillKind::Lore)
        .expect("Mercantile Lore");
    assert_eq!(merchant_lore.label, "Mercantile Lore");
    assert_eq!(merchant_lore.modifier, FactValue::Value(12));

    let fortune_dragon = convert_pinned(
        source_root,
        "pathfinder-monster-core",
        "BvYVgvlTcRbim4Xb",
        "packs/pathfinder-monster-core/fortune-dragon-young-spellcaster.json",
    );
    let RecordBody::Creature(fortune_dragon) = &fortune_dragon.body;
    assert_eq!(
        fortune_dragon
            .movement
            .value
            .as_value()
            .expect("movement")
            .iter()
            .map(|speed| (&speed.mode, speed.value.as_value().copied()))
            .collect::<Vec<_>>(),
        [
            (&CreatureMovementMode::Land, Some(60)),
            (&CreatureMovementMode::Fly, Some(100)),
        ]
    );

    let acolyte = convert_pinned(
        source_root,
        "pathfinder-npc-core",
        "dwzAEnCBpDWnZBWr",
        "packs/pathfinder-npc-core/devotee/acolyte-of-iomedae.json",
    );
    let RecordBody::Creature(acolyte) = &acolyte.body;
    let focus = &acolyte.resources.value.as_value().expect("resources")[0];
    assert_eq!(focus.kind.as_str(), "focus");
    assert_eq!(
        focus.maximum,
        FactValue::Value(CreatureResourceAmount::Integer(1))
    );

    let oriole = convert_pinned(
        source_root,
        "curtain-call-bestiary",
        "NpIoefm1VZTTnR8h",
        "packs/curtain-call-bestiary/book-2-singer-stalker-skinsaw-man/oriole.json",
    );
    assert!(!oriole.diagnostics.iter().any(|diagnostic| {
        diagnostic.source_field.starts_with("$.system.skills.")
            && diagnostic.source_field != "$.system.skills.performance.special[0]"
    }));
    let RecordBody::Creature(oriole) = &oriole.body;
    assert_eq!(
        oriole
            .skills
            .value
            .as_value()
            .expect("skills")
            .iter()
            .filter(|skill| skill.kind == CreatureSkillKind::Unmodeled)
            .count(),
        3
    );
    let performance = oriole
        .skills
        .value
        .as_value()
        .expect("skills")
        .iter()
        .find(|skill| skill.kind == CreatureSkillKind::Performance)
        .expect("Performance");
    assert_eq!(
        performance.variants.as_value().expect("variant")[0].modifier,
        FactValue::Value(31)
    );

    let risen_nemesis = convert_pinned(
        source_root,
        "curtain-call-bestiary",
        "RUuUJtm8jAJ66z0Q",
        "packs/curtain-call-bestiary/book-3-bring-the-house-down/risen-nemesis.json",
    );
    let RecordBody::Creature(risen_nemesis) = &risen_nemesis.body;
    let resistance = &risen_nemesis
        .defenses
        .value
        .as_value()
        .expect("defenses")
        .resistances
        .as_value()
        .expect("resistances")[0];
    assert_eq!(resistance.value, FactValue::Value(20));
    assert_eq!(
        resistance.exceptions.as_value().expect("exceptions").len(),
        3
    );
    assert_eq!(resistance.double_vs.as_value().expect("double vs").len(), 1);
}

#[test]
fn pinned_actor_root_disposition_fixtures_match_exact_source_facts() {
    let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
        return;
    };
    let source_root = Path::new(&source_root);

    let priest = convert_pinned(
        source_root,
        "curtain-call-bestiary",
        "BgPshRvqaBvy4ulr",
        "packs/curtain-call-bestiary/book-2-singer-stalker-skinsaw-man/karumzek-priest-dispel-magic.json",
    );
    let RecordBody::Creature(priest) = &priest.body;
    let abilities = priest
        .legacy_abilities
        .value
        .as_value()
        .expect("legacy abilities");
    assert_eq!(
        [
            &abilities.strength,
            &abilities.dexterity,
            &abilities.constitution,
            &abilities.intelligence,
            &abilities.wisdom,
            &abilities.charisma,
        ],
        [
            &FactValue::Value(6),
            &FactValue::Value(6),
            &FactValue::Value(5),
            &FactValue::Value(2),
            &FactValue::Value(8),
            &FactValue::Value(6),
        ]
    );

    let barrel_launcher = convert_pinned(
        source_root,
        "agents-of-edgewatch-bestiary",
        "azyIfDNNW44jY8YX",
        "packs/agents-of-edgewatch-bestiary/book-5-belly-of-the-black-whale/barrel-launcher.json",
    );
    let RecordBody::Creature(barrel_launcher) = &barrel_launcher.body;
    assert_eq!(
        barrel_launcher.adjustment.value,
        FactValue::Value(CreatureAdjustment::Elite)
    );

    let siege_shard = convert_pinned(
        source_root,
        "agents-of-edgewatch-bestiary",
        "XDt87cqF85zWnlC8",
        "packs/agents-of-edgewatch-bestiary/book-1-devil-at-the-dreaming-palace/siege-shard.json",
    );
    let RecordBody::Creature(siege_shard) = &siege_shard.body;
    assert_eq!(
        siege_shard
            .defenses
            .value
            .as_value()
            .expect("defenses")
            .hardness,
        FactValue::Value(5)
    );

    let tar_tree = convert_pinned(
        source_root,
        "quest-for-the-frozen-flame-bestiary",
        "cnTAIcjKu01I7eJ0",
        "packs/quest-for-the-frozen-flame-bestiary/book-3-burning-tundra/arboreal-tar-tree.json",
    );
    let RecordBody::Creature(tar_tree) = &tar_tree.body;
    let shield = tar_tree
        .defenses
        .value
        .as_value()
        .expect("defenses")
        .shield
        .as_value()
        .expect("shield");
    assert_eq!(shield.armor_class_bonus, FactValue::Value(2));
    assert_eq!(shield.broken_threshold, FactValue::Value(0));
    assert_eq!(shield.hardness, FactValue::Value(0));
    assert_eq!(shield.maximum_hit_points, FactValue::Value(0));
    assert_eq!(shield.serialized_hit_points, FactValue::Value(0));

    let queen = convert_pinned(
        source_root,
        "spore-war-bestiary",
        "SxcJlOBouEOuH0AJ",
        "packs/spore-war-bestiary/book-3-a-voice-in-the-blight/queen-telandia-edasseril.json",
    );
    let RecordBody::Creature(queen) = &queen.body;
    let CreatureSourceAlliance::Named(alliance) = queen
        .source_alliance
        .value
        .as_value()
        .expect("source alliance")
    else {
        panic!("party alliance should be a source fact");
    };
    assert_eq!(alliance.as_str(), "party");

    let beluthus = convert_pinned(
        source_root,
        "abomination-vaults-bestiary",
        "3ry9WSvMMXHUe3kE",
        "packs/abomination-vaults-bestiary/abomination-vaults-hardcover-compilation/beluthus.json",
    );
    let RecordBody::Creature(beluthus) = &beluthus.body;
    let CreatureInitiativeStatistic::Named(statistic) = beluthus
        .initiative
        .value
        .as_value()
        .expect("initiative")
        .statistic
        .as_value()
        .expect("statistic")
    else {
        panic!("perception should be a named initiative statistic");
    };
    assert_eq!(statistic.as_str(), "perception");

    let herexen_conversion = convert_pinned(
        source_root,
        "outlaws-of-alkenstar-bestiary",
        "8jYqoyei7X2Bikb0",
        "packs/outlaws-of-alkenstar-bestiary/book-2-cradle-of-quartz/brighite-herexen.json",
    );
    let RecordBody::Creature(herexen) = &herexen_conversion.body;
    let focus = &herexen.resources.value.as_value().expect("resources")[0];
    assert_eq!(
        focus.maximum,
        FactValue::Value(CreatureResourceAmount::Integer(1))
    );
    assert_eq!(
        focus.serialized_value,
        FactValue::Value(CreatureResourceAmount::Integer(1))
    );
    assert_eq!(
        focus.source_drift.as_value().expect("maxx drift")[0]
            .value
            .value,
        "1"
    );
    assert!(herexen_conversion.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == NpcCoreDiagnosticKind::ResourceMaximumDrift
            && diagnostic.record_key == "outlaws-of-alkenstar-bestiary:8jYqoyei7X2Bikb0"
            && diagnostic.source_field == "$.system.resources.focus.maxx"
    }));
}

#[test]
fn pinned_component_id_failures_preserve_all_eight_parent_records() {
    let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
        return;
    };
    let source_root = Path::new(&source_root);
    let fixtures = [
        (
            "agents-of-edgewatch-bestiary",
            "vwxNCuBksHYU2Dwf",
            "packs/agents-of-edgewatch-bestiary/book-6-ruins-of-the-radiant-siege/hundun-chaos-mage.json",
            "Air Walk",
        ),
        (
            "agents-of-edgewatch-bestiary",
            "cLlOUUpCIAQwUuOP",
            "packs/agents-of-edgewatch-bestiary/book-6-ruins-of-the-radiant-siege/ilsetsya-wyrmtouched.json",
            "Freedom Of Movement",
        ),
        (
            "agents-of-edgewatch-bestiary",
            "RyXA4wOGY8lenKVw",
            "packs/agents-of-edgewatch-bestiary/book-6-ruins-of-the-radiant-siege/nenchuuj.json",
            "Air Walk",
        ),
        (
            "agents-of-edgewatch-bestiary",
            "rm0iJOMwruWSE93I",
            "packs/agents-of-edgewatch-bestiary/book-6-ruins-of-the-radiant-siege/olansa-terimor.json",
            "Spider Climb",
        ),
        (
            "extinction-curse-bestiary",
            "YZcw33uhsPHWKcHM",
            "packs/extinction-curse-bestiary/book-5-lord-of-the-black-sands/hollow-hush.json",
            "Air Walk",
        ),
        (
            "pathfinder-bestiary-2",
            "2Kw49I6EZbKKlTtK",
            "packs/pathfinder-bestiary-2/blizzardborn.json",
            "Ice Burrow",
        ),
        (
            "pathfinder-bestiary-2",
            "iW6WKMVV3Ug8sa2q",
            "packs/pathfinder-bestiary-2/thanadaemon.json",
            "Air Walk",
        ),
        (
            "the-slithering-bestiary",
            "wgeuaYe2fbGNg42z",
            "packs/the-slithering-bestiary/ahvothian.json",
            "Freedom Of Movement",
        ),
    ];

    for (pack, id, relative_path, source_type) in fixtures {
        let converted = convert_pinned(source_root, pack, id, relative_path);
        let RecordBody::Creature(creature) = &converted.body;
        assert_eq!(
            creature.identity.record_key.to_string(),
            format!("{pack}:{id}")
        );
        let speed = creature
            .movement
            .value
            .as_value()
            .expect("movement")
            .iter()
            .find(|speed| matches!(&speed.mode, CreatureMovementMode::Unsupported(value) if value.value == source_type))
            .unwrap_or_else(|| panic!("{relative_path} preserves {source_type}"));
        assert!(
            speed
                .id
                .as_str()
                .starts_with(&format!("speed:source:{pack}:{id}:"))
        );
        let diagnostic = converted
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.kind == NpcCoreDiagnosticKind::ComponentIdSourceFallback)
            .unwrap_or_else(|| panic!("{relative_path} has fallback diagnostic"));
        assert_eq!(diagnostic.record_key, format!("{pack}:{id}"));
        assert_eq!(diagnostic.source_path, relative_path);
        assert_eq!(diagnostic.source_value, source_type);
        assert_eq!(
            diagnostic.disposition,
            NpcCoreDiagnosticDisposition::StableSourceFallback
        );
    }
}

#[test]
fn pinned_full_source_load_preserves_all_records_and_component_id_parents() {
    let Some(source_root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
        return;
    };
    let source_root = Path::new(&source_root);
    let loaded = super::loader::load_foundry_source_records(source_root, None)
        .expect("exact pinned source loads");

    assert_eq!(loaded.source_record_count, 25_641);
    assert!(loaded.skipped_records.is_empty());
    assert_eq!(
        loaded.source_signature,
        "foundry-pf2e:sha256:dd78d67f5b6d25bf65e30ca4da66af76e7a31e1e7d990562f139154b1752603a"
    );

    let expected_keys = [
        "agents-of-edgewatch-bestiary:vwxNCuBksHYU2Dwf",
        "agents-of-edgewatch-bestiary:cLlOUUpCIAQwUuOP",
        "agents-of-edgewatch-bestiary:RyXA4wOGY8lenKVw",
        "agents-of-edgewatch-bestiary:rm0iJOMwruWSE93I",
        "extinction-curse-bestiary:YZcw33uhsPHWKcHM",
        "pathfinder-bestiary-2:2Kw49I6EZbKKlTtK",
        "pathfinder-bestiary-2:iW6WKMVV3Ug8sa2q",
        "the-slithering-bestiary:wgeuaYe2fbGNg42z",
    ];
    for expected_key in expected_keys {
        let parent = loaded
            .records
            .iter()
            .find(|record| record.record.identity.key.to_string() == expected_key)
            .unwrap_or_else(|| panic!("{expected_key} is represented"));
        assert!(parent.facts.canonical_body.is_some());
        assert!(parent.facts.npc_source.is_some());
        assert!(parent.facts.npc_core_diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == NpcCoreDiagnosticKind::ComponentIdSourceFallback
                && diagnostic.record_key == expected_key
                && !diagnostic.source_path.starts_with('/')
        }));
    }

    let fallback_diagnostics = loaded
        .records
        .iter()
        .flat_map(|record| &record.facts.npc_core_diagnostics)
        .filter(|diagnostic| diagnostic.kind == NpcCoreDiagnosticKind::ComponentIdSourceFallback)
        .count();
    assert_eq!(fallback_diagnostics, 8);

    let herexen = loaded
        .records
        .iter()
        .find(|record| {
            record.record.identity.key.to_string()
                == "outlaws-of-alkenstar-bestiary:8jYqoyei7X2Bikb0"
        })
        .expect("maxx parent is represented");
    assert!(herexen.facts.npc_core_diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == NpcCoreDiagnosticKind::ResourceMaximumDrift
            && diagnostic.source_path
                == "packs/outlaws-of-alkenstar-bestiary/book-2-cradle-of-quartz/brighite-herexen.json"
    }));
}

fn assert_iwr(
    entries: &[atlas_record::CreatureIwr],
    kind: CreatureIwrKind,
    iwr_type: &str,
    value: Option<i64>,
) {
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, kind);
    assert_eq!(entries[0].iwr_type.as_str(), iwr_type);
    assert_eq!(entries[0].value.as_value().copied(), value);
    assert_eq!(entries[0].authored_order, 0);
}

fn parse(raw: Value, key: &str) -> super::dto::VersionedNpcSource {
    parse_npc_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(key, format!("packs/{key}.json")),
        raw,
    )
    .expect("valid NPC source")
}

fn record_key(pack: &str, id: &str) -> RecordKey {
    RecordKey::new(
        PackName::new(pack.to_string()).expect("pack name"),
        RecordId::new(id.to_string()).expect("record id"),
    )
}

fn convert_pinned(
    source_root: &Path,
    pack: &str,
    id: &str,
    relative_path: &str,
) -> super::npc_core::NpcCoreConversion {
    let raw = std::fs::read_to_string(source_root.join(relative_path))
        .unwrap_or_else(|error| panic!("read pinned fixture {relative_path}: {error}"));
    let raw = serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("parse pinned fixture {relative_path}: {error}"));
    let source = parse_npc_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(format!("{pack}:{id}"), relative_path),
        raw,
    )
    .unwrap_or_else(|error| panic!("source DTO {relative_path}: {error}"));
    convert_npc_core(record_key(pack, id), relative_path, &source)
        .unwrap_or_else(|error| panic!("canonical core {relative_path}: {error}"))
}

fn night_hag_source() -> Value {
    json!({
        "_id": "WQy7HBUcgDLsfVJd",
        "name": "Night Hag",
        "type": "npc",
        "system": {
            "attributes": {
                "ac": {"details": "", "value": 28},
                "allSaves": {"value": "+2 status to all saves vs. magic, -2 to all saves (if heartstone is lost)"},
                "hp": {"details": "", "max": 170, "temp": 0, "value": 170},
                "immunities": [{"type": "sleep"}],
                "resistances": [{"type": "mental", "value": 10}],
                "speed": {"otherSpeeds": [], "value": 25},
                "weaknesses": [{"type": "cold-iron", "value": 10}]
            },
            "details": {
                "level": {"value": 9},
                "languages": {"details": "", "value": ["aklo", "chthonian", "common", "diabolic", "empyrean"]},
                "publication": {"license": "OGL", "remaster": false, "title": "Pathfinder Bestiary"}
            },
            "perception": {"details": "", "mod": 18, "senses": [{"type": "darkvision"}]},
            "resources": {"focus": {"max": 1, "value": 1}},
            "saves": {
                "fortitude": {"saveDetail": "", "value": 19},
                "reflex": {"saveDetail": "", "value": 17},
                "will": {"saveDetail": "", "value": 18}
            },
            "skills": {
                "arcana": {"base": 18},
                "deception": {"base": 18},
                "diplomacy": {"base": 18},
                "intimidation": {"base": 14},
                "occultism": {"base": 20},
                "religion": {"base": 20}
            },
            "traits": {"rarity": "common", "size": {"value": "med"}, "value": ["evil", "fiend", "hag", "humanoid", "unholy"]}
        },
        "items": []
    })
}

fn dense_source() -> Value {
    json!({
        "_id": "dense",
        "name": "Dense Fixture",
        "type": "npc",
        "system": {
            "attributes": {
                "ac": {"details": "", "value": 43},
                "allSaves": {"value": "+1 vs. controlled"},
                "hp": {"details": "void healing", "max": 290, "temp": 0, "value": 290},
                "resistances": [{"applyOnce": true, "doubleVs": ["non-magical"], "exceptions": ["force", "ghost-touch", "vitality"], "type": "all-damage", "value": 20}],
                "speed": {"otherSpeeds": [{"type": "fly", "value": 30}], "value": 0}
            },
            "details": {
                "level": {"value": 20},
                "languages": {"details": "telepathy 100 feet", "value": ["common", "necril"]},
                "publication": {"license": "ORC", "remaster": true, "title": "Fixture Book"}
            },
            "perception": {"details": "", "mod": 35, "senses": [{"type": "greater-darkvision"}, {"acuity": "imprecise", "range": 60, "type": "scent"}]},
            "resources": {},
            "saves": {"fortitude": {"saveDetail": "", "value": 33}, "reflex": {"saveDetail": "", "value": 33}, "will": {"saveDetail": "", "value": 33}},
            "skills": {"performance": {"base": 29, "note": "stage only", "special": [{"base": 31, "label": "when singing", "predicate": ["action:perform", {"or": ["mode:sing", "mode:orate"]}]}]}},
            "traits": {"rarity": "unique", "size": {"value": "med"}, "value": ["undead"]}
        },
        "items": [{
            "_id": "lore-source-id",
            "name": "Theater Lore",
            "type": "lore",
            "sort": 6000000,
            "system": {"mod": {"value": 25}}
        }]
    })
}
