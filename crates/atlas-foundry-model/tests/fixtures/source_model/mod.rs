//! Compile the generic regression graph against the actual source parsing primitives.
#[path = "../../../examples/support/model_inspection.rs"]
mod inspection;

#[path = "../../../src/source_model/keyed.rs"]
mod keyed;
#[path = "../../../src/source_model/parse.rs"]
mod parse;
#[path = "../../../src/source_model/presence.rs"]
mod presence;
#[path = "../../../src/source_model/union.rs"]
mod union;
#[path = "../../../src/source_model/value.rs"]
mod value;
pub use keyed::SourceMap;
#[cfg(test)]
mod fidelity_tests;
pub mod generated;
#[path = "../../../examples/support/rule_fidelity.rs"]
mod rule_fidelity;

#[cfg(test)]
mod tests {
    use super::inspection;
    use super::{generated, parse, value};
    use serde_json::json;
    fn context() -> parse::SourceContext {
        parse::SourceContext::new("fixture", "fixture.json", "$")
    }
    #[test]
    fn numeric_key_domain_matches_typescript_without_key_coercion() {
        let cases: Vec<(String, bool)> =
            serde_json::from_str(include_str!("../numeric-keys.json")).unwrap();
        for (key, expected) in cases {
            assert_eq!(parse::is_numeric_key(&key), expected, "{key:?}");
        }
    }
    #[test]
    fn numeric_maps_preserve_unmodeled_names_order_and_duplicates() {
        let raw = value::parse_source(
            br#"{"1":9007199254740993,"-1":2,"label":null,"01":1,"01":2,"-0":false}"#,
        )
        .unwrap();
        let parsed = generated::parse_numeric_keys(&raw, &context(), "$").unwrap();
        assert_eq!(
            parsed
                .indexed_fields
                .entries
                .iter()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            ["1", "-1"]
        );
        assert_eq!(
            parsed.indexed_fields.entries[0].1.as_u64(),
            Some(9007199254740993)
        );
        assert_eq!(
            parsed
                .additional_fields
                .fields()
                .iter()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            ["label", "01", "01", "-0"]
        );
        let graph = super::rule_fidelity::FidelityGraph::new(&json!({"nodes":[
            {"id":"number","kind":"primitive","value":"number"},
            {"id":"numeric","kind":"object","fields":[],"indexSignatures":[{"key":"primitive:number","value":"number"}]}
        ]}));
        let model = inspection::to_value(&parsed).unwrap();
        assert!(graph.compare("numeric", &raw, &model).is_ok());
        let mut lost = model.clone();
        lost["additional_fields"]["fields"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(graph.compare("numeric", &raw, &lost).is_err());
        let mut changed = model;
        changed["indexed_fields"]["entries"][0][0] = json!("01");
        assert!(graph.compare("numeric", &raw, &changed).is_err());
        for (source, path) in [
            (br#"{"1":"bad"}"#.as_slice(), r#"$["1"]"#),
            (br#"{"1":1,"1":2}"#, r#"$["1"]"#),
        ] {
            let raw = value::parse_source(source).unwrap();
            assert_eq!(
                generated::parse_numeric_keys(&raw, &context(), "$")
                    .unwrap_err()
                    .json_path,
                path
            );
        }
        let raw = value::parse_source(br#"{"keys":{"0":3,"extra":true}}"#).unwrap();
        generated::parse_numeric_consumer(&raw, &context(), "$").unwrap();
    }
    #[test]
    fn numeric_named_constraints_apply_only_to_numeric_names_and_recursion_has_a_struct_anchor() {
        let raw =
            value::parse_source(br#"{"label":"b","2":"a","4":"a","3":false,"3":null,"other":1}"#)
                .unwrap();
        let parsed = generated::parse_numeric_bag(&raw, &context(), "$").unwrap();
        assert_eq!(parsed.indexed_fields.entries[0].0, "4");
        assert_eq!(parsed.additional_fields.fields().len(), 3);
        let raw = value::parse_source(br#"{"2":"b"}"#).unwrap();
        assert_eq!(
            generated::parse_numeric_bag(&raw, &context(), "$")
                .unwrap_err()
                .json_path,
            r#"$["2"]"#
        );
        let raw =
            value::parse_source(br#"{"1":{"2":{},"note":null},"note":{"future":true}}"#).unwrap();
        generated::parse_recursive_numeric_keys(&raw, &context(), "$").unwrap();
    }
    #[test]
    fn null_only_fields_and_unusual_enum_tokens_remain_exact_and_strict() {
        for token in ["", "0", "-"] {
            let raw = value::parse_source(
                format!(
                    r#"{{"only":null,"token":{}}}"#,
                    serde_json::to_string(token).unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
            let parsed = generated::parse_null_only_fields(&raw, &context(), "$").unwrap();
            let model = inspection::to_value(parsed).unwrap();
            assert_eq!(model["only"], json!("null"));
            assert_eq!(model["maybe"], json!("missing"));
            assert_eq!(model["token"], json!({"value":token}));
        }
        for (source, path) in [
            (br#"{"only":0}"#.as_slice(), "$.only"),
            (br#"{"maybe":"x"}"#, "$.maybe"),
            (br#"{"token":"1"}"#, "$.token"),
        ] {
            let raw = value::parse_source(source).unwrap();
            assert_eq!(
                generated::parse_null_only_fields(&raw, &context(), "$")
                    .unwrap_err()
                    .json_path,
                path
            );
        }
    }
    #[test]
    fn tagged_documents_identify_family_before_defaults_and_check_present_payloads() {
        for source in [
            br#"{"type":"a"}"#.as_slice(),
            br#"{"type":"a","payload":null,"retired":false}"#,
            br#"{"type":"b"}"#,
        ] {
            let raw = value::parse_source(source).unwrap();
            generated::parse_tagged_documents(&raw, &context(), "$").unwrap();
        }
        for (source, path) in [
            (br#"{"type":"a","payload":3}"#.as_slice(), "$.payload"),
            (br#"{"type":"b","count":"bad"}"#, "$.count"),
            (br#"{"type":"a","type":"a"}"#, "$.type"),
            (br#"{"type":null}"#, "$"),
            (br#"{"type":"unknown"}"#, "$"),
            (br#"{}"#, "$"),
        ] {
            let raw = value::parse_source(source).unwrap();
            assert_eq!(
                generated::parse_tagged_documents(&raw, &context(), "$")
                    .unwrap_err()
                    .json_path,
                path
            );
        }
    }
    #[test]
    fn nullable_collections_preserve_positions_nulls_numbers_and_ordered_keys() {
        let raw = value::parse_source(br#"{"nullable":[null,18446744073709551615,null],"undefined":[null,3],"map":{"z":null,"a":2},"nulls":[null,null],"undefineds":[null],"nested":[null,[1,null],[]],"nodes":[null,{"next":{"next":null}},null]}"#).unwrap();
        let parsed = generated::parse_collections(&raw, &context(), "$").unwrap();
        let super::presence::SourcePresence::Value(numbers) = &parsed.nullable else {
            panic!()
        };
        assert_eq!(numbers.len(), 3);
        assert!(numbers[0].is_none() && numbers[2].is_none());
        assert_eq!(numbers[1].as_ref().unwrap().as_u64(), Some(u64::MAX));
        let super::presence::SourcePresence::Value(map) = &parsed.map else {
            panic!()
        };
        assert_eq!(
            map.entries
                .iter()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            ["z", "a"]
        );
        let serialized = inspection::to_value(parsed).unwrap();
        assert_eq!(
            serialized["nullable"]["value"],
            json!([null, u64::MAX, null])
        );
        assert_eq!(serialized["undefined"]["value"], json!([null, 3]));
        assert_eq!(serialized["nulls"]["value"], json!([null, null]));
        assert_eq!(serialized["undefineds"]["value"], json!([null]));
        assert_eq!(serialized["nested"]["value"], json!([null, [1, null], []]));
        assert_eq!(serialized["nodes"]["value"][0], json!(null));
        assert_eq!(serialized["nodes"]["value"][2], json!(null));
        let graph = super::rule_fidelity::FidelityGraph::new(&json!({"nodes":[
            {"id":"number","kind":"primitive","value":"number"},
            {"id":"null","kind":"primitive","value":"null"},
            {"id":"undefined","kind":"primitive","value":"undefined"},
            {"id":"maybe","kind":"union","members":["number","null","undefined"]},
            {"id":"numbers","kind":"array","element":"maybe"},
            {"id":"optional_numbers","kind":"union","members":["numbers","undefined"]},
            {"id":"nested","kind":"array","element":"optional_numbers"},
            {"id":"nulls","kind":"array","element":"null"},
            {"id":"map","kind":"object","fields":[],"indexSignatures":[{"key":"primitive:string","value":"maybe"}]}
        ]}));
        let value::SourceValue::Object(object) = &raw else {
            panic!()
        };
        for (key, reference) in [
            ("nullable", "numbers"),
            ("undefined", "numbers"),
            ("nested", "nested"),
            ("nulls", "nulls"),
            ("map", "map"),
        ] {
            let source = &object
                .fields()
                .iter()
                .find(|(name, _)| name == key)
                .unwrap()
                .1;
            assert!(
                graph
                    .compare(reference, source, &serialized[key]["value"])
                    .is_ok(),
                "{key}"
            );
        }
    }

    #[test]
    fn collection_entry_diagnostics_and_duplicate_checks_stay_precise() {
        for (bytes, path) in [
            (br#"{"nullable":[null,"bad"]}"#.as_slice(), "$.nullable[1]"),
            (br#"{"map":{"a":null,"bad":false}}"#, r#"$.map["bad"]"#),
            (br#"{"map":{"a":null,"a":2}}"#, r#"$.map["a"]"#),
            (br#"{"nulls":[null,1]}"#, "$.nulls[1]"),
            (br#"{"nested":[null,[null,false]]}"#, "$.nested[1][1]"),
        ] {
            let raw = value::parse_source(bytes).unwrap();
            let error = generated::parse_collections(&raw, &context(), "$").unwrap_err();
            assert_eq!(error.json_path, path);
            assert_eq!(*error.context, context());
        }
        let raw = value::parse_source(br#"{"fixed":2,"dynamic":null}"#).unwrap();
        let bag = generated::parse_nullable_number_bag(&raw, &context(), "$").unwrap();
        assert!(bag.indexed_fields.entries[0].1.is_none());
        let bad = value::parse_source(br#"{"fixed":false}"#).unwrap();
        assert!(generated::parse_nullable_number_bag(&bad, &context(), "$").is_err());
    }

    #[test]
    fn nullable_tuple_union_guards_preserve_arity_and_literal_identity() {
        for bytes in [b"[null]".as_slice(), br#"["a"]"#, br#"["heal"]"#] {
            let raw = value::parse_source(bytes).unwrap();
            let parsed = generated::parse_nullable_tuple_union(&raw, &context(), "$").unwrap();
            assert_eq!(
                inspection::to_value(parsed).unwrap(),
                serde_json::from_slice::<serde_json::Value>(bytes).unwrap()
            );
        }
        for bytes in [b"[]".as_slice(), b"[null,null]", br#"["bad"]"#, b"[1]"] {
            let raw = value::parse_source(bytes).unwrap();
            assert!(generated::parse_nullable_tuple_union(&raw, &context(), "$").is_err());
        }
        let raw = value::parse_source(br#"{"tuple":[null],"numbers":[null,1]}"#).unwrap();
        let parsed = generated::parse_collection_consumer(&raw, &context(), "$").unwrap();
        assert_eq!(
            inspection::to_value(parsed).unwrap()["tuple"]["value"],
            json!([null])
        );
    }
    #[test]
    fn nullable_array_union_keeps_null_identity_and_reports_empty_ambiguity() {
        for bytes in [b"[null]".as_slice(), br#"[null,"a",null]"#, br#"["b"]"#] {
            let raw = value::parse_source(bytes).unwrap();
            let parsed = generated::parse_nullable_array_union(&raw, &context(), "$").unwrap();
            assert_eq!(
                inspection::to_value(parsed).unwrap(),
                serde_json::from_slice::<serde_json::Value>(bytes).unwrap()
            );
        }
        for bytes in [b"[]".as_slice(), br#"[null,"b"]"#, b"[1]"] {
            let raw = value::parse_source(bytes).unwrap();
            assert!(generated::parse_nullable_array_union(&raw, &context(), "$").is_err());
        }
    }
    #[test]
    fn mapped_source_keys_preserve_values_serialized_names_and_diagnostics() {
        let raw =
            value::parse_source(br#"{"_id":"abc","greater-darkvision":60,"self":"x","1st":"y"}"#)
                .unwrap();
        let parsed = generated::parse_mapped_fields(&raw, &context(), "$").unwrap();
        assert_eq!(
            parsed._id,
            super::presence::SourcePresence::Value("abc".into())
        );
        let serialized = inspection::to_value(parsed).unwrap();
        assert_eq!(serialized["greater-darkvision"]["value"], json!(60));
        assert_eq!(serialized["self"]["value"], json!("x"));
        assert_eq!(serialized["1st"]["value"], json!("y"));
        let bad = value::parse_source(br#"{"greater-darkvision":"bad"}"#).unwrap();
        assert_eq!(
            generated::parse_mapped_fields(&bad, &context(), "$")
                .unwrap_err()
                .json_path,
            "$.greater-darkvision"
        );
        let duplicate = value::parse_source(br#"{"_id":"a","_id":"b"}"#).unwrap();
        assert!(generated::parse_mapped_fields(&duplicate, &context(), "$").is_err());
    }

    #[test]
    fn indexed_intersections_enforce_resolved_constraints_and_recursive_layouts() {
        let raw = value::parse_source(br#"{"fixed":"a","dynamic":"a"}"#).unwrap();
        let parsed = generated::parse_intersection_bag(&raw, &context(), "$").unwrap();
        assert_eq!(parsed.indexed_fields.entries[0].0, "dynamic");
        for bytes in [
            br#"{"fixed":"b"}"#.as_slice(),
            br#"{"fixed":"a","dynamic":"b"}"#,
        ] {
            let raw = value::parse_source(bytes).unwrap();
            assert!(generated::parse_intersection_bag(&raw, &context(), "$").is_err());
        }
        let raw = value::parse_source(br#"{"z":2,"a":1}"#).unwrap();
        let parsed = generated::parse_intersection_map(&raw, &context(), "$").unwrap();
        assert_eq!(
            parsed
                .entries
                .iter()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            vec!["z", "a"]
        );
        let raw = value::parse_source(br#"{"next":{"leaf":{}},"branch":{"next":null}}"#).unwrap();
        let parsed = generated::parse_recursive_intersection(&raw, &context(), "$").unwrap();
        assert_eq!(parsed.indexed_fields.entries[0].0, "branch");
    }
    #[test]
    fn template_strings_preserve_literals_and_arbitrary_interpolations() {
        for text in ["#", "#蓝\n.[]", "#ff00ff"] {
            let raw = value::SourceValue::String(text.into());
            assert_eq!(generated::parse_color(&raw, &context(), "$").unwrap(), text);
        }
        for text in ["Actor..Item.", "Actor.a.Item.b", "Actor.a.Item.x.Item.b"] {
            let raw = value::SourceValue::String(text.into());
            assert_eq!(generated::parse_uuid(&raw, &context(), "$").unwrap(), text);
        }
        for text in [".png", "图片\n.svg", "[a](b).png"] {
            let raw = value::SourceValue::String(text.into());
            assert_eq!(
                generated::parse_image_path(&raw, &context(), "$").unwrap(),
                text
            );
        }
        for text in ["xActor.a.Item.b", "Actor.a", "actor.a.Item.b"] {
            let raw = value::SourceValue::String(text.into());
            assert_eq!(
                generated::parse_uuid(&raw, &context(), "$.uuid")
                    .unwrap_err()
                    .json_path,
                "$.uuid"
            );
        }
        for text in ["x.png.bad", "image.PNG", "imagepng"] {
            let raw = value::SourceValue::String(text.into());
            assert!(generated::parse_image_path(&raw, &context(), "$").is_err());
        }
        assert!(!parse::matches_template("x", &["", "x", "x"]));
        assert!(parse::matches_template("xx", &["", "x", "x"]));
        assert!(parse::matches_template("🙂", &["", "", ""]));
    }
    #[test]
    fn template_constraints_identify_mixed_and_tuple_union_arms() {
        for source in [json!("#"), json!({"label":"x"})] {
            let raw = value::parse_source(&serde_json::to_vec(&source).unwrap()).unwrap();
            generated::parse_mixed_template(&raw, &context(), "$").unwrap();
        }
        for source in [json!(["#蓝"]), json!(3)] {
            let raw = value::parse_source(&serde_json::to_vec(&source).unwrap()).unwrap();
            let parsed = generated::parse_template_tuple_union(&raw, &context(), "$").unwrap();
            assert_eq!(inspection::to_value(parsed).unwrap(), source);
        }
        let raw = value::parse_source(br#"["bad"]"#).unwrap();
        assert!(generated::parse_template_tuple_union(&raw, &context(), "$").is_err());
        let raw = value::SourceValue::String("Actor.a.Item.b".into());
        assert_eq!(
            generated::parse_overlapping_templates(&raw, &context(), "$").unwrap(),
            "Actor.a.Item.b"
        );
        let raw = value::parse_source(r##"{"color":"#蓝","image":"x.svg"}"##.as_bytes()).unwrap();
        generated::parse_template_consumer(&raw, &context(), "$").unwrap();
        let raw = value::SourceValue::Boolean(false);
        assert!(generated::parse_color(&raw, &context(), "$").is_err());
    }
    #[test]
    fn instantiated_generic_owners_compile_share_and_parse() {
        for source in [
            json!({"first":"x"}),
            json!({"second":18446744073709551615u64}),
        ] {
            let raw = value::parse_source(&serde_json::to_vec(&source).unwrap()).unwrap();
            let parsed = generated::parse_generic_union(&raw, &context(), "$").unwrap();
            let actual = inspection::to_value(parsed).unwrap();
            for (name, expected) in source.as_object().unwrap() {
                assert_eq!(&actual[name]["value"], expected);
            }
        }
        let raw = value::parse_source(br#"{"repeated":{"first":"x"},"anonymous":{"anonymous":3}}"#)
            .unwrap();
        generated::parse_generic_consumer(&raw, &context(), "$").unwrap();
    }
    #[test]
    fn required_template_fields_discriminate_object_alternatives() {
        for tag in ["#", "Actor.a.Item.b"] {
            let raw =
                value::parse_source(&serde_json::to_vec(&json!({"tag":tag})).unwrap()).unwrap();
            generated::parse_template_tagged(&raw, &context(), "$").unwrap();
        }
        let raw = value::parse_source(br#"{"tag":"unmatched"}"#).unwrap();
        assert!(generated::parse_template_tagged(&raw, &context(), "$").is_err());
    }
    #[test]
    fn mixed_inline_cycles_have_compilable_owners_and_finite_parsing() {
        let value =
            value::parse_source(br#"{"next":{"expression":"a"},"expression":{"next":null}}"#)
                .unwrap();
        let parsed = generated::parse_node(&value, &context(), "$").unwrap();
        assert_eq!(
            inspection::to_value(parsed).unwrap()["next"]["value"]["expression"],
            json!({"value":"a"})
        );
        generated::parse_expr(&value, &context(), "$").unwrap();
    }
    #[test]
    fn empty_and_single_tuples_serialize_as_arrays() {
        let empty = value::parse_source(b"[]").unwrap();
        let parsed = generated::parse_empty(&empty, &context(), "$").unwrap();
        assert_eq!(inspection::to_value(parsed).unwrap(), json!([]));
        let single = value::parse_source(br#"["a"]"#).unwrap();
        assert_eq!(
            inspection::to_value(generated::parse_single(&single, &context(), "$").unwrap())
                .unwrap(),
            json!(["a"])
        );
    }
    #[test]
    fn nullable_discriminators_and_keyword_names_preserve_source_state() {
        for source in [br#"{"type":null,"if":true}"#.as_slice(), br#"{"type":"b"}"#] {
            let value = value::parse_source(source).unwrap();
            generated::parse_discriminated(&value, &context(), "$").unwrap();
        }
        let value = value::parse_source(br#"{"type":"a","if":false}"#).unwrap();
        assert!(generated::parse_discriminated(&value, &context(), "$").is_err());
    }
    #[test]
    fn literal_roots_and_cross_module_primitive_parsers_are_callable() {
        let yes = value::parse_source(b"true").unwrap();
        assert!(generated::parse_yes(&yes, &context(), "$").unwrap());
        let one = value::parse_source(b"1").unwrap();
        assert_eq!(
            generated::parse_one(&one, &context(), "$")
                .unwrap()
                .as_u64(),
            Some(1)
        );
        let object = value::parse_source(br#"{"enabled":true}"#).unwrap();
        generated::parse_bool_consumer(&object, &context(), "$").unwrap();
    }
    #[test]
    fn optional_arm_identity_stays_distinct_from_shared_payload_owners() {
        let other = value::parse_source(br#"{"other":"x"}"#).unwrap();
        generated::parse_strict(&other, &context(), "$").unwrap();
        generated::parse_loose(&other, &context(), "$").unwrap();
        let empty = value::parse_source(b"{}").unwrap();
        assert!(generated::parse_strict(&empty, &context(), "$").is_err());
        generated::parse_loose(&empty, &context(), "$").unwrap();
    }
    #[test]
    fn authored_choices_preserve_missing_predicates_and_reject_conflicting_shapes() {
        for (source, predicate) in [
            (
                r#"{"choices":{"config":"weaponGroups"}}"#,
                serde_json::json!("missing"),
            ),
            (
                r#"{"choices":{"ownedItems":true,"types":["weapon"]}}"#,
                serde_json::json!("missing"),
            ),
            (
                r#"{"choices":{"attacks":true}}"#,
                serde_json::json!("missing"),
            ),
            (
                r#"{"choices":{"unarmedAttacks":true,"predicate":["item:unarmed"]}}"#,
                serde_json::json!({"value":["item:unarmed"]}),
            ),
            (
                r#"{"choices":{"config":"weaponGroups","predicate":[]}}"#,
                serde_json::json!({"value":[]}),
            ),
        ] {
            let raw = value::parse_source(source.as_bytes()).unwrap();
            let model = inspection::to_value(
                generated::parse_authored_choice_rule(&raw, &context(), "$").unwrap(),
            )
            .unwrap();
            assert_eq!(model["choices"]["value"]["predicate"], predicate);
        }
        let query = value::parse_source(br#"{"choices":{"filter":["item:feat"]}}"#).unwrap();
        generated::parse_authored_choice_rule(&query, &context(), "$").unwrap();
        for source in [
            r#"{"choices":"weaponGroups"}"#,
            r#"{"choices":[{"value":"a"},{"value":"b","predicate":[]}]}"#,
        ] {
            let raw = value::parse_source(source.as_bytes()).unwrap();
            generated::parse_authored_choice_rule(&raw, &context(), "$").unwrap();
        }
        for source in [
            r#"{"choices":{"config":"x","attacks":true}}"#,
            r#"{"choices":{"config":"x","ownedItems":null}}"#,
            r#"{"choices":{"config":"x","ownedItems":false}}"#,
            r#"{"choices":{"ownedItems":true}}"#,
            r#"{"choices":{"config":"x","predicate":[{}]}}"#,
            r#"{"choices":{"config":"x","config":"y"}}"#,
        ] {
            let raw = value::parse_source(source.as_bytes()).unwrap();
            assert!(generated::parse_authored_choice_rule(&raw, &context(), "$").is_err());
        }
        let malformed = value::parse_source(br#"{"choices":{"filter":"bad"}}"#).unwrap();
        assert_eq!(
            generated::parse_authored_choice_rule(&malformed, &context(), "$")
                .unwrap_err()
                .json_path,
            "$.choices.filter"
        );
    }
    #[test]
    fn authored_damage_overrides_keep_expressions_and_closed_boolean_fields() {
        for (source, expected) in [
            (
                r#"{"override":{"diceNumber":3}}"#,
                serde_json::json!({"value":3}),
            ),
            (
                r#"{"override":{"diceNumber":"@actor.level"}}"#,
                serde_json::json!({"value":"@actor.level"}),
            ),
            (
                r#"{"override":{"diceNumber":null}}"#,
                serde_json::json!("null"),
            ),
            (r#"{"override":{}}"#, serde_json::json!("missing")),
        ] {
            let raw = value::parse_source(source.as_bytes()).unwrap();
            let model = inspection::to_value(
                generated::parse_authored_damage_rule(&raw, &context(), "$").unwrap(),
            )
            .unwrap();
            assert_eq!(model["override"]["value"]["dice_number"], expected);
        }
        for (damage, die) in [("fire", "d6"), ("{item|flags.damage}", "{item|flags.die}")] {
            let source =
                serde_json::json!({"override":{"damageType":damage,"dieSize":die,"upgrade":true}});
            let raw = value::parse_source(source.to_string().as_bytes()).unwrap();
            let model = inspection::to_value(
                generated::parse_authored_damage_rule(&raw, &context(), "$").unwrap(),
            )
            .unwrap();
            assert_eq!(
                model["override"]["value"]["damage_type"],
                serde_json::json!({"value":damage})
            );
            assert_eq!(
                model["override"]["value"]["die_size"],
                serde_json::json!({"value":die})
            );
        }
        for source in [
            r#"{"override":{"diceNumber":true}}"#,
            r#"{"override":{"damageType":3}}"#,
            r#"{"override":{"upgrade":"true"}}"#,
        ] {
            let raw = value::parse_source(source.as_bytes()).unwrap();
            assert!(generated::parse_authored_damage_rule(&raw, &context(), "$").is_err());
        }
    }
    #[test]
    fn authored_battle_form_base_types_preserve_strings_without_widening_direct_strikes() {
        for (members, expected) in [
            (r#""baseType":"leg""#, serde_json::json!({"value":"leg"})),
            (
                r#""baseType":"future-base","future":1,"future":2"#,
                serde_json::json!({"value":"future-base"}),
            ),
            (r#""baseType":null"#, serde_json::json!("null")),
            ("", serde_json::json!("missing")),
        ] {
            let source = format!(r#"{{"immunities":[],"strikes":{{"limb":{{{members}}}}}}}"#);
            let raw = value::parse_source(source.as_bytes()).unwrap();
            let model = inspection::to_value(
                generated::parse_authored_form(&raw, &context(), "$").unwrap(),
            )
            .unwrap();
            assert_eq!(
                model["strikes"]["value"]["entries"][0][1]["base_type"],
                expected
            );
            if members.contains("future") {
                assert_eq!(
                    model["strikes"]["value"]["entries"][0][1]["additional_fields"]["fields"],
                    serde_json::json!([["future", {"Number":1}], ["future", {"Number":2}]])
                );
            }
        }
        for source in [
            r#"{"immunities":[],"strikes":{"limb":{"baseType":1}}}"#,
            r#"{"immunities":[],"strikes":{"limb":{"baseType":true}}}"#,
            r#"{"immunities":[],"strikes":{"limb":{"baseType":"leg","range":100}}}"#,
        ] {
            let raw = value::parse_source(source.as_bytes()).unwrap();
            assert!(generated::parse_authored_form(&raw, &context(), "$").is_err());
        }
        let direct = value::parse_source(br#"{"traits":[],"baseType":"leg"}"#).unwrap();
        assert!(generated::parse_authored_strike_rule(&direct, &context(), "$").is_err());
    }
    #[test]
    fn authored_strike_scalar_and_array_traits_share_the_open_vocabulary() {
        for (source, expected) in [
            (
                r#"{"traits":"future-trait"}"#,
                serde_json::json!({"value":"future-trait"}),
            ),
            (
                r#"{"traits":["agile","future-trait"]}"#,
                serde_json::json!({"value":["agile","future-trait"]}),
            ),
            (r#"{"traits":[]}"#, serde_json::json!({"value":[]})),
        ] {
            let raw = value::parse_source(source.as_bytes()).unwrap();
            let model = inspection::to_value(
                generated::parse_authored_strike_rule(&raw, &context(), "$").unwrap(),
            )
            .unwrap();
            assert_eq!(model["traits"], expected);
        }
        let raw = value::parse_source(br#"{"traits":["agile",3]}"#).unwrap();
        assert!(generated::parse_authored_strike_rule(&raw, &context(), "$").is_err());
    }
    #[test]
    fn divine_font_alternatives_use_arity_and_position_literals() {
        for source in [
            b"[]".as_slice(),
            br#"["harm"]"#,
            br#"["heal"]"#,
            br#"["harm","heal"]"#,
        ] {
            let value = value::parse_source(source).unwrap();
            let parsed = generated::parse_divine_fonts(&value, &context(), "$").unwrap();
            assert_eq!(
                inspection::to_value(parsed).unwrap(),
                serde_json::from_slice::<serde_json::Value>(source).unwrap()
            );
        }
        for source in [
            br#"["heal","harm"]"#.as_slice(),
            br#"["unknown"]"#,
            br#"["harm","heal","harm"]"#,
            b"null",
        ] {
            let value = value::parse_source(source).unwrap();
            assert!(generated::parse_divine_fonts(&value, &context(), "$").is_err());
        }
    }
    #[test]
    fn tuple_cycles_use_nominal_union_indirection_and_finite_identity_guards() {
        let value = value::parse_source(br#"[[["leaf"]]]"#).unwrap();
        let parsed = generated::parse_tuple_expr(&value, &context(), "$").unwrap();
        assert_eq!(inspection::to_value(parsed).unwrap(), json!([[["leaf"]]]));
        let parsed = generated::parse_tuple_entry(&value, &context(), "$").unwrap();
        assert_eq!(inspection::to_value(parsed).unwrap(), json!([[["leaf"]]]));
    }
    #[test]
    fn primitive_unions_preserve_all_scalar_alternatives_and_restricted_literals() {
        for scalar in [
            json!("option"),
            json!(18446744073709551615u64),
            json!(false),
        ] {
            let source = json!({"numeric":18446744073709551615u64,"logical":false,
                "numeric_boolean":true,"any_scalar":scalar,"restricted":true});
            let value = value::parse_source(&serde_json::to_vec(&source).unwrap()).unwrap();
            let parsed = generated::parse_scalar_envelope(&value, &context(), "$").unwrap();
            let actual = inspection::to_value(parsed).unwrap();
            for field in [
                "numeric",
                "logical",
                "numeric_boolean",
                "any_scalar",
                "restricted",
            ] {
                assert_eq!(actual[field]["value"], source[field]);
            }
        }
        let value = value::parse_source(br#"{"restricted":false}"#).unwrap();
        let error = generated::parse_scalar_envelope(&value, &context(), "$").unwrap_err();
        assert_eq!(error.json_path, "$.restricted");
    }
    #[test]
    fn inline_tuple_types_share_cross_module_parsers_and_named_element_owners() {
        for scalar in [
            json!("option"),
            json!(18446744073709551615u64),
            json!(false),
        ] {
            let source = json!({"pair":[scalar,18446744073709551615u64]});
            let value = value::parse_source(&serde_json::to_vec(&source).unwrap()).unwrap();
            let parsed = generated::parse_scalar_consumer(&value, &context(), "$").unwrap();
            assert_eq!(
                inspection::to_value(parsed).unwrap()["pair"]["value"],
                source["pair"]
            );
        }
    }

    #[test]
    fn open_domains_preserve_values_and_enforce_json_kind_boundaries() {
        for bytes in [
            b"null".as_slice(),
            b"false",
            b"18446744073709551615",
            br#""text""#,
            br#"[null,{"repeated":1,"repeated":2}]"#,
            br#"{"repeated":1,"repeated":2}"#,
        ] {
            let value = value::parse_source(bytes).unwrap();
            assert_eq!(
                generated::parse_any_value(&value, &context(), "$").unwrap(),
                value
            );
            assert_eq!(
                generated::parse_unknown_value(&value, &context(), "$").unwrap(),
                value
            );
            assert_eq!(
                generated::parse_non_nullish_value(&value, &context(), "$").is_ok(),
                !matches!(value, value::SourceValue::Null)
            );
            assert_eq!(
                generated::parse_object_value(&value, &context(), "$").is_ok(),
                matches!(
                    value,
                    value::SourceValue::Object(_) | value::SourceValue::Array(_)
                )
            );
        }
    }

    #[test]
    fn indexed_fields_preserve_order_values_and_forbidden_additional_members() {
        let value = value::parse_source(
            br#"{"z":null,"label":"bag","a":{"x":1,"x":2},"retired":null,"retired":[]}"#,
        )
        .unwrap();
        let parsed = generated::parse_open_bag(&value, &context(), "$").unwrap();
        assert_eq!(
            parsed.label,
            super::presence::SourcePresence::Value("bag".into())
        );
        assert_eq!(
            parsed
                .indexed_fields
                .entries
                .iter()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            vec!["z", "a"]
        );
        let value::SourceValue::Object(raw) = value else {
            panic!()
        };
        assert_eq!(parsed.indexed_fields.entries[0].1, raw.fields()[0].1);
        assert_eq!(parsed.indexed_fields.entries[1].1, raw.fields()[2].1);
        assert_eq!(parsed.additional_fields.fields(), &raw.fields()[3..]);
    }

    #[test]
    fn index_constraints_check_named_values_and_fail_at_escaped_dynamic_paths() {
        let valid = value::parse_source(br#"{"fixed":"a","dynamic":"a"}"#).unwrap();
        generated::parse_constrained_bag(&valid, &context(), "$").unwrap();
        let invalid = value::parse_source(br#"{"fixed":"b"}"#).unwrap();
        let error = generated::parse_constrained_bag(&invalid, &context(), "$").unwrap_err();
        assert_eq!(error.json_path, r#"$["fixed"]"#);
        for (bytes, path) in [
            (br#"{"fixed":1,"x":null}"#.as_slice(), r#"$["x"]"#),
            (br#"{"fixed":1,"x":2,"x":3}"#, r#"$["x"]"#),
            (br#"{"fixed":1,"quote\"key":"bad"}"#, r#"$["quote\"key"]"#),
            (br#"{"fixed":1,"fixed":2}"#, "$.fixed"),
        ] {
            let value = value::parse_source(bytes).unwrap();
            let error = generated::parse_number_bag(&value, &context(), "$").unwrap_err();
            assert_eq!(error.json_path, path);
            assert_eq!(error.context.record_key, "fixture");
        }
        // The ordinary named-field pre-default policy remains unchanged.
        let null = value::parse_source(br#"{"fixed":null}"#).unwrap();
        assert_eq!(
            generated::parse_number_bag(&null, &context(), "$")
                .unwrap()
                .fixed,
            super::presence::SourcePresence::Null
        );
    }

    #[test]
    fn indexed_recursion_and_cross_module_open_owners_compile_and_parse() {
        let value =
            value::parse_source(br#"{"child":{"leaf":{}},"branch":{"child":null}}"#).unwrap();
        let parsed = generated::parse_recursive_bag(&value, &context(), "$").unwrap();
        assert_eq!(parsed.indexed_fields.entries[0].0, "branch");
        let consumer =
            value::parse_source(br#"{"bag":{"label":"x","future":[null,3]},"payload":[]}"#)
                .unwrap();
        generated::parse_open_consumer(&consumer, &context(), "$").unwrap();
    }

    #[test]
    fn open_union_arms_keep_kind_constraints_and_defer_to_known_shapes() {
        for bytes in [br#""string""#.as_slice(), b"[]", b"{}"] {
            let value = value::parse_source(bytes).unwrap();
            generated::parse_object_union(&value, &context(), "$").unwrap();
        }
        let boolean = value::parse_source(b"true").unwrap();
        assert!(generated::parse_object_union(&boolean, &context(), "$").is_err());
        let overlapping = value::parse_source(br#"{"label":"x"}"#).unwrap();
        let parsed = generated::parse_overlapping_open(&overlapping, &context(), "$").unwrap();
        assert!(matches!(parsed, generated::OverlappingOpen::KnownObject(_)));
        let malformed = value::parse_source(br#"{"label":3}"#).unwrap();
        assert_eq!(
            generated::parse_overlapping_open(&malformed, &context(), "$")
                .unwrap_err()
                .json_path,
            "$.label"
        );
        let future = value::parse_source(br#"{"future":1,"future":2}"#).unwrap();
        let parsed = generated::parse_overlapping_open(&future, &context(), "$").unwrap();
        assert!(matches!(parsed, generated::OverlappingOpen::Object(_)));
        let ambiguous = value::parse_source(b"{}").unwrap();
        assert!(generated::parse_ambiguous_fallback(&ambiguous, &context(), "$").is_err());
    }

    #[test]
    fn unknown_maps_accept_null_but_optional_number_entries_stay_constrained() {
        let raw = value::parse_source(br#"{"z":null,"a":[1],"o":{"x":1,"x":2}}"#).unwrap();
        let parsed = generated::parse_unknown_map(&raw, &context(), "$").unwrap();
        let value::SourceValue::Object(object) = raw else {
            panic!()
        };
        assert_eq!(parsed.entries, object.fields());
        for bytes in [b"{}".as_slice(), br#"{"x":3}"#] {
            let value = value::parse_source(bytes).unwrap();
            generated::parse_maybe_number_map(&value, &context(), "$").unwrap();
        }
        let null = value::parse_source(br#"{"x":null}"#).unwrap();
        assert_eq!(
            generated::parse_maybe_number_map(&null, &context(), "$")
                .unwrap_err()
                .json_path,
            r#"$["x"]"#
        );
    }

    #[test]
    fn open_tuple_union_guards_compile_and_preserve_every_json_kind() {
        for bytes in [
            b"[null]".as_slice(),
            b"[false]",
            b"[18446744073709551615]",
            br#"["text"]"#,
            br#"[[]]"#,
            br#"[{"x":1,"x":2}]"#,
        ] {
            let raw = value::parse_source(bytes).unwrap();
            let parsed = generated::parse_open_tuple_union(&raw, &context(), "$").unwrap();
            let value::SourceValue::Array(values) = raw else {
                panic!()
            };
            assert_eq!(
                inspection::to_value(parsed).unwrap(),
                inspection::to_value(values).unwrap()
            );
        }
        for bytes in [b"[]".as_slice(), b"[null,null]", b"null"] {
            let raw = value::parse_source(bytes).unwrap();
            assert!(generated::parse_open_tuple_union(&raw, &context(), "$").is_err());
        }
    }
}
