use atlas_domain::{PackName, PublicationCategory, Rarity, RecordId, RecordKey, RecordKind};

use crate::*;

const SOURCE_COMMIT: &str = "4cbdaa37d6c33e9519561bae2c59a23e0288cbce";
const HIDDEN_PIT_PATH: &str = "packs/hazards/hidden-pit.json";

#[test]
fn hazard_conveniences_are_central_guarded_and_noncanonical() {
    let mut hazard = hidden_pit_source_fixture();
    let projection = project_hazard_conveniences(&hazard);
    assert_eq!(projection.rule_id, HAZARD_CONVENIENCE_RULE_ID);
    assert_eq!(projection.rule_version, HAZARD_CONVENIENCE_RULE_VERSION);
    assert_eq!(projection.detection_dc, Some(18));
    assert_eq!(projection.broken_threshold, Some(6));
    assert_eq!(projection.initiative_suggestion, None);

    hazard.complexity = typed(HazardComplexity::Complex, "/system/details/isComplex");
    let projection = project_hazard_conveniences(&hazard);
    assert_eq!(
        projection.initiative_suggestion,
        Some(HazardInitiativeSuggestion {
            statistic: HazardInitiativeStatistic::Stealth,
            modifier: 8,
        })
    );

    let detection = hazard.detection.typed().expect("typed detection").clone();
    hazard.detection = typed(
        HazardDetection {
            stealth_modifier: HazardFact::source(
                FactValue::Null,
                "/system/attributes/stealth/value",
            ),
            ..detection
        },
        "/system/attributes/stealth",
    );
    let hit_points = hazard
        .defenses
        .typed()
        .and_then(|defenses| defenses.hit_points.typed())
        .expect("typed hit points")
        .clone();
    let mut defenses = hazard.defenses.typed().expect("typed defenses").clone();
    defenses.hit_points = typed(
        HazardHitPoints {
            maximum: unsupported(
                "\"many\"",
                HazardExpectedShape::Integer,
                HazardSourceShape::String,
                "/system/attributes/hp/max",
                HazardUnsupportedOwner::Record(hazard.identity.record_key.clone()),
                HazardDiagnosticCode::UnexpectedShape,
            ),
            ..hit_points
        },
        "/system/attributes/hp",
    );
    hazard.defenses = typed(defenses, "/system/attributes");

    let guarded = project_hazard_conveniences(&hazard);
    assert_eq!(guarded.detection_dc, None);
    assert_eq!(guarded.broken_threshold, None);
    assert_eq!(guarded.initiative_suggestion, None);
}

#[test]
fn hidden_pit_uses_canonical_hazard_presentation_and_fts() {
    let mut record = atlas_record_for(&hidden_pit_source_fixture());
    record.mechanics.document = FoundryDocumentMechanics::Actor(ActorMechanics {
        size: Some("grg".to_string()),
        is_complex: true,
        ..ActorMechanics::default()
    });
    record.mechanics.metrics.push(MetricRow {
        domain: atlas_domain::MetricDomain::Actor,
        key: "stealth.mod".to_string(),
        value: MetricValue::Number(99.0),
    });
    let hazard = hidden_pit_source_fixture();
    assert_eq!(hazard.identity.source_id.as_str(), "BHq5wpQU8hQEke8D");
    assert_eq!(
        hazard
            .traits
            .typed()
            .expect("traits")
            .iter()
            .map(HazardTrait::as_str)
            .collect::<Vec<_>>(),
        vec!["mechanical", "trap"]
    );
    let detection = hazard.detection.typed().expect("detection");
    assert_eq!(
        detection.details.typed().map(render_plain_text).as_deref(),
        Some("(or 0 if the trapdoor is disabled or broken)")
    );
    let lifecycle = hazard.lifecycle.typed().expect("lifecycle");
    assert_eq!(
        lifecycle.reset.typed().map(render_plain_text).as_deref(),
        Some(
            "Creatures can still fall into the trap, but the trapdoor must be reset manually for the trap to become hidden again."
        )
    );
    let embedded = hazard.embedded_entities.typed().expect("entities");
    assert_eq!(embedded.entities[0].id.as_str(), "lY83oUjx0DLxDByK");
    assert_eq!(embedded.entities[0].label, "Pitfall");
    let action_content = hazard
        .content
        .documents
        .iter()
        .find(|document| {
            matches!(document.owner, ContentOwner::HazardOccurrence(_))
                && document.source_kind == ContentSourceKind::EmbeddedItemDescription
        })
        .expect("source-faithful Pitfall content");
    assert!(render_plain_text(&action_content.document).contains(
        "The triggering creature falls in and takes falling damage (typically 10[bludgeoning] damage)."
    ));
    assert_eq!(action_content.reference_occurrences.len(), 1);
    let reference = &action_content.reference_occurrences[0];
    assert_eq!(reference.ordinal, 0);
    assert!(matches!(reference.owner, ContentOwner::HazardOccurrence(_)));
    assert!(matches!(
        &reference.target,
        RichLinkTarget::Unresolved { target, fallback_label }
            if target == "Compendium.pf2e.actionspf2e.Item.Grab an Edge"
                && fallback_label == "Grab an Edge"
    ));
    let RichNode::HtmlElement { children, .. } = &action_content.document.nodes[4] else {
        panic!("Pitfall effect paragraph")
    };
    let RichNode::FoundryLink { link } = &children[4] else {
        panic!("Grab an Edge reference")
    };
    assert_eq!(link.source.macro_kind, FoundryLinkMacroKind::Uuid);
    assert_eq!(
        link.source.authored_target,
        "Compendium.pf2e.actionspf2e.Item.Grab an Edge"
    );
    assert_eq!(
        reference.provenance.nested_source_id.as_deref(),
        Some("lY83oUjx0DLxDByK")
    );
    let body = RecordBody::Hazard(hazard.clone());

    let presentation =
        build_search_presentation_document_with_content_filter(&record, Some(&body), None, |_| {
            true
        });
    assert_eq!(presentation.title, "Hidden Pit");
    assert!(has_fact(&presentation, "Detection DC", "18"));
    assert!(has_fact(&presentation, "Armor Class", "10"));
    assert!(has_fact(&presentation, "Hardness", "3"));
    assert!(has_fact(&presentation, "Broken Threshold", "6"));
    assert!(has_fact(&presentation, "Will", "+0"));
    assert!(!has_fact(
        &presentation,
        "Image",
        "systems/pf2e/icons/default-icons/hazard.svg"
    ));
    assert!(!has_fact(
        &presentation,
        "Image",
        "systems/pf2e/icons/actions/Reaction.webp"
    ));
    assert!(!has_fact(&presentation, "License", "ORC"));
    assert!(!has_fact(&presentation, "License", "OGL"));
    assert!(!has_fact(
        &presentation,
        "Source Rarity (unsupported)",
        "common"
    ));
    assert!(
        presentation
            .badges
            .iter()
            .any(|badge| { badge.label == "Rarity" && badge.value == "common" })
    );
    assert!(!has_fact(&presentation, "Detection DC", "109"));

    let fts = build_search_fts_projection(&record, &[], Some(&body), None);
    assert!(fts.mechanic_terms.contains("simple hazard"));
    for provenance_only in [
        "systems/pf2e/icons/default-icons/hazard.svg",
        "systems/pf2e/icons/actions/Reaction.webp",
        "ORC",
        "OGL",
        "pitfall",
    ] {
        assert!(
            !fts.mechanic_terms.contains(provenance_only),
            "source metadata must not become a mechanic/FTS term: {provenance_only}"
        );
    }
    assert!(fts.metric_terms.contains("Stealth DC"));
    assert!(fts.facts.contains("Thievery DC 12"));
    assert!(
        fts.embedded_content
            .contains("triggering creature falls in")
    );
    assert!(fts.references.contains("Grab an Edge"));
    assert!(
        fts.references
            .contains("Compendium.pf2e.actionspf2e.Item.Grab an Edge")
    );
    assert!(!fts.mechanic_terms.contains("gargantuan"));
    assert!(!fts.metric_terms.contains("99"));

    let projected = project_hazard_facts(&hazard);
    for key in [
        "disable.dc.min",
        "disable.dc.max",
        "disable.thievery.dc.min",
        "disable.thievery.dc.max",
    ] {
        assert!(
            projected
                .metrics
                .iter()
                .any(|metric| { metric.key == key && metric.value == MetricValue::Number(12.0) }),
            "missing Hidden Pit structured disable metric {key}"
        );
    }
    assert!(
        projected
            .metrics
            .iter()
            .all(|metric| !metric.key.contains("rank")),
        "adjacent prose rank must not become a metric"
    );
}

