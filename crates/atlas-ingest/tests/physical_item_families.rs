use atlas_ingest::item_source::{ItemNumberValueSource, ItemParentSource};
use atlas_ingest::physical_item_source::{
    BookCategorySource, PhysicalFamilySource, TreasureStackGroupSource,
    VersionedPhysicalItemSource, parse_physical_item_source,
};
use atlas_ingest::{
    ActorType, ItemType, SourceDiagnostic, SourceDiagnosticKind, SourceIdentity, SourcePresence,
    pinned_source_version_metadata,
};
use serde_json::{Value, json};

fn identity() -> SourceIdentity {
    SourceIdentity::new("fixture:family", "packs/fixture/family.json")
}
fn document(family: &str, system: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({"name":"Authored item", "type":family, "system":system})).unwrap()
}
fn parse_bytes(bytes: &[u8], parent: Option<ItemParentSource>) -> VersionedPhysicalItemSource {
    parse_physical_item_source(pinned_source_version_metadata(), identity(), parent, bytes)
        .expect("family source parses")
}
fn parse(family: &str, system: Value) -> VersionedPhysicalItemSource {
    parse_bytes(&document(family, system), None)
}
fn error(family: &str, system: Value) -> SourceDiagnostic {
    parse_physical_item_source(
        pinned_source_version_metadata(),
        identity(),
        Some(ItemParentSource {
            actor_type: ActorType::Npc,
            item_ordinal: 3,
        }),
        &document(family, system),
    )
    .unwrap_err()
}

#[test]
fn backpack_bulk_composes_shared_value_with_typed_container_fields() {
    let parsed = parse(
        "backpack",
        json!({"bulk":{"value":0.1,"heldOrStowed":0,"capacity":12.5,"ignored":0},"stowing":false,"collapsed":false}),
    );
    let PhysicalFamilySource::Backpack(family) = &parsed.source.family else {
        panic!("backpack refinement")
    };
    assert_eq!(family.stowing, SourcePresence::Value(false));
    assert_eq!(family.collapsed, SourcePresence::Value(false));
    let bulk = family.bulk.as_value().unwrap();
    assert_eq!(bulk.held_or_stowed.as_value().unwrap().as_i64(), Some(0));
    assert_eq!(bulk.capacity.as_value().unwrap().as_f64(), Some(12.5));
    assert_eq!(bulk.ignored.as_value().unwrap().as_i64(), Some(0));
    assert_eq!(
        parsed
            .source
            .physical
            .bulk
            .as_value()
            .unwrap()
            .value
            .as_value()
            .unwrap()
            .as_f64(),
        Some(0.1)
    );
    assert!(
        parsed
            .source
            .physical
            .bulk
            .as_value()
            .unwrap()
            .additional_fields
            .fields()
            .is_empty()
    );
    assert!(
        parsed
            .source
            .common
            .system
            .pending_family_fields
            .fields()
            .is_empty()
    );
}

#[test]
fn declaration_only_books_keep_category_capacity_and_ordered_references() {
    for (category, expected) in [
        ("formula", BookCategorySource::Formula),
        ("spell", BookCategorySource::Spell),
    ] {
        let refs = vec![
            "Compendium.pf2e.spells-srd.Item.first",
            "Compendium.pf2e.equipment-srd.Item.second",
            "Compendium.pf2e.spells-srd.Item.first",
        ];
        let parsed = parse(
            "book",
            json!({"category":category,"capacity":0,"contents":refs,"traits":{"value":["future-equipment-trait"],"rarity":"common"}}),
        );
        let PhysicalFamilySource::Book(family) = &parsed.source.family else {
            panic!("book refinement")
        };
        assert_eq!(family.category, SourcePresence::Value(expected));
        assert_eq!(family.capacity.as_value().unwrap().as_i64(), Some(0));
        assert_eq!(family.contents.as_value().unwrap(), &refs);
        assert!(
            parsed
                .source
                .common
                .system
                .pending_family_fields
                .fields()
                .is_empty()
        );
    }
}

