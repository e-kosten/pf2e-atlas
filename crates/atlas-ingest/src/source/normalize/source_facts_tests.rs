use std::path::Path;

use atlas_domain::{PackName, Rarity, RecordKind};
use atlas_record::{
    ActivationTimeSourceField, ContentSourceKind, FoundryDocumentMechanics, FoundryDocumentType,
    FoundryRecordType, ItemTypeMechanics, render_plain_text,
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
                "publication": { "title": "Bestiary Fixture", "remaster": true },
                "disable": "<p>DC 22 Athletics</p>",
                "isComplex": true
            },
            "attributes": {
                "speed": { "otherSpeeds": [{ "type": "fly" }] },
                "immunities": [{ "type": "fire" }],
                "resistances": [{ "type": "cold" }],
                "weaknesses": [{ "type": "holy" }]
            },
            "perception": {
                "senses": [{ "type": "darkvision" }]
            },
            "traits": {
                "rarity": "rare",
                "size": { "value": "lg" },
                "value": ["dragon"]
            },
            "slug": "fixture-creature"
        }
    });

    let loaded = normalize_record(
        &manifest_pack("Actor"),
        &PackName::new("bestiary".to_string()).expect("pack name"),
        Path::new("packs/bestiary/actor.json"),
        Path::new("."),
        raw,
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

    let FoundryDocumentMechanics::Actor(actor) = &record.mechanics.document else {
        panic!("actor mechanics should be nested under record mechanics");
    };
    assert_eq!(actor.size.as_deref(), Some("lg"));
    assert_eq!(actor.languages, vec!["common", "draconic"]);
    assert_eq!(actor.speed_types, vec!["fly", "land"]);
    assert_eq!(actor.senses, vec!["darkvision"]);
    assert_eq!(actor.immunities, vec!["fire"]);
    assert_eq!(actor.resistances, vec!["cold"]);
    assert_eq!(actor.weaknesses, vec!["holy"]);
    assert_eq!(actor.disable_text.as_deref(), Some("DC 22 Athletics"));
    assert!(actor.is_complex);
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
                        "value": "<p><strong>Saving Throw</strong> Fortitude</p><p><strong>Stage 1</strong> Sickened</p>"
                    }
                }
            },
            {
                "_id": "spell1",
                "name": "Staged Spell",
                "type": "spell",
                "system": {
                    "traits": { "value": ["curse"] },
                    "spell": {
                        "system": {
                            "description": { "value": "<p>Nested spell text.</p>" }
                        }
                    }
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
    )
    .expect("record normalizes");

    let facts = &loaded.facts.source_facts;
    assert_eq!(facts.slug.as_deref(), Some("host-record"));
    assert_eq!(
        facts.compendium_source.as_deref(),
        Some("Compendium.pf2e.bestiary.Actor.host1")
    );
    assert_eq!(facts.embedded_items.len(), 2);

    let affliction = &facts.embedded_items[0];
    assert_eq!(affliction.item_id, "bite1");
    assert_eq!(affliction.normalized_name, "ghoul fever");
    assert_eq!(affliction.foundry_item_type, "action");
    assert_eq!(affliction.system_category.as_deref(), Some("offensive"));
    assert_eq!(affliction.traits, vec!["disease"]);
    assert_eq!(affliction.slug.as_deref(), Some("ghoul-fever"));
    assert!(affliction.raw_provenance.is_some());
    assert_eq!(affliction.content_refs.len(), 1);
    assert_eq!(
        affliction.content_refs[0].source_kind,
        ContentSourceKind::EmbeddedItemDescription
    );
    assert_eq!(
        affliction.content_refs[0].local_key,
        "#item:bite1:description"
    );
    let affliction_content = facts
        .source_content
        .get("#item:bite1:description")
        .expect("embedded description is source content");
    assert_eq!(
        render_plain_text(&affliction_content.document),
        "Saving Throw Fortitude\nStage 1 Sickened"
    );

    let spell = &facts.embedded_items[1];
    assert_eq!(
        spell.content_refs[0].local_key,
        "#item:spell1:spell-description"
    );
    assert_eq!(
        facts
            .source_content
            .get("#item:spell1:spell-description")
            .map(|content| render_plain_text(&content.document))
            .as_deref(),
        Some("Nested spell text.")
    );

    assert_eq!(facts.journal_pages.len(), 1);
    assert_eq!(facts.journal_pages[0].page_id.as_deref(), Some("page1"));
    assert_eq!(facts.journal_pages[0].normalized_name, "rules");
    assert_eq!(facts.journal_pages[0].source_ref, "journal:Rules");
    assert_eq!(
        render_plain_text(&facts.journal_pages[0].document),
        "Rules\nPage body."
    );
    assert_eq!(facts.skipped_journal_pages.len(), 3);
    assert_eq!(
        facts.skipped_journal_pages[0].reason,
        JournalPageSkipReason::MissingTextContent
    );
    assert_eq!(
        facts.skipped_journal_pages[1].reason,
        JournalPageSkipReason::EmptyTextContent
    );
    assert_eq!(
        facts.skipped_journal_pages[2].reason,
        JournalPageSkipReason::EmptyParsedDocument
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
