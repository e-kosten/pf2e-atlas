//! Incoming authored references to real-record or standalone definitions.
use crate::{DefinitionKey, QueryRecordSelection};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryDefinitionReferenceTarget {
    Record {
        selection: Box<QueryRecordSelection>,
    },
    Definition {
        key: DefinitionKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QueryDefinitionReference {
    pub target: QueryDefinitionReferenceTarget,
    pub include_contents: bool,
}
