#[path = "../examples/support/model_inspection.rs"]
mod inspection;

use atlas_foundry_model::{
    ItemSourcePF2e, SourceAdmission, SourceContext, SourcePresence, SourceValue,
    admit_actor_source_pf2e, admit_item_source_pf2e, admit_journal_entry_source,
    admit_macro_source, admit_roll_table_source, admit_rule_source, parse_actor_source_pf2e,
    parse_item_source_pf2e,
};
use serde::Serialize;
use serde_json::{Value, json};

fn context() -> SourceContext {
    SourceContext::new("fixture:id", "fixture.json", "$")
}
fn model<T: Serialize>(admitted: &SourceAdmission<T>) -> Value {
    inspection::to_value(admitted.model.as_ref().unwrap()).unwrap()
}

#[test]
fn valid_sources_have_identical_models_and_preserve_large_numbers_and_repeated_unknowns() {
    let bytes = br#"{"type":"action","name":"Leap","system":{"future":9007199254740993,"future":null},"extra":1,"extra":2}"#;
    let strict = parse_item_source_pf2e(context(), bytes).unwrap();
    let admitted = admit_item_source_pf2e(context(), bytes).unwrap();
    assert_eq!(admitted.model, Some(strict));
    assert!(admitted.diagnostics.is_empty());
    assert_eq!(
        admitted.raw,
        atlas_foundry_model::parse_source_value(bytes).unwrap()
    );
    for admitted in [
        inspection::to_value(admit_journal_entry_source(context(), br#"{"pages":[]}"#).unwrap())
            .unwrap(),
        inspection::to_value(admit_macro_source(context(), br#"{"type":"script"}"#).unwrap())
            .unwrap(),
        inspection::to_value(admit_roll_table_source(context(), br#"{"results":[]}"#).unwrap())
            .unwrap(),
    ] {
        assert!(admitted["model"].is_object());
        assert_eq!(admitted["diagnostics"], json!([]));
    }
}

#[test]
fn invalid_numeric_fields_are_neither_missing_nor_coerced() {
    let bytes = br#"{"type":"ancestry","name":"Catfolk","system":{"items":{"feature":{"uuid":"Compendium.pf2e.ancestryfeatures.Item.x","level":"1"}}}}"#;
    assert!(parse_item_source_pf2e(context(), bytes).is_err());
    let admitted = admit_item_source_pf2e(context(), bytes).unwrap();
    let ItemSourcePF2e::AncestrySource(ancestry) = admitted.model.as_ref().unwrap() else {
        panic!()
    };
    let system = ancestry.system.as_value().unwrap();
    let feature = &system.items.as_value().unwrap().entries[0].1;
    let SourcePresence::Invalid(rejected) = &feature.level else {
        panic!()
    };
    assert!(feature.level.as_value().is_none());
    assert_eq!(rejected.values, vec![SourceValue::String("1".into())]);
    assert_eq!(rejected.json_path, "$.system.items[\"feature\"].level");
    assert_eq!(admitted.diagnostics.len(), 1);
    assert_eq!(
        feature.uuid.as_value().unwrap(),
        "Compendium.pf2e.ancestryfeatures.Item.x"
    );
    assert_eq!(ancestry.name.as_value().unwrap(), "Catfolk");
}

#[test]
fn unsupported_vocabularies_preserve_the_parent_and_raw_collection() {
    let bytes = br#"{"type":"deity","name":"Desna","system":{"domains":{"primary":["luck"],"alternate":["nothingness","travel"]}}}"#;
    let admitted = admit_item_source_pf2e(context(), bytes).unwrap();
    let model = model(&admitted);
    assert_eq!(model["name"]["value"], "Desna");
    let domains = &model["system"]["value"]["domains"]["value"];
    assert!(domains["primary"].get("value").is_some());
    assert_eq!(
        domains["alternate"]["invalid"]["values"],
        json!([{"Array":[{"String":"nothingness"},{"String":"travel"}]}])
    );
    assert_eq!(
        admitted.diagnostics[0].json_path,
        "$.system.domains.alternate[0]"
    );
}

#[test]
fn embedded_spellcasting_entries_and_spell_references_survive_multiple_invalid_fields() {
    let bytes = br#"{"type":"npc","_id":"actor","name":"Caster","system":{"resources":{"focus":{"max":"2","value":1}}},"items":[{"type":"spellcastingEntry","_id":"entry","name":"Innate","system":{"slots":{"slot1":{"max":"4","value":0,"prepared":["darkness (at will)",{"id":"spell"}]}}}},{"type":"spell","_id":"spell","name":"Darkness","system":{"location":{"value":"entry"},"heightening":{"damage":{}}}},{"type":"action","_id":"action","name":"Strike"}]}"#;
    let admitted = admit_actor_source_pf2e(context(), bytes).unwrap();
    let model = model(&admitted);
    let items = model["items"]["value"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["_id"]["value"], "entry");
    assert_eq!(
        items[1]["system"]["value"]["location"]["value"]["value"]["value"],
        "entry"
    );
    assert_eq!(items[2]["name"]["value"], "Strike");
    let slot = &items[0]["system"]["value"]["slots"]["value"]["slot1"]["value"];
    assert!(slot["max"].get("invalid").is_some());
    assert_eq!(slot["value"]["value"], 0);
    assert_eq!(
        slot["prepared"]["invalid"]["values"][0]["Array"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(admitted.diagnostics.len(), 4);
    assert_eq!(
        admitted.raw,
        atlas_foundry_model::parse_source_value(bytes).unwrap()
    );
}

#[test]
fn a_bad_predicate_excludes_the_entire_specific_rule_without_changing_the_parent() {
    let bytes = br#"{"key":"ChoiceSet","choices":[],"predicate":[{"nor":["a"],"not":"b"}]}"#;
    let admitted = admit_rule_source(context(), bytes).unwrap();
    assert!(admitted.model.is_none());
    assert_eq!(admitted.diagnostics.len(), 1);
    assert_eq!(
        admitted.raw,
        atlas_foundry_model::parse_source_value(bytes).unwrap()
    );
    let parent = admit_item_source_pf2e(context(), br#"{"type":"feat","name":"Choice","system":{"rules":[{"key":"ChoiceSet","choices":[],"predicate":[{"nor":["a"],"not":"b"}]}]}}"#).unwrap();
    assert_eq!(model(&parent)["name"]["value"], "Choice");
    assert!(parent.model.is_some());
}

#[test]
fn boolean_strings_stay_invalid_and_union_identity_is_never_invented() {
    let bytes = br#"{"type":"spell","system":{"heightening":{"damage":{}},"damage":{"0":{"applyMod":"0"}}}}"#;
    let admitted = admit_item_source_pf2e(context(), bytes).unwrap();
    let model = model(&admitted);
    let system = &model["system"]["value"];
    assert!(system["heightening"].get("invalid").is_some());
    assert_eq!(
        system["damage"]["value"]["entries"][0][1]["apply_mod"]["invalid"]["values"],
        json!([{"String":"0"}])
    );
    assert_eq!(admitted.diagnostics.len(), 2);
}

#[test]
fn duplicate_structural_members_are_retained_without_selecting_a_winner() {
    let bytes = br#"{"type":"action","name":"one","name":"two","system":{}}"#;
    let admitted = admit_item_source_pf2e(context(), bytes).unwrap();
    assert_eq!(
        model(&admitted)["name"]["invalid"]["values"],
        json!([{"String":"one"},{"String":"two"}])
    );
    assert_eq!(admitted.diagnostics.len(), 1);
    let duplicate_type =
        admit_item_source_pf2e(context(), br#"{"type":"action","type":"spell"}"#).unwrap();
    assert!(duplicate_type.model.is_none());
}

#[test]
fn unknown_roots_stay_raw_and_invalid_json_or_non_document_envelopes_are_errors() {
    for bytes in [br#"{"type":"future"}"#.as_slice(), br#"{}"#] {
        let admitted = admit_item_source_pf2e(context(), bytes).unwrap();
        assert!(admitted.model.is_none());
        assert_eq!(admitted.diagnostics.len(), 1);
        assert_eq!(
            admitted.raw,
            atlas_foundry_model::parse_source_value(bytes).unwrap()
        );
    }
    for bytes in [b"{".as_slice(), b"[]", b"null", b"{} true"] {
        assert!(admit_item_source_pf2e(context(), bytes).is_err());
    }
}

#[test]
fn diagnostics_preserve_the_supplied_embedded_source_context() {
    let admitted = admit_item_source_pf2e(
        SourceContext::new("pack:actor", "pack/actor.json", "$.items[4]"),
        br#"{"type":"spell","system":{"level":"3"}}"#,
    )
    .unwrap();
    assert_eq!(admitted.diagnostics[0].json_path, "$.items[4].system.level");
    assert_eq!(
        *admitted.diagnostics[0].context,
        SourceContext::new("pack:actor", "pack/actor.json", "$.items[4]")
    );
}

#[test]
fn npc_senses_use_constructor_inputs_without_applying_foundry_defaults() {
    let bytes = br#"{"type":"npc","system":{"perception":{"senses":[{"type":"lifesense"},{"type":"darkvision","range":null}]}}}"#;
    let strict = parse_actor_source_pf2e(context(), bytes).unwrap();
    let admitted = admit_actor_source_pf2e(context(), bytes).unwrap();
    assert_eq!(admitted.model, Some(strict));
    assert!(admitted.diagnostics.is_empty());
    let model = model(&admitted);
    let senses = &model["system"]["value"]["perception"]["value"]["senses"]["value"];
    assert_eq!(senses[0]["acuity"], "missing");
    assert_eq!(senses[0]["range"], "missing");
    assert_eq!(senses[1]["range"], "null");
    assert!(
        parse_actor_source_pf2e(
            context(),
            br#"{"type":"npc","system":{"perception":{"senses":[{"type":"future-sense"}]}}}"#
        )
        .is_err()
    );
}
