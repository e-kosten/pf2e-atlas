use super::tests::{database, result};
use crate::foundation_tests::record;
use atlas_record::source_content::{ContentAudience, ContentVisibilityRule};
use atlas_record::source_record::prepare_record_content;
use rusqlite::{Connection, params};
use serde_json::{Value, json};

fn fixtures(records: Vec<(&str, Value)>) -> Connection {
    let db = database();
    let mut ids = crate::projections::ProjectionIds::default();
    for (index, (kind, mut raw)) in records.into_iter().enumerate() {
        let id = i64::try_from(index + 1).unwrap();
        raw["_id"] = json!(format!("{id:016}"));
        raw["name"] = json!(format!("Fixture {id}"));
        let mut input = record("test", kind, raw);
        input.content = prepare_record_content(
            &input.record,
            ContentAudience {
                include_gm: false,
                include_owner: false,
                implicit_check_dc: ContentVisibilityRule::Gm,
            },
            None,
            None,
        );
        db.execute(
            "INSERT INTO record_identity(id,key,availability) VALUES(?,?,'available')",
            params![id, input.record.key().to_string()],
        )
        .unwrap();
        for row in crate::projections::root_rows(&input, "Test", id, &mut ids).unwrap() {
            row.insert(&db).unwrap();
        }
    }
    db
}

#[test]
fn backgrounds_combine_skill_and_boost_options_with_nested_or() {
    let free = json!({"value":["str","dex","con","int","wis","cha"]});
    let direct = json!({"value":["medicine"]});
    let choice = json!({
        "key":"ChoiceSet","flag":"training",
        "choices":[
            {"label":"Medicine","value":{"skill":"medicine"},"predicate":["skill:medicine:rank:0"]},
            {"label":"Society","value":{"skill":"society"}}
        ]
    });
    let upgrade = json!({
        "key":"ActiveEffectLike","mode":"upgrade","value":1,
        "path":"system.skills.{item|flags.system.rulesSelections.training.skill}.rank"
    });
    let db = fixtures(vec![
        (
            "Item",
            json!({"type":"background","system":{"boosts":{"0":{"value":["cha","int"]},"1":free},"trainedSkills":direct}}),
        ),
        (
            "Item",
            json!({"type":"background","system":{"boosts":{"0":{"value":["int","wis"]},"1":free},"trainedSkills":{"value":[]},"rules":[choice,upgrade]}}),
        ),
        (
            "Item",
            json!({"type":"background","system":{"boosts":{"0":{"value":["cha"]},"1":{"value":[]}},"trainedSkills":direct}}),
        ),
        (
            "Item",
            json!({"type":"background","system":{"boosts":{"0":{"value":["str","wis"]}},"trainedSkills":direct}}),
        ),
        (
            "Item",
            json!({"type":"background","system":{"boosts":{"0":{"value":["cha"]}},"trainedSkills":{"value":[]},"rules":[choice,choice,upgrade]}}),
        ),
        (
            "Item",
            json!({"type":"background","system":{"boosts":{"0":{"value":["cha"]},"1":{"value":null}}}}),
        ),
        (
            "Item",
            json!({"type":"background","system":{"boosts":{"0":{"value":null},"1":free}}}),
        ),
    ]);
    assert_eq!(
        result(&db, "'cha' in background.restricted_attribute_boosts"),
        vec![1, 3, 5]
    );
    assert_eq!(
        result(&db, "background.has_free_attribute_boost == true"),
        vec![1, 2, 7]
    );
    assert_eq!(
        result(&db, "background.has_free_attribute_boost == false"),
        vec![3, 4, 5]
    );
    assert_eq!(
        result(&db, "'medicine' in background.skill_training"),
        vec![1, 2, 3, 4]
    );
    assert_eq!(
        result(&db, "'society' in background.skill_training"),
        vec![2]
    );
    assert_eq!(
        result(
            &db,
            "kind == 'background' && 'medicine' in background.skill_training && ('cha' in background.restricted_attribute_boosts || background.has_free_attribute_boost == true)"
        ),
        vec![1, 2, 3]
    );
}

