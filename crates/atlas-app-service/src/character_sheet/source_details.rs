//! Contextual display of meaningful values the Pathbuilder decoder cannot model.
use crate::{
    AppServiceError, AppServiceResult,
    pathbuilder_sheet::{BUILD_FIELDS, FieldShape, ImportedFeat},
};
use atlas_app_model::{ImportCollectionKind, ImportFieldView, ImportScalarKind};
use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, Visitor},
};
use serde_json::value::RawValue;
use std::collections::HashSet;

struct ObjectFields<'a>(Vec<(String, &'a RawValue)>);

impl<'de> Deserialize<'de> for ObjectFields<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldsVisitor;

        impl<'de> Visitor<'de> for FieldsVisitor {
            type Value = ObjectFields<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a Pathbuilder source object")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut fields = Vec::new();
                while let Some(field) = map.next_entry()? {
                    fields.push(field);
                }
                Ok(ObjectFields(fields))
            }
        }
        deserializer.deserialize_map(FieldsVisitor)
    }
}

#[derive(Default)]
pub(super) struct SourceDetails {
    pub(super) build: Vec<ImportFieldView>,
    pub(super) metadata: Vec<ImportFieldView>,
}

pub(super) fn source_fields(source: &str) -> AppServiceResult<Vec<ImportFieldView>> {
    let SourceDetails {
        mut build,
        metadata,
    } = source_details(source)?;
    build.extend(collection(
        "Export metadata".into(),
        ImportCollectionKind::Object,
        metadata,
    ));
    Ok(build)
}

pub(super) fn source_details(source: &str) -> AppServiceResult<SourceDetails> {
    DetailsReader { first_only: false }.read(source)
}

pub(super) fn has_source_details(source: &str) -> AppServiceResult<bool> {
    let details = DetailsReader { first_only: true }.read(source)?;
    Ok(!details.build.is_empty() || !details.metadata.is_empty())
}

struct DetailsReader {
    first_only: bool,
}

const DETAIL_DEPTH: usize = 24;

