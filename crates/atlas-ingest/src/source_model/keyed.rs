use serde::Serialize;

/// Typed string-keyed source members in authored order. Duplicate modeled keys fail parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceMap<T> {
    pub entries: Vec<(String, T)>,
}
