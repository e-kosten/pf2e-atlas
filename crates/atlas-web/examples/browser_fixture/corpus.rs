use atlas_index::SourceArtifactRecordInput;
use atlas_search::test_support::record;
use serde_json::{Value, json};

const CONDITION_LINK: &str =
    "@UUID[Compendium.pf2e.browser-conditions.Item.offguard00000001]{Off-Guard}";
const RULE_LINK: &str =
    "@UUID[Compendium.pf2e.browser-rules.Item.rule000000000001]{Signal Protocol}";

pub(super) fn records() -> Vec<SourceArtifactRecordInput> {
    let mut records = vec![
        record("browser-actors", "Actor", sentinel()),
        record(
            "browser-conditions",
            "Item",
            json!({
                "_id":"offguard00000001", "type":"condition", "name":"Off-Guard",
                "system": {
                    "slug":"off-guard", "value":{"isValued":false,"value":null},
                    "description":{"value":format!("<p>Your defenses are distracted. Read {RULE_LINK} for the sentinel's signal sequence.</p>")}
                }
            }),
        ),
        record(
            "browser-rules",
            "Item",
            json!({
                "_id":"rule000000000001", "type":"action", "name":"Signal Protocol",
                "system": {
                    "actionType":{"value":"action"}, "actions":{"value":1},
                    "traits":{"value":["auditory","concentrate"]},
                    "description":{"value":format!("<p>The lantern flashes twice before the patrol advances. The marked creature becomes {CONDITION_LINK}.</p><p>@UUID[Compendium.pf2e.browser-spells.Item.spell00000000001]{{Lantern Ward}} describes the protective signal.</p>")}
                }
            }),
        ),
        record("browser-spells", "Item", lantern_ward()),
        record(
            "browser-hazards",
            "Actor",
            json!({
                "_id":"hazard0000000001", "type":"hazard", "name":"Resonant Bell Trap",
                "system": {
                    "details":{"level":{"value":3}, "description":"<p>A brass bell rings when the threshold wire breaks.</p>","disable":"<p>Thievery DC 20 to release the wire.</p>","isComplex":false},
                    "attributes":{"ac":{"value":18},"hp":{"max":40},"hardness":8,"stealth":{"value":12,"details":"A fine wire crosses the doorway."}},
                    "saves":{"fortitude":{"value":12},"reflex":{"value":9},"will":{"value":6}},
                    "traits":{"value":["mechanical","trap"],"rarity":"common","size":{"value":"med"}}
                }, "items":[]
            }),
        ),
        record(
            "browser-tables",
            "RollTable",
            json!({
                "_id":"table00000000001", "name":"Night Patrol Signals", "formula":"1d4", "replacement":false,
                "description":"<p>Choose the signal shown by the watchtower lantern.</p>",
                "results":[
                    {"_id":"result0000000001","type":"text","range":[1,4],"weight":2,"description":format!("<p>{RULE_LINK}</p>")},
                    {"_id":"result0000000002","type":"text","range":[1,4],"weight":2,"description":"<p>The lantern remains dark while the patrol changes watch.</p>"}
                ]
            }),
        ),
    ];
    // Exact source fixtures exercise the same rank-aware pipeline as full-corpus reading.
    records.push(record(
        "browser-effects",
        "Item",
        serde_json::from_str(include_str!(
            "../../../atlas-app-service/tests/fixtures/spell-effect-animal-form-bear.json"
        ))
        .expect("Bear source fixture"),
    ));
    records.push(record(
        "browser-effects",
        "Item",
        serde_json::from_str(include_str!(
            "../../../atlas-app-service/tests/fixtures/spell-effect-enlarge.json"
        ))
        .expect("Enlarge source fixture"),
    ));
    records.push(record("browser-spells", "Item", json!({
        "_id":"battleform000001", "type":"spell", "name":"Browser Battle Form",
        "system":{"level":{"value":2}, "description":{"value":"<p>Choose @UUID[Compendium.pf2e.browser-effects.Item.gQnDKDeBTtjwOWAk]{Bear} or @UUID[Compendium.pf2e.browser-effects.Item.sPCWrhUHqlbGhYSD]{Enlarge}.</p>"}}
    })));
    records.push(record("browser-hazards", "Actor", json!({
        "_id":"routine000000001", "type":"hazard", "name":"Browser Routine Trap",
        "system": {
            "details":{"level":{"value":8}, "isComplex":true, "description":"<p>A set of blades follows a repeating pattern.</p>",
                "disable":"<p>@Check[thievery|dc:31] check to disable a blade.</p>",
                "routine":"<p>(7 actions; illusion, mental) Each blade acts once. The target must attempt a @Check[will|dc:26|traits:illusion,mental] save. Reduce the number of actions for each destroyed blade.</p>"},
            "traits":{"value":["magical","trap"],"size":{"value":"med"}}
        },
        "items":[{"_id":"shuffle000000001","type":"action","name":"Blade Shuffle","system":{"actionType":{"value":"action"},"actions":{"value":1},"traits":{"value":["visual"]},"description":{"value":"<p>A separate effect calls for a @Check[reflex|dc:25|basic] save.</p>"}}}]
    })));
    records.extend((1..=32).map(|number| {
        record(
            "browser-actors",
            "Actor",
            json!({
                "_id":format!("patrol{number:010}"), "type":"npc",
                "name":format!("Watchtower Patrol {number:02} — Keeper of the Eastern Lanterns and the Narrow Mountain Passage"),
                "system":actor_system(number % 8, format!("<p>Patrol {number} guards the mountain passage and records each approaching caravan.</p>")),
                "items":[]
            }),
        )
    }));
    records.extend([
        record("browser-editions", "Item", json!({"_id":"legacy0000000001","type":"action","name":"Legacy signal","system":{"description":{"value":"<p>Legacy signal instructions.</p>"}}})),
        record("browser-editions", "Item", json!({"_id":"remaster00000001","type":"action","name":"Modern signal","system":{"description":{"value":"<p>Modern signal instructions.</p>"}}})),
        record("browser-journals", "JournalEntry", json!({"_id":"editionjournal01","name":"Remaster Changes","pages":[{"_id":"editionpage00001","name":"Class Features","type":"text","text":{"content":"<p>Explicit edition references: @UUID[Compendium.pf2e.browser-editions.Item.legacy0000000001]{Legacy signal} becomes @UUID[Compendium.pf2e.browser-editions.Item.remaster00000001]{Modern signal}.</p>","format":1}}]})),
    ]);
    records
}