impl DetailsReader {
    fn read(&self, source: &str) -> AppServiceResult<SourceDetails> {
        let project = || -> serde_json::Result<_> {
            let mut details = SourceDetails::default();
            for (label, raw) in serde_json::from_str::<ObjectFields<'_>>(source)?.0 {
                match label.as_str() {
                    "success" => {}
                    "build" => {
                        details.build =
                            self.map_children(raw.get(), 0, |key| field_shape(BUILD_FIELDS, key))?
                    }
                    _ => details
                        .metadata
                        .extend(self.property(label, raw.get(), None, 0)?),
                }
                if self.first_only && (!details.build.is_empty() || !details.metadata.is_empty()) {
                    break;
                }
            }
            Ok(details)
        };
        project().map_err(|_| {
            AppServiceError::invalid_request("Could not read the Pathbuilder import details.")
        })
    }

    fn property(
        &self,
        label: String,
        raw: &str,
        shape: Option<FieldShape>,
        depth: usize,
    ) -> serde_json::Result<Option<ImportFieldView>> {
        Ok(self.residual(label, raw, shape, depth)?.map(|mut field| {
            field.display_label = readable_key(&field.label);
            field
        }))
    }

    fn residual(
        &self,
        label: String,
        raw: &str,
        shape: Option<FieldShape>,
        depth: usize,
    ) -> serde_json::Result<Option<ImportFieldView>> {
        if raw == "null" || matches!(shape, Some(FieldShape::Hidden)) {
            return Ok(None);
        }
        if depth >= DETAIL_DEPTH {
            return Ok(contains_value(raw).then(|| {
                let text = if self.first_only {
                    String::new()
                } else {
                    raw.to_owned()
                };
                scalar(label, text, ImportScalarKind::SourceFragment)
            }));
        }
        match shape {
            Some(FieldShape::Leaf(accepts)) if accepts(raw) => Ok(None),
            Some(FieldShape::Object(fields)) if raw.starts_with('{') => {
                let entries = serde_json::from_str::<ObjectFields<'_>>(raw)?.0;
                let mut seen = HashSet::new();
                if entries
                    .iter()
                    .any(|(key, _)| field_shape(fields, key).is_some() && !seen.insert(key))
                {
                    // The typed decoder rejects objects that repeat a modeled property.
                    return self.residual(label, raw, None, depth);
                }
                let children = self.children(entries.into_iter().map(|(key, value)| {
                    let shape = field_shape(fields, &key);
                    self.property(key, value.get(), shape, depth + 1)
                }))?;
                Ok(collection(label, ImportCollectionKind::Object, children))
            }
            Some(FieldShape::Map(value)) if raw.starts_with('{') => {
                let children = self.map_children(raw, depth + 1, |_| Some(*value))?;
                Ok(collection(label, ImportCollectionKind::Object, children))
            }
            Some(FieldShape::List(item)) if raw.starts_with('[') => {
                let children = self.array_children(raw, Some(*item), depth + 1)?;
                Ok(collection(label, ImportCollectionKind::Array, children))
            }
            Some(FieldShape::Feat) if ImportedFeat::accepts(raw) => {
                let children = self.children(
                    serde_json::from_str::<Vec<&RawValue>>(raw)?
                        .into_iter()
                        .enumerate()
                        .skip(ImportedFeat::REQUIRED_FIELDS)
                        .filter(|(index, raw)| {
                            *index >= ImportedFeat::REQUIRED_FIELDS + ImportedFeat::OPTIONAL_FIELDS
                                || !raw.get().starts_with('"')
                        })
                        .map(|(index, raw)| {
                            self.residual(format!("Item {}", index + 1), raw.get(), None, depth + 1)
                        }),
                )?;
                Ok(collection(label, ImportCollectionKind::Array, children))
            }
            _ if raw.starts_with('{') => {
                let children = self.children(
                    serde_json::from_str::<ObjectFields<'_>>(raw)?
                        .0
                        .into_iter()
                        .map(|(key, raw)| self.property(key, raw.get(), None, depth + 1)),
                )?;
                Ok(collection(label, ImportCollectionKind::Object, children))
            }
            _ if raw.starts_with('[') => {
                let children = self.array_children(raw, None, depth + 1)?;
                Ok(collection(label, ImportCollectionKind::Array, children))
            }
            _ => {
                let (text, kind) = if raw.starts_with('"') {
                    (serde_json::from_str::<String>(raw)?, ImportScalarKind::Text)
                } else if matches!(raw, "true" | "false") {
                    (raw.to_owned(), ImportScalarKind::Boolean)
                } else {
                    (raw.to_owned(), ImportScalarKind::Number)
                };
                Ok((!text.trim().is_empty()).then(|| scalar(label, text, kind)))
            }
        }
    }

    fn map_children(
        &self,
        raw: &str,
        depth: usize,
        shape: impl Fn(&str) -> Option<FieldShape>,
    ) -> serde_json::Result<Vec<ImportFieldView>> {
        let entries = serde_json::from_str::<ObjectFields<'_>>(raw)?.0;
        let mut seen = HashSet::new();
        let last: Vec<_> = entries
            .iter()
            .rev()
            .map(|(key, _)| seen.insert(key))
            .collect();
        self.children(
            entries
                .into_iter()
                .zip(last.into_iter().rev())
                .map(|((key, raw), last)| {
                    // The decoder's maps retain the last value for each key.
                    let shape = if last { shape(&key) } else { None };
                    self.property(key, raw.get(), shape, depth)
                }),
        )
    }

    fn array_children(
        &self,
        raw: &str,
        shape: Option<FieldShape>,
        depth: usize,
    ) -> serde_json::Result<Vec<ImportFieldView>> {
        self.children(
            serde_json::from_str::<Vec<&RawValue>>(raw)?
                .into_iter()
                .enumerate()
                .map(|(index, raw)| {
                    let label = if self.first_only {
                        String::new()
                    } else {
                        entry_label(index, raw.get())
                    };
                    self.residual(label, raw.get(), shape, depth)
                }),
        )
    }

    fn children(
        &self,
        entries: impl Iterator<Item = serde_json::Result<Option<ImportFieldView>>>,
    ) -> serde_json::Result<Vec<ImportFieldView>> {
        let mut children = Vec::new();
        for entry in entries {
            if let Some(field) = entry? {
                children.push(field);
                if self.first_only {
                    break;
                }
            }
        }
        Ok(children)
    }
}

