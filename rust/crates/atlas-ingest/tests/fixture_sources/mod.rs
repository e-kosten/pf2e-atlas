use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn write_generated_affliction_fixture_source(
    root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/ghoul.json"),
        r#"{
          "_id": "testGhoul0001",
          "name": "Test Ghoul",
          "type": "npc",
          "items": [
            {
              "_id": "ghoulFeverItem",
              "name": "Ghoul Fever",
              "type": "action",
              "system": {
                "category": "offensive",
                "description": {
                  "value": "<p><strong>Saving Throw</strong> @Check[fortitude|dc:18]</p><p><strong>Stage 1</strong> @UUID[Compendium.pf2e.conditionitems.Item.Sickened]{Sickened 1} (1 day)</p>"
                },
                "traits": { "value": ["disease"] }
              }
            },
            {
              "_id": "serpentDaggerItem",
              "name": "Serpent Dagger",
              "type": "weapon",
              "system": {
                "category": "simple",
                "description": {
                  "value": "<p>Dagger Venom (poison) <strong>Saving Throw</strong> @Check[fortitude|dc:21]</p><p><strong>Stage 1</strong> @Damage[1d8[poison]] damage</p>"
                },
                "traits": { "value": ["agile", "poison"] }
              }
            }
          ],
          "system": {
            "details": {
              "level": { "value": 2 },
              "publication": { "title": "Pathfinder Monster Core" }
            },
            "traits": { "rarity": "common", "value": ["undead"] }
          }
        }"#,
    )?;
    Ok(())
}

