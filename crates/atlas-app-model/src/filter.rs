//! The editor sends the shared typed predicate; CEL is an independent CLI surface.
use crate::RetrievalModeView;
use atlas_domain::{
    QueryError, QueryFieldCounts, QueryFieldDefinition, QueryLimits, QueryPredicate,
    QueryValueOptions,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FilterDiscoveryContext {
    Filtered {
        filter: Option<QueryPredicate>,
        text: Option<String>,
        mode: RetrievalModeView,
    },
    SavedList {
        list_ref: String,
        filter: Option<QueryPredicate>,
        text: Option<String>,
        mode: RetrievalModeView,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DiscoverFilterEditorRequest {
    pub context: FilterDiscoveryContext,
    #[serde(default)]
    pub selected_field_ids: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DiscoverFilterValuesRequest {
    pub context: FilterDiscoveryContext,
    pub field_id: String,
    pub clause_id: Option<String>,
    pub text: Option<String>,
    #[serde(default)]
    pub offset: usize,
    pub limit: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DiscoverFilterCountsRequest {
    pub context: FilterDiscoveryContext,
    pub field_id: String,
    pub clause_id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct FilterEditorView {
    pub catalog_version: u32,
    pub limits: QueryLimits,
    pub groups: Vec<FilterEditorGroupView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct FilterEditorGroupView {
    pub id: String,
    pub label: String,
    pub fields: Vec<FilterEditorFieldView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct FilterEditorFieldView {
    pub definition: QueryFieldDefinition,
    pub control: FilterControlView,
    pub placement: FilterFieldPlacement,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FilterControlView {
    Text,
    Numeric,
    Boolean,
    Set,
    Collection,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FilterFieldPlacement {
    InitiallyVisible,
    Addable,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct FilterValidationResult {
    pub predicate: Option<QueryPredicate>,
    pub errors: Vec<QueryError>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct FilterValueListView {
    pub values: QueryValueOptions,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct FilterCountsView {
    pub counts: QueryFieldCounts,
}