#[test]
fn plain_text_disable_dc_and_rank_do_not_create_metrics() {
    let mut hazard = hidden_pit_source_fixture();
    let mut lifecycle = hazard.lifecycle.typed().expect("lifecycle").clone();
    lifecycle.disable = typed(
        paragraph_document("DC 40 master Arcana or expert Thievery"),
        "/system/details/disable",
    );
    hazard.lifecycle = typed(lifecycle, "/system/details");

    let projected = project_hazard_facts(&hazard);
    assert!(
        projected
            .metrics
            .iter()
            .all(|metric| !metric.key.starts_with("disable.")),
        "plain-text DCs, skills, and ranks must not become disable metrics"
    );
}

#[test]
fn hazard_action_model_is_schema_bounded_and_owner_identity_is_explicit() {
    let hazard = hidden_pit_source_fixture();
    let embedded = hazard.embedded_entities.typed().expect("entities");
    let HazardCapability::Action(action) = &embedded.entities[0].capability else {
        panic!("action")
    };
    assert_eq!(
        action.action_type.typed(),
        Some(&HazardActionType::Reaction)
    );
    assert!(matches!(action.actions.value, FactValue::Null));
    assert!(matches!(action.category.value, FactValue::Null));
    assert!(matches!(action.death_note.value, FactValue::Missing));
    assert!(matches!(action.frequency.value, FactValue::Null));
    assert!(matches!(action.self_effect.value, FactValue::Null));
    assert!(matches!(
        hazard
            .content
            .documents
            .last()
            .map(|document| &document.owner),
        Some(ContentOwner::HazardOccurrence(_))
    ));
}

#[test]
fn hazard_projection_includes_structured_action_and_iwr_details() {
    let mut hazard = hidden_pit_source_fixture();
    let FactValue::Value(HazardSourceValue::Typed(embedded)) = &mut hazard.embedded_entities.value
    else {
        panic!("typed embedded entities")
    };
    let HazardCapability::Action(action) = &mut embedded.entities[0].capability else {
        panic!("action")
    };
    action.actions = typed(HazardActionCount::One, "/items/0/system/actions/value");
    action.category = typed(HazardActionCategory::Offensive, "/items/0/system/category");
    action.death_note = typed(true, "/items/0/system/deathNote");
    action.frequency = typed(
        HazardFrequency {
            value: typed(0, "/items/0/system/frequency/value"),
            maximum: typed(1, "/items/0/system/frequency/max"),
            per: typed(
                HazardFrequencyInterval::Round,
                "/items/0/system/frequency/per",
            ),
        },
        "/items/0/system/frequency",
    );
    action.self_effect = typed(
        HazardSelfEffect {
            target_uuid: typed(
                "Compendium.pf2e.conditionitems.Item.Prone".to_string(),
                "/items/0/system/selfEffect/uuid",
            ),
            label: typed("Prone".to_string(), "/items/0/system/selfEffect/name"),
        },
        "/items/0/system/selfEffect",
    );
    action.common.slug = typed("pitfall".to_string(), "/items/0/system/slug");
    action.common.traits = typed(
        vec![HazardTrait::new("attack").expect("trait")],
        "/items/0/system/traits/value",
    );

    let defenses = hazard.defenses.typed().expect("defenses").clone();
    let mut immunities = defenses.immunities.typed().expect("immunities").clone();
    immunities[0].exceptions = typed(
        vec!["adamantine".to_string()],
        "/system/attributes/immunities/0/exceptions",
    );
    immunities[0].double_vs = typed(
        vec!["wood".to_string()],
        "/system/attributes/immunities/0/doubleVs",
    );
    let resistance = HazardIwr {
        id: HazardComponentId::new("resistance-0").expect("component id"),
        authored_order: 0,
        iwr_type: typed(
            "electricity".to_string(),
            "/system/attributes/resistances/0/type",
        ),
        value: typed(15, "/system/attributes/resistances/0/value"),
        exceptions: HazardFact::source(
            FactValue::Missing,
            "/system/attributes/resistances/0/exceptions",
        ),
        double_vs: HazardFact::source(
            FactValue::Missing,
            "/system/attributes/resistances/0/doubleVs",
        ),
    };
    hazard.defenses = typed(
        HazardDefenses {
            immunities: typed(immunities, "/system/attributes/immunities"),
            resistances: typed(vec![resistance], "/system/attributes/resistances"),
            ..defenses
        },
        "/system/attributes",
    );

    let presentation = build_hazard_presentation_document(&hazard, |_| true);
    assert!(has_fact(&presentation, "Pitfall", "Reaction"));
    assert!(has_fact(&presentation, "Actions", "1"));
    assert!(has_fact(&presentation, "Category", "Offensive"));
    assert!(has_fact(&presentation, "Death Note", "yes"));
    assert!(has_fact(&presentation, "Frequency Value", "0"));
    assert!(has_fact(&presentation, "Frequency Maximum", "1"));
    assert!(has_fact(&presentation, "Frequency Period", "round"));
    assert!(has_fact(&presentation, "Self Effect", "Prone"));
    assert!(has_fact(&presentation, "Traits", "attack"));
    assert!(has_fact(
        &presentation,
        "Immunities",
        "critical-hits (except adamantine) (double vs wood), precision"
    ));
    assert!(has_fact(&presentation, "Resistances", "electricity 15"));

    let facts = project_hazard_facts(&hazard);
    for expected in [
        "Pitfall",
        "reaction",
        "1 actions",
        "offensive action",
        "death note true",
        "frequency value 0",
        "frequency maximum 1",
        "frequency per round",
        "Compendium.pf2e.conditionitems.Item.Prone",
        "attack",
        "except adamantine",
        "double vs wood",
        "resistance electricity 15",
    ] {
        assert!(
            facts.mechanic_terms.iter().any(|value| value == expected),
            "missing structured term {expected}"
        );
    }
    let fts = build_search_fts_projection(
        &atlas_record_for(&hazard),
        &[],
        Some(&RecordBody::Hazard(hazard.clone())),
        None,
    );
    for expected in [
        "frequency maximum 1",
        "Compendium.pf2e.conditionitems.Item.Prone",
        "except adamantine",
        "double vs wood",
        "resistance electricity 15",
    ] {
        assert!(
            fts.mechanic_terms.contains(expected),
            "missing FTS term {expected}"
        );
    }
}