fn sentinel() -> Value {
    let mut items = vec![json!({
        "_id":"claw000000000001", "type":"melee", "name":"Claw",
        "system": {
            "bonus":{"value":12}, "traits":{"value":["agile","finesse"]},
            "damageRolls":{"main":{"damage":"1d6+4","damageType":"slashing"}},
            "description":{"value":format!("<p>The sentinel's brass talons rake its target. Consult {RULE_LINK} before sounding an alarm.</p>")}
        }
    })];
    items.extend((0..70).map(|number| {
        let (id, name) = if number == 68 {
            ("signalpulse00001".to_owned(), "Signal Pulse".to_owned())
        } else {
            (format!("ability{number:09}"), format!("Lantern Technique {number:02}"))
        };
        json!({
            "_id":id, "type":"action", "name":name,
            "system": {
                "actionType":{"value":"action"}, "actions":{"value":2},
                "traits":{"value":["auditory","concentrate","manipulate"]},
                "description":{"value":format!("<p>Technique {number} directs the watchtower's lantern toward the narrow passage. A creature caught in the light becomes {CONDITION_LINK}.</p>{}<p>Continue with {RULE_LINK} for the nested signal reference.</p>", reading_prose(3))}
            }
        })
    }));
    json!({
        "_id":"sentinel00000001", "type":"npc", "name":"Atlas Sentinel",
        "system":actor_system(5, format!("<p>The Atlas Sentinel watches the narrow mountain passage. Its lantern marks each traveler before its brass talons move.</p><p>Follow {RULE_LINK} to inspect the signal reference.</p>{}", reading_prose(18))),
        "items":items
    })
}

fn actor_system(level: i32, notes: String) -> Value {
    json!({
        "details":{"level":{"value":level},"publicNotes":notes,"languages":{"value":["common","draconic"],"details":"Understands the watchtower's lantern signals."}},
        "attributes":{"adjustment":null,"ac":{"value":22},"hp":{"max":60,"details":"A steady blue light burns behind the breastplate."},"speed":{"value":25,"otherSpeeds":[]}},
        "saves":{"fortitude":{"value":15},"reflex":{"value":12},"will":{"value":12}},
        "perception":{"mod":13,"details":"Recognizes familiar lantern patterns."},
        "abilities":{"str":{"mod":4},"dex":{"mod":3},"con":{"mod":2},"int":{"mod":1},"wis":{"mod":2},"cha":{"mod":0}},
        "skills":{"athletics":{"base":12},"stealth":{"base":10},"arcana":{"base":9}},
        "traits":{"value":["construct","clockwork","fire","earth","magical","mindless"],"rarity":"uncommon","size":{"value":"med"}}
    })
}

fn lantern_ward() -> Value {
    json!({
        "_id":"spell00000000001", "type":"spell", "name":"Lantern Ward",
        "system": {
            "level":{"value":2}, "traits":{"value":["concentrate","manipulate","light"],"traditions":["arcane","divine"],"rarity":"common"},
            "time":{"value":"2"}, "range":{"value":"60 feet"}, "target":{"value":"one willing creature"},
            "duration":{"value":"1 minute","sustained":false}, "area":null,"defense":null,
            "description":{"value":format!("<p>A warm light traces the warded creature's outline. The ward's signal follows {RULE_LINK}.</p>{}", reading_prose(2))}
        }
    })
}

fn reading_prose(paragraphs: usize) -> String {
    (1..=paragraphs)
        .map(|number| format!("<p>Watch log {number}: The keeper checks the amber lens, secures the shutter, and waits until the last caravan has crossed the bridge. Wind moves through the iron railing while the lantern reflects on wet stone. The next watch repeats the signal slowly so every lookout can see the change.</p>"))
        .collect()
}
