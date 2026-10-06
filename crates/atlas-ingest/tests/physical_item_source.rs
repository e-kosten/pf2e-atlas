use atlas_ingest::item_source::{
    ItemParentSource, SerializedSourceValue, parse_common_item_source,
};
use atlas_ingest::physical_item_source::*;
use atlas_ingest::{
    ActorType, ItemType, SourceDiagnostic, SourceDiagnosticKind, SourceIdentity, SourcePresence,
    pinned_source_version_metadata,
};
use atlas_record::{CreatureFrequencyPeriod, CreatureSize};
use serde_json::{Value, json};

fn identity() -> SourceIdentity {
    SourceIdentity::new("fixture:item", "packs/fixture/item.json")
}
fn parse_bytes(bytes: &[u8], parent: Option<ItemParentSource>) -> VersionedPhysicalItemSource {
    parse_physical_item_source(pinned_source_version_metadata(), identity(), parent, bytes)
        .expect("physical source parses")
}
fn parse(system: Value) -> VersionedPhysicalItemSource {
    parse_bytes(
        &serde_json::to_vec(&json!({"name":"Physical item","type":"equipment","system":system}))
            .unwrap(),
        None,
    )
}
fn error(system: Value) -> SourceDiagnostic {
    parse_physical_item_source(
        pinned_source_version_metadata(),
        identity(),
        Some(ItemParentSource {
            actor_type: ActorType::Hazard,
            item_ordinal: 2,
        }),
        &serde_json::to_vec(&json!({"name":"Physical item","type":"equipment","system":system}))
            .unwrap(),
    )
    .unwrap_err()
}
fn activation() -> Value {
    json!({"id":"authored-id","description":{"value":"<p>Activate</p>"},"actionCost":{"type":"action","value":2},"components":{"command":false,"envision":true,"interact":false,"cast":true},"frequency":{"value":0,"max":2.5,"per":"PT10M"},"traits":{"value":["manipulate","manipulate"]}})
}

#[test]
fn declaration_only_activation_variants_preserve_their_source_tokens() {
    for (token, expected) in [
        ("action", SourceActionType::Action),
        ("reaction", SourceActionType::Reaction),
        ("free", SourceActionType::Free),
    ] {
        let parsed = parse(json!({"activations":{"a":{"actionCost":{"type":token,"value":null}}}}));
        let cost = parsed.source.physical.activations.as_value().unwrap()[0]
            .1
            .as_value()
            .unwrap()
            .action_cost
            .as_value()
            .unwrap();
        assert_eq!(cost.action_type, SourcePresence::Value(expected));
        assert_eq!(cost.value, SourcePresence::Null);
    }
    for token in [
        "turn", "round", "PT1M", "PT10M", "PT1H", "PT24H", "day", "P1W", "P1M", "P1Y",
    ] {
        let parsed = parse(json!({"activations":{"a":{"frequency":{"per":token}}}}));
        let period = parsed.source.physical.activations.as_value().unwrap()[0]
            .1
            .as_value()
            .unwrap()
            .frequency
            .as_value()
            .unwrap()
            .per
            .as_value()
            .unwrap();
        assert_eq!(period.source_token(), token);
    }
}

#[test]
fn physical_byte_boundary_rejects_malformed_json_and_unknown_discriminators() {
    for bytes in [b"[]".as_slice(), b"{} {}", b"{", b"null"] {
        let error =
            parse_physical_item_source(pinned_source_version_metadata(), identity(), None, bytes)
                .unwrap_err();
        assert_eq!(error.kind, SourceDiagnosticKind::MalformedShape);
        assert_eq!(error.json_path(), "$");
    }
    let error = parse_physical_item_source(
        pinned_source_version_metadata(),
        identity(),
        None,
        br#"{"name":"n","type":"invented","system":{}}"#,
    )
    .unwrap_err();
    assert_eq!(error.kind, SourceDiagnosticKind::UnknownDiscriminator);
    assert_eq!(error.json_path(), "$.type");
}