#[test]
fn hazard_strike_retains_ordered_damage_and_named_source_metadata() {
    let mut hazard = hidden_pit_source_fixture();
    let owner_record_key = hazard.identity.record_key.clone();
    let FactValue::Value(HazardSourceValue::Typed(embedded)) = &mut hazard.embedded_entities.value
    else {
        panic!("typed embedded entities")
    };
    let entity_id = HazardEntityId::new("synthetic-strike").expect("entity id");
    let occurrence_id = HazardOccurrenceId::new("synthetic-strike").expect("occurrence id");
    embedded.entities.push(HazardEntity {
        id: entity_id.clone(),
        family: HazardEntityFamily::Strike,
        label: "Scythe".to_string(),
        image: typed("icons/scythe.webp".to_string(), "/items/1/img"),
        source_identity: HazardEntitySourceIdentity::Stable {
            source_id: HazardSourceId::new("synthetic-strike").expect("source id"),
        },
        capability: HazardCapability::Strike(Box::new(HazardStrikeCapability {
            common: item_common(),
            bonus: typed(8, "/items/1/system/bonus/value"),
            attack_effects: typed(Vec::new(), "/items/1/system/attackEffects/value"),
            damage_rolls: typed(
                vec![
                    strike_damage("0", 0, "1d8", "slashing"),
                    strike_damage("1", 1, "1d6", "fire"),
                ],
                "/items/1/system/damageRolls",
            ),
            source_metadata: HazardStrikeSourceMetadata {
                attack: typed(8, "/items/1/system/attack/value"),
                weapon_type: typed(
                    HazardSourceAttackMode::Melee,
                    "/items/1/system/weaponType/value",
                ),
                attack_effects_custom: typed(String::new(), "/items/1/system/attackEffects/custom"),
            },
            unsupported_fields: Vec::new(),
        })),
    });
    embedded.occurrences.push(HazardEntityOccurrence {
        id: occurrence_id,
        owner_record_key,
        entity_id,
        family: HazardEntityFamily::Strike,
        authored_order: 1,
        source_sort: typed(200_000, "/items/1/sort"),
        source_folder: HazardFact::source(FactValue::Null, "/items/1/folder"),
        source_ordinal: 1,
        contextual_label: typed("Scythe".to_string(), "/items/1/name"),
        identity_stability: HazardOccurrenceIdentityStability::StableSourceIdentity,
    });

    let embedded = hazard.embedded_entities.typed().expect("embedded entities");
    let HazardCapability::Strike(strike) = &embedded.entities[1].capability else {
        panic!("strike")
    };
    let damage = strike.damage_rolls.typed().expect("damage rolls");
    assert_eq!(
        damage
            .iter()
            .map(|value| value.source_key.as_str())
            .collect::<Vec<_>>(),
        vec!["0", "1"]
    );
    assert!(strike.unsupported_fields.is_empty());
    assert_eq!(strike.source_metadata.attack.typed(), Some(&8));

    let presentation = build_hazard_presentation_document(&hazard, |_| true);
    assert!(has_fact(&presentation, "Strike Bonus", "+8"));
    assert!(has_fact(&presentation, "Damage 0", "1d8 slashing"));
    assert!(has_fact(&presentation, "Damage 1", "1d6 fire"));
    let facts = project_hazard_facts(&hazard);
    for expected in [
        "hazard strike",
        "strike bonus +8",
        "damage key 0",
        "1d8",
        "slashing",
        "damage key 1",
        "1d6",
        "fire",
    ] {
        assert!(
            facts.mechanic_terms.iter().any(|value| value == expected),
            "missing structured strike term {expected}"
        );
    }
}

#[test]
fn bounded_hazard_consistency_and_strike_rules_use_only_typed_family_owners() {
    let mut hazard = hidden_pit_source_fixture();
    assert_eq!(
        project_hazard_has_health_consistency(&hazard),
        Some(HazardHasHealthConsistency::Consistent)
    );
    let mut defenses = hazard.defenses.typed().expect("defenses").clone();
    defenses.source_metadata.has_health = typed(false, "/system/attributes/hasHealth");
    hazard.defenses = typed(defenses.clone(), "/system/attributes");
    assert_eq!(
        project_hazard_has_health_consistency(&hazard),
        Some(HazardHasHealthConsistency::Conflict {
            source_has_health: false,
            derived_has_health: true,
        })
    );
    let mut hit_points = defenses.hit_points.typed().expect("hp").clone();
    hit_points.maximum = typed(0, "/system/attributes/hp/max");
    defenses.hit_points = typed(hit_points, "/system/attributes/hp");
    hazard.defenses = typed(defenses.clone(), "/system/attributes");
    assert_eq!(
        project_hazard_has_health_consistency(&hazard),
        Some(HazardHasHealthConsistency::Consistent)
    );
    defenses.source_metadata.has_health =
        HazardFact::source(FactValue::Null, "/system/attributes/hasHealth");
    hazard.defenses = typed(defenses, "/system/attributes");
    assert_eq!(project_hazard_has_health_consistency(&hazard), None);

    let mut strike = synthetic_strike_entity();
    assert_eq!(
        project_hazard_attack_mode(&strike).map(|projection| projection.mode),
        Some(HazardAttackMode::Melee)
    );
    assert_eq!(
        project_hazard_weapon_type_consistency(&strike),
        Some(HazardWeaponTypeConsistency::Consistent)
    );
    let cost = project_hazard_strike_action_cost(&strike).expect("Strike family cost");
    assert_eq!(cost.cost, HazardActionCount::One);
    assert_eq!(cost.basis, HazardEntityFamily::Strike);

    let HazardCapability::Strike(capability) = &mut strike.capability else {
        panic!("strike")
    };
    capability.common.traits = typed(
        vec![HazardTrait::new("range-120").expect("trait")],
        "/items/1/system/traits/value",
    );
    assert_eq!(
        project_hazard_attack_mode(&strike).map(|projection| projection.mode),
        Some(HazardAttackMode::Ranged)
    );
    assert_eq!(
        project_hazard_weapon_type_consistency(&strike),
        Some(HazardWeaponTypeConsistency::Conflict {
            source_weapon_type: HazardSourceAttackMode::Melee,
            derived_mode: HazardAttackMode::Ranged,
        })
    );

    let HazardCapability::Strike(capability) = &mut strike.capability else {
        panic!("strike")
    };
    capability.common.traits =
        HazardFact::source(FactValue::Missing, "/items/1/system/traits/value");
    capability.source_metadata.weapon_type = typed(
        HazardSourceAttackMode::Ranged,
        "/items/1/system/weaponType/value",
    );
    strike.label = "Misleading Ranged Strike".to_string();
    assert_eq!(project_hazard_attack_mode(&strike), None);
    assert_eq!(project_hazard_weapon_type_consistency(&strike), None);
    assert!(project_hazard_strike_action_cost(&strike).is_some());

    let HazardCapability::Strike(capability) = &mut strike.capability else {
        panic!("strike")
    };
    capability.common.traits = HazardFact::source(FactValue::Null, "/items/1/system/traits/value");
    assert_eq!(project_hazard_attack_mode(&strike), None);
    let HazardCapability::Strike(capability) = &mut strike.capability else {
        panic!("strike")
    };
    capability.common.traits = unsupported(
        "\"range-120\"",
        HazardExpectedShape::Array,
        HazardSourceShape::String,
        "/items/1/system/traits/value",
        HazardUnsupportedOwner::Entity(strike.id.clone()),
        HazardDiagnosticCode::UnexpectedShape,
    );
    assert_eq!(project_hazard_attack_mode(&strike), None);
    assert!(project_hazard_strike_action_cost(&strike).is_some());

    let strike_owner = strike.id.clone();
    let HazardCapability::Strike(capability) = &mut strike.capability else {
        panic!("strike")
    };
    capability.common.traits = typed(Vec::new(), "/items/1/system/traits/value");
    capability.source_metadata.weapon_type = unsupported(
        "\"future\"",
        HazardExpectedShape::ClosedVocabulary,
        HazardSourceShape::String,
        "/items/1/system/weaponType/value",
        HazardUnsupportedOwner::Entity(strike_owner),
        HazardDiagnosticCode::InvalidCanonicalValue,
    );
    assert_eq!(
        project_hazard_attack_mode(&strike).map(|projection| projection.mode),
        Some(HazardAttackMode::Melee)
    );
    assert_eq!(project_hazard_weapon_type_consistency(&strike), None);

    let mut action = hidden_pit_source_fixture()
        .embedded_entities
        .typed()
        .expect("embedded")
        .entities[0]
        .clone();
    action.label = "Strike".to_string();
    assert!(project_hazard_strike_action_cost(&action).is_none());
}

