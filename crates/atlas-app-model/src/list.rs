use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::RecordSummaryView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListSummaryView {
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListIndexView {
    pub lists: Vec<SavedListSummaryView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListDetailView {
    pub list: SavedListSummaryView,
    pub items: Vec<SavedListItemView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListItemView {
    pub record_key: String,
    pub position: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub status: SavedListItemStatusView,
    pub snapshot: SavedListItemSnapshotView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record: Option<RecordSummaryView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SavedListItemStatusView {
    Active,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListItemSnapshotView {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kind: Option<String>,
}
