//! Developer-only view of union payloads for existing source-fidelity assertions.
//! Snapshot tests independently compare exact typed variants, including tags.
use serde::Serialize;
use serde_json::Value;

pub fn to_value<T: Serialize>(model: T) -> Result<Value, serde_json::Error> {
    fn payloads(value: Value) -> Value {
        match value {
            Value::Array(values) => Value::Array(values.into_iter().map(payloads).collect()),
            Value::Object(mut fields) => {
                if fields.len() == 2
                    && fields.get("$variant").is_some_and(Value::is_string)
                    && let Some(value) = fields.remove("$value")
                {
                    return payloads(value);
                }
                Value::Object(
                    fields
                        .into_iter()
                        .map(|(key, value)| (key, payloads(value)))
                        .collect(),
                )
            }
            value => value,
        }
    }
    serde_json::to_value(model).map(payloads)
}