#[test]
fn hazard_source_metadata_projection_preserves_values_and_localizes_only_actionable_issues() {
    let benign = hazard_source_metadata_fixture(false);
    let benign_projection = project_hazard_source_metadata(&benign);
    assert!(benign_projection.issues.is_empty());
    let benign_json = serde_json::to_value(&benign_projection.facts).expect("metadata JSON");
    let benign_facts = benign_json.as_array().expect("metadata facts");
    let fact = |field: &str| {
        benign_facts
            .iter()
            .find(|fact| fact["field"] == field)
            .unwrap_or_else(|| panic!("missing {field} source metadata"))
    };
    assert_eq!(
        fact("token_name")["value"]["value"]["value"]["value"],
        "Hidden Pit"
    );
    assert_eq!(
        fact("token_name")["value"]["provenance"]["relativeSourcePath"],
        "/prototypeToken/name"
    );
    assert_eq!(fact("has_health")["value"]["value"]["value"]["value"], true);
    assert_eq!(
        fact("temporary_maximum")["value"]["value"]["value"]["value"],
        0
    );
    assert_eq!(fact("save_detail")["value"]["value"]["value"]["value"], "");
    assert_eq!(
        fact("item_rarity")["value"]["value"]["value"]["value"],
        "common"
    );
    assert_eq!(
        fact("item_lineage")["value"]["value"]["value"]["value"]["compendiumSource"]["value"]["state"],
        "null"
    );
    assert_eq!(fact("strike_attack")["value"]["value"]["state"], "missing");

    let actionable = hazard_source_metadata_fixture(true);
    let projection = project_hazard_source_metadata(&actionable);
    assert_eq!(projection.issues.len(), 7);
    for field in [
        "provenance.token.name",
        "defenses.source_metadata.has_health",
        "defenses.hit_points.source_metadata.temporary_maximum",
        "defenses.saves.source_metadata.fortitude_detail",
        "activity.source_metadata.rarity",
        "activity.strike.source_metadata.weapon_type",
        "activity.strike.source_metadata.attack_effects_custom",
    ] {
        assert_eq!(
            projection
                .issues
                .iter()
                .filter(|issue| issue.field_key() == field)
                .count(),
            1,
            "{field} should produce one localized issue"
        );
    }
    let facts = serde_json::to_value(&projection.facts).expect("metadata JSON");
    let facts = facts.as_array().expect("metadata facts");
    let has_health = facts
        .iter()
        .find(|fact| fact["field"] == "has_health")
        .expect("has-health metadata");
    assert_eq!(
        has_health["value"]["value"]["value"]["value"], false,
        "a conflicting false source value must remain exact"
    );
    let token = facts
        .iter()
        .find(|fact| fact["field"] == "token_name")
        .expect("token metadata");
    assert_eq!(token["value"]["value"]["value"]["value"]["exactJson"], "17");
    assert_eq!(
        token["value"]["value"]["value"]["value"]["relativeSourcePath"],
        "/prototypeToken/name"
    );
    let rarity = facts
        .iter()
        .find(|fact| fact["field"] == "item_rarity" && fact["entity_id"] == "lY83oUjx0DLxDByK")
        .expect("child rarity metadata");
    assert_eq!(
        rarity["value"]["value"]["value"]["value"]["exactJson"],
        "42"
    );
    assert_eq!(
        rarity["value"]["value"]["value"]["value"]["relativeSourcePath"],
        "/items/0/system/traits/rarity"
    );
    let custom = facts
        .iter()
        .find(|fact| fact["field"] == "strike_attack_effects_custom")
        .expect("custom attack effects metadata");
    assert_eq!(custom["value"]["value"]["value"]["value"], "corrosive mist");
    let entity_fact_order = facts
        .iter()
        .filter_map(|fact| {
            fact.get("entity_id").map(|entity_id| {
                (
                    fact["field"].as_str().expect("field"),
                    entity_id.as_str().expect("entity id"),
                )
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        entity_fact_order,
        vec![
            ("item_rarity", "lY83oUjx0DLxDByK"),
            ("item_lineage", "lY83oUjx0DLxDByK"),
            ("item_rarity", "synthetic-strike-rule"),
            ("item_lineage", "synthetic-strike-rule"),
            ("strike_attack", "synthetic-strike-rule"),
            ("strike_weapon_type", "synthetic-strike-rule"),
            ("strike_attack_effects_custom", "synthetic-strike-rule"),
        ],
        "source-authored entity order must remain stable in provenance"
    );
}

#[test]
fn hazard_record_json_uses_canonical_source_metadata_for_provenance_and_availability() {
    let benign_hazard = hazard_source_metadata_fixture(false);
    let benign = RetrievedRecord {
        record: atlas_record_for(&benign_hazard),
        body: Some(RecordBody::Hazard(benign_hazard)),
        spell_children: Vec::new(),
        consumable_occurrences: Default::default(),
    };
    let ordinary = serde_json::to_value(
        record_json(
            &benign,
            RecordJsonOptions {
                detail: atlas_domain::DetailLevel::Standard,
                include_source_json: false,
            },
        )
        .expect("ordinary hazard JSON"),
    )
    .expect("ordinary hazard value");
    assert!(ordinary.get("provenance").is_none());
    assert!(
        ordinary["availability"]
            .as_array()
            .is_none_or(|availability| {
                availability.iter().all(|entry| {
                    !entry["field"].as_str().is_some_and(|field| {
                        field.contains("source_metadata")
                            || field == "provenance.token.name"
                            || field == "entity.unsupported.action.unexpected.traits.rarity"
                    })
                })
            })
    );
    let benign_with_provenance = serde_json::to_value(
        record_json_with_context(
            &benign,
            RecordJsonOptions {
                detail: atlas_domain::DetailLevel::Full,
                include_source_json: false,
            },
            RecordJsonContext::without_lookups(&benign.record).with_provenance_evidence(),
        )
        .expect("benign hazard provenance JSON"),
    )
    .expect("benign hazard provenance value");
    let benign_availability = benign_with_provenance["availability"]
        .as_array()
        .expect("benign availability");
    assert!(benign_availability.iter().all(|entry| {
        !entry["field"].as_str().is_some_and(|field| {
            field.contains("source_metadata")
                || field == "provenance.token.name"
                || field == "entity.unsupported.action.unexpected.traits.rarity"
        })
    }));
    assert!(benign_availability.iter().any(|entry| {
        entry["field"] == "entity.unsupported.action.unexpected./items/0/system/traits/selected"
            && entry["state"] == "unsupported"
    }));
    let benign_metadata = benign_with_provenance["provenance"]["source_metadata"]
        .as_array()
        .expect("benign source metadata provenance");
    assert!(benign_metadata.iter().any(|fact| {
        fact["field"] == "token_name" && fact["value"]["value"]["value"]["value"] == "Hidden Pit"
    }));
    assert!(benign_metadata.iter().any(|fact| {
        fact["field"] == "temporary_maximum" && fact["value"]["value"]["value"]["value"] == 0
    }));
    assert!(benign_metadata.iter().any(|fact| {
        fact["field"] == "save_detail"
            && fact["save"] == "fortitude"
            && fact["value"]["value"]["value"]["value"] == ""
    }));
    assert_eq!(
        benign_metadata
            .iter()
            .filter(|fact| {
                fact["field"] == "item_rarity"
                    && fact["entity_id"] == "lY83oUjx0DLxDByK"
                    && fact["value"]["value"]["value"]["support"] == "typed"
                    && fact["value"]["value"]["value"]["value"] == "common"
                    && fact["value"]["provenance"]["relativeSourcePath"]
                        == "/items/0/system/traits/rarity"
            })
            .count(),
        1
    );
    let benign_unsupported = benign_with_provenance["provenance"]["unsupported_fields"]
        .as_array()
        .expect("benign genuine unsupported fields");
    assert!(benign_unsupported.iter().any(|fact| {
        fact["field"]["kind"] == "action_unexpected"
            && fact["field"]["value"] == "/items/0/system/traits/selected"
            && fact["value"]["exactJson"] == "{}"
            && fact["value"]["relativeSourcePath"] == "/items/0/system/traits/selected"
    }));
    assert!(benign_unsupported.iter().all(|fact| {
        fact["field"]["value"] != "traits.rarity"
            && fact["value"]["relativeSourcePath"] != "/items/0/system/traits/rarity"
            && fact["value"]["exactJson"] != "\"common\""
    }));

    let hazard = hazard_source_metadata_fixture(true);
    let retrieved = RetrievedRecord {
        record: atlas_record_for(&hazard),
        body: Some(RecordBody::Hazard(hazard)),
        spell_children: Vec::new(),
        consumable_occurrences: Default::default(),
    };
    let value = serde_json::to_value(
        record_json_with_context(
            &retrieved,
            RecordJsonOptions {
                detail: atlas_domain::DetailLevel::Full,
                include_source_json: false,
            },
            RecordJsonContext::without_lookups(&retrieved.record).with_provenance_evidence(),
        )
        .expect("hazard provenance JSON"),
    )
    .expect("hazard provenance value");
    let availability = value["availability"].as_array().expect("availability");
    for field in [
        "provenance.token.name",
        "activity.source_metadata.rarity",
        "defenses.hit_points.source_metadata.temporary_maximum",
        "activity.strike.source_metadata.weapon_type",
    ] {
        assert_eq!(
            availability
                .iter()
                .filter(|entry| entry["field"] == field)
                .count(),
            1,
            "{field} should reach public availability once"
        );
    }
    let source_metadata = value["provenance"]["source_metadata"]
        .as_array()
        .expect("source metadata provenance");
    assert_eq!(
        source_metadata
            .iter()
            .filter(|fact| fact["field"] == "token_name")
            .count(),
        1
    );
    assert!(source_metadata.iter().any(|fact| {
        fact["field"] == "token_name"
            && fact["value"]["value"]["value"]["value"]["exactJson"] == "17"
    }));
    assert_eq!(
        source_metadata
            .iter()
            .filter(|fact| {
                fact["field"] == "item_rarity" && fact["entity_id"] == "lY83oUjx0DLxDByK"
            })
            .count(),
        1
    );
    assert!(source_metadata.iter().any(|fact| {
        fact["field"] == "item_rarity"
            && fact["entity_id"] == "lY83oUjx0DLxDByK"
            && fact["value"]["value"]["value"]["value"]["exactJson"] == "42"
    }));
    assert!(source_metadata.iter().any(|fact| {
        fact["field"] == "temporary_maximum" && fact["value"]["value"]["value"]["value"] == 9
    }));
    let unsupported_fields = value["provenance"]["unsupported_fields"]
        .as_array()
        .expect("genuine unsupported fields");
    assert!(unsupported_fields.iter().any(|fact| {
        fact["field"]["kind"] == "action_unexpected"
            && fact["field"]["value"] == "/items/0/system/traits/selected"
            && fact["value"]["exactJson"] == "{}"
            && fact["value"]["relativeSourcePath"] == "/items/0/system/traits/selected"
            && fact["value"]["owner"]["kind"] == "entity"
            && fact["value"]["owner"]["value"] == "lY83oUjx0DLxDByK"
    }));
    assert!(unsupported_fields.iter().all(|fact| {
        fact["field"]["value"] != "traits.rarity"
            && fact["value"]["relativeSourcePath"] != "/items/0/system/traits/rarity"
            && fact["value"]["exactJson"] != "\"common\""
            && fact["value"]["exactJson"] != "17"
            && fact["value"]["exactJson"] != "42"
    }));
}

#[test]
fn hazard_record_json_carries_typed_terminal_annotations_without_changing_machine_json() {
    let mut hazard = hazard_source_metadata_fixture(false);
    let owner_record_key = hazard.identity.record_key.clone();
    let FactValue::Value(HazardSourceValue::Typed(embedded)) = &mut hazard.embedded_entities.value
    else {
        panic!("embedded entities")
    };
    let strike = embedded
        .entities
        .iter_mut()
        .find(|entity| entity.id.as_str() == "synthetic-strike-rule")
        .expect("synthetic strike");
    let HazardCapability::Strike(strike) = &mut strike.capability else {
        panic!("strike capability")
    };
    strike.damage_rolls = typed(
        vec![strike_damage("0", 0, "2d8", "piercing")],
        "/items/1/system/damageRolls",
    );
    embedded.occurrences.push(HazardEntityOccurrence {
        id: HazardOccurrenceId::new("synthetic-strike-occurrence").expect("occurrence id"),
        owner_record_key,
        entity_id: HazardEntityId::new("synthetic-strike-rule").expect("entity id"),
        family: HazardEntityFamily::Strike,
        authored_order: 1,
        source_sort: typed(200_000, "/items/1/sort"),
        source_folder: HazardFact::source(FactValue::Null, "/items/1/folder"),
        source_ordinal: 1,
        contextual_label: typed("Melee Strike".to_string(), "/items/1/name"),
        identity_stability: HazardOccurrenceIdentityStability::StableSourceIdentity,
    });
    let retrieved = RetrievedRecord {
        record: atlas_record_for(&hazard),
        body: Some(RecordBody::Hazard(hazard)),
        spell_children: Vec::new(),
        consumable_occurrences: Default::default(),
    };
    let json = record_json(
        &retrieved,
        RecordJsonOptions {
            detail: atlas_domain::DetailLevel::Standard,
            include_source_json: false,
        },
    )
    .expect("hazard JSON");
    let machine = serde_json::to_value(&json).expect("hazard machine JSON");
    let machine_text = serde_json::to_string(&machine).expect("hazard machine text");
    assert!(!machine_text.contains("\"terminal\""));
    assert!(!machine_text.contains("\"human_label\""));
    assert!(!machine_text.contains("\"human_message\""));
    assert!(machine_text.contains("\"hit_points.temporary\""));
    assert!(machine_text.contains("\"value\":\"0\""));

    let RecordPresentationJson::Hazard {
        sections,
        availability,
        ..
    } = &json.presentation
    else {
        panic!("hazard presentation")
    };
    let facts = sections
        .iter()
        .flat_map(|section| &section.blocks)
        .flat_map(|block| match block {
            RecordBlockJson::FactList { facts } => facts.as_slice(),
            RecordBlockJson::Prose { .. }
            | RecordBlockJson::Content { .. }
            | RecordBlockJson::Relationships { .. } => &[],
        })
        .collect::<Vec<_>>();
    let terminal = |key: &str| {
        facts
            .iter()
            .find(|fact| fact.key == key)
            .and_then(|fact| fact.terminal)
    };
    assert_eq!(
        terminal("armor_class"),
        Some(RecordFactTerminalPresentation::HazardDefense(
            HazardDefenseTerminalFact::ArmorClass(10)
        ))
    );
    assert_eq!(
        terminal("hit_points.current"),
        Some(RecordFactTerminalPresentation::HazardDefense(
            HazardDefenseTerminalFact::HitPointsCurrent(12)
        ))
    );
    assert_eq!(
        terminal("hit_points.maximum"),
        Some(RecordFactTerminalPresentation::HazardDefense(
            HazardDefenseTerminalFact::HitPointsMaximum(12)
        ))
    );
    assert_eq!(
        terminal("hit_points.temporary"),
        Some(RecordFactTerminalPresentation::HazardDefense(
            HazardDefenseTerminalFact::HitPointsTemporary(0)
        ))
    );
    assert_eq!(
        terminal("hit_points.broken_threshold"),
        Some(RecordFactTerminalPresentation::HazardDefense(
            HazardDefenseTerminalFact::BrokenThreshold(6)
        ))
    );
    assert_eq!(
        terminal("save.will"),
        Some(RecordFactTerminalPresentation::HazardDefense(
            HazardDefenseTerminalFact::Will(0)
        ))
    );
    assert_eq!(
        terminal("activity.synthetic-strike-occurrence"),
        Some(RecordFactTerminalPresentation::HazardStrike {
            mode: Some("Melee"),
            action_cost: Some(1),
        })
    );
    assert_eq!(
        terminal("activity.synthetic-strike-occurrence.damage.0"),
        Some(RecordFactTerminalPresentation::HazardDamage)
    );
    assert!(availability.iter().any(|entry| {
        entry.field == "entity.unsupported.action.unexpected./items/0/system/traits/selected"
            && entry.human_label == "Action source fact"
            && !entry.human_message.contains("/items/")
    }));
}

#[test]
fn hazard_body_addition_leaves_creature_projection_behavior_unchanged() {
    let creature = CreatureRecord {
        identity: CreatureIdentity {
            record_key: key("npc-core", "CreatureFixture"),
            source_id: CreatureSourceId::new("CreatureFixture").expect("source id"),
            name: "Creature Fixture".to_string(),
            family: CreatureFamily::Npc,
        },
        level: creature_missing(CreatureSourceField::Level),
        rarity: creature_missing(CreatureSourceField::Rarity),
        traits: creature_missing(CreatureSourceField::Traits),
        size: creature_missing(CreatureSourceField::Size),
        publication: creature_missing(CreatureSourceField::Publication),
        adjustment: creature_missing(CreatureSourceField::Adjustment),
        source_alliance: creature_missing(CreatureSourceField::SourceAlliance),
        perception: creature_missing(CreatureSourceField::Perception),
        initiative: creature_missing(CreatureSourceField::Initiative),
        languages: creature_missing(CreatureSourceField::Languages),
        skills: creature_missing(CreatureSourceField::Skills),
        legacy_abilities: creature_missing(CreatureSourceField::LegacyAbilities),
        defenses: creature_missing(CreatureSourceField::Defenses),
        movement: creature_missing(CreatureSourceField::Movement),
        resources: creature_missing(CreatureSourceField::Resources),
        embedded_entities: creature_missing(CreatureSourceField::EmbeddedEntities),
        content: OwnedRichContent::default(),
        provenance: CreatureProvenance {
            source_path: "packs/npc-core/creature-fixture.json".to_string(),
            source_contract_version: "pf2e-serialized-source/v1".to_string(),
            source_system_version: "6.12.4".to_string(),
            source_upstream_commit: SOURCE_COMMIT.to_string(),
        },
    };

    assert!(project_creature_facts(&creature).metrics.is_empty());
    assert!(matches!(
        RecordBody::Creature(creature),
        RecordBody::Creature(_)
    ));
}

fn hidden_pit_source_fixture() -> HazardRecord {
    let record_key = key("hazards", "BHq5wpQU8hQEke8D");
    let lifecycle = HazardLifecycle {
        description: typed(
            paragraph_document(
                "A wooden trapdoor covers a pit that's 10 feet square and 20 feet deep.",
            ),
            "/system/details/description",
        ),
        disable: typed(hidden_pit_disable_document(), "/system/details/disable"),
        routine: typed(RichDocument::default(), "/system/details/routine"),
        reset: typed(
            paragraph_document(
                "Creatures can still fall into the trap, but the trapdoor must be reset manually for the trap to become hidden again.",
            ),
            "/system/details/reset",
        ),
    };
    let action_id = HazardEntityId::new("lY83oUjx0DLxDByK").expect("entity id");
    let occurrence_id = HazardOccurrenceId::new("lY83oUjx0DLxDByK").expect("occurrence id");
    let action = HazardActionCapability {
        common: item_common(),
        action_type: typed(
            HazardActionType::Reaction,
            "/items/0/system/actionType/value",
        ),
        actions: HazardFact::source(FactValue::Null, "/items/0/system/actions/value"),
        category: HazardFact::source(FactValue::Null, "/items/0/system/category"),
        death_note: HazardFact::source(FactValue::Missing, "/items/0/system/deathNote"),
        frequency: HazardFact::source(FactValue::Null, "/items/0/system/frequency"),
        self_effect: HazardFact::source(FactValue::Null, "/items/0/system/selfEffect"),
        unsupported_fields: vec![HazardUnsupportedFact {
            field: HazardUnsupportedField::ActionUnexpected(
                "/items/0/system/traits/selected".to_string(),
            ),
            value: unsupported_value(
                "{}",
                HazardExpectedShape::Any,
                HazardSourceShape::Object,
                "/items/0/system/traits/selected",
                HazardUnsupportedOwner::Entity(action_id.clone()),
                HazardDiagnosticCode::UnsupportedValue,
            ),
        }],
    };
    let embedded = HazardEmbeddedEntities {
        entities: vec![HazardEntity {
            id: action_id.clone(),
            family: HazardEntityFamily::Action,
            label: "Pitfall".to_string(),
            image: typed(
                "systems/pf2e/icons/actions/Reaction.webp".to_string(),
                "/items/0/img",
            ),
            source_identity: HazardEntitySourceIdentity::Stable {
                source_id: HazardSourceId::new("lY83oUjx0DLxDByK").expect("source id"),
            },
            capability: HazardCapability::Action(Box::new(action)),
        }],
        occurrences: vec![HazardEntityOccurrence {
            id: occurrence_id.clone(),
            owner_record_key: record_key.clone(),
            entity_id: action_id,
            family: HazardEntityFamily::Action,
            authored_order: 0,
            source_sort: typed(100_000, "/items/0/sort"),
            source_folder: HazardFact::source(FactValue::Null, "/items/0/folder"),
            source_ordinal: 0,
            contextual_label: typed("Pitfall".to_string(), "/items/0/name"),
            identity_stability: HazardOccurrenceIdentityStability::StableSourceIdentity,
        }],
    };

    let mut content = OwnedRichContent::default();
    for (order, key, role, source_kind, label, source_path, pointer_family, document) in [
        (
            0,
            "details-description",
            ContentRole::PrimaryDescription,
            ContentSourceKind::Description,
            None,
            "/system/details/description",
            "hazard.description",
            paragraph_document(
                "A wooden trapdoor covers a pit that's 10 feet square and 20 feet deep.",
            ),
        ),
        (
            1,
            "stealth-details",
            ContentRole::SupplementalRules,
            ContentSourceKind::StealthDetails,
            Some("Detection"),
            "/system/attributes/stealth/details",
            "hazard.stealth.details",
            paragraph_document("(or 0 if the trapdoor is disabled or broken)"),
        ),
        (
            2,
            "details-disable",
            ContentRole::SupplementalRules,
            ContentSourceKind::Disable,
            Some("Disable"),
            "/system/details/disable",
            "hazard.disable",
            hidden_pit_disable_document(),
        ),
        (
            3,
            "details-reset",
            ContentRole::SupplementalRules,
            ContentSourceKind::Reset,
            Some("Reset"),
            "/system/details/reset",
            "hazard.reset",
            paragraph_document(
                "Creatures can still fall into the trap, but the trapdoor must be reset manually for the trap to become hidden again.",
            ),
        ),
    ] {
        content.documents.push(OwnedRichContentDocument::new(
            ContentId::new(
                record_key.clone(),
                ContentKey::new(key).expect("content key"),
            ),
            ContentIdentityStability::StableSourceIdentity,
            ContentOwner::Record(record_key.clone()),
            role,
            ContentOrigin::RecordField {
                source_kind,
                relative_source_path: source_path.to_string(),
            },
            ContentVisibility::Public,
            ContentProvenance {
                source_record_key: record_key.clone(),
                relative_source_path: source_path.to_string(),
                field_or_pointer_family: pointer_family.to_string(),
                nested_source_id: None,
                authored_ordinal_or_range: None,
                authored_label: label.map(str::to_string),
            },
            source_kind,
            order,
            label.map(str::to_string),
            document,
            DuplicateContentStatus::Unique,
            Vec::new(),
        ));
    }
    content.documents.push(OwnedRichContentDocument::new(
        ContentId::new(
            record_key.clone(),
            ContentKey::new("action-lY83oUjx0DLxDByK-description").expect("content key"),
        ),
        ContentIdentityStability::StableSourceIdentity,
        ContentOwner::HazardOccurrence(occurrence_id),
        ContentRole::EmbeddedCapability,
        ContentOrigin::HazardEmbeddedEntityField {
            family: HazardEntityFamily::Action,
            nested_source_id: Some("lY83oUjx0DLxDByK".to_string()),
            relative_source_path: "/items/0/system/description/value".to_string(),
        },
        ContentVisibility::Public,
        ContentProvenance {
            source_record_key: record_key.clone(),
            relative_source_path: "/items/0/system/description/value".to_string(),
            field_or_pointer_family: "hazard.action.description".to_string(),
            nested_source_id: Some("lY83oUjx0DLxDByK".to_string()),
            authored_ordinal_or_range: Some("0".to_string()),
            authored_label: Some("Pitfall".to_string()),
        },
        ContentSourceKind::EmbeddedItemDescription,
        4,
        Some("Pitfall".to_string()),
        pitfall_document(),
        DuplicateContentStatus::Unique,
        Vec::new(),
    ));

    HazardRecord {
        identity: HazardIdentity {
            record_key: record_key.clone(),
            source_id: HazardSourceId::new("BHq5wpQU8hQEke8D").expect("source id"),
            name: "Hidden Pit".to_string(),
        },
        level: typed(0, "/system/details/level/value"),
        rarity: typed(Rarity::Common, "/system/traits/rarity"),
        traits: typed(
            vec![
                HazardTrait::new("mechanical").expect("trait"),
                HazardTrait::new("trap").expect("trait"),
            ],
            "/system/traits/value",
        ),
        size: typed(HazardSize::Medium, "/system/traits/size/value"),
        publication: typed(
            HazardPublication {
                title: typed(
                    "Pathfinder GM Core".to_string(),
                    "/system/details/publication/title",
                ),
                remaster: typed(true, "/system/details/publication/remaster"),
                license: typed(
                    PublicationLicense::new("ORC").expect("license"),
                    "/system/details/publication/license",
                ),
            },
            "/system/details/publication",
        ),
        complexity: typed(HazardComplexity::Simple, "/system/details/isComplex"),
        detection: typed(
            HazardDetection {
                stealth_modifier: typed(8, "/system/attributes/stealth/value"),
                details: typed(
                    paragraph_document("(or 0 if the trapdoor is disabled or broken)"),
                    "/system/attributes/stealth/details",
                ),
            },
            "/system/attributes/stealth",
        ),
        defenses: typed(
            HazardDefenses {
                armor_class: typed(10, "/system/attributes/ac/value"),
                hardness: typed(3, "/system/attributes/hardness"),
                hit_points: typed(
                    HazardHitPoints {
                        current: typed(12, "/system/attributes/hp/value"),
                        maximum: typed(12, "/system/attributes/hp/max"),
                        temporary: typed(0, "/system/attributes/hp/temp"),
                        details: typed(RichDocument::default(), "/system/attributes/hp/details"),
                        source_metadata: HazardHitPointSourceMetadata {
                            temporary_maximum: typed(0, "/system/attributes/hp/tempmax"),
                        },
                    },
                    "/system/attributes/hp",
                ),
                saves: typed(
                    HazardSaves {
                        fortitude: typed(1, "/system/saves/fortitude/value"),
                        reflex: typed(1, "/system/saves/reflex/value"),
                        will: typed(0, "/system/saves/will/value"),
                        source_metadata: HazardSaveSourceMetadata {
                            fortitude_detail: typed(
                                String::new(),
                                "/system/saves/fortitude/saveDetail",
                            ),
                            reflex_detail: typed(String::new(), "/system/saves/reflex/saveDetail"),
                            will_detail: typed(String::new(), "/system/saves/will/saveDetail"),
                        },
                    },
                    "/system/saves",
                ),
                immunities: typed(
                    vec![
                        iwr("immunity-0", 0, "critical-hits"),
                        iwr("immunity-1", 1, "precision"),
                    ],
                    "/system/attributes/immunities",
                ),
                weaknesses: HazardFact::source(FactValue::Missing, "/system/attributes/weaknesses"),
                resistances: HazardFact::source(
                    FactValue::Missing,
                    "/system/attributes/resistances",
                ),
                source_metadata: HazardDefenseSourceMetadata {
                    has_health: typed(true, "/system/attributes/hasHealth"),
                },
            },
            "/system/attributes",
        ),
        lifecycle: typed(lifecycle, "/system/details"),
        emits_sound: typed(
            HazardEmitsSound::Named("encounter".to_string()),
            "/system/attributes/emitsSound",
        ),
        embedded_entities: typed(embedded, "/items"),
        content,
        relationships: Vec::new(),
        unsupported_fields: Vec::new(),
        provenance: HazardProvenance {
            source_path: HIDDEN_PIT_PATH.to_string(),
            source_contract_version: "pf2e-serialized-source/v1".to_string(),
            source_system_version: "6.12.4".to_string(),
            source_upstream_commit: SOURCE_COMMIT.to_string(),
            source_folder: HazardFact::source(FactValue::Null, "/folder"),
            image: typed(
                "systems/pf2e/icons/default-icons/hazard.svg".to_string(),
                "/img",
            ),
            source_creature_type: typed(String::new(), "/system/creatureType"),
            source_status_effects: typed(Vec::new(), "/system/statusEffects"),
            actor_effects: HazardFact::source(FactValue::Missing, "/effects"),
            token: typed(
                HazardTokenSourceMetadata {
                    name: typed("Hidden Pit".to_string(), "/prototypeToken/name"),
                },
                "/prototypeToken",
            ),
        },
    }
}

fn item_common() -> HazardItemCommon {
    HazardItemCommon {
        description: typed(pitfall_document(), "/items/0/system/description/value"),
        publication: typed(
            HazardPublication {
                title: typed(String::new(), "/items/0/system/publication/title"),
                remaster: typed(false, "/items/0/system/publication/remaster"),
                license: typed(
                    PublicationLicense::new("OGL").expect("license"),
                    "/items/0/system/publication/license",
                ),
            },
            "/items/0/system/publication",
        ),
        rules: typed(Vec::new(), "/items/0/system/rules"),
        slug: HazardFact::source(FactValue::Null, "/items/0/system/slug"),
        traits: typed(Vec::new(), "/items/0/system/traits/value"),
        rarity: typed(Rarity::Common, "/items/0/system/traits/rarity"),
        lineage: typed(
            HazardItemLineage {
                compendium_source: HazardFact::source(
                    FactValue::Null,
                    "/items/0/_stats/compendiumSource",
                ),
            },
            "/items/0/_stats",
        ),
    }
}

fn hazard_source_metadata_fixture(actionable: bool) -> HazardRecord {
    let mut hazard = hidden_pit_source_fixture();
    let mut strike = synthetic_strike_entity();
    if actionable {
        let record_owner = HazardUnsupportedOwner::Record(hazard.identity.record_key.clone());
        hazard.provenance.token = typed(
            HazardTokenSourceMetadata {
                name: unsupported(
                    "17",
                    HazardExpectedShape::String,
                    HazardSourceShape::Number,
                    "/prototypeToken/name",
                    record_owner,
                    HazardDiagnosticCode::UnexpectedShape,
                ),
            },
            "/prototypeToken",
        );

        let mut defenses = hazard.defenses.typed().expect("defenses").clone();
        defenses.source_metadata.has_health = typed(false, "/system/attributes/hasHealth");
        let mut hit_points = defenses.hit_points.typed().expect("hit points").clone();
        hit_points.source_metadata.temporary_maximum = typed(9, "/system/attributes/hp/tempmax");
        defenses.hit_points = typed(hit_points, "/system/attributes/hp");
        let mut saves = defenses.saves.typed().expect("saves").clone();
        saves.source_metadata.fortitude_detail = typed(
            "against forced movement".to_string(),
            "/system/saves/fortitude/saveDetail",
        );
        defenses.saves = typed(saves, "/system/saves");
        hazard.defenses = typed(defenses, "/system/attributes");

        let FactValue::Value(HazardSourceValue::Typed(embedded)) =
            &mut hazard.embedded_entities.value
        else {
            panic!("embedded entities")
        };
        let action = &mut embedded.entities[0];
        let action_id = action.id.clone();
        let HazardCapability::Action(action) = &mut action.capability else {
            panic!("action")
        };
        action.common.rarity = unsupported(
            "42",
            HazardExpectedShape::ClosedVocabulary,
            HazardSourceShape::Number,
            "/items/0/system/traits/rarity",
            HazardUnsupportedOwner::Entity(action_id),
            HazardDiagnosticCode::UnexpectedShape,
        );

        let HazardCapability::Strike(capability) = &mut strike.capability else {
            panic!("strike")
        };
        capability.common.traits = typed(
            vec![HazardTrait::new("range-120").expect("trait")],
            "/items/1/system/traits/value",
        );
        capability.source_metadata.weapon_type = typed(
            HazardSourceAttackMode::Melee,
            "/items/1/system/weaponType/value",
        );
        capability.source_metadata.attack_effects_custom = typed(
            "corrosive mist".to_string(),
            "/items/1/system/attackEffects/custom",
        );
    }
    let FactValue::Value(HazardSourceValue::Typed(embedded)) = &mut hazard.embedded_entities.value
    else {
        panic!("embedded entities")
    };
    embedded.entities.push(strike);
    hazard
}

fn synthetic_strike_entity() -> HazardEntity {
    let id = HazardEntityId::new("synthetic-strike-rule").expect("entity id");
    HazardEntity {
        id: id.clone(),
        family: HazardEntityFamily::Strike,
        label: "Melee Strike".to_string(),
        image: HazardFact::source(FactValue::Missing, "/items/1/img"),
        source_identity: HazardEntitySourceIdentity::Stable {
            source_id: HazardSourceId::new("synthetic-strike-rule").expect("source id"),
        },
        capability: HazardCapability::Strike(Box::new(HazardStrikeCapability {
            common: item_common(),
            bonus: typed(8, "/items/1/system/bonus/value"),
            attack_effects: typed(Vec::new(), "/items/1/system/attackEffects/value"),
            damage_rolls: typed(Vec::new(), "/items/1/system/damageRolls"),
            source_metadata: HazardStrikeSourceMetadata {
                attack: HazardFact::source(FactValue::Missing, "/items/1/system/attack/value"),
                weapon_type: typed(
                    HazardSourceAttackMode::Melee,
                    "/items/1/system/weaponType/value",
                ),
                attack_effects_custom: typed(String::new(), "/items/1/system/attackEffects/custom"),
            },
            unsupported_fields: Vec::new(),
        })),
    }
}

fn iwr(id: &str, order: u32, iwr_type: &str) -> HazardIwr {
    HazardIwr {
        id: HazardComponentId::new(id).expect("component id"),
        authored_order: order,
        iwr_type: typed(
            iwr_type.to_string(),
            &format!("/system/attributes/immunities/{order}/type"),
        ),
        value: HazardFact::source(
            FactValue::Missing,
            format!("/system/attributes/immunities/{order}/value"),
        ),
        exceptions: HazardFact::source(
            FactValue::Missing,
            format!("/system/attributes/immunities/{order}/exceptions"),
        ),
        double_vs: HazardFact::source(
            FactValue::Missing,
            format!("/system/attributes/immunities/{order}/doubleVs"),
        ),
    }
}

fn strike_damage(
    source_key: &str,
    authored_order: u32,
    damage: &str,
    damage_type: &str,
) -> HazardStrikeDamage {
    HazardStrikeDamage {
        source_key: source_key.to_string(),
        authored_order,
        damage: typed(damage.to_string(), "/items/1/system/damageRolls/damage"),
        damage_type: typed(
            damage_type.to_string(),
            "/items/1/system/damageRolls/damageType",
        ),
        category: HazardFact::source(FactValue::Null, "/items/1/system/damageRolls/category"),
    }
}

fn typed<T>(value: T, path: &str) -> HazardFact<T> {
    HazardFact::source(
        FactValue::Value(HazardSourceValue::Typed(value)),
        path.to_string(),
    )
}

fn unsupported<T>(
    exact_json: &str,
    expected_shape: HazardExpectedShape,
    actual_shape: HazardSourceShape,
    path: &str,
    owner: HazardUnsupportedOwner,
    diagnostic_code: HazardDiagnosticCode,
) -> HazardFact<T> {
    HazardFact::source(
        FactValue::Value(HazardSourceValue::Unsupported(unsupported_value(
            exact_json,
            expected_shape,
            actual_shape,
            path,
            owner,
            diagnostic_code,
        ))),
        path.to_string(),
    )
}

fn unsupported_value(
    exact_json: &str,
    expected_shape: HazardExpectedShape,
    actual_shape: HazardSourceShape,
    path: &str,
    owner: HazardUnsupportedOwner,
    diagnostic_code: HazardDiagnosticCode,
) -> HazardUnsupportedValue {
    HazardUnsupportedValue {
        exact_json: exact_json.to_string(),
        expected_shape,
        actual_shape,
        relative_source_path: path.to_string(),
        owner,
        diagnostic_code,
    }
}

fn paragraph_document(value: &str) -> RichDocument {
    RichDocument::new(vec![html_element(
        "p",
        vec![RichNode::Text {
            text: value.to_string(),
        }],
    )])
}

fn hidden_pit_disable_document() -> RichDocument {
    RichDocument::new(vec![
        RichNode::Foundry {
            node: FoundryNode::Check {
                statistic: Some("thievery".to_string()),
                options: [
                    ("dc".to_string(), "12".to_string()),
                    ("name".to_string(), "Remove the Trapdoor".to_string()),
                ]
                .into_iter()
                .collect(),
                label: None,
            },
        },
        RichNode::Text {
            text: " to remove the trapdoor".to_string(),
        },
    ])
}

fn pitfall_document() -> RichDocument {
    RichDocument::new(vec![
        html_element(
            "p",
            vec![
                html_element(
                    "strong",
                    vec![RichNode::Text {
                        text: "Trigger".to_string(),
                    }],
                ),
                RichNode::Text {
                    text: " A creature walks onto the trapdoor.".to_string(),
                },
            ],
        ),
        RichNode::Text {
            text: "\n".to_string(),
        },
        html_element("hr", Vec::new()),
        RichNode::Text {
            text: "\n".to_string(),
        },
        html_element(
            "p",
            vec![
                html_element(
                    "strong",
                    vec![RichNode::Text {
                        text: "Effect".to_string(),
                    }],
                ),
                RichNode::Text {
                    text: " The triggering creature falls in and takes falling damage (typically "
                        .to_string(),
                },
                RichNode::Foundry {
                    node: FoundryNode::Damage {
                        formula: "10[bludgeoning]".to_string(),
                        options: [("options".to_string(), "fall-damage".to_string())]
                            .into_iter()
                            .collect(),
                        damage_parts: vec![DamagePart {
                            formula: "10".to_string(),
                            damage_type: Some("bludgeoning".to_string()),
                        }],
                        label: None,
                    },
                },
                RichNode::Text {
                    text: " damage). That creature can use the ".to_string(),
                },
                RichNode::FoundryLink {
                    link: FoundryLink {
                        target: RichLinkTarget::Unresolved {
                            target: "Compendium.pf2e.actionspf2e.Item.Grab an Edge".to_string(),
                            fallback_label: "Grab an Edge".to_string(),
                        },
                        label: None,
                        source: FoundryLinkSource {
                            macro_kind: FoundryLinkMacroKind::Uuid,
                            authored_target: "Compendium.pf2e.actionspf2e.Item.Grab an Edge"
                                .to_string(),
                            relation: None,
                        },
                        behavior: FoundryLinkBehavior::Reference,
                    },
                },
                RichNode::Text {
                    text: " reaction to avoid falling.".to_string(),
                },
            ],
        ),
    ])
}

fn html_element(tag: &str, children: Vec<RichNode>) -> RichNode {
    RichNode::HtmlElement {
        tag: tag.to_string(),
        attributes: Default::default(),
        children,
    }
}

fn atlas_record_for(hazard: &HazardRecord) -> AtlasRecord {
    AtlasRecord {
        identity: RecordIdentity {
            key: hazard.identity.record_key.clone(),
            name: hazard.identity.name.clone(),
        },
        classification: RecordClassification {
            kind: RecordKind::Hazard,
            level: Some(0),
            rarity: Some(Rarity::Common),
            traits: vec!["mechanical".to_string(), "trap".to_string()],
            taxonomy: RecordTaxonomy::default(),
        },
        foundry: FoundryRecordInfo {
            pack_label: "Hazards".to_string(),
            document_type: FoundryDocumentType::Actor,
            record_type: FoundryRecordType::Hazard,
            folder_id: None,
        },
        provenance: RecordProvenance {
            source_path: HIDDEN_PIT_PATH.to_string(),
            raw_json: None,
        },
        publication: RecordPublication {
            title: None,
            remaster: false,
            category: PublicationCategory::Core,
        },
        requirements: RecordRequirements::default(),
        timing: RecordTiming::default(),
        mechanics: RecordMechanics::default(),
        content: RecordContent::default(),
        variant: None,
        visibility: RecordVisibility::default(),
    }
}

fn has_fact(document: &RecordPresentationDocument, label: &str, value: &str) -> bool {
    document.sections.iter().any(|section| {
        section.blocks.iter().any(|block| {
            matches!(block, PresentationBlock::FactList(facts) if facts.iter().any(|fact| fact.label == label && fact.value == value))
        })
    })
}

fn key(pack: &str, id: &str) -> RecordKey {
    RecordKey::new(
        PackName::new(pack).expect("pack"),
        RecordId::new(id).expect("record id"),
    )
}

fn creature_missing<T>(field: CreatureSourceField) -> CreatureFact<T> {
    CreatureFact::source(FactValue::Missing, field)
}
