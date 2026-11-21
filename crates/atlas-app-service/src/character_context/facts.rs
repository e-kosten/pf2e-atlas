//! Serialization of already-decoded facts; source interpretation stays in pathbuilder_sheet.
use crate::pathbuilder_sheet::{ImportedEquipment, ImportedFeat, ImportedField};
use atlas_app_model::{ImportedFactValue as Value, ImportedFactView as Fact};
use std::collections::BTreeMap;

pub(crate) trait FactSource {
    fn fact_value(&self) -> Value;
}

pub(crate) trait ToFact {
    fn to_fact(&self) -> Fact;
}

impl<T: FactSource> ToFact for ImportedField<T> {
    fn to_fact(&self) -> Fact {
        match self {
            Self::Missing => Fact::Missing,
            Self::Null => Fact::Null,
            Self::Unrecognized => Fact::Unrecognized,
            Self::Value(value) => Fact::Value(value.fact_value()),
        }
    }
}

impl FactSource for String {
    fn fact_value(&self) -> Value {
        Value::Text(self.clone())
    }
}
impl FactSource for i64 {
    fn fact_value(&self) -> Value {
        Value::Integer(*self)
    }
}
impl FactSource for bool {
    fn fact_value(&self) -> Value {
        Value::Boolean(*self)
    }
}
impl<T: ToFact> FactSource for Vec<T> {
    fn fact_value(&self) -> Value {
        Value::List(self.iter().map(ToFact::to_fact).collect())
    }
}
impl<T: ToFact> FactSource for BTreeMap<String, T> {
    fn fact_value(&self) -> Value {
        Value::Object(
            self.iter()
                .map(|(key, value)| (key.clone(), value.to_fact()))
                .collect(),
        )
    }
}
impl FactSource for (String, i64) {
    fn fact_value(&self) -> Value {
        object([
            ("name", present(self.0.fact_value())),
            ("proficiency", present(self.1.fact_value())),
        ])
    }
}

pub(crate) fn present(value: Value) -> Fact {
    Fact::Value(value)
}

pub(crate) fn object<const N: usize>(fields: [(&str, Fact); N]) -> Value {
    Value::Object(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

impl FactSource for ImportedEquipment {
    fn fact_value(&self) -> Value {
        let (name, quantity, container, investment) = match self {
            Self::InContainer(name, quantity, container, investment) => {
                (name, quantity, Some(container), investment)
            }
            Self::Uncontained(name, quantity, investment) => (name, quantity, None, investment),
        };
        object([
            ("name", present(name.fact_value())),
            ("quantity", present(quantity.fact_value())),
            (
                "container_id",
                container.map_or(Fact::Missing, |v| present(v.fact_value())),
            ),
            ("investment", present(investment.fact_value())),
        ])
    }
}

impl FactSource for ImportedFeat {
    fn fact_value(&self) -> Value {
        object([
            ("name", present(self.name.fact_value())),
            (
                "choice",
                self.choice
                    .as_ref()
                    .map_or(Fact::Null, |v| present(v.fact_value())),
            ),
            ("category", present(self.category.fact_value())),
            ("level", present(self.level.fact_value())),
            ("slot", self.slot.to_fact()),
            ("selection_kind", self.selection_kind.to_fact()),
            ("selection_detail", self.selection_detail.to_fact()),
        ])
    }
}