pub(crate) fn write_family_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/pathfinder-npc-core"))?;
    fs::create_dir_all(root.join("packs/bestiary-family-ability-glossary/ghost"))?;
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::create_dir_all(root.join("packs/adventure-bestiary"))?;
    fs::create_dir_all(root.join("packs/equipment"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "pathfinder-npc-core", "label": "NPC Core", "type": "Actor", "path": "packs/pathfinder-npc-core" },
            { "name": "bestiary-family-ability-glossary", "label": "Family Abilities", "type": "Item", "path": "packs/bestiary-family-ability-glossary" },
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" },
            { "name": "adventure-bestiary", "label": "Adventure Bestiary", "type": "Actor", "path": "packs/adventure-bestiary" },
            { "name": "equipment", "label": "Equipment", "type": "Item", "path": "packs/equipment" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/pathfinder-npc-core/_folders.json"),
        r#"[
          { "_id": "folderSeafarer", "name": "Seafarer", "folder": null }
        ]"#,
    )?;
    fs::write(
        root.join("packs/pathfinder-npc-core/bosun.json"),
        r#"{
          "_id": "bosun00000001",
          "name": "Bosun",
          "type": "npc",
          "folder": "folderSeafarer",
          "system": {
            "traits": { "value": ["human", "humanoid"], "rarity": "common" },
            "description": { "value": "<p>A ship officer.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary-family-ability-glossary/ghost/frightful-moans.json"),
        r#"{
          "_id": "ghostAbility01",
          "name": "Frightful Moans",
          "type": "action",
          "system": {
            "description": { "value": "<p>A ghost family ability.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/ghost-commoner.json"),
        r#"{
          "_id": "ghostCommoner1",
          "name": "Ghost Commoner",
          "type": "npc",
          "system": {
            "traits": { "value": ["ghost", "undead"] },
            "rules": [
              {
                "key": "Note",
                "text": "@UUID[Compendium.pf2e.bestiary-family-ability-glossary.Item.ghostAbility01]{Frightful Moans}"
              }
            ],
            "description": { "value": "<p>A ghostly commoner with @UUID[Compendium.pf2e.bestiary-family-ability-glossary.Item.ghostAbility01]{Frightful Moans}.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/storm-dragon-young.json"),
        r#"{
          "_id": "stormYoung001",
          "name": "Storm Dragon (Young)",
          "type": "npc",
          "system": {
            "traits": { "value": ["dragon", "electricity"] },
            "description": { "value": "<p>A young storm dragon.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/storm-dragon-adult.json"),
        r#"{
          "_id": "stormAdult01",
          "name": "Storm Dragon (Adult)",
          "type": "npc",
          "system": {
            "traits": { "value": ["dragon", "electricity"] },
            "description": { "value": "<p>An adult storm dragon.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/adventure-bestiary/venexus.json"),
        r#"{
          "_id": "venexus000001",
          "name": "Venexus",
          "type": "npc",
          "system": {
            "traits": { "value": ["dragon", "electricity"], "rarity": "unique" },
            "details": { "blurb": "Female young storm dragon" },
            "description": { "value": "<p>A named storm dragon.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/equipment/wondrous-figurine-rubber-bear.json"),
        r#"{
          "_id": "figurineBear1",
          "name": "Wondrous Figurine (Rubber Bear)",
          "type": "equipment",
          "system": {
            "description": { "value": "<p>A rubber bear figurine.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/equipment/wondrous-figurine-golden-lions.json"),
        r#"{
          "_id": "figurineLions1",
          "name": "Wondrous Figurine (Golden Lions)",
          "type": "equipment",
          "system": {
            "description": { "value": "<p>A golden lions figurine.</p>" }
          }
        }"#,
    )?;
    Ok(())
}

pub(crate) fn write_remaster_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/actions"))?;
    fs::create_dir_all(root.join("packs/conditionitems"))?;
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::create_dir_all(root.join("packs/journals"))?;
    fs::create_dir_all(root.join("src/module/migration/migrations"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" },
            { "name": "conditionitems", "label": "Conditions", "type": "Item", "path": "packs/conditionitems" },
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" },
            { "name": "journals", "label": "Journals", "type": "JournalEntry", "path": "packs/journals" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/reactive-strike.json"),
        r#"{
          "_id": "reactiveStrike1",
          "name": "Reactive Strike",
          "type": "action",
          "system": {
            "publication": { "title": "Player Core", "remaster": true },
            "description": { "value": "<p>Strike as a reaction.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/attack-of-opportunity.json"),
        r#"{
          "_id": "attackOpportunity1",
          "name": "Attack of Opportunity",
          "type": "action",
          "system": {
            "publication": { "title": "Core Rulebook", "remaster": false },
            "description": { "value": "<p>Legacy reaction.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/conditionitems/off-guard.json"),
        r#"{
          "_id": "offGuard1",
          "name": "Off-Guard",
          "type": "condition",
          "system": {
            "publication": { "title": "Player Core", "remaster": true },
            "description": { "value": "<p>You are distracted.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/conditionitems/flat-footed.json"),
        r#"{
          "_id": "flatFooted1",
          "name": "flat-footed",
          "type": "condition",
          "system": {
            "publication": { "title": "Core Rulebook", "remaster": false },
            "description": { "value": "<p>You are distracted.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/journals/remaster-changes.json"),
        r#"{
          "_id": "journal1",
          "name": "Remaster Changes",
          "pages": [
            {
              "_id": "page1",
              "name": "Class Features",
              "type": "text",
              "text": {
                "content": "<table><tbody><tr><td>Attack of Opportunity</td><td>Multiple</td><td>Renamed</td><td>@UUID[Compendium.pf2e.actions.Item.Reactive Strike]{Reactive Strike}</td></tr></tbody></table>",
                "format": 1
              }
            }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/alias-carrier.json"),
        r#"{
          "_id": "aliasCarrier1",
          "name": "Alias Carrier",
          "type": "npc",
          "items": [
            {
              "_id": "embeddedLegacy1",
              "name": "Legacy Guard",
              "type": "condition",
              "_stats": {
                "compendiumSource": "Compendium.pf2e.conditionitems.Item.offGuard1"
              },
              "system": {
                "publication": { "title": "Core Rulebook", "remaster": false }
              }
            }
          ],
          "system": {
            "publication": { "title": "Bestiary", "remaster": false },
            "traits": { "value": ["humanoid"] },
            "description": { "value": "<p>Fixture carrier.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("src/module/migration/migrations/850-flat-footed-to-off-guard.ts"),
        r#"/* Rename all uses and mentions of "flat-footed" to "Off-Guard" */"#,
    )?;
    Ok(())
}

pub(crate) fn write_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/actions"))?;
    fs::create_dir_all(root.join("packs/spells"))?;
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::create_dir_all(root.join("packs/equipment"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" },
            { "name": "spells", "label": "Spells", "type": "Item", "path": "packs/spells" },
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" },
            { "name": "equipment", "label": "Equipment", "type": "Item", "path": "packs/equipment" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/treat-wounds.json"),
        r#"{
          "_id": "testAction0001",
          "name": "Treat Wounds",
          "type": "action",
          "system": {
            "traits": { "value": ["healing", "exploration"] },
            "prerequisites": {
              "value": [
                { "value": "trained in Medicine" },
                { "value": "healer's toolkit" },
                { "value": "   " }
              ]
            },
            "actions": { "value": 1 },
            "usage": { "value": "held-in-one-hand" },
            "price": { "value": { "sp": 3 } },
            "publication": { "title": "Player Core", "remaster": true },
            "rules": [
              {
                "key": "Note",
                "text": "@UUID[Compendium.pf2e.spells.Item.testSpell0001]{Heal} @UUID[Compendium.pf2e.spells.Item.testSpell0001]{Heal} @UUID[Compendium.pf2e.spells.Item.missingSpell]{Missing}"
              }
            ],
            "description": { "value": "<p>You spend 10 minutes treating one injured living creature with @UUID[Compendium.pf2e.spells.Item.testSpell0001]{Heal}.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/demoralize.json"),
        r#"{
          "_id": "testAction0002",
          "name": "Demoralize",
          "type": "action",
          "system": {
            "traits": { "value": ["auditory", "concentrate", "emotion", "fear", "mental"] },
            "rules": [
              {
                "key": "Note",
                "text": "@UUID[Compendium.pf2e.spells.Item.Heal]{Heal Spell}"
              }
            ],
            "description": { "value": "<p>Use Intimidation to frighten a creature.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/spells/heal.json"),
        r#"{
          "_id": "testSpell0001",
          "name": "Heal",
          "type": "spell",
          "system": {
            "level": { "value": 1 },
            "traits": { "value": ["healing", "vitality", "cantrip"], "traditions": ["divine", "primal"], "rarity": "common" },
            "time": { "value": "1 minute" },
            "duration": { "value": "10 minutes" },
            "range": { "value": "30 feet" },
            "target": { "value": "<p>1 willing creature</p>" },
            "defense": { "save": { "statistic": "fortitude", "basic": true } },
            "damage": { "main": { "type": "vitality" } },
            "publication": { "title": "Player Core" },
            "description": { "value": "<p>You channel vital energy.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/goblin.json"),
        r#"{
          "_id": "testActor0001",
          "name": "Goblin Scout",
          "type": "npc",
          "system": {
            "traits": { "value": ["goblin", "humanoid"], "size": { "value": "small" } },
            "details": { "languages": { "value": ["goblin"] } },
            "abilities": { "dex": { "mod": 4 } },
            "perception": { "mod": 7, "senses": [{ "type": "darkvision", "range": 60 }] },
            "attributes": {
              "ac": { "value": 17 },
              "hp": { "value": 16, "max": 16 },
              "speed": { "value": 25, "otherSpeeds": [{ "type": "climb", "value": 10 }] }
            },
            "saves": {
              "fortitude": { "mod": 5 },
              "reflex": { "mod": 8 },
              "will": { "mod": 4 }
            },
            "skills": {
              "stealth": { "mod": 9, "rank": 1 }
            },
            "description": { "value": "<p>A small scout.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/equipment/longbow.json"),
        r#"{
          "_id": "testWeapon0001",
          "name": "Longbow",
          "type": "weapon",
          "system": {
            "traits": { "value": ["deadly-d10"] },
            "group": "bow",
            "usage": { "value": "held-in-two-hands" },
            "bulk": { "value": 2 },
            "range": { "increment": 100 },
            "reload": { "value": 0 },
            "damage": { "dice": 1, "die": "d8", "damageType": "piercing" },
            "description": { "value": "<p>A ranged weapon.</p>" }
          }
        }"#,
    )?;
    Ok(())
}

pub(crate) fn fixture_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-ingest-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    path
}
