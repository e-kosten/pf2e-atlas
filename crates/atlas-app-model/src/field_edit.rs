use std::fmt;
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer, Serialize};

/// One proposed change and the saved value observed when editing began.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FieldEdit<T> {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub expected: T,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub value: T,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "T: Deserialize<'de>"))]
struct FieldEditFields<T> {
    #[serde(deserialize_with = "required_value")]
    expected: T,
    #[serde(deserialize_with = "required_value")]
    value: T,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for FieldEdit<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let SavedEditPayload(FieldEditFields { expected, value }) =
            SavedEditPayload::deserialize(deserializer)?;
        Ok(Self { expected, value })
    }
}

/// Restricts saved-edit JSON to named fields, excluding positional arrays.
pub struct SavedEditPayload<T>(pub T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for SavedEditPayload<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a saved edit object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                T::deserialize(MapAccessDeserializer::new(map))
            }
        }

        deserializer
            .deserialize_map(ObjectVisitor(PhantomData))
            .map(Self)
    }
}

fn required_value<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer)
}

/// The containing field may be omitted, but if present it must be an edit object.
pub(crate) fn optional_edit<'de, D, T>(deserializer: D) -> Result<Option<FieldEdit<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    FieldEdit::deserialize(deserializer).map(Some)
}

#[cfg(test)]
mod tests {
    use crate::{UpdateSavedListItemNoteRequest, UpdateSavedListRequest};
    use serde_json::{json, to_value};

    #[test]
    fn sparse_nullable_edits_distinguish_omit_set_and_clear() {
        let absent: UpdateSavedListRequest = serde_json::from_value(json!({})).unwrap();
        assert_eq!(to_value(absent).unwrap(), json!({}));

        for (expected, value) in [(None, Some("new")), (Some("old"), None)] {
            let input = json!({"description":{"expected":expected,"value":value}});
            let patch: UpdateSavedListRequest = serde_json::from_value(input.clone()).unwrap();
            assert_eq!(to_value(patch).unwrap(), input);
        }
    }

    #[test]
    fn nullable_note_edit_requires_both_keys_even_when_the_value_is_null() {
        for note in [
            json!(null),
            json!({"value":null}),
            json!({"expected":null}),
            json!(["old", null]),
        ] {
            assert!(
                serde_json::from_value::<UpdateSavedListItemNoteRequest>(json!({
                    "note":note
                }))
                .is_err(),
                "{note}"
            );
        }
        let clear = json!({
            "note":{"expected":"old","value":null}
        });
        let patch: UpdateSavedListItemNoteRequest = serde_json::from_value(clear.clone()).unwrap();
        assert_eq!(to_value(patch).unwrap(), clear);
        assert!(
            serde_json::from_value::<UpdateSavedListRequest>(json!({
                 "description":null
            }))
            .is_err()
        );
    }
}
