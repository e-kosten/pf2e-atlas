//! Product summaries keep source availability and numeric domains without hydrating bodies.
use crate::{RecordKey, RecordKind, query::QueryFieldState};
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceQueryFact<T> {
    pub state: QueryFieldState,
    pub value: Option<T>,
}
impl<T> SourceQueryFact<T> {
    pub fn known(value: T) -> Self {
        Self {
            state: QueryFieldState::Value,
            value: Some(value),
        }
    }
    pub fn unavailable(state: QueryFieldState) -> Option<Self> {
        (state != QueryFieldState::Value).then_some(Self { state, value: None })
    }
    pub fn as_value(&self) -> Option<&T> {
        if self.state == QueryFieldState::Value {
            self.value.as_ref()
        } else {
            None
        }
    }
    pub fn is_valid(&self) -> bool {
        (self.state == QueryFieldState::Value) == self.value.is_some()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLevelBasis {
    ActorLevel,
    ItemLevel,
    SpellRank,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRecordSummary {
    pub key: RecordKey,
    pub pack_label: String,
    pub document_kind: String,
    pub source_type: SourceQueryFact<String>,
    pub name: SourceQueryFact<String>,
    pub record_kind: SourceQueryFact<RecordKind>,
    pub level: SourceQueryFact<Number>,
    pub level_basis: Option<SourceLevelBasis>,
    pub rarity: SourceQueryFact<String>,
    pub traits: SourceQueryFact<Vec<String>>,
    pub size: SourceQueryFact<String>,
    pub publication_title: SourceQueryFact<String>,
    pub publication_remaster: SourceQueryFact<bool>,
    pub source_path: String,
    pub content_hash: String,
}
