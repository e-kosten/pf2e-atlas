use std::path::PathBuf;

use atlas_domain::PackName;
use atlas_record::{
    ContentOwner, FactValue, FoundryLinkBehavior, FoundryLinkMacroKind, HazardCapability,
    HazardEntityFamily, HazardRuleElement, HazardUnsupportedField, RecordBody, RichLinkTarget,
    build_hazard_presentation_document, build_search_fts_projection, iter_foundry_links,
    project_hazard_facts, render_plain_text,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::ManifestPack;
use super::normalize::{classify_record, normalize_record, normalize_record_from_source_bytes};
use super::owned_content::finalize_hazard_owned_content;
use crate::records::references::{
    build_record_reference_index, resolve_content_references, resolve_reference_edges,
};

#[test]
fn acid_spray_preserves_sorted_identity_and_ordered_strike_damage() {
    let loaded = normalize_fixture(
        "pfs-season-1-bestiary",
        "Actor",
        "packs/pfs-season-1-bestiary/1-00/acid-spray-fountain.json",
    );
    let RecordBody::Hazard(hazard) = loaded.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard body")
    };
    let embedded = hazard.embedded_entities.typed().expect("embedded entities");
    assert_eq!(
        embedded
            .occurrences
            .iter()
            .map(|occurrence| occurrence.id.as_str())
            .collect::<Vec<_>>(),
        ["3EJeaEsJhTMJqDGi", "587VL0W2V2l2E1uh", "nkbi91SwDmuZucwd"]
    );
    let strike = embedded
        .entities
        .iter()
        .find(|entity| entity.id.as_str() == "3EJeaEsJhTMJqDGi")
        .expect("acid spray strike");
    let HazardCapability::Strike(strike) = &strike.capability else {
        panic!("strike capability")
    };
    assert_eq!(strike.bonus.typed(), Some(&14));
    assert_eq!(
        strike.attack_effects.typed(),
        Some(&vec!["no-map".to_string()])
    );
    let damage = strike.damage_rolls.typed().expect("damage rolls");
    assert_eq!(
        damage
            .iter()
            .map(|entry| (entry.source_key.as_str(), entry.authored_order))
            .collect::<Vec<_>>(),
        [("0", 0), ("1", 1)]
    );
    assert_eq!(damage[0].damage.typed(), Some(&"2d10+8".to_string()));
    assert_eq!(damage[1].damage.typed(), Some(&"2d4".to_string()));
}

#[test]
fn false_door_sorts_consumable_first_and_retains_it_as_unsupported_child() {
    let loaded = normalize_fixture(
        "agents-of-edgewatch-bestiary",
        "Actor",
        "packs/agents-of-edgewatch-bestiary/book-4-assault-on-hunting-lodge-seven/false-door-trap.json",
    );
    let RecordBody::Hazard(hazard) = loaded.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard body")
    };
    let embedded = hazard.embedded_entities.typed().expect("embedded");
    assert_eq!(
        embedded
            .occurrences
            .iter()
            .map(|occurrence| (occurrence.id.as_str(), occurrence.family))
            .collect::<Vec<_>>(),
        [
            ("4XbK3Gnf0R1gkYPJ", HazardEntityFamily::UnsupportedChild),
            ("gzMwqFLG38aJHUIa", HazardEntityFamily::Action),
            ("VeiaoxMMsn7EH0Ri", HazardEntityFamily::Action),
        ]
    );
    let child = embedded
        .entities
        .iter()
        .find(|entity| entity.id.as_str() == "4XbK3Gnf0R1gkYPJ")
        .expect("consumable child");
    let HazardCapability::UnsupportedChild(child) = &child.capability else {
        panic!("unsupported child")
    };
    assert_eq!(child.child_type, "consumable");
    assert!(child.unsupported_fields.iter().any(|field| {
        matches!(&field.field, HazardUnsupportedField::UnsupportedChildField(path) if path.ends_with("/system/category"))
            && field.value.exact_json == "\"potion\""
    }));
    assert!(
        render_plain_text(child.common.description.typed().expect("description"))
            .contains("Maximum Duration")
    );
    assert_eq!(
        child
            .common
            .traits
            .typed()
            .expect("traits")
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["alchemical", "consumable", "injury", "poison"]
    );
}

