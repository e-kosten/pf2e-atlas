//! Structural field parsing for the independently callable Item source parsers.

use super::{
    SerializedSourceMember, SerializedSourceObject, SerializedSourceValue, SourceDiagnostic,
    SourceDiagnosticKind, SourceIdentity, SourcePresence,
};
use serde_json::Number;

pub(super) type ParseResult<T> = Result<T, SourceDiagnostic>;

pub(super) struct Fields<'a> {
    pub(super) object: &'a SerializedSourceObject,
    pub(super) identity: &'a SourceIdentity,
    pub(super) path: &'a str,
}

impl<'a> Fields<'a> {
    pub(super) fn new(
        value: &'a SerializedSourceValue,
        identity: &'a SourceIdentity,
        path: &'a str,
    ) -> ParseResult<Self> {
        Ok(Self {
            object: value
                .object()
                .ok_or_else(|| shape_error(value, identity, path, "object"))?,
            identity,
            path,
        })
    }

    pub(super) fn presence<T>(
        &self,
        key: &str,
        parse: impl FnOnce(&SerializedSourceValue, &SourceIdentity, &str) -> ParseResult<T>,
    ) -> ParseResult<SourcePresence<T>> {
        let path = format!("{}.{}", self.path, key);
        match self.object.member(key) {
            SerializedSourceMember::Missing => Ok(SourcePresence::Missing),
            SerializedSourceMember::Null => Ok(SourcePresence::Null),
            SerializedSourceMember::Value(value) => {
                parse(value, self.identity, &path).map(SourcePresence::Value)
            }
            SerializedSourceMember::Duplicate(values) => Err(malformed(
                self.identity,
                &path,
                "one source member",
                format!(
                    "duplicate members: [{}]",
                    values
                        .iter()
                        .map(|v| v.compact_json())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )),
        }
    }

    pub(super) fn required<T>(
        &self,
        key: &str,
        parse: impl FnOnce(&SerializedSourceValue, &SourceIdentity, &str) -> ParseResult<T>,
    ) -> ParseResult<T> {
        match self.presence(key, parse)? {
            SourcePresence::Value(value) => Ok(value),
            SourcePresence::Missing => Err(malformed(
                self.identity,
                &format!("{}.{}", self.path, key),
                "present non-null field",
                "missing",
            )),
            SourcePresence::Null => Err(malformed(
                self.identity,
                &format!("{}.{}", self.path, key),
                "present non-null field",
                "null",
            )),
        }
    }

    pub(super) fn rest(&self, known: &[&str]) -> SerializedSourceObject {
        SerializedSourceObject::from_fields(
            self.object
                .fields()
                .iter()
                .filter(|(key, _)| !known.contains(&key.as_str()))
                .cloned()
                .collect(),
        )
    }
}

pub(super) fn keyed<T>(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
    parse: impl Fn(&SerializedSourceValue, &SourceIdentity, &str) -> ParseResult<T>,
) -> ParseResult<Vec<(String, SourcePresence<T>)>> {
    let f = Fields::new(v, i, p)?;
    f.object
        .fields()
        .iter()
        .map(|(key, value)| {
            let path = format!("{p}[{}]", serde_json::Value::String(key.clone()));
            let value = if matches!(value, SerializedSourceValue::Null) {
                SourcePresence::Null
            } else {
                SourcePresence::Value(parse(value, i, &path)?)
            };
            Ok((key.clone(), value))
        })
        .collect()
}

pub(super) fn array<T>(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
    parse: impl Fn(&SerializedSourceValue, &SourceIdentity, &str) -> ParseResult<T>,
) -> ParseResult<Vec<T>> {
    let SerializedSourceValue::Array(values) = v else {
        return Err(shape_error(v, i, p, "array"));
    };
    values
        .iter()
        .enumerate()
        .map(|(n, v)| parse(v, i, &format!("{p}[{n}]")))
        .collect()
}

pub(super) fn strings(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<Vec<String>> {
    array(v, i, p, string)
}
pub(super) fn objects(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<Vec<SerializedSourceObject>> {
    array(v, i, p, object)
}
pub(super) fn object(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<SerializedSourceObject> {
    v.object()
        .cloned()
        .ok_or_else(|| shape_error(v, i, p, "object"))
}
pub(super) fn structured_value(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<SerializedSourceValue> {
    match v {
        SerializedSourceValue::Object(_) | SerializedSourceValue::Array(_) => Ok(v.clone()),
        _ => Err(shape_error(v, i, p, "object | array")),
    }
}

pub(super) fn string(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<String> {
    v.string()
        .map(str::to_string)
        .ok_or_else(|| shape_error(v, i, p, "string"))
}
pub(super) fn number(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
) -> ParseResult<Number> {
    match v {
        SerializedSourceValue::Number(n) => Ok(n.clone()),
        _ => Err(shape_error(v, i, p, "number")),
    }
}
pub(super) fn integer(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<i64> {
    number(v, i, p)?
        .as_i64()
        .ok_or_else(|| shape_error(v, i, p, "integer"))
}
pub(super) fn boolean(v: &SerializedSourceValue, i: &SourceIdentity, p: &str) -> ParseResult<bool> {
    match v {
        SerializedSourceValue::Boolean(b) => Ok(*b),
        _ => Err(shape_error(v, i, p, "boolean")),
    }
}
pub(super) fn shape_error(
    v: &SerializedSourceValue,
    i: &SourceIdentity,
    p: &str,
    expected: &str,
) -> SourceDiagnostic {
    malformed(i, p, expected, v.compact_json())
}
pub(super) fn malformed(
    i: &SourceIdentity,
    p: &str,
    expected: &str,
    actual: impl Into<String>,
) -> SourceDiagnostic {
    SourceDiagnostic::new(SourceDiagnosticKind::MalformedShape, i, p, expected, actual)
}
