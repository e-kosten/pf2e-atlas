use serde::{Deserialize, Serialize};

use crate::{AppError, RecordSummaryView};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListSummaryView {
    pub list_key: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub item_count: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListIndexView {
    pub lists: Vec<SavedListSummaryView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateSavedListRequest {
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListCreateView {
    pub list: SavedListSummaryView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct UpdateSavedListRequest {
    pub list_key: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub slug: Option<crate::FieldEdit<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub name: Option<crate::FieldEdit<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub description: Option<crate::FieldEdit<Option<String>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub tags: Option<crate::FieldEdit<Vec<String>>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListUpdateView {
    pub list: SavedListSummaryView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListDetailView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reference_error: Option<crate::AppError>,
    pub sort: atlas_domain::ResultSort,
    pub list: SavedListSummaryView,
    pub items: Vec<SavedListItemView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FilterSavedListRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<atlas_domain::ResultSort>,
    pub list_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<atlas_domain::QueryPredicate>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AddSavedListItemRequest {
    pub list_ref: String,
    pub record_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BatchAddSavedListItemsRequest {
    pub list_ref: String,
    pub items: Vec<BatchSavedListItemInput>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BatchSavedListItemInput {
    pub record_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RemoveSavedListItemRequest {
    pub list_ref: String,
    pub record_ref: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct ReorderSavedListItemRequest {
    pub list_ref: String,
    pub record_ref: String,
    pub target_record_ref: String,
    pub placement: ReorderSavedListItemPlacementView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReorderSavedListItemPlacementView {
    Before,
    After,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListItemMutationView {
    pub list_key: String,
    pub slug: String,
    pub record_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub record_name: Option<String>,
    pub outcome: SavedListItemMutationOutcomeView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BatchSavedListItemMutationView {
    pub list_key: String,
    pub slug: String,
    pub requested_count: u64,
    pub added_count: u64,
    pub already_present_count: u64,
    pub failed_count: u64,
    pub items: Vec<BatchSavedListItemResultView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BatchSavedListItemResultView {
    pub input: String,
    pub outcome: BatchSavedListItemOutcomeView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AppError>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedListItemMutationOutcomeView {
    Added,
    AlreadyPresent,
    Removed,
    NotPresent,
    Reordered,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchSavedListItemOutcomeView {
    Added,
    AlreadyPresent,
    Failed,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeleteSavedListView {
    pub list_key: String,
    pub slug: String,
    pub deleted: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListItemView {
    pub record_key: String,
    pub position: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub note: Option<String>,
    pub status: SavedListItemStatusView,
    pub snapshot: SavedListItemSnapshotView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub record: Option<RecordSummaryView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedListItemStatusView {
    Unchecked,
    Active,
    Unresolved,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListItemSnapshotView {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub kind: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListExportDocumentView {
    pub format: String,
    pub version: u64,
    pub exported_at: String,
    pub list: SavedListExportListView,
    pub items: Vec<SavedListExportItemView>,
}

impl SavedListExportDocumentView {
    pub const FORMAT: &str = "pf2e-atlas.saved-list";
    pub const VERSION: u64 = 2;

    pub fn validate_format(&self) -> Result<(), crate::AppError> {
        if self.format != Self::FORMAT {
            return Err(crate::AppError::new(
                crate::AppErrorCode::InvalidRequest,
                format!("unsupported saved-list import format `{}`", self.format),
            ));
        }
        if self.version != Self::VERSION {
            return Err(crate::AppError::new(
                crate::AppErrorCode::InvalidRequest,
                format!("unsupported saved-list import version `{}`", self.version),
            ));
        }
        Ok(())
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListExportListView {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedListExportItemView {
    pub record_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub snapshot: SavedListItemSnapshotView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImportSavedListRequest {
    pub document: SavedListExportDocumentView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub replace: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImportSavedListView {
    pub list: SavedListSummaryView,
    pub replaced: bool,
    pub resolution: ImportedListResolutionView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ImportedListResolutionView {
    Checked {
        active_count: u64,
        unresolved_count: u64,
    },
    Unchecked,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn saved_list_integer_wire_values_round_trip_as_json_numbers() {
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
            reference_error: None,
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
    }

    #[test]
    fn saved_list_export_and_mutation_counts_are_json_numbers() {
        let document = SavedListExportDocumentView {
            format: "pf2e-atlas.saved-list".into(),
            version: 2,
            exported_at: "2026-10-10".into(),
            list: SavedListExportListView {
                id: "reference".into(),
                name: "Reference".into(),
                description: None,
                tags: vec![],
            },
            items: vec![SavedListExportItemView {
                record_key: "items:retained".into(),
                note: None,
                snapshot: SavedListItemSnapshotView {
                    title: "Retained".into(),
                    kind: None,
                },
            }],
        };
        let wire = serde_json::to_value(&document).unwrap();
        assert_eq!(wire["version"], json!(2));
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSavedListItemNoteRequest {
    pub list_ref: String,
    pub record_key: String,
    pub note: crate::FieldEdit<Option<String>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedListItemNoteView {
    pub list_key: String,
    pub slug: String,
    pub record_key: String,
    pub title: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub note: Option<String>,
}