#[test]
fn dragon_content_uses_hazard_occurrence_owner_and_neutral_reference_resolution() {
    let hazard = normalize_fixture(
        "age-of-ashes-bestiary",
        "Actor",
        "packs/age-of-ashes-bestiary/book-2-cult-of-cinders/dragon-pillar.json",
    );
    let detect_magic = normalize_fixture(
        "spells-srd",
        "Item",
        "packs/spells/cantrip/detect-magic.json",
    );
    let dispel_magic = normalize_fixture(
        "spells-srd",
        "Item",
        "packs/spells/2nd-rank/dispel-magic.json",
    );
    let mut records = vec![hazard, detect_magic, dispel_magic];
    finalize_hazard_owned_content(&mut records);
    let index = build_record_reference_index(&records);
    resolve_content_references(&mut records, &index);

    let RecordBody::Hazard(hazard) = records[0].facts.canonical_body.as_ref().expect("hazard")
    else {
        panic!("hazard body")
    };
    assert_eq!(
        hazard
            .embedded_entities
            .typed()
            .expect("embedded")
            .occurrences
            .len(),
        12
    );
    let action_content = hazard
        .content
        .documents
        .iter()
        .find(|document| {
            document.provenance.nested_source_id.as_deref() == Some("HPLwJZH8RwLW9c13")
        })
        .expect("black eye beam content");
    assert_eq!(
        action_content.owner,
        ContentOwner::HazardOccurrence(
            atlas_record::HazardOccurrenceId::new("HPLwJZH8RwLW9c13").expect("id")
        )
    );
    let detect = hazard
        .content
        .documents
        .iter()
        .flat_map(|document| iter_foundry_links(&document.document))
        .find(|link| link.source.authored_target.contains("Detect Magic"))
        .expect("Detect Magic reference");
    assert_eq!(
        detect.source.authored_target,
        "Compendium.pf2e.spells-srd.Item.Detect Magic"
    );
    assert!(matches!(
        &detect.target,
        RichLinkTarget::Record { key, .. } if key.to_string() == "spells-srd:gpzpAAAJ1Lza2JVl"
    ));
    assert_eq!(detect.source.macro_kind, FoundryLinkMacroKind::Uuid);
    assert_eq!(detect.behavior, FoundryLinkBehavior::Reference);

    let strike = hazard
        .embedded_entities
        .typed()
        .expect("embedded")
        .entities
        .iter()
        .find(|entity| entity.id.as_str() == "uapfpdQSjUz13nBn")
        .expect("eye beam strike");
    let HazardCapability::Strike(strike) = &strike.capability else {
        panic!("strike")
    };
    assert!(strike.unsupported_fields.iter().any(|field| {
        field.field == HazardUnsupportedField::StrikeAttackEffectsCustom
            && field.value.exact_json == "\"\""
    }));
    assert!(strike.unsupported_fields.iter().any(|field| {
        field.field == HazardUnsupportedField::StrikeTraitRarity
            && field.value.exact_json == "\"common\""
    }));
    let recognize_ally = hazard
        .embedded_entities
        .typed()
        .expect("embedded")
        .entities
        .iter()
        .find(|entity| entity.id.as_str() == "HSs8knjTJuMzsAQr")
        .expect("Recognize Ally");
    let HazardCapability::Action(recognize_ally) = &recognize_ally.capability else {
        panic!("action")
    };
    assert!(recognize_ally.unsupported_fields.iter().any(|field| {
        matches!(&field.field, HazardUnsupportedField::ActionUnexpected(path) if path.ends_with("/system/traits/selected"))
            && field.value.exact_json == "{}"
    }));
    assert!(hazard.unsupported_fields.iter().any(|field| {
        matches!(&field.field, HazardUnsupportedField::HazardUnexpected(path) if path == "/system/customModifiers")
            && field.value.exact_json == "{}"
    }));
    let edges = resolve_reference_edges(&records);
    assert!(edges.iter().any(|edge| {
        edge.from_record_key.to_string() == "age-of-ashes-bestiary:zNIjGSxkG8xyDLgR"
            && edge.to_record_key.to_string() == "spells-srd:gpzpAAAJ1Lza2JVl"
            && edge.reference_text == "Compendium.pf2e.spells-srd.Item.Detect Magic"
    }));
}

