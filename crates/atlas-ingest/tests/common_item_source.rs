use atlas_domain::Rarity;
use atlas_ingest::item_source::{
    ItemGrantDeleteAction, ItemLicenseSource, ItemOwnershipLevel, ItemParentSource,
    ItemRuleSelectionSource, ItemTraitsShape, SerializedSourceValue, VersionedCommonItemSource,
    parse_common_item_source,
};
use atlas_ingest::{
    ActorType, ItemType, SourceDiagnosticKind, SourceIdentity, SourcePresence,
    pinned_source_version_metadata,
};
use serde_json::{Value, json};

fn identity() -> SourceIdentity {
    SourceIdentity::new("fixture:item", "packs/fixture/item.json")
}

fn parse_bytes(bytes: &[u8], parent: Option<ItemParentSource>) -> VersionedCommonItemSource {
    parse_common_item_source(pinned_source_version_metadata(), identity(), parent, bytes)
        .expect("common source parses")
}

fn parse(value: Value) -> VersionedCommonItemSource {
    parse_bytes(&serde_json::to_vec(&value).unwrap(), None)
}

fn minimal(item_type: &str, system: Value) -> Value {
    json!({"_id":"item", "name":"Authored name", "type":item_type, "system":system})
}

#[test]
fn typed_common_fields_retain_authored_values_and_presence() {
    let source = parse(json!({
        "_id":"item", "name":"Authored name", "type":"weapon", "img":"icons/item.webp",
        "folder":null, "sort":0, "ownership":{"default":-1,"user":3}, "effects":[],
        "_stats":{"systemId":"pf2e","systemVersion":"6.12.4","coreVersion":"13", "createdTime":0,"modifiedTime":12.5,"lastModifiedBy":null,"compendiumSource":"Compendium.pf2e.equipment.Item.item","duplicateSource":null},
        "flags":{"pf2e":{"rulesSelections":{"choice":"sword","count":0,"object":{"x":false}},"grantedBy":{"id":"parent","onDelete":"cascade"},"itemGrants":{"child":{"id":"child-id","onDelete":"detach","nested":false}},"custom":false},"module":{"value":0}},
        "system":{"level":{"value":2.5},"description":{"value":"<p>Authored text</p>","gm":""},"traits":{"value":["agile","agile"],"rarity":"rare","otherTags":[]},"rules":[],"slug":null,"publication":{"title":"Book","authors":"Author","license":"ORC","remaster":false},"_migration":{"version":0.932,"previous":{"foundry":null,"system":"6.11.0","schema":0.9}},"schema":{},"damage":{"dice":0}}
    }));
    let envelope = &source.source.envelope;
    let system = &source.source.system;
    assert_eq!(envelope.id, SourcePresence::Value("item".into()));
    assert_eq!(envelope.item_type, ItemType::Weapon);
    assert_eq!(envelope.folder, SourcePresence::Null);
    assert_eq!(envelope.sort, SourcePresence::Value(0));
    assert_eq!(
        envelope.ownership.as_value().unwrap(),
        &vec![
            (
                "default".into(),
                SourcePresence::Value(ItemOwnershipLevel::Inherit)
            ),
            (
                "user".into(),
                SourcePresence::Value(ItemOwnershipLevel::Owner)
            )
        ]
    );
    assert!(envelope.effects.as_value().unwrap().is_empty());
    let stats = envelope.stats.as_value().unwrap();
    assert_eq!(stats.created_time.as_value().unwrap().as_u64(), Some(0));
    assert_eq!(stats.modified_time.as_value().unwrap().as_f64(), Some(12.5));
    assert_eq!(stats.last_modified_by, SourcePresence::Null);
    assert_eq!(stats.system_id, SourcePresence::Value("pf2e".into()));
    assert_eq!(
        system
            .level
            .as_value()
            .unwrap()
            .value
            .as_value()
            .unwrap()
            .as_f64(),
        Some(2.5)
    );
    let description = system.description.as_value().unwrap();
    assert_eq!(description.gm, SourcePresence::Value("".into()));
    assert_eq!(
        description.value,
        SourcePresence::Value("<p>Authored text</p>".into())
    );
    let traits = system.traits.as_value().unwrap();
    assert_eq!(traits.shape(), ItemTraitsShape::ValuesAndRarity);
    assert_eq!(
        traits.value,
        SourcePresence::Value(vec!["agile".into(), "agile".into()])
    );
    assert_eq!(traits.rarity, SourcePresence::Value(Rarity::Rare));
    assert_eq!(traits.other_tags, SourcePresence::Value(vec![]));
    assert_eq!(system.slug, SourcePresence::Null);
    let publication = system.publication.as_value().unwrap();
    assert_eq!(publication.title, SourcePresence::Value("Book".into()));
    assert_eq!(publication.authors, SourcePresence::Value("Author".into()));
    assert_eq!(
        publication.license,
        SourcePresence::Value(ItemLicenseSource::Orc)
    );
    assert_eq!(publication.remaster, SourcePresence::Value(false));
    let previous = system
        .migration
        .as_value()
        .unwrap()
        .previous
        .as_value()
        .unwrap();
    assert_eq!(previous.foundry, SourcePresence::Null);
    assert_eq!(previous.schema.as_value().unwrap().as_f64(), Some(0.9));
    assert_eq!(previous.system, SourcePresence::Value("6.11.0".into()));
    assert_eq!(
        system.pending_family_fields.compact_json(),
        r#"{"damage":{"dice":0}}"#
    );
    let flags = envelope.flags.as_value().unwrap();
    assert_eq!(flags.extensions.compact_json(), r#"{"module":{"value":0}}"#);
    let pf2e = flags.pf2e.as_value().unwrap();
    assert_eq!(pf2e.extensions.compact_json(), r#"{"custom":false}"#);
    assert_eq!(
        pf2e.granted_by.as_value().unwrap().on_delete,
        SourcePresence::Value(ItemGrantDeleteAction::Cascade)
    );
    let grant = pf2e.item_grants.as_value().unwrap()[0]
        .1
        .as_value()
        .unwrap();
    assert_eq!(grant.grant.id, "child-id");
    assert_eq!(grant.nested, SourcePresence::Value(false));
    assert_eq!(
        grant.grant.on_delete,
        SourcePresence::Value(ItemGrantDeleteAction::Detach)
    );
    assert!(matches!(
        pf2e.rules_selections.as_value().unwrap()[0].1,
        SourcePresence::Value(ItemRuleSelectionSource::String(_))
    ));
}

#[test]
fn declaration_only_families_are_recognized_without_claiming_system_completion() {
    for item_type in ItemType::ALL {
        let parsed = parse(minimal(
            item_type.as_str(),
            json!({"familyField":{"value":false}}),
        ));
        assert_eq!(parsed.source.envelope.item_type, item_type);
        assert_eq!(parsed.source.system.description, SourcePresence::Missing);
        assert_eq!(
            parsed.source.system.pending_family_fields.compact_json(),
            r#"{"familyField":{"value":false}}"#
        );
    }
}

#[test]
fn standalone_and_actor_occurrences_share_intrinsic_fields_for_family_fixtures() {
    let sources: &[&[u8]] = &[
        include_bytes!(
            "fixtures/foundry-source/families/packs/equipment/wondrous-figurine-rubber-bear.json"
        ),
        include_bytes!("fixtures/foundry-source/spell-source-contract/packs/spells/heal.json"),
    ];
    for bytes in sources {
        let root = parse_bytes(bytes, None);
        assert!(
            root.source
                .system
                .description
                .as_value()
                .unwrap()
                .value
                .as_value()
                .unwrap()
                .contains("<p>")
        );
        match root.source.envelope.item_type {
            ItemType::Equipment => {
                assert_eq!(root.source.system.publication, SourcePresence::Missing)
            }
            ItemType::Spell => assert_eq!(
                root.source.system.publication.as_value().unwrap().license,
                SourcePresence::Value(ItemLicenseSource::Orc)
            ),
            other => panic!("unexpected fixture type {other:?}"),
        }
        for actor_type in ActorType::ALL {
            let parent = ItemParentSource {
                actor_type,
                item_ordinal: 3,
            };
            let embedded = parse_bytes(bytes, Some(parent));
            assert_eq!(embedded.source, root.source);
            assert_eq!(embedded.parent, Some(parent));
        }
    }
}

#[test]
fn all_four_trait_shapes_preserve_forbidden_field_absence() {
    for (traits, expected) in [
        (
            json!({"value":[],"rarity":"common","otherTags":[]}),
            ItemTraitsShape::ValuesAndRarity,
        ),
        (
            json!({"value":[],"otherTags":[]}),
            ItemTraitsShape::ValuesOnly,
        ),
        (
            json!({"rarity":"unique","otherTags":[]}),
            ItemTraitsShape::RarityOnly,
        ),
        (json!({"otherTags":[]}), ItemTraitsShape::OtherTagsOnly),
    ] {
        let parsed = parse(minimal("action", json!({"traits":traits})));
        let actual = parsed.source.system.traits.as_value().unwrap();
        assert_eq!(actual.shape(), expected);
        assert_eq!(actual.value.is_missing(), traits.get("value").is_none());
        assert_eq!(actual.rarity.is_missing(), traits.get("rarity").is_none());
    }
}

#[test]
fn sparse_and_explicit_null_sources_are_not_defaulted() {
    let missing = parse(minimal("effect", json!({})));
    let null = parse(minimal(
        "effect",
        json!({"description":null,"traits":null,"rules":null,"slug":null,"publication":null,"_migration":null,"schema":null,"level":null}),
    ));
    assert_eq!(missing.source.system.description, SourcePresence::Missing);
    assert_eq!(null.source.system.description, SourcePresence::Null);
    assert_eq!(null.source.system.level, SourcePresence::Null);
    assert_eq!(null.source.system.rules, SourcePresence::Null);
    assert_eq!(null.source.system.migration, SourcePresence::Null);
    let new_document = parse(minimal(
        "effect",
        json!({"_migration":{"version":null,"previous":null},"description":{"gm":null,"value":""},"traits":{"value":null,"rarity":null}}),
    ));
    assert_eq!(
        new_document
            .source
            .system
            .migration
            .as_value()
            .unwrap()
            .version,
        SourcePresence::Null
    );
    assert_eq!(
        new_document
            .source
            .system
            .description
            .as_value()
            .unwrap()
            .gm,
        SourcePresence::Null
    );
    assert_eq!(
        new_document
            .source
            .system
            .description
            .as_value()
            .unwrap()
            .value,
        SourcePresence::Value("".into())
    );
}

#[test]
fn source_ids_remain_missing_or_null_without_inventing_child_identity() {
    let parent = Some(ItemParentSource {
        actor_type: ActorType::Npc,
        item_ordinal: 0,
    });
    let missing = parse_bytes(
        br#"{"name":"Unnamed source ID","type":"action","system":{}}"#,
        parent,
    );
    let null = parse_bytes(
        br#"{"_id":null,"name":"Unassigned source ID","type":"action","system":{}}"#,
        parent,
    );
    assert_eq!(missing.source.envelope.id, SourcePresence::Missing);
    assert_eq!(null.source.envelope.id, SourcePresence::Null);
}

#[test]
fn typescript_object_alternatives_include_arrays_without_coercion() {
    let value = json!({"name":"Array choice","type":"effect","flags":{"pf2e":{"rulesSelections":{"choice":[1,false,null]}}},"system":{"schema":[]}});
    let parsed = parse(value);
    assert!(
        matches!(&parsed.source.system.legacy_schema, SourcePresence::Value(SerializedSourceValue::Array(values)) if values.is_empty())
    );
    let selections = parsed
        .source
        .envelope
        .flags
        .as_value()
        .unwrap()
        .pf2e
        .as_value()
        .unwrap()
        .rules_selections
        .as_value()
        .unwrap();
    assert!(
        matches!(&selections[0].1, SourcePresence::Value(ItemRuleSelectionSource::Array(values)) if values.len()==3)
    );
}

#[test]
fn known_duplicate_members_are_diagnosed_without_first_or_last_selection() {
    for (bytes, path) in [
        (
            r#"{"_id":"a","_id":"b","name":"n","type":"action","system":{}}"#,
            "$._id",
        ),
        (
            r#"{"_id":"a","name":"n","type":"action","type":"weapon","system":{}}"#,
            "$.type",
        ),
        (
            r#"{"_id":"a","name":"n","type":"action","system":{},"system":{}}"#,
            "$.system",
        ),
        (
            r#"{"_id":"a","name":"n","type":"action","system":{"description":{"value":"a","value":"b"}}}"#,
            "$.system.description.value",
        ),
    ] {
        let error = parse_common_item_source(
            pinned_source_version_metadata(),
            identity(),
            None,
            bytes.as_bytes(),
        )
        .unwrap_err();
        assert_eq!(error.json_path(), path);
        assert!(error.actual_shape().contains("duplicate members"));
    }
}

#[test]
fn pending_payloads_and_keyed_maps_preserve_order_and_repeated_members() {
    let parsed = parse_bytes(br#"{"_id":"a","name":"n","type":"effect","system":{"rules":[{"key":"Custom","x":1,"x":2},{"key":"Custom"}],"z":0,"a":false,"z":null},"flags":{"module":{"x":1,"x":2},"pf2e":{"itemGrants":{"z":{"id":"one"},"a":{"id":"two"},"z":{"id":"three"}},"rulesSelections":{"z":0,"a":"","z":{"flag":false}}}}}"#, None);
    let system = &parsed.source.system;
    assert_eq!(
        system.pending_family_fields.compact_json(),
        r#"{"z":0,"a":false,"z":null}"#
    );
    assert_eq!(
        system.rules.as_value().unwrap()[0].compact_json(),
        r#"{"key":"Custom","x":1,"x":2}"#
    );
    let pf2e = parsed
        .source
        .envelope
        .flags
        .as_value()
        .unwrap()
        .pf2e
        .as_value()
        .unwrap();
    assert_eq!(
        pf2e.item_grants
            .as_value()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_value().unwrap().grant.id.as_str()))
            .collect::<Vec<_>>(),
        [("z", "one"), ("a", "two"), ("z", "three")]
    );
    let selections = pf2e.rules_selections.as_value().unwrap();
    assert_eq!(
        selections
            .iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<_>>(),
        ["z", "a", "z"]
    );
    assert!(
        matches!(&selections[0].1, SourcePresence::Value(ItemRuleSelectionSource::Number(n)) if n.as_i64()==Some(0))
    );
    assert!(
        matches!(&selections[2].1, SourcePresence::Value(ItemRuleSelectionSource::Object(v)) if matches!(&v.fields()[0].1, SerializedSourceValue::Boolean(false)))
    );
}

#[test]
fn malformed_typed_fields_report_exact_context_and_version() {
    for (system, expected_path) in [
        (
            json!({"level":{"value":false}}),
            "$.items[2].system.level.value",
        ),
        (
            json!({"traits":{"value":["agile",1]}}),
            "$.items[2].system.traits.value[1]",
        ),
        (
            json!({"traits":{"rarity":"invented"}}),
            "$.items[2].system.traits.rarity",
        ),
        (
            json!({"publication":{"license":"invented"}}),
            "$.items[2].system.publication.license",
        ),
        (
            json!({"publication":{"remaster":"false"}}),
            "$.items[2].system.publication.remaster",
        ),
        (json!({"rules":[false]}), "$.items[2].system.rules[0]"),
    ] {
        let bytes = serde_json::to_vec(&minimal("action", system)).unwrap();
        let error = parse_common_item_source(
            pinned_source_version_metadata(),
            identity(),
            Some(ItemParentSource {
                actor_type: ActorType::Hazard,
                item_ordinal: 2,
            }),
            &bytes,
        )
        .unwrap_err();
        assert_eq!(error.kind, SourceDiagnosticKind::MalformedShape);
        assert_eq!(error.json_path(), expected_path);
        assert_eq!(error.record_key(), "fixture:item");
        assert_eq!(error.source_path(), "packs/fixture/item.json");
        assert_eq!(error.source_system_version, "6.12.4");
    }
    let value = json!({"_id":"a","name":"n","type":"effect","system":{},"flags":{"pf2e":{"itemGrants":{"a.b":{"id":5}}}}});
    let error = parse_common_item_source(
        pinned_source_version_metadata(),
        identity(),
        None,
        &serde_json::to_vec(&value).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.json_path(), "$.flags.pf2e.itemGrants[\"a.b\"].id");
}

#[test]
fn malformed_bytes_and_unknown_discriminators_fail_at_the_source_boundary() {
    for bytes in [b"[]".as_slice(), b"{} {}", b"{", b"null"] {
        let error =
            parse_common_item_source(pinned_source_version_metadata(), identity(), None, bytes)
                .unwrap_err();
        assert_eq!(error.kind, SourceDiagnosticKind::MalformedShape);
        assert_eq!(error.json_path(), "$");
    }
    let bytes = serde_json::to_vec(&minimal("invented", json!({}))).unwrap();
    let error =
        parse_common_item_source(pinned_source_version_metadata(), identity(), None, &bytes)
            .unwrap_err();
    assert_eq!(error.kind, SourceDiagnosticKind::UnknownDiscriminator);
    assert_eq!(error.json_path(), "$.type");
}