#[test]
fn physical_common_fields_are_typed_without_defaults_or_numeric_coercion() {
    let source = parse(json!({
        "quantity":0,"baseItem":null,"bulk":{"value":0.1,"heldOrStowed":1},
        "hp":{"value":0,"max":12.5},"hardness":0,"price":{"value":{"pp":0,"gp":1.25,"sp":null,"cp":2},"per":0,"sizeSensitive":false},
        "equipped":{"carryType":"held","handsHeld":0,"inSlot":false,"invested":null},
        "identification":{"status":"unidentified","unidentified":{"name":"Unknown","img":"icons/unknown.webp","data":{"description":{"value":""}}},"misidentified":{"name":"Mistaken","custom":false}},
        "containerId":"container","material":{"grade":"high","type":"cold-iron"},"size":"med",
        "usage":{"value":"held-in-one-hand","canBeAmmo":false},"temporary":false,"subitems":[],
        "apex":{"attribute":"dex","selected":false},"activations":{"activate":activation()},"damage":{"dice":1}
    }));
    let p = &source.source.physical;
    assert_eq!(p.quantity.as_value().unwrap().as_i64(), Some(0));
    assert_eq!(p.base_item, SourcePresence::Null);
    assert_eq!(
        p.bulk
            .as_value()
            .unwrap()
            .value
            .as_value()
            .unwrap()
            .as_f64(),
        Some(0.1)
    );
    assert_eq!(
        p.bulk.as_value().unwrap().additional_fields.compact_json(),
        r#"{"heldOrStowed":1}"#
    );
    assert_eq!(
        p.hp.as_value().unwrap().value.as_value().unwrap().as_i64(),
        Some(0)
    );
    assert_eq!(
        p.hp.as_value().unwrap().max.as_value().unwrap().as_f64(),
        Some(12.5)
    );
    assert_eq!(p.hardness.as_value().unwrap().as_i64(), Some(0));
    let price = p.price.as_value().unwrap();
    let coins = price.value.as_value().unwrap();
    assert_eq!(coins.pp.as_value().unwrap().as_i64(), Some(0));
    assert_eq!(coins.gp.as_value().unwrap().as_f64(), Some(1.25));
    assert_eq!(coins.sp, SourcePresence::Null);
    assert_eq!(coins.cp.as_value().unwrap().as_i64(), Some(2));
    assert_eq!(price.per.as_value().unwrap().as_i64(), Some(0));
    assert_eq!(price.size_sensitive, SourcePresence::Value(false));
    let equipped = p.equipped.as_value().unwrap();
    assert_eq!(
        equipped.carry_type,
        SourcePresence::Value(ItemCarryTypeSource::Held)
    );
    assert_eq!(equipped.hands_held, SourcePresence::Value(0));
    assert_eq!(equipped.in_slot, SourcePresence::Value(false));
    assert_eq!(equipped.invested, SourcePresence::Null);
    let identification = p.identification.as_value().unwrap();
    assert_eq!(
        identification.status,
        SourcePresence::Value(ItemIdentificationStatusSource::Unidentified)
    );
    let unknown = identification.unidentified.as_value().unwrap();
    assert_eq!(unknown.name, SourcePresence::Value("Unknown".into()));
    assert_eq!(
        unknown.image,
        SourcePresence::Value("icons/unknown.webp".into())
    );
    assert_eq!(
        unknown
            .data
            .as_value()
            .unwrap()
            .description
            .as_value()
            .unwrap()
            .value,
        SourcePresence::Value("".into())
    );
    assert_eq!(
        identification
            .misidentified
            .as_value()
            .unwrap()
            .compact_json(),
        r#"{"custom":false,"name":"Mistaken"}"#
    );
    assert_eq!(p.container_id, SourcePresence::Value("container".into()));
    let material = p.material.as_value().unwrap();
    assert_eq!(
        material.grade,
        SourcePresence::Value(ItemMaterialGradeSource::High)
    );
    assert_eq!(
        material.material_type,
        SourcePresence::Value(ItemMaterialTypeSource::ColdIron)
    );
    assert_eq!(p.size, SourcePresence::Value(CreatureSize::Medium));
    assert_eq!(
        p.usage.as_value().unwrap().value,
        SourcePresence::Value("held-in-one-hand".into())
    );
    assert_eq!(
        p.usage.as_value().unwrap().additional_fields.compact_json(),
        r#"{"canBeAmmo":false}"#
    );
    assert_eq!(p.temporary, SourcePresence::Value(false));
    assert!(p.subitems.as_value().unwrap().is_empty());
    assert_eq!(
        p.apex.as_value().unwrap().attribute,
        SourcePresence::Value(ApexAttributeSource::Dexterity)
    );
    assert_eq!(
        p.apex.as_value().unwrap().selected,
        SourcePresence::Value(false)
    );
    assert_eq!(
        source
            .source
            .common
            .system
            .pending_family_fields
            .compact_json(),
        "{}"
    );
    let activation = p.activations.as_value().unwrap()[0].1.as_value().unwrap();
    assert_eq!(activation.id, SourcePresence::Value("authored-id".into()));
    assert_eq!(
        activation.description.as_value().unwrap().value,
        SourcePresence::Value("<p>Activate</p>".into())
    );
    assert_eq!(
        activation.action_cost.as_value().unwrap().action_type,
        SourcePresence::Value(SourceActionType::Action)
    );
    assert_eq!(
        activation.action_cost.as_value().unwrap().value,
        SourcePresence::Value(2)
    );
    let components = activation.components.as_value().unwrap();
    assert_eq!(
        (
            components.command.clone(),
            components.envision.clone(),
            components.interact.clone(),
            components.cast.clone()
        ),
        (
            SourcePresence::Value(false),
            SourcePresence::Value(true),
            SourcePresence::Value(false),
            SourcePresence::Value(true)
        )
    );
    let frequency = activation.frequency.as_value().unwrap();
    assert_eq!(frequency.value.as_value().unwrap().as_i64(), Some(0));
    assert_eq!(frequency.max.as_value().unwrap().as_f64(), Some(2.5));
    assert_eq!(
        frequency.per,
        SourcePresence::Value(CreatureFrequencyPeriod::TenMinutes)
    );
    assert_eq!(
        activation
            .traits
            .as_value()
            .unwrap()
            .value
            .as_value()
            .unwrap(),
        &vec!["manipulate".to_string(), "manipulate".to_string()]
    );
}