#[test]
fn unresolved_hazard_reference_and_synthetic_condition_effect_contexts_are_lossless() {
    let mut records = vec![normalize_fixture(
        "the-slithering-bestiary",
        "Actor",
        "packs/the-slithering-bestiary/scroll-shock-trap.json",
    )];
    finalize_hazard_owned_content(&mut records);
    let index = build_record_reference_index(&records);
    resolve_content_references(&mut records, &index);
    let RecordBody::Hazard(hazard) = records[0].facts.canonical_body.as_ref().expect("hazard")
    else {
        panic!("hazard body")
    };
    let unresolved = hazard
        .content
        .documents
        .iter()
        .flat_map(|document| iter_foundry_links(&document.document))
        .find(|link| link.source.authored_target.contains("Dispel Magic"))
        .expect("unresolved Dispel Magic");
    assert!(matches!(
        &unresolved.target,
        RichLinkTarget::Unresolved { target, .. }
            if target == "Compendium.pf2e.spells-srd.Item.Dispel Magic"
    ));
    let embedded = hazard.embedded_entities.typed().expect("embedded");
    assert_eq!(
        embedded
            .occurrences
            .iter()
            .map(|occurrence| occurrence.id.as_str())
            .collect::<Vec<_>>(),
        [
            "YKm4JByteFxgMLrc",
            "y1EDMIPZrSGC7wRz",
            "enJ4VkXbDUV56rwT",
            "r4GLtMTPNISHccTL",
            "GKCraZhfHY73oosv",
        ]
    );
    let strike = embedded
        .entities
        .iter()
        .find(|entity| entity.id.as_str() == "GKCraZhfHY73oosv")
        .expect("electrical bolt");
    let HazardCapability::Strike(strike) = &strike.capability else {
        panic!("strike")
    };
    assert_eq!(strike.bonus.typed(), Some(&21));
    assert_eq!(
        strike
            .damage_rolls
            .typed()
            .expect("damage")
            .iter()
            .map(|entry| (entry.source_key.as_str(), entry.damage.typed().cloned()))
            .collect::<Vec<_>>(),
        [
            ("0", Some("1d6+2".to_string())),
            ("1", Some("1d6".to_string()))
        ]
    );
    let reactive_content = hazard
        .content
        .documents
        .iter()
        .find(|document| {
            document.provenance.nested_source_id.as_deref() == Some("enJ4VkXbDUV56rwT")
        })
        .expect("Reactive Charge content");
    assert!(matches!(
        reactive_content.owner,
        ContentOwner::HazardOccurrence(_)
    ));

    let path = "packs/hazards/hidden-pit.json";
    let mut raw = read_fixture(path);
    raw["items"] = json!([
        synthetic_common_item("condition-id", "Condition shell", "condition"),
        synthetic_common_item("effect-id", "Effect shell", "effect")
    ]);
    raw["effects"] = json!([{"_id":"active-effect","disabled":false,"name":"Editor context"}]);
    raw["system"]["experimentalHazardData"] = json!({"enabled": true});
    let loaded = normalize_raw("hazards", "Actor", path, raw);
    let RecordBody::Hazard(hazard) = loaded.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard body")
    };
    let embedded = hazard.embedded_entities.typed().expect("embedded");
    assert_eq!(embedded.entities[0].family, HazardEntityFamily::Condition);
    assert_eq!(embedded.entities[1].family, HazardEntityFamily::Effect);
    let actor_effects = hazard
        .provenance
        .actor_effects
        .typed()
        .expect("actor effects");
    assert_eq!(actor_effects.len(), 1);
    assert_eq!(actor_effects[0].authored_order, 0);
    assert_eq!(
        actor_effects[0].exact_json,
        r#"{"_id":"active-effect","disabled":false,"name":"Editor context"}"#
    );
    assert!(hazard.unsupported_fields.iter().any(|field| {
        matches!(&field.field, HazardUnsupportedField::HazardUnexpected(path) if path == "/system/experimentalHazardData/enabled")
            && field.value.exact_json == "true"
    }));
    assert_eq!(
        embedded.entities.len(),
        2,
        "ActiveEffect is not a semantic child"
    );
}