#[test]
fn treasure_group_and_trait_applicability_follow_source_not_prepared_data() {
    for (group, expected) in [
        (
            json!("coins"),
            SourcePresence::Value(TreasureStackGroupSource::Coins),
        ),
        (
            json!("gems"),
            SourcePresence::Value(TreasureStackGroupSource::Gems),
        ),
        (Value::Null, SourcePresence::Null),
    ] {
        let parsed = parse(
            "treasure",
            json!({"stackGroup":group,"traits":{"value":[],"rarity":"unique","otherTags":["authored-tag"]},"equipped":{"invested":true}}),
        );
        let PhysicalFamilySource::Treasure(family) = &parsed.source.family else {
            panic!("treasure refinement")
        };
        assert_eq!(family.stack_group, expected);
        // TreasureEquippedData forbids invested only in prepared Data. It is
        // still an optional member of the shared EquippedData Source contract.
        assert_eq!(
            parsed.source.physical.equipped.as_value().unwrap().invested,
            SourcePresence::Value(true)
        );
        assert!(
            parsed
                .source
                .common
                .system
                .pending_family_fields
                .fields()
                .is_empty()
        );
    }
    let invalid = error("treasure", json!({"traits":{"value":["magical"]}}));
    assert_eq!(invalid.json_path(), "$.items[3].system.traits.value");
    assert_eq!(invalid.actual_shape(), r#"["magical"]"#);
}

#[test]
fn exact_pin_family_and_legacy_fixtures_share_root_and_actor_parsing() {
    for bytes in [
        include_bytes!("fixtures/physical-item-source/equipment.json").as_slice(),
        include_bytes!("fixtures/physical-item-source/backpack.json").as_slice(),
        include_bytes!("fixtures/physical-item-source/gold-pieces.json").as_slice(),
        include_bytes!("fixtures/physical-item-source/belt-of-good-health.json").as_slice(),
    ] {
        let root = parse_bytes(bytes, None);
        assert!(!matches!(root.source.family, PhysicalFamilySource::Pending));
        assert!(
            root.source
                .common
                .system
                .pending_family_fields
                .fields()
                .is_empty()
        );
        for actor_type in ActorType::ALL {
            let parent = Some(ItemParentSource {
                actor_type,
                item_ordinal: 6,
            });
            let embedded = parse_bytes(bytes, parent);
            assert_eq!(embedded.source, root.source);
            assert_eq!(embedded.parent, parent);
        }
    }
}

#[test]
fn observed_equipment_weapon_like_fields_have_typed_legacy_owners() {
    let parsed = parse_bytes(
        include_bytes!("fixtures/physical-item-source/belt-of-good-health.json"),
        None,
    );
    let PhysicalFamilySource::Equipment(family) = &parsed.source.family else {
        panic!("equipment refinement")
    };
    let legacy = &family.legacy;
    assert_eq!(
        legacy.ability.as_value().unwrap().value,
        SourcePresence::Value("str".into())
    );
    assert_eq!(
        legacy.map.as_value().unwrap().value,
        SourcePresence::Value("".into())
    );
    for value in [&legacy.bonus, &legacy.bonus_damage, &legacy.splash_damage] {
        assert_eq!(
            value.as_value().unwrap().value.as_value().unwrap().as_i64(),
            Some(0)
        );
    }
    for value in [&legacy.range, &legacy.reload, &legacy.weapon_type] {
        assert_eq!(
            value.as_value().unwrap().value,
            SourcePresence::Value("".into())
        );
    }
    let damage = legacy.damage.as_value().unwrap();
    assert_eq!(damage.damage_type, SourcePresence::Value("slashing".into()));
    assert_eq!(damage.dice.as_value().unwrap().as_i64(), Some(1));
    assert_eq!(damage.die, SourcePresence::Value("d6".into()));
    assert_eq!(damage.value, SourcePresence::Value("".into()));
    assert_eq!(parsed.source.common.envelope.item_type, ItemType::Equipment);
    assert!(
        parsed
            .source
            .common
            .system
            .pending_family_fields
            .fields()
            .is_empty()
    );
}

#[test]
fn numeric_value_payload_is_shared_across_common_physical_and_legacy_fields() {
    let parsed = parse(
        "equipment",
        json!({"level":{"value":1.5},"bulk":{"value":1.5},"bonus":{"value":1.5},"stowing":false}),
    );
    let PhysicalFamilySource::Equipment(family) = &parsed.source.family else {
        panic!("equipment refinement")
    };
    let level: &SourcePresence<ItemNumberValueSource> = &parsed.source.common.system.level;
    let bulk: &SourcePresence<ItemNumberValueSource> = &parsed.source.physical.bulk;
    let bonus: &SourcePresence<ItemNumberValueSource> = &family.legacy.bonus;
    assert_eq!(level, bulk);
    assert_eq!(bulk, bonus);
    assert_eq!(family.legacy.stowing, SourcePresence::Value(false));
}

#[test]
fn broad_equipment_traits_and_usage_preserve_new_strings() {
    let parsed = parse(
        "equipment",
        json!({"traits":{"value":["future-trait","future-trait"],"rarity":"common"},"usage":{"value":"new-authored-usage"},"equipped":{"invested":null}}),
    );
    assert_eq!(
        parsed
            .source
            .common
            .system
            .traits
            .as_value()
            .unwrap()
            .value
            .as_value()
            .unwrap(),
        &vec!["future-trait".to_string(), "future-trait".to_string()]
    );
    assert_eq!(
        parsed.source.physical.usage.as_value().unwrap().value,
        SourcePresence::Value("new-authored-usage".into())
    );
    assert_eq!(
        parsed.source.physical.equipped.as_value().unwrap().invested,
        SourcePresence::Null
    );
}

#[test]
fn family_fields_preserve_missing_null_empty_and_unknown_evidence() {
    let missing = parse("book", json!({}));
    let PhysicalFamilySource::Book(missing) = missing.source.family else {
        panic!("book refinement")
    };
    assert_eq!(missing.category, SourcePresence::Missing);
    assert_eq!(missing.contents, SourcePresence::Missing);
    let null = parse(
        "book",
        json!({"category":null,"capacity":null,"contents":null}),
    );
    let PhysicalFamilySource::Book(null) = null.source.family else {
        panic!("book refinement")
    };
    assert_eq!(null.category, SourcePresence::Null);
    assert_eq!(null.capacity, SourcePresence::Null);
    assert_eq!(null.contents, SourcePresence::Null);
    let empty = parse("book", json!({"contents":[],"unknown":{"value":false}}));
    let PhysicalFamilySource::Book(book) = empty.source.family else {
        panic!("book refinement")
    };
    assert!(book.contents.as_value().unwrap().is_empty());
    assert_eq!(
        empty
            .source
            .common
            .system
            .pending_family_fields
            .compact_json(),
        r#"{"unknown":{"value":false}}"#
    );
    for bulk in [Value::Null, json!({})] {
        let parsed = parse(
            "backpack",
            json!({"bulk":bulk,"stowing":null,"collapsed":null}),
        );
        let PhysicalFamilySource::Backpack(family) = parsed.source.family else {
            panic!("backpack refinement")
        };
        assert_eq!(family.bulk.is_null(), bulk.is_null());
        assert_eq!(family.stowing, SourcePresence::Null);
        if let Some(b) = family.bulk.as_value() {
            assert_eq!(b.capacity, SourcePresence::Missing);
        }
    }
    let missing = parse("backpack", json!({}));
    let PhysicalFamilySource::Backpack(missing) = missing.source.family else {
        panic!("backpack refinement")
    };
    assert_eq!(missing.bulk, SourcePresence::Missing);
}

#[test]
fn forbidden_fields_require_absence_even_when_null_empty_or_malformed() {
    for (family, key) in [
        ("backpack", "subitems"),
        ("book", "subitems"),
        ("treasure", "subitems"),
        ("treasure", "apex"),
        ("treasure", "usage"),
    ] {
        for value in [Value::Null, json!([]), json!({}), json!(42)] {
            let mut system = json!({});
            system[key] = value.clone();
            let err = error(family, system);
            assert_eq!(err.json_path(), format!("$.items[3].system.{key}"));
            assert_eq!(err.expected_shape(), "field absent in this family source");
            assert_eq!(err.actual_shape(), serde_json::to_string(&value).unwrap());
        }
    }
}

#[test]
fn recursive_children_receive_their_family_refinements_and_paths() {
    let child = json!({"name":"Book","type":"book","system":{"category":"formula","capacity":2,"contents":[]}});
    let parsed = parse("equipment", json!({"subitems":[child.clone(),child]}));
    let children = parsed.source.physical.subitems.as_value().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], children[1]);
    assert!(matches!(children[0].family, PhysicalFamilySource::Book(_)));
    assert_eq!(children[0].common.envelope.id, SourcePresence::Missing);
    let err = error(
        "equipment",
        json!({"subitems":[{"name":"Book","type":"book","system":{"category":"ritual"}}]}),
    );
    assert_eq!(
        err.json_path(),
        "$.items[3].system.subitems[0].system.category"
    );
    for family in ["armor", "consumable", "shield", "weapon"] {
        let parsed = parse(
            family,
            json!({"category":"specialized","damage":{"dice":2}}),
        );
        assert_eq!(parsed.source.family, PhysicalFamilySource::Pending);
        assert_eq!(
            parsed
                .source
                .common
                .system
                .pending_family_fields
                .fields()
                .len(),
            2
        );
    }
}