#[test]
fn all_eight_physical_discriminators_include_declaration_only_book() {
    for item_type in [
        ItemType::Armor,
        ItemType::Backpack,
        ItemType::Book,
        ItemType::Consumable,
        ItemType::Equipment,
        ItemType::Shield,
        ItemType::Treasure,
        ItemType::Weapon,
    ] {
        let mut value = json!({"name":"Source","type":item_type.as_str(),"system":{"quantity":1}});
        if item_type == ItemType::Book {
            value["system"]["category"] = json!("formula");
            value["system"]["capacity"] = json!(10);
            value["system"]["contents"] = json!([]);
        }
        let parsed = parse_bytes(&serde_json::to_vec(&value).unwrap(), None);
        assert_eq!(parsed.source.common.envelope.item_type, item_type);
        assert_eq!(
            parsed.source.physical.quantity.as_value().unwrap().as_i64(),
            Some(1)
        );
        if item_type == ItemType::Book {
            assert_eq!(
                parsed
                    .source
                    .common
                    .system
                    .pending_family_fields
                    .fields()
                    .len(),
                0
            );
        }
    }
    let invalid = json!({"name":"Spell","type":"spell","system":{}});
    let error = parse_physical_item_source(
        pinned_source_version_metadata(),
        identity(),
        None,
        &serde_json::to_vec(&invalid).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.json_path(), "$.type");
}