#[test]
fn rule_elements_model_known_shapes_and_preserve_unknown_exactly() {
    let path = "packs/hazards/hidden-pit.json";
    let mut raw = read_fixture(path);
    raw["items"] = json!([{
        "_id": "rule-action",
        "name": "Rules",
        "type": "action",
        "sort": 1,
        "system": {
            "actionType": {"value": "passive"},
            "actions": {"value": 2},
            "category": "offensive",
            "deathNote": false,
            "description": {"value": ""},
            "frequency": {"value": 1, "max": 2, "per": "round"},
            "publication": {"license": "OGL", "remaster": false, "title": "Synthetic"},
            "rules": [
                {"key":"Immunity","mode":"add","type":"cold"},
                {"key":"ActiveEffectLike","mode":"override","path":"system.attributes.hardness","value":true},
                {"key":"Aura","radius":5,"slug":"cold","traits":["cold"]},
                {"key":"DamageDice","critical":true,"diceNumber":2,"dieSize":"d6","damageType":"cold","selector":"rule-damage"},
                {"key":"FlatModifier","critical":false,"damageType":"cold","selector":"rule-damage","value":3},
                {"key":"FutureHazardRule","payload":{"amount":7}}
            ],
            "selfEffect": {"uuid": "Compendium.pf2e.conditionitems.Item.Test", "name": "Test"},
            "slug": "rules",
            "traits": {"rarity":"common","value":[]}
        }
    }]);
    let loaded = normalize_raw("hazards", "Actor", path, raw);
    let RecordBody::Hazard(hazard) = loaded.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard body")
    };
    let action = match &hazard.embedded_entities.typed().expect("embedded").entities[0].capability {
        HazardCapability::Action(action) => action,
        _ => panic!("action"),
    };
    assert_eq!(action.actions.typed().map(|value| value.value()), Some(2));
    assert_eq!(action.death_note.typed(), Some(&false));
    assert_eq!(
        action.frequency.typed().expect("frequency").maximum.typed(),
        Some(&2)
    );
    assert_eq!(
        action
            .self_effect
            .typed()
            .expect("self effect")
            .target_uuid
            .typed(),
        Some(&"Compendium.pf2e.conditionitems.Item.Test".to_string())
    );
    let rules = action.common.rules.typed().expect("rules");
    assert!(matches!(&rules[2], HazardRuleElement::Aura(rule) if rule.radius.typed() == Some(&5)));
    let HazardRuleElement::Unsupported(rule) = &rules[5] else {
        panic!("unknown rule is typed unsupported")
    };
    assert_eq!(
        rule.source.exact_json,
        "{\"key\":\"FutureHazardRule\",\"payload\":{\"amount\":7}}"
    );
    let facts = project_hazard_facts(hazard);
    let fts =
        build_search_fts_projection(&loaded.record, &[], loaded.facts.canonical_body.as_ref());
    assert!(
        facts
            .mechanic_terms
            .iter()
            .any(|term| term == "aura radius 5")
    );
    for expected in [
        "active effect mode override",
        "active effect value true",
        "damage dice critical true",
        "flat modifier critical false",
    ] {
        assert!(facts.mechanic_terms.iter().any(|term| term == expected));
        assert!(fts.mechanic_terms.contains(expected));
    }
    let presentation = build_hazard_presentation_document(hazard, |_| true);
    let presented_rules = presentation
        .sections
        .iter()
        .flat_map(|section| &section.blocks)
        .filter_map(|block| match block {
            atlas_record::PresentationBlock::FactList(facts) => Some(facts),
            _ => None,
        })
        .flatten()
        .filter(|fact| fact.label == "Rule")
        .map(|fact| fact.value.as_str())
        .collect::<Vec<_>>();
    assert!(presented_rules.contains(&"Immunity add cold"));
    assert!(presented_rules.contains(&"ActiveEffectLike override system.attributes.hardness true"));
    assert!(presented_rules.contains(&"Aura cold 5 cold"));
    assert!(presented_rules.contains(&"DamageDice 2d6 cold critical=true selector=rule-damage"));
    assert!(
        presented_rules
            .contains(&"FlatModifier 3 critical=false damageType=cold selector=rule-damage")
    );
}

