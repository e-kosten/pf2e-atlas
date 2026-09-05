use std::path::Path;

use atlas_domain::{PackName, Rarity, RecordKind};
use atlas_record::{
    ActivationTimeSourceField, ActivityRollAbility, ContentSourceKind, CreatureDamageKind,
    CreatureMovementMode, CreatureResourceAmount, CreatureSourceAlliance,
    CreatureUnsupportedSourceField, FactValue, FoundryDocumentMechanics, FoundryDocumentType,
    FoundryRecordType, ItemTypeMechanics, PresentationBlock, RecordBody,
    build_search_fts_projection, build_search_presentation_document_with_content_filter,
    render_plain_text,
};
use serde_json::json;

use super::normalize_record;
use crate::records::JournalPageSkipReason;
use crate::source::ManifestPack;

#[test]
fn normalizes_actor_record_into_nested_atlas_record_shape() {
    let raw = json!({
        "_id": "actor1",
        "name": "Fixture Creature",
        "type": "npc",
        "folder": "folder-a",
        "_stats": { "compendiumSource": "Compendium.pf2e.bestiary.Actor.actor1" },
        "system": {
            "details": {
                "level": { "value": 5 },
                "languages": { "value": ["common", "draconic"] },
                "alliance": "party",
                "publication": { "title": "Bestiary Fixture", "remaster": true },
                "disable": "<p>DC 22 Athletics</p>",
                "isComplex": true
            },
            "attributes": {
                "adjustment": "elite",
                "hardness": {"value": 5},
                "shield": {"ac": 2, "brokenThreshold": 4, "hardness": 3, "max": 8, "value": 6},
                "speed": { "otherSpeeds": [{ "type": "fly" }] },
                "immunities": [{ "type": "fire" }],
                "resistances": [{ "type": "cold" }],
                "weaknesses": [{ "type": "holy" }]
            },
            "perception": {
                "senses": [{ "type": "darkvision" }]
            },
            "initiative": {"statistic": "perception"},
            "abilities": {
                "str": {"mod": 3, "value": 16}, "dex": {"mod": 2, "value": 14},
                "con": {"mod": 1, "value": 12}, "int": {"mod": 0, "value": 10},
                "wis": {"mod": -1, "value": 8}, "cha": {"mod": -2, "value": 6}
            },
            "resources": {"focus": {"max": 1, "maxx": 2, "value": 1}},
            "traits": {
                "rarity": "rare",
                "size": { "value": "lg" },
                "value": ["dragon"]
            },
            "slug": "fixture-creature"
        }
    });
    let expected_source_envelope = raw.clone();

    let loaded = normalize_record(
        &manifest_pack("Actor"),
        &PackName::new("bestiary".to_string()).expect("pack name"),
        Path::new("packs/bestiary/actor.json"),
        Path::new("."),
        raw,
        None,
    )
    .expect("actor normalizes");
    let record = &loaded.record;

    assert_eq!(record.identity.key.to_string(), "bestiary:actor1");
    assert_eq!(record.identity.name, "Fixture Creature");
    assert_eq!(record.classification.kind, RecordKind::Creature);
    assert_eq!(record.classification.level, Some(5));
    assert_eq!(record.classification.rarity, Some(Rarity::Rare));
    assert_eq!(record.classification.traits, vec!["dragon"]);
    assert_eq!(record.foundry.document_type, FoundryDocumentType::Actor);
    assert_eq!(record.foundry.record_type, FoundryRecordType::Npc);
    assert_eq!(record.foundry.folder_id.as_deref(), Some("folder-a"));
    assert_eq!(
        record.publication.title.as_deref(),
        Some("Bestiary Fixture")
    );
    assert!(record.publication.remaster);
    assert_eq!(record.provenance.source_path, "packs/bestiary/actor.json");
    assert!(record.provenance.raw_json.is_some());

    assert_eq!(
        record.mechanics.document,
        FoundryDocumentMechanics::None,
        "creatures must not retain a generic actor-mechanics projection"
    );

    let RecordBody::Creature(creature) = loaded
        .facts
        .canonical_body
        .as_ref()
        .expect("NPC canonical body");
    assert_eq!(creature.level.value, FactValue::Value(5));
    let CreatureSourceAlliance::Named(alliance) = creature
        .source_alliance
        .value
        .as_value()
        .expect("source alliance")
    else {
        panic!("party should remain an intrinsic source alliance");
    };
    assert_eq!(alliance.as_str(), "party");
    assert_eq!(
        creature
            .legacy_abilities
            .value
            .as_value()
            .expect("legacy abilities")
            .strength,
        FactValue::Value(3)
    );
    let defenses = creature.defenses.value.as_value().expect("defenses");
    assert_eq!(defenses.hardness, FactValue::Value(5));
    assert_eq!(
        defenses
            .shield
            .as_value()
            .expect("shield")
            .serialized_hit_points,
        FactValue::Value(6)
    );
    assert_eq!(
        creature
            .movement
            .value
            .as_value()
            .expect("canonical movement")[0]
            .mode,
        CreatureMovementMode::Fly
    );
    let focus = &creature.resources.value.as_value().expect("resources")[0];
    assert_eq!(
        focus.maximum,
        FactValue::Value(CreatureResourceAmount::Integer(1))
    );
    assert_eq!(
        focus.source_drift.as_value().expect("maxx drift")[0].field,
        CreatureUnsupportedSourceField::ResourceMaximumDrift
    );
    assert_eq!(loaded.facts.npc_core_diagnostics.len(), 1);
    let npc_source = loaded
        .facts
        .npc_source
        .as_ref()
        .expect("full versioned NPC Source envelope");
    assert_eq!(npc_source.raw_json_for_audit(), &expected_source_envelope);
}