#[test]
fn legacy_slot_and_null_deletion_marker_are_typed_with_quoted_error_paths() {
    let parsed = parse("equipment", json!({"equipped":{"slot":"","-=inSlot":null}}));
    let equipped = parsed.source.physical.equipped.as_value().unwrap();
    assert_eq!(equipped.legacy_slot, SourcePresence::Value("".into()));
    assert_eq!(equipped.legacy_in_slot_deletion, SourcePresence::Null);
    assert!(equipped.additional_fields.fields().is_empty());
    let err = error("equipment", json!({"equipped":{"-=inSlot":false}}));
    assert_eq!(err.json_path(), r#"$.items[3].system.equipped["-=inSlot"]"#);
    assert_eq!(err.expected_shape(), "null deletion marker");
}

#[test]
fn malformed_family_values_report_exact_context() {
    for (family, system, path) in [
        ("backpack", json!({"stowing":"false"}), "stowing"),
        ("backpack", json!({"collapsed":1}), "collapsed"),
        (
            "backpack",
            json!({"bulk":{"capacity":"2"}}),
            "bulk.capacity",
        ),
        ("book", json!({"category":"ritual"}), "category"),
        ("book", json!({"capacity":false}), "capacity"),
        ("book", json!({"contents":[1]}), "contents[0]"),
        ("treasure", json!({"stackGroup":"bars"}), "stackGroup"),
        ("equipment", json!({"damage":{"dice":"1"}}), "damage.dice"),
        ("equipment", json!({"bonus":{"value":false}}), "bonus.value"),
        ("equipment", json!({"MAP":{"value":1}}), "MAP.value"),
        ("equipment", json!({"equipped":{"slot":0}}), "equipped.slot"),
    ] {
        let err = error(family, system);
        assert_eq!(err.kind, SourceDiagnosticKind::MalformedShape);
        assert_eq!(err.json_path(), format!("$.items[3].system.{path}"));
        assert_eq!(err.record_key(), "fixture:family");
        assert_eq!(err.source_path(), "packs/fixture/family.json");
        assert_eq!(err.source_system_version, "6.12.4");
    }
}

#[test]
fn duplicate_family_and_legacy_members_fail_without_selecting_one_value() {
    for (family, system, path) in [
        ("backpack", r#"{"stowing":false,"stowing":true}"#, "stowing"),
        (
            "backpack",
            r#"{"bulk":{"value":0,"capacity":1,"capacity":2}}"#,
            "bulk.capacity",
        ),
        (
            "book",
            r#"{"category":"formula","category":"spell"}"#,
            "category",
        ),
        (
            "treasure",
            r#"{"stackGroup":"coins","stackGroup":"gems"}"#,
            "stackGroup",
        ),
        (
            "equipment",
            r#"{"bonus":{"value":1,"value":2}}"#,
            "bonus.value",
        ),
        (
            "equipment",
            r#"{"equipped":{"-=inSlot":null,"-=inSlot":null}}"#,
            r#"equipped["-=inSlot"]"#,
        ),
    ] {
        let bytes = format!(r#"{{"name":"n","type":"{family}","system":{system}}}"#);
        let err = parse_physical_item_source(
            pinned_source_version_metadata(),
            identity(),
            None,
            bytes.as_bytes(),
        )
        .unwrap_err();
        assert_eq!(err.json_path(), format!("$.system.{path}"));
        assert!(err.actual_shape().contains("duplicate members"));
    }
}