#[test]
fn malformed_missing_and_duplicate_child_ids_share_fallback_content_ownership() {
    let path = "packs/hazards/hidden-pit.json";
    let mut raw = read_fixture(path);
    raw["items"] = json!([
        synthetic_identity_item(
            None,
            "Missing",
            "@UUID[Compendium.pf2e.spells-srd.Item.Detect Magic]"
        ),
        synthetic_identity_item(
            Some("bad id"),
            "Invalid",
            "@UUID[Compendium.pf2e.spells-srd.Item.Detect Magic]"
        ),
        synthetic_identity_item(
            Some("duplicate-id"),
            "Duplicate A",
            "@UUID[Compendium.pf2e.spells-srd.Item.Detect Magic]"
        ),
        synthetic_identity_item(
            Some("duplicate-id"),
            "Duplicate B",
            "@UUID[Compendium.pf2e.spells-srd.Item.Detect Magic]"
        )
    ]);
    let mut records = vec![normalize_raw("hazards", "Actor", path, raw)];
    finalize_hazard_owned_content(&mut records);
    let RecordBody::Hazard(hazard) = records[0].facts.canonical_body.as_ref().expect("hazard")
    else {
        panic!("hazard")
    };
    let embedded = hazard.embedded_entities.typed().expect("embedded");
    assert_eq!(
        embedded
            .occurrences
            .iter()
            .map(|occurrence| occurrence.id.as_str())
            .collect::<Vec<_>>(),
        ["fallback-0", "fallback-1", "fallback-2", "fallback-3"]
    );
    assert_eq!(
        hazard
            .relationships
            .iter()
            .map(|relationship| relationship.id.as_str())
            .collect::<Vec<_>>(),
        [
            "contains-fallback-0",
            "contains-fallback-1",
            "contains-fallback-2",
            "contains-fallback-3"
        ]
    );
    assert!(hazard.content.exclusions.is_empty());
    let occurrence_documents = hazard
        .content
        .documents
        .iter()
        .filter_map(|document| match &document.owner {
            ContentOwner::HazardOccurrence(id) => Some((id.as_str(), document)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        occurrence_documents
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>(),
        ["fallback-0", "fallback-1", "fallback-2", "fallback-3"]
    );
    assert!(
        occurrence_documents
            .iter()
            .all(|(_, document)| { iter_foundry_links(&document.document).count() == 1 })
    );
}

#[test]
fn serialized_damage_map_preserves_nonlexical_authored_key_order() {
    let serialized = br#"{"_id":"ordered-damage-hazard","name":"Ordered Damage","type":"hazard","items":[{"_id":"ordered-strike","name":"Strike","type":"melee","sort":0,"system":{"damageRolls":{"10":{"damage":"1d10","damageType":"piercing"},"2":{"damage":"1d2","damageType":"cold"}}}}],"system":{}}"#;
    let loaded = normalize_serialized("hazards", "Actor", "packs/hazards/ordered.json", serialized);
    let RecordBody::Hazard(hazard) = loaded.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard")
    };
    let strike = match &hazard.embedded_entities.typed().expect("embedded").entities[0].capability {
        HazardCapability::Strike(strike) => strike,
        _ => panic!("strike"),
    };
    assert_eq!(
        strike
            .damage_rolls
            .typed()
            .expect("damage")
            .iter()
            .map(|damage| (damage.source_key.as_str(), damage.authored_order))
            .collect::<Vec<_>>(),
        [("10", 0), ("2", 1)]
    );
}

#[test]
fn checked_in_selected_sources_match_the_pinned_blob_digests() {
    for (relative, expected) in [
        (
            "packs/hazards/hidden-pit.json",
            "645d9d31313903e6afa81eb7191a4e011cf5bd74c17304c4f190275a5e1359c3",
        ),
        (
            "packs/book-of-the-dead-bestiary/disembodied-voices.json",
            "fec338025930630ebe09431e1ddf5fb0a614f3ad959f0bcbf61cae6eeb5aceb9",
        ),
        (
            "packs/agents-of-edgewatch-bestiary/book-6-ruins-of-the-radiant-siege/greater-planar-rift.json",
            "abe29d48df2d8b11d9f6617c2b16439b1041bed1ba7a25153d3c1f54912afbfc",
        ),
        (
            "packs/pfs-season-1-bestiary/1-00/acid-spray-fountain.json",
            "21d7c082f1ebb844b7fc66bf491f8237b24f2b4e7bc97646d2e6954fe4cf9032",
        ),
        (
            "packs/agents-of-edgewatch-bestiary/book-4-assault-on-hunting-lodge-seven/false-door-trap.json",
            "6560ba176f07c15d490b0c8eef5308d890c07150eb53dd4900524a4f903b600e",
        ),
        (
            "packs/age-of-ashes-bestiary/book-2-cult-of-cinders/dragon-pillar.json",
            "7401f9ea50dcd8a0bc3f24360faa2b7e7a1e19796fd9b46600a14e25d0cef2ef",
        ),
        (
            "packs/the-slithering-bestiary/scroll-shock-trap.json",
            "8773c92c8cada8b74c72b0c776d5d473b0747f5e473b5a96b659420db29d2b38",
        ),
        (
            "packs/hazards/brown-mold.json",
            "c7bede2a081ef621b64b0b134d7156f4074028221fc460534a62b2cefd838c2c",
        ),
        (
            "packs/menace-under-otari-bestiary/scythe-blades-bb.json",
            "b31ae35bad89a7169f4c126806f0a1d9c694bc8625e70006c381e19240f4e579",
        ),
    ] {
        let bytes = std::fs::read(source_root().join(relative)).expect("checked-in source");
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            expected,
            "{relative}"
        );
    }
}