#[test]
fn ancestries_classes_and_deities_use_authored_family_facts() {
    let db = fixtures(vec![
        (
            "Item",
            json!({"type":"ancestry","system":{"vision":"darkvision","speed":25,"hp":8,"alternateAncestryBoosts":["cha","con"],"boosts":{"0":{"value":["str"]},"1":{"value":["str","dex","con","int","wis","cha"]}},"flaws":{"0":{"value":["int"]}}}}),
        ),
        (
            "Item",
            json!({"type":"class","system":{"keyAbility":{"value":["str","dex"]},"attacks":{"martial":2},"defenses":{"heavy":0}}}),
        ),
        (
            "Item",
            json!({"type":"class","system":{"keyAbility":{"value":[]},"defenses":{"heavy":1}}}),
        ),
        (
            "Item",
            json!({"type":"deity","system":{"category":"deity","skill":["religion"],"weapons":["longsword"],"sanctification":{"modal":"can","what":["holy","unholy"]}}}),
        ),
        (
            "Item",
            json!({"type":"deity","system":{"category":"pantheon","sanctification":null}}),
        ),
        (
            "Item",
            json!({"type":"deity","system":{"sanctification":{"modal":"must","what":["holy"]}}}),
        ),
        ("Item", json!({"type":"deity","system":{}})),
    ]);
    assert_eq!(
        result(
            &db,
            "ancestry.vision == 'darkvision' && ancestry.land_speed >= 25 && ancestry.starting_hp == 8 && 'str' in ancestry.default_boosts && 'int' in ancestry.default_flaws"
        ),
        vec![1]
    );
    assert_eq!(
        result(&db, "'cha' in ancestry.default_boosts"),
        Vec::<i64>::new()
    );
    assert_eq!(
        result(
            &db,
            "'dex' in class.key_attributes && class.starting_proficiency.martial_weapons == 'expert' && class.starting_proficiency.heavy_armor == 'untrained'"
        ),
        vec![2]
    );
    assert_eq!(
        result(&db, "class.starting_proficiency.heavy_armor == 'trained'"),
        vec![3]
    );
    assert_eq!(
        result(
            &db,
            "'longsword' in deity.favored_weapons && 'religion' in deity.divine_skills && deity.category == 'deity' && deity.sanctification.requirement == 'optional' && 'unholy' in deity.sanctification.options"
        ),
        vec![4]
    );
    assert_eq!(
        result(&db, "deity.sanctification.requirement == 'none'"),
        vec![5]
    );
    assert_eq!(
        result(
            &db,
            "deity.sanctification.requirement == 'required' && 'holy' in deity.sanctification.options"
        ),
        vec![6]
    );
    assert_eq!(
        result(
            &db,
            "availability['deity.sanctification.requirement'] == 'missing'"
        ),
        vec![7]
    );
}

#[test]
fn hazard_skills_come_from_visible_disable_checks_only() {
    let db = fixtures(vec![
        (
            "Actor",
            json!({"type":"hazard","system":{"details":{
                "disable":"<p>@Check[type:arcana,nature,thievery|dc:20] or Medicine prose.</p><p data-visibility='gm'>@Check[religion|dc:22]</p>",
                "routine":"<p>@Check[athletics|dc:22]</p>"
            }}}),
        ),
        (
            "Actor",
            json!({"type":"hazard","system":{"details":{"disable":"<p data-visibility='gm'>@Check[religion|dc:22]</p>"}}}),
        ),
        (
            "Actor",
            json!({"type":"hazard","system":{"details":{"disable":""}}}),
        ),
        ("Actor", json!({"type":"hazard","system":{"details":{}}})),
    ]);
    for skill in ["arcana", "nature", "thievery"] {
        assert_eq!(
            result(&db, &format!("'{skill}' in hazard.disable_skills")),
            vec![1]
        );
    }
    for skill in ["athletics", "religion", "medicine"] {
        assert!(result(&db, &format!("'{skill}' in hazard.disable_skills")).is_empty());
    }
    assert_eq!(
        result(&db, "availability['hazard.disable_skills'] == 'value'"),
        vec![1, 2, 3]
    );
    assert_eq!(
        result(&db, "availability['hazard.disable_skills'] == 'missing'"),
        vec![4]
    );
}

#[test]
fn casting_range_distinguishes_touch_distance_and_unknown_in_both_scopes() {
    let mixed = json!({"_id":"mixed00000000000","type":"spell","system":{"range":{"value":"touch or 30 feet"}}});
    let db = fixtures(vec![
        (
            "Item",
            json!({"type":"spell","system":{"range":{"value":"Touch"}}}),
        ),
        (
            "Item",
            json!({"type":"spell","system":{"range":{"value":"1 mile"}}}),
        ),
        ("Item", mixed.clone()),
        (
            "Item",
            json!({"type":"spell","system":{"range":{"value":"30 feet (burst only)"}}}),
        ),
        ("Actor", json!({"type":"npc","items":[mixed]})),
        (
            "Item",
            json!({"type":"spell","system":{"range":{"value":""}}}),
        ),
    ]);
    assert_eq!(result(&db, "spell.touch_range == true"), vec![1, 3]);
    assert_eq!(result(&db, "spell.touch_range == false"), vec![2]);
    assert_eq!(result(&db, "spell.cast_range >= 30"), vec![2, 3]);
    assert_eq!(
        result(
            &db,
            "embedded.spells.exists(s, s.spell.cast_range == 30 && s.spell.touch_range == true)"
        ),
        vec![5]
    );
}
