//! Tests execute the selected parser library's interpreter, so its limitations
//! remain visible rather than inferred from a separate SQL evaluator.
use cel::{Context, Program, Value};
use serde_json::json;

fn evaluate(expression: &str, variables: serde_json::Value) -> Result<Value, cel::ExecutionError> {
    let mut context = Context::default();
    for (key, value) in variables.as_object().unwrap() {
        context.add_variable_from_value(key, cel::to_value(value).unwrap());
    }
    Program::compile(expression).unwrap().execute(&context)
}

#[test]
fn cel_error_truth_table_accepts_only_true_like_sql_unknown() {
    // Unavailable selected values are omitted from the CEL environment; their
    // five-state metadata is separately known in availability. Exposing Null
    // as a literal instead would make unavailable != 80 evaluate true.
    for left in ["true", "false", "missing >= 80"] {
        for right in ["true", "false", "missing >= 80"] {
            for operator in ["&&", "||"] {
                let expression = format!("({left}) {operator} ({right})");
                let result = evaluate(&expression, json!({}));
                let expected = match operator {
                    "&&" => {
                        if left == "false" || right == "false" {
                            Some(false)
                        } else if left == "true" && right == "true" {
                            Some(true)
                        } else {
                            None
                        }
                    }
                    _ => {
                        if left == "true" || right == "true" {
                            Some(true)
                        } else if left == "false" && right == "false" {
                            Some(false)
                        } else {
                            None
                        }
                    }
                };
                match expected {
                    Some(value) => assert_eq!(result, Ok(Value::Bool(value)), "{expression}"),
                    None => assert!(result.is_err(), "{expression}"),
                }
            }
        }
    }
    assert!(evaluate("!(missing >= 80)", json!({})).is_err());
    for state in ["missing", "null", "invalid", "not_applicable"] {
        assert_eq!(
            evaluate(
                "availability['actor.hp.maximum'] == 'value'",
                json!({"availability":{"actor.hp.maximum":state}})
            ),
            Ok(Value::Bool(false))
        );
        assert!(
            evaluate(
                "actor.hp.maximum >= 80",
                json!({"actor":{"hp":{}},"availability":{"actor.hp.maximum":state}})
            )
            .is_err()
        );
    }
}

#[test]
fn cel_interpreter_exists_error_is_order_dependent() {
    let expression = "actor.items.exists(i, i.spell.rank >= 3)";
    let true_first = json!({"actor":{"items":[{"spell":{"rank":3}},{"spell":{}}]}});
    let unavailable_first = json!({"actor":{"items":[{"spell":{}},{"spell":{"rank":3}}]}});
    assert_eq!(evaluate(expression, true_first), Ok(Value::Bool(true)));
    // cel0.14.5 propagates the first loop-step error immediately. A conforming
    // existential/Atlas SQL result must accept both orders because true exists.
    assert!(evaluate(expression, unavailable_first).is_err());
    assert_eq!(
        evaluate(expression, json!({"actor":{"items":[]}})),
        Ok(Value::Bool(false))
    );
    assert!(evaluate(expression, json!({"actor":{}})).is_err());
    assert!(
        evaluate(
            expression,
            json!({"actor":{"items":[{"spell":{}},{"spell":{"rank":1}}]}})
        )
        .is_err()
    );
}
