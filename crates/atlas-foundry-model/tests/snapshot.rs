use atlas_foundry_model::{
    ActorSourcePF2e, FoundryDocumentSource, ItemSourcePF2e, SnapshotError, SourceContext,
    SourcePresence, SourceValue, admit_document_source, admit_rule_source, decode_snapshot,
    encode_snapshot, parse_source_value,
};

fn admitted(kind: &str, bytes: &[u8]) -> FoundryDocumentSource {
    admit_document_source(
        kind,
        SourceContext::new("fixture", "fixture.json", "$"),
        bytes,
    )
    .unwrap()
    .model
    .unwrap()
}

#[test]
fn standalone_consumer_hydrates_and_traverses_authored_models() {
    // This integration-test crate depends only on atlas-foundry-model's deps.
    let npc = admitted("Actor", br#"{"type":"npc","name":"Caster","system":{"attributes":{"hp":{"max":40,"value":0}}},"items":[{"type":"spell","_id":"s","system":{"location":{"value":"entry"}}},{"type":"spellcastingEntry","_id":"entry","system":{"slots":{"slot1":{"prepared":{"0":{"id":"s"},"1":{"id":"s"}}}}}}]}"#);
    let restored = decode_snapshot(&encode_snapshot(&npc).unwrap()).unwrap();
    assert_eq!(restored, npc);
    let FoundryDocumentSource::Actor(actor) = restored else {
        panic!("actor")
    };
    let ActorSourcePF2e::NPCSource(actor) = *actor else {
        panic!("npc")
    };
    assert_eq!(actor.name.as_value().unwrap(), "Caster");
    assert_eq!(actor.items.as_value().unwrap().len(), 2);

    let spell = admitted(
        "Item",
        br#"{"type":"spell","name":"Light","system":{"level":{"value":0},"traits":{"value":[]}}}"#,
    );
    let restored = decode_snapshot(&encode_snapshot(&spell).unwrap()).unwrap();
    assert_eq!(restored, spell);
    let FoundryDocumentSource::Item(item) = restored else {
        panic!("item")
    };
    let ItemSourcePF2e::SpellSource(spell) = *item else {
        panic!("spell")
    };
    assert_eq!(spell.name.as_value().unwrap(), "Light");

    let journal = admitted("JournalEntry", br#"{"name":"Notes","pages":[{"_id":"page","type":"text","text":{"content":"<p>Hello</p>"}}]}"#);
    let restored = decode_snapshot(&encode_snapshot(&journal).unwrap()).unwrap();
    assert_eq!(restored, journal);
    let FoundryDocumentSource::JournalEntry(journal) = restored else {
        panic!("journal")
    };
    assert_eq!(journal.pages.as_value().unwrap().len(), 1);

    let partial = admitted("Item", br#"{"type":"ancestry","name":"Catfolk","system":{"items":{"feature":{"uuid":"Compendium.x","level":"1"}}}}"#);
    let restored = decode_snapshot(&encode_snapshot(&partial).unwrap()).unwrap();
    assert_eq!(restored, partial);
    let FoundryDocumentSource::Item(item) = restored else {
        panic!("item")
    };
    let ItemSourcePF2e::AncestrySource(ancestry) = *item else {
        panic!("ancestry")
    };
    let feature = &ancestry
        .system
        .as_value()
        .unwrap()
        .items
        .as_value()
        .unwrap()
        .entries[0]
        .1;
    let SourcePresence::Invalid(invalid) = &feature.level else {
        panic!("invalid")
    };
    assert_eq!(invalid.values, vec![SourceValue::String("1".into())]);
    assert_eq!(invalid.diagnostic.context.source_path, "fixture.json");
}

#[test]
fn ordered_values_and_presence_states_roundtrip_without_source_reparsing() {
    for (kind, bytes) in [
        ("Item", br#"{"type":"action","name":null,"img":"","system":{"actionType":{"value":"passive"},"frequency":{"max":0,"per":"day"}},"future":9007199254740993,"future":null,"other":false}"#.as_slice()),
        ("Actor", br#"{"type":"npc","system":{"attributes":{"hp":{"max":10,"max":20,"value":0}},"traits":{"value":["fire",7,"cold"]}},"items":[]}"#.as_slice()),
        ("Item", br#"{"type":"spellcastingEntry","system":{"slots":{"slot1":{"prepared":["bad",{"id":"s"}],"max":"3","value":0}}}}"#.as_slice()),
        ("RollTable", br#"{"results":[],"replacement":false,"formula":"1d20"}"#.as_slice()),
        ("Macro", br#"{"type":"script","command":"","name":"Macro"}"#.as_slice()),
    ] {
        let model = admitted(kind, bytes);
        let snapshot = encode_snapshot(&model).unwrap();
        assert_eq!(decode_snapshot(&snapshot).unwrap(), model);
        // Snapshot JSON is not admitted authored JSON and never a raw fallback.
        assert!(decode_snapshot(bytes).is_err());
    }
    let bytes = br#"{"2":9007199254740993,"1":null,"2":false}"#;
    let value = parse_source_value(bytes).unwrap();
    let snapshot = serde_json::to_vec(&value).unwrap();
    assert_eq!(
        serde_json::from_slice::<SourceValue>(&snapshot).unwrap(),
        value
    );
    assert_ne!(parse_source_value(&snapshot).unwrap(), value);
    assert!(serde_json::from_slice::<SourceValue>(bytes).is_err());
}

#[test]
fn recursive_specific_rule_variants_roundtrip_exactly() {
    let admitted = admit_rule_source(SourceContext::new("rule", "rule.json", "$"),
        br#"{"key":"FlatModifier","selector":"attack","value":2,"predicate":[{"or":["a",{"and":["b",{"not":"c"}]}]},{"gte":["level",5]}]}"#).unwrap();
    let model = admitted.model.unwrap();
    let bytes = serde_json::to_vec(&model).unwrap();
    let restored = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(model, restored);
}

#[test]
fn raw_only_roots_remain_without_an_invented_snapshot_model() {
    for kind in ["Item", "Unknown"] {
        let admission = admit_document_source(
            kind,
            SourceContext::new("raw", "raw.json", "$"),
            br#"{"type":"future","x":1,"x":2}"#,
        )
        .unwrap();
        assert!(admission.model.is_none());
        assert_eq!(
            admission.raw,
            parse_source_value(br#"{"type":"future","x":1,"x":2}"#).unwrap()
        );
        assert!(!admission.diagnostics.is_empty());
    }
}

#[test]
fn version_identity_tags_and_corrupt_models_fail_explicitly() {
    let model = admitted("Item", br#"{"type":"action"}"#);
    let bytes = encode_snapshot(&model).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut changed = original.clone();
    changed["version"] = 999.into();
    changed["model"] = serde_json::Value::Null;
    assert!(matches!(
        decode_snapshot(&serde_json::to_vec(&changed).unwrap()),
        Err(SnapshotError::UnsupportedVersion(999))
    ));
    let mut changed = original.clone();
    changed["source_contract"] = "different".into();
    assert!(matches!(
        decode_snapshot(&serde_json::to_vec(&changed).unwrap()),
        Err(SnapshotError::SourceContractMismatch(_))
    ));
    for path in [
        vec!["model", "$variant"],
        vec!["model", "$value", "$variant"],
    ] {
        let mut changed = original.clone();
        let mut target = &mut changed;
        for key in path {
            target = &mut target[key];
        }
        *target = "Unknown".into();
        assert!(matches!(
            decode_snapshot(&serde_json::to_vec(&changed).unwrap()),
            Err(SnapshotError::InvalidSnapshot(_))
        ));
    }
    let mut changed = original;
    changed["model"]["$value"]["$value"]["name"] = serde_json::json!({"value":37});
    assert!(decode_snapshot(&serde_json::to_vec(&changed).unwrap()).is_err());
    assert!(decode_snapshot(&bytes[..bytes.len() - 1]).is_err());
    let mut changed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    changed["model"]["$value"]["$value"]["unexpected"] = true.into();
    assert!(decode_snapshot(&serde_json::to_vec(&changed).unwrap()).is_err());
}

#[test]
fn deeply_retained_and_typed_nodes_roundtrip_near_authored_json_limit() {
    for (open, close) in [("[", "]"), ("{\"x\":", "}")] {
        let future = format!("{}0{}", open.repeat(125), close.repeat(125));
        let bytes = format!(r#"{{"type":"action","future":{future}}}"#);
        let admission = admit_document_source(
            "Item",
            SourceContext::new("deep", "deep.json", "$"),
            bytes.as_bytes(),
        )
        .unwrap();
        assert!(admission.diagnostics.is_empty());
        let model = admission.model.unwrap();
        assert_eq!(
            decode_snapshot(&encode_snapshot(&model).unwrap()).unwrap(),
            model
        );
    }
    let predicate = format!("{}\"flag\"{}", "{\"not\":".repeat(120), "}".repeat(120));
    let bytes = format!(
        r#"{{"type":"npc","system":{{"customModifiers":{{"ac":[{{"label":"Deep","modifier":1,"predicate":[{predicate}]}}]}}}}}}"#
    );
    let admission = admit_document_source(
        "Actor",
        SourceContext::new("deep", "deep.json", "$"),
        bytes.as_bytes(),
    )
    .unwrap();
    assert!(admission.diagnostics.is_empty());
    let model = admission.model.unwrap();
    assert_eq!(
        decode_snapshot(&encode_snapshot(&model).unwrap()).unwrap(),
        model
    );
}

#[test]
fn nesting_guard_rejects_excessive_payloads_but_ignores_escaped_text() {
    let model = admitted(
        "Item",
        serde_json::json!({"type":"action", "name": "[{}]\\\"".repeat(1100)})
            .to_string()
            .as_bytes(),
    );
    assert_eq!(
        decode_snapshot(&encode_snapshot(&model).unwrap()).unwrap(),
        model
    );
    let too_deep = format!("{}0{}", "[".repeat(1025), "]".repeat(1025));
    assert!(matches!(
        decode_snapshot(too_deep.as_bytes()),
        Err(SnapshotError::NestingLimit)
    ));
    let malformed = format!("{}0", "[".repeat(700));
    assert!(matches!(
        decode_snapshot(malformed.as_bytes()),
        Err(SnapshotError::InvalidSnapshot(_))
    ));
}
