use super::*;
use serde::Serialize;
pub(super) fn fact<T>(v: SourceFieldView<'_, T>) -> FactView<T> {
    let state = field_state(v.availability());
    FactView {
        state,
        value: match v {
            SourceFieldView::Value(v) => Some(v),
            _ => None,
        },
    }
}
pub(super) fn text(v: SourceFieldView<'_, &String>) -> FactView<String> {
    fact(v.map(Clone::clone))
}
pub(super) fn number(v: SourceFieldView<'_, &serde_json::Number>) -> NumberFactView {
    let f = fact(v.map(Clone::clone));
    NumberFactView {
        state: f.state,
        value: f.value,
        adjustment: None,
    }
}
pub(super) fn unavailable<T>() -> FactView<T> {
    FactView {
        state: QueryFieldState::NotApplicable,
        value: None,
    }
}
// Enum serialization owns the upstream identifier spelling. Tagged singleton
// unions carry exactly one identifier in `$value`; no authored object is queried.
pub(super) fn token(v: &impl Serialize) -> Option<String> {
    let v = serde_json::to_value(v).ok()?;
    v.as_str()
        .or_else(|| v.get("$value").and_then(serde_json::Value::as_str))
        .map(str::to_owned)
}
fn identifier_failure<T>() -> SourceFieldView<'static, T> {
    SourceFieldView::ProjectionInvalid {
        source_path: "$",
        reason: "selected identifier cannot be represented as an identifier string",
    }
}
pub(super) fn identifier<T: Serialize>(v: SourceFieldView<'_, &T>) -> FactView<String> {
    fact(v.and_then(|v| {
        token(v)
            .map(SourceFieldView::Value)
            .unwrap_or_else(identifier_failure)
    }))
}
pub(super) fn identifiers<T: Serialize>(v: SourceFieldView<'_, &Vec<T>>) -> FactView<Vec<String>> {
    fact(v.and_then(|v| {
        v.iter()
            .map(token)
            .collect::<Option<Vec<_>>>()
            .map(SourceFieldView::Value)
            .unwrap_or_else(identifier_failure)
    }))
}