#[test]
fn exact_pin_fixtures_share_the_parser_in_root_and_all_actor_contexts() {
    for (bytes, item_type, bulk) in [
        (
            include_bytes!("fixtures/physical-item-source/equipment.json").as_slice(),
            ItemType::Equipment,
            1.0,
        ),
        (
            include_bytes!("fixtures/physical-item-source/backpack.json").as_slice(),
            ItemType::Backpack,
            1.0,
        ),
        (
            include_bytes!("fixtures/physical-item-source/weapon.json").as_slice(),
            ItemType::Weapon,
            1.0,
        ),
        (
            include_bytes!("fixtures/physical-item-source/consumable.json").as_slice(),
            ItemType::Consumable,
            0.1,
        ),
    ] {
        let root = parse_bytes(bytes, None);
        assert_eq!(root.source.common.envelope.item_type, item_type);
        assert_eq!(
            root.source
                .physical
                .bulk
                .as_value()
                .unwrap()
                .value
                .as_value()
                .unwrap()
                .as_f64(),
            Some(bulk)
        );
        assert_eq!(
            root.source.physical.size,
            SourcePresence::Value(CreatureSize::Medium)
        );
        let common =
            parse_common_item_source(pinned_source_version_metadata(), identity(), None, bytes)
                .unwrap();
        assert_eq!(root.source.common.envelope, common.source.envelope);
        assert_eq!(
            root.source.common.system.description,
            common.source.system.description
        );
        assert_eq!(
            root.source.common.system.traits,
            common.source.system.traits
        );
        assert_eq!(
            root.source.common.system.publication,
            common.source.system.publication
        );
        for actor_type in ActorType::ALL {
            let parent = Some(ItemParentSource {
                actor_type,
                item_ordinal: 4,
            });
            let embedded = parse_bytes(bytes, parent);
            assert_eq!(embedded.source, root.source);
            assert_eq!(embedded.parent, parent);
        }
    }
}

#[test]
fn recursive_subitems_keep_identity_order_multiplicity_and_pending_bodies() {
    let child = json!({"_id":null,"name":"Attachment","type":"weapon","system":{"quantity":0,"damage":{"dice":2},"subitems":[{"name":"Nested","type":"equipment","system":{"temporary":false}}]}});
    let parsed = parse(json!({"subitems":[child.clone(),child]}));
    let children = parsed.source.physical.subitems.as_value().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], children[1]);
    assert_eq!(children[0].common.envelope.id, SourcePresence::Null);
    assert_eq!(
        children[0].physical.quantity.as_value().unwrap().as_i64(),
        Some(0)
    );
    assert_eq!(
        children[0]
            .common
            .system
            .pending_family_fields
            .compact_json(),
        r#"{"damage":{"dice":2}}"#
    );
    let grandchild = &children[0].physical.subitems.as_value().unwrap()[0];
    assert_eq!(grandchild.common.envelope.id, SourcePresence::Missing);
    assert_eq!(grandchild.common.envelope.name, "Nested");
    assert_eq!(grandchild.physical.temporary, SourcePresence::Value(false));
    let invalid = error(json!({"subitems":[{"name":"Wrong child","type":"spell","system":{}}]}));
    assert_eq!(invalid.json_path(), "$.items[2].system.subitems[0].type");
    let invalid = error(
        json!({"subitems":[{"name":"Bad child","type":"weapon","system":{"subitems":[{"name":"Bad grandchild","type":"equipment","system":{"quantity":"one"}}]}}]}),
    );
    assert_eq!(
        invalid.json_path(),
        "$.items[2].system.subitems[0].system.subitems[0].system.quantity"
    );
}

