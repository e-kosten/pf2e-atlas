//! Requested result ordering is distinct from retrieval and its matching set.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Name,
    Level,
    UnitPrice,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct SortRule {
    pub field: SortField,
    pub direction: SortDirection,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResultSort {
    Relevance {},
    SavedOrder {},
    Attributes { rules: Vec<SortRule> },
    Random { seed: u32 },
}
impl ResultSort {
    pub fn attribute(field: SortField, direction: SortDirection) -> Self {
        Self::Attributes {
            rules: vec![SortRule { field, direction }],
        }
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if let Self::Attributes { rules } = self {
            if rules.is_empty() || rules.len() > 3 {
                return Err("Choose one to three sort fields");
            }
            let mut fields = std::collections::BTreeSet::new();
            if rules.iter().any(|rule| !fields.insert(rule.field)) {
                return Err("Each sort field may appear only once");
            }
        }
        Ok(())
    }
    pub fn is_attribute(&self) -> bool {
        matches!(self, Self::Attributes { .. })
    }
    pub fn is_unit_price(&self) -> bool {
        matches!(self, Self::Attributes { rules } if rules.iter().any(|r| r.field == SortField::UnitPrice))
    }
    pub fn is_record_order(&self) -> bool {
        matches!(self, Self::Attributes { .. } | Self::Random { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn attribute_sequences_require_nonempty_unique_fields() {
        assert!(ResultSort::Attributes { rules: vec![] }.validate().is_err());
        let rule = SortRule {
            field: SortField::Name,
            direction: SortDirection::Asc,
        };
        assert!(
            ResultSort::Attributes {
                rules: vec![rule, rule]
            }
            .validate()
            .is_err()
        );
        assert!(
            ResultSort::attribute(SortField::Name, SortDirection::Asc)
                .validate()
                .is_ok()
        );
    }
    #[test]
    fn tagged_sort_contract_rejects_scalar_unknown_and_invalid_seeds() {
        for value in [
            r#""name_asc""#,
            r#"{"kind":"random"}"#,
            r#"{"kind":"random","seed":-1}"#,
            r#"{"kind":"random","seed":4294967296}"#,
            r#"{"kind":"random","seed":1.5}"#,
            r#"{"kind":"random","seed":0,"rules":[]}"#,
            r#"{"kind":"attributes","rules":[],"seed":0}"#,
            r#"{"kind":"relevance","seed":0}"#,
        ] {
            assert!(
                serde_json::from_str::<ResultSort>(value).is_err(),
                "{value}"
            );
        }
        for (seed, wire) in [
            (0, r#"{"kind":"random","seed":0}"#),
            (u32::MAX, r#"{"kind":"random","seed":4294967295}"#),
        ] {
            let value = ResultSort::Random { seed };
            assert_eq!(
                serde_json::to_value(&value).unwrap(),
                serde_json::from_str::<serde_json::Value>(wire).unwrap()
            );
            assert_eq!(serde_json::from_str::<ResultSort>(wire).unwrap(), value);
        }
    }
}
