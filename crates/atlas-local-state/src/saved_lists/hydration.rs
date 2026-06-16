use super::model::{
    HydratedSavedListItem, SavedListItem, SavedListItemSnapshot, SavedListItemStatus,
};

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
