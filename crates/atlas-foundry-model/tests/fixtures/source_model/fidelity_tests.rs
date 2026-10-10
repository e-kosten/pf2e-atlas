use super::inspection;
use super::rule_fidelity::FidelityGraph;
use crate::source_model::{generated, parse::SourceContext, value::parse_source};
use serde_json::json;

#[test]
fn generated_authored_inputs_keep_scalar_array_presence_and_strict_array_boundaries() {
    let context = SourceContext::new("fixture", "fixture.json", "$");
    for (source, selector, selectors) in [
        (
            r#"{"selector":"attack","selectors":["strict"]}"#,
            json!({"value":"attack"}),
            json!({"value":["strict"]}),
        ),
        (
            r#"{"selector":["attack","damage"]}"#,
            json!({"value":["attack","damage"]}),
            json!("missing"),
        ),
        (r#"{"selector":[]}"#, json!({"value":[]}), json!("missing")),
        (r#"{"selector":null}"#, json!("null"), json!("missing")),
        (r#"{}"#, json!("missing"), json!("missing")),
    ] {
        let raw = parse_source(source.as_bytes()).unwrap();
        let model =
            inspection::to_value(generated::parse_authored_arrays(&raw, &context, "$").unwrap())
                .unwrap();
        assert_eq!(model["selector"], selector);
        assert_eq!(model["selectors"], selectors);
    }
    for source in [
        r#"{"selectors":"strict"}"#,
        r#"{"selector":5}"#,
        r#"{"selector":["attack",5]}"#,
        r#"{"selector":"a","selector":"b"}"#,
    ] {
        let raw = parse_source(source.as_bytes()).unwrap();
        assert!(generated::parse_authored_arrays(&raw, &context, "$").is_err());
    }
}

#[test]
fn generated_nested_iwr_keeps_authored_forms_and_strict_exceptions() {
    let context = SourceContext::new("fixture", "fixture.json", "$");
    for (source, expected) in [
        (
            r#"{"immunities":[{"type":"fire"}]}"#,
            json!({"value":"fire"}),
        ),
        (
            r#"{"immunities":[{"type":["fire","cold"]}]}"#,
            json!({"value":["fire","cold"]}),
        ),
        (r#"{"immunities":[{"type":[]}]}"#, json!({"value":[]})),
        (r#"{"immunities":[{"type":null}]}"#, json!("null")),
        (r#"{"immunities":[{}]}"#, json!("missing")),
    ] {
        let raw = parse_source(source.as_bytes()).unwrap();
        let model =
            inspection::to_value(generated::parse_authored_form(&raw, &context, "$").unwrap())
                .unwrap();
        assert_eq!(model["immunities"]["value"][0]["type"], expected);
    }
    for source in [
        r#"{"immunities":[{"type":3}]}"#,
        r#"{"immunities":[{"type":["fire",3]}]}"#,
        r#"{"immunities":[{"type":"fire","exceptions":"cold"}]}"#,
    ] {
        let raw = parse_source(source.as_bytes()).unwrap();
        assert!(generated::parse_authored_form(&raw, &context, "$").is_err());
    }
}

#[test]
fn admission_fidelity_detects_loss_in_retained_invalid_fields() {
    let input = json!({"nodes":[
        {"id":"number","kind":"primitive","value":"number"},
        {"id":"record","kind":"object","fields":[{"name":"level","ref":"number","forbidden":false}],"indexSignatures":[]}
    ]});
    let raw = parse_source(br#"{"level":"1","level":"2"}"#).unwrap();
    let model = json!({"level":{"invalid":{"json_path":"$.items[4].level",
            "values":[{"String":"1"},{"String":"2"}],"diagnostic":{"expected":"one structural member"}}},
            "additional_fields":{"fields":[]}});
    let graph = FidelityGraph::for_admission(&input);
    assert!(
        graph
            .compare_at("record", &raw, &model, "$.items[4]")
            .is_ok()
    );
    assert!(
        FidelityGraph::new(&input)
            .compare("record", &raw, &model)
            .is_err()
    );
    for (key, value) in [
        ("values", json!([{"String":"2"},{"String":"1"}])),
        ("values", json!([{"String":"1"}])),
        ("values", json!([{"Number":1},{"Number":2}])),
        ("json_path", json!("$.items[5].level")),
        ("diagnostic", json!({})),
    ] {
        let mut changed = model.clone();
        changed["level"]["invalid"][key] = value;
        assert!(
            graph
                .compare_at("record", &raw, &changed, "$.items[4]")
                .is_err(),
            "{key}"
        );
    }
}

#[test]
fn fidelity_detects_omission_presence_coercion_number_loss_and_additional_order() {
    let graph = FidelityGraph::new(&json!({"nodes":[
        {"id":"string","kind":"primitive","value":"string"},
        {"id":"number","kind":"primitive","value":"number"},
        {"id":"strings","kind":"array","element":"string"},
        {"id":"selector","kind":"union","members":["string","strings"]},
        {"id":"rule","kind":"object","fields":[
            {"name":"selector","ref":"selector","forbidden":false},
            {"name":"diceNumber","ref":"number","forbidden":false},
            {"name":"nullable","ref":"string","forbidden":false},
            {"name":"absent","ref":"string","forbidden":false}],"indexSignatures":[]}
    ]}));
    let raw = parse_source(br#"{"selector":"attack","diceNumber":9007199254740993,"nullable":null,"future":1,"future":2}"#).unwrap();
    let expected = json!({"selector":{"value":"attack"},"dice_number":{"value":9007199254740993u64},
            "nullable":"null","absent":"missing","additional_fields":{"fields":[["future",{"Number":1}],["future",{"Number":2}]]}});
    assert!(graph.compare("rule", &raw, &expected).is_ok());
    let mut lost = expected.clone();
    lost.as_object_mut().unwrap().remove("selector");
    assert!(graph.compare("rule", &raw, &lost).is_err());
    for (key, replacement) in [
        ("selector", json!({"value":["attack"]})),
        ("dice_number", json!({"value":9007199254740992u64})),
        ("nullable", json!("missing")),
        ("absent", json!("null")),
        (
            "additional_fields",
            json!({"fields":[["future",{"Number":2}],["future",{"Number":1}]]}),
        ),
        (
            "additional_fields",
            json!({"fields":[["future",{"Number":2}]]}),
        ),
    ] {
        let mut changed = expected.clone();
        changed[key] = replacement;
        assert!(graph.compare("rule", &raw, &changed).is_err(), "{key}");
    }
}

#[test]
fn fidelity_checks_actual_generated_values_and_source_key_mapping() {
    let graph = FidelityGraph::new(&json!({"nodes":[
        {"id":"string","kind":"primitive","value":"string"},
        {"id":"number","kind":"primitive","value":"number"},
        {"id":"mapped","kind":"object","fields":[
            {"name":"_id","ref":"string","forbidden":false},
            {"name":"greater-darkvision","ref":"number","forbidden":false},
            {"name":"self","ref":"string","forbidden":false},
            {"name":"1st","ref":"string","forbidden":false}],"indexSignatures":[]}
    ]}));
    let raw = parse_source(
        br#"{"_id":"x","greater-darkvision":60,"self":"a","1st":null,"extra":{"a":1,"a":2}}"#,
    )
    .unwrap();
    let context = SourceContext::new("fixture", "fixture.json", "$");
    let parsed = generated::parse_mapped_fields(&raw, &context, "$").unwrap();
    let actual = inspection::to_value(parsed).unwrap();
    assert!(graph.compare("mapped", &raw, &actual).is_ok());
}

#[test]
fn fidelity_preserves_open_values_and_typed_map_order() {
    let graph = FidelityGraph::new(&json!({"nodes":[
        {"id":"open","kind":"open","domain":"unknown"},
        {"id":"map","kind":"object","fields":[],"indexSignatures":[{"key":"primitive:string","value":"open"}]}
    ]}));
    let raw = parse_source(br#"{"z":null,"a":{"x":1,"x":2}}"#).unwrap();
    let context = SourceContext::new("fixture", "fixture.json", "$");
    let actual =
        inspection::to_value(generated::parse_unknown_map(&raw, &context, "$").unwrap()).unwrap();
    assert!(graph.compare("map", &raw, &actual).is_ok());
    let mut reordered = actual;
    reordered["entries"].as_array_mut().unwrap().reverse();
    assert!(graph.compare("map", &raw, &reordered).is_err());
}

#[test]
fn fidelity_reports_tuple_shape_loss_without_panicking() {
    let graph = FidelityGraph::new(&json!({"nodes":[
        {"id":"string","kind":"primitive","value":"string"},
        {"id":"tuple","kind":"tuple","elements":[{"ref":"string"}]}
    ]}));
    let raw = parse_source(br#"["one","extra"]"#).unwrap();
    assert!(
        graph
            .compare("tuple", &raw, &json!(["one", "extra"]))
            .is_err()
    );
}
