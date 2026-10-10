#[path = "../examples/support/model_inspection.rs"]
mod inspection;

use atlas_foundry_model::{
    ActorSourcePF2e, ItemSourcePF2e, RuleSource, SourceContext, SourcePresence,
    parse_actor_source_pf2e, parse_item_source_pf2e, parse_journal_entry_source,
    parse_macro_source, parse_roll_table_source, parse_rule_source,
};

fn context() -> SourceContext {
    SourceContext::new("fixture:id", "fixture.json", "$")
}

#[test]
fn all_registered_actor_and_item_families_have_callable_full_models() {
    for family in [
        "army",
        "character",
        "familiar",
        "hazard",
        "loot",
        "npc",
        "party",
        "vehicle",
    ] {
        parse_actor_source_pf2e(context(), format!(r#"{{"type":"{family}"}}"#).as_bytes()).unwrap();
    }
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
        parse_item_source_pf2e(context(), format!(r#"{{"type":"{family}"}}"#).as_bytes()).unwrap();
    }
    for source in [br#"{}"#.as_slice(), br#"{"type":"new-family"}"#] {
        assert!(parse_item_source_pf2e(context(), source).is_err());
        assert!(parse_actor_source_pf2e(context(), source).is_err());
    }
}

#[test]
fn every_pinned_rule_key_dispatches_to_its_specific_model() {
    let pin: serde_json::Value = serde_json::from_str(include_str!(
        "../../../dev-tools/source-contracts/source-pin.json"
    ))
    .unwrap();
    let keys: Vec<_> = pin["ruleKeys"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    assert_eq!(keys.len(), 42);
    for key in keys {
        let value =
            parse_rule_source(context(), format!(r#"{{"key":"{key}"}}"#).as_bytes()).unwrap();
        let serialized = inspection::to_value(value).unwrap();
        assert_eq!(serialized["key"]["value"], key);
    }
    let error = parse_rule_source(context(), br#"{"key":"FutureRule"}"#).unwrap_err();
    assert_eq!(error.json_path, "$.key");
    assert!(parse_rule_source(context(), br#"{}"#).is_err());
}

#[test]
fn full_sources_preserve_unknown_data_and_parse_specific_rules_separately() {
    let value = parse_item_source_pf2e(context(), br#"{"type":"action","name":null,"system":{"rules":[{"key":"FlatModifier","selector":"attack","value":2}],"future":9007199254740993,"future":null},"extra":1,"extra":2}"#).unwrap();
    let ItemSourcePF2e::AbilitySource(item) = value else {
        panic!()
    };
    assert_eq!(item.name, SourcePresence::Null);
    assert_eq!(item.additional_fields.fields().len(), 2);
    let SourcePresence::Value(system) = item.system else {
        panic!()
    };
    assert_eq!(system.additional_fields.fields().len(), 2);
    // Item.rules deliberately follows upstream's generic RuleElementSource.
    // Specific rule dispatch is a separate callable parser, not a silent drop.
    let rule = parse_rule_source(
        context(),
        br#"{"key":"FlatModifier","selector":"attack","value":2,"future":true}"#,
    )
    .unwrap();
    assert!(matches!(rule, RuleSource::FlatModifier(_)));
    assert!(parse_rule_source(context(), br#"{"key":"FlatModifier","selector":[3]}"#).is_err());
}

#[test]
fn authored_patches_and_empty_sentinels_work_without_repairs() {
    let value = parse_item_source_pf2e(context(), br#"{"type":"spell","system":{"area":{"value":"20","type":""},"overlays":{"override":{"overlayType":"override","system":{"heightening":{"damage":{}}}}}}}"#).unwrap();
    let serialized = inspection::to_value(value).unwrap();
    assert_eq!(
        serialized["system"]["value"]["area"]["value"]["value"]["value"],
        "20"
    );
    assert!(
        parse_item_source_pf2e(
            context(),
            br#"{"type":"spell","system":{"heightening":{"damage":{}}}}"#
        )
        .is_err()
    );
    assert!(
        parse_item_source_pf2e(
            context(),
            br#"{"type":"spell","system":{"area":{"value":true}}}"#
        )
        .is_err()
    );
    assert!(
        parse_item_source_pf2e(
            context(),
            br#"{"type":"weapon","system":{"damage":{"die":""},"reload":{"value":""}}}"#
        )
        .is_ok()
    );
}

#[test]
fn other_document_kinds_preserve_presence_and_enforce_supplied_values() {
    let journal =
        parse_journal_entry_source(context(), br#"{"name":"Notes","pages":[],"folder":null}"#)
            .unwrap();
    assert_eq!(journal.name, SourcePresence::Value("Notes".into()));
    assert_eq!(journal.folder, SourcePresence::Null);
    let script = parse_macro_source(
        context(),
        br#"{"type":"script","command":"console.log(1)","scope":"global"}"#,
    )
    .unwrap();
    assert_eq!(
        script.command,
        SourcePresence::Value("console.log(1)".into())
    );
    let table = parse_roll_table_source(
        context(),
        br#"{"name":"Results","formula":"1d6","results":[]}"#,
    )
    .unwrap();
    assert_eq!(table.formula, SourcePresence::Value("1d6".into()));
    assert!(parse_macro_source(context(), br#"{"type":"unsupported"}"#).is_err());
    assert!(
        parse_journal_entry_source(
            context(),
            br#"{"pages":[{"type":"text","text":{"content":3}}]}"#
        )
        .is_err()
    );
    assert!(parse_roll_table_source(context(), br#"{"replacement":"yes"}"#).is_err());
}

#[test]
fn top_level_portfolio_enums_do_not_inline_entire_documents() {
    assert!(std::mem::size_of::<ItemSourcePF2e>() <= 32);
    assert!(std::mem::size_of::<ActorSourcePF2e>() <= 32);
    assert!(std::mem::size_of::<RuleSource>() <= 32);
}
