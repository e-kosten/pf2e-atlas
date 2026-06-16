use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SavedList {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
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
pub struct NewSavedListItem {
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

pub fn hydrate_saved_list_item<T>(
    item: SavedListItem,
    record: Option<T>,
) -> HydratedSavedListItem<T> {
    let status = if record.is_some() {
        SavedListItemStatus::Active
    } else {
        SavedListItemStatus::Unresolved
    };
    HydratedSavedListItem {
        record_key: item.record_key,
        position: item.position,
        note: item.note,
        status,
        snapshot: SavedListItemSnapshot {
            title: item.record_title_snapshot,
            kind: item.record_kind_snapshot,
        },
        record,
    }
}
