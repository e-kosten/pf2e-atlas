#[path = "../examples/support/model_inspection.rs"]
mod inspection;

use atlas_foundry_model::{ItemTraits, SourceContext, SourcePresence, parse_item_source_slice};

fn parse(text: &str) -> atlas_foundry_model::ItemSourceSlice {
    parse_item_source_slice(
        SourceContext::new("pack:id", "fixture.json", "$"),
        text.as_bytes(),
    )
    .unwrap()
}

#[test]
fn every_registered_family_has_a_callable_common_slice() {
    for family in [
        "action",
        "affliction",
        "ancestry",
        "armor",
        "background",
        "book",
        "campaignFeature",
        "class",
        "condition",
        "consumable",
        "backpack",
        "deity",
        "effect",
        "equipment",
        "feat",
        "heritage",
        "kit",
        "lore",
        "melee",
        "shield",
        "spell",
        "spellcastingEntry",
        "treasure",
        "weapon",
    ] {
        let value = parse(&format!(
            r#"{{"type":"{family}","system":{{"traits":{{"otherTags":[]}}}}}}"#
        ));
        assert_eq!(value.family, family);
        assert!(matches!(value.description, SourcePresence::Missing));
        assert!(matches!(value.publication, SourcePresence::Missing));
        assert!(matches!(value.traits, SourcePresence::Value(_)));
    }
    let error = parse_item_source_slice(
        SourceContext::new("x", "x", "$"),
        br#"{"type":"newFamily","system":{}}"#,
    )
    .unwrap_err();
    assert_eq!(error.json_path, "$.type");
}

#[test]
fn shared_values_preserve_presence_and_open_traits_without_widening_closed_sets() {
    let value = parse(
        r#"{"type":"equipment","system":{"description":{"value":"@UUID[x]","gm":null,"future":1},"publication":{"title":"Book","license":"ORC","remaster":true},"traits":{"value":["brand-new-trait","lawful"],"rarity":"rare","otherTags":[],"toggles":{"temporary":true}}}}"#,
    );
    let SourcePresence::Value(description) = value.description else {
        panic!()
    };
    assert_eq!(description.value, SourcePresence::Value("@UUID[x]".into()));
    assert_eq!(description.gm, SourcePresence::Null);
    assert_eq!(description.additional_fields.fields()[0].0, "future");
    let SourcePresence::Value(publication) = value.publication else {
        panic!()
    };
    assert_eq!(publication.authors, SourcePresence::Missing);
    let SourcePresence::Value(ItemTraits::Equipment(traits)) = value.traits else {
        panic!()
    };
    assert_eq!(
        traits.value,
        SourcePresence::Value(vec!["brand-new-trait".into(), "lawful".into()])
    );
    assert_eq!(traits.additional_fields.fields()[0].0, "toggles");
    let nulls = parse(
        r#"{"type":"equipment","system":{"description":null,"publication":null,"traits":null},"flags":null}"#,
    );
    assert!(matches!(nulls.traits, SourcePresence::Null));
    assert!(matches!(nulls.flags, SourcePresence::Null));
    let empty = parse(
        r#"{"type":"equipment","system":{"description":{},"publication":{},"traits":{"value":[]}}}"#,
    );
    assert!(matches!(empty.description, SourcePresence::Value(_)));
}