fn field_shape(fields: &[(&str, FieldShape)], key: &str) -> Option<FieldShape> {
    fields
        .iter()
        .find(|(label, _)| *label == key)
        .map(|(_, shape)| *shape)
}

fn entry_label(index: usize, raw: &str) -> String {
    if raw.starts_with('{')
        && let Ok(ObjectFields(fields)) = serde_json::from_str::<ObjectFields<'_>>(raw)
        && let Some(name) = ["display", "name"].into_iter().find_map(|key| {
            fields
                .iter()
                .find(|(label, _)| label == key)
                .and_then(|(_, raw)| serde_json::from_str::<String>(raw.get()).ok())
                .filter(|label| !label.trim().is_empty())
        })
    {
        return name;
    }
    if raw.starts_with('[')
        && let Ok(items) = serde_json::from_str::<Vec<&RawValue>>(raw)
        && let Some(first) = items.first()
        && let Ok(name) = serde_json::from_str::<String>(first.get())
        && !name.trim().is_empty()
    {
        return name;
    }
    format!("Item {}", index + 1)
}

fn readable_key(key: &str) -> Option<String> {
    if !key.as_bytes().first().is_some_and(u8::is_ascii_lowercase) {
        return None;
    }
    let mut label = String::new();
    let mut previous_lowercase = false;
    for (index, c) in key.chars().enumerate() {
        if index == 0 {
            label.push(c.to_ascii_uppercase());
        } else {
            if c.is_ascii_uppercase() && previous_lowercase {
                label.push(' ');
            }
            label.push(c.to_ascii_lowercase());
        }
        previous_lowercase = c.is_ascii_lowercase() || c.is_ascii_digit();
    }
    Some(label)
}

// Inspect a validated JSON fragment without recursing. Object keys, nulls, and
// empty strings do not contribute a value even beyond the displayed tree depth.
fn contains_value(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                let start = index;
                index += 1;
                while bytes[index] != b'"' {
                    index += if bytes[index] == b'\\' { 2 } else { 1 };
                }
                index += 1;
                let next = bytes[index..]
                    .iter()
                    .find(|byte| !byte.is_ascii_whitespace());
                if next != Some(&b':')
                    && serde_json::from_str::<String>(&raw[start..index])
                        .is_ok_and(|text| !text.trim().is_empty())
                {
                    return true;
                }
            }
            b'0'..=b'9' | b'-' | b't' | b'f' => return true,
            b'n' => index += "null".len(),
            _ => index += 1,
        }
    }
    false
}

fn scalar(label: String, value: String, kind: ImportScalarKind) -> ImportFieldView {
    ImportFieldView {
        label,
        display_label: None,
        scalar_kind: Some(kind),
        value: Some(value),
        collection: None,
        children: Vec::new(),
    }
}

fn collection(
    label: String,
    collection: ImportCollectionKind,
    children: Vec<ImportFieldView>,
) -> Option<ImportFieldView> {
    (!children.is_empty()).then_some(ImportFieldView {
        label,
        display_label: None,
        scalar_kind: None,
        value: None,
        collection: Some(collection),
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deeply_nested_values_remain_bounded_and_empty_branches_are_omitted() {
        let wrap = |value: &str| {
            format!(
                r#"{{"success":true,"build":{{"future":{}{value}{}}}}}"#,
                r#"{"key":"", "value":"#.repeat(200),
                "}".repeat(200),
            )
        };
        assert!(!has_source_details(&wrap("null")).unwrap());
        assert!(has_source_details(&wrap("false")).unwrap());
        let fields = source_fields(&wrap("false")).unwrap();
        let mut current = &fields[0];
        while let Some(child) = current.children.first() {
            current = child;
        }
        assert_eq!(current.scalar_kind, Some(ImportScalarKind::SourceFragment));
        assert!(current.value.as_ref().unwrap().contains("false"));
    }
}