#[test]
fn npc_canonical_facts_drive_display_and_fts_without_a_generic_metric_carrier() {
    let raw = json!({
        "_id": "actor-facts",
        "name": "Fact Convergence Creature",
        "type": "npc",
        "system": {
            "attributes": {
                "ac": {"value": 28},
                "hp": {"value": 170, "max": 170},
                "speed": {
                    "value": 25,
                    "otherSpeeds": [{"type": "fly", "value": 40}]
                },
                "resistances": [{"type": "mental", "value": 10}]
            },
            "details": {
                "level": {"value": 9},
                "languages": {"value": ["aklo", "common"]},
                "publication": {"title": "Pathfinder Bestiary"}
            },
            "perception": {
                "mod": 18,
                "senses": [{"type": "scent", "range": 60}]
            },
            "saves": {
                "fortitude": {"value": 19},
                "reflex": {"value": 17},
                "will": {"value": 18}
            },
            "skills": {"arcana": {"base": 18}},
            "traits": {
                "rarity": "common",
                "size": {"value": "med"},
                "value": ["fiend"]
            }
        },
        "items": []
    });

    let loaded = normalize_record(
        &manifest_pack("Actor"),
        &PackName::new("bestiary".to_string()).expect("pack name"),
        Path::new("packs/bestiary/fact-convergence.json"),
        Path::new("."),
        raw,
        None,
    )
    .expect("NPC normalizes");
    let record = &loaded.record;

    assert!(record.mechanics.metrics.is_empty());
    assert!(record.mechanics.actor().is_none());

    let canonical_body = loaded
        .facts
        .canonical_body
        .as_ref()
        .expect("canonical body");
    let presentation = build_search_presentation_document_with_content_filter(
        record,
        Some(canonical_body),
        |_| true,
    );
    assert!(presentation.sections.iter().any(|section| {
        section.blocks.iter().any(|block| {
            matches!(
                block,
                PresentationBlock::FactList(facts)
                    if facts.iter().any(|fact| {
                        fact.label == "Arcana" && fact.value == "18"
                    })
            )
        })
    }));
    let fts = build_search_fts_projection(record, &[], Some(canonical_body));
    assert!(fts.metric_terms.contains("Arcana"));
    assert!(fts.mechanic_terms.contains("AC 28"));
    assert!(fts.mechanic_terms.contains("Max HP 170"));
    assert!(fts.mechanic_terms.contains("Perception 18"));
    assert!(fts.mechanic_terms.contains("Fortitude 19"));
    assert!(fts.mechanic_terms.contains("Arcana 18"));
    assert!(fts.mechanic_terms.contains("Land Speed 25"));
    assert!(fts.mechanic_terms.contains("Fly Speed 40"));
}

