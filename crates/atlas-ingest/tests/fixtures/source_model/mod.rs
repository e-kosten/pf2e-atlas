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
    }
}