#[test]
fn populated_source_rules_are_typed_without_losing_missing_members() {
    let brown_mold = normalize_fixture("hazards", "Actor", "packs/hazards/brown-mold.json");
    let RecordBody::Hazard(brown_mold) = brown_mold.facts.canonical_body.as_ref().expect("hazard")
    else {
        panic!("hazard body")
    };
    let rules = match &brown_mold
        .embedded_entities
        .typed()
        .expect("embedded")
        .entities
        .iter()
        .find(|entity| entity.id.as_str() == "NcyiM0WihYrncbSF")
        .expect("Emit Cold")
        .capability
    {
        HazardCapability::Action(action) => action.common.rules.typed().expect("rules"),
        _ => panic!("action"),
    };
    assert!(matches!(
        &rules[0],
        HazardRuleElement::Aura(rule)
            if rule.radius.typed() == Some(&5)
                && rule.slug.typed() == Some(&"emit-cold".to_string())
                && rule.traits.typed().expect("traits")[0].as_str() == "cold"
    ));

    let blades = normalize_fixture(
        "menace-under-otari-bestiary",
        "Actor",
        "packs/menace-under-otari-bestiary/scythe-blades-bb.json",
    );
    let RecordBody::Hazard(blades) = blades.facts.canonical_body.as_ref().expect("hazard") else {
        panic!("hazard body")
    };
    let rules = match &blades
        .embedded_entities
        .typed()
        .expect("embedded")
        .entities
        .iter()
        .find(|entity| entity.id.as_str() == "LPBpvhfnfUxz7nWz")
        .expect("Scythe")
        .capability
    {
        HazardCapability::Strike(strike) => strike.common.rules.typed().expect("rules"),
        _ => panic!("strike"),
    };
    let HazardRuleElement::DamageDice(rule) = &rules[0] else {
        panic!("damage dice rule")
    };
    assert_eq!(rule.critical.typed(), Some(&true));
    assert_eq!(rule.dice_number.typed(), Some(&6));
    assert_eq!(rule.die_size.typed(), Some(&"d4".to_string()));
    assert!(matches!(rule.damage_type.value, FactValue::Missing));
    assert_eq!(
        rule.selector.typed(),
        Some(&"{item|_id}-damage".to_string())
    );
}

