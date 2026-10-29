use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AppError, RecordSummaryView};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListSummaryView {
    pub list_key: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    pub tags: Vec<String>,
    #[ts(type = "number")]
    pub item_count: u64,
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
pub struct CreateSavedListRequest {
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListCreateView {
    pub list: SavedListSummaryView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct UpdateSavedListRequest {
    pub list_key: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListUpdateView {
    pub list: SavedListSummaryView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListDetailView {
    pub sort: atlas_domain::ResultSort,
    pub list: SavedListSummaryView,
    pub items: Vec<SavedListItemView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct FilterSavedListRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub sort: Option<atlas_domain::ResultSort>,
    pub list_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub filter: Option<atlas_domain::QueryPredicate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AddSavedListItemRequest {
    pub list_ref: String,
    pub record_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct BatchAddSavedListItemsRequest {
    pub list_ref: String,
    pub items: Vec<BatchSavedListItemInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct BatchSavedListItemInput {
    pub record_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RemoveSavedListItemRequest {
    pub list_ref: String,
    pub record_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct ReorderSavedListItemRequest {
    pub list_ref: String,
    pub record_ref: String,
    pub target_record_ref: String,
    pub placement: ReorderSavedListItemPlacementView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum ReorderSavedListItemPlacementView {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListItemMutationView {
    pub list_key: String,
    pub slug: String,
    pub record_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record_name: Option<String>,
    pub outcome: SavedListItemMutationOutcomeView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct BatchSavedListItemMutationView {
    pub list_key: String,
    pub slug: String,
    #[ts(type = "number")]
    pub requested_count: u64,
    #[ts(type = "number")]
    pub added_count: u64,
    #[ts(type = "number")]
    pub already_present_count: u64,
    #[ts(type = "number")]
    pub failed_count: u64,
    pub items: Vec<BatchSavedListItemResultView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct BatchSavedListItemResultView {
    pub input: String,
    pub outcome: BatchSavedListItemOutcomeView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SavedListItemMutationOutcomeView {
    Added,
    AlreadyPresent,
    Removed,
    NotPresent,
    Reordered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum BatchSavedListItemOutcomeView {
    Added,
    AlreadyPresent,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct DeleteSavedListView {
    pub list_key: String,
    pub slug: String,
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListItemView {
    pub record_key: String,
    #[ts(type = "number")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListExportDocumentView {
    pub format: String,
    #[ts(type = "number")]
    pub version: u64,
    pub exported_at: String,
    pub list: SavedListExportListView,
    pub items: Vec<SavedListExportItemView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListExportListView {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SavedListExportItemView {
    #[ts(type = "number")]
    pub position: i64,
    pub record_key: String,
    pub record_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kind: Option<String>,
    pub status: SavedListItemStatusView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub snapshot: SavedListItemSnapshotView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ImportSavedListRequest {
    pub document: SavedListExportDocumentView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<String>,
    pub replace: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ImportSavedListView {
    pub list: SavedListSummaryView,
    pub replaced: bool,
    #[ts(type = "number")]
    pub active_count: u64,
    #[ts(type = "number")]
    pub unresolved_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn saved_list_integer_wire_values_match_typescript_numbers() {
        let summary = SavedListSummaryView {
            list_key: "list-key".into(),
            slug: "reference".into(),
            name: "Reference".into(),
            description: None,
            tags: vec![],
            item_count: 2,
            created_at: "2026-10-10".into(),
            updated_at: "2026-10-10".into(),
        };
        let item = SavedListItemView {
            record_key: "items:known-record".into(),
            position: 7,
            note: None,
            status: SavedListItemStatusView::Unresolved,
            snapshot: SavedListItemSnapshotView {
                title: "Retained title".into(),
                kind: None,
            },
            record: None,
        };
        let encoded = serde_json::to_string(&SavedListDetailView {
            sort: atlas_domain::ResultSort::SavedOrder {},
            list: summary.clone(),
            items: vec![item.clone()],
        })
        .unwrap();
        let wire: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(wire["list"]["item_count"], json!(2));
        assert_eq!(wire["items"][0]["position"], json!(7));
        assert!(wire["list"]["item_count"].is_number());
        assert!(wire["items"][0]["position"].is_number());
        let decoded: SavedListDetailView = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.list, summary);
        assert_eq!(decoded.items, [item]);
        for (declaration, fields) in [
            (SavedListSummaryView::decl(), vec!["item_count"]),
            (SavedListItemView::decl(), vec!["position"]),
            (
                BatchSavedListItemMutationView::decl(),
                vec![
                    "requested_count",
                    "added_count",
                    "already_present_count",
                    "failed_count",
                ],
            ),
            (
                ImportSavedListView::decl(),
                vec!["active_count", "unresolved_count"],
            ),
            (SavedListExportDocumentView::decl(), vec!["version"]),
            (SavedListExportItemView::decl(), vec!["position"]),
        ] {
            for field in fields {
                assert!(
                    declaration.contains(&format!("{field}: number")),
                    "JSON-number field disagrees with TypeScript: {declaration}"
                );
            }
            assert!(
                !declaration.contains("bigint"),
                "List JSON has no bigint decoder: {declaration}"
            );
        }
    }

    #[test]
    fn saved_list_export_and_mutation_counts_are_json_numbers() {
        let document = SavedListExportDocumentView {
            format: "pf2e-atlas-list".into(),
            version: 1,
            exported_at: "2026-10-10".into(),
            list: SavedListExportListView {
                id: "reference".into(),
                name: "Reference".into(),
                description: None,
                tags: vec![],
            },
            items: vec![SavedListExportItemView {
                position: 3,
                record_key: "items:retained".into(),
                record_name: "Retained".into(),
                kind: None,
                status: SavedListItemStatusView::Unresolved,
                note: None,
                snapshot: SavedListItemSnapshotView {
                    title: "Retained".into(),
                    kind: None,
                },
            }],
        };
        let wire = serde_json::to_value(&document).unwrap();
        assert_eq!(wire["version"], json!(1));
        assert_eq!(wire["items"][0]["position"], json!(3));
        assert_eq!(
            serde_json::from_value::<SavedListExportDocumentView>(wire).unwrap(),
            document
        );
        let mutation = BatchSavedListItemMutationView {
            list_key: "list-key".into(),
            slug: "reference".into(),
            requested_count: 2,
            added_count: 1,
            already_present_count: 0,
            failed_count: 1,
            items: vec![],
        };
        let wire = serde_json::to_value(mutation).unwrap();
        assert_eq!(
            wire,
            json!({
                "list_key":"list-key", "slug":"reference", "requested_count":2,
                "added_count":1, "already_present_count":0, "failed_count":1, "items":[],
            })
        );
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct UpdateSavedListItemNoteRequest {
    pub list_ref: String,
    pub record_key: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SavedListItemNoteView {
    pub list_key: String,
    pub slug: String,
    pub record_key: String,
    pub title: String,
    pub note: Option<String>,
}