#[test]
fn declaration_forbidden_and_unselected_members_remain_ordered_source_data() {
    let value = parse(
        r#"{"type":"class","system":{"traits":{"value":[],"value":null,"rarity":"common","otherTags":[]},"future":9007199254740993,"future":0.5},"unknown":{"a":1}}"#,
    );
    let SourcePresence::Value(ItemTraits::Class(traits)) = value.traits else {
        panic!()
    };
    assert_eq!(
        traits
            .additional_fields
            .fields()
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        vec!["value", "value"]
    );
    assert_eq!(value.system_fields.fields().len(), 2);
    assert_eq!(value.envelope_fields.fields()[0].0, "unknown");
    let deity =
        parse(r#"{"type":"deity","system":{"traits":{"value":[],"rarity":null,"otherTags":[]}}}"#);
    let SourcePresence::Value(ItemTraits::Deity(traits)) = deity.traits else {
        panic!()
    };
    assert_eq!(traits.additional_fields.fields().len(), 2);
}

#[test]
fn keyed_grants_are_typed_ordered_and_keep_unknown_fields() {
    let value = parse(
        r#"{"type":"feat","system":{},"flags":{"pf2e":{"itemGrants":{"z":{"id":"second","onDelete":"detach","nested":null,"future":9007199254740993},"a":{"id":"first"}},"other":1},"module":{}}}"#,
    );
    let SourcePresence::Value(flags) = value.flags else {
        panic!()
    };
    let SourcePresence::Value(pf2e) = flags.pf2e else {
        panic!()
    };
    let SourcePresence::Value(grants) = pf2e.item_grants else {
        panic!()
    };
    assert_eq!(
        grants
            .entries
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        vec!["z", "a"]
    );
    assert_eq!(
        grants.entries[0].1.id,
        SourcePresence::Value("second".into())
    );
    assert_eq!(grants.entries[0].1.nested, SourcePresence::Null);
    assert_eq!(grants.entries[1].1.nested, SourcePresence::Missing);
    assert_eq!(
        grants.entries[0].1.additional_fields.fields()[0].0,
        "future"
    );
    assert_eq!(pf2e.indexed_fields.entries[0].0, "other");
    assert_eq!(flags.indexed_fields.entries[0].0, "module");
    assert!(pf2e.additional_fields.fields().is_empty());
    assert!(flags.additional_fields.fields().is_empty());
}

#[test]
fn absent_corpus_families_have_nonempty_selected_value_fixtures() {
    let book = parse(
        r#"{"type":"book","system":{"description":{"gm":"private","value":"text"},"publication":{"title":"Source","authors":"Author","license":"OGL","remaster":false},"traits":{"value":["new-book-trait"],"rarity":"unique","otherTags":["custom"]}}}"#,
    );
    let SourcePresence::Value(ItemTraits::Book(traits)) = book.traits else {
        panic!()
    };
    assert_eq!(
        traits.value,
        SourcePresence::Value(vec!["new-book-trait".into()])
    );
    assert!(matches!(traits.rarity, SourcePresence::Value(_)));
    let SourcePresence::Value(description) = book.description else {
        panic!()
    };
    assert_eq!(description.gm, SourcePresence::Value("private".into()));
    let SourcePresence::Value(publication) = book.publication else {
        panic!()
    };
    assert_eq!(publication.authors, SourcePresence::Value("Author".into()));
    let affliction = parse(
        r#"{"type":"affliction","system":{"description":{"value":"Disease"},"publication":{"title":"Source","license":"ORC"},"traits":{"value":["new-affliction-trait"],"rarity":"rare","otherTags":["custom"]}}}"#,
    );
    let SourcePresence::Value(ItemTraits::Affliction(traits)) = affliction.traits else {
        panic!()
    };
    assert_eq!(
        traits.value,
        SourcePresence::Value(vec!["new-affliction-trait".into()])
    );
    assert_eq!(traits.additional_fields.fields()[0].0, "rarity");
    assert!(matches!(affliction.description, SourcePresence::Value(_)));
    assert!(matches!(affliction.publication, SourcePresence::Value(_)));
}

#[test]
fn malformed_modeled_values_fail_with_nested_context() {
    for (text, path) in [
        (
            r#"{"type":"equipment","system":{"traits":{"value":["ok",2]}}}"#,
            "$.system.traits.value[1]",
        ),
        (
            r#"{"type":"equipment","system":{"traits":{"rarity":"new"}}}"#,
            "$.system.traits.rarity",
        ),
        (
            r#"{"type":"equipment","system":{"publication":{"license":"new"}}}"#,
            "$.system.publication.license",
        ),
        (
            r#"{"type":"feat","system":{},"flags":{"pf2e":{"itemGrants":{"x":{"onDelete":"new"}}}}}"#,
            "$.flags.pf2e.itemGrants[\"x\"].onDelete",
        ),
        (
            r#"{"type":"feat","system":{},"flags":{"pf2e":{"itemGrants":{"x":{},"x":{}}}}}"#,
            "$.flags.pf2e.itemGrants[\"x\"]",
        ),
        (
            r#"{"type":"feat","system":{},"flags":{"pf2e":{"itemGrants":{"x":null}}}}"#,
            "$.flags.pf2e.itemGrants[\"x\"]",
        ),
        (
            r#"{"type":"treasure","system":{"traits":{"value":["unexpected"]}}}"#,
            "$.system.traits.value[0]",
        ),
        (
            r#"{"type":"equipment","system":{"description":{"value":"x","value":"y"}}}"#,
            "$.system.description.value",
        ),
    ] {
        let error = parse_item_source_slice(
            SourceContext::new("pack:id", "fixture.json", "$"),
            text.as_bytes(),
        )
        .unwrap_err();
        assert_eq!(error.json_path, path, "{text}");
        assert_eq!(error.context.record_key, "pack:id");
    }
}

#[test]
fn complete_flags_model_grants_selections_and_open_namespaces() {
    let item = parse(
        r#"{"type":"feat","system":{},"flags":{"pf2e":{"grantedBy":{"id":"parent","onDelete":"cascade"},
        "rulesSelections":{"name":"choice","rank":18446744073709551615,"object":{"x":1,"x":2},"array":[null,3]},
        "future":{"x":1,"x":2}},"module":{"enabled":true,"payload":null,"values":[1,2]}}}"#,
    );
    let SourcePresence::Value(flags) = item.flags else {
        panic!()
    };
    let SourcePresence::Value(pf2e) = flags.pf2e else {
        panic!()
    };
    let SourcePresence::Value(granted) = pf2e.granted_by else {
        panic!()
    };
    assert_eq!(granted.id, SourcePresence::Value("parent".into()));
    let SourcePresence::Value(selections) = pf2e.rules_selections else {
        panic!()
    };
    assert_eq!(
        selections
            .entries
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        vec!["name", "rank", "object", "array"]
    );
    let serialized = inspection::to_value(selections).unwrap();
    assert_eq!(
        serialized["entries"][1][1],
        serde_json::json!(18446744073709551615u64)
    );
    assert_eq!(
        serialized["entries"][2][1]["Object"]["fields"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let atlas_foundry_model::SourceValue::Object(future) = &pf2e.indexed_fields.entries[0].1 else {
        panic!()
    };
    assert_eq!(future.fields().len(), 2);
    let module = &flags.indexed_fields.entries[0].1;
    assert_eq!(
        module
            .entries
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        vec!["enabled", "payload", "values"]
    );
    assert!(matches!(
        module.entries[1].1,
        atlas_foundry_model::SourceValue::Null
    ));
}

#[test]
fn new_flags_fields_reject_bad_declared_values_without_widening_open_data() {
    for (flags, path) in [
        (
            r#"{"pf2e":{"rulesSelections":{"x":false}}}"#,
            r#"$.flags.pf2e.rulesSelections["x"]"#,
        ),
        (
            r#"{"pf2e":{"rulesSelections":{"x":null}}}"#,
            r#"$.flags.pf2e.rulesSelections["x"]"#,
        ),
        (
            r#"{"pf2e":{"grantedBy":{"onDelete":"unknown"}}}"#,
            "$.flags.pf2e.grantedBy.onDelete",
        ),
        (r#"{"module":false}"#, r#"$.flags["module"]"#),
        (
            r#"{"pf2e":{"future":1,"future":2}}"#,
            r#"$.flags.pf2e["future"]"#,
        ),
    ] {
        let source = format!(r#"{{"type":"feat","system":{{}},"flags":{flags}}}"#);
        let error = parse_item_source_slice(
            SourceContext::new("pack:id", "fixture.json", "$"),
            source.as_bytes(),
        )
        .unwrap_err();
        assert_eq!(error.json_path, path);
    }
}