#[test]
fn adventure_and_token_hazard_contexts_do_not_dispatch_as_standalone_records() {
    assert_eq!(classify_record("Adventure", "hazard"), None);
    assert_eq!(classify_record("Token", "hazard"), None);
}

fn synthetic_common_item(id: &str, name: &str, item_type: &str) -> Value {
    json!({
        "_id": id,
        "name": name,
        "type": item_type,
        "sort": 0,
        "system": {
            "description": {"value": ""},
            "publication": {"license": "OGL", "remaster": false, "title": "Synthetic"},
            "rules": [],
            "slug": null,
            "traits": {"value": []}
        }
    })
}

fn synthetic_identity_item(id: Option<&str>, name: &str, description: &str) -> Value {
    let mut item = synthetic_common_item(id.unwrap_or("placeholder"), name, "action");
    if let Some(id) = id {
        item["_id"] = Value::String(id.to_string());
    } else {
        item.as_object_mut().expect("item object").remove("_id");
    }
    item["system"]["description"]["value"] = Value::String(description.to_string());
    item
}

fn normalize_fixture(
    pack: &str,
    document_type: &str,
    relative: &str,
) -> crate::records::LoadedSourceRecord {
    let source_root = source_root();
    let serialized = std::fs::read(source_root.join(relative)).expect("fixture");
    normalize_record_from_source_bytes(
        &ManifestPack {
            name: pack.to_string(),
            label: pack.to_string(),
            document_type: document_type.to_string(),
            path: format!("packs/{pack}"),
        },
        &PackName::new(pack.to_string()).expect("pack"),
        &source_root.join(relative),
        &source_root,
        &serialized,
        None,
    )
    .expect("normalization")
}

fn read_fixture(relative: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(source_root().join(relative)).expect("fixture"))
        .expect("fixture JSON")
}

fn normalize_raw(
    pack: &str,
    document_type: &str,
    relative: &str,
    raw: Value,
) -> crate::records::LoadedSourceRecord {
    let source_root = source_root();
    normalize_record(
        &ManifestPack {
            name: pack.to_string(),
            label: pack.to_string(),
            document_type: document_type.to_string(),
            path: format!("packs/{pack}"),
        },
        &PackName::new(pack.to_string()).expect("pack"),
        &source_root.join(relative),
        &source_root,
        raw,
        None,
    )
    .expect("normalization")
}

fn normalize_serialized(
    pack: &str,
    document_type: &str,
    relative: &str,
    serialized: &[u8],
) -> crate::records::LoadedSourceRecord {
    let source_root = source_root();
    normalize_record_from_source_bytes(
        &ManifestPack {
            name: pack.to_string(),
            label: pack.to_string(),
            document_type: document_type.to_string(),
            path: format!("packs/{pack}"),
        },
        &PackName::new(pack.to_string()).expect("pack"),
        &source_root.join(relative),
        &source_root,
        serialized,
        None,
    )
    .expect("normalization")
}

fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hazards/pinned")
}
