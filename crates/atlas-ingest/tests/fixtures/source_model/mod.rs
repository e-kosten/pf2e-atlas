//! Compile the generic regression graph against the actual source parsing primitives.
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
pub mod generated;

#[cfg(test)]
mod tests {
    use super::{generated, parse, value};
    use serde_json::json;
    fn context() -> parse::SourceContext {
        parse::SourceContext::new("fixture", "fixture.json", "$")
    }
    #[test]
    fn mixed_inline_cycles_have_compilable_owners_and_finite_parsing() {
        let value =
            value::parse_source(br#"{"next":{"expression":"a"},"expression":{"next":null}}"#)
                .unwrap();
        let parsed = generated::parse_node(&value, &context(), "$").unwrap();
        assert_eq!(
            serde_json::to_value(parsed).unwrap()["next"]["value"]["expression"],
            json!({"value":"a"})
        );
        generated::parse_expr(&value, &context(), "$").unwrap();
    }
    #[test]
    fn empty_and_single_tuples_serialize_as_arrays() {
        let empty = value::parse_source(b"[]").unwrap();
        let parsed = generated::parse_empty(&empty, &context(), "$").unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), json!([]));
        let single = value::parse_source(br#"["a"]"#).unwrap();
        assert_eq!(
            serde_json::to_value(generated::parse_single(&single, &context(), "$").unwrap())
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
        assert!(generated::parse_loose(&other, &context(), "$").is_err());
        let empty = value::parse_source(b"{}").unwrap();
        assert!(generated::parse_strict(&empty, &context(), "$").is_err());
        generated::parse_loose(&empty, &context(), "$").unwrap();
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
                serde_json::to_value(parsed).unwrap(),
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
        assert_eq!(serde_json::to_value(parsed).unwrap(), json!([[["leaf"]]]));
        let parsed = generated::parse_tuple_entry(&value, &context(), "$").unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), json!([[["leaf"]]]));
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
            let actual = serde_json::to_value(parsed).unwrap();
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
                serde_json::to_value(parsed).unwrap()["pair"]["value"],
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
    fn open_union_arms_keep_kind_constraints_and_report_overlapping_objects() {
        for bytes in [br#""string""#.as_slice(), b"[]", b"{}"] {
            let value = value::parse_source(bytes).unwrap();
            generated::parse_object_union(&value, &context(), "$").unwrap();
        }
        let boolean = value::parse_source(b"true").unwrap();
        assert!(generated::parse_object_union(&boolean, &context(), "$").is_err());
        let overlapping = value::parse_source(br#"{"label":"x"}"#).unwrap();
        assert!(generated::parse_overlapping_open(&overlapping, &context(), "$").is_err());
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
                serde_json::to_value(parsed).unwrap(),
                serde_json::to_value(values).unwrap()
            );
        }
        for bytes in [b"[]".as_slice(), b"[null,null]", b"null"] {
            let raw = value::parse_source(bytes).unwrap();
            assert!(generated::parse_open_tuple_union(&raw, &context(), "$").is_err());
        }
    }
}
