#[path = "../examples/support/model_inspection.rs"]
mod inspection;

use atlas_foundry_model::generated::StringOrNumber;
use atlas_foundry_model::{
    PredicateInput, PredicateStatement, SourceContext, SourcePresence, parse_predicate_input,
    parse_predicate_statement, parse_predicate_statements,
};
use serde_json::json;

fn context() -> SourceContext {
    SourceContext::new(
        "test:predicate",
        "fixture.json",
        "$.system.rules[0].predicate",
    )
}

#[test]
fn all_declared_operators_have_typed_owners_and_recursive_values() {
    let values = [
        r#""option""#,
        r#"{"nand":["a","b"]}"#,
        r#"{"iff":["a","b"]}"#,
        r#"{"if":"a","then":{"not":"b"}}"#,
        r#"{"and":["a",{"or":[{"not":"b"}]}]}"#,
        r#"{"or":["a","b"]}"#,
        r#"{"eq":["level",2]}"#,
        r#"{"xor":["a","b"]}"#,
        r#"{"gt":["level","other-level"]}"#,
        r#"{"gte":["level",2]}"#,
        r#"{"nor":["a","b"]}"#,
        r#"{"lt":["level",2]}"#,
        r#"{"lte":["level",2]}"#,
        r#"{"not":{"not":"a"}}"#,
    ];
    for source in values {
        parse_predicate_statement(context(), source.as_bytes()).unwrap();
    }
    let PredicateStatement::Conditional(value) =
        parse_predicate_statement(context(), values[3].as_bytes()).unwrap()
    else {
        panic!("conditional owner")
    };
    assert!(matches!(
        value.r#if,
        SourcePresence::Value(PredicateStatement::String(_))
    ));
    assert!(matches!(
        value.then,
        SourcePresence::Value(PredicateStatement::Negation(_))
    ));
    let PredicateStatement::EqualTo(value) =
        parse_predicate_statement(context(), br#"{"eq":["level",18446744073709551615]}"#).unwrap()
    else {
        panic!("comparison owner")
    };
    let SourcePresence::Value((name, StringOrNumber::Number(number))) = value.eq else {
        panic!("typed tuple")
    };
    assert_eq!(name, "level");
    assert_eq!(number.as_u64(), Some(u64::MAX));
}

#[test]
fn empty_source_values_and_additional_members_are_preserved_without_runtime_defaults() {
    assert_eq!(
        parse_predicate_statements(context(), b"[]").unwrap().len(),
        0
    );
    assert_eq!(
        parse_predicate_statement(context(), br#""""#).unwrap(),
        PredicateStatement::String(String::new())
    );
    let value =
        parse_predicate_statement(context(), br#"{"not":"a","future":1,"future":2}"#).unwrap();
    let PredicateStatement::Negation(ref negation) = value else {
        panic!("negation owner")
    };
    assert_eq!(negation.additional_fields.fields().len(), 2);
    let serialized = inspection::to_value(value).unwrap();
    assert_eq!(serialized["not"], json!({"value":"a"}));
    assert_eq!(
        serialized["additional_fields"]["fields"][0],
        json!(["future",{"Number":1}])
    );
}

#[test]
fn array_and_choiceset_constructor_inputs_keep_their_distinct_shapes() {
    assert!(matches!(
        parse_predicate_input(context(), br#"{"not":"cold"}"#).unwrap(),
        PredicateInput::PredicateStatement(_)
    ));
    assert!(matches!(
        parse_predicate_input(context(), br#"["a",{"not":"b"}]"#).unwrap(),
        PredicateInput::Array(_)
    ));
    assert!(parse_predicate_statements(context(), br#"{"not":"cold"}"#).is_err());
    assert!(parse_predicate_statement(context(), br#"["a"]"#).is_err());
    assert!(parse_predicate_input(context(), b"null").is_err());
}

#[test]
fn competing_keys_reject_even_if_one_payload_is_null_or_invalid() {
    for source in [
        br#"{"nor":["a"],"not":"a"}"#.as_slice(),
        br#"{"nor":[],"not":null}"#,
        br#"{"nor":[],"not":42}"#,
        br#"{"and":null,"or":[]}"#,
    ] {
        let error = parse_predicate_statement(context(), source).unwrap_err();
        assert_eq!(error.expected, "exactly one union alternative");
        assert!(error.actual.contains(" | "));
    }
}

#[test]
fn malformed_shapes_report_nested_context_and_tuple_positions() {
    let cases = [
        (
            r#"{"and":[{"eq":["a",false]}]}"#,
            "$.system.rules[0].predicate.and[0].eq[1]",
        ),
        (r#"{"eq":["a"]}"#, "$.system.rules[0].predicate.eq"),
        (r#"{"eq":["a",1,2]}"#, "$.system.rules[0].predicate.eq"),
        (r#"{"eq":[1,2]}"#, "$.system.rules[0].predicate.eq[0]"),
        (r#"{"not":null}"#, "$.system.rules[0].predicate.not"),
        (
            r#"{"not":"a","not":"b"}"#,
            "$.system.rules[0].predicate.not",
        ),
        (r#"{"if":"a"}"#, "$.system.rules[0].predicate"),
        (r#"{"unknown":[]}"#, "$.system.rules[0].predicate"),
    ];
    for (source, path) in cases {
        let error = parse_predicate_statement(context(), source.as_bytes()).unwrap_err();
        assert_eq!(error.json_path, path, "{source}");
        assert_eq!(error.context.record_key, "test:predicate");
    }
    assert!(parse_predicate_statement(context(), br#""a" "b""#).is_err());
}
