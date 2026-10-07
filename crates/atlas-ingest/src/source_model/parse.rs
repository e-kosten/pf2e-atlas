use std::fmt;

use super::presence::SourcePresence;
use super::value::{SourceObject, SourceValue};
use serde::Serialize;
use serde_json::Number;

pub(super) type ParseResult<T> = Result<T, SourceDiagnostic>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceContext {
    pub record_key: String,
    pub source_path: String,
    pub json_path: String,
}

impl SourceContext {
    pub fn new(
        record_key: impl Into<String>,
        source_path: impl Into<String>,
        json_path: impl Into<String>,
    ) -> Self {
        Self {
            record_key: record_key.into(),
            source_path: source_path.into(),
            json_path: json_path.into(),
        }
    }
    pub(super) fn error(
        &self,
        path: &str,
        expected: &str,
        actual: &SourceValue,
    ) -> SourceDiagnostic {
        self.message(path, expected, format!("{actual:?}"))
    }
    pub(super) fn message(
        &self,
        path: &str,
        expected: &str,
        actual: impl Into<String>,
    ) -> SourceDiagnostic {
        SourceDiagnostic {
            context: Box::new(self.clone()),
            json_path: path.to_string(),
            expected: expected.to_string(),
            actual: actual.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceDiagnostic {
    pub context: Box<SourceContext>,
    pub json_path: String,
    pub expected: String,
    pub actual: String,
}
impl fmt::Display for SourceDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} in {} at {}: expected {}, found {}",
            self.context.record_key,
            self.context.source_path,
            self.json_path,
            self.expected,
            self.actual
        )
    }
}
impl std::error::Error for SourceDiagnostic {}

pub(super) struct Fields<'a> {
    object: &'a SourceObject,
    context: &'a SourceContext,
    path: &'a str,
}
impl<'a> Fields<'a> {
    pub(super) fn new(
        value: &'a SourceValue,
        context: &'a SourceContext,
        path: &'a str,
    ) -> ParseResult<Self> {
        if let SourceValue::Object(object) = value {
            Ok(Self {
                object,
                context,
                path,
            })
        } else {
            Err(context.error(path, "object", value))
        }
    }
    pub(super) fn presence<T>(
        &self,
        name: &str,
        parse: impl FnOnce(&SourceValue, &SourceContext, &str) -> ParseResult<T>,
    ) -> ParseResult<SourcePresence<T>> {
        let path = format!("{}.{}", self.path, name);
        let values: Vec<_> = self
            .object
            .fields
            .iter()
            .filter_map(|(key, value)| (key == name).then_some(value))
            .collect();
        match values.as_slice() {
            [] => Ok(SourcePresence::Missing),
            [SourceValue::Null] => Ok(SourcePresence::Null),
            [value] => parse(value, self.context, &path).map(SourcePresence::Value),
            _ => Err(self
                .context
                .message(&path, "one structural member", "duplicate members")),
        }
    }
    pub(super) fn required<T>(
        &self,
        name: &str,
        parse: impl FnOnce(&SourceValue, &SourceContext, &str) -> ParseResult<T>,
    ) -> ParseResult<T> {
        match self.presence(name, parse)? {
            SourcePresence::Value(value) => Ok(value),
            SourcePresence::Missing => Err(self.context.message(
                &format!("{}.{}", self.path, name),
                "present non-null field",
                "missing",
            )),
            SourcePresence::Null => Err(self.context.message(
                &format!("{}.{}", self.path, name),
                "present non-null field",
                "null",
            )),
        }
    }
    pub(super) fn remaining(&self, known: &[&str]) -> SourceObject {
        SourceObject {
            fields: self
                .object
                .fields
                .iter()
                .filter(|(key, _)| !known.contains(&key.as_str()))
                .cloned()
                .collect(),
        }
    }
}
pub(super) fn string(
    value: &SourceValue,
    context: &SourceContext,
    path: &str,
) -> ParseResult<String> {
    if let SourceValue::String(value) = value {
        Ok(value.clone())
    } else {
        Err(context.error(path, "string", value))
    }
}
pub(super) fn number(
    value: &SourceValue,
    context: &SourceContext,
    path: &str,
) -> ParseResult<Number> {
    if let SourceValue::Number(value) = value {
        Ok(value.clone())
    } else {
        Err(context.error(path, "number", value))
    }
}
pub(super) fn boolean(
    value: &SourceValue,
    context: &SourceContext,
    path: &str,
) -> ParseResult<bool> {
    if let SourceValue::Boolean(value) = value {
        Ok(*value)
    } else {
        Err(context.error(path, "boolean", value))
    }
}

pub(super) fn array<T>(
    value: &SourceValue,
    context: &SourceContext,
    path: &str,
    parse: impl Fn(&SourceValue, &SourceContext, &str) -> ParseResult<T>,
) -> ParseResult<Vec<T>> {
    let SourceValue::Array(values) = value else {
        return Err(context.error(path, "array", value));
    };
    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse(value, context, &format!("{path}[{index}]")))
        .collect()
}

pub(super) fn keyed<T>(
    value: &SourceValue,
    context: &SourceContext,
    path: &str,
    parse: impl Fn(&SourceValue, &SourceContext, &str) -> ParseResult<T>,
) -> ParseResult<super::SourceMap<T>> {
    let SourceValue::Object(object) = value else {
        return Err(context.error(path, "string-keyed object", value));
    };
    let mut keys = std::collections::HashSet::new();
    let mut entries = Vec::new();
    for (key, value) in &object.fields {
        let key_path = serde_json::to_string(key)
            .map_err(|error| context.message(path, "serializable source key", error.to_string()))?;
        let entry_path = format!("{path}[{key_path}]");
        if !keys.insert(key) {
            return Err(context.message(&entry_path, "one keyed member", "duplicate members"));
        }
        entries.push((key.clone(), parse(value, context, &entry_path)?));
    }
    Ok(super::SourceMap { entries })
}

pub(super) fn tuple<'a>(
    value: &'a SourceValue,
    context: &SourceContext,
    path: &str,
    length: usize,
) -> ParseResult<&'a [SourceValue]> {
    let SourceValue::Array(values) = value else {
        return Err(context.error(path, "tuple array", value));
    };
    if values.len() != length {
        return Err(context.message(
            path,
            &format!("tuple of length {length}"),
            format!("length {}", values.len()),
        ));
    }
    Ok(values)
}