#[test]
fn normalizes_spell_item_into_nested_item_and_spell_shape() {
    let raw = json!({
        "_id": "spell1",
        "name": "Fixture Spell",
        "type": "spell",
        "system": {
            "level": { "value": 3 },
            "publication": { "title": "Player Core", "remaster": true },
            "traits": {
                "rarity": "uncommon",
                "value": ["cantrip", "fire"],
                "traditions": ["arcane", "primal"]
            },
            "prerequisites": { "value": [{ "value": "trained in Arcana" }] },
            "time": { "value": "2 actions" },
            "duration": { "value": "1 minute", "sustained": true },
            "category": "spell",
            "group": "attack",
            "baseItem": "wand",
            "usage": { "value": "held-in-one-hand" },
            "price": { "value": { "gp": 2 } },
            "bulk": { "value": "L" },
            "range": { "value": "30 feet" },
            "target": { "value": "<p>1 creature</p>" },
            "area": { "type": "burst", "value": 10 },
            "defense": { "save": { "statistic": "reflex", "basic": true } },
            "damageRolls": { "0": { "damageType": "fire" } },
            "description": { "value": "<p>Spell body.</p>" }
        }
    });

    let loaded = normalize_record(
        &manifest_pack("Item"),
        &PackName::new("spells".to_string()).expect("pack name"),
        Path::new("packs/spells/spell.json"),
        Path::new("."),
        raw,
        None,
    )
    .expect("spell normalizes");
    let record = &loaded.record;

    assert_eq!(record.identity.key.to_string(), "spells:spell1");
    assert_eq!(record.classification.kind, RecordKind::Spell);
    assert_eq!(record.classification.level, Some(3));
    assert_eq!(record.classification.rarity, Some(Rarity::Uncommon));
    assert_eq!(record.requirements.prerequisites, vec!["trained in Arcana"]);
    assert_eq!(
        record
            .timing
            .activation
            .as_ref()
            .map(|timing| timing.source_field),
        Some(ActivationTimeSourceField::TimeValue)
    );
    assert_eq!(
        record
            .timing
            .duration
            .as_ref()
            .map(|timing| timing.time.text.as_str()),
        Some("1 minute")
    );
    assert_eq!(
        record.content.documents[0].source_kind,
        ContentSourceKind::Description
    );
    assert_eq!(
        render_plain_text(&record.content.documents[0].document),
        "Spell body."
    );

    let item = record
        .mechanics
        .item()
        .expect("item mechanics should exist");
    assert_eq!(item.category.as_deref(), Some("spell"));
    assert_eq!(item.base_item.as_deref(), Some("wand"));
    assert_eq!(item.group.as_deref(), Some("attack"));
    assert_eq!(item.usage.as_deref(), Some("held-in-one-hand"));
    assert_eq!(item.price_json.as_deref(), Some(r#"{"gp":2}"#));
    assert_eq!(item.hands_requirement.as_deref(), Some("one_hand"));
    assert_eq!(item.damage_types, vec!["fire"]);

    let Some(ItemTypeMechanics::Spell(spell)) = item.foundry_type.as_ref() else {
        panic!("spell item should carry spell mechanics");
    };
    assert_eq!(spell.traditions, vec!["arcane", "primal"]);
    assert_eq!(spell.kinds, vec!["cantrip"]);
    assert_eq!(
        spell.range.as_ref().map(|range| range.text.as_str()),
        Some("30 feet")
    );
    assert_eq!(
        spell.target.as_ref().map(|target| target.text.as_str()),
        Some("1 creature")
    );
    assert_eq!(
        spell.area.as_ref().and_then(|area| area.kind.as_deref()),
        Some("burst")
    );
    assert_eq!(spell.area.as_ref().and_then(|area| area.value), Some(10.0));
    assert_eq!(
        spell
            .defense
            .as_ref()
            .and_then(|defense| defense.save.as_deref()),
        Some("reflex")
    );
    assert!(spell.defense.as_ref().is_some_and(|defense| defense.basic));
    assert!(spell.sustained);
    assert_eq!(spell.damage_types, vec!["fire"]);
}

#[test]
fn rejects_present_unsupported_rarity() {
    let raw = json!({
        "_id": "spell1",
        "name": "Fixture Spell",
        "type": "spell",
        "system": {
            "traits": {
                "rarity": "mythic",
                "value": []
            }
        }
    });

    let error = normalize_record(
        &manifest_pack("Item"),
        &PackName::new("spells".to_string()).expect("pack name"),
        Path::new("packs/spells/spell.json"),
        Path::new("."),
        raw,
        None,
    )
    .expect_err("unsupported rarity should fail normalization");

    let message = error.to_string();
    assert!(message.contains("packs/spells/spell.json"));
    assert!(message.contains("unsupported rarity `mythic`"));
}

#[test]
fn normalizes_source_facts_embedded_content_refs_and_journal_pages() {
    let raw = json!({
        "_id": "host1",
        "name": "Host Record",
        "type": "npc",
        "_stats": { "compendiumSource": "Compendium.pf2e.bestiary.Actor.host1" },
        "items": [
            {
                "_id": "bite1",
                "name": "Ghoul Fever",
                "type": "action",
                "_stats": { "compendiumSource": "Compendium.pf2e.afflictions.Item.ghoul-fever" },
                "system": {
                    "slug": "ghoul-fever",
                    "category": "offensive",
                    "publication": { "remaster": false },
                    "traits": { "value": ["disease"] },
                    "description": {
                        "gm": "<p>GM instruction for @UUID[Compendium.pf2e.afflictions.Item.ghoul-fever].</p>",
                        "value": "<p><strong>Saving Throw</strong> Fortitude</p><p><strong>Stage 1</strong> Sickened</p>"
                    }
                }
            },
            {
                "_id": "spell1",
                "name": "Staged Spell",
                "type": "spell",
                "system": {
                    "damage": {
                        "0": {
                            "formula": "1d4",
                            "kinds": ["damage", "healing"],
                            "type": "void"
                        }
                    },
                    "overlays": {
                        "living": {
                            "_id": "living",
                            "name": "Staged Spell (Healing)",
                            "sort": 2,
                            "system": {
                                "damage": {
                                    "0": {
                                        "formula": "1d4+4",
                                        "kinds": ["healing"],
                                        "type": "void"
                                    }
                                },
                                "range": { "value": "30 feet" },
                                "target": { "value": "1 ally" },
                                "time": { "value": "2" }
                            }
                        },
                        "undead": {
                            "_id": "undead",
                            "name": "Staged Spell (Damage)",
                            "sort": 3,
                            "system": {
                                "damage": {
                                    "0": {
                                        "kinds": ["damage"]
                                    }
                                },
                                "target": { "value": "1 enemy" }
                            }
                        }
                    },
                    "location": {
                        "value": "casting1"
                    },
                    "traits": { "value": ["curse"] },
                    "spell": {
                        "system": {
                            "description": { "value": "<p>Nested spell text.</p>" }
                        }
                    }
                }
            },
            {
                "_id": "casting1",
                "name": "Occult Cantrips",
                "type": "spellcastingEntry",
                "system": {
                    "prepared": { "value": "spontaneous" },
                    "spelldc": {
                        "dc": 23,
                        "value": 14
                    },
                    "slots": {
                        "slot0": { "max": 5, "value": 5 }
                    }
                }
            },
            {
                "_id": "claw1",
                "name": "Claw",
                "type": "melee",
                "system": {
                    "bonus": { "value": 12 },
                    "damageRolls": {
                        "main": {
                            "damage": "1d6+2",
                            "damageType": "slashing"
                        }
                    },
                    "traits": { "value": ["agile"] }
                }
            }
        ],
        "pages": [
            {
                "_id": "page1",
                "name": "Rules",
                "text": { "content": "<h2>Rules</h2><p>Page body.</p>" }
            },
            {
                "_id": "page2",
                "name": "Image",
                "type": "image"
            },
            {
                "_id": "page3",
                "name": "Empty",
                "text": { "content": "   " }
            },
            {
                "_id": "page4",
                "name": "Unknown Empty",
                "text": { "content": "<unknown></unknown>" }
            }
        ],
        "system": {
            "slug": "host-record",
            "details": {
                "level": { "value": 1 },
                "publication": { "title": "Fixture" }
            },
            "traits": { "rarity": "common", "value": ["undead"] }
        }
    });
    let loaded = normalize_record(
        &manifest_pack("Actor"),
        &PackName::new("bestiary".to_string()).expect("pack name"),
        Path::new("packs/bestiary/host.json"),
        Path::new("."),
        raw,
        None,
    )
    .expect("record normalizes");
    let mut loaded_records = vec![loaded];
    let reference_index = crate::records::references::build_record_reference_index(&loaded_records);
    crate::source::npc_entities::finalize_npc_embedded_entities(
        &mut loaded_records,
        &reference_index,
    );
    let loaded = &loaded_records[0];

    let facts = &loaded.facts.source_facts;
    assert_eq!(facts.slug.as_deref(), Some("host-record"));
    assert_eq!(
        facts.compendium_source.as_deref(),
        Some("Compendium.pf2e.bestiary.Actor.host1")
    );
    assert_eq!(facts.embedded_items.len(), 4);

    let affliction = &facts.embedded_items[0];
    assert_eq!(affliction.item_id, "bite1");
    assert_eq!(affliction.normalized_name, "ghoul fever");
    assert_eq!(affliction.foundry_item_type, "action");
    assert_eq!(affliction.system_category.as_deref(), Some("offensive"));
    assert_eq!(affliction.traits, vec!["disease"]);
    assert_eq!(affliction.slug.as_deref(), Some("ghoul-fever"));
    assert!(affliction.raw_provenance.is_some());
    assert_eq!(affliction.content_refs.len(), 2);
    assert_eq!(
        affliction.content_refs[0].source_kind,
        ContentSourceKind::EmbeddedItemDescription
    );
    assert_eq!(
        affliction.content_refs[0].local_key,
        "item:bite1:description"
    );
    assert_eq!(
        affliction.content_refs[1].source_kind,
        ContentSourceKind::EmbeddedGmDescription
    );
    assert_eq!(
        affliction.content_refs[1].local_key,
        "item:bite1:gm-description"
    );
    let affliction_content = facts
        .source_content
        .get("item:bite1:description")
        .expect("embedded description is source content");
    assert_eq!(
        render_plain_text(&affliction_content.document),
        "Saving Throw Fortitude\nStage 1 Sickened"
    );
    assert_eq!(
        facts
            .source_content
            .get("item:bite1:gm-description")
            .map(|content| render_plain_text(&content.document))
            .as_deref(),
        Some("GM instruction for ghoul fever.")
    );

    let spell = &facts.embedded_items[1];
    assert_eq!(
        spell.content_refs[0].local_key,
        "item:spell1:spell-description"
    );
    assert_eq!(
        facts
            .source_content
            .get("item:spell1:spell-description")
            .map(|content| render_plain_text(&content.document))
            .as_deref(),
        Some("Nested spell text.")
    );
    let RecordBody::Creature(creature) = loaded
        .facts
        .canonical_body
        .as_ref()
        .expect("canonical creature");
    let embedded = creature
        .embedded_entities
        .value
        .as_value()
        .expect("embedded entities");
    let canonical_spell = embedded
        .occurrences
        .iter()
        .find(|occurrence| occurrence.id.as_str().ends_with(":spell1"))
        .expect("spell occurrence");
    let atlas_record::CreatureCapability::Spell(canonical_spell) = &canonical_spell.capability
    else {
        panic!("spell capability")
    };
    assert_eq!(
        canonical_spell.unsupported_notes.len(),
        2,
        "{:#?}",
        canonical_spell.unsupported_notes
    );
    let spell_damage = canonical_spell.damage.as_value().expect("spell damage");
    assert_eq!(
        spell_damage[0].formula.as_value().map(String::as_str),
        Some("1d4")
    );
    assert_eq!(
        spell_damage[0].damage_type.as_value().map(String::as_str),
        Some("void")
    );
    assert_eq!(
        spell_damage[0].kinds.as_value(),
        Some(&vec![
            CreatureDamageKind::Damage,
            CreatureDamageKind::Healing
        ])
    );
    let strike_activity = embedded
        .occurrences
        .iter()
        .find(|occurrence| occurrence.id.as_str().ends_with(":claw1"))
        .expect("strike occurrence");
    let atlas_record::CreatureCapability::Strike(strike_activity) = &strike_activity.capability
    else {
        panic!("strike capability")
    };
    let strike_damage = strike_activity.damage.as_value().expect("strike damage");
    assert_eq!(
        strike_damage[0].formula.as_value().map(String::as_str),
        Some("1d6+2")
    );
    assert_eq!(
        strike_damage[0].damage_type.as_value().map(String::as_str),
        Some("slashing")
    );
    assert_eq!(strike_activity.rolls.len(), 1);
    assert_eq!(strike_activity.rolls[0].value.as_value(), Some(&12));
    assert_eq!(
        strike_activity.rolls[0].ability.as_value(),
        Some(&ActivityRollAbility::Strength)
    );

    assert_eq!(facts.journal_pages.len(), 2);
    assert_eq!(facts.journal_pages[0].page_id.as_deref(), Some("page1"));
    assert_eq!(facts.journal_pages[0].normalized_name, "rules");
    assert_eq!(facts.journal_pages[0].source_ref, "journal:Rules");
    assert_eq!(
        render_plain_text(&facts.journal_pages[0].document),
        "Rules\nPage body."
    );
    assert_eq!(facts.skipped_journal_pages.len(), 2);
    assert_eq!(
        facts.skipped_journal_pages[0].reason,
        JournalPageSkipReason::MissingTextContent
    );
    assert_eq!(
        facts.skipped_journal_pages[1].reason,
        JournalPageSkipReason::EmptyTextContent
    );
}

fn manifest_pack(document_type: &str) -> ManifestPack {
    ManifestPack {
        name: "bestiary".to_string(),
        label: "Bestiary".to_string(),
        document_type: document_type.to_string(),
        path: "packs/bestiary".to_string(),
    }
}
