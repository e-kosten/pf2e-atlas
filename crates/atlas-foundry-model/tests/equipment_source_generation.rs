use atlas_foundry_model::{
    EquippedDataCarryType, SourceContext, SourcePresence, SourceValue,
    parse_equipment_source_slice, parse_physical_equipment_fields,
};

fn context() -> SourceContext {
    SourceContext::new("equipment:test", "packs/equipment/test.json", "$.items[3]")
}
fn parse(system: &str) -> atlas_foundry_model::EquipmentSourceSlice {
    parse_equipment_source_slice(
        context(),
        format!(r#"{{"type":"equipment","name":"Test","system":{system}}}"#).as_bytes(),
    )
    .unwrap()
}

#[test]
fn generated_fields_preserve_actual_typed_values_and_share_the_physical_owner() {
    let system = r#"{"equipped":{"carryType":"worn","handsHeld":1.0,"inSlot":false,"invested":null},"hp":{"max":9007199254740993,"value":0.1},"price":{"value":{"gp":25,"sp":0},"per":2,"sizeSensitive":false},"usage":{"value":"worn-gloves"}}"#;
    let source = parse(system);
    let physical = parse_physical_equipment_fields(context(), system.as_bytes()).unwrap();
    // Both API roots use the same generated structure; this compares typed data.
    assert_eq!(source.system, physical);
    let equipped = source.system.equipped.as_value().unwrap();
    assert_eq!(
        equipped.carry_type,
        SourcePresence::Value(EquippedDataCarryType::Worn)
    );
    assert_eq!(equipped.hands_held.as_value().unwrap().as_f64(), Some(1.0));
    assert_eq!(equipped.in_slot, SourcePresence::Value(false));
    assert_eq!(equipped.invested, SourcePresence::Null);
    let hp = source.system.hp.as_value().unwrap();
    assert_eq!(
        hp.max.as_value().unwrap().as_u64(),
        Some(9_007_199_254_740_993)
    );
    assert_eq!(hp.value.as_value().unwrap().as_f64(), Some(0.1));
    let price = source.system.price.as_value().unwrap();
    assert_eq!(
        price
            .value
            .as_value()
            .unwrap()
            .gp
            .as_value()
            .unwrap()
            .as_u64(),
        Some(25)
    );
    assert_eq!(price.value.as_value().unwrap().cp, SourcePresence::Missing);
    assert_eq!(price.size_sensitive, SourcePresence::Value(false));
    assert_eq!(
        source
            .system
            .usage
            .as_value()
            .unwrap()
            .value
            .as_value()
            .unwrap(),
        "worn-gloves"
    );
}

#[test]
fn missing_null_and_empty_source_objects_remain_distinct_before_defaults() {
    let missing = parse("{}");
    let null = parse(r#"{"equipped":null,"hp":null,"price":null,"usage":null}"#);
    let empty = parse(r#"{"equipped":{},"hp":{},"price":{},"usage":{}}"#);
    assert_eq!(missing.system.equipped, SourcePresence::Missing);
    assert_eq!(null.system.equipped, SourcePresence::Null);
    assert!(empty.system.equipped.as_value().is_some());
    assert_eq!(
        empty.system.hp.as_value().unwrap().max,
        SourcePresence::Missing
    );
    assert_eq!(
        empty.system.price.as_value().unwrap().value,
        SourcePresence::Missing
    );
    assert_eq!(
        empty.system.usage.as_value().unwrap().value,
        SourcePresence::Missing
    );
    assert_ne!(missing, null);
    assert_ne!(null, empty);
}

#[test]
fn legacy_fields_duplicates_arrays_and_subitems_are_retained_as_pending_source() {
    let source = parse(
        r#"{"price":{"legacy":0,"legacy":false,"value":{"gp":0,"oldCoin":1}},"traits":{"value":["future-trait"]},"subitems":[{"type":"equipment","system":{"hp":{"value":1}}}],"old":1,"old":2}"#,
    );
    let price = source.system.price.as_value().unwrap();
    assert_eq!(price.additional_fields.fields().len(), 2);
    assert_eq!(
        price.additional_fields.fields()[0],
        ("legacy".into(), SourceValue::Number(0.into()))
    );
    assert_eq!(
        price.additional_fields.fields()[1],
        ("legacy".into(), SourceValue::Boolean(false))
    );
    assert_eq!(
        price
            .value
            .as_value()
            .unwrap()
            .additional_fields
            .fields()
            .len(),
        1
    );
    let remaining = source.system.additional_fields.fields();
    assert_eq!(
        remaining
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["traits", "subitems", "old", "old"]
    );
    assert!(matches!(&remaining[1].1, SourceValue::Array(values) if values.len() == 1));
    assert_eq!(source.envelope_fields.fields()[0].0, "name");
}

#[test]
fn structural_duplicates_and_changed_literal_values_have_exact_context() {
    for (system, suffix) in [
        (r#"{"hp":{},"hp":null}"#, ".hp"),
        (r#"{"price":{"value":{"gp":1,"gp":2}}}"#, ".price.value.gp"),
        (
            r#"{"equipped":{"carryType":"teleported"}}"#,
            ".equipped.carryType",
        ),
        (r#"{"equipped":{"handsHeld":3}}"#, ".equipped.handsHeld"),
        (r#"{"equipped":{"handsHeld":1.5}}"#, ".equipped.handsHeld"),
        (r#"{"hp":{"max":"10"}}"#, ".hp.max"),
    ] {
        let bytes = format!(r#"{{"type":"equipment","system":{system}}}"#);
        let error = parse_equipment_source_slice(context(), bytes.as_bytes()).unwrap_err();
        assert_eq!(*error.context, context());
        assert_eq!(error.json_path, format!("$.items[3].system{suffix}"));
    }
}

#[test]
fn malformed_json_wrong_family_and_missing_system_are_rejected() {
    for source in [
        r#"{"type":"equipment","system":{}} false"#,
        r#"{"type":"equipment","system":{}"#,
        r#"[]"#,
        r#"{"type":"backpack","system":{}}"#,
        r#"{"type":"equipment"}"#,
        r#"{"type":"equipment","type":"equipment","system":{}}"#,
        r#"{"type":"equipment","system":null}"#,
    ] {
        assert!(
            parse_equipment_source_slice(context(), source.as_bytes()).is_err(),
            "{source}"
        );
    }
}