#[test]
fn sparse_null_and_empty_physical_structures_remain_distinct() {
    let missing = parse(json!({}));
    assert_eq!(missing.source.physical.quantity, SourcePresence::Missing);
    assert_eq!(
        missing.source.physical.identification,
        SourcePresence::Missing
    );
    assert_eq!(missing.source.physical.subitems, SourcePresence::Missing);
    let null = parse(
        json!({"quantity":null,"baseItem":null,"bulk":null,"hp":null,"hardness":null,"price":null,"equipped":null,"identification":null,"containerId":null,"material":null,"size":null,"usage":null,"activations":null,"temporary":null,"subitems":null,"apex":null}),
    );
    let p = null.source.physical;
    assert_eq!(p.quantity, SourcePresence::Null);
    assert_eq!(p.bulk, SourcePresence::Null);
    assert_eq!(p.hp, SourcePresence::Null);
    assert_eq!(p.price, SourcePresence::Null);
    assert_eq!(p.equipped, SourcePresence::Null);
    assert_eq!(p.identification, SourcePresence::Null);
    assert_eq!(p.material, SourcePresence::Null);
    assert_eq!(p.size, SourcePresence::Null);
    assert_eq!(p.usage, SourcePresence::Null);
    assert_eq!(p.activations, SourcePresence::Null);
    assert_eq!(p.temporary, SourcePresence::Null);
    assert_eq!(p.subitems, SourcePresence::Null);
    assert_eq!(p.apex, SourcePresence::Null);
    let empty = parse(
        json!({"price":{"value":{}},"material":{"grade":null,"type":null},"activations":{},"subitems":[],"apex":{"selected":null},"identification":{"unidentified":{"data":{"description":{}}}}}),
    );
    assert_eq!(
        empty
            .source
            .physical
            .price
            .as_value()
            .unwrap()
            .value
            .as_value()
            .unwrap()
            .gp,
        SourcePresence::Missing
    );
    assert_eq!(
        empty
            .source
            .physical
            .material
            .as_value()
            .unwrap()
            .material_type,
        SourcePresence::Null
    );
    assert!(
        empty
            .source
            .physical
            .activations
            .as_value()
            .unwrap()
            .is_empty()
    );
    assert!(
        empty
            .source
            .physical
            .subitems
            .as_value()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        empty.source.physical.apex.as_value().unwrap().selected,
        SourcePresence::Null
    );
    assert_eq!(
        empty
            .source
            .physical
            .identification
            .as_value()
            .unwrap()
            .unidentified
            .as_value()
            .unwrap()
            .data
            .as_value()
            .unwrap()
            .description
            .as_value()
            .unwrap()
            .value,
        SourcePresence::Missing
    );
}

