use serde::{Deserialize, Serialize};

/// Typed string-keyed source members in authored order. Duplicate modeled keys fail parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceMap<T> {
    pub entries: Vec<(String, T)>,
}
