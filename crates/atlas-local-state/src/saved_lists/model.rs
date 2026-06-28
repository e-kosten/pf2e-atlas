use atlas_domain::RecordKey;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SavedList {
    pub list_key: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub item_count: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SavedListItem {
    pub record_key: String,
    pub position: i64,
    pub note: Option<String>,
    pub record_title_snapshot: String,
    pub record_kind_snapshot: Option<String>,
    pub added_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SavedListWithItems {
    pub list: SavedList,
    pub items: Vec<SavedListItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HydratedSavedListItem<T> {
    pub record_key: String,
    pub position: i64,
    pub note: Option<String>,
    pub status: SavedListItemStatus,
    pub snapshot: SavedListItemSnapshot,
    pub record: Option<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedListItemStatus {
    Active,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SavedListItemSnapshot {
    pub title: String,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSavedList {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSavedList {
    pub list_key: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSavedList {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<ImportSavedListItem>,
    pub replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSavedListItem {
    pub record_key: String,
    pub note: Option<String>,
    pub record_title_snapshot: String,
    pub record_kind_snapshot: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSavedListItem {
    pub record_key: RecordKey,
    pub title_snapshot: String,
    pub kind_snapshot: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewSavedListItem {
    pub record_key: String,
    pub note: Option<String>,
    pub record_title_snapshot: String,
    pub record_kind_snapshot: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddSavedListItemOutcome {
    Added,
    AlreadyPresent,
}