#[test]
fn ordered_keyed_activations_and_declared_open_objects_preserve_repeated_members() {
    let parsed = parse_bytes(br#"{"name":"n","type":"equipment","system":{"activations":{"z":{"id":"one"},"a":null,"z":{"id":"two"}},"identification":{"misidentified":{"x":0,"x":false}},"family":0,"family":false},"flags":{"module":{"x":1,"x":2}}}"#,None);
    let a = parsed.source.physical.activations.as_value().unwrap();
    assert_eq!(
        a.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
        ["z", "a", "z"]
    );
    assert_eq!(
        a[0].1.as_value().unwrap().id,
        SourcePresence::Value("one".into())
    );
    assert_eq!(a[1].1, SourcePresence::Null);
    assert_eq!(
        a[2].1.as_value().unwrap().id,
        SourcePresence::Value("two".into())
    );
    assert_eq!(
        parsed
            .source
            .physical
            .identification
            .as_value()
            .unwrap()
            .misidentified
            .as_value()
            .unwrap()
            .compact_json(),
        r#"{"x":0,"x":false}"#
    );
    assert_eq!(
        parsed
            .source
            .common
            .system
            .pending_family_fields
            .compact_json(),
        r#"{"family":0,"family":false}"#
    );
    let arrays = parse(json!({"identification":{"misidentified":[false,null,0]}}));
    assert!(matches!(
        arrays
            .source
            .physical
            .identification
            .as_value()
            .unwrap()
            .misidentified,
        SourcePresence::Value(SerializedSourceValue::Array(_))
    ));
}

#[test]
fn duplicate_known_physical_members_fail_with_exact_paths() {
    for (system, path) in [
        (r#"{"quantity":1,"quantity":2}"#, "quantity"),
        (r#"{"bulk":{"value":0,"value":1}}"#, "bulk.value"),
        (r#"{"price":{"value":{"gp":1,"gp":2}}}"#, "price.value.gp"),
        (
            r#"{"activations":{"a.b":{"components":{"cast":true,"cast":false}}}}"#,
            "activations[\"a.b\"].components.cast",
        ),
        (
            r#"{"identification":{"unidentified":{"data":{"description":{"value":"a","value":"b"}}}}}"#,
            "identification.unidentified.data.description.value",
        ),
        (
            r#"{"subitems":[{"name":"child","type":"equipment","system":{"hp":{"max":1,"max":2}}}]}"#,
            "subitems[0].system.hp.max",
        ),
    ] {
        let bytes = format!(r#"{{"name":"n","type":"equipment","system":{system}}}"#);
        let error = parse_physical_item_source(
            pinned_source_version_metadata(),
            identity(),
            None,
            bytes.as_bytes(),
        )
        .unwrap_err();
        assert_eq!(error.json_path(), format!("$.system.{path}"));
        assert!(error.actual_shape().contains("duplicate members"));
    }
}

#[test]
fn malformed_shared_fields_have_contextual_diagnostics() {
    for (system, path) in [
        (json!({"quantity":"1"}), "quantity"),
        (json!({"baseItem":5}), "baseItem"),
        (json!({"bulk":{"value":false}}), "bulk.value"),
        (json!({"hp":[]}), "hp"),
        (json!({"price":{"value":{"gp":"2"}}}), "price.value.gp"),
        (
            json!({"equipped":{"carryType":"floating"}}),
            "equipped.carryType",
        ),
        (json!({"equipped":{"handsHeld":3}}), "equipped.handsHeld"),
        (json!({"equipped":{"handsHeld":0.5}}), "equipped.handsHeld"),
        (
            json!({"identification":{"status":"misidentified"}}),
            "identification.status",
        ),
        (
            json!({"identification":{"unidentified":{"data":{"description":{"value":1}}}}}),
            "identification.unidentified.data.description.value",
        ),
        (
            json!({"identification":{"misidentified":false}}),
            "identification.misidentified",
        ),
        (json!({"material":{"grade":"medium"}}), "material.grade"),
        (json!({"material":{"type":"invented"}}), "material.type"),
        (json!({"size":"medium"}), "size"),
        (json!({"usage":{"value":false}}), "usage.value"),
        (json!({"temporary":"false"}), "temporary"),
        (json!({"apex":{"attribute":"strength"}}), "apex.attribute"),
        (
            json!({"activations":{"a.b":{"actionCost":{"type":"passive"}}}}),
            "activations[\"a.b\"].actionCost.type",
        ),
        (
            json!({"activations":{"a.b":{"actionCost":{"value":0}}}}),
            "activations[\"a.b\"].actionCost.value",
        ),
        (
            json!({"activations":{"a.b":{"frequency":{"per":"hour"}}}}),
            "activations[\"a.b\"].frequency.per",
        ),
        (
            json!({"activations":{"a.b":{"components":{"cast":"false"}}}}),
            "activations[\"a.b\"].components.cast",
        ),
        (
            json!({"activations":{"a.b":{"traits":{"value":[1]}}}}),
            "activations[\"a.b\"].traits.value[0]",
        ),
        (json!({"subitems":[null]}), "subitems[0]"),
    ] {
        let error = error(system);
        assert_eq!(error.kind, SourceDiagnosticKind::MalformedShape);
        assert_eq!(error.json_path(), format!("$.items[2].system.{path}"));
        assert_eq!(error.record_key(), "fixture:item");
        assert_eq!(error.source_path(), "packs/fixture/item.json");
        assert_eq!(error.source_system_version, "6.12.4");
    }
}

#[test]
fn small_literal_counts_accept_integral_json_numbers_and_preserve_null() {
    for value in [json!(0), json!(1.0), json!(2)] {
        assert!(
            parse(json!({"equipped":{"handsHeld":value}}))
                .source
                .physical
                .equipped
                .as_value()
                .unwrap()
                .hands_held
                .as_value()
                .is_some()
        );
    }
    for value in [json!(1), json!(2.0), json!(3), Value::Null] {
        let parsed = parse(json!({"activations":{"a":{"actionCost":{"value":value}}}}));
        let count = &parsed.source.physical.activations.as_value().unwrap()[0]
            .1
            .as_value()
            .unwrap()
            .action_cost
            .as_value()
            .unwrap()
            .value;
        assert_eq!(matches!(count, SourcePresence::Null), value.is_null());
    }
}
